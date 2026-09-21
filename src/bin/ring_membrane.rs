//! ring_membrane — the two-ring winding read, baked.
//!
//! Realizes the auto-designed word ⊢∈≻⋈⊤≺⊥⋈⊙⊞∋⊡⊣ (ob3ect: "orbit ring vs unit
//! ring, the gates build the ring, not precomputed cells"). The base and
//! modulus are baked in as IMASM numerals; there is no runtime input.
//!
//!   ∈           fork the register into two arms
//!   ≻ ⋈ ⊤       orbit arm: step multiply-by-a, composing the ring, affirm
//!   ≺ ⊥ ⋈ ⊙     unit arm: hold the identity ring as the static reference
//!   ⊞           engage the paradox: both rings distinct and interlocked
//!   ∋           fuse to the relative rotation
//!   ⊡           fix the winding: the step count where the orbit arm returns to
//!               the unit arm is the order
//!
//! The relative rotation is read by the gates stepping the ring, not by
//! precomputing residues. The winding closes the factors by gcd(a^{r/2} ±1, N).
#![allow(dead_code)]
extern crate alloc;

use vox::morphism_factor::{parse_numeral, dec_of, one, cmp, modulo, mul, tape_u64};
use vox::shor_braid::factor_close_public;

include!(concat!(env!("OUT_DIR"), "/baked_inputs.rs"));

type Tape = Vec<char>;

/// The two-ring relative rotation: fork into orbit arm and unit arm, step the
/// multiply-by-a gate on the orbit arm until it returns to the held unit, and
/// return that step count. The gates build the ring; nothing is precomputed.
fn relative_rotation(a: &[char], n: &[char], cap: u64) -> Option<Tape> {
    let unit = one();                 // the unit ring, held static
    let mut orbit = modulo(a, n);     // orbit arm after one multiply gate step
    let mut k: u64 = 1;
    while cmp(&orbit, &unit) != core::cmp::Ordering::Equal {
        orbit = modulo(&mul(&orbit, a), n);   // ≻⋈ : one more multiply gate step
        k += 1;
        if k > cap { return None; }
    }
    Some(tape_u64(k))
}

fn main() -> Result<(), String> {
    let a = parse_numeral(BAKED_BASE_WORD.ok_or("bake a base word")?)?;
    let n = parse_numeral(BAKED_MODULUS_WORD.ok_or("bake a modulus word")?)?;
    println!("base {}  modulus {}", dec_of(&a), dec_of(&n));
    match relative_rotation(&a, &n, 100_000_000) {
        Some(r) => {
            print!("ring-membrane  relative-rotation {}  ", dec_of(&r));
            match factor_close_public(&a, &n, &r) {
                Ok((p, q)) => println!("factors {} x {}", dec_of(&p), dec_of(&q)),
                Err(e) => println!("no split ({e})"),
            }
        }
        None => println!("ring-membrane  relative rotation exceeds the membrane cap"),
    }
    Ok(())
}
