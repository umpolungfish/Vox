//! tape_delete.rs — the resident tape EDIT machine. A WORD walks a tape of top-level
//! balanced groups `∈…∋` (records), identifies a span of consecutive groups, and commits
//! the complement with an optional REPLACEMENT spliced at the span boundary.
//!
//!     EDIT_WORD = ∈ ∋ ⊥ ≻ ⊡
//!
//! The machine knows one syntactic fact and nothing else: ∈/∋ nest (exactly what
//! `capture_balanced` assumes). It never names a record part — repr, judgment, recognised,
//! next, applied are bytes it copies through. What to drop (and what to put in its place)
//! is decided by the caller's WORD + data; whether that edit MEANS anything is decided by
//! replay, never by the host.
//!
//!   DELETE = one group, empty replacement        (delete_word)
//!   FUSE   = two groups, replacement = merged     (fuse_word)
//!
//! CONTRACTS
//!     delete_word(trace, i) == delete_record(trace, i)     byte-for-byte (oracle)
//!     fuse_word(trace, i)   == trace with records i,i+1 replaced by their merge

use alloc::vec::Vec;
use crate::router_marks::GStep;
use crate::trace_word::{decode_trace, encode_step};

pub type Mark = char;

/// The resident edit word — five marks, no host logic. The store supplies how many
/// groups the span covers and what (if anything) replaces them.
pub const EDIT_WORD: &str = "\u{2208}\u{220B}\u{22A5}\u{227B}\u{22A1}";
/// DELETE is the one-group/empty-replacement case of the same resident word.
pub const DELETE_WORD: &str = EDIT_WORD;

pub struct TStore {
    pub tape: Vec<Mark>,
    /// Structural view of `tape`: only outer TRACE record boundaries remain
    /// ∈/∋. Inner length framing and applied-word glyphs are opaque bytes.
    /// For non-trace inputs this falls back to the original tape shape.
    pub structure: Vec<Mark>,
    pub cursor: usize,
    pub out: Vec<Mark>,
    pub depth: i32,
    pub seen: usize,           // top-level groups opened so far
    pub target: usize,         // the span begins at the target-th top-level group
    pub groups: usize,         // how many consecutive top-level groups the span covers
    pub skip_remaining: usize,
    pub replacement: Vec<Mark>, // spliced at the span boundary (empty for DELETE)
    pub skipping: bool,
    pub skipped: bool,
    pub emitted: bool,
    pub committed: bool,
}

/// Build the structural view of a TRACE word without materialising `GStep`s.
///
/// RECORD := ∈ repr judgment recognised next ∈ <len8> ∋ <applied:len> ∋
///
/// Only the first and last marks of each record are structural for the tape
/// editor.  The inner ∈/∋ around the length and every ∈/∋ inside applied_word
/// are data because applied_word is length-delimited.
fn trace_structure(trace: &[Mark]) -> Option<Vec<Mark>> {
    let mut structure = Vec::with_capacity(trace.len());
    structure.resize(trace.len(), '\0');

    let mut i = 0usize;
    while i < trace.len() {
        let start = i;
        if *trace.get(i)? != '∈' { return None; }
        structure[start] = '∈';
        i += 1;

        // repr, judgment, recognised, next
        i = i.checked_add(4)?;
        if i > trace.len() { return None; }

        // inner length frame
        if *trace.get(i)? != '∈' { return None; }
        i += 1;

        let mut payload_len = 0usize;
        for _ in 0..8 {
            let bit = *trace.get(i)?;
            i += 1;
            payload_len = (payload_len << 1) | match bit {
                '⊤' => 1usize,
                '⊥' => 0usize,
                _ => return None,
            };
        }

        if *trace.get(i)? != '∋' { return None; }
        i += 1;

        i = i.checked_add(payload_len)?;
        if i >= trace.len() { return None; }

        // outer record close
        if trace[i] != '∋' { return None; }
        structure[i] = '∋';
        i += 1;
    }

    Some(structure)
}

impl TStore {
    pub fn new(trace: &[Mark], target: usize, groups: usize, replacement: Vec<Mark>) -> TStore {
        let structure = trace_structure(trace).unwrap_or_else(|| trace.to_vec());
        TStore {
            tape: trace.to_vec(),
            structure,
            cursor: 0,
            out: Vec::new(),
            depth: 0,
            seen: 0,
            target,
            groups,
            skip_remaining: 0,
            replacement,
            skipping: false,
            skipped: false,
            emitted: false,
            committed: false,
        }
    }
    fn at(&self, c: Mark) -> bool {
        self.cursor < self.structure.len() && self.structure[self.cursor] == c
    }
    fn cur(&self) -> Option<Mark> { self.tape.get(self.cursor).copied() }
}

