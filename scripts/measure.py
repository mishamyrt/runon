#!/usr/bin/env python3
"""Measure a release daemon observing every source (only /usr/bin/true actions).

Requires a logged-in macOS GUI session. The sampler is outside the daemon;
CPU times exclude it and exclude initialization. No third-party Python modules.
"""
import argparse
import ctypes
import hashlib
import json
from pathlib import Path
import platform
import subprocess
import tempfile
import time


class Usage(ctypes.Structure):
    _fields_ = [("uuid", ctypes.c_uint8 * 16)] + [
        (name, ctypes.c_uint64) for name in (
            "user", "system", "idle_wakeups", "interrupt_wakeups", "pageins",
            "wired", "resident", "footprint", "start", "exit")
    ]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", default="target/release/runon")
    parser.add_argument("--seconds", type=float, default=300)
    parser.add_argument("--output")
    parser.add_argument("--service", action="store_true", help="use Unified Logging as the LaunchAgent does")
    args = parser.parse_args()
    if args.seconds <= 0:
        parser.error("--seconds must be positive")
    binary = Path(args.binary).resolve()
    binary_bytes = binary.stat().st_size
    binary_hash = hashlib.sha256(binary.read_bytes()).hexdigest()
    lib = ctypes.CDLL("/usr/lib/libproc.dylib", use_errno=True)
    lib.proc_pid_rusage.argtypes = [ctypes.c_int, ctypes.c_int, ctypes.POINTER(Usage)]
    lib.proc_pid_rusage.restype = ctypes.c_int
    with tempfile.TemporaryDirectory(prefix="runon-measure-") as tmp:
        cfg = Path(tmp) / "all.kdl"
        kinds = ("screen.connected", "screen.disconnected", "screen.locked",
                 "screen.unlocked", "audio.connected", "audio.disconnected",
                 "app.activated", "app.deactivated", "app.launched", "app.terminated",
                 "system.wake", "power.changed")
        # Real events may arrive during sampling; their only command is true.
        cfg.write_text("\n".join(
            f'action measure-{i} {{ on {kind}; exec "/usr/bin/true"; }}'
            for i, kind in enumerate(kinds)))
        with (Path(tmp) / "stderr").open("w+") as err:
            command = [str(binary), "run", "-c", str(cfg)]
            if args.service:
                command.append("--service")
            child = subprocess.Popen(command,
                                     stdout=subprocess.DEVNULL, stderr=err)
            def sample():
                if child.poll() is not None:
                    err.seek(0)
                    raise RuntimeError(f"daemon exited {child.returncode}: {err.read()}")
                value = Usage()
                if lib.proc_pid_rusage(child.pid, 0, ctypes.byref(value)) != 0:
                    raise OSError(ctypes.get_errno(), "proc_pid_rusage")
                return value
            try:
                time.sleep(5)  # warm-up, excluded from CPU window
                first = sample()
                peak = first.footprint
                start = time.monotonic()
                print(f"Sampling PID {child.pid} for {args.seconds:g}s", flush=True)
                while time.monotonic() - start < args.seconds:
                    time.sleep(min(1, max(0, args.seconds - (time.monotonic() - start))))
                    last = sample()
                    peak = max(peak, last.footprint)
                seconds = time.monotonic() - start
                result = {
                    "macos": platform.mac_ver()[0], "architecture": platform.machine(),
                    "hardware": subprocess.check_output(["/usr/sbin/sysctl", "-n", "machdep.cpu.brand_string"], text=True).strip(),
                    "binary_bytes": binary_bytes,
                    "binary_sha256": binary_hash,
                    "logging": "unified" if args.service else "stderr",
                    "sample_seconds": round(seconds, 3),
                    "cpu_percent_one_core": (last.user + last.system - first.user - first.system) / (seconds * 1e9) * 100,
                    "physical_footprint_peak_bytes": peak,
                    "physical_footprint_final_bytes": last.footprint,
                    "package_idle_wakeups": last.idle_wakeups - first.idle_wakeups,
                }
                print(json.dumps(result, indent=2))
                if args.output:
                    Path(args.output).write_text(json.dumps(result, indent=2) + "\n")
            finally:
                if child.poll() is None:
                    child.terminate()
                    try:
                        child.wait(timeout=8)
                    except subprocess.TimeoutExpired:
                        child.kill()
                        child.wait()


if __name__ == "__main__":
    main()
