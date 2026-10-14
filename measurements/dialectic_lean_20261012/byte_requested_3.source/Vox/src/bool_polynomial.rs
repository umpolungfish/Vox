//! Core Numeral axis: BoolCoeff / BoolPolynomial
//!
//! Carrier = powerset of a finite index set I.
//! Add = union, multiply = intersection, zero = ∅, one = I.
//! Cauchy product realises the Boolean carry convolution ★.
//! Distinct from the powerset monad μ (see daplan.md §8 risk controls).

use alloc::collections::BTreeSet;
use alloc::vec::Vec;

/// A Boolean coefficient: a subset of the ground set {0‥n}.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct BoolCoeff {
    pub bits: BTreeSet<usize>,
}

impl BoolCoeff {
    pub fn empty() -> Self {
        Self {
            bits: BTreeSet::new(),
        }
    }

    pub fn full(n: usize) -> Self {
        Self {
            bits: (0..n).collect(),
        }
    }

    pub fn singleton(i: usize) -> Self {
        let mut s = BTreeSet::new();
        s.insert(i);
        Self { bits: s }
    }

    /// Union (Boolean addition)
    pub fn union(&self, other: &Self) -> Self {
        Self {
            bits: self.bits.union(&other.bits).copied().collect(),
        }
    }

    /// Intersection (Boolean multiplication)
    pub fn intersect(&self, other: &Self) -> Self {
        Self {
            bits: self.bits.intersection(&other.bits).copied().collect(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.bits.is_empty()
    }

    pub fn len(&self) -> usize {
        self.bits.len()
    }
}

/// Formal power series with BoolCoeff coefficients.
/// Φ(v) = Σ_t S_t(v) z^t, with S_0 = I on the threshold encoding.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BoolPolynomial {
    pub coeffs: Vec<BoolCoeff>,
}

impl BoolPolynomial {
    pub fn zero() -> Self {
        Self { coeffs: Vec::new() }
    }

    pub fn one(n: usize) -> Self {
        Self {
            coeffs: alloc::vec![BoolCoeff::full(n)],
        }
    }

    /// Cauchy product: (S ★ T)_t = ∪_{a+b=t} S_a ∩ T_b.
    pub fn cauchy(&self, other: &Self) -> Self {
        if self.coeffs.is_empty() || other.coeffs.is_empty() {
            return Self::zero();
        }
        let deg = self.coeffs.len() + other.coeffs.len() - 1;
        let mut out = alloc::vec![BoolCoeff::empty(); deg];
        for (i, a) in self.coeffs.iter().enumerate() {
            for (j, b) in other.coeffs.iter().enumerate() {
                let inter = a.intersect(b);
                out[i + j] = out[i + j].union(&inter);
            }
        }
        // drop trailing empty coefficients
        while out.last().map(|c| c.is_empty()).unwrap_or(false) {
            out.pop();
        }
        Self { coeffs: out }
    }

    /// Truncation / saturation at degree `max_deg`.
    pub fn truncate(&self, max_deg: usize) -> Self {
        let mut c = self.coeffs.clone();
        c.truncate(max_deg + 1);
        Self { coeffs: c }
    }

    /// First-layer support (the coefficient of z¹).
    pub fn first_layer(&self) -> BoolCoeff {
        self.coeffs
            .get(1)
            .cloned()
            .unwrap_or_else(BoolCoeff::empty)
    }

    /// Second-layer support (collects the S_1 ∩ T_1 depth-overlap carry).
    pub fn second_layer(&self) -> BoolCoeff {
        self.coeffs
            .get(2)
            .cloned()
            .unwrap_or_else(BoolCoeff::empty)
    }
}

/// Layer-by-layer agreement of two Boolean carry convolutions:
/// the product is non-void unless both factors are the void polynomial.
pub fn carry_agreement(p: &BoolPolynomial, q: &BoolPolynomial) -> bool {
    let prod = p.cauchy(q);
    !prod.coeffs.is_empty() || (p.coeffs.is_empty() && q.coeffs.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn union_intersect_idempotent() {
        let a = BoolCoeff::singleton(0);
        let b = BoolCoeff::singleton(1);
        let u = a.union(&b);
        assert_eq!(u.union(&u), u);
        assert_eq!(u.intersect(&u), u);
    }

    #[test]
    fn cauchy_second_layer_collects_the_carry() {
        let p = BoolPolynomial {
            coeffs: alloc::vec![BoolCoeff::full(2), BoolCoeff::singleton(0)],
        };
        let q = BoolPolynomial {
            coeffs: alloc::vec![BoolCoeff::full(2), BoolCoeff::singleton(0)],
        };
        let r = p.cauchy(&q);
        // S_2 = S_2 ∪ T_2 ∪ (S_1 ∩ T_1); both S_1 and T_1 hold {0}
        assert!(!r.second_layer().is_empty());
        assert!(r.second_layer().bits.contains(&0));
    }
}
