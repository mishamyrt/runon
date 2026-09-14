//! Event-driven child supervision. A leader is reaped only after group cleanup,
//! so delayed signals can never target a recycled process-group identifier.
use dispatch2::DispatchQueue;
use runon_config::Action;
use runon_macos::{
    native::{Source, SourceKind},
    paths::COMMAND_PATH,
};
use std::{
    collections::VecDeque,
    fs::File,
    io::{self, Read},
    os::{
        fd::{AsRawFd, OwnedFd},
        unix::process::CommandExt,
    },
    path::Path,
    process::{Child, Command, Stdio},
    sync::Arc,
    time::{Duration, Instant},
};

pub const OUTPUT_LIMIT: usize = 64 * 1024;

struct Pipe {
    file: Arc<File>,
    watch: Option<Source>,
    tail: VecDeque<u8>,
}

impl Pipe {
    fn new(
        fd: OwnedFd,
        queue: &DispatchQueue,
        wake: Arc<dyn Fn() + Send + Sync>,
    ) -> io::Result<Self> {
        let file = Arc::new(File::from(fd));
        let raw = file.as_raw_fd();
        // SAFETY: raw belongs to this file, and flags are updated without losing existing flags.
        let flags = unsafe { libc::fcntl(raw, libc::F_GETFL) };
        if flags < 0 || unsafe { libc::fcntl(raw, libc::F_SETFL, flags | libc::O_NONBLOCK) } < 0 {
            return Err(io::Error::last_os_error());
        }
        let watch = Source::new(
            SourceKind::Read,
            usize::try_from(raw).unwrap(),
            queue,
            move || wake(),
        );
        watch.hold_file(file.clone());
        Ok(Self {
            file,
            watch: Some(watch),
            tail: VecDeque::new(),
        })
    }

