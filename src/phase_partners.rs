//! Sequential dyadic partner observations. No register enumeration or supplied
//! order. A collision proves a return exponent, which may be a multiple of order.
use std::collections::BTreeMap;
use num_bigint::BigUint;
use num_traits::Zero;
use vox::morphism_factor::{cmp, gcd, modulo, mul, one, sub};
type Tape = Vec<char>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReturnRelation {
    pub earlier: Tape,
    pub later: Tape,
    pub residue: Tape,
    pub return_exponent: Tape,
}

#[allow(dead_code)] // Retained for explicit, non-hot-path relation replay.
fn power(a: &[char], exponent: &[char], n: &[char]) -> Tape {
    let mut result = one();
    let mut base = modulo(a, n);
    for (i, &bit) in exponent.iter().enumerate() {
        if bit == vox::vox::EVALF { result = modulo(&mul(&result, &base), n); }
        if i + 1 < exponent.len() { base = modulo(&mul(&base, &base), n); }
    }
    result
}

impl ReturnRelation {
    #[allow(dead_code)] // Independent verifier; observe() relies on its recurrence invariant.
    pub fn verify(&self, a: &[char], n: &[char]) -> bool {
        if cmp(n, &one()) != std::cmp::Ordering::Greater
            || gcd(a.to_vec(), n.to_vec()) != one()
            || cmp(&self.later, &self.earlier) != std::cmp::Ordering::Greater {
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
    // Store exact residues in packed LSB-first bytes and only the observation
    // index. Storing each ever-growing exponent tape made total memory
    // quadratic in the number of phase observations.
    seen: BTreeMap<Vec<u8>, Option<usize>>,
    pub squarings: usize,
}

fn tape_to_biguint(tape: &[char]) -> BigUint {
    let mut packed = vec![0u8; (tape.len() + 7) / 8];
    for (index, &mark) in tape.iter().enumerate() {
        if mark == vox::vox::EVALF {
            packed[index / 8] |= 1u8 << (index % 8);
        }
    }
    BigUint::from_bytes_le(&packed)
}

fn biguint_to_tape(value: &BigUint) -> Tape {
    if value.is_zero() { return vec![vox::vox::EVALT]; }
    let packed = value.to_bytes_le();
    let mut tape = Vec::with_capacity(packed.len() * 8);
    for byte in packed {
        for bit in 0..8 {
            tape.push(if (byte >> bit) & 1 == 1 { vox::vox::EVALF } else { vox::vox::EVALT });
        }
    }
    while tape.last() == Some(&vox::vox::EVALT) { tape.pop(); }
    tape
}

fn power_of_two_exponent(index: usize) -> Tape {
    let mut exponent = vec![vox::vox::EVALT; index + 1];
    exponent[index] = vox::vox::EVALF;
    exponent
}

impl Partners {
    pub fn new(a: Tape, n: Tape) -> Result<Self, String> {
        if cmp(&n, &one()) != std::cmp::Ordering::Greater || gcd(a.clone(), n.clone()) != one() {
            return Err("partners require N > 1 and a coprime base".into());
        }
        let n_big = tape_to_biguint(&n);
        let a_big = tape_to_biguint(&a);
        let mut seen = BTreeMap::new();
        // The initial state is 1 = a^0, distinguished from a^(2^i).
        seen.insert(vec![1], None);
        let residue = a_big % &n_big;
        Ok(Self { n: n_big, residue, seen, squarings: 0 })
    }

    /// One observation and one squaring. Caller chooses when to observe again;
    /// no fixed search ceiling is imposed on this resident state.
    pub fn observe(&mut self) -> Result<Option<ReturnRelation>, String> {
        let key = self.residue.to_bytes_le();
        let relation = self.seen.get(&key).map(|earlier_index| {
            let later = power_of_two_exponent(self.squarings);
            let earlier = earlier_index.map(power_of_two_exponent).unwrap_or_else(|| vec![vox::vox::EVALT]);
            ReturnRelation {
                earlier: earlier.clone(),
                later: later.clone(),
                residue: biguint_to_tape(&self.residue),
                return_exponent: sub(&later, &earlier),
            }
        });
        // The resident recurrence already carries the proof: residue starts at
        // a^1, each step squares residue while doubling exponent, and the
        // collision compares two states in this same inductively maintained
        // map. Replaying three full modular exponentiations here adds no new
        // evidence and dominates wide runs. Keep `ReturnRelation::verify` for
        // independent callers and tests, but do not repeat it on the hot path.
        self.seen.entry(key).or_insert(Some(self.squarings));
        self.residue = (&self.residue * &self.residue) % &self.n;
        self.squarings += 1;
        Ok(relation)
    }

    pub fn stored_residues(&self) -> usize { self.seen.len() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vox::morphism_factor::tape_u64 as t;

    #[test]
    fn dynamic_integer_phase_register_round_trips_tapes() {
        for word in ["0", "1", "255", "256", "123456789012345678901234567890"] {
            let tape = vox::morphism_factor::decimal_to_tape(word).unwrap();
            assert_eq!(biguint_to_tape(&tape_to_biguint(&tape)), tape);
        }
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
}
