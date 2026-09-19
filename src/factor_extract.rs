//! factor_extract.rs — passive factor extraction from a factor-bearing VOX trace.
//!
//! The carrier is arbitrary-width and marks-native: N, p and q are IMASM numeral
//! tapes, never machine integers.  Extraction is iterated self-entry.  One resident
//! EXTRACT_WALK generation may remove one relaxed-equivalent routing record; the
//! resulting factor-bearing carrier then re-enters as the next object.  Re-entry
//! stops only when another generation draws no admissible distinction.
//!
//! This module DOES NOT factor N.  It verifies the already-carried witness by
//! tape multiplication, preserves it under ≡c, and reduces only the trace projection.

use alloc::string::String;
use alloc::vec::Vec;
use core::cmp::Ordering;

use crate::morphism_factor::{cmp as tape_cmp, dec_of, trim};
use crate::router_marks::{GStep, M_T, M_FIX};
use crate::tape_delete::delete_word;
use crate::trace_algebra::{
    admissible_relaxed_with_tape_witness,
    closure_state,
    relaxed_equivalent_with_tape_witness,
    witness_valid_tape,
};
use crate::trace_word::{decode_trace, encode_trace, is_terminal, judge_trace};
use crate::vox::{EVALF, EVALT};

pub type Mark = char;
pub type Tape = Vec<Mark>;

/// What one self-entry generation actually walks.
pub const EXTRACT_WALK: &str = "∈∋⊤≻⊡";
/// Structural type after FRAME_WORK reconciliation.
pub const EXTRACT_TYPE: &str = "∈⊤≻⊡∋";

/// Arbitrary-width factor-bearing object.  The numeral tapes are LSB-first in
/// the same representation used by `morphism_factor` and `perfect_membrane`.
#[derive(Clone, PartialEq, Debug)]
pub struct FactorCarrier {
    pub n: Tape,
    pub p: Tape,
    pub q: Tape,
    pub trace: Vec<Mark>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct ReentryGeneration {
    pub generation: usize,
    pub records_before: usize,
    pub records_after: usize,
    pub changed: bool,
}

#[derive(Clone, PartialEq, Debug)]
pub struct FactorReadout {
    pub p: Tape,
    pub q: Tape,
    pub normal_form: Vec<Mark>,
    pub transforms: usize,
    pub generations: Vec<ReentryGeneration>,
}

impl FactorCarrier {
    /// Freeze already-carried arbitrary-width witness tapes beside a successful
    /// production trajectory.  No factor is searched for here.
    pub fn from_trace(
        n: &[Mark],
        p: &[Mark],
        q: &[Mark],
        trajectory: &[GStep],
    ) -> Result<Self, String> {
        Self::new(n.to_vec(), p.to_vec(), q.to_vec(), encode_trace(trajectory))
    }

    pub fn new(n: Tape, p: Tape, q: Tape, trace: Vec<Mark>) -> Result<Self, String> {
        let carrier = FactorCarrier {
            n: trim(n),
            p: trim(p),
            q: trim(q),
            trace,
        };
        carrier.validate()?;
        Ok(carrier)
    }

    /// Validate entirely on tapes: p*q == N, then trace T/FIX closure.
    pub fn validate(&self) -> Result<(), String> {
        if !witness_valid_tape(&self.n, &self.p, &self.q) {
            return Err(String::from("factor carrier witness does not reconstruct N"));
        }
        if judge_trace(&self.trace) != M_T {
            return Err(String::from("factor carrier trace is not a closing T trace"));
        }
        let steps = decode_trace(&self.trace)
            .ok_or_else(|| String::from("factor carrier trace is malformed"))?;
        let last = steps.last().ok_or_else(|| String::from("factor carrier trace is empty"))?;
        if !is_terminal(last) || last.next != M_FIX {
            return Err(String::from("factor carrier does not end at terminal FIX"));
        }
        let closure = closure_state(&self.trace)
            .ok_or_else(|| String::from("factor carrier has no closure projection"))?;
        if !closure.closed {
            return Err(String::from("factor carrier replay is not closed"));
        }
        Ok(())
    }

