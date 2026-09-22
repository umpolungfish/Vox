//! Compile-time collapse of the banked fixed-point word vessel.
//!
//! The source implementation in G-mOMonadOS collapses nested marks but leaves a
//! runtime frontier loop. This module draws the boundary in the other place:
//! the compiler consumes that frontier while every value (including the landing
//! count) remains an IMASM numeral word. The compiled membrane retains only N,
//! the one-frame dissolved program, and the derived landing/preimage. It never
//! stores p or q.
//!
//! Runtime `consume()` therefore has no candidate/order/frontier loop. It applies
//! the already-compiled landing once, runs the word arithmetic at that resident
//! fixed point, and consumes N + membrane into the factor pair.

use core::cmp::Ordering;

use crate::fixed_point_imasm::{ImasmNumeralWord, NestedImasmTower};
use crate::fixed_point_word_arithmetic::{
    add, compare, increment, multiply, one, square_gap, square_root_exact, sqrt_ceil, subtract,
    zero,
};
use crate::fixed_point_word_vessel::{FixedPointWordFactorPair, FixedPointWordVessel};
use crate::vox::{AFWD, FFUSE, FSPLIT};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CollapsedFixedPointWordMembrane {
    n: ImasmNumeralWord,
    landing: ImasmNumeralWord,
    word: alloc::vec::Vec<char>,
}

fn tower_for_n(n: &ImasmNumeralWord) -> Result<NestedImasmTower, &'static str> {
    let width = n
        .as_str()
        .chars()
        .filter(|&mark| mark == AFWD)
        .count()
        .max(4);
    NestedImasmTower::from_scale(width)
}

impl CollapsedFixedPointWordMembrane {
    /// Compiler-side frontier consumption. There is deliberately no host
    /// integer step counter and no cap: `landing` itself is the resident IMASM
    /// count, incremented by the word adder while the search remains inside the
    /// already-banked vessel.
    pub fn compile(mut vessel: FixedPointWordVessel) -> Result<Self, &'static str> {
        let n = vessel.n().clone();
        let word = vessel.tower().dissolved_word();
        let mut landing = zero();

        loop {
            if vessel.fixed_pair()?.is_some() {
                return Ok(Self { n, landing, word });
            }
            vessel = vessel.advance_once()?;
            landing = increment(&landing)?;
        }
    }

    /// Reconstitute a compiler-produced membrane from its two glyph values.
    /// This is the runtime artifact constructor: N and landing are both IMASM
    /// words; p and q are not accepted.
    pub fn from_compiled_words(
        n: ImasmNumeralWord,
        landing: ImasmNumeralWord,
    ) -> Result<Self, &'static str> {
        if compare(&n, &one())? != Ordering::Greater {
            return Err("collapsed membrane requires N > 1");
        }
        let tower = tower_for_n(&n)?;
        if !tower.banking_audit().banked {
            return Err("collapsed membrane source topology is not banked");
        }
        Ok(Self {
            n,
            landing,
            word: tower.dissolved_word(),
        })
    }

    pub fn landing(&self) -> &ImasmNumeralWord {
        &self.landing
    }

    pub fn word(&self) -> &[char] {
        &self.word
    }

    pub fn n(&self) -> &ImasmNumeralWord {
        &self.n
    }

    /// Consume the compiled membrane in one shot. There is no AFWD frontier
    /// iteration here: the landing is applied by one word addition before the
    /// fixed gap is read.
    pub fn consume(self) -> Result<FixedPointWordFactorPair, &'static str> {
        // A valid collapse leaves exactly the outer frame. Reject a malformed
        // compiled artifact rather than silently treating a flat word as nested.
        if self.word.iter().filter(|&&mark| mark == FSPLIT).count() != 1
            || self.word.iter().filter(|&&mark| mark == FFUSE).count() != 1
        {
            return Err("compiled fixed-point membrane did not dissolve to one frame");
        }

        let seed = sqrt_ceil(&self.n)?;
        let a = add(&seed, &self.landing)?;
        let gap = square_gap(&a, &self.n)?;
        let b = square_root_exact(&gap)?
            .ok_or("compiled fixed-point landing does not expose a square gap")?;
        let p = subtract(&a, &b)?;
        let q = add(&a, &b)?;

        if compare(&p, &one())? != Ordering::Greater
            || compare(&q, &self.n)? != Ordering::Less
        {
            return Err("compiled fixed-point landing exposed a trivial boundary");
        }
        if compare(&multiply(&p, &q)?, &self.n)? != Ordering::Equal {
            return Err("compiled fixed-point landing does not reconstruct N");
        }

        Ok(FixedPointWordFactorPair { p, q })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compilation_consumes_frontier_and_retains_only_the_landing() {
        let n = ImasmNumeralWord::checked(
            include_str!("../tests/baked/fixed_point_semiprime_40.word").trim(),
        )
        .unwrap();
        let membrane = CollapsedFixedPointWordMembrane::compile(
            FixedPointWordVessel::board(n.clone()).unwrap(),
        )
        .unwrap();

        let two = increment(&one()).unwrap();
        assert_eq!(compare(membrane.landing(), &two).unwrap(), Ordering::Equal);
        assert_eq!(membrane.word().iter().filter(|&&m| m == FSPLIT).count(), 1);
        assert_eq!(membrane.word().iter().filter(|&&m| m == FFUSE).count(), 1);
        assert_eq!(membrane.n(), &n);
    }

    #[test]
    fn collapsed_real_semiprime_consumes_to_factors_without_runtime_walk() {
        let n = ImasmNumeralWord::checked(
            include_str!("../tests/baked/fixed_point_semiprime_40.word").trim(),
        )
        .unwrap();
        let original = n.clone();
        let compiled = CollapsedFixedPointWordMembrane::compile(
            FixedPointWordVessel::board(n).unwrap(),
        )
        .unwrap();
        let landing = compiled.landing().clone();

        // This reconstructs the target artifact from only N + compiler-derived
        // landing. No factor witness crosses this boundary.
        let runtime = CollapsedFixedPointWordMembrane::from_compiled_words(
            original.clone(),
            landing,
        )
        .unwrap();
        let pair = runtime.consume().unwrap();
        assert_eq!(
            compare(&multiply(&pair.p, &pair.q).unwrap(), &original).unwrap(),
            Ordering::Equal
        );
    }
}
