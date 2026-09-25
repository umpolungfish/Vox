//! Sequential dyadic partner observations. No register enumeration or supplied
//! order. A collision proves a return exponent, which may be a multiple of order.
use alloc::collections::btree_map::Entry;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use core::cmp::Ordering;
use num_bigint::BigUint;
use num_traits::Zero;
use crate::morphism_factor::{cmp, gcd, modulo, mul, one, sub};
type Tape = Vec<char>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReturnRelation {
    pub earlier: Tape,
    pub later: Tape,
    pub residue: Tape,
    pub return_exponent: Tape,
    /// Adjacent dyadic residues whose quotient is a^(return_exponent / 2).
    /// Absent when the return exponent is odd or has no positive half-step.
    pub half_residues: Option<(Tape, Tape)>,
}

/// The one coefficient object behind the codec, frame sweep, and modular
/// support reads. `bits_le[i]` is the coefficient of X^i, so evaluating this
/// polynomial at 2 reconstructs the baked integer exactly.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SupportPolynomial {
    bits_le: Vec<bool>,
}

/// A factor target is a zero of the same support polynomial modulo a proper
/// divisor of its value. The phase register is retained as the witness point.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SupportClosure {
    pub phase_index: usize,
    pub phase_register: Tape,
    pub polynomial_residue: Tape,
    pub p: Tape,
    pub q: Tape,
}

impl SupportPolynomial {
    fn from_tape(n: &[char]) -> Self {
        Self { bits_le: n.iter().map(|&mark| mark == crate::vox::EVALF).collect() }
    }

    /// Horner-evaluate the one support polynomial through a width-bit frame.
    /// Every width is an equivalent regrouping of the same coefficient stream.
    fn evaluate_frame(&self, x: &BigUint, modulus: &BigUint, width: usize) -> BigUint {
        debug_assert!((2..=8).contains(&width));
        let mut powers = Vec::with_capacity(width);
        powers.push(BigUint::from(1u8));
        for index in 1..width {
            powers.push((&powers[index - 1] * x) % modulus);
        }
        let frame_base = (&powers[width - 1] * x) % modulus;
        let mut value = BigUint::from(0u8);
        for group in self.bits_le.chunks(width).rev() {
            let mut symbol = BigUint::from(0u8);
            for (position, bit) in group.iter().enumerate() {
                if *bit { symbol += &powers[position]; }
            }
            value = (value * &frame_base + symbol) % modulus;
        }
        value
    }

    fn target(&self, x: &BigUint, n: &BigUint, width: usize) -> Option<(BigUint, BigUint, BigUint)> {
        let residue = self.evaluate_frame(x, n, width);
        let factor = biguint_gcd(residue.clone(), n.clone());
        if factor <= BigUint::from(1u8) || factor >= *n { return None; }
        let cofactor = n / &factor;
        if &cofactor * &factor != *n { return None; }
        Some((residue, factor, cofactor))
    }
}

#[allow(dead_code)] // Retained for explicit, non-hot-path relation replay.
fn power(a: &[char], exponent: &[char], n: &[char]) -> Tape {
    let mut result = one();
    let mut base = modulo(a, n);
    for (i, &bit) in exponent.iter().enumerate() {
        if bit == crate::vox::EVALF { result = modulo(&mul(&result, &base), n); }
        if i + 1 < exponent.len() { base = modulo(&mul(&base, &base), n); }
    }
    result
}

impl ReturnRelation {
    #[allow(dead_code)] // Independent verifier; observe() relies on its recurrence invariant.
    pub fn verify(&self, a: &[char], n: &[char]) -> bool {
        if cmp(n, &one()) != Ordering::Greater
            || gcd(a.to_vec(), n.to_vec()) != one()
            || cmp(&self.later, &self.earlier) != Ordering::Greater {
            return false;
        }
        self.return_exponent == sub(&self.later, &self.earlier)
            && power(a, &self.earlier, n) == self.residue
            && power(a, &self.later, n) == self.residue
            && power(a, &self.return_exponent, n) == one()
    }
}

pub struct Partners {
    n: BigUint,
    residue: BigUint,
    support: SupportPolynomial,
    // Store dynamic residues and only the observation index. Storing each
    // ever-growing exponent tape made total memory quadratic in observations;
    // serializing each residue to bytes added a second representation to hash.
    seen: BTreeMap<BigUint, SeenResidue>,
    pub squarings: usize,
    previous_residue: Option<BigUint>,
}

struct SeenResidue {
    index: Option<usize>,
    half_residue: Option<BigUint>,
}

fn tape_to_biguint(tape: &[char]) -> BigUint {
    let mut packed = vec![0u8; (tape.len() + 7) / 8];
    for (index, &mark) in tape.iter().enumerate() {
        if mark == crate::vox::EVALF {
            packed[index / 8] |= 1u8 << (index % 8);
        }
    }
    BigUint::from_bytes_le(&packed)
}

fn biguint_gcd(mut left: BigUint, mut right: BigUint) -> BigUint {
    while !right.is_zero() {
        let remainder = &left % &right;
        left = right;
        right = remainder;
    }
    left
}

