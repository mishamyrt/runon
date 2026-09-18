#![allow(clippy::print_stdout)] // Standalone test summary.
use runon_config::Config;
use runon_core::event::{Event, Kind};
use runon_runtime::{OUTPUT_LIMIT, Report, Runtime};
use std::{
    fs,
    sync::{
        Arc, OnceLock, Weak,
        mpsc::{self, Receiver},
    },
    time::{Duration, Instant},
};

fn finished(rx: &Receiver<Report>) -> (String, bool, String, String, String) {
    loop {
        match rx
            .recv_timeout(Duration::from_secs(12))
            .expect("runtime stalled")
        {
            Report::Finished {
                name,
                success,
                reason,
                stdout,
                stderr,
            } => return (name, success, reason, stdout, stderr),
            Report::Started { .. } => {}
            Report::Stopped => panic!("unexpected shutdown"),
        }
    }
}

fn send(runtime: &Runtime, tag: &str) {
    runtime.submit(Event::new(Kind::AppActivated).text("bundle-id", tag));
}

fn main() {
    replacement_order_and_callback_reentry();
    let dir = std::env::temp_dir().join(format!("runon-runtime-{}", std::process::id()));
    fs::create_dir(&dir).unwrap();
    let escaped = dir.join("literal $HOME with spaces");
    let child_pid = dir.join("child.pid");
    let forbidden = dir.join("must-not-exist");
    let text = format!(
        r##"
        max-parallel 4
        shell-path "/bin/bash"
        action fast {{ on app.activated bundle-id=fast; exec "/usr/bin/true"; }}
        action custom-shell {{ on app.activated bundle-id=custom-shell; shell "test \"$0\" = /bin/bash"; }}
        action absent {{ on app.activated bundle-id=absent; exec "/does/not/exist"; }}
        action literal {{
            on app.activated bundle-id=literal
            exec "/usr/bin/touch" "{}"
            exec "/bin/sh" "-c" "printf '%s' \"$1\"; exit 7" "sh" "$HOME; literal"
            exec "/usr/bin/touch" "{}"
        }}
        action output {{
            on app.activated bundle-id=output
            shell #"head -c 2000000 /dev/zero | tr '\0' x; printf end; head -c 2000000 /dev/zero | tr '\0' y >&2; printf err >&2; exit 9"#
        }}
        action timeout {{
            on app.activated bundle-id=timeout
            timeout "100ms"
            shell #"trap '' TERM; sleep 30 & echo $! > '{}'; wait"#
        }}
        action stop {{ on app.activated bundle-id=stop; exec "/bin/sleep" "30"; }}
        group order {{ shell-path "/bin/sh"; }}
        action first group=order {{ on app.activated bundle-id=batch; shell "exit 1"; }}
        action second group=order {{ on app.activated bundle-id=batch; shell "test \"$0\" = /bin/sh"; exec "/usr/bin/true"; }}
    "##,
        escaped.display(),
        forbidden.display(),
        child_pid.display()
    );
    let (tx, rx) = mpsc::channel();
    let runtime = Runtime::new(Config::parse(&text).unwrap(), move |r| {
        tx.send(r).unwrap();
    })
    .unwrap();
    send(&runtime, "custom-shell");
    assert!(finished(&rx).1);
    for _ in 0..100 {
        send(&runtime, "fast");
        let (name, success, ..) = finished(&rx);
        assert_eq!(name, "fast");
        assert!(success);
    }
    send(&runtime, "absent");
    let (_, success, reason, ..) = finished(&rx);
    assert!(!success && reason.starts_with("spawn:"));
    send(&runtime, "literal");
    let (_, success, _, stdout, _) = finished(&rx);
    assert!(!success);
    assert_eq!(stdout, "$HOME; literal");
    assert!(escaped.exists());
    assert!(!forbidden.exists());
    send(&runtime, "output");
    let (_, success, _, stdout, stderr) = finished(&rx);
    assert!(!success);
    assert_eq!(stdout.len(), OUTPUT_LIMIT);
    assert!(stdout.ends_with("end"));
    assert_eq!(stderr.len(), OUTPUT_LIMIT);
    assert!(stderr.ends_with("err"));
    send(&runtime, "batch");
    let (name, success, ..) = finished(&rx);
    assert_eq!(name, "first");
    assert!(!success);
    let (name, success, ..) = finished(&rx);
    assert_eq!(name, "second");
    assert!(success);
    let start = Instant::now();
    send(&runtime, "timeout");
    let (_, success, reason, ..) = finished(&rx);
    assert!(!success && reason == "timeout");
    assert!(start.elapsed() >= Duration::from_secs(2));
    let pid: i32 = fs::read_to_string(child_pid)
        .unwrap()
        .trim()
        .parse()
        .unwrap();
    // macOS may briefly keep the killed grandchild as a zombie until launchd reaps it.
    let output = std::process::Command::new("/bin/ps")
        .args(["-o", "stat=", "-p", &pid.to_string()])
        .output()
        .unwrap();
    let state = String::from_utf8_lossy(&output.stdout);
    assert!(
        state.trim().is_empty() || state.trim().starts_with('Z'),
        "grandchild still running: {state}"
    );
    send(&runtime, "fast");
    assert!(finished(&rx).1);
    send(&runtime, "stop");
    loop {
        if matches!(
            rx.recv_timeout(Duration::from_secs(5)).unwrap(),
            Report::Started { .. }
        ) {
            break;
        }
    }
    runtime.shutdown();
    assert!(!finished(&rx).1);
    assert!(matches!(
        rx.recv_timeout(Duration::from_secs(5)).unwrap(),
        Report::Stopped
    ));
    drop(runtime);
    fs::remove_dir_all(dir).unwrap();
    println!(
        "runtime: custom shells, 100 fast exits, literal argv, sequential failure, bounded output, batch ordering, timeout, process-group cleanup and shutdown passed"
    );
}

