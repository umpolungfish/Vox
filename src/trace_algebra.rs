//! trace_algebra.rs — trace transformers over trace-WORDS. A candidate transform is a
//! word-level edit; replay is the verifier, judge is the verdict. No Rust predicate named
//! "minimal/redundant/compressible" — admissibility is defined operationally:
//!
//!     candidate is admissible  iff  replay_state(candidate) == replay_state(original)
//!                              and  judge(candidate) == ⊤
//!                              and  factor(candidate) == factor(original)
//!
//! Deletion is the sharp first transform: remove one record; if the candidate still
//! closes to the SAME factor, that record was operationally redundant in the trajectory.

use alloc::vec::Vec;
use crate::trace_word::{decode_trace, encode_trace, judge_trace};
use crate::router_marks::{M_T, M_N};
use crate::router_marks::M_FIX;
use crate::judge_g::found_factor;

pub type Mark = char;

/// The replay state of a trace word: where it starts, where it validates, and whether it
/// closed (last judgment ⊤ AND every record recognised).
#[derive(Clone, PartialEq, Debug)]
pub struct ReplayState {
    pub start: Mark,      // repr mark of the first record
    pub terminal: Mark,   // repr mark of the last record
    pub closed: bool,
    pub consistent: bool, // each record's next leads to the following record's repr (or FIX)
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

/// The factor a replay state hands for input n — the terminal representation judged.
pub fn factor_of(rp: &ReplayState, n: u64) -> Option<(u64, u64)> {
    if rp.closed { found_factor(rp.terminal, n) } else { None }
}

/// DELETE record i — the first transform. None if i is out of range.
pub fn delete_record(w: &[Mark], i: usize) -> Option<Vec<Mark>> {
    let mut t = decode_trace(w)?;
    if i >= t.len() { return None; }
    t.remove(i);
    Some(encode_trace(&t))
}

/// Is the candidate an admissible compression of the original for input n?
pub fn admissible(orig: &[Mark], cand: &[Mark], n: u64) -> bool {
    let (ro, rc) = match (replay_state(orig), replay_state(cand)) { (Some(a), Some(b)) => (a, b), _ => return false };
    if ro != rc { return false; }
    if judge_trace(cand) != M_T { return false; }
    factor_of(&ro, n) == factor_of(&rc, n)
}

/// Iterate deletion to a fixed point: no single record can be removed while preserving the
/// replay state, the ⊤ verdict and the factor. Returns the irreducible trace and the number
/// of successful transforms (each a shorter carrier found by replay, not by inspection).
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

/// T(τ*) = τ* — does one more transform pass change the trace? (Fixed-point test.)
pub fn is_fixed_point(w: &[Mark], n: u64) -> bool {
    let t = match decode_trace(w) { Some(t) => t, None => return false };
    for i in 0..t.len() {
        if let Some(cand) = delete_record(w, i) {
            if admissible(w, &cand, n) { return false; }
        }
    }
    true
}
