use log::{Level, LevelFilter, Log, Metadata, Record};
use std::{
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
        mpsc::{self, SyncSender},
    },
    time::Duration,
};

const QUEUE_LIMIT: usize = 16;
const FLUSH_TIMEOUT: Duration = Duration::from_millis(100);

enum Message {
    Record(Level, String),
    Flush(SyncSender<()>),
}

struct Logger {
    tx: SyncSender<Message>,
    dropped: Arc<AtomicUsize>,
}
impl Log for Logger {
    fn enabled(&self, metadata: &Metadata<'_>) -> bool {
        metadata.level() <= Level::Info
    }
    fn log(&self, record: &Record<'_>) {
        if !self.enabled(record.metadata()) {
            return;
        }
        if self
            .tx
            .try_send(Message::Record(
                record.level(),
                format!("{}", record.args()),
            ))
            .is_err()
        {
            self.dropped.fetch_add(1, Ordering::Relaxed);
        }
    }
    fn flush(&self) {
        let (tx, rx) = mpsc::sync_channel(1);
        if self.tx.try_send(Message::Flush(tx)).is_ok() {
            // Shutdown must also finish when a log consumer stops reading.
            let _ = rx.recv_timeout(FLUSH_TIMEOUT);
        }
    }
}

struct Sink {
    system: Option<oslog::OsLog>,
}

impl Sink {
    fn write(&self, level: Level, message: &str) {
        if let Some(system) = &self.system {
            // Child output can contain NUL, which the C logging API cannot represent.
            let message = message.replace('\0', "\\0");
            // os_log has a 1024-byte dynamic-content limit. Chunk long tails
            // without breaking UTF-8 so the end of the diagnostic survives.
            for chunk in chunks(&message) {
                match level {
                    Level::Error | Level::Warn => system.error(chunk),
                    _ => system.default(chunk),
                }
            }
        } else {
            use std::io::Write;
            let _ = writeln!(std::io::stderr(), "{level} {message}");
        }
    }
}

fn chunks(mut message: &str) -> impl Iterator<Item = &str> {
    std::iter::from_fn(move || {
        if message.is_empty() {
            return None;
        }
        let (chunk, rest) = message.split_at(message.floor_char_boundary(900));
        message = rest;
        Some(chunk)
    })
}

pub fn init(service: bool) -> Result<(), String> {
    let (tx, rx) = mpsc::sync_channel(QUEUE_LIMIT);
    let dropped = Arc::new(AtomicUsize::new(0));
    let lost = dropped.clone();
    std::thread::Builder::new()
        .name("runon-logs".into())
        .spawn(move || {
            let sink = Sink {
                system: service.then(|| oslog::OsLog::new(runon_macos::APP_ID, "daemon")),
            };
            for message in rx {
                let count = lost.swap(0, Ordering::Relaxed);
                if count != 0 {
                    sink.write(
                        Level::Warn,
                        &format!("dropped {count} log records: log queue full"),
                    );
                }
                match message {
                    Message::Record(level, text) => sink.write(level, &text),
                    Message::Flush(done) => {
                        let _ = done.send(());
                    }
                }
            }
        })
        .map_err(|e| format!("starting log worker: {e}"))?;
    let logger = Box::new(Logger { tx, dropped });
    log::set_logger(Box::leak(logger)).map_err(|e| e.to_string())?;
    log::set_max_level(LevelFilter::Info);
    Ok(())
}

pub fn report(report: runon_runtime::Report) {
    use runon_runtime::Report;
    match report {
        Report::Started { name, .. } => log::info!("action '{name}' started"),
        Report::Finished {
            name,
            success,
            reason,
            stdout,
            stderr,
        } => {
            if success {
                log::info!("action '{name}' completed");
            } else {
                log::error!("action '{name}' failed: {reason}");
                if !stdout.is_empty() {
                    log::error!("{name} stdout (tail): {stdout}");
                }
                if !stderr.is_empty() {
                    log::error!("{name} stderr (tail): {stderr}");
                }
            }
        }
        Report::Stopped => {
            log::info!("daemon stopped");
            runon_macos::native::stop_main();
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn long_unicode_diagnostics_survive_system_log_limits() {
        let message = format!("{} END", "Диагностика 🔊".repeat(5_000));
        let chunks: Vec<_> = super::chunks(&message).collect();
        assert!(chunks.iter().all(|s| !s.is_empty() && s.len() <= 900));
        assert_eq!(chunks.concat(), message);
        assert!(chunks.last().unwrap().ends_with(" END"));
    }
}
