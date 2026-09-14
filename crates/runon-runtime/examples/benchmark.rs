//! Run with `cargo run -p runon-runtime --release --example benchmark` on an otherwise idle Mac.
#![allow(clippy::print_stdout)] // Machine-readable benchmark output.
use runon_config::Config;
use runon_core::event::{Event, Kind};
use runon_runtime::{Report, Runtime};
use std::{
    sync::mpsc,
    time::{Duration, Instant},
};

fn footprint() -> u64 {
    let mut info: libc::rusage_info_v0 = unsafe { std::mem::zeroed() };
    let result = unsafe {
        libc::proc_pid_rusage(
            std::process::id().try_into().unwrap(),
            0,
            (&raw mut info).cast(),
        )
    };
    assert_eq!(result, 0);
    info.ri_phys_footprint
}

fn main() {
    let (tx, rx) = mpsc::channel();
    let runtime = Runtime::new(
        Config::parse("action bench { on system.wake; exec \"/usr/bin/true\"; }").unwrap(),
        move |r| {
            let _ = tx.send(r);
        },
    )
    .unwrap();
    let mut latencies = Vec::new();
    for i in 0..1020 {
        runtime.submit(Event::new(Kind::Wake));
        loop {
            match rx.recv_timeout(Duration::from_secs(10)).unwrap() {
                Report::Started { latency, .. } if i >= 20 => {
                    latencies.push(latency.as_secs_f64() * 1000.0);
                }
                Report::Finished { success, .. } => {
                    assert!(success);
                    break;
                }
                _ => {}
            }
        }
    }
    latencies.sort_by(f64::total_cmp);
    let before = footprint();
    let mut peak = before;
    let start = Instant::now();
    for i in 0..1_000_000 {
        runtime.submit(Event::new(Kind::Wake));
        if i % 10_000 == 0 {
            // Drain benchmark reports too; never confuse harness memory with daemon memory.
            while rx.try_recv().is_ok() {}
            peak = peak.max(footprint());
        }
    }
    let flood_seconds = start.elapsed().as_secs_f64();
    runtime.shutdown();
    loop {
        if matches!(
            rx.recv_timeout(Duration::from_secs(10)).unwrap(),
            Report::Stopped
        ) {
            break;
        }
    }
    println!(
        "{{\"samples\":1000,\"p50_ms\":{},\"p95_ms\":{},\"p99_ms\":{},\"flood_events\":1000000,\"flood_seconds\":{},\"footprint_before_bytes\":{},\"footprint_peak_bytes\":{},\"footprint_after_bytes\":{}}}",
        latencies[500],
        latencies[950],
        latencies[990],
        flood_seconds,
        before,
        peak,
        footprint()
    );
}
