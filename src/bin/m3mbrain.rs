//! m3mbrain — a compiled factor membrane.
//!
//! Build with an IMASM numeral:
//!
//!     VOX_PHASE_MODULUS_WORD='⊢≻⋈∈⊥∋…⊙⊡⊣' cargo build --release --bin m3mbrain
//!     ./target/release/m3mbrain
//!
//! The binary holds an IMASM numeral and emits IMASM numerals. No runtime
//! arguments, no decimals anywhere.

use core::cmp::Ordering;

use vox::morphism_factor::{cmp, divmod, emit_numeral, mul, parse_numeral, zero};
use vox::phase_partners;

include!(concat!(env!("OUT_DIR"), "/baked_inputs.rs"));

#[derive(Clone, Debug)]
enum PhaseEvidence {
    Support(phase_partners::SupportClosure),
    Return(phase_partners::ReturnRelation),
}

fn main() {
    let n_word = BAKED_MODULUS_WORD
        .expect("bake a modulus: VOX_PHASE_MODULUS_WORD=<IMASM numeral>");
    let base_word = BAKED_BASE_WORD.unwrap_or("⊢≻⋈∈⊤∋≻⋈∈⊥∋⊙⊡⊣");  // IMASM 2
    let radix_word = BAKED_WIDTH_WORD.unwrap_or("⊢≻⋈∈⊤∋≻⋈∈⊥∋⊙⊡⊣"); // IMASM 2

    let n_tape = parse_numeral(n_word).expect("baked modulus is a native IMASM numeral");
    let base = parse_numeral(base_word).expect("baked base is a native IMASM numeral");
    let radix = parse_numeral(radix_word).expect("baked radix is a native IMASM numeral");

    let mut partners = match phase_partners::Partners::new(base.clone(), n_tape.clone()) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("phase membrane could not open: {e}");
            emit(&n_tape, None);
            return;
        }
    };
    let evidence = loop {
        if let Some(target) = partners.support_target() {
            break PhaseEvidence::Support(target);
        }
        match partners.observe() {
            Ok(Some(relation)) => break PhaseEvidence::Return(relation),
            Ok(None) => {}
            Err(e) => {
                eprintln!("phase membrane failed: {e}");
                emit(&n_tape, None);
                return;
            }
        }
    };

    let factor_pair = match &evidence {
        PhaseEvidence::Support(t) => Ok((t.p.clone(), t.q.clone())),
        PhaseEvidence::Return(relation) => {
            if let Some((current_half, earlier_half)) = relation.half_residues.as_ref() {
                vox::shor_braid::phase_factor_register_seeds(&n_tape, current_half, earlier_half)
            } else {
                vox::shor_braid::factor_close_public(&base, &n_tape, &relation.return_exponent)
            }
        }
    };

    let Ok((mut p, mut q)) = factor_pair else {
        emit(&n_tape, None);
        return;
    };
    if cmp(&p, &q) == Ordering::Greater {
        core::mem::swap(&mut p, &mut q);
    }

    let product_outer =
        vox::factor_2adic::nest_product_over_prefix(&n_tape, &p, &q, &radix);
    let prefix_outer =
        vox::factor_2adic::nest_prefix_over_product(&n_tape, &p, &q, &radix);
    let meet = product_outer
        .zip(prefix_outer)
        .filter(|(a, b)| a == b)
        .map(|(a, _)| a)
        .and_then(|fixed| {
            vox::factor_2adic::terminal_pair_given_semiprime_promise(&n_tape, fixed)
        });

    match meet {
        Some(fixed) => emit(&n_tape, Some((fixed.p, fixed.q))),
        None => emit(&n_tape, None),
    }
}

/// Emit the membrane's readout as IMASM numeral words on stdout.
/// On success: N, P, Q, P·Q. On failure: N alone.
fn emit(n: &[char], pair: Option<(Vec<char>, Vec<char>)>) {
    println!("N = {}", emit_numeral(n));
    if let Some((p, q)) = pair {
        let prod = mul(&p, &q);
        let (_, remainder) = divmod(n, &p);
        debug_assert!(zero(&remainder));
        debug_assert_eq!(cmp(&prod, n), Ordering::Equal);
        println!("P = {}", emit_numeral(&p));
        println!("Q = {}", emit_numeral(&q));
        println!("R = {}", emit_numeral(&prod));
    }
}