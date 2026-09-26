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
curl -fsSL https://raw.githubusercontent.com/mishamyrt/runon/main/scripts/install.sh | bash
```

The installer verifies the archive's SHA-256 and places `runon` in `~/.local/bin`. Add `export PATH="$HOME/.local/bin:$PATH"` to your shell configuration, then open a new terminal. Installation does not start the service.

To update, rerun the installer, then `runon restart`. For source builds and development, see [CONTRIBUTING](https://github.com/mishamyrt/runon/blob/main/CONTRIBUTING.md).

## Quick start

Create the configuration directory:

```sh
mkdir -p "${XDG_CONFIG_HOME:-$HOME/.config}/runon"
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
runon run
```

Stop with Ctrl-C, then run `runon start` to enable the background service and start it at login. After editing the config, use `runon restart` to apply changes.

Run `runon events` and trigger a device or app change to discover selectors you can paste into an action. It observes events without running actions. More examples: [desk and power actions](examples/config.kdl), [editor actions](examples/editors.kdl).

## Configuration

The default path is `~/.config/runon/config.kdl`, or `$XDG_CONFIG_HOME/runon/config.kdl` when `XDG_CONFIG_HOME` is nonempty (it must be absolute). Use `-c PATH` to select another file.

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
| `runon check [-c PATH]` | Validate configuration without running actions |
| `runon run [-c PATH]` | Run in the foreground; Ctrl-C to stop |
| `runon events` | Print observed events as KDL selectors; Ctrl-C to stop |
| `runon start [-c PATH]` | Validate, install and start the user LaunchAgent |
| `runon restart [-c PATH]` | Validate and restart with the new configuration |
| `runon stop` | Stop the service, retaining its settings |
| `runon status` | Show service status; also the default without a command |
| `runon logs` | Follow service diagnostics in macOS Unified Logging |

`--config` is an alias for `-c`; `--help` and `--version` are also available.

The service runs in your logged-in GUI session. `start` leaves an already loaded service unchanged. Both `start` and `restart` reuse the saved config path unless `-c` is given; an invalid config does not stop a running service. Foreground diagnostics go to stderr; service diagnostics use the `co.myrt.runon` logging subsystem.

The LaunchAgent is saved at `~/Library/LaunchAgents/co.myrt.runon.plist` with absolute binary and config paths. Keep the binary at that path. It starts at login and restarts after an unsuccessful exit. To remove login startup, run `runon stop`, then remove the plist.
