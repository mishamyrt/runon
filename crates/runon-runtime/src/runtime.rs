use crate::{matching::Matcher, process::Process};
use dispatch2::{DispatchQueue, DispatchRetained};
use runon_config::Config;
use runon_core::{
    event::Event,
    scheduler::{Scheduler, Start},
};
use runon_macos::{
    native::{Source, SourceKind},
    paths,
};
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

#[derive(Clone)]
pub struct Runtime {
    matcher: Arc<Matcher>,
    scheduler: Arc<Mutex<Scheduler>>,
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
    scheduler: Arc<Mutex<Scheduler>>,
    active: Vec<Option<Active>>,
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
            let (report, reports) = {
                let mut state = state.lock().unwrap();
                (state.report.clone(), state.drive())
            };
            for event in reports {
                report(event);
            }
        }
    }
}

impl Runtime {
    /// Reports run on the scheduler queue, outside its state locks. The callback
    /// must not block or panic; hand blocking I/O off to a separate worker.
    pub fn new(
        config: Config,
        report: impl Fn(Report) + Send + Sync + 'static,
    ) -> Result<Self, String> {
        let home = paths::home()?;
        let config = Arc::new(config);
        let clock = Instant::now();
        let scheduler = Arc::new(Mutex::new(Scheduler::new(
            config.max_parallel,
            config.groups.iter().map(|group| group.debounce).collect(),
        )));
        let state = Arc::new_cyclic(|weak| {
            Mutex::new(State {
                active: (0..config.groups.len()).map(|_| None).collect(),
                config: config.clone(),
                scheduler: scheduler.clone(),
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
            matcher: Arc::new(Matcher::new(config)),
            scheduler,
            wake,
            state,
            clock,
        })
    }

    #[allow(clippy::needless_pass_by_value)] // Consuming event sink for native callbacks.
    pub fn submit(&self, event: Event) {
        let at = self.clock.elapsed();
        let batches = self.matcher.batches(&event);
        if batches.is_empty() {
            return;
        }
        let mut scheduler = self.scheduler.lock().unwrap();
        for (group, actions) in batches {
            scheduler.enqueue(group, actions, at);
        }
        drop(scheduler);
        // Coalesced native wakeup, never one queued closure per incoming event.
        self.wake.wake();
    }

    pub fn shutdown(&self) {
        self.scheduler.lock().unwrap().stop();
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

    fn drive(&mut self) -> Vec<Report> {
        let mut reports = Vec::new();
        let now = self.clock.elapsed();
        let (stop, starts) = {
            let mut scheduler = self.scheduler.lock().unwrap();
            (scheduler.is_stopped(), scheduler.poll(now))
        };
        if stop && !self.stopped {
            self.stopped = true;
            for active in self.active.iter_mut().flatten() {
                active.process.terminate(now, "daemon stopped");
            }
        }
        for start in starts {
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
                    reports.push(Report::Started {
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
                reports.push(Report::Finished {
                    name: action.name.clone(),
                    success: result.success,
                    reason: result.reason,
                    stdout: result.stdout,
                    stderr: result.stderr,
                });
                self.active[group] = None;
                let next = self
                    .scheduler
                    .lock()
                    .unwrap()
                    .finished(group, self.clock.elapsed());
                if let Some(next) = next {
                    self.start(next);
                }
            }
        }
        // A finished group may release capacity for another group. Schedule one
        // native turn; do not recurse through an arbitrarily large config.
        let now = self.clock.elapsed();
        let deadline = self
            .scheduler
            .lock()
            .unwrap()
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
            reports.push(Report::Stopped);
        }
        reports
    }
}
