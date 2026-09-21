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

/// a^e mod n over tapes, square-and-multiply.
fn powm(a: &[char], e: u64, n: &[char]) -> Tape {
    let mut result = one();
    let mut base = modulo(a, n);
    let mut e = e;
    while e > 0 {
        if e & 1 == 1 { result = modulo(&mul(&result, &base), n); }
        base = modulo(&mul(&base, &base), n);
        e >>= 1;
    }
    result
}

/// The two leapers on the ring, meeting in about sqrt(order) rather than the
/// full walk. Inspired by the native leaping winding: each point takes a leap
/// of a pseudo-random exponent drawn from the point itself, so the walk is a
/// deterministic function and two leapers on it must meet. Their meeting gives
/// a^E == a^E', so the order divides E - E'. Holds only the two leapers, no
/// table; the exterior fork sets them, the fuse resolves the crossing.
fn leaping_rotation(a: &[char], n: &[char], steps: u64) -> Option<Tape> {
    let jump = |x: &[char]| -> u64 {
        // the point chooses the leap: a small nonzero exponent from its low limbs
        let mut h: u64 = 1469598103934665603;
        for &c in x.iter().take(24) { h = (h ^ (c as u64)).wrapping_mul(1099511628211); }
        1 + (h % 1024)
    };
    // tortoise/hare over y -> a^{jump(y)} * y ; track the exponent sum on each.
    let start = modulo(a, n);
    let (mut t, mut te) = (start.clone(), 1u64);
    let (mut h, mut he) = (start.clone(), 1u64);
    for _ in 0..steps {
        let j = jump(&t); t = modulo(&mul(&powm(a, j, n), &t), n); te = te.wrapping_add(j);
        for _ in 0..2 {
            let j = jump(&h); h = modulo(&mul(&powm(a, j, n), &h), n); he = he.wrapping_add(j);
        }
        if cmp(&t, &h) == core::cmp::Ordering::Equal {
            // a^te == a^he  =>  order divides |he - te|
            let diff = if he > te { he - te } else { te - he };
            if diff == 0 { return None; }
            // reduce the multiple to the true order by stripping small factors
            let mut r = diff;
            while r % 2 == 0 && cmp(&powm(a, r / 2, n), &one()) == core::cmp::Ordering::Equal { r /= 2; }
            let mut p = 3u64;
            while p * p <= r { while r % p == 0 && cmp(&powm(a, r / p, n), &one()) == core::cmp::Ordering::Equal { r /= p; } p += 2; }
            if cmp(&powm(a, r, n), &one()) == core::cmp::Ordering::Equal {
                return Some(tape_u64(r));
            }
            return Some(tape_u64(diff));
        }
    }
    None
}

fn main() -> Result<(), String> {
    let a = parse_numeral(BAKED_BASE_WORD.ok_or("bake a base word")?)?;
    let n = parse_numeral(BAKED_MODULUS_WORD.ok_or("bake a modulus word")?)?;
    println!("base {}  modulus {}", dec_of(&a), dec_of(&n));
    match leaping_rotation(&a, &n, 20_000_000) {
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
