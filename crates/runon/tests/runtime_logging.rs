//! Exercise the real logger with stderr left unread, including queue overflow.
#![allow(clippy::print_stdout)] // Standalone test summary.
use runon_config::Config;
use runon_core::event::{Event, Kind};
use runon_macos::native::Signals;
use runon_runtime::{Report, Runtime};
use std::{
    fs,
    path::Path,
    process::{Command, Stdio},
    sync::{Arc, mpsc},
    time::{Duration, Instant},
};

#[path = "../src/logging.rs"]
mod logging;

fn child(dir: &Path) {
    logging::init(false).unwrap();
    let config = Config::parse(
        r##"
        action noisy {
            on system.wake
            shell #"head -c 65536 /dev/zero | tr '\0' x; exit 1"#
        }
        action timed {
            on system.wake
            timeout "100ms"
            shell "trap '' TERM; sleep 3; touch timeout-was-bypassed"
        }
        action stop { on screen.locked; exec "/bin/sleep" "30"; }
    "##,
    )
    .unwrap();
    let (tx, rx) = mpsc::channel();
    let runtime = Runtime::new(config, move |report| {
        let summary = report_summary(&report);
        logging::report(report);
        tx.send(summary).unwrap();
    })
    .unwrap();
    let stop = runtime.clone();
    let _signals = Signals::new(&runtime.queue(), Arc::new(move || stop.shutdown())).unwrap();
    runtime.submit(Event::new(Kind::Wake));
    let mut completed = 0;
    while completed < 2 {
        if let Report::Finished {
            name,
            success,
            reason,
            ..
        } = rx.recv_timeout(Duration::from_secs(5)).unwrap()
        {
            assert!(!success);
            if name == "timed" {
                assert_eq!(reason, "timeout");
            }
            completed += 1;
        }
    }
    assert!(!dir.join("timeout-was-bypassed").exists());
    // The worker is stuck in a 64 KiB write. Flooding its bounded queue must
    // neither allocate one task per message nor delay process supervision.
    for _ in 0..1_000 {
        log::info!("overflow probe");
    }
    let before = Instant::now();
    log::logger().flush();
    assert!(before.elapsed() < Duration::from_secs(1));
    runtime.submit(Event::new(Kind::ScreenLocked));
    assert!(matches!(
        rx.recv_timeout(Duration::from_secs(5)).unwrap(),
        Report::Started { .. }
    ));
    fs::write(dir.join("ready-for-signal"), "").unwrap();
    // Parent sends SIGTERM without draining stderr.
    assert!(matches!(
        rx.recv_timeout(Duration::from_secs(5)).unwrap(),
        Report::Finished { success: false, .. }
    ));
    assert!(matches!(
        rx.recv_timeout(Duration::from_secs(5)).unwrap(),
        Report::Stopped
    ));
    log::logger().flush();
}

fn report_summary(report: &Report) -> Report {
    match report {
        Report::Started { name, latency } => Report::Started {
            name: name.clone(),
            latency: *latency,
        },
        Report::Finished {
            name,
            success,
            reason,
            ..
        } => Report::Finished {
            name: name.clone(),
            success: *success,
            reason: reason.clone(),
            stdout: String::new(),
            stderr: String::new(),
        },
        Report::Stopped => Report::Stopped,
    }
}

fn main() {
    if let Some(dir) = std::env::args_os().nth(1) {
        child(Path::new(&dir));
        return;
    }
    let dir = std::env::temp_dir().join(format!("runon-logging-{}", std::process::id()));
    fs::create_dir(&dir).unwrap();
    let mut process = Command::new(std::env::current_exe().unwrap())
        .arg(&dir)
        .env("HOME", &dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(15);
    let mut signalled = false;
    loop {
        if process.try_wait().unwrap().is_some() {
            break;
        }
        if !signalled && dir.join("ready-for-signal").exists() {
            unsafe {
                libc::kill(process.id().try_into().unwrap(), libc::SIGTERM);
            }
            signalled = true;
        }
        if Instant::now() >= deadline {
            process.kill().unwrap();
            break;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    let output = process.wait_with_output().unwrap();
    fs::remove_dir_all(dir).unwrap();
    assert!(
        output.status.success(),
        "helper failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(signalled, "helper never reached the shutdown check");
    println!("logging: unread stderr and full log queue preserve timeouts and SIGTERM shutdown");
}
