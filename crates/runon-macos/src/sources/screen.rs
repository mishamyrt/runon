use super::Observer;
use crate::native::Source;
use objc2::MainThreadMarker;
use objc2_app_kit::{NSApplicationDidChangeScreenParametersNotification, NSScreen};
use objc2_foundation::{NSNotificationCenter, NSNumber, NSString};
use runon_core::event::{Event, Kind, Value};
use std::{collections::BTreeMap, sync::Arc};

pub(super) fn subscribe(refresh: Arc<Source>) -> Observer {
    Observer::new(
        &NSNotificationCenter::defaultCenter(),
        unsafe { NSApplicationDidChangeScreenParametersNotification },
        move |_| {
            refresh.wake();
        },
    )
}

pub(super) fn snapshot() -> Result<BTreeMap<u32, Event>, String> {
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
