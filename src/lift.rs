//! The lift, once.
//!
//! Every lane in V⊙x used to carry its own copy of the same loop: open with
//! VINIT, then for each step emit a merge if flow joins there and the glyph the
//! step lifts to. EVM, WASM, CPython and x86 each wrote it out again, and each
//! wrote its opcode judgements as Rust `match` arms buried in the reader that
//! parsed the container. Four copies of one law is four chances for them to
//! disagree, and none of the four is the part that differs between instruction
//! sets.
//!
//! What actually differs is two things, and they are separable. A container has
//! to be parsed into steps, which is irreducibly per-format. And each step's
//! name has to be judged into the twelve, which is a TABLE — data, not code.
//! This module holds the loop and the table type; the readers keep only the
//! parsing.
//!
//! The genetic lane is deliberately not here. A transcript is a linear read
//! with one terminal and no branching, so it has no merges to place and no
//! control flow to close over; putting it through a control-flow lift would be
//! forcing a shape it does not have.

use alloc::vec::Vec;
use crate::vox::{VINIT, FFUSE};

/// One step of a process: what it is called, what it operates on, and whether
/// control flow converges on it from more than one predecessor.
///
/// `joined` is computed by the reader because the evidence for it is
/// format-shaped — a predecessor count over jump targets on an addressed
/// instruction set, a control stack on a nested one like WASM. Where the
/// evidence lives is per-format; what it means is not.
pub struct Step<'a> {
    pub name: &'a str,
    pub ops: &'a str,
    pub joined: bool,
}

impl<'a> Step<'a> {
    pub fn new(name: &'a str, joined: bool) -> Self {
        Step { name, ops: "", joined }
    }
}

/// One judgement: the steps matching `pat` lift to `glyph`.
///
/// `prefix` matches a family by its stem, which is how real instruction sets
/// name their variants — `POP_JUMP_IF_FALSE` and `POP_JUMP_IF_TRUE` are one
/// judgement, and so are every `RETURN_*` and every `CALL*`.
pub struct Rule<'a> {
    pub pat: &'a str,
    pub glyph: char,
    pub prefix: bool,
}

impl<'a> Rule<'a> {
    pub const fn exact(pat: &'a str, glyph: char) -> Self {
        Rule { pat, glyph, prefix: false }
    }
    pub const fn stem(pat: &'a str, glyph: char) -> Self {
        Rule { pat, glyph, prefix: true }
    }
    fn hits(&self, name: &str) -> bool {
        if self.prefix { name.starts_with(self.pat) } else { name == self.pat }
    }
}

/// An instruction set, as the only thing V⊙x needs to know about one: its name
/// and how its steps lift into the twelve.
///
/// Rules are tried in order and the first hit wins, so a table reads as the
/// if-else chain it replaces. A step no rule hits lifts to nothing, which is
/// how a lane stays silent about stack shuffling and other steps that move no
/// control: silence is a judgement, not a gap.
pub struct Isa<'a> {
    pub name: &'a str,
    pub rules: &'a [Rule<'a>],
}

impl<'a> Isa<'a> {
    pub fn glyph(&self, name: &str) -> Option<char> {
        self.rules.iter().find(|r| r.hits(name)).map(|r| r.glyph)
    }
}

/// The lift: steps in, a word in the twelve out.
///
/// `glyph` decides what a step lifts to. A table supplies it for the bytecode
/// lanes; x86 supplies a closure instead, because its judgement reads operands
/// as well as the mnemonic — a `call` through a register is not the `call` to a
/// label, and only the operand says which. That is a richer judgement, not a
/// different law, so it goes through the same loop.
pub fn lift_steps<F>(steps: &[Step], glyph: F) -> Vec<char>
where
    F: Fn(&Step) -> Option<char>,
{
    let mut word = alloc::vec![VINIT];
    for s in steps {
        if s.joined {
            word.push(FFUSE);
        }
        if let Some(g) = glyph(s) {
            word.push(g);
        }
    }
    word
}

/// The lift with a table, which is every lane that judges by name alone.
pub fn lift_with(isa: &Isa, steps: &[Step]) -> Vec<char> {
    lift_steps(steps, |s| isa.glyph(s.name))
}
