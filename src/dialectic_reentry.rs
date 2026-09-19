//! Dialectic re-entry: the operator imscribes its space, walks that lattice,
//! employs FOUR, and re-imscribes the transformed space.
//!
//! A descent consumes the complete current object.  It does not append another
//! hosted search state.  If a walk does not close a factor, the next object is a
//! new IMASM inscription whose support embodies the distinction already walked.
//! The first installed pair of spaces is the 64-cell short Fermat frontier and
//! the continuation beginning exactly at cell 64.

use alloc::string::String;
use alloc::vec::Vec;
use core::cmp::Ordering;

use crate::factor_extract::{FactorCarrier, Mark, Tape};
use crate::morphism_factor::{add, cmp, divmod, isqrt, mul, sub, tape_u64, trim, zero};
use crate::producer_provenance::{
    SUPPORT_EXTENDED_FERMAT, SUPPORT_PARITY, SUPPORT_PRIMALITY,
    SUPPORT_PRODUCT_BOUNDARY, SUPPORT_SHORT_FRONTIER,
};
use crate::provenance_envelope::{restored_support, LaneSupport, RestoredSupport};
use crate::router_marks::{GStep, M_FIX, M_T};
use crate::trace_word::encode_trace;
use crate::vox::{verdict, EVALF, EVALT, IMSCRIB, TANCH, VINIT};

/// An unresolved inscription deliberately leaves its factor fork open: FOUR=B.
pub const SHORT_FRONTIER_WORD: &str = "⊢⊙∈≻⊤≺⊥⊡⊣";
/// The continuation is a new whole inscription, not the old word plus a cursor.
pub const EXTENDED_FERMAT_WORD: &str = "⊢⊙∈≻⊤⊞≺⊥⊡⊣";

const SHORT_FRONTIER_CLOSED_WORD: &str = "⊢⊙∈≻⊤≺⊥∋⊡⊣";
const EXTENDED_FERMAT_CLOSED_WORD: &str = "⊢⊙∈≻⊤⊞≺⊥∋⊡⊣";
const SUPPORT_BITS: usize = 6;
const FRONTIER_CELLS: usize = 64;
const FERMAT_CELLS: usize = 1_000_000;
const BASE_SUPPORT: LaneSupport = SUPPORT_PARITY | SUPPORT_PRIMALITY;

/// One complete, restartable operator-space object.
///
/// `word` identifies the current inscription. `support` is the distinction
/// embodied by that inscription now; previous objects are not retained.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct DialecticObject {
    pub n: Tape,
    pub word: Vec<Mark>,
    pub support: LaneSupport,
}

/// A terminal contraction: the IMASM inscription has closed around one factor
/// pair and is handed to the passive factor-carrier layer.
#[derive(Clone, PartialEq, Debug)]
pub struct DialecticClosure {
    pub carrier: FactorCarrier,
    pub word: Vec<Mark>,
    pub support: LaneSupport,
    /// Absolute Fermat lattice cell at which the pair closed.
    pub lattice_cell: usize,
}

#[derive(Clone, PartialEq, Debug)]
pub enum Descent {
    Continue(DialecticObject),
    Closed(DialecticClosure),
}

impl DialecticObject {
    /// Initial operator-space inscription. Parity and primality are the ground
    /// support already carried when the factor lattice is entered.
    pub fn new(n: Tape) -> Result<Self, String> {
        let object = Self {
            n: trim(n),
            word: SHORT_FRONTIER_WORD.chars().collect(),
            support: BASE_SUPPORT,
        };
        object.validate()?;
        Ok(object)
    }

    pub fn validate(&self) -> Result<(), String> {
        if cmp(&self.n, &tape_u64(1)) != Ordering::Greater {
            return Err(String::from("dialectic object requires N > 1"));
        }
        let expected_support = if self.word == SHORT_FRONTIER_WORD.chars().collect::<Vec<_>>() {
            BASE_SUPPORT
        } else if self.word == EXTENDED_FERMAT_WORD.chars().collect::<Vec<_>>() {
            BASE_SUPPORT | SUPPORT_SHORT_FRONTIER
        } else {
            return Err(String::from("unknown dialectic inscription"));
        };
        if self.support != expected_support {
            return Err(String::from("dialectic inscription/support mismatch"));
        }
        if !self.word.contains(&IMSCRIB) {
            return Err(String::from("dialectic inscription does not contain IMSCRIB"));
        }
        if verdict(&self.word) != 'B' {
            return Err(String::from("unresolved dialectic inscription is not FOUR=B"));
        }
        Ok(())
    }

    /// Native support currently restored by this whole inscription.
    pub fn envelope(&self) -> RestoredSupport {
        restored_support(&[self.support])
    }

    /// Marks-only persistence of the complete current object:
    ///
    /// `⊢ ⊙ ∈N∋ ∈support[6]∋ <current IMASM word> ⊣`
    pub fn encode(&self) -> Vec<Mark> {
        let mut out = Vec::with_capacity(self.n.len() + self.word.len() + SUPPORT_BITS + 7);
        out.push(VINIT);
        out.push(IMSCRIB);
        push_tape(&mut out, &self.n);
        out.push('∈');
        for bit in 0..SUPPORT_BITS {
            out.push(if (self.support >> bit) & 1 == 1 { EVALF } else { EVALT });
        }
        out.push('∋');
        out.extend_from_slice(&self.word);
        out.push(TANCH);
        out
    }

