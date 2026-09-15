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

use crate::vox::{
    verdict, AFWD, AREV, CLINK, ENGAGR, EVALF, EVALT, FFUSE, FSPLIT, IFIX, IMSCRIB, TANCH, VINIT,
};
use alloc::vec;
use alloc::vec::Vec;

type Word = Vec<char>;

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
