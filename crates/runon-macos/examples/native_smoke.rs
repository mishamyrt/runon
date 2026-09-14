//! Real `NSWorkspace` delivery with a temporary app opened without activation:
//! `cargo run -p runon-macos --locked --example native_smoke`
#![allow(clippy::print_stdout, clippy::print_stderr)] // Interactive validation output.
use dispatch2::DispatchQueue;
use objc2::MainThreadMarker;
use objc2_app_kit::{
    NSApplication, NSApplicationActivationPolicy,
    NSApplicationDidChangeScreenParametersNotification, NSEvent, NSEventMask, NSEventModifierFlags,
    NSEventType, NSWorkspace, NSWorkspaceDidWakeNotification,
};
use objc2_foundation::{NSDate, NSDefaultRunLoopMode, NSNotificationCenter, NSPoint};
use runon_core::event::{Event, Kind, Value};
use runon_macos::{
    Sources,
    native::{RunLoop, Source, SourceKind, stop_main},
};
use std::{
    fs,
    process::Command,
    sync::{Arc, Mutex},
    time::Duration,
};

fn main() {
    let runloop = RunLoop::new().unwrap();
    if std::env::args().any(|a| a == "--helper") {
        let app = NSApplication::sharedApplication(MainThreadMarker::new().unwrap());
        app.setActivationPolicy(NSApplicationActivationPolicy::Regular);
        let timeout = Source::new(SourceKind::Timer, 0, DispatchQueue::main(), || {
            NSApplication::sharedApplication(MainThreadMarker::new().unwrap()).terminate(None);
        });
        timeout.arm(Some(Duration::from_secs(2)));
        app.run();
        return;
    }
    let events = Arc::new(Mutex::new(Vec::<Event>::new()));
    let id = format!("co.myrt.runon.smoke.{}", std::process::id());
    let out = events.clone();
    let expected = id.clone();
    let sources = objc2::rc::autoreleasepool(|_| {
        Sources::subscribe(
            Kind::ALL.into(),
            Arc::new(move |e| {
                let done = e.kind == Kind::AppTerminated
                    && e.fields.get("bundle-id") == Some(&Value::Text(expected.clone()));
                out.lock().unwrap().push(e);
                if done {
                    stop_main();
                }
            }),
        )
        .unwrap()
    });
    assert!(events.lock().unwrap().is_empty(), "startup emitted events");
    // A bare CFRunLoop delivers notifications but never drains AppKit's event
    // queue, which is also needed for real display-change notifications.
    let application = NSApplication::sharedApplication(MainThreadMarker::new().unwrap());
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
    ).unwrap();
    application.postEvent_atStart(&event, false);
    // These centers are process-local: no fake system notifications are broadcast.
    for _ in 0..10 {
        unsafe {
            NSNotificationCenter::defaultCenter().postNotificationName_object(
                NSApplicationDidChangeScreenParametersNotification,
                None,
            );
        }
    }
    unsafe {
        NSWorkspace::sharedWorkspace()
            .notificationCenter()
            .postNotificationName_object(NSWorkspaceDidWakeNotification, None);
    }
    let dir = std::env::temp_dir().join(format!("runon-native-{}", std::process::id()));
    let app = dir.join("RunOn Smoke.app");
    fs::create_dir_all(app.join("Contents/MacOS")).unwrap();
    fs::copy(
        std::env::current_exe().unwrap(),
        app.join("Contents/MacOS/smoke"),
    )
    .unwrap();
    let plist = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
<key>CFBundleExecutable</key><string>smoke</string>
<key>CFBundleIdentifier</key><string>{id}</string>
<key>CFBundleName</key><string>RunOn Smoke</string>
<key>CFBundlePackageType</key><string>APPL</string>
</dict></plist>"#
    );
    fs::write(app.join("Contents/Info.plist"), plist).unwrap();
    let launch = Source::new(SourceKind::Timer, 0, DispatchQueue::main(), move || {
        let status = Command::new("/usr/bin/open")
            .args(["-g", "-j"])
            .arg(&app)
            .args(["--args", "--helper"])
            .status();
        eprintln!("helper launch: {status:?}");
    });
    launch.arm(Some(Duration::from_secs(1)));
    let timeout = Source::new(SourceKind::Timer, 0, DispatchQueue::main(), stop_main);
    timeout.arm(Some(Duration::from_secs(15)));
    runloop.run();
    drop(sources);
    fs::remove_dir_all(dir).unwrap();
    assert!(
        application
            .nextEventMatchingMask_untilDate_inMode_dequeue(
                NSEventMask::ApplicationDefined,
                Some(&NSDate::distantPast()),
                unsafe { NSDefaultRunLoopMode },
                true,
            )
            .is_none(),
        "main loop did not process the AppKit event queue"
    );
    let events = events.lock().unwrap();
    for event in events.iter() {
        println!("{}", runon_config::format_selector(event));
    }
    for kind in [Kind::AppLaunched, Kind::AppTerminated] {
        assert!(
            events
                .iter()
                .any(|e| e.kind == kind
                    && e.fields.get("bundle-id") == Some(&Value::Text(id.clone()))),
            "missing {kind:?}"
        );
    }
    assert_eq!(events.iter().filter(|e| e.kind == Kind::Wake).count(), 1);
    assert!(
        !events
            .iter()
            .any(|e| matches!(e.kind, Kind::ScreenConnected | Kind::ScreenDisconnected)),
        "unexpected screen diff: keep displays unchanged during this check"
    );
    println!(
        "native: AppKit event queue, all subscriptions, silent baseline, repeat notifications, wake refresh, actual app launch/termination passed"
    );
}
