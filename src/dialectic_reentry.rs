//! Dialectic re-entry: the operator imscribes its space, walks that lattice,
//! employs FOUR, and re-imscribes the transformed space.
//!
//! The `m` in imscription is load-bearing. IMSCRIB is the live read/write/execute
//! relation between the bulk object and its boundary. A descent consumes that
//! complete relation: the boundary reads the bulk N, executes the current IMASM
//! word, and writes the transformed lattice boundary carried by the next object.
//! Nothing from the consumed walk is replayed to reconstruct the next starting
//! point.

use alloc::string::String;
use alloc::vec::Vec;
use core::cmp::Ordering;

use crate::factor_extract::{FactorCarrier, Mark, Tape};
use crate::morphism_factor::{add, cmp, divmod, isqrt, mul, sub, tape_u64, trim, zero};
use crate::producer_provenance::{
    SUPPORT_DEEP_ARM, SUPPORT_EXTENDED_FERMAT, SUPPORT_PARITY, SUPPORT_PRIMALITY,
    SUPPORT_PRODUCT_BOUNDARY, SUPPORT_SHORT_FRONTIER,
};
use crate::provenance_envelope::{restored_support, LaneSupport, RestoredSupport};
use crate::router_marks::{GStep, M_FIX, M_T};
use crate::trace_word::encode_trace;
use crate::vox::{verdict, EVALF, EVALT, IMSCRIB, TANCH, VINIT};

/// An unresolved imscription deliberately leaves its factor fork open: FOUR=B.
pub const SHORT_FRONTIER_WORD: &str = "⊢⊙∈≻⊤≺⊥⊡⊣";
/// The continuation is a new whole imscription, not the old word plus a cursor.
pub const EXTENDED_FERMAT_WORD: &str = "⊢⊙∈≻⊤⊞≺⊥⊡⊣";
/// The first change of lattice: the boundary is now Lehman's multiplier k.
pub const LEHMAN_WORD: &str = "⊢⊙∈≻⋈⊤⊥⊡⊣";

const SHORT_FRONTIER_CLOSED_WORD: &str = "⊢⊙∈≻⊤≺⊥∋⊡⊣";
const EXTENDED_FERMAT_CLOSED_WORD: &str = "⊢⊙∈≻⊤⊞≺⊥∋⊡⊣";
const LEHMAN_CLOSED_WORD: &str = "⊢⊙∈≻⋈⊤⊥∋⊡⊣";
const SUPPORT_BITS: usize = 6;
const RWX_BITS: usize = 3;
const FRONTIER_CELLS: usize = 64;
/// The second Fermat ring ends at absolute cell 4096. The next B changes lattice.
const FERMAT_CELLS: usize = 4096;
/// One Lehman imscription executes one multiplier and a local 64-cell a-window.
const LEHMAN_A_CELLS: usize = 64;
const BASE_SUPPORT: LaneSupport = SUPPORT_PARITY | SUPPORT_PRIMALITY;

/// Dynamic bulk/boundary coupling carried by IMSCRIB.
pub const IM_READ: u8 = 1 << 0;
pub const IM_WRITE: u8 = 1 << 1;
pub const IM_EXEC: u8 = 1 << 2;
pub const IM_RWX: u8 = IM_READ | IM_WRITE | IM_EXEC;

/// The live imscription relation.
///
/// `boundary` is not a host cursor. It is the current IMASM numeral at the
/// operator's boundary. `rwx` says what relation the boundary has to the bulk:
/// read the bulk, write the transformed boundary, execute the current word.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Imscription {
    pub boundary: Tape,
    pub rwx: u8,
}

impl Imscription {
    fn active(boundary: Tape) -> Self {
        Self { boundary: trim(boundary), rwx: IM_RWX }
    }

    pub fn can_read(&self) -> bool { self.rwx & IM_READ != 0 }
    pub fn can_write(&self) -> bool { self.rwx & IM_WRITE != 0 }
    pub fn can_execute(&self) -> bool { self.rwx & IM_EXEC != 0 }
}

/// One complete, restartable operator-space object.
///
/// `n` is the bulk. `imscription` is the live bulk/boundary relation. `word`
/// is what the operator executes at that boundary. `support` is the distinction
/// already embodied by this transformed whole object.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct DialecticObject {
    pub n: Tape,
    pub word: Vec<Mark>,
    pub support: LaneSupport,
    pub imscription: Imscription,
}

