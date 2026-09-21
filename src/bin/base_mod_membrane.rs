//! base_mod_membrane — the base-ring-against-modulus-ring winding read, baked.
//!
//! Realizes the auto-designed word ⊢∈≻⊤≺⊥⊞⋈⊙∋⊡⊣⋈⊙⊡ (ob3ect: "the two rings are
//! the baked base word and modulus word, their interference the winding, read
//! per-coordinate/banked rather than the saturating aggregate"). Base and
//! modulus are baked in; no runtime input.
//!
//!   ∈       fork into the base-word trajectory and the modulus-word trajectory
//!   ≻ ⊤     base ring rotates forward: one multiply-by-a step, affirmed
//!   ≺ ⊥     modulus ring counter-rotates: the fixed residue cycle it turns in
//!   ⊞       asymmetric participation: the two rings hold distinct phase counts
//!   ⋈ ⊙ ∋   chain, self-reference, fuse to the relative rotation
//!   ⊡       fix the winding
//!
//! The relative rotation of the base ring inside the modulus cycle is the
//! order; it closes the factors. Read per coordinate (one residue at a step),
//! never the saturating aggregate.
#![allow(dead_code)]
extern crate alloc;

use vox::morphism_factor::{parse_numeral, dec_of, one, cmp, modulo, mul, tape_u64};
use vox::shor_braid::factor_close_public;

include!(concat!(env!("OUT_DIR"), "/baked_inputs.rs"));

fn main() -> Result<(), String> {
    let a = parse_numeral(BAKED_BASE_WORD.ok_or("bake a base word")?)?;
    let n = parse_numeral(BAKED_MODULUS_WORD.ok_or("bake a modulus word")?)?;
    println!("base {}  modulus {}", dec_of(&a), dec_of(&n));

    // base ring rotates by one multiply-by-a per coordinate inside the modulus
    // cycle; the relative rotation completes when it lands back on the unit.
    let unit = one();
    let mut base_ring = modulo(&a, &n);
    let mut k: u64 = 1;
    let cap: u64 = 100_000_000;
    while cmp(&base_ring, &unit) != core::cmp::Ordering::Equal {
        base_ring = modulo(&mul(&base_ring, &a), &n);
        k += 1;
        if k > cap {
            println!("base_mod-membrane  relative rotation exceeds the membrane cap");
            return Ok(());
        }
    }
    let r = tape_u64(k);
    print!("base_mod-membrane  relative-rotation {}  ", dec_of(&r));
    match factor_close_public(&a, &n, &r) {
        Ok((p, q)) => println!("factors {} x {}", dec_of(&p), dec_of(&q)),
        Err(e) => println!("no split ({e})"),
    }
    Ok(())
}
