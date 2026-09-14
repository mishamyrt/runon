//! Native subscriptions, Dispatch ownership and LaunchAgent integration.
#[cfg(not(all(target_os = "macos", target_arch = "aarch64")))]
compile_error!("RunOn requires macOS 26 or later on Apple Silicon");

pub mod native;
pub mod paths;
mod service;
mod sources;

pub use service::LaunchAgent;
pub use sources::Sources;

pub const APP_ID: &str = "co.myrt.runon";
