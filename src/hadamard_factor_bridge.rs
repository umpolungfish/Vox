//! Order-two Hadamard bridge into passive factor extraction.
//!
//! The seed object is `HadamardCarrier`: it owns N and no factor witness.  A
//! preceding phase boundary may imscribe a residue x into that whole object.
//! This bridge consumes the resulting carrier and judges the exposed relation
//! x^2 = 1 (mod N):
//!
//!   T -- x is a nontrivial involution whose two gcd boundaries are
//!        complementary and reconstruct N; the succeeding object is a
//!        factor-bearing `FactorCarrier`.
//!   B -- x is a nontrivial involution and both gcd boundaries distinguish N,
//!        but the two distinctions overlap rather than closing a factor pair.
//!   N -- the imscribed relation yields no new factor distinction (including
//!        x = +/-1 or x^2 != 1 mod N); the transformed Hadamard carrier remains.
//!   F -- the newly imscribed tape is structurally malformed.
//!
//! No search for x, p, or q occurs here.  In particular, the passive extractor
//! remains downstream: it only receives T, where p and q are already carried by
//! the limiting object.

use core::cmp::Ordering;

use crate::factor_extract::FactorCarrier;
use crate::hadamard_gate::{HadamardCarrier, Tape};
use crate::morphism_factor::{add, cmp, gcd, modulo, mul, one, sub};
use crate::router_marks::{GStep, M_FIX, M_T};

/// Productive but not yet terminal order-two split.
#[derive(Clone, PartialEq, Debug)]
pub struct HadamardFork {
    pub carrier: HadamardCarrier,
    pub left: Tape,
    pub right: Tape,
}

/// FOUR judgment of one order-two descent.
#[derive(Clone, PartialEq, Debug)]
pub enum HadamardDescent {
    T(FactorCarrier),
    B(HadamardFork),
    N(HadamardCarrier),
    F,
}

fn closing_trace() -> [GStep; 1] {
    [GStep {
        repr: '⋈',
        judgment: M_T,
        recognised: M_T,
        next: M_FIX,
        applied_word: "⊢⊙⊡⊣".chars().collect(),
    }]
}

impl HadamardCarrier {
    /// Consume the current whole object, imscribe an order-two residue, and
    /// descend to the next semantic object.
    ///
    /// This operation does not discover the residue.  The role of the upstream
    /// Hadamard/phase machinery is to expose that relation from N.  Once a
    /// nontrivial square root of one is resident, the two factor boundaries are
    /// read directly as gcd(x-1,N) and gcd(x+1,N).
    pub fn descend_involution(self, relation: &[char]) -> HadamardDescent {
        let carrier = match self.imscribe_source(relation) {
            Ok(carrier) => carrier,
            Err(_) => return HadamardDescent::F,
        };

        let n = carrier.n().to_vec();
        if cmp(&n, &one()) != Ordering::Greater {
            return HadamardDescent::N(carrier);
        }

        // Canonicalize the exposed relation inside the current modulus before
        // judging it.  This is still the succeeding whole object, not a host
        // side-channel: both N and x remain IMASM numeral tapes.
        let x = modulo(carrier.source(), &n);
        let carrier = match carrier.imscribe_source(&x) {
            Ok(carrier) => carrier,
            Err(_) => return HadamardDescent::F,
        };

        if cmp(&modulo(&mul(&x, &x), &n), &one()) != Ordering::Equal {
            return HadamardDescent::N(carrier);
        }

        let minus_one = sub(&n, &one());
        if cmp(&x, &one()) == Ordering::Equal || cmp(&x, &minus_one) == Ordering::Equal {
            return HadamardDescent::N(carrier);
        }

        let left = gcd(sub(&x, &one()), n.clone());
        let right = gcd(add(&x, &one()), n.clone());
        let nontrivial = |d: &[char]| {
            cmp(d, &one()) == Ordering::Greater && cmp(d, &n) == Ordering::Less
        };

        if !nontrivial(&left) || !nontrivial(&right) {
            return HadamardDescent::N(carrier);
        }

        if cmp(&mul(&left, &right), &n) == Ordering::Equal {
            let trace = closing_trace();
            return match FactorCarrier::from_trace(&n, &left, &right, &trace) {
                Ok(factor_carrier) => HadamardDescent::T(factor_carrier),
                Err(_) => HadamardDescent::F,
            };
        }

        HadamardDescent::B(HadamardFork { carrier, left, right })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::factor_extract::extract;
    use crate::morphism_factor::tape_u64;

    #[test]
    fn nontrivial_involution_closes_into_the_passive_factor_extractor() {
        // 4^2 = 1 mod 15, while 4 != +/-1.  The seed carries only N; p and q
        // appear only after the order-two distinction closes.
        let n = tape_u64(15);
        let seed = HadamardCarrier::new(&n).unwrap();
        let factor_carrier = match seed.descend_involution(&tape_u64(4)) {
            HadamardDescent::T(carrier) => carrier,
            other => panic!("expected T factor carrier, got {other:?}"),
        };

        assert_eq!(factor_carrier.n, n);
        let readout = extract(&factor_carrier).unwrap();
        assert_eq!(readout.p.0, tape_u64(3));
        assert_eq!(readout.q.0, tape_u64(5));
        assert_eq!(readout.transforms, 0);
    }

    #[test]
    fn trivial_or_noninvolutive_relations_route_around_as_n() {
        let n = tape_u64(15);
        for x in [1u64, 14, 2] {
            let seed = HadamardCarrier::new(&n).unwrap();
            assert!(matches!(
                seed.descend_involution(&tape_u64(x)),
                HadamardDescent::N(_)
            ));
        }
    }

    #[test]
    fn overlapping_order_two_split_is_productive_b() {
        // 5^2 = 1 mod 12, but the two boundaries overlap on 2:
        // gcd(4,12)=4 and gcd(6,12)=6.  Their product is 24, not 12, so the
        // relation is productive but is not yet a terminal factor carrier.
        let seed = HadamardCarrier::new(&tape_u64(12)).unwrap();
        match seed.descend_involution(&tape_u64(5)) {
            HadamardDescent::B(fork) => {
                assert_eq!(fork.left, tape_u64(4));
                assert_eq!(fork.right, tape_u64(6));
                assert_eq!(fork.carrier.n(), tape_u64(12).as_slice());
            }
            other => panic!("expected B fork, got {other:?}"),
        }
    }

    #[test]
    fn malformed_relation_is_f() {
        let seed = HadamardCarrier::new(&tape_u64(15)).unwrap();
        assert_eq!(seed.descend_involution(&['?']), HadamardDescent::F);
    }
}