    /// Marks-only serialization with no numeric width field:
    ///
    /// ```text
    ///   ⊢ ∈ <N tape> ∋ ∈ <p tape> ∋ ∈ <q tape> ∋ <trace> ⊣
    /// ```
    ///
    /// Numeral fields contain only EVALT/EVALF, so ∈/∋ self-delimit them.  The
    /// remainder before the final anchor is the trace, whose own applied payloads
    /// are already length-delimited by `trace_word`.
    pub fn encode(&self) -> Vec<Mark> {
        let mut out = Vec::with_capacity(
            self.n.len() + self.p.len() + self.q.len() + self.trace.len() + 8,
        );
        out.push('⊢');
        push_tape_field(&mut out, &self.n);
        push_tape_field(&mut out, &self.p);
        push_tape_field(&mut out, &self.q);
        out.extend_from_slice(&self.trace);
        out.push('⊣');
        out
    }

    pub fn decode(word: &[Mark]) -> Result<Self, String> {
        if word.len() < 2 || word[0] != '⊢' || *word.last().unwrap() != '⊣' {
            return Err(String::from("malformed factor carrier framing"));
        }
        let mut i = 1usize;
        let n = read_tape_field(word, &mut i)?;
        let p = read_tape_field(word, &mut i)?;
        let q = read_tape_field(word, &mut i)?;
        if i >= word.len() - 1 {
            return Err(String::from("factor carrier trace is empty"));
        }
        let trace = word[i..word.len() - 1].to_vec();
        Self::new(n, p, q, trace)
    }

    pub fn n_decimal(&self) -> String { dec_of(&self.n) }
    pub fn p_decimal(&self) -> String { dec_of(&self.p) }
    pub fn q_decimal(&self) -> String { dec_of(&self.q) }
}

fn push_tape_field(out: &mut Vec<Mark>, tape: &[Mark]) {
    out.push('∈');
    out.extend_from_slice(tape);
    out.push('∋');
}

fn read_tape_field(word: &[Mark], i: &mut usize) -> Result<Tape, String> {
    if word.get(*i).copied() != Some('∈') {
        return Err(String::from("malformed factor carrier tape field"));
    }
    *i += 1;
    let mut tape = Vec::new();
    loop {
        let mark = *word.get(*i).ok_or_else(|| String::from("truncated factor carrier tape field"))?;
        *i += 1;
        if mark == '∋' { break; }
        if mark != EVALT && mark != EVALF {
            return Err(String::from("factor carrier numeral tape contains a non-numeral mark"));
        }
        tape.push(mark);
    }
    if tape.is_empty() {
        return Err(String::from("factor carrier numeral tape is empty"));
    }
    Ok(trim(tape))
}

/// Resident state for exactly one self-entry generation.  It scans deletion
/// candidates in EXTRACT_WALK order and halts immediately after the first
/// admissible commit.  If the whole trace is scanned with no commit, this object
/// is the relaxed fixed point and re-entry halts unchanged.
struct ReentryStore {
    carrier: FactorCarrier,
    cursor: usize,
    candidate: Option<Vec<Mark>>,
    admissible: bool,
    changed: bool,
    halted: bool,
}

impl ReentryStore {
    fn new(carrier: FactorCarrier) -> Self {
        Self {
            carrier,
            cursor: 0,
            candidate: None,
            admissible: false,
            changed: false,
            halted: false,
        }
    }

    fn record_count(&self) -> usize {
        decode_trace(&self.carrier.trace).map(|t| t.len()).unwrap_or(0)
    }

