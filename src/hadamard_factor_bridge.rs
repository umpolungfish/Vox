//! Order-two Hadamard bridge into passive factor extraction.
//!
//! The seed object is `HadamardCarrier`: it owns N and no factor witness.  A
//! preceding phase boundary may imscribe a residue x directly, an `(a, r)`
//! phase/order relation, or one measured phase fraction `k/M` into the
//! succeeding whole object.  This bridge consumes that object and judges the
//! exposed order-two relation:
//!
//!   T -- x is a nontrivial involution whose two gcd boundaries are
//!        complementary and reconstruct N; the succeeding object is a
//!        factor-bearing `FactorCarrier`.
//!   B -- x is a nontrivial involution and both gcd boundaries distinguish N,
//!        but the two distinctions overlap rather than closing a factor pair.
//!   N -- the imscribed relation yields no new factor distinction (including
//!        x = +/-1, x^2 != 1 mod N, an odd/unusable period, or a phase sample
//!        whose continued-fraction readout certifies no period); the transformed
//!        Hadamard carrier remains.
//!   F -- a newly imscribed numeral tape is structurally malformed.
//!
//! No search for x, r, p, or q occurs here.  `descend_phase_sample` reads only
//! the continued-fraction convergents of the one supplied phase measurement and
//! certifies them against the resident modular relation.  It never walks the
//! modular orbit to discover an order and never enumerates factor candidates.
//! The passive extractor remains downstream: it only receives T, where p and q
//! are already carried by the limiting object.

use core::cmp::Ordering;

use crate::factor_extract::FactorCarrier;
use crate::hadamard_gate::{HadamardCarrier, Tape};
use crate::morphism_factor::{add, cmp, divmod, gcd, modulo, mul, one, sub, zero};
use crate::router_marks::{GStep, M_FIX, M_T};
use crate::vox::{EVALF, EVALT};

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

fn valid_numeral_tape(tape: &[char]) -> bool {
    !tape.is_empty() && tape.iter().all(|&mark| mark == EVALT || mark == EVALF)
}

/// Tape-native modular exponentiation.  The exponent is consumed LSB-first
/// directly from its IMASM numeral tape; no host-width integer is reconstructed.
fn phase_power(base: &[char], exponent: &[char], n: &[char]) -> Tape {
    let mut result = one();
    let mut power = modulo(base, n);
    for &cell in exponent {
        if cell == EVALF {
            result = modulo(&mul(&result, &power), n);
        }
        power = modulo(&mul(&power, &power), n);
    }
    result
}

