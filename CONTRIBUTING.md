# Contributing to RunOn

For installation, configuration and CLI usage, see the [README](README.md).

## Build and run

Use macOS 15 or newer on Apple Silicon or Intel, with the Xcode Command Line Tools and Rust installed through rustup. Cargo selects the toolchain pinned in `rust-toolchain.toml`, including rustfmt and Clippy.

Run these commands from the repository root:

```sh
cargo build --locked -p runon
target/debug/runon check -c examples/config.kdl
target/debug/runon -c examples/config.kdl
```

Edit the example or use your own configuration before running it: its commands are examples, not bundled programs. Foreground mode stops with Ctrl-C. Use `target/debug/runon events` to inspect native events without executing actions.

To install a release build in `~/.local/bin` without `sudo`:

```sh
make install
```

Add that directory to your shell's PATH. Installation does not start a service. Run `runon` in the foreground; Homebrew manages background operation of the Homebrew-installed binary, as described in the README.

## Checks

```sh
make check
```

This runs rustfmt, Clippy with warnings treated as errors and workspace tests. Use `make lint` or `make test` to run those parts separately. Tests cover parsing, matching, scheduling, subprocesses, logging and the CLI.

The platform-independent crates can also be tested outside macOS:

```sh
cargo test --locked -p runon-core -p runon-config
```

Native integration checks require a real logged-in Mac:

```sh
cargo run --locked -p runon-macos --example native_smoke
```

The native smoke test opens a temporary app without activating it and checks subscriptions, snapshots and event delivery. It does not replace physical display, audio, lock, wake or power testing. Follow the [hardware acceptance checklist](docs/validation.md#hardware-acceptance-still-required) for event-source changes. Validate the Homebrew service in a disposable GUI account: start, inspect status, restart, check diagnostics, stop and verify it is no longer registered for login startup.

CI runs `make check`, packaging and example-config validation on macOS 15 ARM64/Intel and macOS 26 ARM64. Local validation records and outstanding hardware checks are in [docs/validation.md](docs/validation.md).

## Code layout

| Crate | Responsibility | Internal dependencies |
| --- | --- | --- |
| [`runon-core`](crates/runon-core) | Typed events and clock-controlled scheduler; no platform APIs, unsafe code or external dependencies | None |
| [`runon-config`](crates/runon-config) | Config types, KDL loading and validation, event-selector formatting | `runon-core` |
| [`runon-macos`](crates/runon-macos) | Native event sources, Dispatch/RunLoop/signal ownership and system paths | `runon-core`; `runon-config` for examples only |
| [`runon-runtime`](crates/runon-runtime) | Matching, action execution, process supervision and scheduler queue | `runon-config`, `runon-core`, `runon-macos` |
| [`runon`](crates/runon) | CLI, logging and component wiring | All four libraries |

The root `Cargo.toml` owns shared dependency versions, package metadata, lint policy and release settings. Every crate uses workspace lints; the workspace shares `Cargo.lock` and `target/`. Tests and executable examples live with their owning crates; user-facing KDL examples live in `examples/`.

Keep dependencies flowing toward the libraries: they never depend on the CLI. Native sources emit typed events through a callback without depending on the runtime. `runon-config` loads an explicitly supplied path and formats events with `format_selector`; `runon-macos::paths` owns home/config discovery and the command search path. The scheduler takes the parallelism limit and debounce durations without depending on the config parser. Process supervision stays private to `runon-runtime`, whose public entry points are `Runtime` and `Report`.

## Runtime behavior

Configuration is parsed once into typed rules, then the KDL document is released. The main thread runs AppKit's event loop. Only configured sources are subscribed, plus wake notifications needed to refresh device and power snapshots. Snapshot comparisons suppress duplicate changes; native refresh requests are coalesced.

`Runtime::submit` matches events and updates the scheduler under a short mutex. The scheduler owns batch replacement and readiness order, keeping at most one pending batch per group. Ready groups run in readiness order; replacing a waiting zero-debounce batch preserves its position.

A serial DispatchQueue drives process lifecycle through native data, process, pipe, timer and signal sources, without an async runtime, polling or idle timer. Report callbacks execute on that queue outside state locks and must not block; the CLI forwards diagnostics to a log worker.

Each step owns a process group. Timeout or shutdown sends SIGTERM, allows two seconds for cleanup, then sends SIGKILL before reaping the leader and releasing its slot. The grace period also covers descendants when the leader exits first. Normal completion kills leftover descendants before the next step. Deadlines use monotonic awake time, excluding sleep.

Both output streams are drained without blocking and retain only their last 64 KiB for failure reports. Diagnostic writes use a separate thread with a 16-record queue. A full queue drops new records and reports the count when writing resumes; exit waits at most 100 ms for queued diagnostics. Logging must not delay process timeouts or shutdown.

Diagnostics always go to stderr. The Homebrew formula redirects stderr to `var/"log/runon.log"` under the active Homebrew prefix (`/opt/homebrew` on Apple Silicon, `/usr/local` on Intel). `runon logs` executes `tail -F` on that file, using `HOMEBREW_PREFIX` when set and the architecture's standard prefix otherwise. File rotation is not configured.

## Measurements

```sh
make measure
```

This builds the release binary, measures idle behavior for five minutes through an external Python 3 sampler, then runs the launch-latency and event-flood benchmark. It starts a separate daemon without installing a service. Use a quiet, logged-in Mac; see [measurement methods, budgets and recorded results](docs/validation.md#performance-reproduction).

## Packaging and releases

Use the cargo-dist version pinned in `dist-workspace.toml`. `dist build` prepares release archives, checksums and installers locally without publishing. Release settings use size optimization, LTO, one codegen unit, symbol stripping and abort-on-panic. The deployment target is macOS 15.0; the toolchain and dependency lockfile are pinned.

For a release, update the workspace version and lockfile, add the matching `vVERSION` changelog entry, and run `make check` and `dist build`. Validate the release binary's examples:

```sh
target/release/runon check -c examples/config.kdl
target/release/runon check -c examples/editors.kdl
```

The [release workflow](.github/workflows/release.yml) builds archives and installers with cargo-dist. Pushing a version tag publishes a GitHub release and calls [publish-homebrew.yml](.github/workflows/publish-homebrew.yml) to update `mishamyrt/homebrew-tap`. That job adds the service definition and macOS requirement with `scripts/prepare_homebrew.py` before publishing the generated formula. Do not edit the generated formula directly in the tap: the next release replaces it. Publishing requires the `HOMEBREW_TAP_TOKEN` secret.

After changing dist configuration, run `dist generate --mode ci` and verify `dist generate --mode ci --check` with the pinned version. The Homebrew publisher is a separate reusable workflow so regeneration preserves the service preparation step.
