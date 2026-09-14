//! Native subscriptions. `AppKit` access stays on the main thread; snapshots hold
//! owned Rust values, never borrowed Objective-C pointers.
use crate::native::{Source, SourceKind};
use block2::RcBlock;
use dispatch2::DispatchQueue;
use objc2::{MainThreadMarker, Message, rc::Retained, runtime::ProtocolObject};
use objc2_app_kit::{
    NSApplication, NSApplicationActivationPolicy,
    NSApplicationDidChangeScreenParametersNotification, NSRunningApplication, NSScreen,
    NSWorkspace, NSWorkspaceApplicationKey, NSWorkspaceDidActivateApplicationNotification,
    NSWorkspaceDidDeactivateApplicationNotification, NSWorkspaceDidLaunchApplicationNotification,
    NSWorkspaceDidTerminateApplicationNotification, NSWorkspaceDidWakeNotification,
};
use objc2_core_audio::{
    AudioObjectAddPropertyListenerBlock, AudioObjectGetPropertyData,
    AudioObjectGetPropertyDataSize, AudioObjectPropertyAddress,
    AudioObjectRemovePropertyListenerBlock, kAudioDevicePropertyDeviceUID,
    kAudioHardwarePropertyDevices, kAudioObjectPropertyElementMain, kAudioObjectPropertyName,
    kAudioObjectPropertyScopeGlobal, kAudioObjectSystemObject,
};
use objc2_core_foundation::{CFRetained, CFString};
use objc2_foundation::{
    NSDistributedNotificationCenter, NSNotification, NSNotificationCenter, NSNumber,
    NSObjectProtocol, NSOperationQueue, NSString,
};
use runon_core::event::{Event, Kind, Value, changes};
use std::{
    collections::{BTreeMap, BTreeSet},
    ffi::{c_char, c_int},
    ptr::NonNull,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
};

type Emit = Arc<dyn Fn(Event) + Send + Sync>;
type AudioBlock = RcBlock<dyn Fn(u32, NonNull<AudioObjectPropertyAddress>)>;

unsafe extern "C" {
    fn notify_register_dispatch(
        name: *const c_char,
        token: *mut c_int,
        queue: &DispatchQueue,
        handler: &block2::DynBlock<dyn Fn(c_int)>,
    ) -> u32;
    fn notify_cancel(token: c_int) -> u32;
}

struct Observer {
    center: Retained<NSNotificationCenter>,
    token: Retained<ProtocolObject<dyn NSObjectProtocol>>,
}
impl Observer {
    fn new(
        center: &NSNotificationCenter,
        name: &NSString,
        cb: impl Fn(&NSNotification) + Send + 'static,
    ) -> Self {
        let block = RcBlock::new(move |n: NonNull<NSNotification>| {
            objc2::rc::autoreleasepool(|_| cb(unsafe { n.as_ref() }));
        });
        let token = unsafe {
            center.addObserverForName_object_queue_usingBlock(
                Some(name),
                None,
                Some(&NSOperationQueue::mainQueue()),
                &block,
            )
        };
        Self {
            center: center.retain(),
            token,
        }
    }
}
impl Drop for Observer {
    fn drop(&mut self) {
        unsafe {
            self.center.removeObserver((*self.token).as_ref());
        }
    }
}

struct AudioListener {
    block: AudioBlock,
}
impl Drop for AudioListener {
    fn drop(&mut self) {
        let mut address = address(kAudioHardwarePropertyDevices);
        let code = unsafe {
            AudioObjectRemovePropertyListenerBlock(
                kAudioObjectSystemObject as u32,
                NonNull::from(&mut address),
                Some(DispatchQueue::main()),
                RcBlock::as_ptr(&self.block),
            )
        };
        if code != 0 {
            log::error!("removing audio listener: OSStatus {code}");
        }
    }
}

