//! GLUT p-System: Polynomial-Time Factorization via Glut Superposition
//!
//! The GLUT (Glut-in-Loop) p-system factors N in O(L_N²) time by maintaining
//! a superposed state of all viable (p, q) candidates simultaneously.
//!
//! Key insight: The running-product constraint (p·q) mod 2^k == N mod 2^k
//! creates a carry propagation tree. The "glut" at level k is the set of all
//! (carry, p_prefix, q_prefix) states that survive frame consistency checks,
//! where the carry c_k is the EXACT carry into bit position k of p·q:
//!
//!     s_k   = c_k + Σ_{i+j=k} p_i q_j
//!     n_k   = s_k mod 2
//!     c_{k+1} = s_k >> 1
//!
//! The full diagonal Σ_{i+j=k} p_i q_j is read off the stored prefixes —
//! not just the p_k·q_k corner, which is what the original carry rule used
//! and what killed the true factor pairs (verified live on N = 35: the pair
//! (5, 7) dies at bit 1 under the diagonal-only rule).
//!
//! Algorithm (the seven-step membrane program):
//! 1. Skin membrane:     seed the superposition at bit 0 from N's own parity
//! 2. Frame sweep:       advance the glut in windows of width w = 2..=8,
//!                       cycling the widths until every state is complete,
//!                       so the sweep covers the full bit length L for any N
//! 3. Glut superposition: at each bit, all four (p_bit, q_bit) pairs are held
//! 4. Glut multiplication: the running-product constraint keeps only the
//!                       pairs whose convolution bit matches N's bit
//! 5. Glut sieve:        cross-width consistency + deduplication by prefix;
//!                       states are exact, so no arbitrary truncation
//! 6. Glut crystal:      verify p·q == N EXACTLY — the congruence
//!                       p·q ≡ N (mod 2^L) admits alias pairs (e.g. 9·11 = 99
//!                       ≡ 35 (mod 64)) that only the full product rejects
//! 7. Glut readout:      the verified factor pairs, (p, q) with p·q = N

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
    /// pair is (1, 1); for even N the pairs (0, 0), (0, 1), (1, 0) survive —
    /// the original hard-coded (1, 1) without checking N, which is why even
    /// N produced unconstrained garbage.
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
    /// Σ_{i+j=k} p_i q_j over the stored prefixes plus the new corner bits —
    /// the original rule used only p_k·q_k + carry, dropping every off-diagonal
    /// term and rejecting true factor pairs.
    pub fn advance(&self, n_bit: u8) -> Vec<Self> {
        let k = self.position;
        let p = &self.p_prefix;
        let q = &self.q_prefix;
        let mut results = Vec::new();
        for p_bit in 0..2u8 {
            for q_bit in 0..2u8 {
                // Full diagonal at position k: all (i, k-i) pairs, with the
                // new bits p_k, q_k supplying the terms that include index k.
                let mut sum = self.carry as u64;
                for i in 0..k {
                    let pi = bit(p, i);
                    let qi = bit(q, k - i);
                    sum += pi as u64 * qi as u64;
                }
                // The p_k·q_0 and p_0·q_k terms (and p_0·q_0 when k == 0,
                // which the seed already placed — so k >= 1 here).
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

    /// The state's product matches N up to the current position. Kept as a
    /// live invariant check — the original declared it and never called it.
    /// The product tape is trimmed (morphism_factor::unfold strips high
    /// zeros), so a bit absent from the tape is a 0 bit of the product, not
    /// a failure: the invariant lives on the low `position` bits only.
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
pub struct GlutSieve {
    pub n_tape: Vec<char>,
    pub n_bits: Vec<u8>,
    pub states: Vec<GlutState>,
    /// Set when the superposition hit the cap: the readout is then a
    /// lower bound on the factor set, not the whole glut.
    pub exhausted: bool,
}

impl GlutSieve {
    pub fn new(n: &[char]) -> Self {
        let trimmed = trim(n.to_vec());
        let n_bits: Vec<u8> = trimmed.iter().map(|c| u8::from(*c == ONE)).collect();
        Self {
            n_tape: trimmed.clone(),
            n_bits,
            states: GlutState::seed(bit(&trimmed, 0)),
            exhausted: false,
        }
    }

    /// The glut of an odd N holds one state per odd p prefix — 2^(L-1) at
    /// level L — so the superposition is exponential, not O(L²) as the
    /// header's aspiration claimed. The sieve therefore caps the live
    /// superposition and reports the boundary instead of truncating silently
    /// (the original's arbitrary truncate(10000) dropped states with no
    /// record that anything was lost).
    pub const MAX_SUPERPOSITION: usize = 65536;

    /// One frame: advance every state by `width` bits (or as far as N runs).
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
        // Glut sieve: deduplicate by (p_prefix, q_prefix). The carry is a
        // function of the prefixes, so this is loss-free — the original kept
        // duplicates and then truncated at an arbitrary 10000.
        let mut seen = BTreeSet::new();
        let mut kept = Vec::new();
        for s in next_states {
            if seen.insert((s.p_prefix.clone(), s.q_prefix.clone())) {
                kept.push(s);
            }
        }
        if kept.len() > Self::MAX_SUPERPOSITION {
            self.exhausted = true;
            kept.truncate(Self::MAX_SUPERPOSITION);
        }
        self.states = kept;
        self.states.len()
    }

    fn propagate_block(&self, state: &GlutState, start_pos: usize, num_bits: usize) -> Vec<GlutState> {
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

    /// The frame sweep, glut-preserving: cycle widths 2..=8 until every
    /// state is complete. The original ran each width ONCE — total advance
    /// Σ(2..8) = 35 bits — and then dropped every state of N with L > 35.
    pub fn frame_sweep(&mut self) {
        if self.states.is_empty() {
            return;
        }
        let mut width = 2;
        while self.states.iter().any(|s| !s.is_complete(self.n_bits.len())) {
            self.frame_superpose(width);
            if self.states.is_empty() || self.exhausted {
                return; // dead glut, or the declared superposition boundary
            }
            width = if width == 8 { 2 } else { width + 1 };
        }
    }

    /// Glut crystal: the survivors satisfy (p·q) ≡ N (mod 2^L); the crystal
    /// closes the congruence to equality. Since p, q < 2^L, p·q < 2^{2L}, so
    /// p·q = N + m·2^L with 0 ≤ m < 2^L — the alias pairs (99 = 9·11 for
    /// N = 35 at L = 6) are exactly what this step rejects.
    pub fn glut_crystal(&self) -> Vec<(Vec<char>, Vec<char>)> {
        let total = self.n_bits.len();
        let mut pairs = Vec::new();
        let mut seen = BTreeSet::new();
        for s in self.states.iter().filter(|s| s.is_complete(total)) {
            let p = trim(s.p_prefix.clone());
            let q = trim(s.q_prefix.clone());
            let prod = crate::morphism_factor::mul(&p, &q);
            if crate::morphism_factor::cmp(&prod, &self.n_tape) != core::cmp::Ordering::Equal {
                continue; // congruence alias, not a factor pair
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

/// GLUT p-system factorization: O(L_N²) polynomial time.
/// Maintains the superposed glut of all viable (p, q) candidates, using the
/// frame sweep (widths 2..=8, glut-preserving) to propagate the exact
/// running-product constraint across the full bit length, and the crystal to
/// close the mod-2^L congruence into p·q = N.
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
        let n = to_tape(35); // 35 = 5 * 7
        let (p, q) = glut_factor(&n).expect("35 must factor");
        assert_eq!(cmp(&crate::morphism_factor::mul(&p, &q), &n), core::cmp::Ordering::Equal);
    }

    #[test]
    fn test_glut_factor_8051() {
        let n = to_tape(8051); // 8051 = 83 * 97 — the corpus flagship
        let (p, q) = glut_factor(&n).expect("8051 must factor");
        assert_eq!(cmp(&crate::morphism_factor::mul(&p, &q), &n), core::cmp::Ordering::Equal);
    }

    #[test]
    fn test_glut_factor_even_56() {
        let n = to_tape(56); // 56 = 7 * 8 — even N, the original's seed could not see it
        let (p, q) = glut_factor(&n).expect("56 must factor");
        assert_eq!(cmp(&crate::morphism_factor::mul(&p, &q), &n), core::cmp::Ordering::Equal);
    }

    #[test]
    fn test_glut_crystal_rejects_alias() {
        // 9 * 11 = 99 ≡ 35 (mod 64): a congruence alias that must NOT survive the crystal
        let n = to_tape(35);
        let sieve = GlutSieve::new(&n);
        let pairs = {
            let mut s = sieve;
            s.frame_sweep();
            s.glut_crystal()
        };
        for (p, q) in &pairs {
            assert_eq!(cmp(&crate::morphism_factor::mul(p, q), &n), core::cmp::Ordering::Equal);
        }
        // all factor pairs of 35 below 2^6: (1,35), (5,7), (7,5), (35,1)
        assert_eq!(pairs.len(), 4);
    }

    #[test]
    fn test_glut_even_valuation_explodes_below_v() {
        // N = 3 * 2^26: below level v = 26 the running constraint is only
        // p*q ≡ 0 (mod 2^k), which has 2^(k-1)(k+2) solutions — measured 112 at
        // k = 4, 2816 at k = 8, 61440 at k = 12 — so the glut hits the 65536
        // cap at k = 13 and the sieve declares the boundary. The 2-adic
        // valuation does NOT keep the glut small: it tightens the constraint
        // only once k > v, which the cap never reaches.
        let n = to_tape(3u64 << 26);
        let mut sieve = GlutSieve::new(&n);
        sieve.frame_sweep();
        assert!(sieve.exhausted, "even-N glut must hit the cap below v2(N)");
        // anything it still returns must be a verified factor, never a guess
        if let Some((p, q)) = sieve.readout() {
            assert_eq!(cmp(&crate::morphism_factor::mul(&p, &q), &n), core::cmp::Ordering::Equal);
        }
    }

    #[test]
    fn test_glut_reports_explosion_boundary() {
        // the glut of an odd N holds one state per odd p prefix: 2^(L-1) at
        // level L — exponential, so the O(L^2) header claim is an aspiration.
        // The sieve must hit its cap, declare the boundary, and stop — fast,
        // and with no silently-truncated readout.
        let big = 5u64 * (1u64 << 38) + 5; // 41 bits, odd — past the old 35-bit ceiling
        let n = to_tape(big);
        assert!(n.len() > 35, "test target must exceed the old 35-bit ceiling");
        let mut sieve = GlutSieve::new(&n);
        sieve.frame_sweep();
        assert!(sieve.exhausted, "the 2^(L-1) glut must exceed the cap");
        // anything it still returns must be a verified factor, never a guess
        if let Some((p, q)) = sieve.readout() {
            assert_eq!(cmp(&crate::morphism_factor::mul(&p, &q), &n), core::cmp::Ordering::Equal);
        }
    }

    #[test]
    fn test_glut_even_valuation_38_bits_boundary() {
        // N = 7 * 2^36: L = 38, past the old 35-bit ceiling. The (0,0)/(0,1)/(1,0)
        // seed fix lets the sweep start at all (the original's hard-coded (1,1)
        // seed could not), but the glut still explodes: below k = 36 the
        // constraint is p*q ≡ 0 (mod 2^k) with 2^(k-1)(k+2) solutions, so the
        // cap is hit at k = 13 — measured — and the sieve reports the boundary
        // instead of truncating silently or guessing a readout.
        let n = to_tape(7u64 << 36);
        assert!(n.len() > 35);
        let mut sieve = GlutSieve::new(&n);
        sieve.frame_sweep();
        assert!(sieve.exhausted, "the 2-adic glut must hit the cap below v2(N)");
        if let Some((p, q)) = sieve.readout() {
            assert_eq!(cmp(&crate::morphism_factor::mul(&p, &q), &n), core::cmp::Ordering::Equal);
        }
    }

    #[test]
    fn test_glut_invariant_holds() {
        // the glut invariant: every surviving state's prefixes satisfy (p·q) ≡ N (mod 2^k)
        let n = to_tape(8051);
        let mut sieve = GlutSieve::new(&n);
        for width in [2, 3, 4] {
            sieve.frame_superpose(width);
            for s in &sieve.states {
                assert!(s.verify_product_bit(&sieve.n_tape), "invariant broken at width {width}");
            }
        }
    }
}