/// Deterministically read the continued-fraction convergents of one measured
/// phase fraction k/M.  A denominator is returned only when the resident
/// modular relation certifies a^q = 1 (mod N).  This is phase readout, not an
/// order walk: no successive modular orbit states are generated to find q.
fn certified_period_from_sample(
    base: &[char],
    k: &[char],
    m: &[char],
    n: &[char],
) -> Option<Tape> {
    if zero(m) {
        return None;
    }

    let mut num = modulo(k, m);
    if zero(&num) {
        return None;
    }
    let mut den = m.to_vec();

    // Continued-fraction recurrence:
    // p[-2]=0, p[-1]=1; q[-2]=1, q[-1]=0.
    let z = sub(&one(), &one());
    let mut p_prev = z.clone();
    let mut p_curr = one();
    let mut q_prev = one();
    let mut q_curr = z;

    while !zero(&den) {
        let (a, rem) = divmod(&num, &den);
        let p_next = add(&mul(&a, &p_curr), &p_prev);
        let q_next = add(&mul(&a, &q_curr), &q_prev);

        if !zero(&q_next) {
            // For N > 2, a multiplicative order useful here is strictly below N.
            // Convergent denominators only increase, so there is no useful later
            // candidate once this boundary is reached.
            if cmp(&q_next, n) != Ordering::Less {
                break;
            }
            if cmp(&phase_power(base, &q_next, n), &one()) == Ordering::Equal {
                return Some(q_next);
            }
        }

        p_prev = p_curr;
        p_curr = p_next;
        q_prev = q_curr;
        q_curr = q_next;
        num = den;
        den = rem;
    }

    None
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

    /// Consume an already-exposed phase/order relation `(a, r)` and descend
    /// through its order-two boundary in the same outer invocation.
    ///
    /// The phase boundary owns discovery of `a` and `r`; this function performs
    /// no base scan and no order walk.  For an even nonzero r it reads
    /// x = a^(r/2) mod N on tapes and immediately reuses `descend_involution`.
    /// Odd, zero, or nonclosing period relations route around as N.
    pub fn descend_phase_order(self, base: &[char], period: &[char]) -> HadamardDescent {
        if !valid_numeral_tape(base) || !valid_numeral_tape(period) {
            return HadamardDescent::F;
        }

        if cmp(self.n(), &one()) != Ordering::Greater || zero(period) {
            return HadamardDescent::N(self);
        }

        let two = add(&one(), &one());
        let (half, remainder) = divmod(period, &two);
        if !zero(&remainder) || zero(&half) {
            return HadamardDescent::N(self);
        }

        let x = phase_power(base, &half, self.n());
        self.descend_involution(&x)
    }

    /// Consume one measured phase fraction k/M and, in this same outer
    /// invocation, read its continued-fraction convergents until the resident
    /// modular relation certifies one denominator as a period.  The certified
    /// relation is then handed directly to the order-two descent above.
    ///
    /// This method does not repeat the measurement, lift an uncertified
    /// denominator through multiples, scan bases, walk an orbit, or enumerate
    /// factors.  An uninformative single read therefore returns N.
    pub fn descend_phase_sample(
        self,
        base: &[char],
        k: &[char],
        m: &[char],
    ) -> HadamardDescent {
        if !valid_numeral_tape(base) || !valid_numeral_tape(k) || !valid_numeral_tape(m) {
            return HadamardDescent::F;
        }
        if cmp(self.n(), &one()) != Ordering::Greater || zero(m) {
            return HadamardDescent::N(self);
        }

        let n = self.n().to_vec();
        match certified_period_from_sample(base, k, m, &n) {
            Some(period) => self.descend_phase_order(base, &period),
            None => HadamardDescent::N(self),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::factor_extract::extract;
    use crate::morphism_factor::tape_u64;

    fn read_t(carrier: FactorCarrier) -> (Tape, Tape) {
        let readout = extract(&carrier).unwrap();
        assert_eq!(readout.transforms, 0);
        (readout.p.0, readout.q.0)
    }

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
        assert_eq!(read_t(factor_carrier), (tape_u64(3), tape_u64(5)));
    }

    #[test]
    fn phase_order_relation_closes_in_the_same_outer_descent() {
        // ord_15(2) = 4.  The upstream phase relation supplies (2,4), not p/q.
        // The bridge reads x = 2^(4/2) = 4 mod 15 and closes 3 * 5.
        let n = tape_u64(15);
        let seed = HadamardCarrier::new(&n).unwrap();
        assert_eq!(seed.n(), n.as_slice());
        assert_eq!(seed.source(), [EVALT]);

        let factor_carrier = match seed.descend_phase_order(&tape_u64(2), &tape_u64(4)) {
            HadamardDescent::T(carrier) => carrier,
            other => panic!("expected T factor carrier, got {other:?}"),
        };
        assert_eq!(read_t(factor_carrier), (tape_u64(3), tape_u64(5)));
    }

    #[test]
    fn phase_order_relation_is_not_specific_to_order_four() {
        // ord_21(2) = 6.  The half-period boundary gives x = 2^3 = 8;
        // x^2 = 1 mod 21 and the passive readout receives 3 * 7.
        let seed = HadamardCarrier::new(&tape_u64(21)).unwrap();
        let factor_carrier = match seed.descend_phase_order(&tape_u64(2), &tape_u64(6)) {
            HadamardDescent::T(carrier) => carrier,
            other => panic!("expected T factor carrier, got {other:?}"),
        };
        assert_eq!(read_t(factor_carrier), (tape_u64(3), tape_u64(7)));
    }

    #[test]
    fn exact_phase_sample_closes_without_an_order_walk() {
        // One exact Fourier support point for r=4: k/M = 4/16 = 1/4.
        // The CF denominator 4 is certified by 2^4 = 1 mod 15 and immediately
        // feeds the order-two boundary; no orbit is walked to discover r.
        let seed = HadamardCarrier::new(&tape_u64(15)).unwrap();
        let factor_carrier = match seed.descend_phase_sample(
            &tape_u64(2),
            &tape_u64(4),
            &tape_u64(16),
        ) {
            HadamardDescent::T(carrier) => carrier,
            other => panic!("expected T factor carrier, got {other:?}"),
        };
        assert_eq!(read_t(factor_carrier), (tape_u64(3), tape_u64(5)));
    }

    #[test]
    fn one_near_peak_phase_sample_certifies_order_six() {
        // 43/256 is a one-shot phase read near 1/6.  Its continued-fraction
        // sequence contains 1/6; q=6 certifies because 2^6 = 1 mod 21.  The
        // same outer descent then exposes x=2^3=8 and closes 3 * 7.
        let seed = HadamardCarrier::new(&tape_u64(21)).unwrap();
        let factor_carrier = match seed.descend_phase_sample(
            &tape_u64(2),
            &tape_u64(43),
            &tape_u64(256),
        ) {
            HadamardDescent::T(carrier) => carrier,
            other => panic!("expected T factor carrier, got {other:?}"),
        };
        assert_eq!(read_t(factor_carrier), (tape_u64(3), tape_u64(7)));
    }

    #[test]
    fn uninformative_single_phase_read_routes_around_as_n() {
        // 85/256 reduces toward 1/3 for this register, but q=3 does not close
        // 2 mod 21.  With no denominator lifting or second shot, this read is N.
        let seed = HadamardCarrier::new(&tape_u64(21)).unwrap();
        assert!(matches!(
            seed.descend_phase_sample(&tape_u64(2), &tape_u64(85), &tape_u64(256)),
            HadamardDescent::N(_)
        ));
    }

    #[test]
    fn odd_or_nonclosing_phase_relations_route_around_as_n() {
        let n = tape_u64(15);
        for period in [3u64, 2] {
            let seed = HadamardCarrier::new(&n).unwrap();
            assert!(matches!(
                seed.descend_phase_order(&tape_u64(2), &tape_u64(period)),
                HadamardDescent::N(_)
            ));
        }
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

        let seed = HadamardCarrier::new(&tape_u64(15)).unwrap();
        assert_eq!(seed.descend_phase_order(&['?'], &tape_u64(4)), HadamardDescent::F);

        let seed = HadamardCarrier::new(&tape_u64(15)).unwrap();
        assert_eq!(seed.descend_phase_order(&tape_u64(2), &['?']), HadamardDescent::F);

        let seed = HadamardCarrier::new(&tape_u64(15)).unwrap();
        assert_eq!(
            seed.descend_phase_sample(&['?'], &tape_u64(4), &tape_u64(16)),
            HadamardDescent::F
        );

        let seed = HadamardCarrier::new(&tape_u64(15)).unwrap();
        assert_eq!(
            seed.descend_phase_sample(&tape_u64(2), &['?'], &tape_u64(16)),
            HadamardDescent::F
        );

        let seed = HadamardCarrier::new(&tape_u64(15)).unwrap();
        assert_eq!(
            seed.descend_phase_sample(&tape_u64(2), &tape_u64(4), &['?']),
            HadamardDescent::F
        );
    }
}
