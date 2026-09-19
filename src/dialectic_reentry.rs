//! Dialectic re-entry: the operator imscribes its space, walks that lattice,
//! employs FOUR, and re-imscribes the transformed space.
//!
//! The `m` in imscription is load-bearing. IMSCRIB is the live read/write/execute
//! relation between the bulk object and its boundary. A descent consumes that
//! complete relation: the boundary reads the bulk N, executes the current IMASM
//! word, and writes the transformed lattice boundary carried by the next object.
//! The finite lattice span is part of that relation too; it is persisted as a
//! tape numeral rather than reconstructed from hidden host state.

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

/// The word semantics define finite imscribed regions, but the region size is
/// carried by the object itself as a tape numeral.
pub const SHORT_FRONTIER_SPAN: u64 = 64;
pub const EXTENDED_FERMAT_SPAN: u64 = 4096 - SHORT_FRONTIER_SPAN;
pub const LEHMAN_LOCAL_SPAN: u64 = 64;

const BASE_SUPPORT: LaneSupport = SUPPORT_PARITY | SUPPORT_PRIMALITY;

/// Dynamic bulk/boundary coupling carried by IMSCRIB.
pub const IM_READ: u8 = 1 << 0;
pub const IM_WRITE: u8 = 1 << 1;
pub const IM_EXEC: u8 = 1 << 2;
pub const IM_RWX: u8 = IM_READ | IM_WRITE | IM_EXEC;

/// The live imscription relation.
///
/// `boundary` is the current IMASM numeral at the operator's boundary. `span`
/// is the finite lattice region that boundary currently affords. `rwx` says what
/// relation the boundary has to the bulk: read the bulk, write the transformed
/// boundary/space, execute the current word.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Imscription {
    pub boundary: Tape,
    pub span: Tape,
    pub rwx: u8,
}

impl Imscription {
    fn active(boundary: Tape, span: Tape) -> Self {
        Self {
            boundary: trim(boundary),
            span: trim(span),
            rwx: IM_RWX,
        }
    }

    pub fn can_read(&self) -> bool {
        self.rwx & IM_READ != 0
    }

    pub fn can_write(&self) -> bool {
        self.rwx & IM_WRITE != 0
    }

    pub fn can_execute(&self) -> bool {
        self.rwx & IM_EXEC != 0
    }
}

