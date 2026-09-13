//! Small ownership adapters for libdispatch. Callbacks must never block or unwind.
use block2::RcBlock;
use dispatch2::{DispatchObject, DispatchQueue, DispatchRetained, DispatchSource, DispatchTime};
use std::{fs::File, sync::Arc, time::Duration};

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
            DispatchTime::NOW.time(d.as_nanos().min(i64::MAX as u128) as i64)
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
            if unsafe { libc::sigaction(sig, &action, &mut old) } != 0 {
                return Err(std::io::Error::last_os_error());
            }
            result.previous.push((sig, old));
            let cb = callback.clone();
            result.sources.push(Source::new(
                SourceKind::Signal,
                sig as usize,
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
    if let Some(runloop) = objc2_core_foundation::CFRunLoop::main() {
        runloop.stop();
    }
}

/// An inert source keeps an empty-config run loop asleep without a polling timer.
pub struct RunLoop {
    source: objc2_core_foundation::CFRetained<objc2_core_foundation::CFRunLoopSource>,
}
impl RunLoop {
    pub fn new() -> Result<Self, String> {
        use objc2_core_foundation::*;
        let mut context: CFRunLoopSourceContext = unsafe { std::mem::zeroed() };
        let source = unsafe { CFRunLoopSource::new(None, 0, &mut context) }
            .ok_or("cannot create run loop source")?;
        CFRunLoop::main()
            .ok_or("no main run loop")?
            .add_source(Some(&source), unsafe { kCFRunLoopCommonModes });
        Ok(Self { source })
    }
    pub fn run(&self) {
        objc2_core_foundation::CFRunLoop::run();
    }
}
impl Drop for RunLoop {
    fn drop(&mut self) {
        self.source.invalidate();
    }
}
