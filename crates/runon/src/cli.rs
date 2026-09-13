use crate::logging;
use runon_core::{
    config::{self, Config},
    event::Kind,
};
use runon_macos::{
    LaunchAgent, Sources,
    native::{RunLoop, Signals},
};
use runon_runtime::Runtime;
use std::{path::PathBuf, sync::Arc};

const HELP: &str = "RunOn 2 — run commands on macOS events\n\nUsage: runon <command> [-c PATH]\n\nCommands:\n  run       Run in the foreground\n  check     Validate KDL configuration\n  events    Print observed events as KDL selectors\n  start     Install and start the user LaunchAgent\n  stop      Stop the LaunchAgent\n  restart   Validate and restart the LaunchAgent\n  status    Print service status (default)\n  logs      Follow the service's system log\n\nOptions:\n  -c, --config PATH   Config for run/check/start/restart\n  -h, --help          Show this help\n  --version           Print version\n";

pub fn run() -> Result<(), String> {
    use lexopt::prelude::*;
    let mut parser = lexopt::Parser::from_env();
    let mut command = None;
    let mut path = None;
    let mut service = false;
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
            Long("service") if !service => service = true,
            Value(v) if command.is_none() => command = Some(v.string().map_err(|e| e.to_string())?),
            _ => return Err(arg.unexpected().to_string()),
        }
    }
    let command = command.as_deref().unwrap_or("status");
    if path.is_some() && !["run", "check", "start", "restart"].contains(&command) {
        return Err(format!("{command} does not accept --config"));
    }
    if service && command != "run" {
        return Err("--service is only valid with run".into());
    }
    logging::init(service)?;
    match command {
        "run" => {
            let path = path.map(Ok).unwrap_or_else(config::default_path)?;
            let config = Config::load(&path)?;
            let kinds = config.kinds();
            let runloop = RunLoop::new()?;
            let runtime = Runtime::new(config, logging::report)?;
            let stop = runtime.clone();
            let _signals = Signals::new(&runtime.queue(), Arc::new(move || stop.shutdown()))
                .map_err(|e| e.to_string())?;
            let out = runtime.clone();
            let _sources = objc2::rc::autoreleasepool(|_| {
                Sources::subscribe(kinds, Arc::new(move |e| out.submit(e)))
            })?;
            log::info!("ready; config={}", path.display());
            runloop.run();
            Ok(())
        }
        "events" => {
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
                        if writeln!(std::io::stdout(), "{}", e.selector()).is_err() {
                            runon_macos::native::stop_main();
                        }
                    }),
                )
            })?;
            log::info!("observing all sources");
            runloop.run();
            Ok(())
        }
        "check" => {
            let path = path.map(Ok).unwrap_or_else(config::default_path)?;
            let config = Config::load(&path)?;
            println!(
                "{}: valid ({} actions, {} groups)",
                path.display(),
                config.actions.len(),
                config.groups.len()
            );
            Ok(())
        }
        "start" | "restart" => {
            let agent = LaunchAgent::current()?;
            agent.start(path.as_deref(), command == "restart")?;
            println!("{}", agent.status()?);
            Ok(())
        }
        "stop" => {
            LaunchAgent::current()?.stop()?;
            println!("stopped");
            Ok(())
        }
        "status" => {
            println!("{}", LaunchAgent::current()?.status()?);
            Ok(())
        }
        "logs" => {
            use std::os::unix::process::CommandExt;
            Err(std::process::Command::new("/usr/bin/log")
                .args([
                    "stream",
                    "--style",
                    "compact",
                    "--level",
                    "info",
                    "--predicate",
                    &format!("subsystem == '{}'", runon_macos::APP_ID),
                ])
                .exec()
                .to_string())
        }
        _ => Err(format!("unknown command '{command}'\n{HELP}")),
    }
}
