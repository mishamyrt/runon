use std::{fs, os::unix::fs::PermissionsExt, process::Command};

#[test]
fn installer_checks_checksum_and_handles_spaces_without_starting_service() {
    let dir = std::env::temp_dir().join(format!("runon install test {}", std::process::id()));
    let home = dir.join("home with spaces");
    let archive = dir.join("archive");
    let mock = dir.join("mock");
    for path in [&home, &archive, &mock] {
        fs::create_dir_all(path).unwrap();
    }
    fs::copy(env!("CARGO_BIN_EXE_runon"), archive.join("runon")).unwrap();
    assert!(
        Command::new("/usr/bin/tar")
            .current_dir(&archive)
            .args(["-czf", "runon-macos-arm64.tar.gz", "runon"])
            .status()
            .unwrap()
            .success()
    );
    let checksum = Command::new("/usr/bin/shasum")
        .current_dir(&archive)
        .args(["-a", "256", "runon-macos-arm64.tar.gz"])
        .output()
        .unwrap();
    assert!(checksum.status.success());
    fs::write(archive.join("SHA256SUMS"), &checksum.stdout).unwrap();
    fs::write(
        mock.join("curl"),
        r#"#!/bin/sh
set -eu
while [ "$#" -gt 0 ]; do
    case "$1" in
        https://*) url="$1" ;;
        -o) shift; dest="$1" ;;
    esac
    shift
done
cp "$RUNON_TEST_ARCHIVE/${url##*/}" "$dest"
"#,
    )
    .unwrap();
    fs::set_permissions(mock.join("curl"), fs::Permissions::from_mode(0o755)).unwrap();
    let install = || {
        Command::new("/bin/bash")
            .args(["scripts/install.sh", "v2.0.0"])
            .current_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
            .env("HOME", &home)
            .env("RUNON_TEST_ARCHIVE", &archive)
            .env(
                "PATH",
                format!("{}:/usr/bin:/bin:/usr/sbin:/sbin", mock.display()),
            )
            .output()
            .unwrap()
    };
    let first = install();
    assert!(
        first.status.success(),
        "{}",
        String::from_utf8_lossy(&first.stderr)
    );
    let binary = home.join(".local/bin/runon");
    let original = fs::read(&binary).unwrap();
    assert_eq!(original, fs::read(env!("CARGO_BIN_EXE_runon")).unwrap());
    assert!(!home.join("Library/LaunchAgents").exists());
    fs::write(
        archive.join("SHA256SUMS"),
        format!("{}  runon-macos-arm64.tar.gz\n", "0".repeat(64)),
    )
    .unwrap();
    assert!(!install().status.success());
    assert_eq!(fs::read(binary).unwrap(), original);
    fs::remove_dir_all(dir).unwrap();
}
