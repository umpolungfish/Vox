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

/// The lattice action decoded from the current IMASM word.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ImscriptionLattice {
    ShortFrontier,
    ExtendedFermat,
    Lehman,
}

impl ImscriptionLattice {
    fn open_word(self) -> &'static str {
        match self {
            Self::ShortFrontier => SHORT_FRONTIER_WORD,
            Self::ExtendedFermat => EXTENDED_FERMAT_WORD,
            Self::Lehman => LEHMAN_WORD,
        }
    }

    fn closed_word(self) -> &'static str {
        match self {
            Self::ShortFrontier => SHORT_FRONTIER_CLOSED_WORD,
            Self::ExtendedFermat => EXTENDED_FERMAT_CLOSED_WORD,
            Self::Lehman => LEHMAN_CLOSED_WORD,
        }
    }

    fn span(self) -> u64 {
        match self {
            Self::ShortFrontier => SHORT_FRONTIER_SPAN,
            Self::ExtendedFermat => EXTENDED_FERMAT_SPAN,
            Self::Lehman => LEHMAN_LOCAL_SPAN,
        }
    }

    fn unresolved_support(self) -> LaneSupport {
        match self {
            Self::ShortFrontier => BASE_SUPPORT,
            Self::ExtendedFermat => BASE_SUPPORT | SUPPORT_SHORT_FRONTIER,
            Self::Lehman => BASE_SUPPORT | SUPPORT_SHORT_FRONTIER | SUPPORT_EXTENDED_FERMAT,
        }
    }
}

/// Semantic execution descriptor decoded from the marks of one IMASM word.
///
/// The lattice is not selected by comparing the whole word to a Rust constant.
/// The grammar marks themselves identify the lattice, while FOUR determines
/// whether the word is an unresolved `B` imscription or a closing `T` one.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ImasmExecution {
    pub lattice: ImscriptionLattice,
    pub four: Mark,
    pub closed: bool,
}

/// Decode the current IMASM word into the lattice action it commands.
///
/// Supported kernels are:
/// `⊤≺⊥` short Fermat, `⊤⊞≺⊥` extended Fermat, and `⋈⊤⊥` Lehman.
/// A trailing `∋` before `⊡` is the closing form. FOUR must agree with that
/// grammar state (`B` unresolved, `T` closed), otherwise the word is rejected.
pub fn decode_imasm_execution(word: &[Mark]) -> Result<ImasmExecution, String> {
    if word.len() < 9
        || word.first().copied() != Some(VINIT)
        || word.get(1).copied() != Some(IMSCRIB)
        || word.get(2).copied() != Some('∈')
        || word.get(3).copied() != Some('≻')
        || word.get(word.len() - 2).copied() != Some('⊡')
        || word.last().copied() != Some(TANCH)
    {
        return Err(String::from("malformed dialectic IMASM execution framing"));
    }

    let mut kernel = &word[4..word.len() - 2];
    let closed = kernel.last().copied() == Some('∋');
    if closed {
        kernel = &kernel[..kernel.len() - 1];
    }

    let lattice = match kernel {
        ['⊤', '≺', '⊥'] => ImscriptionLattice::ShortFrontier,
        ['⊤', '⊞', '≺', '⊥'] => ImscriptionLattice::ExtendedFermat,
        ['⋈', '⊤', '⊥'] => ImscriptionLattice::Lehman,
        _ => return Err(String::from("unknown dialectic IMASM lattice kernel")),
    };

    let four = verdict(word);
    let expected_four = if closed { 'T' } else { 'B' };
    if four != expected_four {
        return Err(String::from(
            "dialectic IMASM grammar state does not agree with its FOUR verdict",
        ));
    }

    Ok(ImasmExecution {
        lattice,
        four,
        closed,
    })
}

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

/// The live imscription relation. Boundary, span and executable word are owned
/// only here, by the r/w/x endpoints; there is no surrounding mirror copy.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Imscription {
    pub rwx: ImscriptionRwx,
}

impl Imscription {
    fn active(n: &[Mark], boundary: Tape, span: Tape, word: &[Mark]) -> Self {
        Self {
            rwx: ImscriptionRwx::live(n, &boundary, &span, word),
        }
    }

    pub fn boundary(&self) -> &Tape {
        &self.rwx.write_boundary
    }

    pub fn span(&self) -> &Tape {
        &self.rwx.execute_span
    }

    pub fn word(&self) -> &[Mark] {
        &self.rwx.execute_word
    }

    pub fn relation_is_live_for(&self, n: &[Mark]) -> bool {
        self.rwx.rights == IM_RWX
            && self.rwx.can_read()
            && self.rwx.can_write()
            && self.rwx.can_execute()
            && self.rwx.read_bulk == trim(n.to_vec())
            && !self.rwx.write_boundary.is_empty()
            && !self.rwx.execute_span.is_empty()
            && self
                .rwx
                .write_boundary
                .iter()
                .chain(self.rwx.execute_span.iter())
                .all(|&mark| mark == EVALT || mark == EVALF)
            && !self.rwx.execute_word.is_empty()
    }
}

