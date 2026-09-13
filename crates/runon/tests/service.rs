use runon_macos::LaunchAgent;
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::PathBuf,
    time::{Duration, Instant},
};

struct Cleanup {
    agent: LaunchAgent,
    dir: PathBuf,
}
impl Drop for Cleanup {
    fn drop(&mut self) {
        let _ = self.agent.stop();
        let _ = fs::remove_dir_all(&self.dir);
    }
}

fn pid(agent: &LaunchAgent) -> Option<i32> {
    agent
        .status()
        .unwrap()
        .strip_prefix("running (pid ")
        .and_then(|s| s.strip_suffix(')'))
        .and_then(|s| s.parse().ok())
}
fn wait_pid(agent: &LaunchAgent, old: Option<i32>) -> i32 {
    let deadline = Instant::now() + Duration::from_secs(20);
    loop {
        if let Some(pid) = pid(agent)
            && Some(pid) != old
        {
            return pid;
        }
        assert!(
            Instant::now() < deadline,
            "service failed to start: {}",
            agent.status().unwrap()
        );
        std::thread::sleep(Duration::from_millis(50));
    }
}

#[test]
#[ignore = "requires a logged-in GUI session; installs a temporary, uniquely labelled LaunchAgent"]
fn isolated_launchd_lifecycle_and_crash_recovery() {
    let dir = std::env::temp_dir().join(format!("runon service test {}", std::process::id()));
    fs::create_dir(&dir).unwrap();
    let agent = LaunchAgent::new(
        format!("co.myrt.runon.test.{}", std::process::id()),
        dir.clone(),
        PathBuf::from(env!("CARGO_BIN_EXE_runon")),
    );
    let cleanup = Cleanup { agent, dir };
    let agent = &cleanup.agent;
    let a = cleanup.dir.join("config one.kdl");
    let b = cleanup.dir.join("config two.kdl");
    fs::write(&a, "").unwrap();
    fs::write(&b, "").unwrap();
    assert_eq!(agent.status().unwrap(), "stopped");
    assert!(!agent.path().exists(), "status must be read-only");
    agent.start(Some(&a), false).unwrap();
    let first = wait_pid(agent, None);
    let plist = fs::read(agent.path()).unwrap();
    agent.start(Some(&b), false).unwrap();
    assert_eq!(pid(agent), Some(first));
    assert_eq!(fs::read(agent.path()).unwrap(), plist);
    fs::write(&a, "bad 1").unwrap();
    assert!(agent.start(None, true).is_err());
    assert_eq!(pid(agent), Some(first));
    agent.start(Some(&b), true).unwrap();
    let second = wait_pid(agent, Some(first));
    assert_eq!(
        agent.saved_config().unwrap().unwrap(),
        fs::canonicalize(&b).unwrap()
    );
    agent.start(None, true).unwrap();
    let third = wait_pid(agent, Some(second));
    assert_eq!(
        agent.saved_config().unwrap().unwrap(),
        fs::canonicalize(&b).unwrap()
    );
    unsafe {
        libc::kill(third, libc::SIGKILL);
    }
    let _recovered = wait_pid(agent, Some(third));
    agent.stop().unwrap();
    assert_eq!(agent.status().unwrap(), "stopped");
    assert!(agent.path().exists());
    agent.stop().unwrap();

    // Exercise launchctl's asynchronous bootout with an intentionally slow
    // SIGTERM handler, without relying on hardware events to start an action.
    let script = cleanup.dir.join("slow exit.sh");
    fs::write(&script, "#!/bin/sh\ntrap 'kill \"$kid\" 2>/dev/null || :; sleep 1; exit 0' TERM\nsleep 30 &\nkid=$!\necho ready > \"$HOME/ready\"\nwait \"$kid\"\n").unwrap();
    fs::set_permissions(&script, fs::Permissions::from_mode(0o755)).unwrap();
    let slow = LaunchAgent::new(
        format!("co.myrt.runon.test.{}", std::process::id()),
        cleanup.dir.clone(),
        script,
    );
    slow.start(Some(&b), false).unwrap();
    let old = wait_pid(&slow, None);
    let deadline = Instant::now() + Duration::from_secs(5);
    while !cleanup.dir.join("ready").exists() {
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(10));
    }
    let start = Instant::now();
    slow.start(None, true).unwrap();
    assert!(
        start.elapsed() >= Duration::from_millis(900),
        "restart overlapped the old process"
    );
    let _new = wait_pid(&slow, Some(old));
    slow.stop().unwrap();
}
