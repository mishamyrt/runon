//! Native subscriptions. `AppKit` access stays on the main thread; snapshots hold
//! owned Rust values, never borrowed Objective-C pointers.
mod app;
mod audio;
mod observer;
mod power;
mod screen;
mod screen_lock;
mod wake;

use crate::native::{Source, SourceKind};
use dispatch2::DispatchQueue;
use objc2::MainThreadMarker;
use objc2_app_kit::{NSApplication, NSApplicationActivationPolicy, NSWorkspace};
use observer::Observer;
use runon_core::event::{Event, Kind, changes};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
};

type Emit = Arc<dyn Fn(Event) + Send + Sync>;

#[derive(Default)]
struct Snapshots {
    screens: Option<BTreeMap<u32, Event>>,
    audio: Option<BTreeMap<String, Event>>,
    power: Option<String>,
    locked: Option<bool>,
}

pub struct Sources {
    alive: Arc<AtomicBool>,
    observers: Vec<Observer>,
    audio_listener: Option<audio::Listener>,
    power_listener: Option<power::Listener>,
}

impl Sources {
    #[allow(clippy::needless_pass_by_value)] // The subscription owns its interest filter.
    pub fn subscribe(kinds: BTreeSet<Kind>, emit: Emit) -> Result<Self, String> {
        let mtm = MainThreadMarker::new().ok_or("event sources must start on the main thread")?;
        let alive = Arc::new(AtomicBool::new(true));
        let gate = alive.clone();
        let interested = kinds.clone();
        let emit: Emit = Arc::new(move |event| {
            if gate.load(Ordering::Relaxed) && interested.contains(&event.kind) {
                emit(event);
            }
        });
        let screen =
            kinds.contains(&Kind::ScreenConnected) || kinds.contains(&Kind::ScreenDisconnected);
        let audio =
            kinds.contains(&Kind::AudioConnected) || kinds.contains(&Kind::AudioDisconnected);
        let power = kinds.contains(&Kind::PowerChanged);
        let snapshots = Arc::new(Mutex::new(Snapshots::default()));
        let mut result = Self {
            alive,
            observers: Vec::new(),
            audio_listener: None,
            power_listener: None,
        };
        if kinds.is_empty() {
            return Ok(result);
        }
        let app = NSApplication::sharedApplication(mtm);
        app.setActivationPolicy(NSApplicationActivationPolicy::Prohibited);
        let workspace = NSWorkspace::sharedWorkspace().notificationCenter();
        // Notifications only schedule work. Coalesced sources keep repeated
        // notifications from growing the queue; AppKit snapshots stay on main.
        let refreshers: Vec<_> = [screen, audio, power]
            .into_iter()
            .enumerate()
            .map(|(index, enabled)| {
                enabled.then(|| {
                    let state = snapshots.clone();
                    let out = emit.clone();
                    Arc::new(Source::new(
                        SourceKind::Data,
                        0,
                        DispatchQueue::main(),
                        move || {
                            objc2::rc::autoreleasepool(|_| {
                                refresh(&state, &out, index == 0, index == 1, index == 2);
                            });
                        },
                    ))
                })
            })
            .collect();
        result
            .observers
            .extend(app::subscribe(&kinds, &workspace, &emit));
        if screen {
            result
                .observers
                .push(screen::subscribe(refreshers[0].clone().unwrap()));
        }
        result
            .observers
            .extend(screen_lock::subscribe(&kinds, &snapshots, &emit));
        if audio {
            result.audio_listener = Some(audio::Listener::new(refreshers[1].clone().unwrap())?);
        }
        if power {
            result.power_listener = Some(power::Listener::new(refreshers[2].clone().unwrap())?);
        }
        if screen || audio || power || kinds.contains(&Kind::Wake) {
            result
                .observers
                .push(wake::subscribe(&workspace, emit.clone(), refreshers));
        }
        // Subscribe before taking the baseline: callbacks are queued on the main
        // run loop and cannot interleave with this initial snapshot.
        let mut state = snapshots.lock().unwrap();
        if screen {
            state.screens = Some(screen::snapshot()?);
        }
        if audio {
            state.audio = Some(audio::snapshot()?);
        }
        if power {
            state.power = Some(power::snapshot()?);
        }
        Ok(result)
    }
}
impl Drop for Sources {
    fn drop(&mut self) {
        self.alive.store(false, Ordering::Relaxed);
    }
}

fn refresh(state: &Mutex<Snapshots>, emit: &Emit, screen: bool, audio: bool, power: bool) {
    let mut state = state.lock().unwrap();
    let mut events = Vec::new();
    if screen {
        match screen::snapshot() {
            Ok(new) => {
                if let Some(old) = state.screens.replace(new.clone()) {
                    events.extend(changes(&old, &new, Kind::ScreenDisconnected));
                }
            }
            Err(e) => log::error!("refreshing screens: {e}"),
        }
    }
    if audio {
        match audio::snapshot() {
            Ok(new) => {
                if let Some(old) = state.audio.replace(new.clone()) {
                    events.extend(changes(&old, &new, Kind::AudioDisconnected));
                }
            }
            Err(e) => log::error!("refreshing audio devices: {e}"),
        }
    }
    if power {
        match power::snapshot() {
            Ok(new) => {
                if state.power.as_ref().is_some_and(|old| old != &new) {
                    events.push(Event::new(Kind::PowerChanged).text("source", &new));
                }
                state.power = Some(new);
            }
            Err(e) => log::error!("refreshing power source: {e}"),
        }
    }
    drop(state);
    for event in events {
        emit(event);
    }
}
