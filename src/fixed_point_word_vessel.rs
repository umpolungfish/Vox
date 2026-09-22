//! Fixed-point factor vessel over native IMASM numeral words.
//!
//! The vessel owns N, the properly nested tower, the Fermat midpoint seed `a`,
//! and the gap `a²-N`. Every numeric resident is an `ImasmNumeralWord`; the
//! arithmetic layer never folds those values into machine limbs.
//!
//! This module intentionally exposes only one explicit candidate advance. It
//! does not hide a factor/order/search loop. The compiler-side tower collapse is
//! a separate rung; until that exists, callers cannot accidentally relabel a
//! sequential walk as a collapsed membrane.

use core::cmp::Ordering;

use crate::fixed_point_imasm::{ImasmNumeralWord, NestedImasmTower};
use crate::fixed_point_word_arithmetic::{
    add, compare, increment, multiply, one, square_gap, square_root_exact, sqrt_ceil,
};
use crate::vox::AFWD;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FixedPointWordFactorPair {
    pub p: ImasmNumeralWord,
    pub q: ImasmNumeralWord,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FixedPointWordVessel {
    n: ImasmNumeralWord,
    tower: NestedImasmTower,
    a: ImasmNumeralWord,
    gap: ImasmNumeralWord,
}

impl FixedPointWordVessel {
    /// Board N into the properly nested vessel, then construct the square-root
    /// seed inside that vessel. The nesting scale is read structurally from the
    /// canonical numeral's AFWD cells; no numeric N is decoded.
    pub fn board(n: ImasmNumeralWord) -> Result<Self, &'static str> {
        if compare(&n, &one())? != Ordering::Greater {
            return Err("fixed-point word vessel requires N > 1");
        }

        let width = n
            .as_str()
            .chars()
            .filter(|&mark| mark == AFWD)
            .count()
            .max(4);
        let tower = NestedImasmTower::from_scale(width)?;
        if !tower.banking_audit().banked {
            return Err("fixed-point vessel reversal is not banked");
        }

        // The seed is computed only after N and the enclosing tower are both
        // resident. This closes the previous 'sqrt seed outside the vessel' gap.
        let a = sqrt_ceil(&n)?;
        let gap = square_gap(&a, &n)?;

        Ok(Self { n, tower, a, gap })
    }

    pub fn n(&self) -> &ImasmNumeralWord {
        &self.n
    }

    pub fn tower(&self) -> &NestedImasmTower {
        &self.tower
    }

    pub fn midpoint(&self) -> &ImasmNumeralWord {
        &self.a
    }

    pub fn gap(&self) -> &ImasmNumeralWord {
        &self.gap
    }

    /// Read a factor pair only if the current resident gap is already a square.
    /// There is no fallback, divisor scan, order walk, or automatic advance.
    pub fn fixed_pair(&self) -> Result<Option<FixedPointWordFactorPair>, &'static str> {
        let Some(b) = square_root_exact(&self.gap)? else {
            return Ok(None);
        };

        let p = crate::fixed_point_word_arithmetic::subtract(&self.a, &b)?;
        let q = add(&self.a, &b)?;
        if compare(&p, &one())? != Ordering::Greater
            || compare(&q, &self.n)? != Ordering::Less
        {
            return Ok(None);
        }
        if compare(&multiply(&p, &q)?, &self.n)? != Ordering::Equal {
            return Err("fixed-point word vessel closed a non-factor product");
        }
        Ok(Some(FixedPointWordFactorPair { p, q }))
    }

    /// One explicit AFWD inside the already-open vessel. The gap is transported
    /// by `(a+1)^2-N = (a^2-N) + 2a + 1`, entirely through IMASM word addition.
    /// The enclosing frame topology is retained unchanged, so the resident count
    /// remains banked across its reversal.
    pub fn advance_once(mut self) -> Result<Self, &'static str> {
        if self.fixed_pair()?.is_some() {
            return Err("fixed-point word vessel is already fixed");
        }
        let twice_a = add(&self.a, &self.a)?;
        let delta = add(&twice_a, &one())?;
        self.gap = add(&self.gap, &delta)?;
        self.a = increment(&self.a)?;
        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sqrt_seed_and_candidate_state_live_inside_the_nested_vessel() {
        let n = ImasmNumeralWord::checked(
            include_str!("../tests/baked/fixed_point_semiprime_40.word").trim(),
        )
        .unwrap();
        let vessel = FixedPointWordVessel::board(n).unwrap();
        assert_eq!(vessel.tower().max_depth(), 37);
        assert!(vessel.tower().banking_audit().banked);
        assert_eq!(vessel.tower().banking_audit().exposed, 0);
    }

    #[test]
    fn real_40_bit_semiprime_closes_after_two_explicit_banked_advances() {
        let n = ImasmNumeralWord::checked(
            include_str!("../tests/baked/fixed_point_semiprime_40.word").trim(),
        )
        .unwrap();
        let original = n.clone();

        let vessel = FixedPointWordVessel::board(n).unwrap();
        assert!(vessel.fixed_pair().unwrap().is_none());
        let vessel = vessel.advance_once().unwrap();
        assert!(vessel.fixed_pair().unwrap().is_none());
        let vessel = vessel.advance_once().unwrap();
        let pair = vessel.fixed_pair().unwrap().expect("40-bit membrane did not close");

        assert_eq!(
            compare(&multiply(&pair.p, &pair.q).unwrap(), &original).unwrap(),
            Ordering::Equal
        );
    }

    #[test]
    fn vessel_refuses_to_hide_a_search_loop() {
        let n = ImasmNumeralWord::checked(
            include_str!("../tests/baked/fixed_point_semiprime_31.word").trim(),
        )
        .unwrap();
        let vessel = FixedPointWordVessel::board(n).unwrap();
        // This rung exposes only the current fixed point plus one explicit AFWD.
        // A non-fixed initial state must remain non-fixed rather than walking.
        assert!(vessel.fixed_pair().unwrap().is_none());
    }
}
