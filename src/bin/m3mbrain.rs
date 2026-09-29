//! m3mbrain — Core Numeral membrane probe with a compile-time-baked N.
//!
//! Bake at build time:
//!
//!     BAKED_N=106545994355809 cargo build --release --bin m3mbrain
//!     BAKED_N=106545994355809 cargo run  --release --bin m3mbrain
//!
//! Or edit `BAKED_N_DEFAULT` below and rebuild. The binary takes no runtime
//! arguments: whatever value was baked at build time is what it submits to
//! every Core Numeral axis, and the readouts print to stdout.

use std::collections::BTreeSet;

use vox::bool_polynomial::{BoolCoeff, BoolPolynomial, carry_agreement};
use vox::fibre_geometry::{
    factorial, fibre_size, hasse_edge_count, minimal_cover_count, n_kr, stirling2,
    stirling_bridge_holds,
};
use vox::layer_stack::LayerStack;
use vox::lean_evidence::default_registry;
use vox::morphism_factor::{decimal_to_tape, miller_rabin};
use vox::suffix_envelope::{
    fixed_union_count, image_count, mass_polynomial, SuffixEnvelope,
};

const BAKED_N_DEFAULT: &str = "8051";

/// Compile-time bake:
///     BAKED_N=<decimal> cargo build --release --bin m3mbrain
const BAKED_N: &str = match option_env!("BAKED_N") {
    Some(s) => s,
    None => BAKED_N_DEFAULT,
};

fn section(name: &str) {
    println!("\n==== {} ====", name);
}

fn bits_of(value: u64) -> Vec<u8> {
    if value == 0 {
        return vec![0];
    }
    let mut out = Vec::new();
    let mut n = value;
    while n > 0 {
        out.push((n & 1) as u8);
        n >>= 1;
    }
    out
}

/// Threshold encoding Φ(v): S_0 = I, S_1 = support(v).
fn threshold_encoding(bits: &[u8]) -> BoolPolynomial {
    let n = bits.len();
    let s0 = BoolCoeff::full(n);
    let mut s1_bits = BTreeSet::new();
    for (i, &b) in bits.iter().enumerate() {
        if b == 1 {
            s1_bits.insert(i);
        }
    }
    BoolPolynomial {
        coeffs: vec![s0, BoolCoeff { bits: s1_bits }],
    }
}

fn main() {
    section("m3mbrain");
    println!("BAKED_N = {} (compile-time)", BAKED_N);
    let n_tape = decimal_to_tape(BAKED_N).expect("BAKED_N must be a valid decimal");
    println!("N tape bit length = {}", n_tape.len());
    println!("N is prime (miller_rabin) = {}", miller_rabin(&n_tape));

    let n_u64: Option<u64> = BAKED_N.parse().ok();

    section("FibreGeometry: N(k, r) and closed-form counts, k = 0..4");
    for k in 0..=4u32 {
        let max_r: u128 = 1u128 << k;
        let row: Vec<i128> = (0..=max_r).map(|r| n_kr(k, r)).collect();
        println!(
            "k={} fibre_size={} minimal_covers={} hasse_edges={}",
            k,
            fibre_size(k),
            minimal_cover_count(k),
            hasse_edge_count(k),
        );
        println!("   N(k, r) = {:?}", row);
    }

    section("Stirling bridge over k = 0..4, d = 1..6");
    let all_ok = (0..=4u32).all(|k| (1..=6usize).all(|d| stirling_bridge_holds(k, d)));
    println!("stirling_bridge_holds: {}", all_ok);
    let (k, d) = (2u32, 4usize);
    let lhs: u128 = (2u128.pow(d as u32) - 1).pow(k);
    let rhs: i128 = (0..=1u128 << k)
        .map(|r| n_kr(k, r) * factorial(r as u32) as i128 * stirling2(d, r as usize) as i128)
        .sum();
    println!(
        "(k={}, d={}): (2^d - 1)^k = {} = Σ_r N(k,r) r! S(d,r) = {}",
        k, d, lhs, rhs
    );

    section("SuffixEnvelope on the baked tape");
    match SuffixEnvelope::from_frame_sweep(&n_tape, 2) {
        Ok(env) => {
            println!("gamma       = {:?}", env.gamma);
            println!("reconstruct = {:?}", env.reconstruct());
            println!("r(0)={:?} r(2)={:?}", env.r(0), env.r(2));
        }
        Err(e) => println!("from_frame_sweep rejected the baked tape: {}", e),
    }

    if let Some(v) = n_u64 {
        section("BoolPolynomial / LayerStack on the baked value");
        let n_enc = threshold_encoding(&bits_of(v));
        let stack = LayerStack::from_polynomial(&n_enc);
        println!(
            "N layers={}  stack: mass={} l1={} min={} max={} |union|={}",
            n_enc.coeffs.len(),
            stack.layer_cake_mass(),
            stack.l1_area(),
            stack.min_area(),
            stack.max_area(),
            stack.union_support().len(),
        );
        for (p, q) in [(5u64, 7u64), (83, 97), (127, 131)] {
            let pe = threshold_encoding(&bits_of(p));
            let qe = threshold_encoding(&bits_of(q));
            let prod = pe.cauchy(&qe);
            println!(
                "p={} q={} n={}  layers p={} q={} prod={}  carry_agreement={}",
                p,
                q,
                p * q,
                pe.coeffs.len(),
                qe.coeffs.len(),
                prod.coeffs.len(),
                carry_agreement(&pe, &qe),
            );
        }
    } else {
        section("BoolPolynomial / LayerStack");
        println!("skipped: BAKED_N exceeds u64");
    }

    section("SuffixEnvelope closed forms");
    for (kk, dd) in [(1u32, 6u32), (2, 4), (3, 3), (4, 2)] {
        println!(
            "k={} d={}: image_count={:?} fixed_union_count={:?}",
            kk,
            dd,
            image_count(kk, dd),
            fixed_union_count(kk, dd),
        );
        if let Some(mass) = mass_polynomial(kk, dd) {
            println!("   mass_polynomial = {:?}", mass);
        }
    }

    section("LeanEvidence registry");
    let reg = default_registry();
    println!("len={} ids={:?}", reg.len(), reg.all_ids());
    println!(
        "carry_identity  = {}",
        reg.lookup("carry_identity").is_some()
    );
    println!(
        "syzygy_roundtrip= {}",
        reg.lookup("syzygy_roundtrip").is_some()
    );

    section("done");
}