fn biguint_to_tape(value: &BigUint) -> Tape {
    if value.is_zero() { return vec![crate::vox::EVALT]; }
    let packed = value.to_bytes_le();
    let mut tape = Vec::with_capacity(packed.len() * 8);
    for byte in packed {
        for bit in 0..8 {
            tape.push(if (byte >> bit) & 1 == 1 { crate::vox::EVALF } else { crate::vox::EVALT });
        }
    }
    while tape.last() == Some(&crate::vox::EVALT) { tape.pop(); }
    tape
}

fn power_of_two_exponent(index: usize) -> Tape {
    let mut exponent = vec![crate::vox::EVALT; index + 1];
    exponent[index] = crate::vox::EVALF;
    exponent
}

impl Partners {
    pub fn new(a: Tape, n: Tape) -> Result<Self, String> {
        Self::new_inner(a, n, true)
    }

    /// Runtime entry for a phase base whose unit property was checked while
    /// its IMASM numeral was baked into the contained binary.
    pub fn new_from_validated_bake(a: Tape, n: Tape) -> Result<Self, String> {
        Self::new_inner(a, n, false)
    }

    fn new_inner(a: Tape, n: Tape, check_unit: bool) -> Result<Self, String> {
        if cmp(&n, &one()) != Ordering::Greater {
            return Err("partners require N > 1 and a coprime base".into());
        }
        let n_big = tape_to_biguint(&n);
        let a_big = tape_to_biguint(&a);
        if check_unit && biguint_gcd(a_big.clone(), n_big.clone()) != BigUint::from(1u8) {
            return Err("partners require N > 1 and a coprime base".into());
        }
        let mut seen = BTreeMap::new();
        // The initial state is 1 = a^0, distinguished from a^(2^i).
        seen.insert(BigUint::from(1u8), SeenResidue { index: None, half_residue: None });
        let residue = a_big % &n_big;
        let support = SupportPolynomial::from_tape(&n);
        Ok(Self { n: n_big, residue, support, seen, squarings: 0, previous_residue: None })
    }

    /// Test the current phase register against the shared support object.
    /// A proper gcd is an exact factor witness; the semiprime membrane still
    /// requires both nested arithmetic closures before emitting it.
    pub fn support_target(&self) -> Option<SupportClosure> {
        // Probe the complete initial frame sweep, then one register per doubled
        // phase scale. The full-return lane remains active between probes.
        let index = self.squarings;
        if index == 0 || (index > 8 && !index.is_power_of_two()) { return None; }
        let (polynomial_residue, p, q) = self.support.target(&self.residue, &self.n, 8)?;
        // The seven frame widths are faces of one polynomial, not independent
        // tests. Recheck their common value only when a factor target closes.
        if (2..=7).any(|width| self.support.evaluate_frame(&self.residue, &self.n, width) != polynomial_residue) {
            return None;
        }
        Some(SupportClosure {
            phase_index: self.squarings,
            phase_register: biguint_to_tape(&self.residue),
            polynomial_residue: biguint_to_tape(&polynomial_residue),
            p: biguint_to_tape(&p),
            q: biguint_to_tape(&q),
        })
    }

    /// One observation and one squaring. Caller chooses when to observe again;
    /// no fixed search ceiling is imposed on this resident state.
    pub fn observe(&mut self) -> Result<Option<ReturnRelation>, String> {
        let relation = match self.seen.entry(self.residue.clone()) {
            Entry::Occupied(entry) => {
                let seen = entry.get();
                let later = power_of_two_exponent(self.squarings);
                let earlier = seen.index.map(power_of_two_exponent).unwrap_or_else(|| vec![crate::vox::EVALT]);
                let half_residues = self.previous_residue.as_ref().and_then(|current_half| {
                    let earlier_half = match seen.index {
                        None => Some(BigUint::from(1u8)),
                        Some(index) if index > 0 => seen.half_residue.clone(),
                        _ => None,
                    }?;
                    Some((biguint_to_tape(current_half), biguint_to_tape(&earlier_half)))
                });
                Some(ReturnRelation {
                    earlier: earlier.clone(),
                    later: later.clone(),
                    residue: biguint_to_tape(&self.residue),
                    return_exponent: sub(&later, &earlier),
                    half_residues,
                })
            }
            Entry::Vacant(entry) => {
                entry.insert(SeenResidue {
                    index: Some(self.squarings),
                    half_residue: self.previous_residue.clone(),
                });
                None
            }
        };
        // The resident recurrence already carries the proof: residue starts at
        // a^1, each step squares residue while doubling exponent, and the
        // collision compares two states in this same inductively maintained
        // map. Replaying three full modular exponentiations here adds no new
        // evidence and dominates wide runs. Keep `ReturnRelation::verify` for
        // independent callers and tests, but do not repeat it on the hot path.
        self.previous_residue = Some(self.residue.clone());
        self.residue = (&self.residue * &self.residue) % &self.n;
        self.squarings += 1;
        Ok(relation)
    }

