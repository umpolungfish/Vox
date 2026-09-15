//! Baked GPU membrane for the complete bvalsd input set.
//!
//! The decimal payload is compiled into this launcher.  Each value is handed
//! to the existing G-mOMonadOS CUDA GNFS membrane, with two resident workers
//! assigned across the two physical GPUs.  Child output is buffered so
//! the launcher emits nothing until every baked value has completed.

use std::process::Command;
use std::thread;
use std::time::Instant;

const BAKED_VALUES: &str = include_str!("/home/mrnob0dy666/imsgct/bvalsd.txt");
const GPU_MEMBRANE: &str = "/home/mrnob0dy666/imsgct/G-mOMonadOS/target/release/g-momonados";

fn main() {
    let values: Vec<String> = BAKED_VALUES.lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.trim().to_owned())
        .collect();
    let started = Instant::now();
    let mut reports = Vec::with_capacity(values.len());
    for chunk in values.chunks(2) {
        let mut workers = Vec::with_capacity(chunk.len());
        for (offset, value) in chunk.iter().enumerate() {
            let index = reports.len() + offset;
            let value = value.clone();
            workers.push(thread::spawn(move || {
            let device = (index % 2).to_string();
            let item_started = Instant::now();
            let output = Command::new(GPU_MEMBRANE)
                .env("CUDA_VISIBLE_DEVICES", &device)
                .arg("gpu_gnfs")
                .arg(&value)
                .output();
            let elapsed_ms = item_started.elapsed().as_millis();
            match output {
                Ok(result) => {
                    let mut text = String::from_utf8_lossy(&result.stdout).into_owned();
                    if !result.status.success() {
                        text.push_str(&String::from_utf8_lossy(&result.stderr));
                    }
                    format!("index={} device={} digits={} elapsed_ms={} {}",
                        index + 1, device, value.len(), elapsed_ms,
                        text.lines().rev().find(|line| line.contains("gpu_factor "))
                            .unwrap_or("gpu_factor produced no result").trim())
                }
                Err(error) => format!("index={} device={} digits={} elapsed_ms={} launch-error={error}",
                    index + 1, device, value.len(), elapsed_ms),
            }
            }));
        }
        for worker in workers {
            reports.push(worker.join().unwrap_or_else(|_| "worker panicked".into()));
        }
    }
    println!("gpu_bvalsd: {} baked inputs completed in {} ms", reports.len(), started.elapsed().as_millis());
    for report in reports { println!("{report}"); }
}
