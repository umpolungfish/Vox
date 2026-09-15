//! perfect_membrane.rs — the membrane as matched circuitry, not as search.
//!
//! A membrane closes when mu∘delta = id: split the object, transform it, fuse it
//! back, and recover what you started with. The sieve membranes reach that closure
//! by collecting relations until a dependency appears. A PERFECT membrane reaches
//! it by construction: every delta (a fork, FSPLIT ∈) is wired to its own mu (a
//! fuse, FFUSE ∋), so split-then-fuse is the identity by the shape of the circuit.
//!
//! The tower has a depth n >= 2 (two is the base Frobenius cell, one split wired to
//! one fuse) and no maximum. Each level forks down and, through CLINK bridges,
//! reconnects to every deeper level and back, the all-to-all lattice, so no level
//! sits on a single path. The closure auditor `vox::verdict` reads this: matched
//! forks with work in their interior return T, an unmatched fork would return B and
//! an over-fuse F. A perfect membrane returns T at every depth with zero surplus.

use crate::morphism_factor::{cmp, dec_of, mul, trim};
use crate::vox::{
    verdict, AFWD, AREV, CLINK, ENGAGR, EVALF, EVALT, FFUSE, FSPLIT, IFIX, IMSCRIB, TANCH, VINIT,
};
use alloc::format;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

type Word = Vec<char>;

// ---- executable delta / mu over the bit-tape ----
// The tape is little-endian bits: index i is bit i, EVALF = 1, EVALT = 0.

/// delta: deinterlace the bits into two lanes. Lane 0 takes the even-index bits,
/// lane 1 the odd. This is the split the tower forks on.
fn delta(v: &[char]) -> (Word, Word) {
    let mut a = Word::new();
    let mut b = Word::new();
    for (i, &bit) in v.iter().enumerate() {
        if i % 2 == 0 {
            a.push(bit);
        } else {
            b.push(bit);
        }
    }
    (trim(a), trim(b))
}

/// mu: interlace the two lanes back, the exact inverse of delta. Bit 2j comes from
/// lane 0, bit 2j+1 from lane 1. mu(delta(v)) = v on the nose.
fn mu(a: &[char], b: &[char]) -> Word {
    let n = a.len().max(b.len());
    let mut out = Word::new();
    for j in 0..n {
        out.push(*a.get(j).unwrap_or(&EVALT));
        out.push(*b.get(j).unwrap_or(&EVALT));
    }
    trim(out)
}

/// Run a value through a depth-n perfect membrane and report each stage: the split
/// lanes at every level, the transform carried at the core (here the product of the
/// two deepest lanes, a real computed quantity), and the fuse back up, with the
/// mu∘delta = id recovery checked against the input at every level.
pub fn run(value: &[char], depth: usize) -> String {
    let n = depth.max(2);
    let v = trim(value.to_vec());
    let mut out = String::new();
    out.push_str(&format!("value in: {}\n", dec_of(&v)));

    // fork down n levels, always following lane 0, keeping each sibling to fuse back
    let mut cur = v.clone();
    let mut siblings: Vec<Word> = Vec::new();
    for level in 1..=n {
        let (a, b) = delta(&cur);
        out.push_str(&format!(
            "  delta L{level}: lane0={} lane1={}\n",
            dec_of(&a),
            dec_of(&b)
        ));
        siblings.push(b);
        cur = a;
    }

    // the transform at the core: multiply the deepest lane by its last sibling, a
    // genuine computation the membrane carries while it stays closed
    let product = mul(&cur, siblings.last().unwrap());
    out.push_str(&format!("  core transform: lane product = {}\n", dec_of(&product)));

    // fuse back up: mu recombines each level, recovering the input exactly
    for level in (1..=n).rev() {
        let b = siblings.pop().unwrap();
        cur = mu(&cur, &b);
        out.push_str(&format!("  mu    L{level}: recombined = {}\n", dec_of(&cur)));
    }

    let closed = cmp(&cur, &v) == core::cmp::Ordering::Equal;
    out.push_str(&format!(
        "value out: {}\n  mu∘delta = id : {}\n",
        dec_of(&cur),
        if closed { "CLOSED (identity recovered)" } else { "LEAK" }
    ));
    out
}

/// The transformed object at the core: advance, engage the paradox, imscribe. This
/// is the "transformed" in "mu∘delta = id over a transformed object" — without a
/// real transform the closure is trivial.
fn core() -> Word {
    vec![AFWD, EVALT, AREV, EVALF, ENGAGR, IMSCRIB]
}

/// Depth-n perfect membrane word: VINIT, then n forks each bridged to the deeper
/// levels (delta with its all-to-all cross-link), the transformed core, then n
/// fuses (mu), latched with IFIX and closed with TANCH. Forks and fuses are equal
/// in number, so the auditor sees no surplus and the round trip is the identity.
pub fn perfect_membrane(depth: usize) -> Word {
    let n = depth.max(2);
    let mut w = vec![VINIT];
    for _ in 0..n {
        w.push(FSPLIT); // delta: fork this level
        w.push(CLINK); // bridge it to the deeper levels (the lateral link)
    }
    w.extend(core());
    for _ in 0..n {
        w.push(FFUSE); // mu: fuse the matching level back
    }
    w.push(IFIX);
    w.push(TANCH);
    w
}

/// Read the closure of a depth-n perfect membrane with the instrument: its verdict
/// mark, and the fork/fuse surplus (zero when every delta has its mu).
pub fn report(depth: usize) -> (char, i32, Word) {
    let w = perfect_membrane(depth);
    let of = crate::vox::open_forks(&w);
    (verdict(&w), of.surplus, w)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn closes_with_identity_at_every_depth() {
        // mu∘delta = id holds by construction: T verdict, zero surplus, 2..=32.
        for n in 2..=32 {
            let (v, surplus, _) = report(n);
            assert_eq!(surplus, 0, "depth {n}: forks and fuses must match");
            assert_eq!(v, 'T', "depth {n}: perfect membrane must close with work");
        }
    }
}
