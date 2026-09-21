//! cylinder_membrane — the moduli as rotating cylinders, baked.
//!
//! n couples two rotors, the register mod p and the register mod q. Multiplying
//! by a base spins both at their own periods. The factor transports out the
//! moment one rotor returns to 1 while the other has not: then a^k − 1 is
//! divisible by that prime alone, and gcd(a^k − 1, n) is the split. The
//! membrane spins many cylinders (bases generated inside, not input) and reads
//! off the first alignment. The modulus is baked in; there is no runtime input.
#![allow(dead_code)]
extern crate alloc;

use alloc::vec::Vec;
use vox::morphism_factor::{parse_numeral, dec_of, one, cmp, modulo, mul, sub, gcd, tape_u64, divmod};

include!(concat!(env!("OUT_DIR"), "/baked_inputs.rs"));

type Tape = Vec<char>;

fn main() -> Result<(), String> {
    let n = parse_numeral(BAKED_MODULUS_WORD.ok_or("bake a modulus word")?)?;
    let one_t = one();
    // reach and cylinder count come from the baked width if present, else set here.
    let (bases, cap): (u64, u64) = match BAKED_WIDTH_WORD {
        Some(w) => { let b = dec_of(&parse_numeral(w)?).parse::<u64>().unwrap_or(300); (b, 60_000) }
        None => (300, 60_000),
    };
    println!("modulus {}  spinning {} cylinders, reach {}", dec_of(&n), bases, cap);

    for a_val in 2..bases {
        let a = tape_u64(a_val);
        // a base sharing a factor with n is a cylinder already seized.
        let g0 = gcd(a.clone(), n.clone());
        if cmp(&g0, &one_t) != core::cmp::Ordering::Equal {
            let (q, _r) = divmod(&n, &g0);
            println!("factor {} x {}  [base {a_val} shares a factor]", dec_of(&g0), dec_of(&q));
            return Ok(());
        }
        // spin this cylinder: x = a^k mod n, watch gcd(x - 1, n) at each turn.
        let mut x = modulo(&a, &n);
        for k in 1..cap {
            x = modulo(&mul(&x, &a), &n);
            if cmp(&x, &one_t) == core::cmp::Ordering::Equal { break; } // rotor closed, no split this base
            let d = sub(&x, &one_t);
            let g = gcd(d, n.clone());
            if cmp(&g, &one_t) != core::cmp::Ordering::Equal && cmp(&g, &n) != core::cmp::Ordering::Equal {
                let (q, _r) = divmod(&n, &g);
                println!("factor {} x {}  [cylinder base {a_val} aligned at turn {k}]",
                    dec_of(&g), dec_of(&q));
                return Ok(());
            }
        }
    }
    println!("no cylinder aligned within reach");
    Ok(())
}
