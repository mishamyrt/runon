//! Small ownership adapters for libdispatch. Callbacks must never block or unwind.
use block2::RcBlock;
use dispatch2::{DispatchObject, DispatchQueue, DispatchRetained, DispatchSource, DispatchTime};
use objc2::{MainThreadMarker, rc::Retained};
use objc2_app_kit::{
    NSApplication, NSApplicationActivationPolicy, NSEvent, NSEventModifierFlags, NSEventType,
};
use objc2_foundation::NSPoint;
use std::{fs::File, sync::Arc, time::Duration};

#[derive(Clone, Copy)]
pub enum SourceKind {
    Data,
    Timer,
    Process,
    Read,
    Signal,
}

pub struct Source(pub DispatchRetained<DispatchSource>);

impl Source {
    pub fn new(
        kind: SourceKind,
        handle: usize,
        queue: &DispatchQueue,
        callback: impl Fn() + Send + 'static,
    ) -> Self {
        // SAFETY: handles and masks correspond to the selected dispatch source type.
        // libdispatch copies the block and executes it on the retained target queue.
        unsafe {
            let (ty, mask) = match kind {
                SourceKind::Data => (&raw const dispatch2::_dispatch_source_type_data_add, 0),
                SourceKind::Timer => (&raw const dispatch2::_dispatch_source_type_timer, 0),
                SourceKind::Process => (
                    &raw const dispatch2::_dispatch_source_type_proc,
                    0x8000_0000,
                ),
                SourceKind::Read => (&raw const dispatch2::_dispatch_source_type_read, 0),
                SourceKind::Signal => (&raw const dispatch2::_dispatch_source_type_signal, 0),
            };
            let source = DispatchSource::new(ty.cast_mut(), handle, mask, Some(queue));
            let block = RcBlock::new(callback);
            source.set_event_handler_with_block(RcBlock::as_ptr(&block));
            source.activate();
            Self(source)
        }
    }

    pub fn hold_file(&self, file: Arc<File>) {
        // Keep the descriptor alive until the kernel has detached the source.
        let block = RcBlock::new(move || {
            let _ = &file;
        });
        unsafe {
            self.0
                .set_cancel_handler_with_block(RcBlock::as_ptr(&block));
        }
    }

    pub fn wake(&self) {
        self.0.merge_data(1);
    }

    pub fn arm(&self, delay: Option<Duration>) {
        let start = delay.map_or(DispatchTime::FOREVER, |d| {
            DispatchTime::NOW.time(i64::try_from(d.as_nanos()).unwrap_or(i64::MAX))
        });
        self.0.set_timer(start, u64::MAX, 1_000_000);
    }
}

impl Drop for Source {
    fn drop(&mut self) {
        self.0.cancel();
    }
}

extern "C" fn signal_handler(_: libc::c_int) {}

pub struct Signals {
    sources: Vec<Source>,
    previous: Vec<(i32, libc::sigaction)>,
}

impl Signals {
    #[allow(clippy::needless_pass_by_value)] // The subscriptions retain shared ownership of the callback.
    pub fn new(
        queue: &DispatchQueue,
        callback: Arc<dyn Fn() + Send + Sync>,
    ) -> std::io::Result<Self> {
        let mut result = Self {
            sources: Vec::new(),
            previous: Vec::new(),
        };
        for sig in [libc::SIGINT, libc::SIGTERM] {
            // A caught handler (not SIG_IGN) is reset to default by exec in children.
            let mut action: libc::sigaction = unsafe { std::mem::zeroed() };
            let mut old = action;
            action.sa_sigaction = signal_handler as *const () as usize;
            action.sa_flags = libc::SA_RESTART;
            if unsafe { libc::sigaction(sig, &raw const action, &raw mut old) } != 0 {
                return Err(std::io::Error::last_os_error());
            }
            result.previous.push((sig, old));
            let cb = callback.clone();
            result.sources.push(Source::new(
                SourceKind::Signal,
                usize::try_from(sig).unwrap(),
                queue,
                move || cb(),
            ));
        }
        Ok(result)
    }
}

impl Drop for Signals {
    fn drop(&mut self) {
        self.sources.clear();
        for (sig, previous) in &self.previous {
            unsafe {
                libc::sigaction(*sig, previous, std::ptr::null_mut());
            }
        }
    }
}

pub fn stop_main() {
    // Runtime shutdown can arrive on a worker queue. AppKit must stay on main.
    DispatchQueue::main().exec_async(|| {
        objc2::rc::autoreleasepool(|_| {
            let app = NSApplication::sharedApplication(MainThreadMarker::new().unwrap());
            app.stop(None);
            // stop: only takes effect after an NSEvent is dispatched; signals
            // and dispatch callbacks alone do not wake the AppKit event wait.
            let event = NSEvent::otherEventWithType_location_modifierFlags_timestamp_windowNumber_context_subtype_data1_data2(
                NSEventType::ApplicationDefined,
                NSPoint::new(0.0, 0.0),
                NSEventModifierFlags::empty(),
                0.0,
                0,
                None,
                0,
                0,
                0,
            ).expect("valid application-defined event");
            app.postEvent_atStart(&event, true);
        });
    });
}

/// `AppKit` must dispatch `WindowServer` events to update `NSScreen` and deliver
/// `NSApplicationDidChangeScreenParametersNotification`. `CFRunLoop` alone cannot.
pub struct RunLoop {
    app: Retained<NSApplication>,
}
impl RunLoop {
    pub fn new() -> Result<Self, String> {
        let mtm = MainThreadMarker::new().ok_or("event loop must start on the main thread")?;
        let app = NSApplication::sharedApplication(mtm);
        app.setActivationPolicy(NSApplicationActivationPolicy::Prohibited);
        Ok(Self { app })
    }
    pub fn run(&self) {
        self.app.run();
    }
}
