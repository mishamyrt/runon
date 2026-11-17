//! Native subscriptions, Dispatch ownership and system paths.
#[cfg(not(all(
    target_os = "macos",
    any(target_arch = "aarch64", target_arch = "x86_64")
)))]
compile_error!("RunOn requires macOS 15 or later on Apple Silicon or Intel");

pub mod native;
pub mod paths;
mod sources;

pub use sources::Sources;
