//! Native IMASM topology for the fixed-point spectral tower.
//!
//! This module is deliberately structural. It does not factor, decode a numeral
//! into a host integer, allocate a phase table, or simulate nesting with a Rust
//! recursion over modular residues. It records the actual matched `∈ ... ∋`
//! topology required by the tower-collapse construction.
//!
//! The canonical properly nested tower is
//!
//!   ⊢ ∈^d ≻⋈⊤≻⋈⊥≺⋈⊞⊙ ∋^d ⊡ ⊣
//!
//! with every inner frame contained by the same outer frame. The paired edges
//! are part of the program: a glued node word without these edges is not an
//! equivalent program.

use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::vox::{
    AFWD, AREV, CLINK, ENGAGR, EVALF, EVALT, FFUSE, FSPLIT, IFIX, IMSCRIB, TANCH,
    VINIT,
};

/// The non-frame body of the properly nested fixed-point tower.
pub const FIXED_POINT_NESTED_BODY: [char; 10] = [
    AFWD, CLINK, EVALT, AFWD, CLINK, EVALF, AREV, CLINK, ENGAGR, IMSCRIB,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FrameEdge {
    /// Position of the opening `∈` in the program word.
    pub open: usize,
    /// Position of the matching `∋` in the program word.
    pub close: usize,
    /// One-based depth of this frame. The outermost frame is depth 1.
    pub depth: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BankingAudit {
    pub deposits: usize,
    pub live_clears: usize,
    pub exposed: usize,
    pub banked: bool,
}

/// A properly nested IMASM tower. `word` is the node sequence; `frames` is the
/// missing wiring information that distinguishes two programs with the same
/// glued word but different topology.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NestedImasmTower {
    word: Vec<char>,
    frames: Vec<FrameEdge>,
    max_depth: usize,
}

fn is_imasm(mark: char) -> bool {
    matches!(
        mark,
        VINIT
            | TANCH
            | AFWD
            | AREV
            | FSPLIT
            | FFUSE
            | IMSCRIB
            | IFIX
            | CLINK
            | EVALT
            | EVALF
            | ENGAGR
    )
}

impl NestedImasmTower {
    /// Construct the tower at an explicit nesting depth. No numeric carrier is
    /// accepted here: this builds topology only.
    pub fn from_depth(depth: usize) -> Result<Self, &'static str> {
        if depth == 0 {
            return Err("fixed-point tower requires at least one enclosing frame");
        }

        let mut word = Vec::with_capacity(depth * 2 + FIXED_POINT_NESTED_BODY.len() + 3);
        word.push(VINIT);
        for _ in 0..depth {
            word.push(FSPLIT);
        }
        word.extend_from_slice(&FIXED_POINT_NESTED_BODY);
        for _ in 0..depth {
            word.push(FFUSE);
        }
        word.push(IFIX);
        word.push(TANCH);

        Self::from_word(word)
    }

    /// The construction's documented `d = m - 3` scale/depth relation.
    pub fn from_scale(m: usize) -> Result<Self, &'static str> {
        let depth = m
            .checked_sub(3)
            .ok_or("fixed-point scale must be at least four")?;
        Self::from_depth(depth)
    }

    fn from_word(word: Vec<char>) -> Result<Self, &'static str> {
        if word.first() != Some(&VINIT) || word.last() != Some(&TANCH) {
            return Err("fixed-point tower must run from VINIT to TANCH");
        }
        if word.iter().any(|&mark| !is_imasm(mark)) {
            return Err("fixed-point tower contains a non-IMASM mark");
        }

        let mut stack: Vec<(usize, usize)> = Vec::new();
        let mut frames = Vec::new();
        let mut max_depth = 0usize;
        for (at, &mark) in word.iter().enumerate() {
            match mark {
                FSPLIT => {
                    let depth = stack.len() + 1;
                    max_depth = max_depth.max(depth);
                    stack.push((at, depth));
                }
                FFUSE => {
                    let (open, depth) = stack.pop().ok_or("unmatched IMASM fuse")?;
                    frames.push(FrameEdge {
                        open,
                        close: at,
                        depth,
                    });
                }
                _ => {}
            }
        }
        if !stack.is_empty() {
            return Err("unmatched IMASM split");
        }
        frames.sort_by_key(|edge| edge.depth);

        let tower = Self {
            word,
            frames,
            max_depth,
        };
        tower.validate_fixed_point_shape()?;
        Ok(tower)
    }

    fn validate_fixed_point_shape(&self) -> Result<(), &'static str> {
        if self.frames.len() != self.max_depth {
            return Err("fixed-point tower frames are not a single nested chain");
        }
        for expected in 1..=self.max_depth {
            let edge = self
                .frames
                .get(expected - 1)
                .ok_or("missing fixed-point frame depth")?;
            if edge.depth != expected {
                return Err("fixed-point tower skips a frame depth");
            }
            if expected > 1 {
                let outer = self.frames[expected - 2];
                if !(outer.open < edge.open && edge.close < outer.close) {
                    return Err("fixed-point frames are not properly nested");
                }
            }
        }

        let reversal = self
            .word
            .iter()
            .position(|&mark| mark == AREV)
            .ok_or("fixed-point tower has no reversal")?;
        if self
            .frames
            .iter()
            .any(|edge| !(edge.open < reversal && reversal < edge.close))
        {
            return Err("fixed-point reversal is exposed outside an enclosing bank");
        }
        Ok(())
    }

    pub fn word(&self) -> &[char] {
        &self.word
    }

    pub fn word_string(&self) -> String {
        self.word.iter().collect()
    }

    pub fn frames(&self) -> &[FrameEdge] {
        &self.frames
    }

    pub fn max_depth(&self) -> usize {
        self.max_depth
    }

    /// Structural banking invariant for the canonical body. The three resident
    /// deposits are protected by every enclosing frame when the single live
    /// reversal fires, so none is exposed.
    pub fn banking_audit(&self) -> BankingAudit {
        let reversal = self.word.iter().position(|&mark| mark == AREV);
        let banked = reversal
            .map(|at| {
                self.frames
                    .iter()
                    .all(|edge| edge.open < at && at < edge.close)
            })
            .unwrap_or(false);
        BankingAudit {
            deposits: 3,
            live_clears: 1,
            exposed: if banked { 0 } else { 1 },
            banked,
        }
    }

    /// Apply the documented dissolution rule to the topology: all matched
    /// split/fuse pairs contained by the outer pair dissolve to the same one-
    /// frame structural representative. A dangling split/fuse can never reach
    /// this method because construction fails closed above.
    pub fn dissolved_word(&self) -> Vec<char> {
        let mut out = Vec::with_capacity(FIXED_POINT_NESTED_BODY.len() + 5);
        out.push(VINIT);
        out.push(FSPLIT);
        out.extend_from_slice(&FIXED_POINT_NESTED_BODY);
        out.push(FFUSE);
        out.push(IFIX);
        out.push(TANCH);
        out
    }
}

