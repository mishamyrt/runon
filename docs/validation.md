# RunOn 2 validation

Implementation validation was performed on an Apple M3 Pro, macOS 27.0, ARM64, with Rust 1.98.0. The deployment target is macOS 15.0 for ARM64 and Intel. Results on macOS 27 ARM64 do **not** establish event delivery on macOS 15 or Intel; that hardware/OS acceptance remains open.

## Automated checks

```sh
make check
make build-release
cargo test --locked -p runon --test service -- --ignored --nocapture
cargo run --locked -p runon-macos --example native_smoke
```

| Area | Checked locally |
| --- | --- |
| Config | Bundled KDL examples, Unicode and raw strings, exact field matching, missing fields, OR/AND, numeric IDs, every matching action, forward group references; invalid nodes/types/events/filters/durations/duplicates and Unicode line/column positions |
| Scheduler | Controlled monotonic clock, ordered batches, latest pending replacement, debounce reset, independent groups, concurrency limit, readiness FIFO, shutdown, bounded pending storage under thousands of replacements |
| Runtime boundaries | Replacements before the first dispatch preserve FIFO and latest batch contents, with and without debounce; report callbacks can access the runtime queue; unread stderr and a flooded log queue preserve process timeouts and SIGTERM shutdown |
| Processes | 100 rapid exits, spawn failure, literal argv, sequential failure, following batch actions, 2 MB per output stream with exact 64 KiB tails, timeout, SIGTERM/SIGKILL escalation, killed descendant, slot reuse, graceful daemon shutdown |
| CLI/install | Config paths/defaults, invalid arguments, empty config staying asleep, SIGTERM shutdown; installer paths with spaces, archive selection for both architectures from a combined checksum manifest, rejection of mismatched/missing checksums preserving the previous binary, and rejection of unsupported systems before downloading |
| LaunchAgent | Isolated start/status/restart/stop, idempotent start, retained config path, invalid restart preserving PID, recovery after SIGKILL, restart waiting for a slow SIGTERM handler, retained settings after stop |
| Native sources | AppKit event queue processing; all subscriptions and initial snapshots on a real GUI session; no startup connection events, repeated screen notifications and wake refresh without false display changes; real NSWorkspace launch/termination of a temporary app |

The native test opens its own short-lived app hidden and without activation. It posts screen/wake notifications only to notification centers inside its own process, then removes its app bundle. It does not simulate a physical wake or broadcast fake lock notifications. The LaunchAgent test uses its own temporary home, config files and unique label; it leaves the user's `co.myrt.runon` service alone.

The AppKit queue regression check posts a process-local `NSEvent` and verifies that the main loop consumes it. It failed with the previous bare `CFRunLoop` and passes with `NSApplication.run`, which processes the WindowServer events needed for display-change notifications. Synthetic notification delivery alone did not detect this bug; physical monitor acceptance below is still required. Shutdown dispatches `stop` to the main queue and posts a wake event so SIGINT/SIGTERM can exit an idle AppKit loop. See Apple's [event loop](https://developer.apple.com/documentation/appkit/nsapplication/run%28%29) and [stop behavior](https://developer.apple.com/documentation/appkit/nsapplication/stop%28_%3A%29) documentation.

The scripts and workflows cover macOS 15 ARM64/Intel and macOS 26 ARM64. Release archives are built separately on macOS 15 for each architecture, then combined with a shared checksum manifest. GitHub Actions has not been run from this local checkout. Publishing requires an explicit version tag; no tag or release was created.

### Compatibility checks — 2026-09-18

`make check` passed natively on ARM64 and for `x86_64-apple-darwin` through Rosetta on macOS 27.0. The CLI test waits for the daemon's readiness log before checking idle behavior and SIGTERM, rather than assuming startup finishes within 150 ms. The explicitly ignored LaunchAgent lifecycle test was not rerun.

Both release binaries have `LC_BUILD_VERSION minos 15.0`: ARM64 is 723,488 bytes and x86_64 is 779,560 bytes. Both validate the bundled configurations. ARM64 packaging and the release workflow's archive/checksum combination passed locally with real binaries for both architectures. The Intel `native_smoke` example also passed through Rosetta, including AppKit event processing, subscriptions, snapshots and real launch/termination notifications from its temporary app. Physical Intel hardware and macOS 15 acceptance remain outstanding.

## Performance reproduction

Use an otherwise quiet, logged-in Mac. Build first, finish other tests, then measure:

```sh
cargo build --release --locked -p runon
python3 scripts/measure.py --service --seconds 300 --output /tmp/runon-idle.json
cargo run --release --locked -p runon-runtime --example benchmark > /tmp/runon-engine.json
```