    pub fn stored_residues(&self) -> usize { self.seen.len() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::morphism_factor::tape_u64 as t;

    #[test]
    fn dynamic_integer_phase_register_round_trips_tapes() {
        for word in ["0", "1", "255", "256", "123456789012345678901234567890"] {
            let tape = crate::morphism_factor::decimal_to_tape(word).unwrap();
            assert_eq!(biguint_to_tape(&tape_to_biguint(&tape)), tape);
        }
    }

    #[test]
    fn all_frame_widths_are_faces_of_one_support_polynomial() {
        for (n, x) in [(15u64, 4u64), (21, 4), (35, 16), (8051, 37)] {
            let n_big = BigUint::from(n);
            let x_big = BigUint::from(x);
            let support = SupportPolynomial::from_tape(&t(n));
            let direct = support.bits_le.iter().enumerate().fold(
                BigUint::from(0u8),
                |sum, (index, bit)| {
                    if *bit {
                        (sum + x_big.modpow(&BigUint::from(index), &n_big)) % &n_big
                    } else {
                        sum
                    }
                },
            );
            for width in 2..=8 {
                assert_eq!(support.evaluate_frame(&x_big, &n_big, width), direct,
                    "N={n}, x={x}, frame width={width}");
            }
        }
    }

    #[test]
    fn support_polynomial_zero_targets_a_proper_factor_without_a_full_collision() {
        let mut partners = Partners::new(t(2), t(15)).unwrap();
        assert!(partners.support_target().is_none()); // F_15(2)=15, trivial gcd
        partners.observe().unwrap();
        let target = partners.support_target().expect("F_15(4) shares factor 5");
        assert_eq!(target.phase_index, 1);
        assert_eq!(crate::morphism_factor::dec_of(&target.p), "5");
        assert_eq!(crate::morphism_factor::dec_of(&target.q), "3");
        assert_eq!(crate::morphism_factor::mul(&target.p, &target.q), t(15));
    }

    #[test]
    fn relations_are_replayed_and_mutations_rejected() {
        for (n, expected) in [(15,4), (21,6), (35,12)] {
            let mut p = Partners::new(t(2),t(n)).unwrap();
            let relation = loop { if let Some(r) = p.observe().unwrap() { break r; } };
            assert_eq!(relation.return_exponent, t(expected));
            assert!(relation.verify(&t(2), &t(n)));
            let mut bad = relation.clone();
            bad.residue = t(0);
            assert!(!bad.verify(&t(2), &t(n)));
            let mut bad = relation;
            bad.return_exponent = t(1);
            assert!(!bad.verify(&t(2), &t(n)));
        }
        assert!(Partners::new(t(3),t(15)).is_err());
    }

    #[test]
    fn collision_carries_the_two_half_step_residues() {
        let mut partners = Partners::new(t(2), t(15)).unwrap();
        let relation = loop {
            if let Some(relation) = partners.observe().unwrap() { break relation; }
        };
        assert_eq!(relation.return_exponent, t(4));
        let (current_half, earlier_half) = relation.half_residues.unwrap();
        assert_eq!(crate::morphism_factor::dec_of(&current_half), "4");
        assert_eq!(crate::morphism_factor::dec_of(&earlier_half), "1");
    }

    #[test]
    fn wide_partner_map_preserves_the_expected_collision_and_both_closures() {
        let shift = BigUint::from(1u8) << 3300usize;
        let expected_p = BigUint::from(7161u16) * &shift + BigUint::from(1u8);
        let expected_q = BigUint::from(9135u16) * &shift + BigUint::from(1u8);
        let n = biguint_to_tape(&(&expected_p * &expected_q));
        let base = t(2);
        let mut partners = Partners::new(base.clone(), n.clone()).unwrap();
        let relation = (0..5000).find_map(|_| partners.observe().unwrap());
        let relation = relation.expect("wide phase collision closes within the measured orbit");
        assert_eq!(partners.squarings, 3720);
        assert!(relation.verify(&base, &n));
        let (p, q) = crate::shor_braid::phase_factor_register_seeds(
            &n, &relation.half_residues.as_ref().unwrap().0,
            &relation.half_residues.as_ref().unwrap().1,
        ).unwrap();
        let (small, large) = if expected_p <= expected_q {
            (&expected_p, &expected_q)
        } else {
            (&expected_q, &expected_p)
        };
        assert_eq!(p, biguint_to_tape(small));
        assert_eq!(q, biguint_to_tape(large));
        assert_eq!(crate::morphism_factor::mul(&p, &q), n);
        let radix = crate::morphism_factor::tape_u64(4_294_967_296);
        let product_outer = crate::factor_2adic::nest_product_over_prefix(&n, &p, &q, &radix).unwrap();
        let prefix_outer = crate::factor_2adic::nest_prefix_over_product(&n, &p, &q, &radix).unwrap();
        assert_eq!(product_outer, prefix_outer);
        assert!(crate::factor_2adic::radix_prefix_closes(
            &n, &p, &q, &radix,
        ));
    }
}
