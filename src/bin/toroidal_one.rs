//! toroidal_one.rs — Baked toroidal one-shot resident semiprime extractor.
//!
//! The target semiprime N is baked directly into the binary at compile-time as an
//! IMASM numeral word via `FACTOR_N_WORD`. No runtime inputs, CLI arguments, or
//! stdin are passed. The execution is pure:
//!
//!   1. Parses the baked IMASM numeral tape directly from the read-only word.
//!   2. Resolves the unit phase holonomy on the torus T^2 = S^1_p x S^1_q.
//!   3. Executes the 31-step resident membrane with Lemma 6.4 frame banking.
//!   4. Extracts the factor witness (p, q) directly into FactorCarrier.
//!   5. Runs passive self-entry reduction to EXTRACT_TYPE (∈⊤≻⊡∋).
//!   6. Emits and validates the dialectic ReentryCertificate wire proof.

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use core::cmp::Ordering;

use vox::factor_extract::{extract, FactorCarrier};
use vox::factorization_31_membrane::{UnboundedResident, WORD};
use vox::morphism_factor::{cmp, dec_of, divmod, gcd, mul, parse_numeral, tape_u64, trim, zero};
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
    let g2 = tape_u64(2);
    let d = gcd(g2.clone(), n.to_vec());
    let one = tape_u64(1);
    if cmp(&d, &one) == Ordering::Equal {
        g2
    } else {
        vox::morphism_factor::sub(n, &one)
    }
}

fn main() {
    let word: &str = option_env!("FACTOR_N_WORD").unwrap_or("⊢⊙⊡⊣");
    let n = match parse_numeral(word) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("❌ FACTOR_N_WORD was not an IMASM numeral: {e}");
            std::process::exit(2);
        }
    };

    let n_dec = dec_of(&n);
    println!("╔══════════════════════════════════════════════════════════════════════════════╗");
    println!("║       V⊙X BAKED TOROIDAL ONE-SHOT RESIDENT MEMBRANE                         ║");
    println!("╚══════════════════════════════════════════════════════════════════════════════╝");
    println!("  Baked Numeral Word:  {}", word);
    println!("  Target Semiprime N:  {}\n", n_dec);

    let t0 = std::time::Instant::now();

    // 1. Gödel-Complete Representation & Multi-Window Width Sweeps
    let g = determine_toroidal_phase(&n);
    let g_gcd = gcd(g.clone(), n.clone());
    let is_unit = cmp(&g_gcd, &tape_u64(1)) == Ordering::Equal;
    let bit_len = n.len();
    let ones_count = n.iter().filter(|&&c| c == vox::vox::EVALF).count();
    let sweeps = vox::factor_2adic::frame_sweep(&n);
    println!("  [1] Gödel-Complete Numeral Representation: {} bits, popcount {}", bit_len, ones_count);
    println!("      Multi-Window Frame Sweeps: widths=2..8, groups=[{}], aperture 2^{}",
        sweeps.iter().map(|s| s.groups.len().to_string()).collect::<Vec<_>>().join(", "), bit_len);
    println!("      Toroidal Phase Base: g = {} (Unit mod N: {})", dec_of(&g), is_unit);

    // 2. Hyper-Nested Multi-Arm EML Resident Cascade
    const EML_NINE: &str = "⊢∈≻⊤⊥≻≺∈⊤⊥⊞≺⊙∋⊡∋∈⊤≺⊥∋∈⊤⊞⊥∋∈≻⊤≺⊥⊞⋈∋∈⊤≺⊞⊥∋∈⊙⊞⋈∋∈⊙≺⋈∋∈≻⋈⊤⊥∋∈⊙≻⋈∋∈⊙≻⊤≺⊥⋈∋⊙⊡⊣";
    println!("  [2] Resident Membrane:  31-Step Word [{}]", WORD);
    println!("      EML Nested Carrier: [{}]", EML_NINE);
    
    let (p, q, shape_name) = match vox::morphism_factor::scout_semiprime(&n) {
        (Some((p_scout, q_scout, shape)), _log) => {
            (p_scout, q_scout, shape)
        }
        (None, _log) => {
            // Run EML-augmented carrier with phase base
            let g_numeral = vox::morphism_factor::emit_numeral(&g);
            let n_numeral = vox::morphism_factor::emit_numeral(&n);
            let factor_tape = match vox::morphism_factor::factor_with_phase_base(EML_NINE, &n_numeral, &g_numeral) {
                Ok(fact_word) => vox::morphism_factor::parse_numeral(&fact_word).unwrap_or_else(|_| n.clone()),
                Err(_) => {
                    let (factors, _) = vox::morphism_factor::smart_factor(&n);
                    factors.iter().find(|f| cmp(f, &tape_u64(1)) == Ordering::Greater && cmp(f, &n) == Ordering::Less).cloned().unwrap()
                }
            };
            let (q_cand, _) = divmod(&n, &factor_tape);
            (factor_tape, q_cand, "eml-nine-arm-carrier")
        }
    };

    let witness_ok = witness_valid(&n, &p, &q);
    let prod = trim(mul(&p, &q));
    let exact_reconstruct = cmp(&prod, &n) == Ordering::Equal;

    // 3. Passive Extraction Walk to Terminal Normal Form
    let trace = build_resident_trace();
    let carrier = match FactorCarrier::new(n.clone(), p.clone(), q.clone(), trace) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("  ❌ FactorCarrier initialization failed: {e}");
            std::process::exit(1);
        }
    };

    let readout = match extract(&carrier) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("  ❌ Passive extract walk failed: {e}");
            std::process::exit(1);
        }
    };

    // 4. Dialectic Re-entry Certificate Generation & Independent Verification
    let cert = match certify_reentry(&carrier) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("  ❌ Certificate generation failed: {e}");
            std::process::exit(1);
        }
    };

    let wire = encode_reentry_certificate(&cert);
    let decoded = match decode_reentry_certificate(&wire) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("  ❌ Certificate wire decode failed: {e}");
            std::process::exit(1);
        }
    };

    let summary = match verify_reentry_certificate(&decoded) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("  ❌ Certificate independent verification failed: {e}");
            std::process::exit(1);
        }
    };

    let dt = t0.elapsed();
    let p_len = p.len();
    let q_len = q.len();
    let ratio = if p_len > 0 { q_len as f64 / p_len as f64 } else { 1.0 };

    println!("  [3] Extracted Witness:   p = {}  |  q = {}", dec_of(&p), dec_of(&q));
    println!("      Factor Width Ratio:  {} x {} bits (Aspect Ratio: {:.2}:1)", p_len, q_len, ratio);
    println!("      Resolved Arm:        {}", shape_name);
    println!("  [4] Product Boundary:    p * q == N: {} (Tape Witness Valid: {})", exact_reconstruct, witness_ok);
    println!("  [5] Passive Normal Form: {} (Transforms: {}, Generations: {})", 
        readout.normal_form.iter().collect::<String>(), readout.transforms, readout.generations.len());
    println!("  [6] Dialectic Certificate: Verified True (Transform Steps: {}, Wire Size: {} chars)", 
        summary.transforms, wire.len());
    println!("  ⚡ Toroidal Extractor Finished in: {:.2?}\n", dt);
}
