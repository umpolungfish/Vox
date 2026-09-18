//! router_marks.rs — the router whose clause KEYS are marks.
//!
//! The production clause type stores no enum. A clause is four marks and a word:
//!
//!     RouteClauseG { judgment: Mark, source: Mark, transform_word: Vec<Mark>, next: Mark }
//!
//! Matching is grammatical equality — `clause.judgment == jmark && (clause.source ==
//! smark || clause.source == ⊙)` — and ⊙ is exactly what it always was: no-distinction
//! in the judgment slot, wildcard/no-restriction in the source slot. Same mark; the
//! SLOT supplies the reading. `Belnap` and `ReprTag` survive only in `router_object.rs`
//! as the oracle the equivalence harness is measured against.

use alloc::string::String;
use alloc::vec::Vec;
use crate::router_object::{RouterObject, R_ANY};
use crate::judge_g::{judge_g, repr_value, found_factor, REPR_TAG_SYM, REPR_TAG_RES};
use crate::nested_frame::isqrt_u64;

pub type Mark = char;

pub const M_T: Mark = '⊤';
pub const M_F: Mark = '⊥';
pub const M_B: Mark = '⊞';
pub const M_N: Mark = '⊙';   // == R_ANY: no-distinction in judgment, wildcard in source
pub const M_FIX: Mark = '⊡';

/// One routing clause, all marks: (judgment × source) → (transform_word, next).
#[derive(Clone)]
pub struct RouteClauseG {
    pub judgment: Mark,
    pub source: Mark,          // ⊙ = wildcard (any source)
    pub transform_word: Vec<Mark>,
    pub next: Mark,            // ⊡ = FIX (terminal)
}

/// The router, keyed by marks. No enum field appears.
#[derive(Clone)]
pub struct RouterG {
    pub clauses: Vec<RouteClauseG>,
}

impl RouterG {
    /// Encode the enum oracle's clauses into mark keys — the one place the enum is read.
    pub fn from_enum(r: &RouterObject) -> RouterG {
        RouterG {
            clauses: r.transitions.iter().map(|c| RouteClauseG {
                judgment: c.judgment.glyph(),
                source: match c.source { Some(t) => t.glyph(), None => R_ANY },
                transform_word: c.transform_word.chars().collect(),
                next: match c.next { Some(t) => t.glyph(), None => M_FIX },
            }).collect(),
        }
    }

    /// The control law: read a clause by grammatical equality on two marks.
    pub fn apply_g(&self, jmark: Mark, smark: Mark) -> Option<&RouteClauseG> {
        self.clauses.iter().find(|c| c.judgment == jmark && (c.source == smark || c.source == M_N))
    }

    /// The router word, reassembled from the mark clauses — the object is still the word.
    pub fn encode(&self) -> String {
        let mut s = String::new();
        for c in &self.clauses {
            s.push('∈');
            s.push(c.judgment);
            s.push(c.source);
            s.push('∈');
            let n = c.transform_word.len();
            for bit in (0..8).rev() { s.push(if (n >> bit) & 1 == 1 { M_T } else { M_F }); }
            s.push('∋');
            for m in &c.transform_word { s.push(*m); }
            s.push(c.next);
            s.push('∋');
        }
        s
    }
}

/// One resident trace step, all marks. (Stage 10 keeps `recognised` a bool in the
/// oracle; here it is already a mark — ⊤ recognised, ⊙ unmatched.)
#[derive(Clone)]
pub struct GStep {
    pub repr: Mark,
    pub judgment: Mark,
    pub recognised: Mark,      // ⊤ recognised · ⊙ no clause matched
    pub next: Mark,
    pub applied_word: Vec<Mark>,
}

fn tag_param(tag: Mark, n: u64) -> u64 {
    match tag { REPR_TAG_SYM => isqrt_u64(n) + 1, REPR_TAG_RES => 1, _ => 0 }
}

/// The mark-keyed control law, mirroring `router_object::run` exactly: loop to
/// `max_steps` unless a closure fires; advance the tag only when the judgment is not N
/// and the clause hands a next; a missing clause is a mark step (⊙), never a bool.
pub fn run_mark(router: &RouterG, n: u64, max_steps: usize) -> (Option<(u64, u64)>, Vec<GStep>) {
    let mut tag: Mark = REPR_TAG_SYM;
    let mut traj: Vec<GStep> = Vec::new();
    for _ in 0..max_steps {
        let carrier = repr_value(tag, n, tag_param(tag, n));
        let jmark = judge_g(&carrier).mark0().unwrap_or(M_F);
        let (word, next, recog) = match router.apply_g(jmark, tag) {
            Some(c) => (c.transform_word.clone(), c.next, M_T),
            None => (alloc::vec![M_N], M_FIX, M_N),
        };
        traj.push(GStep { repr: tag, judgment: jmark, recognised: recog, next, applied_word: word });
        if jmark == M_T {
            if let Some((p, q)) = found_factor(tag, n) {
                if p > 1 && q > 1 && p * q == n { return (Some((p, q)), traj); }
            }
        }
        if jmark != M_N {
            if next != M_FIX { tag = next; }
        }
    }
    (None, traj)
}

// ---- Stage 12: the clause is only a word with a reader ----

