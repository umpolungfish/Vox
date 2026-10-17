//! shor_one.rs — Baked Shor Order-Finding Membrane one-shot.
//!
//! The target semiprime N is baked directly into the binary at compile-time as an
//! IMASM numeral word via `FACTOR_N_WORD`. No runtime inputs, CLI arguments, or
//! stdin are passed. The execution is pure:
//!
//!   1. Parses the baked IMASM numeral tape directly from the read-only word.
//!   2. Executes the Shor δ/μ bracket: δ (order-finding) → arms (even/nontrivial) → μ (factor close).
//!   3. Falls back to smart_factor (MPQS/QS/Nine-Arm) if Shor fails or N is too large
//!      for BSGS (baby-table size is isqrt(N)).
//!   4. Emits the factor pair.
//!   5. Produces no diagnostic output during execution (silent operation).
#![deny(warnings)]

extern crate alloc;

use vox::morphism_factor::{dec_of, parse_numeral, smart_factor};
use vox::shor_membrane::shor_factor;

fn main() {
    let word: &str = option_env!("FACTOR_N_WORD").unwrap_or("⊢⊙⊣");
    let n = match parse_numeral(word) {
        Ok(t) => t,
        Err(_) => { return; }
    };

    // Shor order-finding with multiple bases (2,3,5,7,11,13)
    if let Ok((p, q, _report)) = shor_factor(&n, 6) {
        println!("{} = {} x {}", dec_of(&n), dec_of(&p), dec_of(&q));
        return;
    }

    // Fall back to smart_factor (MPQS/QS/Nine-Arm)
    let (factors, _) = smart_factor(&n);
    if factors.len() >= 2 {
        let p = dec_of(&factors[0]);
        let q = dec_of(&factors[1]);
        println!("{} = {} x {}", dec_of(&n), p, q);
    }
}