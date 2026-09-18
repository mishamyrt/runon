use super::{Emit, Observer};
use objc2_app_kit::{
    NSRunningApplication, NSWorkspaceApplicationKey, NSWorkspaceDidActivateApplicationNotification,
    NSWorkspaceDidDeactivateApplicationNotification, NSWorkspaceDidLaunchApplicationNotification,
    NSWorkspaceDidTerminateApplicationNotification,
};
use objc2_foundation::NSNotificationCenter;
use runon_core::event::{Event, Kind};
use std::collections::BTreeSet;

pub(super) fn subscribe(
    kinds: &BTreeSet<Kind>,
    workspace: &NSNotificationCenter,
    emit: &Emit,
) -> Vec<Observer> {
    let mut observers = Vec::new();
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
        observers.push(Observer::new(workspace, name, move |n| {
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
    observers
}