/// One complete, restartable operator-space object.
///
/// `n` is the bulk. `imscription` owns the live boundary and executable space.
/// `support` is the distinction already embodied by this transformed whole object.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct DialecticObject {
    pub n: Tape,
    pub support: LaneSupport,
    pub imscription: Imscription,
}

impl DialecticObject {
    pub fn word(&self) -> &[Mark] {
        self.imscription.word()
    }

    pub fn boundary(&self) -> &Tape {
        self.imscription.boundary()
    }

    pub fn span(&self) -> &Tape {
        self.imscription.span()
    }
}

/// A terminal contraction: the imscription has closed around one factor pair
/// and is handed to the passive factor-carrier layer.
#[derive(Clone, PartialEq, Debug)]
pub struct DialecticClosure {
    pub carrier: FactorCarrier,
    pub support: LaneSupport,
    pub imscription: Imscription,
    /// Closing coordinate inside the current lattice, carried as a native numeral tape.
    pub lattice_cell: Tape,
    /// Present only when closure occurred in the Lehman multiplier lattice.
    pub lehman_multiplier: Option<Tape>,
}

impl DialecticClosure {
    pub fn word(&self) -> &[Mark] {
        self.imscription.word()
    }

    pub fn boundary(&self) -> &Tape {
        self.imscription.boundary()
    }

    pub fn span(&self) -> &Tape {
        self.imscription.span()
    }
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
        if !self.imscription.relation_is_live_for(&self.n) {
            return Err(String::from(
                "dialectic imscription r/w/x relation is not live for this whole object",
            ));
        }

        let execution = decode_imasm_execution(self.word())?;
        if execution.four != 'B' || execution.closed {
            return Err(String::from("unresolved dialectic imscription is not FOUR=B"));
        }

        let expected_support = execution.lattice.unresolved_support();
        if self.support != expected_support {
            return Err(String::from("dialectic imscription/support mismatch"));
        }

        let expected_span = tape_u64(execution.lattice.span());
        if cmp(self.span(), &expected_span) != Ordering::Equal {
            return Err(String::from(
                "dialectic imscription span does not match the current IMASM lattice",
            ));
        }

        match execution.lattice {
            ImscriptionLattice::ShortFrontier | ImscriptionLattice::ExtendedFermat => {
                let origin = fermat_origin(&self.n);
                let expected_boundary = if execution.lattice == ImscriptionLattice::ShortFrontier {
                    origin
                } else {
                    add(&origin, &tape_u64(SHORT_FRONTIER_SPAN))
                };
                if cmp(self.boundary(), &expected_boundary) != Ordering::Equal {
                    return Err(String::from(
                        "dialectic imscription boundary does not match the IMASM-selected Fermat space",
                    ));
                }
            }
            ImscriptionLattice::Lehman => {
                if cmp(self.boundary(), &tape_u64(1)) == Ordering::Less {
                    return Err(String::from(
                        "dialectic Lehman boundary must be a positive imscribed multiplier",
                    ));
                }
            }
        }