    pub fn decode(encoded: &[Mark]) -> Result<Self, String> {
        if encoded.len() < 4
            || encoded[0] != VINIT
            || encoded[1] != IMSCRIB
            || *encoded.last().unwrap() != TANCH
        {
            return Err(String::from("malformed dialectic object framing"));
        }
        let mut i = 2usize;
        let n = read_tape(encoded, &mut i)?;
        if encoded.get(i).copied() != Some('∈') {
            return Err(String::from("missing dialectic support field"));
        }
        i += 1;
        let mut support = 0u32;
        for bit in 0..SUPPORT_BITS {
            match encoded.get(i).copied() {
                Some(EVALT) => {}
                Some(EVALF) => support |= 1u32 << bit,
                _ => return Err(String::from("malformed dialectic support mark")),
            }
            i += 1;
        }
        if encoded.get(i).copied() != Some('∋') {
            return Err(String::from("unterminated dialectic support field"));
        }
        i += 1;
        if i >= encoded.len() - 1 {
            return Err(String::from("dialectic inscription is empty"));
        }
        let object = Self {
            n,
            word: encoded[i..encoded.len() - 1].to_vec(),
            support,
        };
        object.validate()?;
        Ok(object)
    }

    /// Consume the whole operator-space object and either re-imscribe the next
    /// space or close it around a factor pair.
    pub fn descend(self) -> Result<Descent, String> {
        self.validate()?;
        if self.word == SHORT_FRONTIER_WORD.chars().collect::<Vec<_>>() {
            if let Some((p, q, cell)) = fermat_lattice(&self.n, 0, FRONTIER_CELLS) {
                return close(
                    self.n,
                    p,
                    q,
                    SHORT_FRONTIER_CLOSED_WORD,
                    self.support | SUPPORT_SHORT_FRONTIER | SUPPORT_PRODUCT_BOUNDARY,
                    cell,
                );
            }
            let next = DialecticObject {
                n: self.n,
                word: EXTENDED_FERMAT_WORD.chars().collect(),
                support: self.support | SUPPORT_SHORT_FRONTIER,
            };
            next.validate()?;
            return Ok(Descent::Continue(next));
        }

        if self.word == EXTENDED_FERMAT_WORD.chars().collect::<Vec<_>>() {
            if let Some((p, q, cell)) = fermat_lattice(
                &self.n,
                FRONTIER_CELLS,
                FERMAT_CELLS - FRONTIER_CELLS,
            ) {
                return close(
                    self.n,
                    p,
                    q,
                    EXTENDED_FERMAT_CLOSED_WORD,
                    self.support | SUPPORT_EXTENDED_FERMAT | SUPPORT_PRODUCT_BOUNDARY,
                    cell,
                );
            }
            return Err(String::from(
                "dialectic descent reached an uninscribed deeper factor space",
            ));
        }

        Err(String::from("unknown dialectic inscription"))
    }
}

fn close(
    n: Tape,
    p: Tape,
    q: Tape,
    closed_word: &str,
    support: LaneSupport,
    lattice_cell: usize,
) -> Result<Descent, String> {
    let word: Vec<Mark> = closed_word.chars().collect();
    if verdict(&word) != 'T' {
        return Err(String::from("closed dialectic inscription is not FOUR=T"));
    }
    let trace = encode_trace(&[GStep {
        repr: '⋈',
        judgment: M_T,
        recognised: M_T,
        next: M_FIX,
        applied_word: word.clone(),
    }]);
    let carrier = FactorCarrier::new(n, p, q, trace)?;
    Ok(Descent::Closed(DialecticClosure {
        carrier,
        word,
        support,
        lattice_cell,
    }))
}

/// Walk a contiguous interval of the Fermat lattice.  `start` is an absolute
/// cell offset from ceil(sqrt(N)); later inscriptions therefore continue the
/// previous walk rather than replaying it.
fn fermat_lattice(n: &[Mark], start: usize, cells: usize) -> Option<(Tape, Tape, usize)> {
    let one = tape_u64(1);
    let mut a = isqrt(n);
    if cmp(&mul(&a, &a), n) == Ordering::Less {
        a = add(&a, &one);
    }
    for _ in 0..start {
        a = add(&a, &one);
    }

    for offset in 0..cells {
        let a2 = mul(&a, &a);
        if cmp(&a2, n) != Ordering::Less {
            let b2 = sub(&a2, n);
            let b = isqrt(&b2);
            if mul(&b, &b) == b2 {
                let p = sub(&a, &b);
                if cmp(&p, &one) == Ordering::Greater && cmp(&p, n) == Ordering::Less {
                    let (q, remainder) = divmod(n, &p);
                    if zero(&remainder) && cmp(&q, &one) == Ordering::Greater {
                        return Some((p, q, start + offset));
                    }
                }
            }
        }
        a = add(&a, &one);
    }
    None
}

fn push_tape(out: &mut Vec<Mark>, tape: &[Mark]) {
    out.push('∈');
    out.extend_from_slice(tape);
    out.push('∋');
}

fn read_tape(encoded: &[Mark], i: &mut usize) -> Result<Tape, String> {
    if encoded.get(*i).copied() != Some('∈') {
        return Err(String::from("missing dialectic numeral field"));
    }
    *i += 1;
    let mut tape = Vec::new();
    loop {
        let mark = *encoded
            .get(*i)
            .ok_or_else(|| String::from("truncated dialectic numeral field"))?;
        *i += 1;
        if mark == '∋' {
            break;
        }
        if mark != EVALT && mark != EVALF {
            return Err(String::from("dialectic numeral contains a non-numeral mark"));
        }
        tape.push(mark);
    }
    if tape.is_empty() {
        return Err(String::from("dialectic numeral is empty"));
    }
    Ok(trim(tape))
}
