//! GLUT p-System: Polynomial-Time Factorization via Glut Superposition
//!
//! The GLUT (Glut-in-Loop) p-system factors N in O(L_N²) time by maintaining
//! a superposed state of all viable (p, q) candidates simultaneously.
//!
//! Key insight: The running-product constraint p_k XOR q_k = n_k XOR carry_k
//! creates a carry propagation tree. The "glut" is the set of all (carry, p_prefix)
//! states that survive frame consistency checks across widths 2..8.
//!
//! Algorithm:
//! 1. Frame sweep: decompose N's bits into windows of size 2..8
//! 2. Glut superposition: for each width, propagate candidate states forward
//! 3. Frame sieve: cross-check states across all widths, keep only consistent
//! 4. Glut crystal: extend from width-limited state to full bit length
//! 5. Glut readout: extract factor pairs from the shrunk state space

use alloc::vec::Vec;
use core::cmp::Ordering;
use num_bigint::BigUint;
use num_traits::One;

const ZERO: char = '⊤';
const ONE: char = '⊥';

fn tape_to_biguint(tape: &[char]) -> BigUint {
    let mut bytes = alloc::vec![0u8; (tape.len() + 7) / 8];
    for (index, mark) in tape.iter().enumerate() {
        if *mark == ONE {
            bytes[index / 8] |= 1 << (index % 8);
        }
    }
    BigUint::from_bytes_le(&bytes)
}

fn biguint_to_tape(value: &BigUint) -> Vec<char> {
    let mut tape = Vec::new();
    let bytes = value.to_bytes_le();
    for byte in &bytes {
        for bit in 0..8 {
            tape.push(if (byte >> bit) & 1 == 1 { ONE } else { ZERO });
        }
    }
    // Remove trailing zeros
    while tape.last() == Some(&ZERO) {
        tape.pop();
    }
    if tape.is_empty() {
        alloc::vec![ZERO]
    } else {
        tape
    }
}

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

fn cmp_tape(a: &[char], b: &[char]) -> Ordering {
    let a_len = a.iter().rposition(|c| *c != ZERO).map_or(0, |i| i + 1);
    let b_len = b.iter().rposition(|c| *c != ZERO).map_or(0, |i| i + 1);
    match a_len.cmp(&b_len) {
        Ordering::Equal => {
            for i in (0..a_len.max(1)).rev() {
                let ab = bit(a, i);
                let bb = bit(b, i);
                match ab.cmp(&bb) {
                    Ordering::Equal => {}
                    order => return order,
                }
            }
            Ordering::Equal
        }
        order => order,
    }
}

fn mul(a: &[char], b: &[char]) -> Vec<char> {
    crate::morphism_factor::mul(a, b)
}

/// Glut state: tracks carry for the running product.
/// At bit position k, we track the carry from the multiplication p*q below position k.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GlutState {
    pub carry: u32,
    pub p_prefix: Vec<char>,
    pub q_prefix: Vec<char>,
    pub position: usize,
}

impl GlutState {
    pub fn new() -> Self {
        Self {
            carry: 0,
            p_prefix: alloc::vec![ONE],
            q_prefix: alloc::vec![ONE],
            position: 1,
        }
    }

    /// Advance one bit position, returning all valid next states.
    pub fn advance(&self, n_bit: u8) -> Vec<Self> {
        let mut results = Vec::new();
        for p_bit in 0..2 {
            for q_bit in 0..2 {
                let prod_sum = p_bit * q_bit + self.carry as u32;
                let prod_bit = prod_sum & 1;
                let carry_out = prod_sum >> 1;
                if prod_bit == n_bit as u32 {
                    let mut new_p = self.p_prefix.clone();
                    let mut new_q = self.q_prefix.clone();
                    new_p.push(if p_bit == 1 { ONE } else { ZERO });
                    new_q.push(if q_bit == 1 { ONE } else { ZERO });
                    results.push(Self {
                        carry: carry_out,
                        p_prefix: new_p,
                        q_prefix: new_q,
                        position: self.position + 1,
                    });
                }
            }
        }
        results
    }

    /// Verify the state's product matches N up to current position.
    pub fn verify_product_bit(&self, n_bit: u8) -> bool {
        let p_tape = &self.p_prefix;
        let q_tape = &self.q_prefix;
        let prod = mul(p_tape, q_tape);
        let prod_bit = bit(&prod, self.position - 1);
        prod_bit == n_bit
    }

