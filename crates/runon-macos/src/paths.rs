//! System paths and defaults shared by the CLI, service and runtime.
use std::path::PathBuf;

pub const COMMAND_PATH: &str = "/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin";

pub fn home() -> Result<PathBuf, String> {
    std::env::var_os("HOME")
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .ok_or_else(|| "HOME must be an absolute path".into())
}

pub fn default_config_path() -> Result<PathBuf, String> {
    let base = match std::env::var_os("XDG_CONFIG_HOME").filter(|s| !s.is_empty()) {
        Some(p) => {
            let path = PathBuf::from(p);
            if !path.is_absolute() {
                return Err("XDG_CONFIG_HOME must be an absolute path".into());
            }
            path
        }
        None => home()?.join(".config"),
    };
    Ok(base.join("runon/config.kdl"))
}
