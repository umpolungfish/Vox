//! GLUT p-System: Glut Superposition Factorization
//!
//! The GLUT (Glut-in-Loop) p-system factors N by maintaining a superposed
//! state of all viable (p, q) candidates simultaneously and closing the
//! mod-2^L congruence into equality at the crystal.
//!
//! The running-product constraint (p·q) mod 2^k == N mod 2^k creates a
//! carry propagation tree. The "glut" at level k is the set of all
//! (carry, p_prefix, q_prefix) states that survive frame consistency
//! checks, where the carry c_k is the EXACT carry into bit position k of
//! p·q:
//!
//! ```text
//! s_k     = c_k + Σ_{i+j=k} p_i q_j
//! n_k     = s_k mod 2
//! c_{k+1} = s_k >> 1
//! ```
//!
//! Algorithm (the seven-step membrane program):
//!
//! 1. Skin membrane:      seed the superposition at bit 0 from N's own parity
//! 2. Frame sweep:        advance the glut in windows of width w = 2..=8,
//!                        cycling the widths until every state is complete,
//!                        so the sweep covers the full bit length L for any N
//! 3. Glut superposition: at each bit, all four (p_bit, q_bit) pairs are held
//! 4. Glut multiplication: the running-product constraint keeps only the
//!                        pairs whose convolution bit matches N's bit
//! 5. Glut sieve:         cross-width consistency + deduplication by prefix;
//!                        states are exact, so no arbitrary truncation
//! 6. Glut crystal:       verify p·q == N EXACTLY — the congruence
//!                        p·q ≡ N (mod 2^L) admits alias pairs (e.g. 9·11 = 99
//!                        ≡ 35 (mod 64)) that only the full product rejects
//! 7. Glut readout:       the verified factor pairs, (p, q) with p·q = N

use alloc::collections::BTreeSet;
use alloc::vec::Vec;

const ZERO: char = '⊤';
const ONE: char = '⊥';

fn trim(mut tape: Vec<char>) -> Vec<char> {
    while tape.last() == Some(&ZERO) {
        tape.pop();
    }
    if tape.is_empty() {
        alloc::vec![ZERO]
    } else {
        tape
    }
}

fn bit(tape: &[char], index: usize) -> u8 {
    u8::from(tape.get(index) == Some(&ONE))
}

/// Glut state: the exact carry into bit `position` plus the factor prefixes
/// that produced it. Invariant: (p·q) mod 2^position == N mod 2^position and
/// `carry` is the true carry c_position of the running product.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GlutState {
    pub carry: u32,
    pub p_prefix: Vec<char>,
    pub q_prefix: Vec<char>,
    pub position: usize,
}

impl GlutState {
    /// Seed the superposition from N's own bit 0. For odd N the only live
    /// pair is (1, 1); for even N the pairs (0, 0), (0, 1), (1, 0) survive.
    pub fn seed(n_bit0: u8) -> Vec<Self> {
        let mut seeds = Vec::new();
        for p_bit in 0..2u8 {
            for q_bit in 0..2u8 {
                if p_bit * q_bit == n_bit0 {
                    seeds.push(Self {
                        carry: 0,
                        p_prefix: alloc::vec![if p_bit == 1 { ONE } else { ZERO }],
                        q_prefix: alloc::vec![if q_bit == 1 { ONE } else { ZERO }],
                        position: 1,
                    });
                }
            }
        }
        seeds
    }

    /// Advance one bit position, returning all valid next states.
    ///
    /// The convolution at position k sums the FULL diagonal
    /// Σ_{i+j=k} p_i q_j over the stored prefixes plus the new corner bits.
    pub fn advance(&self, n_bit: u8) -> Vec<Self> {
        let k = self.position;
        let p = &self.p_prefix;
        let q = &self.q_prefix;
        let mut results = Vec::new();
        for p_bit in 0..2u8 {
            for q_bit in 0..2u8 {
                let mut sum = self.carry as u64;
                for i in 0..k {
                    let pi = bit(p, i);
                    let qi = bit(q, k - i);
                    sum += pi as u64 * qi as u64;
                }
                sum += p_bit as u64 * bit(q, 0) as u64;
                sum += bit(p, 0) as u64 * q_bit as u64;
                if sum & 1 == n_bit as u64 {
                    let mut new_p = p.clone();
                    let mut new_q = q.clone();
                    new_p.push(if p_bit == 1 { ONE } else { ZERO });
                    new_q.push(if q_bit == 1 { ONE } else { ZERO });
                    results.push(Self {
                        carry: (sum >> 1) as u32,
                        p_prefix: new_p,
                        q_prefix: new_q,
                        position: k + 1,
                    });
                }
            }
        }
        results
    }

    pub fn verify_product_bit(&self, n: &[char]) -> bool {
        let prod = crate::morphism_factor::mul(&self.p_prefix, &self.q_prefix);
        (0..self.position).all(|i| {
            prod.get(i).copied().unwrap_or(ZERO) == n.get(i).copied().unwrap_or(ZERO)
        })
    }

    pub fn is_complete(&self, total_bits: usize) -> bool {
        self.position >= total_bits
    }
}

/// The GLUT sieve: maintains the superposed glut across the frame sweep.
/// No cap. The state vector is the exact live set at every level.
pub struct GlutSieve {
    pub n_tape: Vec<char>,
    pub n_bits: Vec<u8>,
    pub states: Vec<GlutState>,
}

impl GlutSieve {
    pub fn new(n: &[char]) -> Self {
        let trimmed = trim(n.to_vec());
        let n_bits: Vec<u8> = trimmed.iter().map(|c| u8::from(*c == ONE)).collect();
        Self {
            n_tape: trimmed.clone(),
            n_bits,
            states: GlutState::seed(bit(&trimmed, 0)),
        }
    }

