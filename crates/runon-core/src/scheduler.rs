use crate::config::Config;
use std::{collections::VecDeque, sync::Arc, time::Duration};

#[derive(Debug)]
struct Pending {
    actions: VecDeque<usize>,
    ready_at: Duration,
    received_at: Duration,
    order: u64,
}

#[derive(Default, Debug)]
struct Slot {
    pending: Option<Pending>,
    running: Option<Pending>,
    free_at: Duration,
}

#[derive(Clone, Copy, Debug)]
pub struct Start {
    pub group: usize,
    pub action: usize,
    pub received_at: Duration,
}

pub struct Scheduler {
    config: Arc<Config>,
    slots: Vec<Slot>,
    order: u64,
    stopped: bool,
}

impl Scheduler {
    pub fn new(config: Arc<Config>) -> Self {
        Self {
            slots: (0..config.groups.len()).map(|_| Slot::default()).collect(),
            config,
            order: 0,
            stopped: false,
        }
    }

    pub fn enqueue(&mut self, group: usize, actions: Vec<usize>, received_at: Duration) {
        if self.stopped || actions.is_empty() {
            return;
        }
        self.order += 1;
        let slot = &mut self.slots[group];
        let order = slot.pending.as_ref().map_or(self.order, |p| p.order);
        let debounce = self.config.groups[group].debounce;
        // Replacing an already-ready packet without debounce retains its queue
        // position. With debounce, the new event makes it unready again.
        let ready_at = if debounce.is_zero() {
            slot.pending.as_ref().map_or(received_at, |p| p.ready_at)
        } else {
            received_at.saturating_add(debounce)
        };
        slot.pending = Some(Pending {
            actions: actions.into(),
            ready_at,
            received_at,
            order,
        });
    }

    pub fn poll(&mut self, now: Duration) -> Vec<Start> {
        if self.stopped {
            return Vec::new();
        }
        let running = self.slots.iter().filter(|s| s.running.is_some()).count();
        // ponytail: scan configured groups; index deadlines if very large configs need it.
        let mut ready: Vec<_> = self
            .slots
            .iter()
            .enumerate()
            .filter_map(|(g, s)| {
                s.pending
                    .as_ref()
                    .filter(|p| s.running.is_none() && p.ready_at <= now)
                    .map(|p| (p.ready_at.max(s.free_at), p.order, g))
            })
            .collect();
        ready.sort_unstable();
        ready
            .into_iter()
            .take(self.config.max_parallel.saturating_sub(running))
            .map(|(_, _, group)| {
                let slot = &mut self.slots[group];
                let mut batch = slot.pending.take().unwrap();
                let action = batch.actions.pop_front().unwrap();
                let start = Start {
                    group,
                    action,
                    received_at: batch.received_at,
                };
                slot.running = Some(batch);
                start
            })
            .collect()
    }

    pub fn finished(&mut self, group: usize, now: Duration) -> Option<Start> {
        let slot = &mut self.slots[group];
        if !self.stopped
            && let Some(batch) = &mut slot.running
            && let Some(action) = batch.actions.pop_front()
        {
            return Some(Start {
                group,
                action,
                received_at: batch.received_at,
            });
        }
        slot.running = None;
        slot.free_at = now;
        None
    }

    pub fn deadline(&self, now: Duration) -> Option<Duration> {
        if self.stopped
            || self.slots.iter().filter(|s| s.running.is_some()).count() >= self.config.max_parallel
        {
            return None;
        }
        self.slots
            .iter()
            .filter(|s| s.running.is_none())
            .filter_map(|s| s.pending.as_ref())
            .map(|p| p.ready_at.max(now))
            .min()
    }

    pub fn stop(&mut self) {
        self.stopped = true;
        for slot in &mut self.slots {
            slot.pending = None;
        }
    }

    pub fn pending_count(&self) -> usize {
        self.slots.iter().filter(|s| s.pending.is_some()).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn time(ms: u64) -> Duration {
        Duration::from_millis(ms)
    }
    fn scheduler() -> Scheduler {
        Scheduler::new(Arc::new(Config::parse("max-parallel 1\ngroup g { debounce \"10ms\"; }\naction a group=g { on system.wake; exec p; }\naction b group=g { on system.wake; exec p; }\naction c { on system.wake; exec p; }").unwrap()))
    }

    #[test]
    fn latest_debounce_batches_and_fairness() {
        let mut s = scheduler();
        s.enqueue(0, vec![0], time(0));
        s.enqueue(0, vec![0, 1], time(5));
        assert!(s.poll(time(14)).is_empty());
        assert_eq!(s.poll(time(15))[0].action, 0);
        s.enqueue(1, vec![2], time(16));
        for t in 17..10_000 {
            s.enqueue(0, vec![1], time(t));
        }
        assert_eq!(s.pending_count(), 2);
        assert!(s.poll(time(20_000)).is_empty());
        assert_eq!(s.finished(0, time(20_000)).unwrap().action, 1);
        assert!(s.finished(0, time(20_000)).is_none());
        assert_eq!(s.poll(time(20_000))[0].action, 2);
        assert!(s.finished(1, time(20_000)).is_none());
        assert_eq!(s.poll(time(20_000))[0].action, 1);
    }

    #[test]
    fn fifo_uses_readiness_and_retains_zero_debounce_position() {
        let c = Arc::new(Config::parse("max-parallel 1\ngroup slow { debounce \"100ms\"; }\naction slow group=slow { on system.wake; exec p; }\naction fast { on system.wake; exec p; }\naction blocker { on system.wake; exec p; }").unwrap());
        let mut s = Scheduler::new(c);
        s.enqueue(2, vec![2], time(0));
        assert_eq!(s.poll(time(0))[0].group, 2);
        s.enqueue(0, vec![0], time(1));
        s.enqueue(1, vec![1], time(10));
        s.enqueue(1, vec![1], time(150));
        s.finished(2, time(200));
        // Fast was ready at 10ms, slow at 101ms. Replacement kept fast's place.
        assert_eq!(s.poll(time(200))[0].group, 1);
        s.enqueue(1, vec![1], time(201));
        s.finished(1, time(202));
        // A just-freed group goes behind an already-ready group.
        assert_eq!(s.poll(time(202))[0].group, 0);
    }

    #[test]
    fn independent_groups_and_shutdown() {
        let c = Arc::new(
            Config::parse(
                "action a { on system.wake; exec p; }\naction b { on system.wake; exec p; }",
            )
            .unwrap(),
        );
        let mut s = Scheduler::new(c);
        s.enqueue(0, vec![0], time(0));
        s.enqueue(1, vec![1], time(0));
        assert_eq!(s.poll(time(0)).len(), 2);
        s.enqueue(0, vec![0], time(1));
        s.stop();
        assert!(s.finished(0, time(2)).is_none());
        assert!(s.poll(time(2)).is_empty());
        assert_eq!(s.pending_count(), 0);
    }
}
