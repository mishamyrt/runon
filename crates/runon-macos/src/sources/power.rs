use crate::native::Source;
use block2::RcBlock;
use dispatch2::DispatchQueue;
use std::{
    ffi::{c_char, c_int},
    sync::Arc,
};

unsafe extern "C" {
    fn notify_register_dispatch(
        name: *const c_char,
        token: *mut c_int,
        queue: &DispatchQueue,
        handler: &block2::DynBlock<dyn Fn(c_int)>,
    ) -> u32;
    fn notify_cancel(token: c_int) -> u32;
}

pub(super) struct Listener(c_int);
impl Drop for Listener {
    fn drop(&mut self) {
        unsafe {
            notify_cancel(self.0);
        }
    }
}

impl Listener {
    pub(super) fn new(refresh: Arc<Source>) -> Result<Self, String> {
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
        Ok(Self(token))
    }
}

pub(super) fn snapshot() -> Result<String, String> {
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
