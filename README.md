<h1 align="center">
    <img src="./docs/logo.svg" width="200" alt="RunOn logo" /><br>
    RunOn <br/>
</h1>
<p align="center">
    Runs commands on macOS events<br/><br/>
    <a href="https://github.com/mishamyrt/runon/actions/workflows/qa.yml">
        <img src="https://github.com/mishamyrt/runon/actions/workflows/qa.yml/badge.svg" alt="Quality Assurance badge" />
    </a>
    <a href="https://github.com/mishamyrt/runon/releases/latest">
        <img src="https://img.shields.io/github/v/tag/mishamyrt/runon?label=version" alt="Version badge" />
    </a><br/><br/>
</p>

RunOn runs commands when your Mac's displays, audio devices, applications, lock state or power source change, or when it wakes from sleep.

## Install

Requires macOS 15 or newer on Apple Silicon or Intel. Install a published release:

```sh
brew install mishamyrt/tap/runon
```

Homebrew installs the release binary for your architecture and verifies its SHA-256. Installation does not start the service.

To update, use `brew upgrade runon`, then `runon check && brew services restart runon`. For source builds and development, see [CONTRIBUTING](https://github.com/mishamyrt/runon/blob/main/CONTRIBUTING.md).

## Quick start

Create the configuration directory:

```sh
mkdir -p "$HOME/.config/runon"
```

Save this as `config.kdl` in that directory. This example speaks after your Mac wakes; replace the `exec` line with your own command:

```kdl
action after-wake {
    on system.wake
    debounce "500ms"
    exec "/usr/bin/say" "Welcome back"
}
```

Validate and test in the foreground:

```sh
runon check
runon
```

Stop with Ctrl-C, then run `runon check && brew services start runon` to enable the background service and start it at login. After editing the config, use `runon check && brew services restart runon` to apply changes. Run these commands without `sudo`, in your logged-in macOS GUI session.

If your terminal sets `XDG_CONFIG_HOME`, pass `-c "$HOME/.config/runon/config.kdl"` to both `runon` and `runon check` in these steps so they use the same configuration as the service.

Run `runon events` and trigger a device or app change to discover selectors you can paste into an action. It observes events without running actions. More examples: [desk and power actions](examples/config.kdl), [editor actions](examples/editors.kdl).

## Configuration

The default path is `~/.config/runon/config.kdl`, or `$XDG_CONFIG_HOME/runon/config.kdl` when `XDG_CONFIG_HOME` is nonempty (it must be absolute). Use `runon -c PATH` or `runon check -c PATH` to select another file. The Homebrew service does not inherit your terminal's environment; use `~/.config/runon/config.kdl` for the service, or make that path a symlink to your configuration. Validate the same file before starting or restarting the service.

Configuration uses **KDL 2**: separate nodes with a newline or `;`; quote strings containing spaces. Comments and multiline raw strings are supported. Old YAML configurations must be rewritten; changes require a restart.

| Setting | Where | Meaning / default |
| --- | --- | --- |
| `max-parallel 4` | Top level | Maximum simultaneously running groups; positive integer, default `4` |
| `shell-path "/bin/zsh"` | Top level or group | Executable for `shell` steps; default `/bin/sh`, with group settings overriding the global value |
| `group NAME { … }` | Top level | Share execution order, debounce and shell settings between actions |
| `action NAME group=NAME { … }` | Top level | Define an action; omit `group` for an independent group |
| `on EVENT filter=value` | Action | Match an event; at least one `on` is required |
| `debounce "500ms"` | Group or ungrouped action | Wait for this quiet period after the latest matching event; default `0ms` |
| `timeout "30s"` | Action | Time limit for all steps together, starting when the action runs; default `30s` |
| `exec "program" "argument"` | Action | Run a program with literal string arguments, without a shell |
| `shell "script"` | Action | Run one script string through the selected shell with `-c` |

Every action needs a unique, nonempty name, at least one `on` and at least one `exec` or `shell` step. Group names must also be unique and nonempty; groups may be declared before or after their actions. An action with `group=NAME` must use that group's `debounce`, not declare its own.

Durations are unsigned integer strings ending in `ms`, `s` or `m`. Timeout must be positive; debounce may be zero. Unknown settings, invalid values, duplicate single-value settings and undeclared groups are errors. `runon check` reports the file, line and column without executing commands.

### Events and filters

| Event | Optional filters |
| --- | --- |
| `screen.connected`, `screen.disconnected` | `name` (string), `id` (unsigned 32-bit integer) |
| `screen.locked`, `screen.unlocked` | None |
| `audio.connected`, `audio.disconnected` | `name`, `uid` (strings) |
| `app.activated`, `app.deactivated`, `app.launched`, `app.terminated` | `bundle-id`, `name` (strings) |
| `system.wake` | None |
| `power.changed` | `source`: `ac`, `battery`, `ups` |

Multiple `on` lines are **OR**; filters on one line are **AND**. Values match exactly, including case; missing fields do not match a filter. Without filters, any event of that kind matches. Each event selects a matching action only once.

For example, `on app.activated bundle-id="com.apple.TextEdit"` matches TextEdit, and `on power.changed source=battery` matches switching to battery power.

Display IDs can change between sessions; filter by `name` for a persistent rule. Use audio `uid` to distinguish devices with identical names. Audio events cover both input and output devices; disconnect events retain the last known metadata.

Startup does not emit connection or power events for the current state. Wake refreshes device and power state to detect changes missed during sleep. There is no sleep event. App launch/termination excludes background and `LSUIElement` apps; lock/unlock relies on undocumented macOS notifications. See [event validation and limitations](docs/validation.md#hardware-acceptance-still-required).

### Groups and debounce

Actions without `group` run independently. Use a shared group for actions that must not overlap:

```kdl
group desk {
    debounce "500ms"
}

action display-connected group=desk {
    on screen.connected name="My Display"
    exec "/usr/bin/say" "Display connected"
}

action display-disconnected group=desk {
    on screen.disconnected name="My Display"
    exec "/usr/bin/say" "Display disconnected"
}
```

Within each group, an event forms a batch of matching actions in configuration order. A group runs one batch and keeps at most one pending batch. A new matching event replaces the pending batch in full and resets debounce; the running batch finishes unchanged. Ready groups run up to `max-parallel` at once.

Steps run in order. Failure or timeout skips the rest of that action, then proceeds to the next action in the batch. Timeouts exclude time spent asleep. Commands must finish their work before exiting; background child processes are cleaned up after each step.

### Shell and environment

`exec` treats `$HOME`, `~`, globs and pipes literally. Use `shell` for expansion or pipelines, for example `shell "echo $HOME"`. With sh-compatible shells, use `set -e` to stop a multiline script on failure, or split it into separate steps.

`shell-path` takes one executable path or name, without arguments. It must support `-c`; RunOn does not request an interactive or login shell.

Commands run in your home directory with no standard input. They inherit the foreground terminal's environment or launchd's environment in service mode, but RunOn always sets this command search path:

```text
/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin
```

Use absolute paths for programs outside these directories, including scripts in `~/.local/bin`. Successful command output is discarded; failures log the reason and the last 64 KiB of each output stream.

## CLI and service

| Command | Purpose |
| --- | --- |
| `runon [-c PATH]` | Run in the foreground; Ctrl-C to stop |
| `runon check [-c PATH]` | Validate configuration without running actions |
| `runon events` | Print observed events as KDL selectors; Ctrl-C to stop |
| `runon logs` | Follow Homebrew's `var/log/runon.log` with `tail -F` |
| `runon --help` | Show usage and options |
| `brew services start runon` | Start the service and enable startup at login |
| `brew services restart runon` | Restart the service to apply config or binary changes |
| `brew services stop runon` | Stop the service and disable startup at login |
| `brew services info runon` | Show service status |

`--config` is an alias for `-c`; `--help` and `--version` are also available.

The service runs in your logged-in GUI session. Homebrew manages the LaunchAgent, binary path and startup at login. It restarts RunOn after an unsuccessful exit. Homebrew does not validate KDL before restarting: use `runon check && brew services restart runon` so an invalid config does not stop a running service. If your terminal sets `XDG_CONFIG_HOME`, explicitly check the service file with `runon check -c "$HOME/.config/runon/config.kdl"` instead.

RunOn always writes diagnostics to stderr. The Homebrew service redirects stderr to its `var/log/runon.log`: `/opt/homebrew/var/log/runon.log` on Apple Silicon or `/usr/local/var/log/runon.log` on Intel. The file is created when the service starts. Use `runon logs` to follow it with `tail -F`; it uses `HOMEBREW_PREFIX` when set, otherwise the standard prefix for the binary's architecture. Foreground runs write to your terminal and do not append to this file.

Log rotation is not configured. If you rotate or rename the log file, restart the service to reopen its output file.
