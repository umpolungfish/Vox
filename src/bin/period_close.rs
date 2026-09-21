//! period-close — factor N through the period-finding ladder and the winding
//! decode, the twin of the OPI/DQI reading.
//!
//! Usage:
//!   period_close <a> <N> <s> <r0>   close from a winding sample s/r0 (a single
//!                                   eigenphase); arbitrary width, no orbit walk.
//!   period_close <a> <N>            read the period from the band ladder for the
//!                                   reachable regime (orbit walk, bounded), then
//!                                   close.
extern crate alloc;

use vox::morphism_factor::{decimal_to_tape, dec_of, tape_u64, cmp, modulo, mul, one};
use vox::period_ladder::{close_from_sample, band_second_ritz, period_from_ritz};

fn orbit_order(a: &[char], n: &[char], cap: usize) -> Option<usize> {
    let mut v = one();
    for r in 1..=cap {
        v = modulo(&mul(&v, a), n);
        if cmp(&v, &one()) == core::cmp::Ordering::Equal { return Some(r); }
    }
    None
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() < 2 {
        eprintln!("usage: period_close <a> <N> [<s> <r0>]");
        std::process::exit(2);
    }
    let a = decimal_to_tape(&args[0]).expect("base");
    let n = decimal_to_tape(&args[1]).expect("modulus");

    if args.len() >= 4 {
        let s = decimal_to_tape(&args[2]).expect("sample numerator");
        let r0 = decimal_to_tape(&args[3]).expect("sample denominator");
        match close_from_sample(&a, &n, &s, &r0) {
            Ok((p, q)) => println!("factors {} x {}", dec_of(&p), dec_of(&q)),
            Err(e) => println!("no split: {e}"),
        }
        return;
    }

    // No sample: read the period from the band ladder over the reachable orbit.
    let cap = 1_000_000usize;
    let Some(r) = orbit_order(&a, &n, cap) else {
        println!("order exceeds the walked band ({cap}); supply a winding sample s/r0");
        return;
    };
    let band = core::cmp::min(64, r / 2 + 4);
    let lam2 = band_second_ritz(r, band);
    let read = period_from_ritz(lam2, 3 * band + 4);
    println!("ladder: orbit r={r} band={band} second_ritz={lam2:.6} read_r={read}");
    // one winding sample s/r with s=1 (a single eigenphase), decode and close.
    match close_from_sample(&a, &n, &one(), &tape_u64(read as u64)) {
        Ok((p, q)) => println!("factors {} x {}", dec_of(&p), dec_of(&q)),
        Err(e) => println!("no split from this sample: {e}"),
    }
}
