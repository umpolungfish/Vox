//! factor_extract.rs — passive factor extraction from a factor-bearing VOX trace.
//!
//! This module DOES NOT factor N.  It accepts an object that already carries a
//! verified factor witness together with the successful trace that produced it.
//! The resident membrane walks REDUCE_WORD under the frozen relaxed relation ≡c,
//! removes routing scaffold when closure+factor are preserved, and emits the
//! carried witness from the terminal carrier.
//!
//! Stage-24/25 split is explicit:
//!     WALK = ∈∋⊤≻⊡       (resident store execution)
//!     TYPE = ∈⊤≻⊡∋       (structurally closing form)
//!
//! No call in this module reaches `found_factor`, `morphism_factor`, rho,
//! Fermat, a sieve, primality search, order finding, or candidate enumeration.

use alloc::string::String;
use alloc::vec::Vec;

use crate::router_marks::{GStep, M_T, M_F, M_FIX};
use crate::tape_delete::delete_word;
use crate::trace_algebra::{
    admissible_relaxed_with_witness,
    closure_state,
    relaxed_equivalent_with_witness,
    witness_valid,
};
use crate::trace_word::{decode_trace, encode_trace, is_terminal, judge_trace};

pub type Mark = char;

/// What the resident reducer actually walks.
pub const EXTRACT_WALK: &str = "∈∋⊤≻⊡";
/// What the same operation verifies structurally after FRAME_WORK reconciliation.
pub const EXTRACT_TYPE: &str = "∈⊤≻⊡∋";

/// A successful factorization result made into data.  The trace is untouched;
/// the factor witness is a separate projection, so trace semantics stay frozen.
#[derive(Clone, PartialEq, Debug)]
pub struct FactorCarrier {
    pub n: u64,
    pub p: u64,
    pub q: u64,
    pub trace: Vec<Mark>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct FactorReadout {
    pub p: u64,
    pub q: u64,
    pub normal_form: Vec<Mark>,
    pub transforms: usize,
}

impl FactorCarrier {
    /// Freeze an already-successful route result into the factor-bearing object.
    /// `found` is consumed as data; this function performs no search.
    pub fn from_run_result(
        n: u64,
        found: Option<(u64, u64)>,
        trajectory: &[GStep],
    ) -> Result<Self, String> {
        let (p, q) = found.ok_or_else(|| String::from("route result carries no factor witness"))?;
        FactorCarrier::new(n, p, q, encode_trace(trajectory))
    }

    pub fn new(n: u64, p: u64, q: u64, trace: Vec<Mark>) -> Result<Self, String> {
        let carrier = FactorCarrier { n, p, q, trace };
        carrier.validate()?;
        Ok(carrier)
    }

