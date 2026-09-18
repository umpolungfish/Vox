//! carrier.rs — one mark carrier. A representation tag, a Belnap judgment, and a
//! probe pair are admissible SHAPES of the same value, not three host enums.
use alloc::vec::Vec;

pub type Mark = char;

/// GValue — the single carrier. A single mark is a tag or a judgment; a two-mark
/// value is a probe. The arity is the only difference, and the marks are the content.
#[derive(Clone, PartialEq, Debug)]
pub struct GValue { pub marks: Vec<Mark> }

impl GValue {
    pub fn new(marks: &[Mark]) -> GValue { GValue { marks: marks.to_vec() } }
    pub fn single(m: Mark) -> GValue { GValue { marks: alloc::vec![m] } }
    pub fn arity(&self) -> usize { self.marks.len() }
    pub fn mark0(&self) -> Option<Mark> { self.marks.first().copied() }
}


/// The carrier's grammars: the single-mark values that ARE a representation tag or a
/// judgment, and the one mark the two grammars share.
pub const REPR_MARKS: [Mark; 3] = ['⊢', '⊣', '⋈'];
pub const JUDG_MARKS: [Mark; 4] = ['⊤', '⊥', '⊞', '⊙'];
/// ⊙ — judgment N and wildcard source are the SAME mark: no distinction = no
/// restriction on distinction. Which one it is, is decided by SLOT, never by a host case.
pub const ANY_MARK: Mark = '⊙';

/// Anything grammatical exposes a GValue.
pub trait GrammarObject { fn gvalue(&self) -> &GValue; }
impl GrammarObject for GValue { fn gvalue(&self) -> &GValue { self } }

pub fn is_repr(v: &GValue) -> bool { v.arity() == 1 && v.mark0().map(|m| REPR_MARKS.contains(&m)).unwrap_or(false) }
pub fn is_judgment(v: &GValue) -> bool { v.arity() == 1 && v.mark0().map(|m| JUDG_MARKS.contains(&m)).unwrap_or(false) }
/// A source slot is a representation tag OR the wildcard ⊙.
pub fn is_source(v: &GValue) -> bool { is_repr(v) || v.mark0() == Some(ANY_MARK) }
pub fn is_probe(v: &GValue) -> bool { v.arity() == 2 }

/// probe(judgment, source) — the two-mark value, built by composition, no conversion.
pub fn probe(j: &GValue, source: &GValue) -> GValue {
    let mut m = Vec::new();
    if let Some(a) = j.mark0() { m.push(a); }
    if let Some(b) = source.mark0() { m.push(b); }
    GValue { marks: m }
}
