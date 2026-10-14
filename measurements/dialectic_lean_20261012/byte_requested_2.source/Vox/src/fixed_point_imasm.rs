//! Native IMASM topology for the fixed-point spectral tower.
//!
//! This module is deliberately structural. It does not factor, decode a numeral
//! into a host integer, allocate a phase table, or simulate nesting with a Rust
//! recursion over modular residues. It records the actual matched `∈ ... ∋`
//! topology required by the tower-collapse construction.
//!
//! The older frame tower remains available for the repaired fixed-point word.
//! Hypernesting is represented separately because a hypernest level is a whole
//! carrier, not merely another split frame.  Its canonical recursive shape is
//!
//!   H₁ = ⊢⊙∈≻⊤≺⊥⋈∋⊡⊣
//!   Hₙ₊₁ = ⊢⊙∈ Hₙ ⋈∋⊡⊣
//!
//! Thus every embedding contributes exactly one `⊡` winding while the single
//! innermost `≺` remains banked through every enclosing `∈ ... ∋` ancestry edge.

use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::vox::{
    verdict, AFWD, AREV, CLINK, ENGAGR, EVALF, EVALT, FFUSE, FSPLIT, IFIX, IMSCRIB,
    TANCH, VINIT,
};

/// The non-frame body of the properly nested fixed-point tower.
pub const FIXED_POINT_NESTED_BODY: [char; 10] = [
    AFWD, CLINK, EVALT, AFWD, CLINK, EVALF, AREV, CLINK, ENGAGR, IMSCRIB,
];

/// The only payload that owns a live clear in a hypernest.
pub const HYPERNEST_PAYLOAD: [char; 5] = [AFWD, EVALT, AREV, EVALF, CLINK];
const HYPERNEST_PREFIX: [char; 3] = [VINIT, IMSCRIB, FSPLIT];
const HYPERNEST_INNER_SUFFIX: [char; 3] = [FFUSE, IFIX, TANCH];
const HYPERNEST_OUTER_SUFFIX: [char; 4] = [CLINK, FFUSE, IFIX, TANCH];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FrameEdge {
    pub open: usize,
    pub close: usize,
    pub depth: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BankingAudit {
    pub deposits: usize,
    pub live_clears: usize,
    pub exposed: usize,
    pub banked: bool,
}

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

fn frame_edges(word: &[char]) -> Result<(Vec<FrameEdge>, usize), &'static str> {
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
    Ok((frames, max_depth))
}

#[cfg(test)]
pub(crate) fn is_imasm_for_test(mark: char) -> bool {
    is_imasm(mark)
}

impl NestedImasmTower {
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

        let (frames, max_depth) = frame_edges(&word)?;
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

/// Instrument reading for one executed hypernest.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HypernestAudit {
    /// One `⊡` per complete carrier level.
    pub winding: usize,
    /// The unique innermost `≺`.
    pub live_clears: usize,
    /// A clear outside at least one enclosing ancestry region.
    pub exposed: usize,
    /// The unique live clear is inside every enclosing frame.
    pub banked: bool,
    /// SIXTEEN_3 closure of the executed word.
    pub closed: bool,
}

/// A carrier holding a carrier holding ... holding the one live payload.
///
/// Storage and construction are O(depth): every level is emitted once.  No
/// payload branch is copied exponentially and no residue orbit is enumerated.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HypernestedImasmCarrier {
    word: Vec<char>,
    frames: Vec<FrameEdge>,
    depth: usize,
}

impl HypernestedImasmCarrier {
    pub fn from_depth(depth: usize) -> Result<Self, &'static str> {
        if depth == 0 {
            return Err("hypernest requires at least one complete carrier");
        }

        // H_d = prefix^d · payload · inner_suffix · outer_suffix^(d-1).
        // This is exactly the recursive carrier containment, emitted linearly.
        let mut word = Vec::with_capacity(
            depth
                .checked_mul(7)
                .and_then(|n| n.checked_add(4))
                .ok_or("hypernest word length overflow")?,
        );
        for _ in 0..depth {
            word.extend_from_slice(&HYPERNEST_PREFIX);
        }
        word.extend_from_slice(&HYPERNEST_PAYLOAD);
        word.extend_from_slice(&HYPERNEST_INNER_SUFFIX);
        for _ in 1..depth {
            word.extend_from_slice(&HYPERNEST_OUTER_SUFFIX);
        }

