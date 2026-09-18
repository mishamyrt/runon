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
    // Both fixtures use the host binary; mocked uname checks archive selection.
    for arch in ["arm64", "x86_64"] {
        assert!(
            Command::new("/usr/bin/tar")
                .current_dir(&archive)
                .args(["-czf", &format!("runon-macos-{arch}.tar.gz"), "runon"])
                .status()
                .unwrap()
                .success()
        );
    }
    let checksum = Command::new("/usr/bin/shasum")
        .current_dir(&archive)
        .args([
            "-a",
            "256",
            "runon-macos-arm64.tar.gz",
            "runon-macos-x86_64.tar.gz",
        ])
        .output()
        .unwrap();
    assert!(checksum.status.success());
    for (name, script) in [
        (
            "uname",
            "#!/bin/sh\ncase \"$1\" in\n-s) echo \"$RUNON_TEST_OS\" ;;\n-m) echo \"$RUNON_TEST_ARCH\" ;;\nesac\n",
        ),
        ("sw_vers", "#!/bin/sh\necho \"$RUNON_TEST_VERSION\"\n"),
        (
            "curl",
            r#"#!/bin/sh
set -eu
while [ "$#" -gt 0 ]; do
    case "$1" in
        https://*) url="$1" ;;
        -o) shift; dest="$1" ;;
    esac
    shift
done
case "$url" in
    *.tar.gz) test "${url##*/}" = "runon-macos-$RUNON_TEST_ARCH.tar.gz" ;;
esac
echo "$url" >> "$RUNON_TEST_ARCHIVE/downloads"
cp "$RUNON_TEST_ARCHIVE/${url##*/}" "$dest"
"#,
        ),
    ] {
        fs::write(mock.join(name), script).unwrap();
        fs::set_permissions(mock.join(name), fs::Permissions::from_mode(0o755)).unwrap();
    }
    let install = |arch: &str, os: &str, version: &str| {
        Command::new("/bin/bash")
            .args(["scripts/install.sh", "v2.0.0"])
            .current_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
            .env("HOME", &home)
            .env("RUNON_TEST_ARCHIVE", &archive)
            .env("RUNON_TEST_ARCH", arch)
            .env("RUNON_TEST_OS", os)
            .env("RUNON_TEST_VERSION", version)
            .env(
                "PATH",
                format!("{}:/usr/bin:/bin:/usr/sbin:/sbin", mock.display()),
            )
            .output()
            .unwrap()
    };
    let binary = home.join(".local/bin/runon");
    let original = fs::read(env!("CARGO_BIN_EXE_runon")).unwrap();
    for arch in ["arm64", "x86_64"] {
        fs::write(archive.join("SHA256SUMS"), &checksum.stdout).unwrap();
        for version in ["15.0", "26.0"] {
            let result = install(arch, "Darwin", version);
            assert!(
                result.status.success(),
                "{}",
                String::from_utf8_lossy(&result.stderr)
            );
            assert_eq!(fs::read(&binary).unwrap(), original);
            assert!(!home.join("Library/LaunchAgents").exists());
        }
        for sums in [
            format!("{}  runon-macos-{arch}.tar.gz\n", "0".repeat(64)),
            String::from_utf8(checksum.stdout.clone())
                .unwrap()
                .lines()
                .find(|line| !line.ends_with(&format!("runon-macos-{arch}.tar.gz")))
                .unwrap()
                .to_owned(),
        ] {
            fs::write(archive.join("SHA256SUMS"), sums).unwrap();
            assert!(!install(arch, "Darwin", "15.0").status.success());
            assert_eq!(fs::read(&binary).unwrap(), original);
        }
    }
    let downloads = fs::read(archive.join("downloads")).unwrap();
    for (arch, os, version) in [
        ("arm64", "Darwin", "14.7"),
        ("x86_64", "Darwin", "14.7"),
        ("x86_64", "Linux", "15.0"),
        ("ppc", "Darwin", "15.0"),
    ] {
        assert!(!install(arch, os, version).status.success());
        assert_eq!(fs::read(&binary).unwrap(), original);
        assert_eq!(fs::read(archive.join("downloads")).unwrap(), downloads);
    }
    fs::remove_dir_all(dir).unwrap();
}