/// An opaque IMASM numeral value. The value remains a glyph word; this type
/// intentionally provides no conversion to `u64`, `u128`, limbs, or a host
/// numeric representation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImasmNumeralWord(String);

impl ImasmNumeralWord {
    pub fn checked(word: &str) -> Result<Self, &'static str> {
        let marks: Vec<char> = word.chars().collect();
        if marks.len() < 4 || marks.first() != Some(&VINIT) || marks.last() != Some(&TANCH) {
            return Err("IMASM numeral must run from VINIT to TANCH");
        }
        if marks.iter().any(|&mark| !is_imasm(mark)) {
            return Err("IMASM numeral contains a non-IMASM mark");
        }
        if marks.iter().all(|&mark| mark != EVALT && mark != EVALF) {
            // The canonical zero word is the only numeral with no payload bit.
            if marks.as_slice() != [VINIT, IMSCRIB, IFIX, TANCH] {
                return Err("IMASM numeral contains no numeral payload");
            }
        }
        Ok(Self(word.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nested_tower_records_real_frame_edges() {
        let tower = NestedImasmTower::from_depth(13).unwrap();
        assert_eq!(tower.max_depth(), 13);
        assert_eq!(tower.frames().len(), 13);
        assert_eq!(tower.frames()[0].open, 1);
        assert_eq!(tower.frames()[0].depth, 1);
        assert!(tower.frames().windows(2).all(|pair| {
            pair[0].open < pair[1].open && pair[1].close < pair[0].close
        }));
    }

    #[test]
    fn documented_scales_keep_the_reversal_banked() {
        for m in 16..=100 {
            let tower = NestedImasmTower::from_scale(m).unwrap();
            assert_eq!(tower.max_depth(), m - 3);
            assert_eq!(
                tower.banking_audit(),
                BankingAudit {
                    deposits: 3,
                    live_clears: 1,
                    exposed: 0,
                    banked: true,
                }
            );
        }
    }

    #[test]
    fn dissolution_is_depth_invariant() {
        let canonical = NestedImasmTower::from_depth(1).unwrap().dissolved_word();
        for depth in [1usize, 2, 13, 61, 97] {
            let tower = NestedImasmTower::from_depth(depth).unwrap();
            assert_eq!(tower.dissolved_word(), canonical);
        }
    }

    #[test]
    fn imasm_value_stays_an_opaque_glyph_word() {
        let value = ImasmNumeralWord::checked("⊢≻⋈∈⊥∋≻⋈∈⊤∋⊙⊡⊣").unwrap();
        assert_eq!(value.as_str(), "⊢≻⋈∈⊥∋≻⋈∈⊤∋⊙⊡⊣");
        assert!(ImasmNumeralWord::checked("12345").is_err());
    }
}
