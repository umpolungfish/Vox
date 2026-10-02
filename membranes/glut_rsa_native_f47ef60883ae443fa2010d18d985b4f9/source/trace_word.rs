//! trace_word.rs — the trace is only a word. A trace record is
//!
//! ```text
//!     ∈ <repr> <judgment> <recognised> <next> ∈ <length> ∋ <applied-word> ∋
//! ```
//!
//! and TRACE := RECORD*. The applied word travels BY COUNT over {⊤=1,⊥=0}.
//! The length frame grows with the payload; existing eight-mark frames retain
//! their byte representation.
//! An applied word that itself contains ∈/∋ is read unambiguously and the whole
//! trace round-trips byte-for-byte. `GStep` is a reader/view over positions — the
//! production representation is the word.

use alloc::vec::Vec;
use crate::router_marks::{GStep, M_T, M_F, M_B, M_N, M_FIX};

pub type Mark = char;

/// Encode one record with its complete payload length.
pub fn encode_step(s: &GStep) -> Vec<Mark> {
    let n = s.applied_word.len();
    let width = ((usize::BITS - n.leading_zeros()) as usize).max(8);
    let mut v: Vec<Mark> = Vec::with_capacity(8 + width + n);
    v.push('∈');
    v.push(s.repr);
    v.push(s.judgment);
    v.push(s.recognised);
    v.push(s.next);
    v.push('∈');
    for bit in (0..width).rev() { v.push(if (n >> bit) & 1 == 1 { M_T } else { M_F }); }
    v.push('∋');
    v.extend_from_slice(&s.applied_word);
    v.push('∋');
    v
}

/// Read the delimited length and payload without allocating from an unchecked
/// count. Both the trace reader and resident editor use this framing rule.
pub(crate) fn read_payload<'a>(w: &'a [Mark], i: &mut usize) -> Option<&'a [Mark]> {
    if *w.get(*i)? != '∈' { return None; }
    *i += 1;
    let start = *i;
    let mut len = 0usize;
    loop {
        let mark = *w.get(*i)?;
        if mark == '∋' { break; }
        let bit = match mark { '⊤' => 1usize, '⊥' => 0usize, _ => return None };
        len = len.checked_mul(2)?.checked_add(bit)?;
        *i += 1;
    }
    if *i == start { return None; }
    *i += 1;
    let end = i.checked_add(len)?;
    let payload = w.get(*i..end)?;
    *i = end;
    Some(payload)
}

/// Encode a whole trajectory: concatenated record words.
pub fn encode_trace(t: &[GStep]) -> Vec<Mark> {
    let mut v = Vec::new();
    for s in t { v.extend_from_slice(&encode_step(s)); }
    v
}

/// Decode a trace word back to steps. None if any record is malformed — F's address.
pub fn decode_trace(w: &[Mark]) -> Option<Vec<GStep>> {
    let mut i = 0usize;
    let mut out: Vec<GStep> = Vec::new();
    while i < w.len() {
        if w[i] != '∈' { return None; }
        i += 1;
        let repr = *w.get(i)?; i += 1;
        let judgment = *w.get(i)?; i += 1;
        let recognised = *w.get(i)?; i += 1;
        let next = *w.get(i)?; i += 1;
        let aw = read_payload(w, &mut i)?.to_vec();
        if *w.get(i)? != '∋' { return None; }
        i += 1;
        out.push(GStep { repr, judgment, recognised, next, applied_word: aw });
    }
    Some(out)
}

/// Judge a TRACE word — the trajectory re-entered as an object. The same four marks:
///   ⊤ = the trajectory closed and reconstruction verified
///   ⊞ = the trajectory contains a productive unresolved fork
///   ⊙ = the trajectory contains no distinction for the encountered state
///   ⊥ = the trace word itself is malformed
pub fn judge_trace(w: &[Mark]) -> Mark {
    let t = match decode_trace(w) { Some(t) => t, None => return M_F };
    let closed = t.last().map(|s| s.judgment == M_T).unwrap_or(false);
    let all_recognised = t.iter().all(|s| s.recognised == M_T);
    if closed && all_recognised { return M_T; }
    if t.iter().any(|s| s.judgment == M_B) { return M_B; }
    if t.iter().any(|s| s.judgment == M_N) { return M_N; }
    // a trajectory that neither closed nor forked nor saw no-distinction: still no distinction
    M_N
}

/// Replay a trace word by POSITION, not by semantics: an applied word of ⊙ means no
/// transformation was committed; any other applied word is the operation to execute.
/// `exec` receives each committed applied word in order; ⊙ is never handed to it.
pub fn replay<F: FnMut(&[Mark])>(w: &[Mark], mut exec: F) -> Option<usize> {
    let t = decode_trace(w)?;
    let mut committed = 0usize;
    for s in &t {
        if s.applied_word.len() == 1 && s.applied_word[0] == M_N { continue; } // no operation
        if s.applied_word.is_empty() { continue; }
        exec(&s.applied_word);
        committed += 1;
    }
    Some(committed)
}

/// A GStep is terminal iff its next is FIX (⊡).
pub fn is_terminal(s: &GStep) -> bool { s.next == M_FIX }

/// The router judged from its trajectory WORDS — the same clauses as judge_router_v2,
/// read off the trace tape instead of the enum steps. ⊤ all closed · ⊞ partial · ⊙ any
/// unmatched · ⊥ a malformed trace word.
pub fn judge_router_trace(trace_words: &[Vec<Mark>]) -> Mark {
    let mut any_unmatched = false;
    let mut n_closed = 0usize;
    for w in trace_words {
        let t = match decode_trace(w) { Some(t) => t, None => return M_F };
        if t.iter().any(|s| s.recognised == M_N) { any_unmatched = true; }
        if t.last().map(|s| s.judgment == M_T).unwrap_or(false) { n_closed += 1; }
    }
    if any_unmatched { return M_N; }
    if n_closed == trace_words.len() { return M_T; }
    if n_closed == 0 { return M_N; }
    M_B
}