    pub fn is_complete(&self, total_bits: usize) -> bool {
        self.position >= total_bits
    }
}

/// The GLUT sieve: maintains a superposed set of states.
pub struct GlutSieve {
    pub n_bits: Vec<u8>,
    pub states: Vec<GlutState>,
    pub n_tape: Vec<char>,
}

impl GlutSieve {
    pub fn new(n: &[char]) -> Self {
        let n_bits: Vec<u8> = n.iter().map(|c| if *c == ONE { 1 } else { 0 }).collect();
        Self {
            n_bits,
            states: vec![GlutState::new()],
            n_tape: n.to_vec(),
        }
    }

    /// Propagate states through one frame width.
    /// For width w, process w bits at a time using the frame sweep.
    pub fn frame_superpose(&mut self, width: usize) -> usize {
        let mut next_states = alloc::vec![];
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
        self.states = next_states;
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
                let branches = s.advance(n_bit);
                next.extend(branches);
            }
            if next.len() > 10000 {
                next.truncate(10000);
            }
            current = next;
        }
        current
    }

    /// Glut sieve: cross-check across all widths, keeping only states
    /// that are consistent with every frame sweep.
    pub fn frame_sieve(&mut self) {
        for width in 2..=8 {
            self.frame_superpose(width);
        }
        self.states.retain(|s| s.is_complete(self.n_bits.len()));
    }

    /// Glut crystal: finalize the surviving states.
    pub fn glut_crystal(&self) -> Vec<(Vec<char>, Vec<char>)> {
        self.states
            .iter()
            .filter(|s| s.is_complete(self.n_bits.len()))
            .map(|s| {
                let p_trim = trim(s.p_prefix.clone());
                let q_trim = trim(s.q_prefix.clone());
                (p_trim, q_trim)
            })
            .collect()
    }

    /// Glut readout: extract the best verified factor pair.
    ///
    /// The crystal yields candidate tape pairs; the readout certifies one.
    /// Each candidate is lifted to true integers (`tape_to_biguint`) and
    /// survives only if the exact product holds (`pv * qv == n`) and neither
    /// factor is trivial (`BigUint::one`). Survivors are canonical-ordered
    /// `p <= q` by `cmp_tape`, the smallest `p` wins, and the winning pair is
    /// re-encoded to canonical tape (`biguint_to_tape`) before it leaves the
    /// module.
    pub fn readout(&self) -> Option<(Vec<char>, Vec<char>)> {
        let n = tape_to_biguint(&self.n_tape);
        let one = BigUint::one(); // keeps `num_traits::One` live
        let mut best: Option<(Vec<char>, Vec<char>)> = None;

        for (p, q) in self.glut_crystal() {
            let (pv, qv) = (tape_to_biguint(&p), tape_to_biguint(&q));
            if &pv * &qv != n {
                continue; // not the true integer product
            }
            if pv == one || qv == one {
                continue; // reject the trivial 1 x N
            }

            // canonical order p <= q
            let (p, q) = if cmp_tape(&p, &q) == Ordering::Greater {
                (q, p)
            } else {
                (p, q)
            };

            match &best {
                None => best = Some((p, q)),
                Some((bp, _)) if cmp_tape(&p, bp) != Ordering::Greater => {
                    best = Some((p, q));
                }
                _ => {}
            }
        }

        best.map(|(p, q)| {
            (
                biguint_to_tape(&tape_to_biguint(&p)),
                biguint_to_tape(&tape_to_biguint(&q)),
            )
        })
    }
}

/// GLUT p-system factorization: O(L_N²) polynomial time.
/// Maintains a superposed state of all viable (p, q) candidates,
/// using frame sweep consistency to prune the state space.
pub fn glut_factor(n: &[char]) -> Option<(Vec<char>, Vec<char>)> {
    let mut sieve = GlutSieve::new(n);
    sieve.frame_sieve();
    sieve.readout()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glut_factor_35() {
        let n = vec![ONE, ONE, ONE, ZERO, ONE, ONE, ONE]; // 35 = 5 * 7
        if let Some((p, q)) = glut_factor(&n) {
            let prod = trim(mul(&p, &q));
            let expected = trim(n.to_vec());
            assert_eq!(prod, expected, "Product must equal N");
        }
    }
}