//! trace_algebra.rs — trace transformers over trace-WORDS. A candidate transform is a
//! word-level edit; replay is the verifier, judge is the verdict. No Rust predicate named
//! "minimal/redundant/compressible" — admissibility is defined operationally:
//!
//! ```text
//!     candidate is admissible  iff  replay_state(candidate) == replay_state(original)
//!                              and  judge(candidate) == ⊤
//!                              and  factor(candidate) == factor(original)
//! ```
//!
//! Deletion is the sharp first transform: remove one record; if the candidate still
//! closes to the SAME factor, that record was operationally redundant in the trajectory.

use alloc::vec::Vec;
use core::cmp::Ordering;
use crate::trace_word::{decode_trace, encode_trace, judge_trace};
use crate::router_marks::{M_T, M_N};
use crate::router_marks::M_FIX;
use crate::judge_g::found_factor;
use crate::morphism_factor::{cmp as tape_cmp, mul as tape_mul, tape_u64};

pub type Mark = char;

/// The replay state of a trace word: where it starts, where it validates, and whether it
/// closed (last judgment ⊤ AND every record recognised).
#[derive(Clone, PartialEq, Debug)]
pub struct ReplayState {
    pub start: Mark,
    pub terminal: Mark,
    pub closed: bool,
    pub consistent: bool,
}

pub fn replay_state(w: &[Mark]) -> Option<ReplayState> {
    let t = decode_trace(w)?;
    if t.is_empty() {
        return Some(ReplayState { start: M_N, terminal: M_N, closed: false, consistent: true });
    }
    let start = t[0].repr;
    let mut consistent = true;
    for k in 0..t.len().saturating_sub(1) {
        if t[k].next != t[k + 1].repr && t[k].next != M_FIX { consistent = false; }
    }
    let closed = t.last().map(|s| s.judgment == M_T).unwrap_or(false)
        && t.iter().all(|s| s.recognised == M_T);
    let terminal = t.last().map(|s| s.repr).unwrap_or(M_N);
    Some(ReplayState { start, terminal, closed, consistent })
}

/// Legacy strict replay projection.  This belongs to the historical strict
/// reducer, not to passive factor-carrier extraction.
pub fn factor_of(rp: &ReplayState, n: u64) -> Option<(u64, u64)> {
    if rp.closed { found_factor(rp.terminal, n) } else { None }
}

pub fn delete_record(w: &[Mark], i: usize) -> Option<Vec<Mark>> {
    let mut t = decode_trace(w)?;
    if i >= t.len() { return None; }
    t.remove(i);
    Some(encode_trace(&t))
}

pub fn admissible(orig: &[Mark], cand: &[Mark], n: u64) -> bool {
    let (ro, rc) = match (replay_state(orig), replay_state(cand)) { (Some(a), Some(b)) => (a, b), _ => return false };
    if ro != rc { return false; }
    if judge_trace(cand) != M_T { return false; }
    factor_of(&ro, n) == factor_of(&rc, n)
}

pub fn reduce(w: &[Mark], n: u64) -> (Vec<Mark>, usize) {
    let mut cur: Vec<Mark> = w.to_vec();
    let mut transforms = 0usize;
    'outer: loop {
        let t = match decode_trace(&cur) { Some(t) => t, None => break };
        for i in 0..t.len() {
            if let Some(cand) = delete_record(&cur, i) {
                if admissible(&cur, &cand, n) { cur = cand; transforms += 1; continue 'outer; }
            }
        }
        break;
    }
    (cur, transforms)
}

pub fn is_fixed_point(w: &[Mark], n: u64) -> bool {
    let t = match decode_trace(w) { Some(t) => t, None => return false };
    for i in 0..t.len() {
        if let Some(cand) = delete_record(w, i) {
            if admissible(w, &cand, n) { return false; }
        }
    }
    true
}

// ---- canonical relaxed carrier relation (≡c): arbitrary-width tapes only ----

#[derive(Clone, PartialEq, Debug)]
pub struct ClosureState {
    pub terminal: Mark,
    pub closed: bool,
}

pub fn closure_state(w: &[Mark]) -> Option<ClosureState> {
    let rp = replay_state(w)?;
    Some(ClosureState { terminal: rp.terminal, closed: rp.closed })
}

/// Verify an already-carried arbitrary-width factor witness entirely over IMASM
/// numeral tapes.  This multiplies p×q and compares with N; it never searches
/// for either factor and never narrows an operand to a machine integer.
pub fn witness_valid(n: &[Mark], p: &[Mark], q: &[Mark]) -> bool {
    let one = tape_u64(1);
    if tape_cmp(p, &one) != Ordering::Greater || tape_cmp(q, &one) != Ordering::Greater {
        return false;
    }
    tape_cmp(&tape_mul(p, q), n) == Ordering::Equal
}

fn same_witness(a: (&[Mark], &[Mark]), b: (&[Mark], &[Mark])) -> bool {
    let direct = tape_cmp(a.0, b.0) == Ordering::Equal
        && tape_cmp(a.1, b.1) == Ordering::Equal;
    let swapped = tape_cmp(a.0, b.1) == Ordering::Equal
        && tape_cmp(a.1, b.0) == Ordering::Equal;
    direct || swapped
}

/// Frozen relaxed relation ≡c for an arbitrary-width factor-bearing carrier.
/// The factor witness is object data; equality and reconstruction are evaluated
/// on tapes only.
pub fn relaxed_equivalent_with_witness(
    a: &[Mark],
    witness_a: (&[Mark], &[Mark]),
    b: &[Mark],
    witness_b: (&[Mark], &[Mark]),
    n: &[Mark],
) -> bool {
    if !witness_valid(n, witness_a.0, witness_a.1)
        || !witness_valid(n, witness_b.0, witness_b.1)
    {
        return false;
    }
    if !same_witness(witness_a, witness_b) { return false; }
    if judge_trace(a) != M_T || judge_trace(b) != M_T { return false; }
    match (closure_state(a), closure_state(b)) {
        (Some(ca), Some(cb)) => ca == cb,
        _ => false,
    }
}

/// One relaxed candidate step for a factor-bearing carrier.  The witness remains
/// data while only the trace projection is edited.
pub fn admissible_relaxed_with_witness(
    orig: &[Mark],
    cand: &[Mark],
    n: &[Mark],
    p: &[Mark],
    q: &[Mark],
) -> bool {
    relaxed_equivalent_with_witness(orig, (p, q), cand, (p, q), n)
}
