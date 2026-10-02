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
//! 1. Skin membrane: fold the source's low zero run through product valuation.
//! 2. Frame sweep: visit all proper factor-width geometries fairly.
//! 3. Glut superposition: retain free factor cells as symbolic masks and intervals.
//! 4. Glut multiplication: constrain high product bounds and low convolution carry.
//! 5. Glut sieve: intersect exact mask extrema; split unresolved cells dynamically.
//! 6. Glut crystal: materialize the selected assignment through actual carry
//!    transitions and verify the complete product and checkpoint ancestry.
//! 7. Glut readout: transport the exact proper pair and executed checkpoints.
//!
//! Production extraction folds the relation without a state, width or decision
//! quota. Explicit frame queries remain available for concrete state censuses.

use alloc::collections::BTreeSet;
use alloc::rc::Rc;
use alloc::string::String;
use alloc::vec::Vec;
use core::cmp::Ordering;
use crate::morphism_factor as mf;

#[path = "glut_fold.rs"]
mod fold;
pub use fold::FoldStats;
#[path = "glut_carrier.rs"]
mod carrier;
pub use carrier::PreparedGlutMembrane;

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
    pub carry: Vec<char>,
    pub p_prefix: Vec<char>,
    pub q_prefix: Vec<char>,
    pub position: usize,
}

