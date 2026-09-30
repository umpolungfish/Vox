//! External observation instrument. This is not the prepared carrier.
#![deny(warnings)]
use std::sync::{Arc, Mutex, mpsc};
use std::time::{Duration, Instant};
use vox::glut_system::GlutSieve;
use vox::morphism_factor::parse_numeral;
fn main() {
    let n = parse_numeral(option_env!("GLUT_SOURCE_WORD").expect("baked source")).unwrap();
    let observation = Arc::new(Mutex::new(None));
    let observed = observation.clone();
    let (sender, receiver) = mpsc::channel();
    let start = Instant::now();
    std::thread::spawn(move || {
        let mut sieve = GlutSieve::new(&n);
        sieve.frame_sweep_observed(|sieve, _| {
            if sieve.fold_stats.active {
                *observed.lock().unwrap() = Some((start.elapsed(), sieve.fold_stats.clone()));
            }
        });
        let result = sieve.readout_execution();
        if let Some(ref execution) = result { execution.verify(&n).unwrap(); }
        sender.send((result.is_some(), sieve.fold_stats)).unwrap();
    });
    let result = receiver.recv_timeout(Duration::from_secs(25));
    println!("lane={} elapsed={:?} result={:?} latest={:?}",
        option_env!("GLUT_DIAGNOSTIC_LANE").unwrap_or("combined"),
        start.elapsed(), result, *observation.lock().unwrap());
}