        let (frames, max_depth) = frame_edges(&word)?;
        if max_depth != depth || frames.len() != depth {
            return Err("hypernest ancestry depth does not match carrier depth");
        }
        let carrier = Self { word, frames, depth };
        carrier.validate()?;
        Ok(carrier)
    }

    fn validate(&self) -> Result<(), &'static str> {
        if self.word.iter().any(|&mark| !is_imasm(mark)) {
            return Err("hypernest contains a non-IMASM mark");
        }
        if self.word.iter().filter(|&&mark| mark == VINIT).count() != self.depth
            || self.word.iter().filter(|&&mark| mark == TANCH).count() != self.depth
            || self.word.iter().filter(|&&mark| mark == IMSCRIB).count() != self.depth
            || self.word.iter().filter(|&&mark| mark == FSPLIT).count() != self.depth
            || self.word.iter().filter(|&&mark| mark == FFUSE).count() != self.depth
            || self.word.iter().filter(|&&mark| mark == IFIX).count() != self.depth
        {
            return Err("hypernest level is not a complete carrier");
        }
        if self.word.iter().filter(|&&mark| mark == AREV).count() != 1 {
            return Err("hypernest must contain exactly one live clear");
        }
        if self.frames.iter().enumerate().any(|(i, edge)| {
            edge.depth != i + 1
                || (i > 0
                    && !(self.frames[i - 1].open < edge.open
                        && edge.close < self.frames[i - 1].close))
        }) {
            return Err("hypernest is juxtaposed instead of ancestry-contained");
        }
        let audit = self.audit();
        if !audit.banked || audit.exposed != 0 {
            return Err("hypernest live clear escaped an enclosing bank");
        }
        if audit.winding != self.depth {
            return Err("hypernest winding does not equal carrier depth");
        }
        if !audit.closed {
            return Err("hypernest carrier does not close");
        }
        Ok(())
    }

    pub fn word(&self) -> &[char] { &self.word }
    pub fn word_string(&self) -> String { self.word.iter().collect() }
    pub fn frames(&self) -> &[FrameEdge] { &self.frames }
    pub fn depth(&self) -> usize { self.depth }

    /// Integer winding is measured, not inferred: count the executed `⊡` marks.
    pub fn winding(&self) -> usize {
        self.word.iter().filter(|&&mark| mark == IFIX).count()
    }

    pub fn audit(&self) -> HypernestAudit {
        let clear_positions: Vec<usize> = self
            .word
            .iter()
            .enumerate()
            .filter_map(|(i, &mark)| (mark == AREV).then_some(i))
            .collect();
        let live_clears = clear_positions.len();
        let banked = live_clears == 1
            && self
                .frames
                .iter()
                .all(|edge| edge.open < clear_positions[0] && clear_positions[0] < edge.close);
        HypernestAudit {
            winding: self.winding(),
            live_clears,
            exposed: if banked { 0 } else { live_clears },
            banked,
            closed: verdict(&self.word) == 'T',
        }
    }

    /// Interior of the ancestry region at a one-based depth.
    pub fn region_interior(&self, depth: usize) -> Option<&[char]> {
        let edge = self.frames.get(depth.checked_sub(1)?)?;
        Some(&self.word[edge.open + 1..edge.close])
    }
}

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
    use crate::vox::pairing;

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
    fn hypernest_pairing_is_ancestry_containment() {
        let inner = HypernestedImasmCarrier::from_depth(1).unwrap();
        let outer = HypernestedImasmCarrier::from_depth(2).unwrap();
        let mut expected = inner.word().to_vec();
        expected.push(CLINK);
        assert_eq!(outer.region_interior(1).unwrap(), expected.as_slice());

        let (_regions, unanswered, unopened) = pairing(outer.word());
        assert!(unanswered.is_empty());
        assert!(unopened.is_empty());
    }

    #[test]
    fn hypernest_winding_is_exactly_depth() {
        for depth in 1usize..=32 {
            let carrier = HypernestedImasmCarrier::from_depth(depth).unwrap();
            assert_eq!(carrier.depth(), depth);
            assert_eq!(carrier.winding(), depth);
            assert_eq!(carrier.audit().winding, depth);
        }
    }

    #[test]
    fn hypernest_banked_cost_is_flat() {
        for depth in [1usize, 2, 3, 8, 32] {
            let audit = HypernestedImasmCarrier::from_depth(depth).unwrap().audit();
            assert_eq!(audit.live_clears, 1);
            assert_eq!(audit.exposed, 0);
            assert!(audit.banked);
        }
    }

    #[test]
    fn first_carrier_closes_and_every_embedding_stays_closed() {
        assert_eq!(verdict(&HYPERNEST_PAYLOAD), 'N');
        for depth in 1usize..=32 {
            let carrier = HypernestedImasmCarrier::from_depth(depth).unwrap();
            assert_eq!(verdict(carrier.word()), 'T');
            assert!(carrier.audit().closed);
        }
    }

    #[test]
    fn hypernest_storage_is_linear_in_winding() {
        for depth in [1usize, 2, 3, 16, 64] {
            let carrier = HypernestedImasmCarrier::from_depth(depth).unwrap();
            assert_eq!(carrier.word().len(), 7 * depth + 4);
        }
    }

    #[test]
    fn imasm_value_stays_an_opaque_glyph_word() {
        let value = ImasmNumeralWord::checked("⊢≻⋈∈⊥∋≻⋈∈⊤∋⊙⊡⊣").unwrap();
        assert_eq!(value.as_str(), "⊢≻⋈∈⊥∋≻⋈∈⊤∋⊙⊡⊣");
        assert!(ImasmNumeralWord::checked("12345").is_err());
    }
}