fn position_tape(mut value: usize) -> Vec<char> {
    let mut tape = Vec::new();
    while value != 0 {
        tape.push(if value & 1 == 1 { ONE } else { ZERO });
        value >>= 1;
    }
    trim(tape)
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
                        carry: vec![ZERO],
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
                let mut sum = self.carry.clone();
                for i in 0..k {
                    let pi = bit(p, i);
                    let qi = bit(q, k - i);
                    if pi * qi == 1 { sum = mf::add(&sum, &[ONE]); }
                }
                if p_bit * bit(q, 0) == 1 { sum = mf::add(&sum, &[ONE]); }
                if bit(p, 0) * q_bit == 1 { sum = mf::add(&sum, &[ONE]); }
                if bit(&sum, 0) == n_bit {
                    let mut new_p = p.clone();
                    let mut new_q = q.clone();
                    new_p.push(if p_bit == 1 { ONE } else { ZERO });
                    new_q.push(if q_bit == 1 { ONE } else { ZERO });
                    results.push(Self {
                        carry: trim(sum[1..].to_vec()),
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
        self.checked_carry(n) == Some(self.carry.clone())
    }

    /// Reverse rail: replay every convolution column and recover its carry.
    fn checked_carry(&self, n: &[char]) -> Option<Vec<char>> {
        if self.position == 0 || self.position > trim(n.to_vec()).len()
            || self.p_prefix.len() != self.position
            || self.q_prefix.len() != self.position
            || n.is_empty()
            || n.iter().chain(&self.p_prefix).chain(&self.q_prefix)
                .any(|c| !matches!(*c, ZERO | ONE))
        {
            return None;
        }
        let mut carry = vec![ZERO];
        for k in 0..self.position {
            let mut sum = carry.clone();
            for i in 0..=k {
                if bit(&self.p_prefix, i) * bit(&self.q_prefix, k - i) == 1 {
                    sum = mf::add(&sum, &[ONE]);
                }
            }
            if bit(&sum, 0) != bit(n, k) { return None; }
            carry = trim(sum[1..].to_vec());
        }
        Some(carry)
    }

    /// Every completion has a product at least as large as this prefix product.
    pub fn can_close(&self, n: &[char]) -> bool {
        self.verify_product_bit(n)
            && mf::cmp(&mf::mul(&self.p_prefix, &self.q_prefix), n) != Ordering::Greater
    }

    pub fn is_complete(&self, total_bits: usize) -> bool {
        self.position >= total_bits
    }
}

/// The glut source relation. Production extraction keeps free cells folded.
/// `states` exposes concrete frame materializations and the selected closure.
/// Explicit `frame_superpose` queries enumerate their requested finite frame;
/// `frame_sweep` solves the source relation without a breadth-state vector.
pub struct GlutSieve {
    pub n_tape: Vec<char>,
    pub n_bits: Vec<u8>,
    pub states: Vec<GlutState>,
    paths: Vec<Rc<GlutPath>>,
    pub fold_stats: FoldStats,
}

struct GlutPath {
    state: GlutState,
    parent: Option<Rc<GlutPath>>,
}

/// The selected factor's executed frame boundaries, including its seed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GlutExecution {
    pub p: Vec<char>,
    pub q: Vec<char>,
    pub checkpoints: Vec<GlutState>,
}

impl GlutSieve {
    pub fn new(n: &[char]) -> Self {
        let trimmed = trim(n.to_vec());
        let n_bits: Vec<u8> = trimmed.iter().map(|c| u8::from(*c == ONE)).collect();
        let states = if n.iter().all(|c| matches!(*c, ZERO | ONE)) && !n.is_empty() {
            GlutState::seed(bit(&trimmed, 0))
        } else { Vec::new() };
        let paths = states.iter().map(|state| Rc::new(GlutPath {
            state: state.clone(), parent: None,
        })).collect();
        Self {
            n_tape: trimmed.clone(),
            n_bits,
            states,
            paths,
            fold_stats: FoldStats::default(),
        }
    }

    /// One frame: advance every state by `width` bits (or as far as N runs).
    /// Dedup by (p_prefix, q_prefix) is loss-free — the carry is a function
    /// of the prefixes.
    pub fn frame_superpose(&mut self, width: usize) -> usize {
        let mut next_states = Vec::new();
        let mut next_paths = Vec::new();
        for (index, state) in self.states.iter().enumerate() {
            // A public-state mutation must pass both ancestry and reverse checks.
            let Some(parent) = self.paths.get(index) else { continue; };
            if parent.state != *state || !state.can_close(&self.n_tape) { continue; }
            let pos = state.position.min(self.n_bits.len());
            if pos >= self.n_bits.len() || width == 0 {
                next_states.push(state.clone());
                next_paths.push(parent.clone());
                continue;
            }
            let bits_to_process = width.min(self.n_bits.len() - pos);
            let branches = self.propagate_block(state, pos, bits_to_process);
            for branch in branches {
                next_paths.push(Rc::new(GlutPath {
                    state: branch.clone(), parent: Some(parent.clone()),
                }));
                next_states.push(branch);
            }
        }
        let mut seen = BTreeSet::new();
        let mut kept = Vec::new();
        let mut kept_paths = Vec::new();
        for (s, path) in next_states.into_iter().zip(next_paths) {
            if seen.insert((s.p_prefix.clone(), s.q_prefix.clone())) {
                kept.push(s);
                kept_paths.push(path);
            }
        }
        self.states = kept;
        self.paths = kept_paths;
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
                // Descend one cell, then ascend through the product/carry check.
                next.extend(s.advance(n_bit).into_iter()
                    .filter(|candidate| candidate.can_close(&self.n_tape)));
            }
            current = next;
            if current.is_empty() {
                return alloc::vec![];
            }
        }
        current
    }

    /// Fold the source relation and close at the first verified proper product.
    pub fn frame_sweep(&mut self) {
        self.frame_sweep_observed(|_, _| {});
    }

    /// Observe the materialized boundary and symbolic frontier without a quota.
    /// Width zero identifies seed or fold-progress observations; `fold_stats`
    /// distinguishes them. A closing observation projects the selected fold.
    pub fn frame_sweep_observed(&mut self, mut observe: impl FnMut(&Self, usize)) {
        // Validate materialized ancestry before opening the source relation.
        // Resuming a frame uses folding too, rather than a breadth fallback.
        self.frame_superpose(0);
        observe(self, 0);
        if self.states.is_empty() { return; }
        let source = self.n_tape.clone();
        let (pair, stats) = fold::solve(&source, |stats| {
            self.fold_stats = stats.clone();
            observe(self, 0);
        });
        self.fold_stats = stats;
        self.states.clear(); self.paths.clear();
        if let Some((p, q)) = pair {
            let execution = materialize_fold(&source, p, q)
                .expect("folded assignment failed its verified materialization");
            let mut parent = None;
            for state in &execution.checkpoints {
                parent = Some(Rc::new(GlutPath { state: state.clone(), parent }));
            }
            self.states.push(execution.checkpoints.last().unwrap().clone());
            self.paths.push(parent.unwrap());
            observe(self, self.states[0].position - 1);
        }
    }

    /// Close the current prefix congruence to full integer product equality.
    pub fn glut_crystal(&self) -> Vec<(Vec<char>, Vec<char>)> {
        let mut pairs = Vec::new();
        let mut seen = BTreeSet::new();
        for s in &self.states {
            if !s.verify_product_bit(&self.n_tape) { continue; }
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

    /// Glut readout: a proper pair in canonical numeric order.
    pub fn readout(&self) -> Option<(Vec<char>, Vec<char>)> {
        self.glut_crystal().into_iter().filter_map(|(p, q)| {
            if !crate::trace_algebra::witness_valid(&self.n_tape, &p, &q) { return None; }
            Some(if mf::cmp(&p, &q) == Ordering::Greater { (q, p) } else { (p, q) })
        }).min_by(|a, b| mf::cmp(&a.0, &b.0))
    }

    pub fn readout_execution(&self) -> Option<GlutExecution> {
        let (p, q) = self.readout()?;
        let path = self.paths.iter().find(|path| {
            let left = trim(path.state.p_prefix.clone());
            let right = trim(path.state.q_prefix.clone());
            (left == p && right == q) || (left == q && right == p)
        })?;
        let swap = trim(path.state.p_prefix.clone()) != p;
        let mut checkpoints = Vec::new();
        let mut current = Some(path.clone());
        while let Some(link) = current {
            let mut state = link.state.clone();
            if swap { core::mem::swap(&mut state.p_prefix, &mut state.q_prefix); }
            checkpoints.push(state);
            current = link.parent.clone();
        }
        checkpoints.reverse();
        let execution = GlutExecution { p, q, checkpoints };
        execution.verify(&self.n_tape).ok()?;
        Some(execution)
    }
}

/// Materialize the selected symbolic assignment through the actual convolution
/// rail. The checkpoints record these executed transitions, including folded
/// zero runs, and the same reverse rail checks the returned integer closure.
fn materialize_fold(n: &[char], p: Vec<char>, q: Vec<char>) -> Result<GlutExecution, String> {
    let mut state = GlutState::seed(bit(n, 0)).into_iter().find(|s|
        bit(&s.p_prefix, 0) == bit(&p, 0) && bit(&s.q_prefix, 0) == bit(&q, 0))
        .ok_or("folded assignment has no parity seed")?;
    let mut checkpoints = vec![state.clone()];
    let end = p.len().max(q.len());
    let mut boundary = 2;
    while state.position < end {
        let k = state.position;
        state = state.advance(bit(n, k)).into_iter().find(|candidate|
            bit(&candidate.p_prefix, k) == bit(&p, k)
                && bit(&candidate.q_prefix, k) == bit(&q, k))
            .ok_or("folded assignment failed its convolution return")?;
        if state.position == boundary || state.position == end {
            checkpoints.push(state.clone());
            boundary = boundary.checked_mul(2).unwrap_or(end).min(end);
        }
    }
    let execution = GlutExecution { p, q, checkpoints };
    execution.verify(n)?;
    Ok(execution)
}

/// GLUT p-system factorization.
pub fn glut_factor(n: &[char]) -> Option<(Vec<char>, Vec<char>)> {
    glut_factor_with_stats(n).0
}

/// GLUT factorization with the completed symbolic-fold work record.
pub fn glut_factor_with_stats(n: &[char]) -> (Option<(Vec<char>, Vec<char>)>, FoldStats) {
    let mut sieve = GlutSieve::new(n);
    sieve.frame_sweep();
    (sieve.readout(), sieve.fold_stats)
}

pub fn glut_factor_execution(n: &[char]) -> Option<GlutExecution> {
    let mut sieve = GlutSieve::new(n);
    sieve.frame_sweep();
    sieve.readout_execution()
}

pub fn glut_factor_execution_with_stats(
    n: &[char],
) -> (Option<GlutExecution>, FoldStats) {
    let mut sieve = GlutSieve::new(n);
    sieve.frame_sweep();
    (sieve.readout_execution(), sieve.fold_stats)
}

/// Close one nested product-correlation membrane without running the
/// multi-port frame, square, admission, or mask schedule.
pub fn glut_correlation_execution(n: &[char]) -> Option<GlutExecution> {
    fold::correlation_only(n)
}

impl GlutExecution {
    /// Replay adjacent frame maps from the parity seed to exact product closure.
    pub fn verify(&self, n: &[char]) -> Result<(), String> {
        if !crate::trace_algebra::witness_valid(n, &self.p, &self.q)
            || mf::cmp(&self.p, &self.q) == Ordering::Greater
        {
            return Err("glut execution does not carry a canonical proper pair".into());
        }
        let seed = self.checkpoints.first().ok_or("glut execution has no seed")?;
        if !GlutState::seed(bit(n, 0)).contains(seed) || !seed.can_close(n) {
            return Err("glut execution has an invalid parity seed".into());
        }
        for pair in self.checkpoints.windows(2) {
            let (before, after) = (&pair[0], &pair[1]);
            if after.position <= before.position || !after.can_close(n) {
                return Err("glut frame failed its reverse carry/product check".into());
            }
            let mut current = before.clone();
            for k in before.position..after.position {
                current = current.advance(bit(n, k)).into_iter().find(|candidate| {
                    candidate.p_prefix == after.p_prefix[..k + 1]
                        && candidate.q_prefix == after.q_prefix[..k + 1]
                    // The checked frame bounds every intervening prefix product.
                    // Advance preserves the verified carry recurrence; replay reuses
                    // that boundary instead of rechecking all earlier columns.
                }).ok_or("glut frame does not descend from its preceding state")?;
            }
            if current != *after {
                return Err("glut frame's returned state differs from its descent".into());
            }
        }
        let last = self.checkpoints.last().unwrap();
        if trim(last.p_prefix.clone()) != self.p
            || trim(last.q_prefix.clone()) != self.q
        {
            return Err("glut execution does not end on its factor witness".into());
        }
        Ok(())
    }

    /// Actual frame states travel in the applied payload of the routing trace.
    /// Each record follows the resident source frame; its length grows with
    /// the retained payload instead of truncating to a fixed bit count.
    pub fn trace(&self, n: &[char]) -> Result<Vec<char>, String> {
        use crate::router_marks::{GStep, M_B, M_FIX, M_T};
        self.verify(n)?;
        let mut payload = vec!['⊢'];
        for state in &self.checkpoints {
            for field in [position_tape(state.position), state.carry.clone(),
                state.p_prefix.clone(), state.q_prefix.clone()]
            {
                payload.push('∈');
                payload.extend(field);
                payload.push('∋');
            }
        }
        payload.push('⊣');
        let extent = n.len() * 2;
        let count = payload.len().div_ceil(extent);
        let steps: Vec<_> = payload.chunks(extent).enumerate().map(|(index, chunk)| {
            let terminal = index + 1 == count;
            GStep { repr: '⋈', judgment: if terminal { M_T } else { M_B },
                recognised: M_T, next: if terminal { M_FIX } else { '⋈' },
                applied_word: chunk.to_vec() }
        }).collect();
        Ok(crate::trace_word::encode_trace(&steps))
    }
}

/// Verify the glut computation transported inside a factor carrier's trace.
/// Passive re-entry verification can then operate on the same source trace.
pub fn verify_glut_trace(n: &[char], p: &[char], q: &[char], trace: &[char]) -> Result<(), String> {
    use crate::router_marks::{M_B, M_FIX, M_T};
    let steps = crate::trace_word::decode_trace(trace).ok_or("malformed glut trace")?;
    if steps.is_empty() { return Err("empty glut trace".into()); }
    let mut payload = Vec::new();
    for (index, step) in steps.iter().enumerate() {
        let terminal = index + 1 == steps.len();
        if step.repr != '⋈' || step.recognised != M_T
            || step.judgment != if terminal { M_T } else { M_B }
            || step.next != if terminal { M_FIX } else { '⋈' }
            || step.applied_word.is_empty()
        {
            return Err("invalid glut trace routing".into());
        }
        payload.extend_from_slice(&step.applied_word);
    }
    if payload.first() != Some(&'⊢') || payload.last() != Some(&'⊣') {
        return Err("invalid glut execution envelope".into());
    }
    fn field(payload: &[char], cursor: &mut usize) -> Result<Vec<char>, String> {
        if payload.get(*cursor) != Some(&'∈') { return Err("missing glut state field".into()); }
        *cursor += 1;
        let start = *cursor;
        while matches!(payload.get(*cursor), Some(&ZERO) | Some(&ONE)) { *cursor += 1; }
        if start == *cursor || payload.get(*cursor) != Some(&'∋') {
            return Err("malformed glut state field".into());
        }
        let value = payload[start..*cursor].to_vec();
        *cursor += 1;
        Ok(value)
    }
    fn integer(tape: &[char]) -> Result<usize, String> {
        tape.iter().rev().try_fold(0usize, |value, &mark| {
            value.checked_mul(2)?.checked_add(usize::from(mark == ONE))
        }).ok_or("glut state index overflows host address space".into())
    }
    let mut cursor = 1;
    let mut checkpoints = Vec::new();
    while cursor < payload.len() - 1 {
        let position = integer(&field(&payload, &mut cursor)?)?;
        let carry = field(&payload, &mut cursor)?;
        let p_prefix = field(&payload, &mut cursor)?;
        let q_prefix = field(&payload, &mut cursor)?;
        checkpoints.push(GlutState { position, carry, p_prefix, q_prefix });
    }
    if cursor != payload.len() - 1 { return Err("trailing glut execution data".into()); }
    GlutExecution { p: p.to_vec(), q: q.to_vec(), checkpoints }.verify(n)
}

/// Bind passive reduction to the independently replayed glut producer.
pub fn verify_glut_reentry_certificate(
    certificate: &crate::reentry_certificate::ReentryCertificate,
) -> Result<crate::reentry_certificate::ReentryCertificateSummary, String> {
    let source = &certificate.links.first().ok_or("glut certificate has no source")?.before;
    verify_glut_trace(&certificate.n, &certificate.p, &certificate.q, source)?;
    crate::reentry_certificate::verify_reentry_certificate(certificate)
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
    fn resumed_sparse_frame_uses_the_same_folded_source_relation() {
        let mut n = vec![ZERO; 256]; n.push(ONE);
        let mut sieve = GlutSieve::new(&n);
        sieve.frame_superpose(2);
        assert!(sieve.states.len() > 3);
        sieve.frame_sweep();
        let execution = sieve.readout_execution().unwrap();
        execution.verify(&n).unwrap();
        assert_eq!(sieve.fold_stats.zero_run, 256);
        assert_eq!(sieve.fold_stats.peak_frontier, 1);
        assert_eq!(execution.p, execution.q);
    }

    #[test]
    fn folded_sweep_rejects_mutated_materialized_ancestry() {
        let n = to_tape(35);
        let mut sieve = GlutSieve::new(&n);
        sieve.states[0].carry = vec![ONE];
        sieve.frame_sweep();
        assert!(sieve.states.is_empty());
        assert!(sieve.readout_execution().is_none());
    }

    #[test]
    fn carry_folds_beyond_machine_width() {
        let mut state = GlutState::seed(1).remove(0);
        state.carry = vec![ONE; 130];
        let next = state.advance(1);
        assert!(!next.is_empty());
        assert!(next.iter().all(|s| s.carry.len() >= 129));
        assert!(next.iter().all(|s| !s.verify_product_bit(&to_tape(3))));
    }

    #[test]
    fn test_glut_factor_35() {
        let n = to_tape(35);
        let (p, q) = glut_factor(&n).expect("35 must factor");
        assert_eq!((p.clone(), q.clone()), (to_tape(5), to_tape(7)));
        assert_eq!(
            cmp(&crate::morphism_factor::mul(&p, &q), &n),
            core::cmp::Ordering::Equal
        );
    }

    #[test]
    fn test_glut_factor_8051() {
        let n = to_tape(8051);
        let (p, q) = glut_factor(&n).expect("8051 must factor");
        assert_eq!((p.clone(), q.clone()), (to_tape(83), to_tape(97)));
        assert_eq!(
            cmp(&crate::morphism_factor::mul(&p, &q), &n),
            core::cmp::Ordering::Equal
        );
    }

    #[test]
    fn test_glut_factor_even_56() {
        let n = to_tape(56);
        let (p, q) = glut_factor(&n).expect("56 must factor");
        assert_eq!((p.clone(), q.clone()), (to_tape(7), to_tape(8)));
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
            // Enumerate the full crystal here; factor extraction itself latches
            // as soon as a proper pair reaches exact equality.
            while s.states.iter().any(|state| !state.is_complete(n.len())) {
                s.frame_superpose(2);
            }
            let mut alias = GlutState { carry: vec![ZERO], p_prefix: to_tape(9),
                q_prefix: to_tape(11), position: n.len() };
            alias.p_prefix.resize(n.len(), ZERO);
            alias.q_prefix.resize(n.len(), ZERO);
            alias.carry = alias.checked_carry(&n).unwrap();
            assert!(alias.verify_product_bit(&n));
            assert!(!alias.can_close(&n));
            s.states.push(alias);
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
        // The raw congruence relation doubles. The live frame rail additionally
        // rejects prefix products above N, so these are separate state sets.
        let n = to_tape(8051);
        let mut states = GlutState::seed(bit(&n, 0));
        for position in 1..=n.len() {
            assert_eq!(states.len(), 1usize << (position - 1));
            assert!(states.iter().all(|s| s.verify_product_bit(&n)));
            if position < n.len() {
                states = states.iter().flat_map(|s| s.advance(bit(&n, position))).collect();
            }
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

    #[test]
    fn corrupt_carry_prefix_and_position_are_rejected() {
        let n = to_tape(8051);
        let mut sieve = GlutSieve::new(&n);
        sieve.frame_superpose(3);
        let valid = sieve.states[0].clone();
        let mut changed = valid.clone();
        changed.carry = mf::add(&changed.carry, &[ONE]);
        assert!(!changed.verify_product_bit(&n));
        changed = valid.clone();
        changed.p_prefix[0] = ZERO;
        assert!(!changed.verify_product_bit(&n));
        changed = valid;
        changed.position += 1;
        assert!(!changed.verify_product_bit(&n));
        for state in &mut sieve.states { state.carry = mf::add(&state.carry, &[ONE]); }
        assert_eq!(sieve.frame_superpose(2), 0);
    }

    #[test]
    fn live_frames_preserve_every_exact_product_and_reject_overflow_aliases() {
        for value in 0..=128 {
            let n = to_tape(value);
            let mut expected = BTreeSet::new();
            for p in 0..=value {
                for q in 0..=value {
                    if p * q == value { expected.insert((p, q)); }
                }
            }
            for width in [1, 2, 5, 8] {
                let mut sieve = GlutSieve::new(&n);
                while sieve.states.iter().any(|s| !s.is_complete(n.len())) {
                    sieve.frame_superpose(width);
                    assert!(sieve.states.iter().all(|s| s.can_close(&n)));
                }
                let actual: BTreeSet<_> = sieve.glut_crystal().into_iter().map(|(p, q)| {
                    (mf::dec_of(&p).parse::<u64>().unwrap(), mf::dec_of(&q).parse::<u64>().unwrap())
                }).collect();
                // Zero also has (0,1)/(1,0) in the one-bit carrier.
                if value > 0 { assert_eq!(actual, expected, "N={value}, width={width}"); }
                let proper = expected.iter().find(|&&(p, q)| p > 1 && q > 1 && p <= q);
                assert_eq!(sieve.readout(), proper.map(|&(p, q)| (to_tape(p), to_tape(q))));
            }
        }
    }

    #[test]
    fn executed_trace_survives_certificate_transport_and_rejects_mutations() {
        use crate::factor_extract::FactorCarrier;
        use crate::reentry_certificate::{certify_reentry, decode_reentry_certificate,
            encode_reentry_certificate, verify_reentry_certificate};
        let n = to_tape(8051);
        let execution = glut_factor_execution(&n).unwrap();
        assert!(execution.checkpoints.last().unwrap().position < n.len());
        let trace = execution.trace(&n).unwrap();
        verify_glut_trace(&n, &execution.p, &execution.q, &trace).unwrap();
        let carrier = FactorCarrier::new(n.clone(), execution.p.clone(), execution.q.clone(), trace).unwrap();
        let certificate = certify_reentry(&carrier).unwrap();
        let decoded = decode_reentry_certificate(&encode_reentry_certificate(&certificate)).unwrap();
        verify_reentry_certificate(&decoded).unwrap();
        verify_glut_reentry_certificate(&decoded).unwrap();
        verify_glut_trace(&decoded.n, &decoded.p, &decoded.q, &decoded.links[0].before).unwrap();

        let mut changed = execution.clone();
        let last = changed.checkpoints.last_mut().unwrap();
        last.carry = mf::add(&last.carry, &[ONE]);
        assert!(changed.verify(&n).is_err());
        assert!(changed.trace(&n).is_err());
        let mut steps = crate::trace_word::decode_trace(&decoded.links[0].before).unwrap();
        // The seed's first prefix bit is preceded by position and carry fields.
        let payload = &mut steps[0].applied_word;
        let prefix_field = payload.iter().enumerate().filter(|(_, c)| **c == '∈')
            .nth(2).unwrap().0;
        payload[prefix_field + 1] = ZERO;
        let altered_trace = crate::trace_word::encode_trace(&steps);
        assert!(verify_glut_trace(&n, &execution.p, &execution.q, &altered_trace).is_err());
        let altered_carrier = FactorCarrier::new(n.clone(), execution.p.clone(),
            execution.q.clone(), altered_trace).unwrap();
        let altered_certificate = certify_reentry(&altered_carrier).unwrap();
        assert!(verify_glut_reentry_certificate(&altered_certificate).is_err());
        assert!(verify_glut_trace(&n, &execution.p, &execution.q, &[]).is_err());
    }

    #[test]
    fn wide_execution_payload_crosses_trace_records_losslessly() {
        use crate::factor_extract::FactorCarrier;
        use crate::reentry_certificate::{certify_reentry, decode_reentry_certificate,
            encode_reentry_certificate};
        let n = mf::parse_numeral(include_str!("../tests/fixtures/random_rsa_2048.imasm").trim()).unwrap();
        let p = mf::parse_numeral(include_str!("../tests/fixtures/random_rsa_2048.p.imasm").trim()).unwrap();
        let q = mf::parse_numeral(include_str!("../tests/fixtures/random_rsa_2048.q.imasm").trim()).unwrap();
        let execution = materialize_fold(&n, p, q).unwrap();
        let trace = execution.trace(&n).unwrap();
        let decoded = crate::trace_word::decode_trace(&trace).unwrap();
        assert!(decoded.len() > 1);
        assert!(decoded.iter().any(|step| step.applied_word.len() > u8::MAX as usize));
        verify_glut_trace(&n, &execution.p, &execution.q, &trace).unwrap();
        let carrier = FactorCarrier::new(n, execution.p, execution.q, trace).unwrap();
        let certificate = certify_reentry(&carrier).unwrap();
        let decoded = decode_reentry_certificate(&encode_reentry_certificate(&certificate)).unwrap();
        verify_glut_reentry_certificate(&decoded).unwrap();
    }
}