`measure.py` creates a temporary config covering all 12 event kinds. Any real event can only run `/usr/bin/true`. It starts a separate daemon, excludes five seconds of initialization, then reads `proc_pid_rusage` once per second from the external Python sampler. The CPU percentage is the daemon's user+system CPU delta divided by elapsed time and refers to **one core**; sampler CPU and child command CPU are excluded. Physical footprint is the kernel's `ri_phys_footprint`, not RSS. Reported peak is the largest one-second sample, not an instantaneous high-water mark. `--service` uses the same Unified Logging path as launchd without installing a service. The script terminates and reaps its daemon and removes its config afterward. It records the measured binary's initial SHA-256, byte size, logging mode and host details.

The engine benchmark takes 1,000 sequential `/usr/bin/true` samples after 20 warm-ups. Each sample is timed from acceptance into `Runtime::submit` to immediately before the first `Command::spawn` call, including rule matching and dispatch. Report delivery happens afterward, so the measurement excludes report callback timing. No group is busy and no debounce is configured. This measures launch initiation, not OS notification delivery or time until the new program executes its first instruction.

It then submits 1,000,000 typed events, sampling its own physical footprint every 10,000 events. This exercises the same matching, bounded scheduler and subprocess engine used by the daemon. It measures engine memory under a flood, not a million physical device transitions. Benchmark reports are drained so their channel does not masquerade as daemon memory growth. Native refresh requests also use coalesced DispatchSources, rather than one queued task per notification.

| Metric | Target |
| --- | --- |
| Release binary | ≤ 5 MiB |
| Physical footprint with every source | ≤ 50 MiB |
| Calm idle over 300 seconds | < 0.1% of one CPU core |
| Acceptance → launch initiation, without queue/debounce | p95 ≤ 10 ms |

### Measured results — 2026-09-13

Baseline from the release binary before the workspace split, Apple M3 Pro / macOS 27.0:

| Metric | Result | Budget |
| --- | --- | --- |
| Binary | 773,328 bytes / 0.738 MiB | Met |
| All-source footprint, sampled peak | 7,619,304 bytes / 7.27 MiB | Met |
| Idle CPU, 300.005 seconds, Unified Logging | 0.000348% of one core | Met |
| Package idle wakeups | 0 | — |
| Launch initiation p50 / p95 / p99 | 0.006334 / 0.009542 / 0.011917 ms | Met |
| Engine footprint before / sampled peak / after 1M events | 3,080,720 / 3,080,720 / 3,080,720 bytes | No measured growth |
| Submission time for 1M events | 0.096777 s | — |

The measured pre-workspace binary had SHA-256 `71ded6f5eb344cccbf80b91c63a4e6ecefb9cdc9c4b1ba589ff8d1700c7d2016`. These measurements are retained as a historical baseline; rebuilding the workspace produces a different artifact and does not remeasure idle CPU or memory.

Raw data: [idle daemon](measurements/2026-09-13-idle.json), [latency and engine flood](measurements/2026-09-13-engine.json). These are local measurements, not guarantees for every machine or config. No measured budget needs revision. The native hardware checks below remain outstanding.

The clocks used by Rust's Apple `Instant` and dispatch deadlines measure awake uptime. See the [Rust clock implementation](https://doc.rust-lang.org/src/std/sys/time/unix.rs.html).

## Hardware acceptance still required

Run `target/release/runon events` in a GUI terminal on macOS 15 for both Apple Silicon and Intel, and on newer supported OS versions. Keep the output as the validation record. The following transitions have **not** been physically exercised during this rewrite:

| Transition | Expected observation |
| --- | --- |
| Disconnect/reconnect an external monitor | One disconnected selector with saved name/ID, then one connected selector; changing resolution alone creates no connection event |
| Disconnect/reconnect an audio device | Saved UID/name on removal, current metadata on addition; two devices with the same name remain distinct |
| Activate/deactivate a GUI application | Its exact bundle ID/name, each corresponding transition; missing OS fields remain absent |
| Lock and unlock | One `screen.locked`, then one `screen.unlocked`; verify real delivery on each tested OS/architecture explicitly |
| Sleep, change attached devices/power, then wake | `system.wake`, followed by any missed snapshot differences, with no duplicate device changes from later notifications |
| Switch AC/battery/UPS | One `power.changed` per source transition; battery percentage changes alone do not wake the daemon |

Use disposable `/usr/bin/true` actions when checking a daemon against these selectors. A physical display/audio device and battery/UPS are needed for their respective transitions. Locking or sleeping the user's active session was not performed automatically.

NSWorkspace launch/termination notifications intentionally follow Apple's delivery rules: background applications and `LSUIElement` apps are excluded. The native smoke test uses a regular app without activating it. [Apple documentation](https://developer.apple.com/documentation/appkit/nsworkspace/didlaunchapplicationnotification).

Screen lock uses undocumented distributed notification names. If real delivery fails on a supported OS/architecture, it is a release blocker for that event source; a successful subscription or a synthetic notification is not sufficient evidence.