    fn drain(&mut self) -> io::Result<()> {
        let mut buf = [0; 8192];
        // Bound each turn so an endlessly writing child cannot starve timeouts/signals.
        for _ in 0..32 {
            match (&*self.file).read(&mut buf) {
                Ok(0) => {
                    self.watch = None;
                    break;
                }
                Ok(n) => {
                    let excess = (self.tail.len() + n).saturating_sub(OUTPUT_LIMIT);
                    self.tail.drain(..excess);
                    self.tail.extend(&buf[..n]);
                }
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => break,
                Err(e) if e.kind() == io::ErrorKind::Interrupted => {}
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }

    fn text(&mut self) -> String {
        String::from_utf8_lossy(self.tail.make_contiguous()).into_owned()
    }
}

struct ChildState {
    child: Child,
    reaped: bool,
    _watch: Source,
    stdout: Pipe,
    stderr: Pipe,
}

impl ChildState {
    fn exited(&self) -> io::Result<bool> {
        let mut info: libc::siginfo_t = unsafe { std::mem::zeroed() };
        // SAFETY: valid output, our unreaped child; WNOWAIT prevents PID reuse.
        let code = unsafe {
            libc::waitid(
                libc::P_PID,
                self.child.id(),
                &raw mut info,
                libc::WEXITED | libc::WNOHANG | libc::WNOWAIT,
            )
        };
        if code != 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(unsafe { info.si_pid() } != 0)
    }

    fn signal(&self, signal: i32) {
        if !self.reaped {
            // SAFETY: the negative, unreaped child PID names only this command's group.
            let result = unsafe { libc::kill(-i32::try_from(self.child.id()).unwrap(), signal) };
            if result != 0 && io::Error::last_os_error().raw_os_error() != Some(libc::ESRCH) {
                log::error!(
                    "cannot signal process group {}: {}",
                    self.child.id(),
                    io::Error::last_os_error()
                );
            }
        }
    }
}

impl Drop for ChildState {
    fn drop(&mut self) {
        if !self.reaped {
            self.signal(libc::SIGKILL);
            let _ = self.child.wait();
        }
    }
}

pub struct Outcome {
    pub success: bool,
    pub reason: String,
    pub stdout: String,
    pub stderr: String,
}

pub struct Process {
    step: usize,
    child: Option<ChildState>,
    deadline: Duration,
    terminating: Option<(Duration, String)>,
    kill_sent: bool,
    started_at: Option<Instant>,
}

impl Process {
    pub fn new(timeout: Duration, now: Duration) -> Self {
        Self {
            step: 0,
            child: None,
            deadline: now.saturating_add(timeout),
            terminating: None,
            kill_sent: false,
            started_at: None,
        }
    }

    pub fn deadline(&self) -> Option<Duration> {
        (!self.kill_sent).then(|| self.terminating.as_ref().map_or(self.deadline, |(d, _)| *d))
    }

    pub fn started_at(&self) -> Option<Instant> {
        self.started_at
    }

    pub fn terminate(&mut self, now: Duration, reason: &str) {
        if self.terminating.is_none() {
            if let Some(child) = &self.child {
                child.signal(libc::SIGTERM);
            }
            self.terminating = Some((now.saturating_add(Duration::from_secs(2)), reason.into()));
        }
    }

    pub fn poll(
        &mut self,
        action: &Action,
        now: Duration,
        home: &Path,
        queue: &DispatchQueue,
        wake: Arc<dyn Fn() + Send + Sync>,
    ) -> Option<Outcome> {
        if self.terminating.is_none() && now >= self.deadline {
            self.terminate(now, "timeout");
        }
        if let Some(child) = &mut self.child
            && let Err(e) = child.stdout.drain().and_then(|()| child.stderr.drain())
        {
            self.terminate(now, &format!("reading output: {e}"));
        }
        if let Some(child) = &mut self.child {
            if let Some((until, _)) = &self.terminating {
                if now < *until {
                    return None;
                }
                if !self.kill_sent {
                    child.signal(libc::SIGKILL);
                    self.kill_sent = true;
                }
            }
            match child.exited() {
                Ok(false) => return None,
                Err(e) => {
                    self.terminate(now, &format!("observing process: {e}"));
                    return None;
                }
                Ok(true) => {}
            }
            // A completed step must not leave background descendants holding pipes or
            // changing state after a following step has started.
            child.signal(libc::SIGKILL);
            let status = child.child.wait();
            child.reaped = true;
            let _ = child.stdout.drain();
            let _ = child.stderr.drain();
            let success = status.as_ref().is_ok_and(std::process::ExitStatus::success)
                && self.terminating.is_none();
            if !success {
                return Some(Outcome {
                    success: false,
                    reason: self
                        .terminating
                        .as_ref()
                        .map(|(_, r)| r.clone())
                        .unwrap_or_else(|| match status {
                            Ok(s) => s.to_string(),
                            Err(e) => e.to_string(),
                        }),
                    stdout: child.stdout.text(),
                    stderr: child.stderr.text(),
                });
            }
            self.child = None;
            self.step += 1;
        }
        if let Some((_, reason)) = &self.terminating {
            return Some(Outcome {
                success: false,
                reason: reason.clone(),
                stdout: String::new(),
                stderr: String::new(),
            });
        }
        if self.step == action.steps.len() {
            return Some(Outcome {
                success: true,
                reason: "completed".into(),
                stdout: String::new(),
                stderr: String::new(),
            });
        }
        let args = &action.steps[self.step];
        let mut command = Command::new(&args[0]);
        command
            .args(&args[1..])
            .current_dir(home)
            .env("PATH", COMMAND_PATH)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .process_group(0);
        let spawn = (|| -> io::Result<ChildState> {
            self.started_at.get_or_insert_with(Instant::now);
            let mut child = command.spawn()?;
            let pid = child.id();
            let cb = wake.clone();
            let watch = Source::new(SourceKind::Process, pid as usize, queue, move || cb());
            let pipes = Pipe::new(
                OwnedFd::from(child.stdout.take().unwrap()),
                queue,
                wake.clone(),
            )
            .and_then(|out| {
                Pipe::new(
                    OwnedFd::from(child.stderr.take().unwrap()),
                    queue,
                    wake.clone(),
                )
                .map(|err| (out, err))
            });
            match pipes {
                Ok((stdout, stderr)) => Ok(ChildState {
                    child,
                    reaped: false,
                    _watch: watch,
                    stdout,
                    stderr,
                }),
                Err(e) => {
                    unsafe {
                        libc::kill(-i32::try_from(pid).unwrap(), libc::SIGKILL);
                    }
                    let _ = child.wait();
                    Err(e)
                }
            }
        })();
        match spawn {
            Ok(child) => {
                // Registration races with /usr/bin/true: recheck after activation.
                let exited = child.exited().unwrap_or(true);
                self.child = Some(child);
                if exited {
                    queue.exec_async(move || wake());
                }
                None
            }
            Err(e) => Some(Outcome {
                success: false,
                reason: format!("spawn: {e}"),
                stdout: String::new(),
                stderr: String::new(),
            }),
        }
    }
}
