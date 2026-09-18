use super::{Emit, Observer};
use crate::native::Source;
use objc2_app_kit::NSWorkspaceDidWakeNotification;
use objc2_foundation::NSNotificationCenter;
use runon_core::event::{Event, Kind};
use std::sync::Arc;

pub(super) fn subscribe(
    workspace: &NSNotificationCenter,
    emit: Emit,
    refreshers: Vec<Option<Arc<Source>>>,
) -> Observer {
    Observer::new(
        workspace,
        unsafe { NSWorkspaceDidWakeNotification },
        move |_| {
            emit(Event::new(Kind::Wake));
            for refresh in refreshers.iter().flatten() {
                refresh.wake();
            }
        },
    )
}
