//! Tape-native symbolic Hadamard gate for the factor-production path.
//!
//! This module is upstream of passive factor extraction.  The initial carrier
//! owns only N; p and q are not fields and cannot be smuggled into the seed.
//! A Hadamard boundary is represented by its exact Walsh character
//! `(-1)^<x,y>` over IMASM numeral tapes.  No 2^k matrix is materialized and no
//! tape is decoded to a host integer.  A later phase boundary may imscribe a
//! source label into the succeeding whole object; the Hadamard gate then reads
//! the sign at a target label directly from those tapes.
//!
//! The returned +/-1 coefficient is an amplitude sign, not a FOUR judgment.
//! FOUR remains responsible for judging the exposed carrier relationship.

use alloc::vec;
use alloc::vec::Vec;

use crate::morphism_factor::trim;
use crate::vox::{EVALF, EVALT};

pub type Tape = Vec<char>;

fn validate_numeral_tape(tape: &[char]) -> Result<(), &'static str> {
    if tape.is_empty() {
        return Err("Hadamard numeral tape is empty");
    }
    if tape.iter().any(|&mark| mark != EVALT && mark != EVALF) {
        return Err("Hadamard numeral tape contains a non-numeral mark");
    }
    Ok(())
}

/// Exact Walsh/Hadamard character on arbitrary-width numeral tapes.
/// Missing high cells are EVALT/zero; no host-width conversion occurs.
pub fn hadamard_character(source: &[char], target: &[char]) -> Result<i8, &'static str> {
    validate_numeral_tape(source)?;
    validate_numeral_tape(target)?;

    let width = source.len().max(target.len());
    let mut odd = false;
    for i in 0..width {
        let a = source.get(i).copied().unwrap_or(EVALT) == EVALF;
        let b = target.get(i).copied().unwrap_or(EVALT) == EVALF;
        odd ^= a && b;
    }
    Ok(if odd { -1 } else { 1 })
}

/// Tape-native XOR for composing Hadamard characters.  This is the additive
/// boundary law over F2, expressed without decoding either tape.
pub fn xor_tape(left: &[char], right: &[char]) -> Result<Tape, &'static str> {
    validate_numeral_tape(left)?;
    validate_numeral_tape(right)?;

    let width = left.len().max(right.len());
    let mut out = Vec::with_capacity(width);
    for i in 0..width {
        let a = left.get(i).copied().unwrap_or(EVALT) == EVALF;
        let b = right.get(i).copied().unwrap_or(EVALT) == EVALF;
        out.push(if a ^ b { EVALF } else { EVALT });
    }
    Ok(trim(out))
}

/// Current whole object at the Hadamard boundary.
///
/// `new` seeds the production path with N and the zero source only.  A prior
/// phase/modular boundary can write a new source by consuming this object with
/// `imscribe_source`; the resulting object is then the one read by `sign_at`.
#[derive(Clone, PartialEq, Debug)]
pub struct HadamardCarrier {
    n: Tape,
    source: Tape,
}

impl HadamardCarrier {
    /// Seed from N only.  No factor witness exists in this object.
    pub fn new(n: &[char]) -> Result<Self, &'static str> {
        validate_numeral_tape(n)?;
        Ok(Self {
            n: trim(n.to_vec()),
            source: vec![EVALT],
        })
    }

    pub fn n(&self) -> &[char] {
        &self.n
    }

    pub fn source(&self) -> &[char] {
        &self.source
    }

    /// Consume the current whole object and write the source exposed by the
    /// preceding phase boundary into the succeeding Hadamard carrier.
    pub fn imscribe_source(mut self, source: &[char]) -> Result<Self, &'static str> {
        validate_numeral_tape(source)?;
        self.source = trim(source.to_vec());
        Ok(self)
    }

    /// Read one exact coefficient of the symbolic Hadamard row.
    pub fn sign_at(&self, target: &[char]) -> Result<i8, &'static str> {
        hadamard_character(&self.source, target)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::morphism_factor::tape_u64;

    #[test]
    fn n_only_seed_opens_the_uniform_hadamard_row() {
        let n = tape_u64(8051);
        let carrier = HadamardCarrier::new(&n).unwrap();
        assert_eq!(carrier.n(), n.as_slice());
        assert_eq!(carrier.source(), [EVALT]);

        for target in 0u64..16 {
            assert_eq!(carrier.sign_at(&tape_u64(target)).unwrap(), 1);
        }
    }

    #[test]
    fn symbolic_gate_matches_the_sylvester_character_without_a_matrix() {
        let n = tape_u64(8051);
        for source in 0u64..16 {
            let carrier = HadamardCarrier::new(&n)
                .unwrap()
                .imscribe_source(&tape_u64(source))
                .unwrap();
            for target in 0u64..16 {
                let expected = if (source & target).count_ones() & 1 == 0 { 1 } else { -1 };
                assert_eq!(carrier.sign_at(&tape_u64(target)).unwrap(), expected);
            }
        }
    }

    #[test]
    fn character_composes_on_tapes_and_second_gate_closes() {
        let n = tape_u64(8051);
        for a in 0u64..16 {
            for b in 0u64..16 {
                let ab = xor_tape(&tape_u64(a), &tape_u64(b)).unwrap();
                for k in 0u64..16 {
                    let target = tape_u64(k);
                    let lhs = hadamard_character(&ab, &target).unwrap();
                    let rhs = hadamard_character(&tape_u64(a), &target).unwrap()
                        * hadamard_character(&tape_u64(b), &target).unwrap();
                    assert_eq!(lhs, rhs, "a={a}, b={b}, k={k}");
                }
            }
        }

        // H H = I up to the unmaterialized normalization: the exact row inner
        // product is 16 on the same basis label and zero on every other label.
        for source in 0u64..16 {
            let carrier = HadamardCarrier::new(&n)
                .unwrap()
                .imscribe_source(&tape_u64(source))
                .unwrap();
            for other in 0u64..16 {
                let mut inner = 0i64;
                for target in 0u64..16 {
                    let y = tape_u64(target);
                    inner += i64::from(carrier.sign_at(&y).unwrap())
                        * i64::from(hadamard_character(&tape_u64(other), &y).unwrap());
                }
                assert_eq!(inner, if source == other { 16 } else { 0 });
            }
        }
    }

    #[test]
    fn character_is_not_limited_to_a_host_word() {
        let mut source = vec![EVALT; 130];
        let mut target = vec![EVALT; 130];
        source[129] = EVALF;
        target[129] = EVALF;
        assert_eq!(hadamard_character(&source, &target).unwrap(), -1);

        target[64] = EVALF;
        assert_eq!(hadamard_character(&source, &target).unwrap(), -1);
        source[64] = EVALF;
        assert_eq!(hadamard_character(&source, &target).unwrap(), 1);
    }
}
