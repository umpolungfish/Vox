//! External observation instrument. This is not the prepared carrier.
#![deny(warnings)]
use std::sync::{Arc, Mutex, mpsc};
use std::time::{Duration, Instant};
use vox::glut_system::GlutSieve;
use vox::morphism_factor::parse_numeral;
fn main() {
    let n = parse_numeral(option_env!("GLUT_SOURCE_WORD").expect("baked source")).unwrap();
    let observation = Arc::new(Mutex::new((None, std::collections::BTreeMap::<char,Duration>::new())));
    let observed = observation.clone();
    let (sender, receiver) = mpsc::channel();
    let start = Instant::now();
    std::thread::spawn(move || {
        let mut sieve = GlutSieve::new(&n);
        let mut previous = Instant::now();
        let mut previous_stage = '∈';
        sieve.frame_sweep_observed(|sieve, _| {
            if sieve.fold_stats.active {
                let now = Instant::now();
                let mut reading = observed.lock().unwrap();
                *reading.1.entry(previous_stage).or_default() += now.duration_since(previous);
                reading.0 = Some((start.elapsed(), sieve.fold_stats.clone()));
                previous = now;
                previous_stage = sieve.fold_stats.diagnostic_stage;
            }
        });
        let result = sieve.readout_execution();
        if let Some(ref execution) = result { execution.verify(&n).unwrap(); }
        let _ = sender.send((result.is_some(), sieve.fold_stats));
    });
    let result = receiver.recv_timeout(Duration::from_secs(25));
    println!("lane={} elapsed={:?} result={:?} latest={:?}",
        option_env!("GLUT_DIAGNOSTIC_LANE").unwrap_or("combined"),
        start.elapsed(), result, *observation.lock().unwrap());
}
