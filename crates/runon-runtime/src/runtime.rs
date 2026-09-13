use crate::process::Process;
use dispatch2::{DispatchQueue, DispatchRetained};
use runon_core::{
    config::{self, Config},
    event::Event,
    scheduler::{Scheduler, Start},
};
use runon_macos::native::{Source, SourceKind};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex, Weak},
    time::{Duration, Instant},
};

#[derive(Debug)]
pub enum Report {
    Started {
        name: String,
        latency: Duration,
    },
    Finished {
        name: String,
        success: bool,
        reason: String,
        stdout: String,
        stderr: String,
    },
    Stopped,
}

struct Envelope {
    actions: Vec<usize>,
    at: Duration,
    order: u64,
}
struct Inbox {
    slots: Vec<Option<Envelope>>,
    order: u64,
    stop: bool,
}

#[derive(Clone)]
pub struct Runtime {
    config: Arc<Config>,
    inbox: Arc<Mutex<Inbox>>,
    wake: Arc<Source>,
    state: Arc<Mutex<State>>,
    clock: Instant,
}

struct Active {
    start: Start,
    process: Process,
    reported_start: bool,
}
struct State {
    config: Arc<Config>,
    scheduler: Scheduler,
    active: Vec<Option<Active>>,
    inbox: Arc<Mutex<Inbox>>,
    queue: DispatchRetained<DispatchQueue>,
    weak: Weak<Mutex<State>>,
    timer: Option<Source>,
    clock: Instant,
    home: PathBuf,
    stopped: bool,
    notified_stop: bool,
    report: Arc<dyn Fn(Report) + Send + Sync>,
}

fn callback(weak: Weak<Mutex<State>>) -> impl Fn() + Send + Sync + 'static {
    move || {
        if let Some(state) = weak.upgrade() {
            state.lock().unwrap().drive();
        }
    }
}

impl Runtime {
    pub fn new(
        config: Config,
        report: impl Fn(Report) + Send + Sync + 'static,
    ) -> Result<Self, String> {
        let home = config::home()?;
        let config = Arc::new(config);
        let clock = Instant::now();
        let inbox = Arc::new(Mutex::new(Inbox {
            slots: (0..config.groups.len()).map(|_| None).collect(),
            order: 0,
            stop: false,
        }));
        let state = Arc::new_cyclic(|weak| {
            Mutex::new(State {
                active: (0..config.groups.len()).map(|_| None).collect(),
                config: config.clone(),
                scheduler: Scheduler::new(config.clone()),
                inbox: inbox.clone(),
                queue: DispatchQueue::new("co.myrt.runon.scheduler", None),
                weak: weak.clone(),
                timer: None,
                clock,
                home,
                stopped: false,
                notified_stop: false,
                report: Arc::new(report),
            })
        });
        let mut s = state.lock().unwrap();
        let wake = Arc::new(Source::new(
            SourceKind::Data,
            0,
            &s.queue,
            callback(Arc::downgrade(&state)),
        ));
        s.timer = Some(Source::new(
            SourceKind::Timer,
            0,
            &s.queue,
            callback(Arc::downgrade(&state)),
        ));
        s.timer.as_ref().unwrap().arm(None);
        drop(s);
        Ok(Self {
            config,
            inbox,
            wake,
            state,
            clock,
        })
    }

    pub fn submit(&self, event: Event) {
        let at = self.clock.elapsed();
        let batches = self.config.batches(&event);
        if batches.is_empty() {
            return;
        }
        let mut inbox = self.inbox.lock().unwrap();
        if inbox.stop {
            return;
        }
        inbox.order += 1;
        let order = inbox.order;
        for (group, actions) in batches {
            inbox.slots[group] = Some(Envelope { actions, at, order });
        }
        drop(inbox);
        // Coalesced native wakeup, never one queued closure per incoming event.
        self.wake.wake();
    }

    pub fn shutdown(&self) {
        self.inbox.lock().unwrap().stop = true;
        self.wake.wake();
    }

    pub fn queue(&self) -> DispatchRetained<DispatchQueue> {
        self.state.lock().unwrap().queue.clone()
    }
}

impl State {
    fn start(&mut self, start: Start) {
        let timeout = self.config.actions[start.action].timeout;
        self.active[start.group] = Some(Active {
            start,
            process: Process::new(timeout, self.clock.elapsed()),
            reported_start: false,
        });
    }

    fn drive(&mut self) {
        let now = self.clock.elapsed();
        let mut inbox = self.inbox.lock().unwrap();
        let stop = inbox.stop;
        let mut messages: Vec<_> = inbox
            .slots
            .iter_mut()
            .enumerate()
            .filter_map(|(g, s)| s.take().map(|e| (g, e)))
            .collect();
        drop(inbox);
        messages.sort_unstable_by_key(|(_, e)| e.order);
        for (group, e) in messages {
            self.scheduler.enqueue(group, e.actions, e.at);
        }
        if stop && !self.stopped {
            self.stopped = true;
            self.scheduler.stop();
            for active in self.active.iter_mut().flatten() {
                active.process.terminate(now, "daemon stopped");
            }
        }
        for start in self.scheduler.poll(now) {
            self.start(start);
        }
        let wake: Arc<dyn Fn() + Send + Sync> = Arc::new(callback(self.weak.clone()));
        // Native callbacks are serialized on this queue. The mutex only bridges
        // public handles; it is never held while waiting for a child or timer.
        for group in 0..self.active.len() {
            while let Some(active) = &mut self.active[group] {
                let action = &self.config.actions[active.start.action];
                let result = active.process.poll(
                    action,
                    self.clock.elapsed(),
                    &self.home,
                    &self.queue,
                    wake.clone(),
                );
                if !active.reported_start
                    && let Some(at) = active.process.started_at()
                {
                    (self.report)(Report::Started {
                        name: action.name.clone(),
                        latency: at
                            .duration_since(self.clock)
                            .saturating_sub(active.start.received_at),
                    });
                    active.reported_start = true;
                }
                let Some(result) = result else {
                    break;
                };
                (self.report)(Report::Finished {
                    name: action.name.clone(),
                    success: result.success,
                    reason: result.reason,
                    stdout: result.stdout,
                    stderr: result.stderr,
                });
                self.active[group] = None;
                if let Some(next) = self.scheduler.finished(group, self.clock.elapsed()) {
                    self.start(next);
                }
            }
        }
        // A finished group may release capacity for another group. Schedule one
        // native turn; do not recurse through an arbitrarily large config.
        let now = self.clock.elapsed();
        let deadline = self
            .scheduler
            .deadline(now)
            .into_iter()
            .chain(
                self.active
                    .iter()
                    .flatten()
                    .filter_map(|a| a.process.deadline()),
            )
            .min();
        self.timer
            .as_ref()
            .unwrap()
            .arm(deadline.map(|d| d.saturating_sub(now)));
        if self.stopped && !self.notified_stop && self.active.iter().all(Option::is_none) {
            self.notified_stop = true;
            self.timer.as_ref().unwrap().arm(None);
            (self.report)(Report::Stopped);
        }
    }
}
