//! hsoa_winding — Topological period readout via exceptional-point winding,
//! with in-binary Shor post-processing: the read period yields the factors.
//!
//! Usage:
//!   hsoa_winding <N> <a> [M]   period readout, then gcd(a^(r/2)−1, N) factors
//!   hsoa_winding gen [s] [M]   build a 128-bit demo semiprime (p, q near 2^64,
//!                              ord_N(a) = 2s via CRT), read the period, factor
//!                              — all in this one binary

use num_bigint::BigUint;
use vox::hsoa_winding;

fn mr_label(n: &BigUint) -> &'static str {
    if hsoa_winding::is_prime_mr(n) {
        "prime"
    } else {
        "composite?"
    }
}

/// Readout + in-binary Shor post-processing: the read period yields the factors.
fn readout_and_factor(n: &BigUint, a: &BigUint, m: usize) -> i32 {
    let comb = hsoa_winding::build_comb(a, n, m);
    match hsoa_winding::read_period(&comb) {
        Some(r) => {
            println!("HSOA Winding Readout");
            println!("  N       = {}", n);
            println!("  a       = {}", a);
            println!("  M       = {}", m);
            println!("  period  = {}", r.period);
            println!("  winding = {}", r.winding);
            println!("  valid   = {}", r.validated);
            println!("  EP order = {}", r.ep_order);
            println!("  EP center = ({:.4}, {:.4})", r.ep_center.re, r.ep_center.im);
            println!("  min discriminant = {:.6e}", r.min_discriminant);
            println!("  ||ψ||² = {}", r.norm_sq);
            println!("  ξ = {:.4}", r.localization_length);
            println!("  time = {} µs", r.elapsed_us);
            match hsoa_winding::shor_factor(a, n, r.period) {
                Some((g, h)) => {
                    let product_ok = g.clone() * h.clone() == *n;
                    println!("Shor Post-Processing (in-binary)");
                    println!("  gcd(a^({}) − 1, N) = {}", r.period / 2, g);
                    println!("  N = {} × {}", g, h);
                    println!("  {} is {} ; {} is {}", g, mr_label(&g), h, mr_label(&h));
                    println!("  g·h = N : {}", product_ok);
                    if !product_ok {
                        return 1;
                    }
                }
                None => {
                    println!("Shor Post-Processing (in-binary)");
                    println!("  no nontrivial split from period {} (odd period, or a^(r/2) ≡ 1)", r.period);
                    return 1;
                }
            }
            0
        }
        None => {
            println!("no period detected");
            1
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    // gen mode: hsoa_winding gen [s] [M]
    if args.first().map(|s| s.as_str()) == Some("gen") {
        let s: u64 = args.get(1).and_then(|t| t.parse().ok()).unwrap_or(3);
        let m: usize = args.get(2).and_then(|t| t.parse().ok()).unwrap_or(64);
        match hsoa_winding::build_demo(s) {
            Some((p, q, n, a)) => {
                println!("Demo Construction (in-binary, CRT recipe)");
                println!("  s       = {}", s);
                println!("  p       = {}  ({})", p, mr_label(&p));
                println!("  q       = {}  ({})", q, mr_label(&q));
                println!("  N       = {}", n);
                println!("  a       = {}  (ord_N(a) = {} verified)", a, 2 * s);
                std::process::exit(readout_and_factor(&n, &a, m));
            }
            None => {
                eprintln!("construction failed: no suitable (p, q, a) found for s = {}", s);
                std::process::exit(1);
            }
        }
    }

    if args.len() < 2 {
        eprintln!("usage: hsoa_winding <N> <a> [M]");
        eprintln!("       hsoa_winding gen [s] [M]");
        std::process::exit(2);
    }
    let n: BigUint = args[0].parse().unwrap_or_else(|e| {
        eprintln!("invalid N: {e}");
        std::process::exit(2);
    });
    let a: BigUint = args[1].parse().unwrap_or_else(|e| {
        eprintln!("invalid a: {e}");
        std::process::exit(2);
    });
    let m: usize = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(64);
    std::process::exit(readout_and_factor(&n, &a, m));
}
