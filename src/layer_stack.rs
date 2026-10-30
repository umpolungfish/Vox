//! Core Numeral axis: LayerStack
//!
//! Threshold supports S_t, reconstruction, layer-cake mass, L¹ area,
//! min/max area. Thin adapter over the Boolean layer carrier; it does not
//! merge the carrier with the powerset monad μ (daplan.md §8).

use crate::bool_polynomial::{BoolCoeff, BoolPolynomial};
use alloc::vec::Vec;

/// Ordered stack of threshold supports S_t (coefficient of z^t).
#[derive(Clone, Debug)]
pub struct LayerStack {
    pub layers: Vec<BoolCoeff>,
}

impl LayerStack {
    pub fn from_polynomial(poly: &BoolPolynomial) -> Self {
        Self {
            layers: poly.coeffs.clone(),
        }
    }

    /// Layer-cake mass = Σ_t |S_t|.
    pub fn layer_cake_mass(&self) -> u128 {
        self.layers.iter().map(|c| c.len() as u128).sum()
    }

    /// L¹ area of the support vector.
    pub fn l1_area(&self) -> u128 {
        self.layer_cake_mass()
    }

    /// Minimum non-empty layer cardinality (0 if no non-empty layer).
    pub fn min_area(&self) -> usize {
        self.layers
            .iter()
            .filter(|c| !c.is_empty())
            .map(|c| c.len())
            .min()
            .unwrap_or(0)
    }

    /// Maximum layer cardinality.
    pub fn max_area(&self) -> usize {
        self.layers.iter().map(|c| c.len()).max().unwrap_or(0)
    }

    /// Union of all layer supports: the atoms that appear at some layer.
    pub fn union_support(&self) -> BoolCoeff {
        self.layers
            .iter()
            .fold(BoolCoeff::empty(), |acc, c| acc.union(c))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bool_polynomial::{BoolCoeff, BoolPolynomial};

    #[test]
    fn empty_stack() {
        let s = LayerStack::from_polynomial(&BoolPolynomial::zero());
        assert_eq!(s.layer_cake_mass(), 0);
        assert_eq!(s.min_area(), 0);
        assert_eq!(s.max_area(), 0);
        assert!(s.union_support().is_empty());
    }

    #[test]
    fn mass_counts_each_layer() {
        let poly = BoolPolynomial {
            coeffs: alloc::vec![
                BoolCoeff::full(3),
                BoolCoeff::singleton(0),
                BoolCoeff::full(2),
            ],
        };
        let s = LayerStack::from_polynomial(&poly);
        assert_eq!(s.layer_cake_mass(), 6);
        assert_eq!(s.l1_area(), 6);
        assert_eq!(s.min_area(), 1);
        assert_eq!(s.max_area(), 3);
    }
}