/// One complete, restartable operator-space object.
///
/// `n` is the bulk. `imscription` is the live bulk/boundary/space relation.
/// `word` is what the operator executes there. `support` is the distinction
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
    /// Closing coordinate inside the current lattice, carried as the same native
    /// numeral tape that will be bound by the dialectic certificate. For Fermat
    /// this is absolute across the two installed rings; for Lehman it is local.
    pub lattice_cell: Tape,
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
    /// support; the boundary begins at ceil(sqrt(N)) and explicitly imscribes
    /// the 64-cell short frontier.
    pub fn new(n: Tape) -> Result<Self, String> {
        let n = trim(n);
        let boundary = fermat_origin(&n);
        let object = Self {
            n,
            word: SHORT_FRONTIER_WORD.chars().collect(),
            support: BASE_SUPPORT,
            imscription: Imscription::active(boundary, tape_u64(SHORT_FRONTIER_SPAN)),
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

        let expected_span = if short {
            tape_u64(SHORT_FRONTIER_SPAN)
        } else if extended {
            tape_u64(EXTENDED_FERMAT_SPAN)
        } else {
            tape_u64(LEHMAN_LOCAL_SPAN)
        };
        if cmp(&self.imscription.span, &expected_span) != Ordering::Equal {
            return Err(String::from("dialectic imscription span does not match the current lattice word"));
        }

        if short || extended {
            let origin = fermat_origin(&self.n);
            let expected_boundary = if short {
                origin
            } else {
                add(&origin, &tape_u64(SHORT_FRONTIER_SPAN))
            };
            if cmp(&self.imscription.boundary, &expected_boundary) != Ordering::Equal {
                return Err(String::from(
                    "dialectic imscription boundary does not match transformed Fermat space",
                ));
            }
        } else {
            let one = tape_u64(1);
            if cmp(&self.imscription.boundary, &one) == Ordering::Less {
                return Err(String::from(
                    "dialectic Lehman boundary must be a positive imscribed multiplier",
                ));
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
    /// `⊢ ⊙ ∈N∋ ∈boundary∋ ∈span∋ ∈rwx[3]∋ ∈support[6]∋ <word> ⊣`
    pub fn encode(&self) -> Vec<Mark> {
        let mut out = Vec::with_capacity(
            self.n.len()
                + self.imscription.boundary.len()
                + self.imscription.span.len()
                + self.word.len()
                + RWX_BITS
                + SUPPORT_BITS
                + 14,
        );
        out.push(VINIT);
        out.push(IMSCRIB);
        push_tape(&mut out, &self.n);
        push_tape(&mut out, &self.imscription.boundary);
        push_tape(&mut out, &self.imscription.span);
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
        let span = read_tape(encoded, &mut i)?;
        let rwx = read_mask(encoded, &mut i, RWX_BITS)? as u8;
        let support = read_mask(encoded, &mut i, SUPPORT_BITS)?;
        if i >= encoded.len() - 1 {
            return Err(String::from("dialectic imscription word is empty"));
        }
        let object = Self {
            n,
            word: encoded[i..encoded.len() - 1].to_vec(),
            support,
            imscription: Imscription {
                boundary,
                span,
                rwx,
            },
        };
        object.validate()?;
        Ok(object)
    }

    /// Consume the whole operator-space relation and either re-imscribe the
    /// transformed space or close it around a factor pair. The executor reads
    /// the finite cell count from the persisted `span` tape.
    pub fn descend(self) -> Result<Descent, String> {
        self.validate()?;
        if !self.imscription.can_read()
            || !self.imscription.can_write()
            || !self.imscription.can_execute()
        {
            return Err(String::from("dialectic imscription lost r/w/x coupling"));
        }

        let cells = span_to_usize(&self.imscription.span)?;

        if self.word == SHORT_FRONTIER_WORD.chars().collect::<Vec<_>>() {
            let base_cell = tape_u64(0);
            match fermat_lattice_from(&self.n, &self.imscription.boundary, &base_cell, cells) {
                LatticeResult::Closed {
                    p,
                    q,
                    cell,
                    boundary,
                } => {
                    return close(
                        self.n,
                        p,
                        q,
                        SHORT_FRONTIER_CLOSED_WORD,
                        self.support | SUPPORT_SHORT_FRONTIER | SUPPORT_PRODUCT_BOUNDARY,
                        Imscription::active(boundary, self.imscription.span),
                        cell,
                        None,
                    );
                }
                LatticeResult::Open { boundary } => {
                    let next = DialecticObject {
                        n: self.n,
                        word: EXTENDED_FERMAT_WORD.chars().collect(),
                        support: self.support | SUPPORT_SHORT_FRONTIER,
                        imscription: Imscription::active(
                            boundary,
                            tape_u64(EXTENDED_FERMAT_SPAN),
                        ),
                    };
                    next.validate()?;
                    return Ok(Descent::Continue(next));
                }
            }
        }

        if self.word == EXTENDED_FERMAT_WORD.chars().collect::<Vec<_>>() {
            let base_cell = tape_u64(SHORT_FRONTIER_SPAN);
            match fermat_lattice_from(
                &self.n,
                &self.imscription.boundary,
                &base_cell,
                cells,
            ) {
                LatticeResult::Closed {
                    p,
                    q,
                    cell,
                    boundary,
                } => {
                    return close(
                        self.n,
                        p,
                        q,
                        EXTENDED_FERMAT_CLOSED_WORD,
                        self.support | SUPPORT_EXTENDED_FERMAT | SUPPORT_PRODUCT_BOUNDARY,
                        Imscription::active(boundary, self.imscription.span),
                        cell,
                        None,
                    );
                }
                LatticeResult::Open { .. } => {
                    // The current Fermat space has been completely consumed.
                    // FOUR=B changes lattice: the next boundary denotes Lehman's
                    // multiplier k and explicitly imscribes its local a-span.
                    let next = DialecticObject {
                        n: self.n,
                        word: LEHMAN_WORD.chars().collect(),
                        support: self.support | SUPPORT_EXTENDED_FERMAT,
                        imscription: Imscription::active(
                            tape_u64(1),
                            tape_u64(LEHMAN_LOCAL_SPAN),
                        ),
                    };
                    next.validate()?;
                    return Ok(Descent::Continue(next));
                }
            }
        }

        if self.word == LEHMAN_WORD.chars().collect::<Vec<_>>() {
            let k = self.imscription.boundary.clone();
            match lehman_multiplier(&self.n, &k, cells) {
                LehmanResult::Closed { p, q, cell } => {
                    return close(
                        self.n,
                        p,
                        q,
                        LEHMAN_CLOSED_WORD,
                        self.support | SUPPORT_DEEP_ARM | SUPPORT_PRODUCT_BOUNDARY,
                        Imscription::active(k.clone(), self.imscription.span),
                        cell,
                        Some(k),
                    );
                }
                LehmanResult::Open => {
                    let next = DialecticObject {
                        n: self.n,
                        word: LEHMAN_WORD.chars().collect(),
                        support: self.support,
                        imscription: Imscription::active(
                            add(&k, &tape_u64(1)),
                            self.imscription.span,
                        ),
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
    lattice_cell: Tape,
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
        cell: Tape,
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
    base_cell: &[Mark],
    cells: usize,
) -> LatticeResult {
    let one = tape_u64(1);
    let mut a = trim(start_boundary.to_vec());
    let mut cell = trim(base_cell.to_vec());

    for _ in 0..cells {
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
                            cell,
                            boundary: a,
                        };
                    }
                }
            }
        }
        a = add(&a, &one);
        cell = add(&cell, &one);
    }

    LatticeResult::Open { boundary: a }
}

enum LehmanResult {
    Closed { p: Tape, q: Tape, cell: Tape },
    Open,
}

/// Execute one complete Lehman-multiplier imscription. The boundary is k. The
/// local a-lattice begins at ceil(sqrt(4*k*N)) and consumes exactly the span
/// carried by the current imscription. A B writes k+1 into the next imscription;
/// no earlier multiplier is replayed.
fn lehman_multiplier(n: &[Mark], k: &[Mark], cells: usize) -> LehmanResult {
    let one = tape_u64(1);
    let four_kn = mul(&tape_u64(4), &mul(k, n));
    let mut a = isqrt(&four_kn);
    let mut cell = tape_u64(0);
    if cmp(&mul(&a, &a), &four_kn) == Ordering::Less {
        a = add(&a, &one);
    }

    for _ in 0..cells {
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
        cell = add(&cell, &one);
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

fn span_to_usize(span: &[Mark]) -> Result<usize, String> {
    if span.is_empty() {
        return Err(String::from("dialectic imscription span is empty"));
    }
    let mut value = 0usize;
    for (bit, &mark) in span.iter().enumerate() {
        match mark {
            EVALT => {}
            EVALF => {
                if bit >= usize::BITS as usize {
                    return Err(String::from(
                        "dialectic imscription span exceeds the finite local executor",
                    ));
                }
                value |= 1usize
                    .checked_shl(bit as u32)
                    .ok_or_else(|| String::from("dialectic imscription span overflow"))?;
            }
            _ => {
                return Err(String::from(
                    "dialectic imscription span contains a non-numeral mark",
                ));
            }
        }
    }
    if value == 0 {
        return Err(String::from("dialectic imscription span must be positive"));
    }
    Ok(value)
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