/// A terminal contraction: the imscription has closed around one factor pair
/// and is handed to the passive factor-carrier layer.
#[derive(Clone, PartialEq, Debug)]
pub struct DialecticClosure {
    pub carrier: FactorCarrier,
    pub word: Vec<Mark>,
    pub support: LaneSupport,
    pub imscription: Imscription,
    /// Cell inside the closing lattice. For Fermat this is the absolute cell;
    /// for Lehman it is the a-offset inside `lehman_multiplier`.
    pub lattice_cell: usize,
    /// Present only when closure occurred in the Lehman multiplier lattice.
    pub lehman_multiplier: Option<Tape>,
}

#[derive(Clone, PartialEq, Debug)]
pub enum Descent {
    Continue(DialecticObject),
    Closed(DialecticClosure),
}

impl DialecticObject {
    /// Initial operator-space imscription. Parity and primality are the ground
    /// support; the boundary begins at ceil(sqrt(N)).
    pub fn new(n: Tape) -> Result<Self, String> {
        let n = trim(n);
        let boundary = fermat_origin(&n);
        let object = Self {
            n,
            word: SHORT_FRONTIER_WORD.chars().collect(),
            support: BASE_SUPPORT,
            imscription: Imscription::active(boundary),
        };
        object.validate()?;
        Ok(object)
    }

    pub fn validate(&self) -> Result<(), String> {
        if cmp(&self.n, &tape_u64(1)) != Ordering::Greater {
            return Err(String::from("dialectic object requires N > 1"));
        }
        let short = self.word == SHORT_FRONTIER_WORD.chars().collect::<Vec<_>>();
        let extended = self.word == EXTENDED_FERMAT_WORD.chars().collect::<Vec<_>>();
        let lehman = self.word == LEHMAN_WORD.chars().collect::<Vec<_>>();
        let expected_support = if short {
            BASE_SUPPORT
        } else if extended {
            BASE_SUPPORT | SUPPORT_SHORT_FRONTIER
        } else if lehman {
            BASE_SUPPORT | SUPPORT_SHORT_FRONTIER | SUPPORT_EXTENDED_FERMAT
        } else {
            return Err(String::from("unknown dialectic imscription"));
        };
        if self.support != expected_support {
            return Err(String::from("dialectic imscription/support mismatch"));
        }
        if self.imscription.rwx != IM_RWX {
            return Err(String::from("dialectic imscription is not live r/w/x"));
        }

        if short || extended {
            let origin = fermat_origin(&self.n);
            let expected_boundary = if short {
                origin
            } else {
                add(&origin, &tape_u64(FRONTIER_CELLS as u64))
            };
            if cmp(&self.imscription.boundary, &expected_boundary) != Ordering::Equal {
                return Err(String::from("dialectic imscription boundary does not match transformed Fermat space"));
            }
        } else {
            let one = tape_u64(1);
            if cmp(&self.imscription.boundary, &one) == Ordering::Less {
                return Err(String::from("dialectic Lehman boundary must be a positive imscribed multiplier"));
            }
        }

        if !self.word.contains(&IMSCRIB) {
            return Err(String::from("dialectic imscription does not contain IMSCRIB"));
        }
        if verdict(&self.word) != 'B' {
            return Err(String::from("unresolved dialectic imscription is not FOUR=B"));
        }
        Ok(())
    }

    /// Native support currently restored by this whole imscription.
    pub fn envelope(&self) -> RestoredSupport {
        restored_support(&[self.support])
    }

    /// Marks-only persistence of the complete current operator-space relation:
    ///
    /// `⊢ ⊙ ∈N∋ ∈boundary∋ ∈rwx[3]∋ ∈support[6]∋ <current IMASM word> ⊣`
    pub fn encode(&self) -> Vec<Mark> {
        let mut out = Vec::with_capacity(
            self.n.len()
                + self.imscription.boundary.len()
                + self.word.len()
                + RWX_BITS
                + SUPPORT_BITS
                + 12,
        );
        out.push(VINIT);
        out.push(IMSCRIB);
        push_tape(&mut out, &self.n);
        push_tape(&mut out, &self.imscription.boundary);
        push_mask(&mut out, self.imscription.rwx as u32, RWX_BITS);
        push_mask(&mut out, self.support, SUPPORT_BITS);
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
        let boundary = read_tape(encoded, &mut i)?;
        let rwx = read_mask(encoded, &mut i, RWX_BITS)? as u8;
        let support = read_mask(encoded, &mut i, SUPPORT_BITS)?;
        if i >= encoded.len() - 1 {
            return Err(String::from("dialectic imscription word is empty"));
        }
        let object = Self {
            n,
            word: encoded[i..encoded.len() - 1].to_vec(),
            support,
            imscription: Imscription { boundary, rwx },
        };
        object.validate()?;
        Ok(object)
    }