fn replacement_order_and_callback_reentry() {
    for (debounce, expected) in [(0, ["a", "b"]), (80, ["b", "a"])] {
        let config = Config::parse(&format!(
            r#"
            max-parallel 1
            group a {{ debounce "{debounce}ms"; }}
            action old group=a {{ on app.activated name=old; exec "/usr/bin/true"; }}
            action a group=a {{ on app.activated name=a; exec "/usr/bin/true"; }}
            action b {{ on app.activated name=b; exec "/usr/bin/true"; }}
        "#
        ))
        .unwrap();
        let handle = Arc::new(OnceLock::<Weak<Runtime>>::new());
        let callback_handle = handle.clone();
        let (tx, rx) = mpsc::channel();
        let runtime = Arc::new(
            Runtime::new(config, move |report| {
                // This used to deadlock because reports held the State mutex.
                let runtime = callback_handle.get().unwrap().upgrade().unwrap();
                let _queue = runtime.queue();
                tx.send(report).unwrap();
            })
            .unwrap(),
        );
        handle.set(Arc::downgrade(&runtime)).unwrap();
        let input = runtime.clone();
        runtime.queue().exec_sync(move || {
            // No drive can interleave with these three submissions.
            for name in ["old", "b", "a"] {
                input.submit(Event::new(Kind::AppActivated).text("name", name));
            }
        });
        let mut started = Vec::new();
        let mut completed = 0;
        while completed < 2 {
            match rx.recv_timeout(Duration::from_secs(5)).unwrap() {
                Report::Started { name, latency } => {
                    if name == "a" {
                        assert!(latency >= Duration::from_millis(debounce));
                    }
                    started.push(name);
                }
                Report::Finished { success, .. } => {
                    assert!(success);
                    completed += 1;
                }
                Report::Stopped => panic!("unexpected stop"),
            }
        }
        assert_eq!(started, expected);
        runtime.shutdown();
        assert!(matches!(
            rx.recv_timeout(Duration::from_secs(5)).unwrap(),
            Report::Stopped
        ));
    }
}
