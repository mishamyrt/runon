mod cli;
mod logging;

#[allow(clippy::print_stderr)] // Startup failures may precede logger initialization.
fn main() -> std::process::ExitCode {
    if let Err(error) = cli::run() {
        if log::max_level() == log::LevelFilter::Off {
            eprintln!("runon: {error}");
        } else {
            log::error!("{error}");
        }
        log::logger().flush();
        return std::process::ExitCode::FAILURE;
    }
    log::logger().flush();
    std::process::ExitCode::SUCCESS
}
