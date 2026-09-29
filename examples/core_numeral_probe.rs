//! core_numeral_probe — submit concrete values to the Core Numeral axes
//! and print the readouts. This is a value-submission probe, not a test:
//! it feeds real numbers through FibreGeometry, BoolPolynomial/LayerStack,
//! SuffixEnvelope, and LeanEvidence, and prints what comes back.

use std::collections::BTreeSet;

use vox::bool_polynomial::{BoolCoeff, BoolPolynomial, carry_agreement};
use vox::fibre_geometry::{
    factorial, fibre_size, hasse_edge_count, minimal_cover_count, n_kr, stirling2,
    stirling_bridge_holds,
};
use vox::layer_stack::LayerStack;
use vox::lean_evidence::default_registry;
use vox::morphism_factor::tape_u64;
use vox::suffix_envelope::{
    fixed_union_count, image_count, mass_polynomial, SuffixEnvelope,
};

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

/// Threshold encoding Φ(v): S_0 = I (all atoms), S_1 = support(v).
/// Higher layers are left to the Cauchy product to fill.
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

fn section(name: &str) {
    println!("\n==== {} ====", name);
}

fn main() {
    section("FibreGeometry: N(k, r), fibre sizes, minimal covers, Hasse edges");
    for k in 0..=4u32 {
        let max_r: u128 = 1u128 << k;
        let row: Vec<i128> = (0..=max_r).map(|r| n_kr(k, r)).collect();
        println!("k={} N(k, r) = {:?}", k, row);
        println!(
            "   fibre_size={} minimal_covers={} hasse_edges={}",
            fibre_size(k),
            minimal_cover_count(k),
            hasse_edge_count(k),
        );
    }

    section("Stirling bridge: (2^d - 1)^k = sum_r N(k, r) r! S(d, r)");
    let mut all_ok = true;
    for k in 0..=4u32 {
        for d in 1..=6usize {
            if !stirling_bridge_holds(k, d) {
                println!("k={} d={} FAILED", k, d);
                all_ok = false;
            }
        }
    }
    println!("stirling_bridge_holds for k=0..4, d=1..6: {}", all_ok);

    let (k, d) = (2u32, 4usize);
    let lhs: u128 = (2u128.pow(d as u32) - 1).pow(k);
    let rhs: i128 = (0..=1u128 << k)
        .map(|r| n_kr(k, r) * factorial(r as u32) as i128 * stirling2(d, r as usize) as i128)
        .sum();
    println!("k={} d={}: lhs={} rhs={}", k, d, lhs, rhs);

    section("BoolPolynomial / LayerStack: threshold encodings of real values");
    for (p, q) in [(5u64, 7u64), (83, 97), (127, 131)] {
        let n = p * q;
        let p_enc = threshold_encoding(&bits_of(p));
        let q_enc = threshold_encoding(&bits_of(q));
        let n_enc = threshold_encoding(&bits_of(n));
        let prod = p_enc.cauchy(&q_enc);

        println!("p={} q={} n={}", p, q, n);
        println!(
            "  layers: p={} q={} prod={} n={}",
            p_enc.coeffs.len(),
            q_enc.coeffs.len(),
            prod.coeffs.len(),
            n_enc.coeffs.len(),
        );
        println!(
            "  carry_agreement(p_enc, q_enc) = {}",
            carry_agreement(&p_enc, &q_enc),
        );

        let stack = LayerStack::from_polynomial(&prod);
        println!(
            "  stack: mass={} l1={} min={} max={} |union|={}",
            stack.layer_cake_mass(),
            stack.l1_area(),
            stack.min_area(),
            stack.max_area(),
            stack.union_support().len(),
        );
    }

    section("SuffixEnvelope: real deposits and real tapes");
    match SuffixEnvelope::from_deposits(&[4u32, 2, 1]) {
        Some(env) => {
            println!(
                "from_deposits([4, 2, 1]): gamma={:?} reconstruct={:?}",
                env.gamma,
                env.reconstruct(),
            );
            println!("  r(0)={:?} r(2)={:?}", env.r(0), env.r(2));
        }
        None => println!("from_deposits([4, 2, 1]) rejected"),
    }

    for value in [35u64, 8051, 106_545_994_355_809] {
        let tape = tape_u64(value);
        match SuffixEnvelope::from_frame_sweep(&tape, 2) {
            Ok(env) => println!(
                "from_frame_sweep(tape_u64({}), w=2): gamma={:?}",
                value, env.gamma,
            ),
            Err(e) => println!(
                "from_frame_sweep(tape_u64({}), w=2): rejected ({})",
                value, e,
            ),
        }
    }

    section("SuffixEnvelope closed forms: image, fixed-union, mass");
    for (k, d) in [(1u32, 6u32), (2, 4), (3, 3), (4, 2)] {
        println!(
            "k={} d={}: image_count={:?} fixed_union_count={:?}",
            k,
            d,
            image_count(k, d),
            fixed_union_count(k, d),
        );
        if let Some(mass) = mass_polynomial(k, d) {
            println!("   mass_polynomial = {:?}", mass);
        }
    }
    println!(
        "(2^3 - 1)^2 = {}   fixed_union_count(2, 3) = {:?}",
        7u128.pow(2),
        fixed_union_count(2, 3),
    );

    section("LeanEvidence: registry contents");
    let reg = default_registry();
    println!("len={} ids={:?}", reg.len(), reg.all_ids());
    for id in ["carry_identity", "syzygy_roundtrip", "nonexistent"] {
        println!("  lookup({:?}) = {}", id, reg.lookup(id).is_some());
    }

    println!("\ndone");
}