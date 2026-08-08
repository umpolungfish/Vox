//! The genetics lane: a coding sequence read as a word in the twelve.
//!
//! Not an analogy. The twelve operations and the twelve axes are one alphabet,
//! read as an operation or as an axis according to where the glyph stands. The
//! chain from nucleotide to glyph is proved in Lean and parsed into
//! `genetic_table.rs` by its generator: G is B because guanine wobble-pairs
//! with both C and U, C is T because it pairs only with G, A is F, U is N;
//! codons carry to amino acids by the genetic code; and exactly twelve amino
//! acids are promoted, bijecting the twelve axes.
//!
//! So a gene is already a word, and the same SIXTEEN_3 engine that verdicts x86
//! verdicts the transcript.

use alloc::string::String;
use alloc::vec::Vec;
use crate::genetic_table::{AA_GLYPH, CODON, NUC_B4};

/// One promoted codon, as read.
pub struct Read { pub codon: String, pub aa: &'static str, pub glyph: char, pub axis: &'static str }

/// What a lifted sequence says.
pub struct Transcript {
    pub word: Vec<char>,
    pub reading: Vec<Read>,
    /// the stop codon that ended the reading frame, if one did
    pub stopped: Option<&'static str>,
    /// the frame start, as an offset into the cleaned sequence
    pub start: usize,
    /// true when no AUG was found and the frame starts at zero
    pub implicit_frame: bool,
}

/// Belnap value of a nucleotide.
pub fn nuc_b4(c: char) -> Option<char> {
    NUC_B4.iter().find(|(n, _)| *n == c).map(|(_, b)| *b)
}

fn codon_meaning(c: &str) -> Option<(&'static str, &'static str)> {
    CODON.iter().find(|(k, _, _)| *k == c).map(|(_, kind, val)| (*kind, *val))
}

fn aa_glyph(aa: &str) -> Option<(char, &'static str)> {
    AA_GLYPH.iter().find(|(a, _, _, _)| *a == aa).map(|(_, g, ax, _)| (*g, *ax))
}

/// An RNA or DNA sequence to a word in the twelve. Reads from the first AUG in
/// frame, stops at a stop codon, and emits a glyph only where the codon names a
/// promoted amino acid; the ground layer activates no axis and is silent, which
/// is a fact of the code and not a gap in the lift.
pub fn lift_rna(seq: &str) -> Transcript {
    let clean: Vec<char> = seq.chars()
        .map(|c| c.to_ascii_uppercase())
        .filter(|c| matches!(c, 'A' | 'C' | 'G' | 'T' | 'U'))
        .map(|c| if c == 'T' { 'U' } else { c })
        .collect();

    let mut start = 0usize;
    let mut implicit_frame = true;
    for k in 0..clean.len().saturating_sub(2) {
        if clean[k] == 'A' && clean[k + 1] == 'U' && clean[k + 2] == 'G' {
            start = k; implicit_frame = false; break;
        }
    }

    let mut t = Transcript { word: Vec::new(), reading: Vec::new(), stopped: None, start, implicit_frame };
    let mut k = start;
    while k + 3 <= clean.len() {
        let codon: String = clean[k..k + 3].iter().collect();
        if let Some((kind, val)) = codon_meaning(&codon) {
            if kind == "stop" { t.stopped = Some(val); break; }
            if let Some((glyph, axis)) = aa_glyph(val) {
                t.word.push(glyph);
                t.reading.push(Read { codon, aa: val, glyph, axis });
            }
        }
        k += 3;
    }
    t
}
