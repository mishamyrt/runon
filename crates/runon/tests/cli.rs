use std::{
    fs,
    io::Write,
    process::{Command, Stdio},
    time::{Duration, Instant},
};

const BIN: &str = env!("CARGO_BIN_EXE_runon");

#[test]
fn help_and_removed_commands() {
    for flag in ["--help", "-h"] {
        let out = Command::new(BIN).arg(flag).output().unwrap();
        assert!(out.status.success());
        let help = String::from_utf8(out.stdout).unwrap();
        assert!(help.contains("With no command, run in the foreground"));
        for command in ["check", "events", "logs"] {
            assert!(help.contains(&format!("\n  {command} ")));
        }
    }
    for command in ["run", "help", "start", "stop", "restart", "status"] {
        let out = Command::new(BIN).arg(command).output().unwrap();
        assert!(!out.status.success());
        assert!(
            String::from_utf8_lossy(&out.stderr).contains(&format!("unknown command '{command}'"))
        );
    }
}

#[test]
fn examples_errors_and_defaults() {
    for path in ["examples/config.kdl", "examples/editors.kdl"] {
        assert!(
            Command::new(BIN)
                .args(["check", "-c", path])
                .current_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
                .status()
                .unwrap()
                .success()
        );
    }
    let dir = std::env::temp_dir().join(format!("runon-cli-{}", std::process::id()));
    fs::create_dir_all(dir.join("runon")).unwrap();
    let config = dir.join("runon/config.kdl");
    fs::write(&config, "\nunknown 1").unwrap();
    let out = Command::new(BIN)
        .arg("check")
        .env("XDG_CONFIG_HOME", &dir)
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains(":2:1: unknown node"));
    for args in [
        vec!["--wat"],
        vec!["events", "-c", "x"],
        vec!["logs", "-c", "x"],
        vec!["check", "-c"],
        vec!["--service"],
        vec!["check", "--service"],
        vec!["unknown"],
    ] {
        let out = Command::new(BIN)
            .args(&args)
            .env("XDG_CONFIG_HOME", &dir)
            .output()
            .unwrap();
        assert!(!out.status.success());
        if args.contains(&"--service") {
            assert!(String::from_utf8_lossy(&out.stderr).contains("--service"));
        }
    }
    fs::write(&config, "// empty config\n").unwrap();
    let invalid = dir.join("invalid.kdl");
    fs::write(&invalid, "unknown 1").unwrap();
    for flag in ["-c", "--config"] {
        let out = Command::new(BIN)
            .arg(flag)
            .arg(&invalid)
            .env("XDG_CONFIG_HOME", &dir)
            .output()
            .unwrap();
        assert!(!out.status.success());
        let error = String::from_utf8_lossy(&out.stderr);
        assert!(error.contains("invalid.kdl:1:1: unknown node"));
    }
    let stderr = dir.join("stderr.log");
    let mut child = Command::new(BIN)
        .env("XDG_CONFIG_HOME", &dir)
        .stdout(Stdio::null())
        .stderr(fs::File::create(&stderr).unwrap())
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    while !fs::read_to_string(&stderr)
        .unwrap()
        .contains("ready; config=")
    {
        assert!(
            child.try_wait().unwrap().is_none(),
            "daemon exited during startup"
        );
        if Instant::now() > deadline {
            let _ = child.kill();
            let _ = child.wait();
            panic!("daemon did not become ready");
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    std::thread::sleep(Duration::from_millis(150));
    assert!(
        child.try_wait().unwrap().is_none(),
        "empty daemon must keep sleeping"
    );
    unsafe {
        libc::kill(child.id().try_into().unwrap(), libc::SIGTERM);
    }
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if let Some(status) = child.try_wait().unwrap() {
            assert!(status.success());
            break;
        }
        if Instant::now() > deadline {
            let _ = child.kill();
            let _ = child.wait();
            panic!("daemon did not stop");
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn logs_follows_service_file() {
    let dir = std::env::temp_dir().join(format!("runon logs {}", std::process::id()));
    fs::create_dir_all(dir.join("var/log")).unwrap();
    let log = dir.join("var/log/runon.log");
    fs::write(&log, "INFO initial record\n").unwrap();
    let output = dir.join("tail.out");
    let mut child = Command::new(BIN)
        .arg("logs")
        .env("HOMEBREW_PREFIX", &dir)
        .stdout(fs::File::create(&output).unwrap())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    for record in ["initial record", "appended record", "replacement record"] {
        match record {
            "appended record" => {
                writeln!(
                    fs::OpenOptions::new().append(true).open(&log).unwrap(),
                    "INFO {record}"
                )
                .unwrap();
            }
            "replacement record" => {
                fs::rename(&log, dir.join("previous.log")).unwrap();
                fs::write(&log, format!("INFO {record}\n")).unwrap();
            }
            _ => {}
        }
        let deadline = Instant::now() + Duration::from_secs(5);
        while !fs::read_to_string(&output).unwrap().contains(record) {
            assert!(child.try_wait().unwrap().is_none(), "logs exited early");
            if Instant::now() > deadline {
                let _ = child.kill();
                let _ = child.wait();
                panic!("logs did not print {record}");
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    }
    child.kill().unwrap();
    child.wait().unwrap();
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn logs_rejects_relative_homebrew_prefix() {
    let out = Command::new(BIN)
        .arg("logs")
        .env("HOMEBREW_PREFIX", "relative")
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("HOMEBREW_PREFIX must be an absolute path")
    );
}
