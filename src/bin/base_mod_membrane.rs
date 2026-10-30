//! base_mod_membrane — the base-ring-against-modulus-ring winding read, baked.
//!
//! Realizes the auto-designed word ⊢∈≻⊤≺⊥⊞⋈⊙∋⊡⊣⋈⊙⊡ (ob3ect: "the two rings are
//! the base word and baked modulus word, their interference the winding, read
//! one residue per coordinate rather than a saturating aggregate"). The
//! modulus is baked in; base 2 is the fixed default unless a base is baked.
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
//! open each residue against N before fuse, never the saturating aggregate.
#![deny(warnings)]
#![allow(dead_code)]
extern crate alloc;

use vox::morphism_factor::{parse_numeral, decimal_to_tape, dec_of, one, cmp, modulo, mul, gcd, divmod, add};
use vox::shor_braid::{factor_close_at_residue, factor_close_public};

include!(concat!(env!("OUT_DIR"), "/baked_inputs.rs"));

fn main() -> Result<(), String> {
    #[cfg(feature = "hosted")]
    ::vox::ordering::cli::intercept();
    let a = match BAKED_BASE_WORD {
        Some(word) => parse_numeral(word)?,
        None => decimal_to_tape("2").ok_or("fixed base 2 did not encode as an IMASM numeral")?,
    };
    let n = parse_numeral(BAKED_MODULUS_WORD.ok_or("bake a modulus word")?)?;
    println!("base {}  modulus {}", dec_of(&a), dec_of(&n));

    let unit = one();
    let common = gcd(a.clone(), n.clone());
    if cmp(&common, &unit) == core::cmp::Ordering::Greater {
        if cmp(&common, &n) == core::cmp::Ordering::Less {
            let (quotient, remainder) = divmod(&n, &common);
            if cmp(&remainder, &unit) != core::cmp::Ordering::Equal {
                return Err("base/modulus gcd failed exact quotient closure".into());
            }
            println!("base_mod-membrane  factors {} x {} [base gcd]", dec_of(&common), dec_of(&quotient));
        } else {
            println!("base_mod-membrane  base shares the full modulus; no split");
        }
        return Ok(());
    }
    let mut base_ring = modulo(&a, &n);
    let mut k = unit.clone();
    if let Some((opening,p,q))=factor_close_at_residue(&base_ring,&n)?{
        println!("base_mod-membrane  coordinate {} residue {} opens {}: factors {} x {}",
            dec_of(&k),dec_of(&base_ring),opening,dec_of(&p),dec_of(&q));
        return Ok(());
    }
    while cmp(&base_ring, &unit) != core::cmp::Ordering::Equal {
        base_ring = modulo(&mul(&base_ring, &a), &n);
        k = add(&k, &unit);
        if let Some((opening,p,q))=factor_close_at_residue(&base_ring,&n)?{
            println!("base_mod-membrane  coordinate {} residue {} opens {}: factors {} x {}",
                dec_of(&k),dec_of(&base_ring),opening,dec_of(&p),dec_of(&q));
            return Ok(());
        }
    }
    print!("base_mod-membrane  relative-rotation {}  ", dec_of(&k));
    match factor_close_public(&a, &n, &k) {
        Ok((p, q)) => println!("factors {} x {}", dec_of(&p), dec_of(&q)),
        Err(e) => println!("no split ({e})"),
    }
    Ok(())
}
