//! Dialectic re-entry: the operator imscribes its space, walks that lattice,
//! employs FOUR, and re-imscribes the transformed space.
//!
//! The `m` in imscription is load-bearing. IMSCRIB is the live read/write/execute
//! relation between the bulk object and its boundary. A descent consumes that
//! complete relation: the boundary reads the bulk N, executes the current IMASM
//! word, and writes the transformed lattice boundary carried by the next object.
//! The finite lattice span is part of that relation too; it is persisted and
//! consumed as a tape numeral rather than reconstructed or narrowed into host state.

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

/// Capabilities of a live imscription relation. The bit-set says which actions
/// are enabled; the endpoints below say what those actions currently relate.
pub const IM_READ: u8 = 1 << 0;
pub const IM_WRITE: u8 = 1 << 1;
pub const IM_EXEC: u8 = 1 << 2;
pub const IM_RWX: u8 = IM_READ | IM_WRITE | IM_EXEC;

/// The load-bearing r/w/x relation of IMSCRIB.
///
/// `rights` is only the capability set. The relation itself is the four exact
/// endpoints: which bulk is read, which boundary is written, which finite span
/// is executed, and which current IMASM word performs that execution.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ImscriptionRwx {
    pub rights: u8,
    pub read_bulk: Tape,
    pub write_boundary: Tape,
    pub execute_span: Tape,
    pub execute_word: Vec<Mark>,
}

impl ImscriptionRwx {
    fn live(
        read_bulk: &[Mark],
        write_boundary: &[Mark],
        execute_span: &[Mark],
        execute_word: &[Mark],
    ) -> Self {
        Self {
            rights: IM_RWX,
            read_bulk: trim(read_bulk.to_vec()),
            write_boundary: trim(write_boundary.to_vec()),
            execute_span: trim(execute_span.to_vec()),
            execute_word: execute_word.to_vec(),
        }
    }

    pub fn can_read(&self) -> bool {
        self.rights & IM_READ != 0
    }

    pub fn can_write(&self) -> bool {
        self.rights & IM_WRITE != 0
    }

    pub fn can_execute(&self) -> bool {
        self.rights & IM_EXEC != 0
    }
}

/// The live imscription relation.
///
/// `boundary` is the current IMASM numeral at the operator's boundary. `span`
/// is the finite lattice region that boundary currently affords. `rwx` binds
/// those values to the actual bulk and current word rather than merely carrying
/// a static permission mask.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Imscription {
    pub boundary: Tape,
    pub span: Tape,
    pub rwx: ImscriptionRwx,
}

impl Imscription {
    fn active(n: &[Mark], boundary: Tape, span: Tape, word: &[Mark]) -> Self {
        let boundary = trim(boundary);
        let span = trim(span);
        let rwx = ImscriptionRwx::live(n, &boundary, &span, word);
        Self {
            boundary,
            span,
            rwx,
        }
    }

