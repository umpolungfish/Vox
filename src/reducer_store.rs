//! reducer_store.rs — the RESIDENT REDUCER (Stage 20). After the edit machine (Stages 14–16),
//! the remaining host contribution is the rewrite SCHEDULE: which span, in which order, when to
//! stop. That schedule is now a WORD. The machine's state (cursor, rule, replacement) lives in
//! marks; the word walks it. replay+judge is the machine's SEMANTICS (an op), not the schedule.
//!
//!     REDUCE_WORD = ∈ ∋ ⊤ ≻ ⊡
//!       ∈ : build candidate = edit(tape, cursor, rules[rule], fill)   (the EDIT word, applied)
//!       ∋ : admissible = replay + judge(candidate) under rel           (the SEMANTICS)
//!       ⊤ : if admissible → commit tape ← candidate, restart the scan
//!       ≻ : advance the (rule, span) cursor
//!       ⊡ : every rule × span scanned with no accept → fixed point, halt
//!
//! The RULE SET and their ORDER are data (`Rule { groups, fill }`), not a Rust for-loop over a
//! hardcoded "try DELETE, then FUSE, then INLINE". DELETE/CANCEL/FUSE/INLINE differ only in the
//! data: span + replacement. The machine is: `replace this balanced span by this word`.

use alloc::vec::Vec;
use crate::router_marks::{M_T, GStep};
use crate::trace_word::{decode_trace, encode_step, judge_trace};
use crate::trace_algebra::{replay_state, factor_of};
use crate::tape_delete::{fuse_records, tape_edit};

pub type Mark = char;

/// One rewrite rule: a span length and a replacement FILL.
///   fill ⊙ : replacement empty            (DELETE / CANCEL)
///   fill ⊞ : replacement = composed span   (FUSE / INLINE)
#[derive(Clone, Copy)]
pub struct Rule { pub groups: usize, pub fill: Mark }

pub const FILL_NONE: Mark = '\u{2299}'; // ⊙
pub const FILL_COMPOSE: Mark = '\u{229E}'; // ⊞

pub struct RStore {
    pub tape: Vec<Mark>,
    pub cursor: usize,
    pub rule: usize,
    pub rules: Vec<Rule>,
    pub n: u64,
    pub rel: Mark,               // ⊤ strict (≡s) · ⊥ relaxed (≡c)
    pub candidate: Option<Vec<Mark>>,
    pub admissible: bool,
    pub restart: bool,
    pub accepted: usize,
    pub halted: bool,
}

impl RStore {
    pub fn new(trace: &[Mark], n: u64, rel: Mark, rules: Vec<Rule>) -> RStore {
        RStore { tape: trace.to_vec(), cursor: 0, rule: 0, rules, n, rel,
                 candidate: None, admissible: false, restart: false, accepted: 0, halted: false }
    }
    fn nspans(&self) -> usize { decode_trace(&self.tape).map(|t| t.len()).unwrap_or(0) }

    /// The replacement fill for the span at `cursor` under the current rule.
    fn replacement(&self, groups: usize) -> Option<Vec<Mark>> {
        let fill = self.rules.get(self.rule)?.fill;
        if fill == FILL_COMPOSE {
            let t = decode_trace(&self.tape)?;
            if self.cursor + groups > t.len() { return None; }
            let span: Vec<GStep> = t[self.cursor..self.cursor + groups].to_vec();
            let mut comp = span[0].clone();
            for r in &span[1..] { comp = fuse_records(&comp, r); }
            Some(encode_step(&comp))
        } else {
            Some(Vec::new())
        }
    }

    /// The SEMANTICS (an op, not the schedule): is the candidate equivalent to the tape?
    fn equiv(&self, cand: &[Mark]) -> bool {
        let fo = replay_state(&self.tape).as_ref().and_then(|r| factor_of(r, self.n));
        let fc = replay_state(cand).as_ref().and_then(|r| factor_of(r, self.n));
        if self.rel == M_T {
            let (ro, rc) = match (replay_state(&self.tape), replay_state(cand)) { (Some(a), Some(b)) => (a, b), _ => return false };
            ro == rc && judge_trace(cand) == M_T && fo == fc
        } else {
            judge_trace(cand) == M_T && fo.is_some() && fo == fc
        }
    }
}

/// The resident schedule word.
pub const REDUCE_WORD: &str = "\u{2208}\u{220B}\u{22A4}\u{227B}\u{22A1}"; // ∈ ∋ ⊤ ≻ ⊡

impl RStore {
    fn build_candidate(&mut self) {
        self.candidate = None;
        if self.rule >= self.rules.len() { return; }
        let ns = self.nspans();
        if self.cursor >= ns { return; }
        let groups = self.rules[self.rule].groups;
        if let Some(repl) = self.replacement(groups) {
            self.candidate = tape_edit(&self.tape, self.cursor, groups, repl);
        }
    }
    fn test(&mut self) {
        self.admissible = match &self.candidate { Some(c) => self.equiv(c), None => false };
    }
    fn commit(&mut self) {
        if self.admissible {
            if let Some(c) = self.candidate.take() {
                self.tape = c;
                self.accepted += 1;
                self.restart = true;
            }
        }
    }
    fn advance(&mut self) {
        if self.restart { self.cursor = 0; self.rule = 0; self.restart = false; return; }
        let ns = self.nspans();
        if ns == 0 { self.cursor = 0; self.rule += 1; return; }
        self.cursor += 1;
        if self.cursor >= ns { self.cursor = 0; self.rule += 1; }
    }
}

/// Run the resident schedule word as a ring until the fixed point (`⊡` sets `halted`).
pub fn run(s: &mut RStore) {
    let op: Vec<Mark> = REDUCE_WORD.chars().collect();
    let mut ip = 0usize;
    let mut budget = 1usize << 22;
    while budget > 0 && !s.halted {
        if ip >= op.len() { ip = 0; budget -= 1; continue; }
        match op[ip] {
            '\u{2208}' => s.build_candidate(),               // ∈
            '\u{220B}' => s.test(),                          // ∋
            '\u{22A4}' => s.commit(),                        // ⊤
            '\u{227B}' => s.advance(),                       // ≻
            '\u{22A1}' => if s.rule >= s.rules.len() { s.halted = true; }, // ⊡
            _ => {}
        }
        ip += 1;
        budget -= 1;
    }
}

/// Reduce `trace` by the resident schedule word under rule set `rules` and relation `rel`.
/// Returns (normal form, number of accepted transforms).
pub fn reduce_resident(trace: &[Mark], n: u64, rel: Mark, rules: &[Rule]) -> (Vec<Mark>, usize) {
    let mut s = RStore::new(trace, n, rel, rules.to_vec());
    run(&mut s);
    (s.tape, s.accepted)
}

/// Convenience constructors mirroring the four transformations.
pub fn rule_delete() -> Rule { Rule { groups: 1, fill: FILL_NONE } }
pub fn rule_cancel(k: usize) -> Rule { Rule { groups: k, fill: FILL_NONE } }
pub fn rule_fuse() -> Rule { Rule { groups: 2, fill: FILL_COMPOSE } }
pub fn rule_inline(k: usize) -> Rule { Rule { groups: k, fill: FILL_COMPOSE } }