    fn dispatch(&mut self, mark: Mark) {
        match mark {
            // ∈ — expose one possible routing distinction.
            '∈' => {
                self.candidate = if self.cursor < self.record_count() {
                    delete_word(&self.carrier.trace, self.cursor)
                } else {
                    None
                };
                self.admissible = false;
            }
            // ∋ — close the candidate under the arbitrary-width frozen ≡c relation.
            '∋' => {
                self.admissible = self.candidate.as_ref().map(|cand| {
                    admissible_relaxed_with_tape_witness(
                        &self.carrier.trace,
                        cand,
                        &self.carrier.n,
                        &self.carrier.p,
                        &self.carrier.q,
                    )
                }).unwrap_or(false);
            }
            // ⊤ — commit one distinction.  The resulting carrier is the NEXT object.
            '⊤' => {
                if self.admissible {
                    if let Some(cand) = self.candidate.take() {
                        self.carrier.trace = cand;
                        self.changed = true;
                        self.halted = true;
                    }
                }
            }
            // ≻ — move to the next possible distinction only if no commit occurred.
            '≻' => {
                if !self.halted { self.cursor = self.cursor.saturating_add(1); }
            }
            // ⊡ — no remaining distinction means this carrier is its own next object.
            '⊡' => {
                if self.cursor >= self.record_count() {
                    self.halted = true;
                }
                self.candidate = None;
                self.admissible = false;
            }
            _ => {}
        }
    }