    /// Validate the object without searching for a factor.
    pub fn validate(&self) -> Result<(), String> {
        if !witness_valid(self.n, (self.p, self.q)) {
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

    /// Marks-only serialization.  Four length-independent u64 fields are fixed
    /// 64-bit ⊤/⊥ words (MSB first), followed by the trace by explicit count:
    ///
    ///   ⊢ ∈N64∋ ∈p64∋ ∈q64∋ ∈len64∋ <trace:len> ⊣
    pub fn encode(&self) -> Vec<Mark> {
        let mut out = Vec::with_capacity(self.trace.len() + 270);
        out.push('⊢');
        push_field(&mut out, self.n);
        push_field(&mut out, self.p);
        push_field(&mut out, self.q);
        push_field(&mut out, self.trace.len() as u64);
        out.extend_from_slice(&self.trace);
        out.push('⊣');
        out
    }

    pub fn decode(word: &[Mark]) -> Result<Self, String> {
        let mut i = 0usize;
        expect(word, &mut i, '⊢')?;
        let n = read_field(word, &mut i)?;
        let p = read_field(word, &mut i)?;
        let q = read_field(word, &mut i)?;
        let len = read_field(word, &mut i)? as usize;
        let end = i.checked_add(len).ok_or_else(|| String::from("factor carrier trace length overflow"))?;
        if end > word.len() { return Err(String::from("factor carrier trace is truncated")); }
        let trace = word[i..end].to_vec();
        i = end;
        expect(word, &mut i, '⊣')?;
        if i != word.len() { return Err(String::from("trailing marks after factor carrier")); }
        FactorCarrier::new(n, p, q, trace)
    }
}

fn push_field(out: &mut Vec<Mark>, value: u64) {
    out.push('∈');
    for bit in (0..64).rev() {
        out.push(if (value >> bit) & 1 == 1 { M_T } else { M_F });
    }
    out.push('∋');
}

fn read_field(word: &[Mark], i: &mut usize) -> Result<u64, String> {
    expect(word, i, '∈')?;
    let mut value = 0u64;
    for _ in 0..64 {
        let mark = *word.get(*i).ok_or_else(|| String::from("truncated factor carrier field"))?;
        *i += 1;
        value = value.checked_mul(2).ok_or_else(|| String::from("factor carrier field overflow"))?;
        match mark {
            M_T => value = value.checked_add(1).ok_or_else(|| String::from("factor carrier field overflow"))?,
            M_F => {}
            _ => return Err(String::from("factor carrier numeric field is not ⊤/⊥")),
        }
    }
    expect(word, i, '∋')?;
    Ok(value)
}

fn expect(word: &[Mark], i: &mut usize, mark: Mark) -> Result<(), String> {
    if word.get(*i).copied() != Some(mark) {
        return Err(String::from("malformed factor carrier framing"));
    }
    *i += 1;
    Ok(())
}

/// Resident store for EXTRACT_WALK.  The factor witness never changes; edits
/// affect only the trace projection.
struct ExtractStore {
    carrier: FactorCarrier,
    cursor: usize,
    candidate: Option<Vec<Mark>>,
    admissible: bool,
    committed: bool,
    transforms: usize,
    halted: bool,
}

impl ExtractStore {
    fn new(carrier: FactorCarrier) -> Self {
        ExtractStore {
            carrier,
            cursor: 0,
            candidate: None,
            admissible: false,
            committed: false,
            transforms: 0,
            halted: false,
        }
    }

    fn record_count(&self) -> usize {
        decode_trace(&self.carrier.trace).map(|t| t.len()).unwrap_or(0)
    }

    fn dispatch(&mut self, mark: Mark) {
        match mark {
            // ∈ — build the next candidate using the resident DELETE word.
            '∈' => {
                self.candidate = if self.cursor < self.record_count() {
                    delete_word(&self.carrier.trace, self.cursor)
                } else {
                    None
                };
                self.admissible = false;
                self.committed = false;
            }
            // ∋ — replay/judge under ≡c; the factor is the carried witness.
            '∋' => {
                self.admissible = self.candidate.as_ref().map(|cand| {
                    admissible_relaxed_with_witness(
                        &self.carrier.trace,
                        cand,
                        self.carrier.n,
                        (self.carrier.p, self.carrier.q),
                    )
                }).unwrap_or(false);
            }
            // ⊤ — accept: commit the shorter trace and restart the scan.
            '⊤' => {
                if self.admissible {
                    if let Some(cand) = self.candidate.take() {
                        self.carrier.trace = cand;
                        self.transforms += 1;
                        self.cursor = 0;
                        self.committed = true;
                    }
                }
            }
            // ≻ — advance only when this pass did not just commit.
            '≻' => {
                if !self.committed { self.cursor = self.cursor.saturating_add(1); }
            }
            // ⊡ — no remaining candidate means the relaxed normal form is fixed.
            '⊡' => {
                if !self.committed && self.cursor >= self.record_count() {
                    self.halted = true;
                }
                self.candidate = None;
                self.admissible = false;
                self.committed = false;
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

/// Extract the already-carried factor.  The only arithmetic performed is the
/// product verification in `FactorCarrier::validate`.
pub fn extract(carrier: &FactorCarrier) -> Result<FactorReadout, String> {
    carrier.validate()?;
    let original = carrier.clone();
    let mut store = ExtractStore::new(carrier.clone());
    store.run();

    if !relaxed_equivalent_with_witness(
        &original.trace,
        (original.p, original.q),
        &store.carrier.trace,
        (store.carrier.p, store.carrier.q),
        original.n,
    ) {
        return Err(String::from("extraction membrane changed closure or factor witness"));
    }

    let steps = decode_trace(&store.carrier.trace)
        .ok_or_else(|| String::from("extraction membrane produced malformed normal form"))?;
    let last = steps.last().ok_or_else(|| String::from("extraction membrane produced empty normal form"))?;
    if judge_trace(&store.carrier.trace) != M_T || !is_terminal(last) {
        return Err(String::from("extraction membrane did not finish on a terminal T carrier"));
    }

    let (p, q) = if store.carrier.p <= store.carrier.q {
        (store.carrier.p, store.carrier.q)
    } else {
        (store.carrier.q, store.carrier.p)
    };
    Ok(FactorReadout { p, q, normal_form: store.carrier.trace, transforms: store.transforms })
}

pub fn extract_word(word: &[Mark]) -> Result<FactorReadout, String> {
    let carrier = FactorCarrier::decode(word)?;
    extract(&carrier)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::router_marks::M_B;

    fn big_trace() -> Vec<Mark> {
        // Shape from the measured 3-record successful trajectory:
        // B@Symmetric -> B@Residue -> T@Multiplicative/FIX.
        let steps = [
            GStep { repr: '⊢', judgment: M_B, recognised: M_T, next: '⊣', applied_word: "⊢∈≻⊤⊣".chars().collect() },
            GStep { repr: '⊣', judgment: M_B, recognised: M_T, next: '⋈', applied_word: "⊢∈≻⊤∈≺∋∋⊙⊡⊣".chars().collect() },
            GStep { repr: '⋈', judgment: M_T, recognised: M_T, next: M_FIX, applied_word: "⊢⊙⊡⊣".chars().collect() },
        ];
        encode_trace(&steps)
    }

    #[test]
    fn carrier_round_trips_as_marks() {
        let c = FactorCarrier::new(106_545_994_355_809, 11_524_607, 9_245_087, big_trace()).unwrap();
        assert_eq!(FactorCarrier::decode(&c.encode()).unwrap(), c);
    }

    #[test]
    fn extractor_collapses_scaffold_without_factoring() {
        let c = FactorCarrier::new(106_545_994_355_809, 11_524_607, 9_245_087, big_trace()).unwrap();
        let r = extract(&c).unwrap();
        assert_eq!((r.p, r.q), (9_245_087, 11_524_607));
        assert_eq!(decode_trace(&r.normal_form).unwrap().len(), 1);
        assert_eq!(r.transforms, 2);
    }

    #[test]
    fn extractor_rejects_a_witness_that_does_not_reconstruct_n() {
        assert!(FactorCarrier::new(8051, 83, 96, big_trace()).is_err());
    }
}
