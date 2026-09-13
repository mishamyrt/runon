use crate::native::{Source, SourceKind};
use dispatch2::DispatchQueue;
use plist::{Dictionary, Value};
use runon_core::config::{self, COMMAND_PATH, Config};
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

impl LaunchAgent {
    pub fn current() -> Result<Self, String> {
        Ok(Self::new(
            crate::APP_ID.into(),
            config::home()?,
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
    fn domain(&self) -> String {
        format!("gui/{}", unsafe { libc::geteuid() })
    }
    fn target(&self) -> String {
        format!("{}/{}", self.domain(), self.label)
    }
    pub fn path(&self) -> PathBuf {
        self.home
            .join("Library/LaunchAgents")
            .join(format!("{}.plist", self.label))
    }

    fn launchctl(&self, args: &[&str]) -> Result<Output, String> {
        Command::new("/bin/launchctl")
            .args(args)
            .output()
            .map_err(|e| format!("launchctl: {e}"))
    }

    fn checked(&self, args: &[&str]) -> Result<(), String> {
        let out = self.launchctl(args)?;
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
        let output = self.launchctl(&["print", &self.target()])?;
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
            Source::new(SourceKind::Process, pid as usize, &queue, move || {
                let _ = tx.try_send(());
            })
        });
        self.checked(&["bootout", &self.target()])?;
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
        let value = Value::from_file(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        let args = value
            .as_dictionary()
            .and_then(|d| d.get("ProgramArguments"))
            .and_then(Value::as_array)
            .ok_or("agent has no ProgramArguments")?;
        for pair in args.windows(2) {
            if matches!(pair[0].as_string(), Some("-c" | "--config")) {
                return pair[1]
                    .as_string()
                    .map(PathBuf::from)
                    .map(Some)
                    .ok_or_else(|| "agent config path must be a string".into());
            }
        }
        Ok(None)
    }

    fn document(&self, path: &Path) -> Result<Value, String> {
        let binary = self.binary.to_str().ok_or("binary path must be UTF-8")?;
        let path = path.to_str().ok_or("config path must be UTF-8")?;
        let home = self.home.to_str().ok_or("home path must be UTF-8")?;
        let mut dict = Dictionary::new();
        dict.insert("Label".into(), self.label.clone().into());
        dict.insert(
            "ProgramArguments".into(),
            Value::Array(
                [binary, "run", "--service", "-c", path]
                    .into_iter()
                    .map(|s| s.into())
                    .collect(),
            ),
        );
        dict.insert("RunAtLoad".into(), true.into());
        let mut keepalive = Dictionary::new();
        keepalive.insert("SuccessfulExit".into(), false.into());
        dict.insert("KeepAlive".into(), Value::Dictionary(keepalive));
        dict.insert("ThrottleInterval".into(), 10u64.into());
        dict.insert("ExitTimeOut".into(), 5u64.into());
        dict.insert("LimitLoadToSessionType".into(), "Aqua".into());
        dict.insert("WorkingDirectory".into(), home.into());
        let mut env = Dictionary::new();
        env.insert("HOME".into(), home.into());
        env.insert("PATH".into(), COMMAND_PATH.into());
        dict.insert("EnvironmentVariables".into(), Value::Dictionary(env));
        Ok(Value::Dictionary(dict))
    }

    pub fn start(&self, path: Option<&Path>, restart: bool) -> Result<(), String> {
        let path = match path {
            Some(p) => p.to_path_buf(),
            None => self
                .saved_config()?
                .map(Ok)
                .unwrap_or_else(config::default_path)?,
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
            document
                .to_writer_xml(&mut file)
                .map_err(|e| e.to_string())?;
            file.flush()
                .and_then(|()| file.sync_all())
                .map_err(|e| e.to_string())?;
            if let Some(inspection) = &loaded {
                self.unload(inspection)?;
            }
            fs::rename(&staging, &agent).map_err(|e| e.to_string())?;
            self.checked(&[
                "bootstrap",
                &self.domain(),
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
        let agent = LaunchAgent::new(
            "co.myrt.runon.test".into(),
            PathBuf::from("/tmp/a & b"),
            PathBuf::from("/tmp/a & b/runon"),
        );
        let doc = agent.document(Path::new("/tmp/config <new>.kdl")).unwrap();
        let mut data = Vec::new();
        doc.to_writer_xml(&mut data).unwrap();
        let back = Value::from_reader(std::io::Cursor::new(data)).unwrap();
        assert_eq!(back, doc);
        let args = back.as_dictionary().unwrap()["ProgramArguments"]
            .as_array()
            .unwrap();
        assert_eq!(args[4].as_string(), Some("/tmp/config <new>.kdl"));
    }
}
