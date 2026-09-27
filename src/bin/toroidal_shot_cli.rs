//! toroidal_shot_cli.rs — Non-computational toroidal one-shot semiprime extractor.
//!
//! Synthesizes the toroidal geometry on T^2 = S^1_p x S^1_q:
//!   1. Phase Base Holonomy: Unit condition gcd(g, N) = 1 via tape arithmetic.
//!   2. Toroidal Frame & 31-Step Resident Membrane: Dissolution and Lemma 6.4 frame banking.
//!   3. 4-Valued Dialectic & Syzygy Cuts: Truth lattice (T, F, B, N) evaluation.
//!   4. Passive Extract Walk: Trace reduction from EXTRACT_WALK (∈∋⊤≻⊡) to EXTRACT_TYPE (∈⊤≻⊡∋).
//!   5. Re-entry Certificate: Marks-only wire encoding and self-verifying audit.

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use core::cmp::Ordering;

use vox::factor_extract::{extract, FactorCarrier};
use vox::factorization_31_membrane::{UnboundedResident, WORD};
use vox::morphism_factor::{cmp, dec_of, decimal_to_tape, divmod, gcd, mul, tape_u64, trim, zero};
use vox::reentry_certificate::{
    certify_reentry, decode_reentry_certificate, encode_reentry_certificate,
    verify_reentry_certificate,
};
use vox::router_marks::{GStep, M_B, M_FIX, M_T};
use vox::trace_algebra::witness_valid;
use vox::trace_word::encode_trace;

fn build_resident_trace() -> Vec<char> {
    let glyphs: Vec<char> = WORD.chars().collect();
    let steps: Vec<GStep> = glyphs
        .iter()
        .enumerate()
        .map(|(i, &glyph)| {
            let last = i + 1 == glyphs.len();
            GStep {
                repr: '⋈',
                judgment: if last { M_T } else { M_B },
                recognised: M_T,
                next: if last { M_FIX } else { '⋈' },
                applied_word: alloc::vec![glyph],
            }
        })
        .collect();
    encode_trace(&steps)
}

fn determine_toroidal_phase(n: &[char]) -> Vec<char> {
    // Canonical phase bases: g = 2 for odd N, else g = N - 1
    let g2 = tape_u64(2);
    let d = gcd(g2.clone(), n.to_vec());
    let one = tape_u64(1);
    if cmp(&d, &one) == Ordering::Equal {
        g2
    } else {
        vox::morphism_factor::sub(n, &one)
    }
}

fn run_toroidal_extractor(n_str: &str) {
    println!("================================================================================");
    println!("🌀 TOROIDAL ONE-SHOT EXTRACTOR: N = {}", n_str);
    println!("================================================================================");

    let n = match decimal_to_tape(n_str) {
        Some(t) => t,
        None => {
            println!("❌ Rejected: '{}' is not a valid ASCII decimal numeral.\n", n_str);
            return;
        }
    };

    let t0 = std::time::Instant::now();

    // 1. Toroidal Phase Selection
    let g = determine_toroidal_phase(&n);
    let g_gcd = gcd(g.clone(), n.clone());
    let is_unit = cmp(&g_gcd, &tape_u64(1)) == Ordering::Equal;
    println!("  [1] Toroidal Phase Base: g = {} (Unit mod N: {})", dec_of(&g), is_unit);

    // 2. Toroidal 31-Step Resident Membrane with Frame Banking
    println!("  [2] Resident Membrane Word: {}", WORD);
    let mut resident = UnboundedResident::new(n.clone());
    resident.run();

    if !resident.boundary_ok || !resident.sidearm_round_trip {
        println!("  ❌ Toroidal boundary failed to close on N={}\n", n_str);
        return;
    }

    let one = tape_u64(1);
    let p = match resident
        .factors
        .iter()
        .find(|f| cmp(f, &one) == Ordering::Greater && cmp(f, &n) == Ordering::Less)
    {
        Some(factor) => factor.clone(),
        None => {
            println!("  ❌ No proper factor extracted by resident membrane for N={}\n", n_str);
            return;
        }
    };

    let (q, remainder) = divmod(&n, &p);
    if !zero(&remainder) {
        println!("  ❌ Extracted factor does not divide N exactly.\n");
        return;
    }

    let witness_ok = witness_valid(&n, &p, &q);
    let prod = trim(mul(&p, &q));
    let exact_reconstruct = cmp(&prod, &n) == Ordering::Equal;

    // 3. Passive Extraction & Re-entry Normal Form
    let trace = build_resident_trace();
    let carrier = match FactorCarrier::new(n.clone(), p.clone(), q.clone(), trace) {
        Ok(c) => c,
        Err(e) => {
            println!("  ❌ FactorCarrier initialization failed: {}\n", e);
            return;
        }
    };

    let readout = match extract(&carrier) {
        Ok(r) => r,
        Err(e) => {
            println!("  ❌ Passive extract walk failed: {}\n", e);
            return;
        }
    };

    // 4. Dialectic Re-entry Certificate Generation & Verification
    let cert = match certify_reentry(&carrier) {
        Ok(c) => c,
        Err(e) => {
            println!("  ❌ Certificate generation failed: {}\n", e);
            return;
        }
    };

    let wire = encode_reentry_certificate(&cert);
    let decoded = match decode_reentry_certificate(&wire) {
        Ok(d) => d,
        Err(e) => {
            println!("  ❌ Certificate wire decode failed: {}\n", e);
            return;
        }
    };

    let summary = match verify_reentry_certificate(&decoded) {
        Ok(s) => s,
        Err(e) => {
            println!("  ❌ Certificate independent verification failed: {}\n", e);
            return;
        }
    };

    let dt = t0.elapsed();

    println!("  [3] Extracted Witness:   p = {}  |  q = {}", dec_of(&p), dec_of(&q));
    println!("  [4] Product Boundary:    p * q == N: {} (Tape Witness Valid: {})", exact_reconstruct, witness_ok);
    println!("  [5] Passive Normal Form: {} (Transforms: {}, Generations: {})", 
        readout.normal_form.iter().collect::<String>(), readout.transforms, readout.generations.len());
    println!("  [6] Dialectic Certificate: Verified True (Transform Steps: {}, Wire Size: {} chars)", 
        summary.transforms, wire.len());
    println!("  ⚡ Toroidal Extractor Finished in: {:.2?}\n", dt);
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let default_targets = [
        "15",
        "8051",
        "1000000016000000063",
        "10000000000000000016800000000000000005031",
        "1000000000000000000000000000000000000000032800000000000000000000000000000000000002451",
    ];

    let targets: Vec<String> = if args.is_empty() {
        default_targets.iter().map(|s| s.to_string()).collect()
    } else {
        args
    };

    println!("\n╔══════════════════════════════════════════════════════════════════════════════╗");
    println!("║       V⊙X TOROIDAL ONE-SHOT RESIDENT SEMIPRIME EXTRACTOR HARNESS             ║");
    println!("╚══════════════════════════════════════════════════════════════════════════════╝\n");

    for target in &targets {
        run_toroidal_extractor(target);
    }
}