    /// Consume the whole operator-space relation and either re-imscribe the
    /// transformed space or close it around a factor pair.
    pub fn descend(self) -> Result<Descent, String> {
        self.validate()?;
        if !self.imscription.can_read()
            || !self.imscription.can_write()
            || !self.imscription.can_execute()
        {
            return Err(String::from("dialectic imscription lost r/w/x coupling"));
        }

        if self.word == SHORT_FRONTIER_WORD.chars().collect::<Vec<_>>() {
            match fermat_lattice_from(&self.n, &self.imscription.boundary, 0, FRONTIER_CELLS) {
                LatticeResult::Closed { p, q, cell, boundary } => {
                    return close(
                        self.n,
                        p,
                        q,
                        SHORT_FRONTIER_CLOSED_WORD,
                        self.support | SUPPORT_SHORT_FRONTIER | SUPPORT_PRODUCT_BOUNDARY,
                        Imscription::active(boundary),
                        cell,
                        None,
                    );
                }
                LatticeResult::Open { boundary } => {
                    let next = DialecticObject {
                        n: self.n,
                        word: EXTENDED_FERMAT_WORD.chars().collect(),
                        support: self.support | SUPPORT_SHORT_FRONTIER,
                        imscription: Imscription::active(boundary),
                    };
                    next.validate()?;
                    return Ok(Descent::Continue(next));
                }
            }
        }

        if self.word == EXTENDED_FERMAT_WORD.chars().collect::<Vec<_>>() {
            match fermat_lattice_from(
                &self.n,
                &self.imscription.boundary,
                FRONTIER_CELLS,
                FERMAT_CELLS - FRONTIER_CELLS,
            ) {
                LatticeResult::Closed { p, q, cell, boundary } => {
                    return close(
                        self.n,
                        p,
                        q,
                        EXTENDED_FERMAT_CLOSED_WORD,
                        self.support | SUPPORT_EXTENDED_FERMAT | SUPPORT_PRODUCT_BOUNDARY,
                        Imscription::active(boundary),
                        cell,
                        None,
                    );
                }
                LatticeResult::Open { .. } => {
                    // The Fermat boundary has been completely consumed. The
                    // transformed operator now imscribes a different lattice:
                    // Lehman's multiplier k, beginning at k=1. The old Fermat
                    // space survives structurally in SUPPORT_EXTENDED_FERMAT.
                    let next = DialecticObject {
                        n: self.n,
                        word: LEHMAN_WORD.chars().collect(),
                        support: self.support | SUPPORT_EXTENDED_FERMAT,
                        imscription: Imscription::active(tape_u64(1)),
                    };
                    next.validate()?;
                    return Ok(Descent::Continue(next));
                }
            }
        }

        if self.word == LEHMAN_WORD.chars().collect::<Vec<_>>() {
            let k = self.imscription.boundary.clone();
            match lehman_multiplier(&self.n, &k) {
                LehmanResult::Closed { p, q, cell } => {
                    return close(
                        self.n,
                        p,
                        q,
                        LEHMAN_CLOSED_WORD,
                        self.support | SUPPORT_DEEP_ARM | SUPPORT_PRODUCT_BOUNDARY,
                        Imscription::active(k.clone()),
                        cell,
                        Some(k),
                    );
                }
                LehmanResult::Open => {
                    let next = DialecticObject {
                        n: self.n,
                        word: LEHMAN_WORD.chars().collect(),
                        support: self.support,
                        imscription: Imscription::active(add(&k, &tape_u64(1))),
                    };
                    next.validate()?;
                    return Ok(Descent::Continue(next));
                }
            }
        }

        Err(String::from("unknown dialectic imscription"))
    }
}

fn close(
    n: Tape,
    p: Tape,
    q: Tape,
    closed_word: &str,
    support: LaneSupport,
    imscription: Imscription,
    lattice_cell: usize,
    lehman_multiplier: Option<Tape>,
) -> Result<Descent, String> {
    let word: Vec<Mark> = closed_word.chars().collect();
    if verdict(&word) != 'T' {
        return Err(String::from("closed dialectic imscription is not FOUR=T"));
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
        imscription,
        lattice_cell,
        lehman_multiplier,
    }))
}

