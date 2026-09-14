use crate::{
    native::{Source, SourceKind},
    paths::{self, COMMAND_PATH},
};
use dispatch2::DispatchQueue;
use objc2::{
    rc::Retained,
    runtime::{AnyObject, ProtocolObject},
};
use objc2_foundation::{
    NSArray, NSData, NSDictionary, NSMutableDictionary, NSNumber, NSPropertyListFormat,
    NSPropertyListMutabilityOptions, NSPropertyListSerialization, NSString, ns_string,
};
use runon_config::Config;
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::mpsc,
    time::Duration,
};

pub struct LaunchAgent {
    label: String,
    home: PathBuf,
    binary: PathBuf,
}

fn read_plist(bytes: Vec<u8>) -> Result<Retained<NSMutableDictionary>, String> {
    // SAFETY: NSData owns its bytes; the optional output format pointer is null.
    let value = unsafe {
        NSPropertyListSerialization::propertyListWithData_options_format_error(
            &NSData::from_vec(bytes),
            NSPropertyListMutabilityOptions::MutableContainers,
            std::ptr::null_mut(),
        )
    }
    .map_err(|e| e.to_string())?;
    value
        .downcast()
        .map_err(|_| "agent plist must be a dictionary".into())
}

impl LaunchAgent {
    pub fn current() -> Result<Self, String> {
        Ok(Self::new(
            crate::APP_ID.into(),
            paths::home()?,
            std::env::current_exe().map_err(|e| e.to_string())?,
        ))
    }

    pub fn new(label: String, home: PathBuf, binary: PathBuf) -> Self {
        Self {
            label,
            home,
            binary,
        }
    }
    fn domain() -> String {
        format!("gui/{}", unsafe { libc::geteuid() })
    }
    fn target(&self) -> String {
        format!("{}/{}", Self::domain(), self.label)
    }
    pub fn path(&self) -> PathBuf {
        self.home
            .join("Library/LaunchAgents")
            .join(format!("{}.plist", self.label))
    }

    fn launchctl(args: &[&str]) -> Result<Output, String> {
        Command::new("/bin/launchctl")
            .args(args)
            .output()
            .map_err(|e| format!("launchctl: {e}"))
    }

    fn checked(args: &[&str]) -> Result<(), String> {
        let out = Self::launchctl(args)?;
        if !out.status.success() {
            return Err(format!(
                "launchctl {}: {}",
                args[0],
                String::from_utf8_lossy(&out.stderr).trim()
            ));
        }
        Ok(())
    }

    fn inspection(&self) -> Result<Option<String>, String> {
        let output = Self::launchctl(&["print", &self.target()])?;
        if output.status.success() {
            return Ok(Some(String::from_utf8_lossy(&output.stdout).into_owned()));
        }
        let error = String::from_utf8_lossy(&output.stderr);
        if error.contains("Could not find service") {
            return Ok(None);
        }
        Err(format!("launchctl print: {}", error.trim()))
    }

    fn unload(&self, inspection: &str) -> Result<(), String> {
        let pid: Option<i32> = inspection
            .lines()
            .find_map(|line| line.trim().strip_prefix("pid = "))
            .and_then(|value| value.parse().ok())
            .filter(|pid| *pid > 0);
        let queue = DispatchQueue::new("co.myrt.runon.launchctl", None);
        let (tx, rx) = mpsc::sync_channel(1);
        // Observe before bootout: launchctl can return before the process exits.
        let _exit = pid.map(|pid| {
            Source::new(
                SourceKind::Process,
                usize::try_from(pid).unwrap(),
                &queue,
                move || {
                    let _ = tx.try_send(());
                },
            )
        });
        Self::checked(&["bootout", &self.target()])?;
        if let Some(pid) = pid {
            // The process may have exited even before observer registration.
            let gone = unsafe { libc::kill(pid, 0) } != 0
                && std::io::Error::last_os_error().raw_os_error() == Some(libc::ESRCH);
            if !gone {
                rx.recv_timeout(Duration::from_secs(7)).map_err(
                    |_| "service did not exit within 7 seconds; replacement was not started",
                )?;
            }
        }
        Ok(())
    }

    pub fn status(&self) -> Result<String, String> {
        match self.inspection()? {
            None => Ok("stopped".into()),
            Some(text) => {
                let running = text.lines().any(|l| l.trim() == "state = running");
                let pid = text.lines().find_map(|l| l.trim().strip_prefix("pid = "));
                Ok(if running {
                    format!("running (pid {})", pid.unwrap_or("unknown"))
                } else {
                    "loaded, waiting to run".into()
                })
            }
        }
    }

    pub fn saved_config(&self) -> Result<Option<PathBuf>, String> {
        let path = self.path();
        if !path.exists() {
            return Ok(None);
        }
        let bytes = fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        let value = read_plist(bytes).map_err(|e| format!("{}: {e}", path.display()))?;
        let args = value
            .objectForKey(ns_string!("ProgramArguments"))
            .and_then(|v| v.downcast::<NSArray>().ok())
            .ok_or("agent has no ProgramArguments")?;
        let args: Vec<_> = args
            .iter()
            .map(|v| {
                v.downcast_ref::<NSString>()
                    .map(std::string::ToString::to_string)
            })
            .collect();
        for pair in args.windows(2) {
            if matches!(pair[0].as_deref(), Some("-c" | "--config")) {
                return pair[1]
                    .as_deref()
                    .map(PathBuf::from)
                    .map(Some)
                    .ok_or_else(|| "agent config path must be a string".into());
            }
        }
        Ok(None)
    }