        Ok(())
    }

    /// Native support currently restored by this whole imscription.
    pub fn envelope(&self) -> RestoredSupport {
        restored_support(&[self.support])
    }

    /// Marks-only persistence of the complete current operator-space relation:
    ///
    /// `⊢ ⊙ ∈N∋ ∈rights[3]∋ ∈read-N∋ ∈write-boundary∋
    ///    ∈exec-span∋ ∈len(exec-word)∋ exec-word ∈support[6]∋ ⊣`
    ///
    /// Boundary, span and word occur exactly once: as endpoints of IMSCRIB.
    pub fn encode(&self) -> Vec<Mark> {
        let mut out = Vec::with_capacity(
            self.n.len()
                + self.imscription.rwx.read_bulk.len()
                + self.imscription.rwx.write_boundary.len()
                + self.imscription.rwx.execute_span.len()
                + self.imscription.rwx.execute_word.len()
                + RWX_BITS
                + SUPPORT_BITS
                + 18,
        );
        out.push(VINIT);
        out.push(IMSCRIB);
        push_tape(&mut out, &self.n);
        push_mask(&mut out, self.imscription.rwx.rights as u32, RWX_BITS);
        push_tape(&mut out, &self.imscription.rwx.read_bulk);
        push_tape(&mut out, &self.imscription.rwx.write_boundary);
        push_tape(&mut out, &self.imscription.rwx.execute_span);
        push_blob(&mut out, &self.imscription.rwx.execute_word);
        push_mask(&mut out, self.support, SUPPORT_BITS);
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
        let rights = read_mask(encoded, &mut i, RWX_BITS)? as u8;
        let read_bulk = read_tape(encoded, &mut i)?;
        let write_boundary = read_tape(encoded, &mut i)?;
        let execute_span = read_tape(encoded, &mut i)?;
        let execute_word = read_blob(encoded, &mut i, end)?;
        let support = read_mask(encoded, &mut i, SUPPORT_BITS)?;
        if i != end {
            return Err(String::from("trailing marks after dialectic imscription relation"));
        }
        let object = Self {
            n,
            support,
            imscription: Imscription {
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
    /// transformed space or close it around a factor pair. The current IMASM word
    /// is decoded into the lattice action it commands; the executor does not select
    /// a lattice by comparing the whole word to a hosted constant.
    pub fn descend(self) -> Result<Descent, String> {
        self.validate()?;

        let execution = decode_imasm_execution(self.word())?;
        let boundary = self.imscription.rwx.write_boundary.clone();
        let span = self.imscription.rwx.execute_span.clone();

        match execution.lattice {
            ImscriptionLattice::ShortFrontier => {
                let base_cell = tape_u64(0);
                match fermat_lattice_from(&self.n, &boundary, &base_cell, &span) {
                    LatticeResult::Closed {
                        p,
                        q,
                        cell,
                        boundary,
                    } => close(
                        self.n,
                        p,
                        q,
                        execution.lattice,
                        self.support | SUPPORT_SHORT_FRONTIER | SUPPORT_PRODUCT_BOUNDARY,
                        boundary,
                        span,
                        cell,
                        None,
                    ),
                    LatticeResult::Open { boundary } => {
                        let next_lattice = ImscriptionLattice::ExtendedFermat;
                        let next_word: Vec<Mark> = next_lattice.open_word().chars().collect();
                        let next_imscription = Imscription::active(
                            &self.n,
                            boundary,
                            tape_u64(next_lattice.span()),
                            &next_word,
                        );
                        let next = DialecticObject {
                            n: self.n,
                            support: self.support | SUPPORT_SHORT_FRONTIER,
                            imscription: next_imscription,
                        };
                        next.validate()?;
                        Ok(Descent::Continue(next))
                    }
                }
            }
            ImscriptionLattice::ExtendedFermat => {
                let base_cell = tape_u64(SHORT_FRONTIER_SPAN);
                match fermat_lattice_from(&self.n, &boundary, &base_cell, &span) {
                    LatticeResult::Closed {
                        p,
                        q,
                        cell,
                        boundary,
                    } => close(
                        self.n,
                        p,
                        q,
                        execution.lattice,
                        self.support | SUPPORT_EXTENDED_FERMAT | SUPPORT_PRODUCT_BOUNDARY,
                        boundary,
                        span,
                        cell,
                        None,
                    ),
                    LatticeResult::Open { .. } => {
                        let next_lattice = ImscriptionLattice::Lehman;
                        let next_word: Vec<Mark> = next_lattice.open_word().chars().collect();
                        let next_imscription = Imscription::active(
                            &self.n,
                            tape_u64(1),
                            tape_u64(next_lattice.span()),
                            &next_word,
                        );
                        let next = DialecticObject {
                            n: self.n,
                            support: self.support | SUPPORT_EXTENDED_FERMAT,
                            imscription: next_imscription,
                        };
                        next.validate()?;
                        Ok(Descent::Continue(next))
                    }
                }
            }
            ImscriptionLattice::Lehman => {
                let k = boundary;
                match lehman_multiplier(&self.n, &k, &span) {
                    LehmanResult::Closed { p, q, cell } => close(
                        self.n,
                        p,
                        q,
                        execution.lattice,
                        self.support | SUPPORT_DEEP_ARM | SUPPORT_PRODUCT_BOUNDARY,
                        k.clone(),
                        span,
                        cell,
                        Some(k),
                    ),
                    LehmanResult::Open => {
                        let next_word: Vec<Mark> = execution.lattice.open_word().chars().collect();
                        let next_imscription = Imscription::active(
                            &self.n,
                            add(&k, &tape_u64(1)),
                            span,
                            &next_word,
                        );
                        let next = DialecticObject {
                            n: self.n,
                            support: self.support,
                            imscription: next_imscription,
                        };
                        next.validate()?;
                        Ok(Descent::Continue(next))
                    }
                }
            }
        }
    }
}

fn close(
    n: Tape,
    p: Tape,
    q: Tape,
    lattice: ImscriptionLattice,
    support: LaneSupport,
    boundary: Tape,
    span: Tape,
    lattice_cell: Tape,
    lehman_multiplier: Option<Tape>,
) -> Result<Descent, String> {
    let word: Vec<Mark> = lattice.closed_word().chars().collect();
    let execution = decode_imasm_execution(&word)?;
    if execution.lattice != lattice || execution.four != 'T' || !execution.closed {
        return Err(String::from(
            "closed dialectic IMASM word does not decode to the closing lattice",
        ));
    }
    let imscription = Imscription::active(&n, boundary, span, &word);
    if !imscription.relation_is_live_for(&n) {
        return Err(String::from(
            "closed dialectic imscription lost its terminal r/w/x relation",
        ));
    }
    let trace = encode_trace(&[GStep {
        repr: '⋈',
        judgment: M_T,
        recognised: M_T,
        next: M_FIX,
        applied_word: word,
    }]);
    let carrier = FactorCarrier::new(n, p, q, trace)?;
    Ok(Descent::Closed(DialecticClosure {
        carrier,
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
/// itself is the countdown tape: each visited cell consumes one numeral step.
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
/// local a-lattice consumes the exact span tape carried by the relation.
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
