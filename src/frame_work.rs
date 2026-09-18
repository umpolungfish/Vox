//! frame_work.rs — the resident FRAME_WORK reconciliation morphism (Stage 26).
//!
//! The host transformer `frame_work.py` moves into the grammar. The machine consumes
//! marks and exposes ONLY balanced structure and positions — it never names an object
//! (no "REDUCE_WORD", no "EDIT_WORD"). One structural rewrite:
//!
//!     \u{2208} \u{220B} tau   ->   \u{2208} tau \u{220B}
//!
//! where tau is the work span following the first ADJACENT \u{2208}\u{220B} dyad, up to
//! the enclosing TANCH \u{22A3} (kept outside the frame) or the end of the word.
//!
//! FRAME_WORK proposes the rewrite; Vox decides whether a closing type resulted. The
//! transformer does NOT know N->T, nor whether tau is \u{22A4}\u{227B}\u{22A1}, anything.

use alloc::vec::Vec;

pub type Mark = char;

pub const FSPLIT: Mark = '\u{2208}'; // in
pub const FFUSE: Mark = '\u{220B}';  // ni
pub const TANCH: Mark = '\u{22A3}';  // tanch

/// Position of the first ADJACENT FSPLIT/FFUSE dyad, if any.
pub fn adjacent_dyad(marks: &[Mark]) -> Option<usize> {
    let mut i = 0;
    while i + 1 < marks.len() {
        if marks[i] == FSPLIT && marks[i + 1] == FFUSE {
            return Some(i);
        }
        i += 1;
    }
    None
}

/// The work span tau following the first adjacent dyad, up to the enclosing TANCH.
pub fn captured_span(marks: &[Mark]) -> Vec<Mark> {
    match adjacent_dyad(marks) {
        None => Vec::new(),
        Some(i) => {
            let after = i + 2;
            let end = marks[after..]
                .iter()
                .position(|&m| m == TANCH)
                .map(|k| after + k)
                .unwrap_or(marks.len());
            marks[after..end].to_vec()
        }
    }
}

/// FRAME_WORK: relocate the fuse so the frame ENCLOSES the work span tau.
/// Proposal only — no object knowledge, no N->T baked in. Identity when no dyad exists.
pub fn frame_work(marks: &[Mark]) -> Vec<Mark> {
    let i = match adjacent_dyad(marks) {
        Some(i) => i,
        None => return marks.to_vec(),
    };
    let after = i + 2;
    let end = marks[after..]
        .iter()
        .position(|&m| m == TANCH)
        .map(|k| after + k)
        .unwrap_or(marks.len());
    let mut out: Vec<Mark> = Vec::with_capacity(marks.len());
    out.extend_from_slice(&marks[..i]);
    out.push(FSPLIT);
    out.extend_from_slice(&marks[after..end]);
    out.push(FFUSE);
    out.extend_from_slice(&marks[end..]);
    out
}