struct PowerListener(c_int);
impl Drop for PowerListener {
    fn drop(&mut self) {
        unsafe {
            notify_cancel(self.0);
        }
    }
}

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
    audio_listener: Option<AudioListener>,
    power_listener: Option<PowerListener>,
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
        for (kind, name) in unsafe {
            [
                (
                    Kind::AppActivated,
                    NSWorkspaceDidActivateApplicationNotification,
                ),
                (
                    Kind::AppDeactivated,
                    NSWorkspaceDidDeactivateApplicationNotification,
                ),
                (
                    Kind::AppLaunched,
                    NSWorkspaceDidLaunchApplicationNotification,
                ),
                (
                    Kind::AppTerminated,
                    NSWorkspaceDidTerminateApplicationNotification,
                ),
            ]
        } {
            if !kinds.contains(&kind) {
                continue;
            }
            let emit = emit.clone();
            result
                .observers
                .push(Observer::new(&workspace, name, move |n| {
                    let Some(info) = n.userInfo() else {
                        return;
                    };
                    let Some(obj) = info.objectForKey(unsafe { NSWorkspaceApplicationKey }) else {
                        return;
                    };
                    let Some(app) = obj.downcast_ref::<NSRunningApplication>() else {
                        return;
                    };
                    let mut event = Event::new(kind);
                    if let Some(id) = app.bundleIdentifier() {
                        event = event.text("bundle-id", id.to_string());
                    }
                    if let Some(name) = app.localizedName() {
                        event = event.text("name", name.to_string());
                    }
                    emit(event);
                }));
        }
        if screen {
            let refresh = refreshers[0].clone().unwrap();
            result.observers.push(Observer::new(
                &NSNotificationCenter::defaultCenter(),
                unsafe { NSApplicationDidChangeScreenParametersNotification },
                move |_| {
                    refresh.wake();
                },
            ));
        }
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
            result.observers.push(Observer::new(
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
        if audio {
            let refresh = refreshers[1].clone().unwrap();
            let block: AudioBlock =
                RcBlock::new(move |_: u32, _: NonNull<AudioObjectPropertyAddress>| {
                    refresh.wake();
                });
            let mut address = address(kAudioHardwarePropertyDevices);
            let code = unsafe {
                AudioObjectAddPropertyListenerBlock(
                    kAudioObjectSystemObject as u32,
                    NonNull::from(&mut address),
                    Some(DispatchQueue::main()),
                    RcBlock::as_ptr(&block),
                )
            };
            if code != 0 {
                return Err(format!("subscribing to audio devices: OSStatus {code}"));
            }
            result.audio_listener = Some(AudioListener { block });
        }
        if power {
            let refresh = refreshers[2].clone().unwrap();
            let block = RcBlock::new(move |_: c_int| {
                refresh.wake();
            });
            let mut token = 0;
            let code = unsafe {
                notify_register_dispatch(
                    c"com.apple.system.powersources.source".as_ptr(),
                    &raw mut token,
                    DispatchQueue::main(),
                    &block,
                )
            };
            if code != 0 {
                return Err(format!("subscribing to power source: notify status {code}"));
            }
            result.power_listener = Some(PowerListener(token));
        }
        if screen || audio || power || kinds.contains(&Kind::Wake) {
            let out = emit.clone();
            result.observers.push(Observer::new(
                &workspace,
                unsafe { NSWorkspaceDidWakeNotification },
                move |_| {
                    out(Event::new(Kind::Wake));
                    for refresh in refreshers.iter().flatten() {
                        refresh.wake();
                    }
                },
            ));
        }
        // Subscribe before taking the baseline: callbacks are queued on the main
        // run loop and cannot interleave with this initial snapshot.
        let mut state = snapshots.lock().unwrap();
        if screen {
            state.screens = Some(screens()?);
        }
        if audio {
            state.audio = Some(audio_devices()?);
        }
        if power {
            state.power = Some(power_source()?);
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
        match screens() {
            Ok(new) => {
                if let Some(old) = state.screens.replace(new.clone()) {
                    events.extend(changes(&old, &new, Kind::ScreenDisconnected));
                }
            }
            Err(e) => log::error!("refreshing screens: {e}"),
        }
    }
    if audio {
        match audio_devices() {
            Ok(new) => {
                if let Some(old) = state.audio.replace(new.clone()) {
                    events.extend(changes(&old, &new, Kind::AudioDisconnected));
                }
            }
            Err(e) => log::error!("refreshing audio devices: {e}"),
        }
    }
    if power {
        match power_source() {
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

fn screens() -> Result<BTreeMap<u32, Event>, String> {
    let mtm = MainThreadMarker::new().ok_or("screen query outside main thread")?;
    let mut result = BTreeMap::new();
    for screen in &NSScreen::screens(mtm) {
        let description = screen.deviceDescription();
        let id = description
            .objectForKey(&NSString::from_str("NSScreenNumber"))
            .and_then(|n| {
                n.downcast_ref::<NSNumber>()
                    .map(objc2_foundation::NSNumber::unsignedIntValue)
            })
            .ok_or("display has no numeric ID")?;
        let mut event =
            Event::new(Kind::ScreenConnected).text("name", screen.localizedName().to_string());
        event.fields.insert("id".into(), Value::Id(id));
        result.insert(id, event);
    }
    Ok(result)
}

fn address(selector: u32) -> AudioObjectPropertyAddress {
    AudioObjectPropertyAddress {
        mSelector: selector,
        mScope: kAudioObjectPropertyScopeGlobal,
        mElement: kAudioObjectPropertyElementMain,
    }
}

fn audio_devices() -> Result<BTreeMap<String, Event>, String> {
    let mut addr = address(kAudioHardwarePropertyDevices);
    // Device lists can change between the size query and data query; bounded retry.
    for _ in 0..3 {
        let mut size = 0u32;
        let code = unsafe {
            AudioObjectGetPropertyDataSize(
                kAudioObjectSystemObject as u32,
                NonNull::from(&mut addr),
                0,
                std::ptr::null(),
                NonNull::from(&mut size),
            )
        };
        if code != 0 {
            return Err(format!("device list size: OSStatus {code}"));
        }
        if size == 0 {
            return Ok(BTreeMap::new());
        }
        if !size.is_multiple_of(4) {
            return Err("invalid audio device list size".into());
        }
        let mut ids = vec![0u32; size as usize / 4];
        let code = unsafe {
            AudioObjectGetPropertyData(
                kAudioObjectSystemObject as u32,
                NonNull::from(&mut addr),
                0,
                std::ptr::null(),
                NonNull::from(&mut size),
                NonNull::new(ids.as_mut_ptr().cast()).unwrap(),
            )
        };
        if code != 0 {
            continue;
        }
        ids.truncate(size as usize / 4);
        let mut result = BTreeMap::new();
        let mut complete = true;
        for id in ids {
            match (
                audio_string(id, kAudioDevicePropertyDeviceUID),
                audio_string(id, kAudioObjectPropertyName),
            ) {
                (Ok(uid), Ok(name)) => {
                    result.insert(
                        uid.clone(),
                        Event::new(Kind::AudioConnected)
                            .text("uid", uid)
                            .text("name", name),
                    );
                }
                _ => {
                    complete = false;
                    break;
                }
            }
        }
        if complete {
            return Ok(result);
        }
    }
    Err("audio devices changed during enumeration; keeping previous snapshot".into())
}

fn audio_string(id: u32, selector: u32) -> Result<String, String> {
    let mut addr = address(selector);
    let mut value: *mut CFString = std::ptr::null_mut();
    let mut size = u32::try_from(std::mem::size_of_val(&value)).unwrap();
    let code = unsafe {
        AudioObjectGetPropertyData(
            id,
            NonNull::from(&mut addr),
            0,
            std::ptr::null(),
            NonNull::from(&mut size),
            NonNull::from(&mut value).cast(),
        )
    };
    if code != 0 {
        return Err(format!("device {id} property {selector}: OSStatus {code}"));
    }
    let ptr = NonNull::new(value).ok_or("audio property is null")?;
    // CoreAudio transfers ownership of CF-valued properties to the caller.
    Ok(unsafe { CFRetained::from_raw(ptr) }.to_string())
}

fn power_source() -> Result<String, String> {
    let snapshot = objc2_io_kit::IOPSCopyPowerSourcesInfo().ok_or("no power-source snapshot")?;
    let source = unsafe { objc2_io_kit::IOPSGetProvidingPowerSourceType(Some(&snapshot)) }
        .ok_or("no active power source")?;
    match source.to_string().as_str() {
        "AC Power" => Ok("ac".into()),
        "Battery Power" => Ok("battery".into()),
        "UPS Power" => Ok("ups".into()),
        other => Err(format!("unknown power source '{other}'")),
    }
}
