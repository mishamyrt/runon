use std::{
    fs,
    process::{Command, Stdio},
    time::{Duration, Instant},
};

const BIN: &str = env!("CARGO_BIN_EXE_runon");

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
        vec!["run", "--wat"],
        vec!["status", "-c", "x"],
        vec!["check", "-c"],
        vec!["stop", "--service"],
        vec!["unknown"],
    ] {
        assert!(
            !Command::new(BIN)
                .args(args)
                .output()
                .unwrap()
                .status
                .success()
        );
    }
    fs::write(&config, "// empty config\n").unwrap();
    let stderr = dir.join("stderr.log");
    let mut child = Command::new(BIN)
        .arg("run")
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
