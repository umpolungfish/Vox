//! hsoa_winding — Topological period readout via exceptional-point winding.
//!
//! Usage: hsoa_winding <N> <a> [M]
//!   N  = modulus
//!   a  = phase base (unit modulo N)
//!   M  = register size (default 64)

use num_bigint::BigUint;
use vox::hsoa_winding;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() < 2 {
        eprintln!("usage: hsoa_winding <N> <a> [M]");
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

    let comb = hsoa_winding::build_comb(&a, &n, m);
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
        }
        None => {
            println!("no period detected");
            std::process::exit(1);
        }
    }
}