    /// One frame: advance every state by `width` bits (or as far as N runs).
    /// Dedup by (p_prefix, q_prefix) is loss-free — the carry is a function
    /// of the prefixes.
    pub fn frame_superpose(&mut self, width: usize) -> usize {
        let mut next_states = Vec::new();
        for state in self.states.iter() {
            let pos = state.position.min(self.n_bits.len());
            if pos >= self.n_bits.len() {
                next_states.push(state.clone());
                continue;
            }
            let bits_to_process = width.min(self.n_bits.len() - pos);
            let branches = self.propagate_block(state, pos, bits_to_process);
            next_states.extend(branches);
        }
        let mut seen = BTreeSet::new();
        let mut kept = Vec::new();
        for s in next_states {
            if seen.insert((s.p_prefix.clone(), s.q_prefix.clone())) {
                kept.push(s);
            }
        }
        self.states = kept;
        self.states.len()
    }

    fn propagate_block(
        &self,
        state: &GlutState,
        start_pos: usize,
        num_bits: usize,
    ) -> Vec<GlutState> {
        let mut current = alloc::vec![state.clone()];
        for k in 0..num_bits {
            let pos = start_pos + k;
            if pos >= self.n_bits.len() {
                break;
            }
            let n_bit = self.n_bits[pos];
            let mut next = Vec::new();
            for s in current.iter() {
                next.extend(s.advance(n_bit));
            }
            current = next;
            if current.is_empty() {
                return alloc::vec![];
            }
        }
        current
    }

    /// The frame sweep: cycle widths 2..=8 until every state is complete.
    pub fn frame_sweep(&mut self) {
        if self.states.is_empty() {
            return;
        }
        let mut width = 2;
        while self.states.iter().any(|s| !s.is_complete(self.n_bits.len())) {
            self.frame_superpose(width);
            if self.states.is_empty() {
                return;
            }
            width = if width == 8 { 2 } else { width + 1 };
        }
    }

    /// Glut crystal: the survivors satisfy (p·q) ≡ N (mod 2^L); the crystal
    /// closes the congruence to equality.
    pub fn glut_crystal(&self) -> Vec<(Vec<char>, Vec<char>)> {
        let total = self.n_bits.len();
        let mut pairs = Vec::new();
        let mut seen = BTreeSet::new();
        for s in self.states.iter().filter(|s| s.is_complete(total)) {
            let p = trim(s.p_prefix.clone());
            let q = trim(s.q_prefix.clone());
            let prod = crate::morphism_factor::mul(&p, &q);
            if crate::morphism_factor::cmp(&prod, &self.n_tape) != core::cmp::Ordering::Equal {
                continue;
            }
            if seen.insert((p.clone(), q.clone())) {
                pairs.push((p, q));
            }
        }
        pairs
    }

    /// Glut readout: the first verified factor pair.
    pub fn readout(&self) -> Option<(Vec<char>, Vec<char>)> {
        self.glut_crystal().into_iter().next()
    }
}

/// GLUT p-system factorization.
pub fn glut_factor(n: &[char]) -> Option<(Vec<char>, Vec<char>)> {
    let mut sieve = GlutSieve::new(n);
    sieve.frame_sweep();
    sieve.readout()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::morphism_factor::cmp;

    fn to_tape(mut n: u64) -> Vec<char> {
        let mut t = Vec::new();
        if n == 0 {
            t.push(ZERO);
            return t;
        }
        while n > 0 {
            t.push(if n & 1 == 1 { ONE } else { ZERO });
            n >>= 1;
        }
        t
    }

    #[test]
    fn test_glut_factor_35() {
        let n = to_tape(35);
        let (p, q) = glut_factor(&n).expect("35 must factor");
        assert_eq!(
            cmp(&crate::morphism_factor::mul(&p, &q), &n),
            core::cmp::Ordering::Equal
        );
    }

    #[test]
    fn test_glut_factor_8051() {
        let n = to_tape(8051);
        let (p, q) = glut_factor(&n).expect("8051 must factor");
        assert_eq!(
            cmp(&crate::morphism_factor::mul(&p, &q), &n),
            core::cmp::Ordering::Equal
        );
    }

    #[test]
    fn test_glut_factor_even_56() {
        let n = to_tape(56);
        let (p, q) = glut_factor(&n).expect("56 must factor");
        assert_eq!(
            cmp(&crate::morphism_factor::mul(&p, &q), &n),
            core::cmp::Ordering::Equal
        );
    }

    #[test]
    fn test_glut_crystal_rejects_alias() {
        let n = to_tape(35);
        let sieve = GlutSieve::new(&n);
        let pairs = {
            let mut s = sieve;
            s.frame_sweep();
            s.glut_crystal()
        };
        for (p, q) in &pairs {
            assert_eq!(
                cmp(&crate::morphism_factor::mul(p, q), &n),
                core::cmp::Ordering::Equal
            );
        }
        assert_eq!(pairs.len(), 4);
    }

    #[test]
    fn test_glut_state_count_is_exact_at_each_level() {
        let n = to_tape(35);
        let mut sieve = GlutSieve::new(&n);
        for level in 1..=12usize {
            sieve.frame_superpose(1);
            assert_eq!(
                sieve.states.len(),
                1usize << (level - 1),
                "state count at level {}",
                level,
            );
        }
    }

    #[test]
    fn test_glut_invariant_holds() {
        let n = to_tape(8051);
        let mut sieve = GlutSieve::new(&n);
        for width in [2, 3, 4] {
            sieve.frame_superpose(width);
            for s in &sieve.states {
                assert!(
                    s.verify_product_bit(&sieve.n_tape),
                    "invariant broken at width {width}"
                );
            }
        }
    }
}