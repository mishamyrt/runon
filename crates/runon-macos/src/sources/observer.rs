use block2::RcBlock;
use objc2::{Message, rc::Retained, runtime::ProtocolObject};
use objc2_foundation::{
    NSNotification, NSNotificationCenter, NSObjectProtocol, NSOperationQueue, NSString,
};
use std::ptr::NonNull;

pub(super) struct Observer {
    center: Retained<NSNotificationCenter>,
    token: Retained<ProtocolObject<dyn NSObjectProtocol>>,
}
impl Observer {
    pub(super) fn new(
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
