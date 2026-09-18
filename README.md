<p align="center">
    <img src="./docs/logo.svg" width="200" alt="RunOn logo" />
</p>

[![Quality Assurance](https://github.com/mishamyrt/runon/actions/workflows/qa.yaml/badge.svg)](https://github.com/mishamyrt/runon/actions/workflows/qa.yaml)

RunOn runs commands when your Mac's displays, audio devices, applications, lock state or power source change.

## Install

From a published 2.x release:

```sh
curl -fsSL https://raw.githubusercontent.com/mishamyrt/runon/main/scripts/install.sh | bash
```

The installer selects the archive for your Mac's architecture, checks its SHA-256 and places `runon` in `~/.local/bin`. Add that directory to your interactive PATH. Installation does not start the service. For a particular release, download the installer and pass a tag such as `v2.0.0`.

To build this checkout, install Rust with rustup and the Xcode Command Line Tools, then:

```sh
make install
```

Cargo uses the pinned toolchain in `rust-toolchain.toml`. No `sudo` is needed. Version 2.0.0 is prepared in this checkout; downloading it requires a published release.

## Quick start

Create `$XDG_CONFIG_HOME/runon/config.kdl`, or `~/.config/runon/config.kdl` if `XDG_CONFIG_HOME` is unset:

```kdl
max-parallel 4

group desk {
    debounce "500ms"
}

action desk-on group=desk {
    on screen.connected name="Mi 27 NU"
    timeout "5s"
    exec "myrt_desk" "on"
}

action desk-off group=desk {
    on screen.disconnected name="Mi 27 NU"
    exec "myrt_desk" "off"
}

action after-wake {
    on system.wake
    debounce "500ms"
    exec "setup_audio"
}

action battery-mode {
    on power.changed source=battery
    shell #"""
        desk_lights off
        lunar set 60
        """#
}
```

Then validate and run:

```sh
runon check
runon run      # foreground, Ctrl-C to stop
runon start    # install and load the user LaunchAgent
```

Use `runon events` to discover event names and device identifiers. It observes all sources and prints selectors that can be pasted into an action:

```kdl
on screen.connected id=3 name="Mi 27 NU"
on audio.connected name="USB Audio" uid="AppleUSBAudioEngine:example"
on app.activated bundle-id="com.apple.TextEdit" name="TextEdit"
on power.changed source="battery"
```

These are illustrative identifiers. Display IDs belong to the current session. Omit `id` when a rule should match a display by name across sessions.

## Configuration reference

KDL 2 supports bare string values such as `desk`, quoted Unicode strings, comments and multiline raw strings. Separate nodes with a newline or `;`. RunOn parses KDL once, validates it, builds typed rules and releases the parser document. Changes take effect on restart.

| Node | Meaning | Default |
| --- | --- | --- |
| `max-parallel 4` | Maximum number of executing groups; positive integer | `4` |
| `shell-path "/bin/zsh"` | Shell executable; at the top level or inside a group | `/bin/sh`; groups inherit the global setting |
| `group NAME { … }` | Named group; names must be unique and nonempty | — |
| `debounce "500ms"` | Quiet period after the latest matching event; inside a group or an action without `group` | `0ms` |
| `action NAME group=NAME { … }` | Named action, optionally assigned to a declared group | Private group |
| `on EVENT filter=value` | Event selector; at least one per action | — |
| `timeout "30s"` | Deadline for the entire action, including every step | `30s` |
| `exec "program" "argument"` | Execute a program with literal string arguments | — |
| `shell "script"` | Execute one string through the selected shell with `-c` | — |

An action needs a unique, nonempty name, at least one selector and at least one step. Group declarations may follow actions that reference them. Durations are unsigned integer strings ending in `ms`, `s` or `m`; timeout must be positive, debounce can be zero. Out-of-range durations are rejected.

Set `debounce` directly inside an action to debounce it independently. Actions with `group=NAME` use the group's debounce; combining `group` with an action-level `debounce` is an error.

Multiple `on` nodes are **OR**. Properties on one selector are **AND**. Values compare exactly, including case. A missing event field never satisfies a filter. A selector without properties matches any event of that kind. One event selects each matching action once, even when several of its selectors match.

```kdl
action editors {
    on app.activated bundle-id="com.apple.TextEdit"
    on app.activated bundle-id="com.microsoft.VSCode"
    exec "setup_keyboard"
    exec "setup_audio" "editing"
}
```

Unknown nodes, events, properties, invalid types, duplicate singleton settings and undefined groups are errors reported with the config path, line and column. Type annotations are not part of the schema. KDL 1, YAML, variable interpolation, state conditions, plugins and live reload are not supported.

### Events

| Event | Optional filters |
| --- | --- |
| `screen.connected`, `screen.disconnected` | `name` (string), `id` (unsigned 32-bit integer) |
| `screen.locked`, `screen.unlocked` | None |
| `audio.connected`, `audio.disconnected` | `name`, `uid` (strings) |
| `app.activated`, `app.deactivated`, `app.launched`, `app.terminated` | `bundle-id`, `name` (strings) |
| `system.wake` | None |
| `power.changed` | `source`: `ac`, `battery`, `ups` |

Display identity is the system display ID; audio identity is the device UID. Devices with identical names remain distinct. Disconnect events use the last saved metadata. Audio covers both input and output devices. Application fields are omitted when the OS does not provide them.

Application launch/termination follow NSWorkspace notifications: macOS excludes background applications and applications declaring `LSUIElement` from these notifications. See [Apple's delivery rules](https://developer.apple.com/documentation/appkit/nsworkspace/didlaunchapplicationnotification).

Startup captures a baseline without generating connection or power events. Duplicate device/power notifications do not create duplicate changes. Wake notifications also refresh subscribed device and power snapshots, detecting changes missed while asleep. There is no sleep event.

Lock/unlock delivery uses the distributed notifications `com.apple.screenIsLocked` and `com.apple.screenIsUnlocked`. These names are undocumented by Apple; delivery must be checked on each supported OS version. See [validation status](docs/validation.md).

### Scheduling

All matching actions in the same group form a batch in config order. A group runs one batch and holds at most one pending batch. Every subsequent matching event replaces that pending batch in full; it does not interrupt the current batch.

Debounce starts again on the latest matching event. A pending batch becomes eligible after its debounce expires and the current batch finishes. Eligible groups run in readiness order, up to `max-parallel` at once. A zero-debounce replacement keeps an already-waiting group's position. Actions without an explicit group each get an independent group. Pending storage is bounded by the number of configured groups, even during an event flood.

Steps run sequentially. Failure or timeout skips the rest of that action, then continues with the next action in its batch. It does not stop the daemon.

### Commands and output

`exec` never invokes a shell. `$HOME`, `~`, globs and pipes remain literal arguments. Use `shell` for shell expansion or pipelines. Shell steps run as `SHELL -c SCRIPT`, using `/bin/sh` by default. With sh-compatible shells, include `set -e` to stop a multiline script on failure, or use separate steps.

Set `shell-path` globally to choose another shell. A group's `shell-path` overrides the global setting for all its actions; other groups and actions without a group inherit the global setting. Declaration order does not matter. The value is one nonempty executable path or name, without additional arguments, and the executable must support `-c`. RunOn does not request login or interactive mode; startup files follow the selected shell's rules.

```kdl
shell-path "/bin/zsh"

group setup {
    shell-path "/bin/bash"
}

action after-wake group=setup {
    on system.wake
    shell "setup_audio && setup_keyboard"
}
```

Commands run in the user's home directory, with inherited environment variables and this explicit PATH in both foreground and service mode:

```text
/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin
```

Standard input is `/dev/null`. Both output streams are drained without blocking, retaining only the last 64 KiB of each stream of the current step. Failures log the exit status/reason and retained output; successful command output is discarded.

Each step has its own process group. Timeout or daemon shutdown sends SIGTERM to the group, allows two seconds for cleanup, then sends SIGKILL before reaping the leader and releasing its slot. The grace period also covers descendants when the leader exits first. A normally completed step cleans up leftover descendants before the next step starts; commands should not leave background jobs running. Deadlines use monotonic awake time, excluding time spent asleep.

## CLI and service

```text
runon run [-c PATH]
runon check [-c PATH]
runon events
runon start [-c PATH]
runon stop
runon restart [-c PATH]
runon status
runon logs
```

Without a subcommand, RunOn prints service status. `--config` is an alias for `-c`. `--help` and `--version` are also available. `check` only reads and validates the configuration.

`start` validates the config, writes `~/Library/LaunchAgents/co.myrt.runon.plist` with absolute binary/config paths, and loads it in the current user's GUI session. Starting an already loaded service leaves its settings and process unchanged. The LaunchAgent starts at login and launchd restarts it after an unsuccessful exit, with a ten-second throttle. Keep the installed binary at its configured path.

`restart` validates and prepares the new config before unloading the current service. Without `-c`, it preserves the config path stored in the installed LaunchAgent. An invalid config does not stop the running daemon. `stop` unloads the agent and retains its settings; `start` can load them again. `status` never modifies files or launchd state. Remove the saved plist after stopping if you also want to remove login startup.

Foreground diagnostics go to stderr. Service diagnostics use Unified Logging with subsystem `co.myrt.runon`; `logs` opens `/usr/bin/log stream` filtered to that subsystem. Commands inherit the foreground shell's environment when run interactively and launchd's environment as a service, with the same explicit HOME working directory and PATH policy.

Diagnostic writes run on a separate thread with a queue of 16 records. If the log destination stalls and the queue fills, new records are dropped; a warning reports the count when writing resumes. Process timeouts and shutdown continue independently. On exit, RunOn waits at most 100 ms for queued diagnostics; remaining records may be lost.

To update an installed binary, rerun the installer or `make install`, then `runon restart`. RunOn 2 does not migrate old YAML files; create and validate a KDL configuration first.

## Development and release

```sh
make check          # fmt, Clippy with -D warnings, tests
make build-release  # release binary, ARM64 archive, SHA256SUMS, release notes
make measure        # five-minute idle measurement + latency/event-flood benchmark
```

The Cargo workspace has five crates. Shared dependency versions, package metadata, lint policy and the release profile live in the root `Cargo.toml`; every crate opts into the workspace lints, and all crates share one `Cargo.lock` and `target/` directory.

| Crate | Responsibility | Internal dependencies |
| --- | --- | --- |
| [`runon-core`](crates/runon-core) | Typed events and clock-controlled scheduling rules; no platform APIs or unsafe code | None |
| [`runon-config`](crates/runon-config) | Configuration structures, KDL loading, parsing, validation and event-selector formatting | `runon-core` |
| [`runon-macos`](crates/runon-macos) | Native event subscriptions, Dispatch/RunLoop/signal ownership, system paths and LaunchAgent integration | `runon-config`, `runon-core` |
| [`runon-runtime`](crates/runon-runtime) | Event matching, action execution, process groups, output capture, deadlines and the serial scheduler queue | `runon-config`, `runon-core`, `runon-macos` |
| [`runon`](crates/runon) | Binary entry point, CLI commands, logging and wiring the components together | All four libraries |

The libraries never depend on the CLI. Native sources emit typed events through a callback and do not depend on the runtime. Process supervision stays private to `runon-runtime`; its public entry points are `Runtime` and `Report`. The scheduler receives the parallelism limit and group debounce durations without depending on the configuration parser; `runon-core` has no external dependencies. `runon-config` reads an explicitly supplied path and formats observed events with `format_selector`; `runon-macos::paths` owns home/config path discovery and the command search path. The core and configuration crates can be built and tested independently of macOS with `cargo test --locked -p runon-core -p runon-config`.

The main thread runs AppKit's event loop. `Runtime::submit` matches events and updates the bounded scheduler under a short mutex; the scheduler alone owns pending replacement and readiness order. A serial DispatchQueue selects ready groups and drives process lifecycle through native data, process, pipe, timer and signal sources. Report callbacks run on that queue outside state locks and must be nonblocking; the CLI sends diagnostic writes to its log worker. There is no async runtime, periodic process check or idle timer. Only configured event sources are subscribed, plus wake notifications required for refreshing device/power snapshots.

`cargo test --workspace` covers parsing, matching, scheduling, subprocess behavior and CLI behavior. Tests and executable examples live with their owning crates; user-facing KDL examples remain in the root `examples/` directory. The LaunchAgent integration test lives in `runon` because it exercises the assembled binary. It is explicit because it requires a GUI session and installs a temporary agent with a unique label:

```sh
cargo test --locked -p runon --test service -- --ignored --nocapture
cargo run --locked -p runon-macos --example native_smoke
```

Hardware event delivery needs a real logged-in Mac. [Validation and measurements](docs/validation.md) describe the reproducible checks, measured results and outstanding hardware checks.

Release builds use size optimization, LTO, one codegen unit, symbol stripping and abort-on-panic, with a macOS 15.0 deployment target. Cargo.lock and the Rust toolchain are pinned. CI runs checks and packages on macOS 15 for both ARM64 and Intel, plus macOS 26 ARM64. Releases build natively on macOS 15 for each architecture. A pushed `v*` tag must match Cargo's version and publishes `runon-macos-arm64.tar.gz`, `runon-macos-x86_64.tar.gz` and a combined `SHA256SUMS`; manually running the workflow only builds artifacts. Local packaging builds the host architecture and does not publish a release.
