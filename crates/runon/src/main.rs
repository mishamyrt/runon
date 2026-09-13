mod cli;
mod logging;

fn main() {
    if let Err(error) = cli::run() {
        if log::max_level() == log::LevelFilter::Off {
            eprintln!("runon: {error}");
        } else {
            log::error!("{error}");
        }
        std::process::exit(1);
    }
}
