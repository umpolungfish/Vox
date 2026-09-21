//! reduce_delete_membrane — the winding tracked by the flip coordinate, baked.
//!
//! Realizes the auto-designed word ⊢⊙∈≻⊤⋈≺⊥⋈⊞∋⊡⋈⊙⊣ (ob3ect: "two short words that
//! differ by one ⊤/⊥ deposit, ∈∋⊤≻⊡ vs ∈∋⊥≻⊡, carried through the modular gate
//! per orbit step; the coordinate that flips tracks the winding"). Base and
//! modulus are baked in; no runtime input.
//!
//! Each orbit step carries the reduce word ∈∋⊤≻⊡ or the delete word ∈∋⊥≻⊡: ⊤
//! when the multiply-by-a gate wrapped the modulus (a reduction fired), ⊥ when
//! it did not. The ⊤/⊥ sequence is periodic with the order, so the coordinate
//! that flips tracks the winding, and the return to the unit fixes it.
#![allow(dead_code)]
extern crate alloc;

use alloc::string::String;
use vox::morphism_factor::{parse_numeral, dec_of, one, cmp, modulo, mul, tape_u64};
use vox::shor_braid::factor_close_public;

include!(concat!(env!("OUT_DIR"), "/baked_inputs.rs"));

fn main() -> Result<(), String> {
    let a = parse_numeral(BAKED_BASE_WORD.ok_or("bake a base word")?)?;
    let n = parse_numeral(BAKED_MODULUS_WORD.ok_or("bake a modulus word")?)?;
    println!("base {}  modulus {}", dec_of(&a), dec_of(&n));

    let unit = one();
    let mut cur = modulo(&a, &n);
    let mut flips = String::new();       // the reduce/delete coordinate per step
    let mut k: u64 = 1;
    let cap: u64 = 4_000_000;
    loop {
        // reduce word (⊤) fired if this step's product wrapped the modulus;
        // delete word (⊥) if the raw product was already below the modulus.
        let raw = mul(&cur, &a);
        let reduced = cmp(&raw, &n) != core::cmp::Ordering::Less;
        if flips.len() < 64 { flips.push(if reduced { '⊤' } else { '⊥' }); }
        cur = modulo(&raw, &n);
        k += 1;
        if cmp(&cur, &unit) == core::cmp::Ordering::Equal { break; }
        if k > cap {
            println!("reduce_delete-membrane  flip coordinate did not close within the cap");
            return Ok(());
        }
    }
    let r = tape_u64(k);
    println!("reduce_delete-membrane  flip-coordinate {}…", flips);
    print!("reduce_delete-membrane  winding {}  ", dec_of(&r));
    match factor_close_public(&a, &n, &r) {
        Ok((p, q)) => println!("factors {} x {}", dec_of(&p), dec_of(&q)),
        Err(e) => println!("no split ({e})"),
    }
    Ok(())
}