    fn document(&self, path: &Path) -> Result<Vec<u8>, String> {
        let binary = self.binary.to_str().ok_or("binary path must be UTF-8")?;
        let path = path.to_str().ok_or("config path must be UTF-8")?;
        let home = self.home.to_str().ok_or("home path must be UTF-8")?;
        let arguments = NSArray::from_retained_slice(
            &[binary, "run", "--service", "-c", path].map(NSString::from_str),
        );
        let dict = NSMutableDictionary::<NSString, AnyObject>::new();
        let keepalive = NSDictionary::from_slices(
            &[ns_string!("SuccessfulExit")],
            &[&*NSNumber::new_bool(false)],
        );
        let env = NSDictionary::from_slices(
            &[ns_string!("HOME"), ns_string!("PATH")],
            &[
                &*NSString::from_str(home),
                &*NSString::from_str(COMMAND_PATH),
            ],
        );
        for (key, value) in [
            (
                ns_string!("Label"),
                &*NSString::from_str(&self.label) as &AnyObject,
            ),
            (ns_string!("ProgramArguments"), &*arguments),
            (ns_string!("RunAtLoad"), &*NSNumber::new_bool(true)),
            (ns_string!("WorkingDirectory"), &*NSString::from_str(home)),
            (ns_string!("KeepAlive"), &*keepalive),
            (ns_string!("EnvironmentVariables"), &*env),
            (ns_string!("ThrottleInterval"), &*NSNumber::new_u64(10)),
            (ns_string!("ExitTimeOut"), &*NSNumber::new_u64(5)),
            (ns_string!("LimitLoadToSessionType"), ns_string!("Aqua")),
        ] {
            // SAFETY: all keys are NSString and all values are property-list types.
            unsafe {
                dict.setObject_forKey(value, ProtocolObject::from_ref(key));
            }
        }
        // SAFETY: the dictionary contains only property-list values.
        unsafe {
            NSPropertyListSerialization::dataWithPropertyList_format_options_error(
                &dict,
                NSPropertyListFormat::XMLFormat_v1_0,
                0,
            )
        }
        .map(|data| data.to_vec())
        .map_err(|e| e.to_string())
    }

    pub fn start(&self, path: Option<&Path>, restart: bool) -> Result<(), String> {
        let path = match path {
            Some(p) => p.to_path_buf(),
            None => self
                .saved_config()?
                .map(Ok)
                .unwrap_or_else(paths::default_config_path)?,
        };
        let path = fs::canonicalize(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        Config::load(&path)?;
        let loaded = self.inspection()?;
        if loaded.is_some() && !restart {
            return Ok(());
        }
        // Prepare and sync the replacement before taking the working service down.
        let document = self.document(&path)?;
        let agent = self.path();
        let dir = agent.parent().unwrap();
        fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        let staging = dir.join(format!(".{}.{}.tmp", self.label, std::process::id()));
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&staging)
            .map_err(|e| e.to_string())?;
        let result = (|| {
            file.write_all(&document).map_err(|e| e.to_string())?;
            file.flush()
                .and_then(|()| file.sync_all())
                .map_err(|e| e.to_string())?;
            if let Some(inspection) = &loaded {
                self.unload(inspection)?;
            }
            fs::rename(&staging, &agent).map_err(|e| e.to_string())?;
            Self::checked(&[
                "bootstrap",
                &Self::domain(),
                agent.to_str().ok_or("agent path must be UTF-8")?,
            ])
        })();
        let _ = fs::remove_file(staging);
        result
    }

    pub fn stop(&self) -> Result<(), String> {
        if let Some(inspection) = self.inspection()? {
            self.unload(&inspection)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn plist_escapes_paths_and_preserves_config() {
        let dir = std::env::temp_dir().join(format!("runon plist & {}", std::process::id()));
        let agent = LaunchAgent::new(
            "co.myrt.runon.test".into(),
            dir.clone(),
            PathBuf::from("/tmp/a & b/runon"),
        );
        let config = Path::new("/tmp/конфиг <new> & 'quoted'.kdl");
        let path = agent.path();
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, agent.document(config).unwrap()).unwrap();
        // Check the launchd contract with macOS's independent plist reader.
        for (key, kind, expected) in [
            ("Label", "string", agent.label.as_str()),
            ("ProgramArguments.0", "string", "/tmp/a & b/runon"),
            ("ProgramArguments.4", "string", config.to_str().unwrap()),
            ("WorkingDirectory", "string", dir.to_str().unwrap()),
            ("EnvironmentVariables.HOME", "string", dir.to_str().unwrap()),
            ("EnvironmentVariables.PATH", "string", COMMAND_PATH),
            ("RunAtLoad", "bool", "true"),
            ("KeepAlive.SuccessfulExit", "bool", "false"),
            ("ThrottleInterval", "integer", "10"),
            ("ExitTimeOut", "integer", "5"),
            ("LimitLoadToSessionType", "string", "Aqua"),
        ] {
            let output = Command::new("/usr/bin/plutil")
                .args(["-extract", key, "raw", "-expect", kind, "-n", "-o", "-"])
                .arg(&path)
                .output()
                .unwrap();
            assert!(output.status.success(), "{key}: {output:?}");
            assert_eq!(String::from_utf8(output.stdout).unwrap(), expected, "{key}");
        }
        assert_eq!(agent.saved_config().unwrap().as_deref(), Some(config));
        assert!(
            Command::new("/usr/bin/plutil")
                .args(["-convert", "binary1"])
                .arg(&path)
                .status()
                .unwrap()
                .success()
        );
        assert_eq!(agent.saved_config().unwrap().as_deref(), Some(config));
        fs::write(&path, "invalid plist").unwrap();
        assert!(agent.saved_config().is_err());
        fs::remove_dir_all(dir).unwrap();
    }
}