    pub fn relation_is_live_for(&self, n: &[Mark], word: &[Mark]) -> bool {
        self.rwx.rights == IM_RWX
            && self.rwx.can_read()
            && self.rwx.can_write()
            && self.rwx.can_execute()
            && self.rwx.read_bulk == trim(n.to_vec())
            && self.rwx.write_boundary == self.boundary
            && self.rwx.execute_span == self.span
            && self.rwx.execute_word == word
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
        let span = tape_u64(SHORT_FRONTIER_SPAN);
        let word: Vec<Mark> = SHORT_FRONTIER_WORD.chars().collect();
        let imscription = Imscription::active(&n, boundary, span, &word);
        let object = Self {
            n,
            word,
            support: BASE_SUPPORT,
            imscription,
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
        if self.imscription.rwx.rights != IM_RWX {
            return Err(String::from("dialectic imscription does not expose live r/w/x capabilities"));
        }
        if self.imscription.rwx.read_bulk != self.n {
            return Err(String::from("dialectic imscription read relation does not bind the current bulk"));
        }
        if self.imscription.rwx.write_boundary != self.imscription.boundary {
            return Err(String::from("dialectic imscription write relation does not bind the current boundary"));
        }
        if self.imscription.rwx.execute_span != self.imscription.span {
            return Err(String::from("dialectic imscription execute relation does not bind the current span"));
        }
        if self.imscription.rwx.execute_word != self.word {
            return Err(String::from("dialectic imscription execute relation does not bind the current word"));
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
        if !self.imscription.relation_is_live_for(&self.n, &self.word) {
            return Err(String::from("dialectic imscription r/w/x relation is not live for this whole object"));
        }
        Ok(())
    }

    /// Native support currently restored by this whole imscription.
    pub fn envelope(&self) -> RestoredSupport {
        restored_support(&[self.support])
    }

    /// Marks-only persistence of the complete current operator-space relation:
    ///
    /// `⊢ ⊙ ∈N∋ ∈boundary∋ ∈span∋ ∈rights[3]∋ ∈read-N∋
    ///    ∈write-boundary∋ ∈exec-span∋ ∈len(exec-word)∋ exec-word
    ///    ∈support[6]∋ <word> ⊣`
    ///
    /// The executed word is length-framed because structural IMASM glyphs are
    /// legitimate data. Older permission-only wire objects are deliberately not
    /// assigned invented relation endpoints by this decoder.
    pub fn encode(&self) -> Vec<Mark> {
        let mut out = Vec::with_capacity(
            self.n.len()
                + self.imscription.boundary.len()
                + self.imscription.span.len()
                + self.imscription.rwx.read_bulk.len()
                + self.imscription.rwx.write_boundary.len()
                + self.imscription.rwx.execute_span.len()
                + self.imscription.rwx.execute_word.len()
                + self.word.len()
                + RWX_BITS
                + SUPPORT_BITS
                + 26,
        );
        out.push(VINIT);
        out.push(IMSCRIB);
        push_tape(&mut out, &self.n);
        push_tape(&mut out, &self.imscription.boundary);
        push_tape(&mut out, &self.imscription.span);
        push_mask(&mut out, self.imscription.rwx.rights as u32, RWX_BITS);
        push_tape(&mut out, &self.imscription.rwx.read_bulk);
        push_tape(&mut out, &self.imscription.rwx.write_boundary);
        push_tape(&mut out, &self.imscription.rwx.execute_span);
        push_blob(&mut out, &self.imscription.rwx.execute_word);
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
        let end = encoded.len() - 1;
        let mut i = 2usize;
        let n = read_tape(encoded, &mut i)?;
        let boundary = read_tape(encoded, &mut i)?;
        let span = read_tape(encoded, &mut i)?;
        let rights = read_mask(encoded, &mut i, RWX_BITS)? as u8;
        let read_bulk = read_tape(encoded, &mut i)?;
        let write_boundary = read_tape(encoded, &mut i)?;
        let execute_span = read_tape(encoded, &mut i)?;
        let execute_word = read_blob(encoded, &mut i, end)?;
        let support = read_mask(encoded, &mut i, SUPPORT_BITS)?;
        if i >= end {
            return Err(String::from("dialectic imscription word is empty"));
        }
        let object = Self {
            n,
            word: encoded[i..end].to_vec(),
            support,
            imscription: Imscription {
                boundary,
                span,
                rwx: ImscriptionRwx {
                    rights,
                    read_bulk,
                    write_boundary,
                    execute_span,
                    execute_word,
                },
            },
        };
        object.validate()?;
        Ok(object)
    }

    /// Consume the whole operator-space relation and either re-imscribe the
    /// transformed space or close it around a factor pair. The executor consumes
    /// the persisted `span` tape directly; no host-sized cell count mediates the walk.
    pub fn descend(self) -> Result<Descent, String> {
        self.validate()?;
        if !self.imscription.relation_is_live_for(&self.n, &self.word) {
            return Err(String::from("dialectic imscription lost its live r/w/x relation"));
        }

        if self.word == SHORT_FRONTIER_WORD.chars().collect::<Vec<_>>() {
            let base_cell = tape_u64(0);
            match fermat_lattice_from(
                &self.n,
                &self.imscription.boundary,
                &base_cell,
                &self.imscription.span,
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
                        SHORT_FRONTIER_CLOSED_WORD,
                        self.support | SUPPORT_SHORT_FRONTIER | SUPPORT_PRODUCT_BOUNDARY,
                        boundary,
                        self.imscription.span,
                        cell,
                        None,
                    );
                }
                LatticeResult::Open { boundary } => {
                    let next_word: Vec<Mark> = EXTENDED_FERMAT_WORD.chars().collect();
                    let next_span = tape_u64(EXTENDED_FERMAT_SPAN);
                    let next_imscription =
                        Imscription::active(&self.n, boundary, next_span, &next_word);
                    let next = DialecticObject {
                        n: self.n,
                        word: next_word,
                        support: self.support | SUPPORT_SHORT_FRONTIER,
                        imscription: next_imscription,
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
                &self.imscription.span,
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
                        boundary,
                        self.imscription.span,
                        cell,
                        None,
                    );
                }
                LatticeResult::Open { .. } => {
                    // The current Fermat space has been completely consumed.
                    // FOUR=B changes lattice: the next boundary denotes Lehman's
                    // multiplier k and explicitly imscribes its local a-span.
                    let next_word: Vec<Mark> = LEHMAN_WORD.chars().collect();
                    let next_boundary = tape_u64(1);
                    let next_span = tape_u64(LEHMAN_LOCAL_SPAN);
                    let next_imscription = Imscription::active(
                        &self.n,
                        next_boundary,
                        next_span,
                        &next_word,
                    );
                    let next = DialecticObject {
                        n: self.n,
                        word: next_word,
                        support: self.support | SUPPORT_EXTENDED_FERMAT,
                        imscription: next_imscription,
                    };
                    next.validate()?;
                    return Ok(Descent::Continue(next));
                }
            }
        }

        if self.word == LEHMAN_WORD.chars().collect::<Vec<_>>() {
            let k = self.imscription.boundary.clone();
            match lehman_multiplier(&self.n, &k, &self.imscription.span) {
                LehmanResult::Closed { p, q, cell } => {
                    return close(
                        self.n,
                        p,
                        q,
                        LEHMAN_CLOSED_WORD,
                        self.support | SUPPORT_DEEP_ARM | SUPPORT_PRODUCT_BOUNDARY,
                        k.clone(),
                        self.imscription.span,
                        cell,
                        Some(k),
                    );
                }
                LehmanResult::Open => {
                    let next_word: Vec<Mark> = LEHMAN_WORD.chars().collect();
                    let next_boundary = add(&k, &tape_u64(1));
                    let next_span = self.imscription.span;
                    let next_imscription = Imscription::active(
                        &self.n,
                        next_boundary,
                        next_span,
                        &next_word,
                    );
                    let next = DialecticObject {
                        n: self.n,
                        word: next_word,
                        support: self.support,
                        imscription: next_imscription,
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
    boundary: Tape,
    span: Tape,
    lattice_cell: Tape,
    lehman_multiplier: Option<Tape>,
) -> Result<Descent, String> {
    let word: Vec<Mark> = closed_word.chars().collect();
    if verdict(&word) != 'T' {
        return Err(String::from("closed dialectic imscription is not FOUR=T"));
    }
    let imscription = Imscription::active(&n, boundary, span, &word);
    if !imscription.relation_is_live_for(&n, &word) {
        return Err(String::from("closed dialectic imscription lost its terminal r/w/x relation"));
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

/// Walk directly from the boundary imscribed by the current object. The span
/// itself is the countdown tape: each visited cell consumes one mark-number step.
/// On an open result, the returned boundary is the first unwalked lattice point
/// and is written into the next imscription. No earlier cell is reconstructed or replayed.
fn fermat_lattice_from(
    n: &[Mark],
    start_boundary: &[Mark],
    base_cell: &[Mark],
    span: &[Mark],
) -> LatticeResult {
    let one = tape_u64(1);
    let mut a = trim(start_boundary.to_vec());
    let mut cell = trim(base_cell.to_vec());
    let mut remaining = trim(span.to_vec());

    while !zero(&remaining) {
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
        remaining = sub(&remaining, &one);
    }

    LatticeResult::Open { boundary: a }
}

enum LehmanResult {
    Closed { p: Tape, q: Tape, cell: Tape },
    Open,
}

/// Execute one complete Lehman-multiplier imscription. The boundary is k. The
/// local a-lattice begins at ceil(sqrt(4*k*N)) and consumes the exact span tape
/// carried by the current imscription. A B writes k+1 into the next imscription;
/// no earlier multiplier is replayed and no host-sized loop count is reconstructed.
fn lehman_multiplier(n: &[Mark], k: &[Mark], span: &[Mark]) -> LehmanResult {
    let one = tape_u64(1);
    let four_kn = mul(&tape_u64(4), &mul(k, n));
    let mut a = isqrt(&four_kn);
    let mut cell = tape_u64(0);
    let mut remaining = trim(span.to_vec());
    if cmp(&mul(&a, &a), &four_kn) == Ordering::Less {
        a = add(&a, &one);
    }

    while !zero(&remaining) {
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
        remaining = sub(&remaining, &one);
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

// Host-sized conversions below are confined to in-memory blob framing. They do
// not mediate N, boundary, span, lattice-cell, multiplier, or r/w/x semantics.
fn usize_to_tape(mut value: usize) -> Tape {
    if value == 0 {
        return vec![EVALT];
    }
    let mut out = Vec::new();
    while value != 0 {
        out.push(if value & 1 == 1 { EVALF } else { EVALT });
        value >>= 1;
    }
    out
}

fn tape_to_usize(tape: &[Mark]) -> Option<usize> {
    if tape.is_empty() {
        return None;
    }
    let mut value = 0usize;
    for (bit, &mark) in tape.iter().enumerate() {
        match mark {
            EVALT => {}
            EVALF => {
                if bit >= usize::BITS as usize {
                    return None;
                }
                value |= 1usize.checked_shl(bit as u32)?;
            }
            _ => return None,
        }
    }
    Some(value)
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

fn push_blob(out: &mut Vec<Mark>, blob: &[Mark]) {
    push_tape(out, &usize_to_tape(blob.len()));
    out.extend_from_slice(blob);
}

fn read_blob(encoded: &[Mark], i: &mut usize, end: usize) -> Result<Vec<Mark>, String> {
    let len_tape = read_tape(encoded, i)?;
    let len = tape_to_usize(&len_tape)
        .ok_or_else(|| String::from("dialectic blob length overflows host address space"))?;
    let blob_end = i
        .checked_add(len)
        .ok_or_else(|| String::from("dialectic blob length overflow"))?;
    if blob_end > end {
        return Err(String::from("truncated dialectic blob"));
    }
    let blob = encoded[*i..blob_end].to_vec();
    *i = blob_end;
    Ok(blob)
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