    fn run(&mut self) {
        while !self.halted {
            for mark in EXTRACT_WALK.chars() {
                self.dispatch(mark);
                if self.halted { break; }
            }
        }
    }
}

/// One explicit self-entry: C_k -> C_{k+1}.  A changed generation strictly
/// shortens the trace while keeping the arbitrary-width factor witness fixed.
pub fn reenter_once(carrier: &FactorCarrier) -> Result<(FactorCarrier, bool), String> {
    carrier.validate()?;
    let mut store = ReentryStore::new(carrier.clone());
    store.run();
    if store.changed {
        if !relaxed_equivalent_with_tape_witness(
            &carrier.trace,
            (&carrier.p, &carrier.q),
            &store.carrier.trace,
            (&store.carrier.p, &store.carrier.q),
            &carrier.n,
        ) {
            return Err(String::from("self-entry changed closure or carried factor witness"));
        }
    }
    Ok((store.carrier, store.changed))
}

/// Iterate self-entry until the carrier re-enters as itself.  Termination does
/// not rely on a numeric generation cap: every changed generation removes one
/// record, so the trace length is a strict descent measure.
pub fn extract(carrier: &FactorCarrier) -> Result<FactorReadout, String> {
    carrier.validate()?;
    let original = carrier.clone();
    let mut current = carrier.clone();
    let mut generations = Vec::new();
    let mut generation = 0usize;

    loop {
        let before = decode_trace(&current.trace)
            .ok_or_else(|| String::from("self-entry carrier trace became malformed"))?
            .len();
        let (next, changed) = reenter_once(&current)?;
        let after = decode_trace(&next.trace)
            .ok_or_else(|| String::from("self-entry produced malformed trace"))?
            .len();

        if changed && after >= before {
            return Err(String::from("self-entry did not strictly reduce the trace"));
        }

        generations.push(ReentryGeneration {
            generation,
            records_before: before,
            records_after: after,
            changed,
        });
        generation = generation.saturating_add(1);
        current = next;
        if !changed { break; }
    }

    if !relaxed_equivalent_with_tape_witness(
        &original.trace,
        (&original.p, &original.q),
        &current.trace,
        (&current.p, &current.q),
        &original.n,
    ) {
        return Err(String::from("extraction membrane changed closure or factor witness"));
    }

    let steps = decode_trace(&current.trace)
        .ok_or_else(|| String::from("extraction membrane produced malformed normal form"))?;
    let last = steps.last().ok_or_else(|| String::from("extraction membrane produced empty normal form"))?;
    if judge_trace(&current.trace) != M_T || !is_terminal(last) {
        return Err(String::from("extraction membrane did not finish on a terminal T carrier"));
    }

    let (p, q) = if tape_cmp(&current.p, &current.q) != Ordering::Greater {
        (current.p.clone(), current.q.clone())
    } else {
        (current.q.clone(), current.p.clone())
    };
    let transforms = generations.iter().filter(|g| g.changed).count();
    Ok(FactorReadout {
        p,
        q,
        normal_form: current.trace,
        transforms,
        generations,
    })
}

pub fn extract_word(word: &[Mark]) -> Result<FactorReadout, String> {
    let carrier = FactorCarrier::decode(word)?;
    extract(&carrier)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::morphism_factor::{dec_of, mul, tape_u64};
    use crate::router_marks::M_B;

    fn big_trace() -> Vec<Mark> {
        let steps = [
            GStep { repr: '⊢', judgment: M_B, recognised: M_T, next: '⊣', applied_word: "⊢∈≻⊤⊣".chars().collect() },
            GStep { repr: '⊣', judgment: M_B, recognised: M_T, next: '⋈', applied_word: "⊢∈≻⊤∈≺∋∋⊙⊡⊣".chars().collect() },
            GStep { repr: '⋈', judgment: M_T, recognised: M_T, next: M_FIX, applied_word: "⊢⊙⊡⊣".chars().collect() },
        ];
        encode_trace(&steps)
    }

    #[test]
    fn carrier_round_trips_as_marks() {
        let p = tape_u64(11_524_607);
        let q = tape_u64(9_245_087);
        let n = mul(&p, &q);
        let c = FactorCarrier::new(n, p, q, big_trace()).unwrap();
        assert_eq!(FactorCarrier::decode(&c.encode()).unwrap(), c);
    }

    #[test]
    fn extractor_self_enters_until_scaffold_is_gone() {
        let p = tape_u64(11_524_607);
        let q = tape_u64(9_245_087);
        let n = mul(&p, &q);
        let c = FactorCarrier::new(n, p, q, big_trace()).unwrap();
        let r = extract(&c).unwrap();
        assert_eq!(dec_of(&r.p), "9245087");
        assert_eq!(dec_of(&r.q), "11524607");
        assert_eq!(decode_trace(&r.normal_form).unwrap().len(), 1);
        assert_eq!(r.transforms, 2);
        assert_eq!(
            r.generations.iter().map(|g| (g.records_before, g.records_after, g.changed)).collect::<Vec<_>>(),
            vec![(3, 2, true), (2, 1, true), (1, 1, false)],
        );
    }

    #[test]
    fn extractor_rejects_a_witness_that_does_not_reconstruct_n() {
        assert!(FactorCarrier::new(
            tape_u64(8051), tape_u64(83), tape_u64(96), big_trace()
        ).is_err());
    }

    #[test]
    fn self_entry_carries_a_factor_object_beyond_the_perfect_membrane_sample_scale() {
        // Two arbitrary-width Mersenne-form carried factors.  Their product is
        // 1128 bits (~340 decimal digits), comfortably beyond the 211-digit
        // perfect-membrane demonstration value, without decoding any operand to
        // a machine integer.  This is an extraction test, not a factoring test.
        let p = vec![EVALF; 521]; // 2^521 - 1 in the LSB-first tape representation
        let q = vec![EVALF; 607]; // 2^607 - 1
        let n = crate::morphism_factor::mul(&p, &q);
        assert!(dec_of(&n).len() > 211);

        let c = FactorCarrier::new(n.clone(), p.clone(), q.clone(), big_trace()).unwrap();
        let encoded = c.encode();
        let decoded = FactorCarrier::decode(&encoded).unwrap();
        assert_eq!(decoded, c);

        let r = extract(&decoded).unwrap();
        assert_eq!(crate::morphism_factor::cmp(&crate::morphism_factor::mul(&r.p, &r.q), &n), Ordering::Equal);
        assert_eq!(r.transforms, 2);
        assert_eq!(r.generations.len(), 3);
        assert_eq!(decode_trace(&r.normal_form).unwrap().len(), 1);
    }
}