/// Run the resident EDIT word as a ring until it commits.
fn run(s: &mut TStore) {
    let op: Vec<Mark> = EDIT_WORD.chars().collect();
    let mut ip = 0usize;
    let mut budget = 1usize << 20;
    while budget > 0 && !s.committed {
        if ip >= op.len() { ip = 0; budget -= 1; continue; }
        match op[ip] {
            // ∈ — a group open. A depth-0 open is a span boundary.
            '\u{2208}' => {
                if s.at('\u{2208}') {
                    if s.depth == 0 {
                        if s.seen == s.target {
                            s.skipping = true; s.skipped = true; s.skip_remaining = s.groups;
                        }
                        s.depth = 1;
                        s.seen += 1;
                    } else {
                        s.depth += 1;
                    }
                    if !s.skipping { s.out.push('\u{2208}'); }
                }
            }
            // ∋ — a group close. The span ends at the `groups`-th depth-0 close after entry.
            '\u{220B}' => {
                if s.at('\u{220B}') {
                    let was_skipping = s.skipping;
                    s.depth -= 1;
                    if s.depth <= 0 {
                        s.depth = 0;
                        if s.skipping {
                            s.skip_remaining -= 1;
                            if s.skip_remaining == 0 {
                                s.skipping = false;
                                if !s.emitted {
                                    s.out.extend_from_slice(&s.replacement);
                                    s.emitted = true;
                                }
                            }
                        }
                    }
                    if !was_skipping { s.out.push('\u{220B}'); }
                }
            }
            // ⊥ — an ordinary byte (length bits, applied-word bytes) → copy unless skipping.
            '\u{22A5}' => {
                if let Some(c) = s.cur() {
                    // Copy every non-structural byte verbatim.  In particular,
                    // ∈/∋ inside the length frame or applied_word are data.
                    if !s.at('\u{2208}') && !s.at('\u{220B}') && !s.skipping {
                        s.out.push(c);
                    }
                }
            }
            // ≻ — advance.
            '\u{227B}' => { if s.cursor < s.tape.len() { s.cursor += 1; } }
            // ⊡ — commit once the whole tape has been walked.
            '\u{22A1}' => { if s.cursor >= s.tape.len() { s.committed = true; } }
            _ => {}
        }
        ip += 1;
        budget -= 1;
    }
}

/// The generic edit: identify a span of `groups` consecutive top-level groups beginning
/// at group `target`, and commit the complement with `replacement` spliced at the span
/// boundary. None iff the span does not fit.
pub fn tape_edit(trace: &[Mark], target: usize, groups: usize, replacement: Vec<Mark>) -> Option<Vec<Mark>> {
    let mut s = TStore::new(trace, target, groups, replacement);
    run(&mut s);
    if !s.skipped { return None; }
    Some(s.out)
}

/// DELETE record `target` — one group, empty replacement. Byte-identical to
/// `trace_algebra::delete_record`; the Rust function is now only an oracle.
pub fn delete_word(trace: &[Mark], target: usize) -> Option<Vec<Mark>> {
    tape_edit(trace, target, 1, Vec::new())
}

/// The FUSE PROPOSAL: merge two adjacent records into one — start where `a` started,
/// carry both applied words, report the later verdict, end where `b` ended. Whether the
/// merge MEANS anything is decided by replay, not here.
pub fn fuse_records(a: &GStep, b: &GStep) -> GStep {
    let mut aw = a.applied_word.clone();
    aw.extend_from_slice(&b.applied_word);
    GStep { repr: a.repr, judgment: b.judgment, recognised: b.recognised, next: b.next, applied_word: aw }
}

/// FUSE records `i`,`i+1` — the same resident word, two groups, the merged record spliced
/// in. None iff `i+1` is out of range.
pub fn fuse_word(trace: &[Mark], i: usize) -> Option<Vec<Mark>> {
    let t = decode_trace(trace)?;
    if i + 1 >= t.len() { return None; }
    let fused = encode_step(&fuse_records(&t[i], &t[i + 1]));
    tape_edit(trace, i, 2, fused)
}

/// CANCEL k consecutive records `i..i+k` — the same word, an empty replacement. k=1 is
/// DELETE; k=2 drops an adjacent pair whose combined replay effect disappears.
pub fn cancel_word(trace: &[Mark], i: usize, k: usize) -> Option<Vec<Mark>> {
    if k == 0 { return None; }
    tape_edit(trace, i, k, Vec::new())
}

/// INLINE k consecutive records into one whose applied word is the composition of all
/// k. k=2 is FUSE. The replacement is the composed record; replay is the semantics.
pub fn inline_word(trace: &[Mark], i: usize, k: usize) -> Option<Vec<Mark>> {
    if k < 2 { return None; }
    let t = decode_trace(trace)?;
    if i + k > t.len() { return None; }
    let mut comp = t[i].clone();
    for j in i + 1..i + k { comp = fuse_records(&comp, &t[j]); }
    tape_edit(trace, i, k, encode_step(&comp))
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::router_marks::{M_B, M_T, M_FIX};
    use crate::trace_algebra::delete_record;
    use crate::trace_word::encode_trace;

    #[test]
    fn delete_word_treats_length_delimited_payload_brackets_as_data() {
        // The first applied word contains an unmatched ∈.  That glyph is data,
        // not a trace-record boundary, because applied_word is length-delimited.
        let steps = [
            GStep {
                repr: '⊢',
                judgment: M_B,
                recognised: M_T,
                next: '⊣',
                applied_word: "⊢∈≻⊤⊣".chars().collect(),
            },
            GStep {
                repr: '⊣',
                judgment: M_B,
                recognised: M_T,
                next: '⋈',
                applied_word: "⊢∈≻⊤∈≺∋∋⊙⊡⊣".chars().collect(),
            },
            GStep {
                repr: '⋈',
                judgment: M_T,
                recognised: M_T,
                next: M_FIX,
                applied_word: "⊢⊙⊡⊣".chars().collect(),
            },
        ];
        let trace = encode_trace(&steps);

        for i in 0..steps.len() {
            assert_eq!(delete_word(&trace, i), delete_record(&trace, i));
        }
    }
}