fn fermat_origin(n: &[Mark]) -> Tape {
    let mut a = isqrt(n);
    if cmp(&mul(&a, &a), n) == Ordering::Less {
        a = add(&a, &tape_u64(1));
    }
    a
}

enum LatticeResult {
    Closed {
        p: Tape,
        q: Tape,
        cell: usize,
        boundary: Tape,
    },
    Open {
        boundary: Tape,
    },
}

/// Walk directly from the boundary imscribed by the current object. On an open
/// result, the returned boundary is the first unwalked lattice point and is
/// written into the next imscription. No earlier cell is reconstructed or replayed.
fn fermat_lattice_from(
    n: &[Mark],
    start_boundary: &[Mark],
    base_cell: usize,
    cells: usize,
) -> LatticeResult {
    let one = tape_u64(1);
    let mut a = trim(start_boundary.to_vec());

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
                        return LatticeResult::Closed {
                            p,
                            q,
                            cell: base_cell + offset,
                            boundary: a,
                        };
                    }
                }
            }
        }
        a = add(&a, &one);
    }

    LatticeResult::Open { boundary: a }
}

enum LehmanResult {
    Closed { p: Tape, q: Tape, cell: usize },
    Open,
}

/// Execute one complete Lehman-multiplier imscription. The boundary is k. The
/// local a-lattice begins at ceil(sqrt(4*k*N)) and is consumed for 64 cells.
/// A B writes k+1 into the next imscription; no earlier multiplier is replayed.
fn lehman_multiplier(n: &[Mark], k: &[Mark]) -> LehmanResult {
    let one = tape_u64(1);
    let four_kn = mul(&tape_u64(4), &mul(k, n));
    let mut a = isqrt(&four_kn);
    if cmp(&mul(&a, &a), &four_kn) == Ordering::Less {
        a = add(&a, &one);
    }

    for cell in 0..LEHMAN_A_CELLS {
        let a2 = mul(&a, &a);
        if cmp(&a2, &four_kn) != Ordering::Less {
            let b2 = sub(&a2, &four_kn);
            let b = isqrt(&b2);
            if mul(&b, &b) == b2 {
                let plus = add(&a, &b);
                if let Some((p, q)) = factor_from_gcd(n, &plus) {
                    return LehmanResult::Closed { p, q, cell };
                }
                if cmp(&a, &b) != Ordering::Less {
                    let minus = sub(&a, &b);
                    if let Some((p, q)) = factor_from_gcd(n, &minus) {
                        return LehmanResult::Closed { p, q, cell };
                    }
                }
            }
        }
        a = add(&a, &one);
    }
    LehmanResult::Open
}

fn factor_from_gcd(n: &[Mark], x: &[Mark]) -> Option<(Tape, Tape)> {
    let one = tape_u64(1);
    let p = gcd_tape(x, n);
    if cmp(&p, &one) != Ordering::Greater || cmp(&p, n) != Ordering::Less {
        return None;
    }
    let (q, remainder) = divmod(n, &p);
    if !zero(&remainder) || cmp(&q, &one) != Ordering::Greater {
        return None;
    }
    Some((p, q))
}

fn gcd_tape(a: &[Mark], b: &[Mark]) -> Tape {
    let mut x = trim(a.to_vec());
    let mut y = trim(b.to_vec());
    while !zero(&y) {
        let (_, r) = divmod(&x, &y);
        x = y;
        y = r;
    }
    x
}

fn push_tape(out: &mut Vec<Mark>, tape: &[Mark]) {
    out.push('∈');
    out.extend_from_slice(tape);
    out.push('∋');
}

fn push_mask(out: &mut Vec<Mark>, mask: u32, bits: usize) {
    out.push('∈');
    for bit in 0..bits {
        out.push(if (mask >> bit) & 1 == 1 { EVALF } else { EVALT });
    }
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

fn read_mask(encoded: &[Mark], i: &mut usize, bits: usize) -> Result<u32, String> {
    if encoded.get(*i).copied() != Some('∈') {
        return Err(String::from("missing dialectic imscription mask field"));
    }
    *i += 1;
    let mut mask = 0u32;
    for bit in 0..bits {
        match encoded.get(*i).copied() {
            Some(EVALT) => {}
            Some(EVALF) => mask |= 1u32 << bit,
            _ => return Err(String::from("malformed dialectic imscription mask")),
        }
        *i += 1;
    }
    if encoded.get(*i).copied() != Some('∋') {
        return Err(String::from("unterminated dialectic imscription mask"));
    }
    *i += 1;
    Ok(mask)
}