/// Read the router WORD on demand: scan clause words `∈ J S ∈ <8 len> ∋ <payload:L> N ∋`
/// and return the first that matches (jmark, smark), with ⊙ in the source slot as the
/// wildcard. No `RouteClauseG` is materialised — the tape is the table.
pub fn match_word(word: &[Mark], jmark: Mark, smark: Mark) -> Option<(Vec<Mark>, Mark)> {
    let mut i = 0usize;
    while i < word.len() {
        if word[i] != '∈' { return None; }
        i += 1;
        let j = *word.get(i)?; i += 1;
        let s = *word.get(i)?; i += 1;
        if *word.get(i)? != '∈' { return None; }
        i += 1;
        let mut len = 0usize;
        for _ in 0..8 {
            let b = *word.get(i)?; i += 1;
            len = (len << 1) | match b { '⊤' => 1usize, '⊥' => 0usize, _ => return None };
        }
        if *word.get(i)? != '∋' { return None; }
        i += 1;
        let mut payload = Vec::with_capacity(len);
        for _ in 0..len { payload.push(*word.get(i)?); i += 1; }
        let nx = *word.get(i)?; i += 1;
        if *word.get(i)? != '∋' { return None; }
        i += 1;
        if j == jmark && (s == smark || s == M_N) { return Some((payload, nx)); }
    }
    None
}

/// The control law with the word as the only clause storage — the exact analogue of
/// `run_mark`, reading the tape instead of a clause vector.
pub fn run_word(word: &[Mark], n: u64, max_steps: usize) -> (Option<(u64, u64)>, Vec<GStep>) {
    let mut tag: Mark = REPR_TAG_SYM;
    let mut traj: Vec<GStep> = Vec::new();
    for _ in 0..max_steps {
        let carrier = repr_value(tag, n, tag_param(tag, n));
        let jmark = judge_g(&carrier).mark0().unwrap_or(M_F);
        let (applied, next, recog) = match match_word(word, jmark, tag) {
            Some((p, nx)) => (p, nx, M_T),
            None => (alloc::vec![M_N], M_FIX, M_N),
        };
        traj.push(GStep { repr: tag, judgment: jmark, recognised: recog, next, applied_word: applied });
        if jmark == M_T {
            if let Some((p, q)) = found_factor(tag, n) {
                if p > 1 && q > 1 && p * q == n { return (Some((p, q)), traj); }
            }
        }
        if jmark != M_N {
            if next != M_FIX { tag = next; }
        }
    }
    (None, traj)
}

// ---- word-native trajectory rewrite: mark + trace-word + router-word -> router-word' ----

use crate::router_store::{IStore, execute, OP_SCAN_APPEND};
use crate::trace_word::decode_trace;

/// The rewrite, word-native. No enum step is traversed and `router_object::rewrite` is not
/// called: the trace READER extracts the (judgment, source) marks of each unmatched record,
/// derives the probe, and feeds it into the SAME SCAN-APPEND machinery that repairs the
/// router — read distinction, scan the reference for the matching distinction, capture the
/// balanced clause, commit. `applied_word = ⊙` records contribute nothing (no committed
/// transformation), so no sentinel is decoded. Byte-equal to the enum rewrite by construction.
pub fn rewrite_word(router_word: &str, reference_word: &str, trace_words: &[Vec<Mark>], verdict: Mark) -> String {
    if verdict == M_T { return String::from(router_word); }   // PRESERVE: the represented class closed
    let mut st = IStore::new(router_word, reference_word);
    let op: Vec<Mark> = OP_SCAN_APPEND.chars().collect();
    let mut seen: Vec<(Mark, Mark)> = Vec::new();
    for tw in trace_words {
        if let Some(steps) = decode_trace(tw) {
            for s in &steps {
                if s.recognised == M_N {
                    let probe = (s.judgment, s.repr);
                    if !seen.contains(&probe) {
                        seen.push(probe);
                        execute(&op, &mut st, [probe.0, probe.1]);
                    }
                }
            }
        }
    }
    st.router_word()
}

/// A zero-enum probe read: the first (judgment, source) marks of an unmatched record in a
/// trace word — the two-mark reading the process hands to SCAN-APPEND.
pub fn probe_from_trace(trace_word: &[Mark]) -> Option<(Mark, Mark)> {
    let steps = decode_trace(trace_word)?;
    for s in &steps { if s.recognised == M_N { return Some((s.judgment, s.repr)); } }
    None
}

// ---- Stage 17: the terminal fixed point — a ⊤ closure that HALTS instead of re-entering ----

/// The terminal-fixed-point control law. Identical to `run_word` EXCEPT that a record whose
/// clause hands the terminal marker ⊡ (M_FIX) HALTS the trajectory, instead of leaving the tag
/// unchanged and re-entering. Under `run_word` a closed terminal state re-enters identically until
/// `max_steps` (the n=2 spin, 64 identical records); here the ⊤ closure is a terminal fixed point.
/// The law is still word-native: it reads only the router word, no enum step is traversed.
pub fn run_word_terminal(word: &[Mark], n: u64, max_steps: usize) -> (Option<(u64, u64)>, Vec<GStep>) {
    let mut tag: Mark = REPR_TAG_SYM;
    let mut traj: Vec<GStep> = Vec::new();
    for _ in 0..max_steps {
        let carrier = repr_value(tag, n, tag_param(tag, n));
        let jmark = judge_g(&carrier).mark0().unwrap_or(M_F);
        let (applied, next, recog) = match match_word(word, jmark, tag) {
            Some((p, nx)) => (p, nx, M_T),
            None => (alloc::vec![M_N], M_FIX, M_N),
        };
        traj.push(GStep { repr: tag, judgment: jmark, recognised: recog, next, applied_word: applied });
        if jmark == M_T {
            if let Some((p, q)) = found_factor(tag, n) {
                if p > 1 && q > 1 && p * q == n { return (Some((p, q)), traj); }
            }
        }
        if next == M_FIX { break; }             // ⊡ => terminal fixed point, halt
        if jmark != M_N { tag = next; }
    }
    (None, traj)
}
