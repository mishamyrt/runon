#![allow(clippy::print_stdout)] // CLI responses are the purpose of this module.
use crate::logging;
use runon_config::Config;
use runon_core::event::Kind;
use runon_macos::{
    Sources,
    native::{RunLoop, Signals},
    paths,
};
use runon_runtime::Runtime;
use std::{path::PathBuf, sync::Arc};

const HELP: &str = "RunOn 2 — run commands on macOS events\n\nUsage: runon [-c PATH]\n       runon <command>\n\nWith no command, run in the foreground; Ctrl-C to stop.\n\nCommands:\n  check     Validate KDL configuration; accepts -c PATH\n  events    Print observed events as KDL selectors\n  logs      Follow Homebrew's var/log/runon.log\n\nOptions:\n  -c, --config PATH   Configuration for foreground run or check\n  -h, --help          Show this help\n  --version          Print version\n";

pub fn run() -> Result<(), String> {
    use lexopt::prelude::*;
    let mut parser = lexopt::Parser::from_env();
    let mut command = None;
    let mut path = None;
    while let Some(arg) = parser.next().map_err(|e| e.to_string())? {
        match arg {
            Short('h') | Long("help") => {
                print!("{HELP}");
                return Ok(());
            }
            Long("version") => {
                println!("{}", env!("CARGO_PKG_VERSION"));
                return Ok(());
            }
            Short('c') | Long("config") if path.is_none() => {
                path = Some(PathBuf::from(parser.value().map_err(|e| e.to_string())?));
            }
            Value(v) if command.is_none() => command = Some(v.string().map_err(|e| e.to_string())?),
            _ => return Err(arg.unexpected().to_string()),
        }
    }
    let command = command.as_deref();
    if let Some(command) = command
        && path.is_some()
        && command != "check"
    {
        return Err(format!("{command} does not accept --config"));
    }
    logging::init()?;
    match command {
        None => {
            let path = path.map(Ok).unwrap_or_else(paths::default_config_path)?;
            let config = Config::load(&path)?;
            let kinds = config.kinds();
            let runloop = RunLoop::new()?;
            let runtime = Runtime::new(config, |report| logging::report(&report))?;
            let stop = runtime.clone();
            let _signals = Signals::new(&runtime.queue(), Arc::new(move || stop.shutdown()))
                .map_err(|e| e.to_string())?;
            let out = runtime;
            let _sources = objc2::rc::autoreleasepool(|_| {
                Sources::subscribe(kinds, Arc::new(move |e| out.submit(e)))
            })?;
            log::info!("ready; config={}", path.display());
            runloop.run();
            Ok(())
        }
        Some("events") => {
            let runloop = RunLoop::new()?;
            let _signals = Signals::new(
                dispatch2::DispatchQueue::main(),
                Arc::new(runon_macos::native::stop_main),
            )
            .map_err(|e| e.to_string())?;
            let _sources = objc2::rc::autoreleasepool(|_| {
                Sources::subscribe(
                    Kind::ALL.into(),
                    Arc::new(|e| {
                        use std::io::Write;
                        if writeln!(std::io::stdout(), "{}", runon_config::format_selector(&e))
                            .is_err()
                        {
                            runon_macos::native::stop_main();
                        }
                    }),
                )
            })?;
            log::info!("observing all sources");
            runloop.run();
            Ok(())
        }
        Some("check") => {
            let path = path.map(Ok).unwrap_or_else(paths::default_config_path)?;
            let config = Config::load(&path)?;
            println!(
                "{}: valid ({} actions, {} groups)",
                path.display(),
                config.actions.len(),
                config.groups.len()
            );
            Ok(())
        }
        Some("logs") => {
            use std::os::unix::process::CommandExt;
            Err(std::process::Command::new("/usr/bin/tail")
                .arg("-F")
                .arg(paths::default_log_path()?)
                .exec()
                .to_string())
        }
        Some(command) => Err(format!("unknown command '{command}'\n{HELP}")),
    }
}
