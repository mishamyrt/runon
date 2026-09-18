use super::{Emit, Observer, Snapshots};
use objc2_foundation::{NSDistributedNotificationCenter, NSString};
use runon_core::event::{Event, Kind};
use std::{
    collections::BTreeSet,
    sync::{Arc, Mutex},
};

pub(super) fn subscribe(
    kinds: &BTreeSet<Kind>,
    snapshots: &Arc<Mutex<Snapshots>>,
    emit: &Emit,
) -> Vec<Observer> {
    let mut observers = Vec::new();
    for (kind, name, locked) in [
        (Kind::ScreenLocked, "com.apple.screenIsLocked", true),
        (Kind::ScreenUnlocked, "com.apple.screenIsUnlocked", false),
    ] {
        // Subscribe to both sides to deduplicate even when only one side is requested.
        if !kinds.contains(&Kind::ScreenLocked) && !kinds.contains(&Kind::ScreenUnlocked) {
            break;
        }
        let state = snapshots.clone();
        let out = emit.clone();
        observers.push(Observer::new(
            &NSDistributedNotificationCenter::defaultCenter(),
            &NSString::from_str(name),
            move |_| {
                let mut state = state.lock().unwrap();
                if state.locked != Some(locked) {
                    state.locked = Some(locked);
                    drop(state);
                    out(Event::new(kind));
                }
            },
        ));
    }
    observers
}
