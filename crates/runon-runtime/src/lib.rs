//! Event-driven action execution on a serial `DispatchQueue`.
mod matching;
mod process;
mod runtime;

pub use process::OUTPUT_LIMIT;
pub use runtime::{Report, Runtime};
