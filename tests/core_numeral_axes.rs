//! Acceptance tests for the remaining Core Numeral axes (§7.8–7.10).
//!
//! SuffixEnvelope itself already owns its own suite (tests/suffix_envelope.rs,
//! tests/suffix_envelope_fibre.rs); here we cross-check the two closed forms
//! the acceptance plan names — the product-fibre identity (2^d−1)^k and the
//! image theorem — against the live module, plus the three new axes.

use vox::bool_polynomial::{BoolCoeff, BoolPolynomial, carry_agreement};
use vox::layer_stack::LayerStack;
use vox::lean_evidence::default_registry;
use vox::suffix_envelope::{fixed_union_count, image_count, SuffixEnvelope};

#[test]
fn bool_polynomial_idempotents() {
    let a = BoolCoeff::singleton(0);
    assert_eq!(a.union(&a), a);
    assert_eq!(a.intersect(&a), a);
}

#[test]
fn bool_polynomial_cauchy_nontrivial() {
    // Threshold form per the plan: S_0 = T_0 = I (the full ground set),
    // S_1 = T_1 = {0}. Then (S ★ T)_0 = I, (S ★ T)_1 = {0},
    // (S ★ T)_2 = {0}: non-void at every layer.
    let p = BoolPolynomial {
        coeffs: vec![BoolCoeff::full(2), BoolCoeff::singleton(0)],
    };
    let q = BoolPolynomial {
        coeffs: vec![BoolCoeff::full(2), BoolCoeff::singleton(0)],
    };
    let r = p.cauchy(&q);
    assert!(carry_agreement(&p, &q));
    assert_eq!(r.coeffs.len(), 3);
    assert_eq!(r.coeffs[0].len(), 2); // S_0 layer: the full set survives
    assert!(r.coeffs[1].bits.contains(&0));
    assert!(r.coeffs[2].bits.contains(&0)); // the carry lands at z²
}

#[test]
fn bool_polynomial_second_layer_carry() {
    // S_2 = S_2 ∪ T_2 ∪ (S_1 ∩ T_1): with S_1 = T_1 = {0}, the carry lands at z².
    let p = BoolPolynomial {
        coeffs: vec![BoolCoeff::full(2), BoolCoeff::singleton(0)],
    };
    let q = BoolPolynomial {
        coeffs: vec![BoolCoeff::full(2), BoolCoeff::singleton(0)],
    };
    let r = p.cauchy(&q);
    assert!(r.second_layer().bits.contains(&0));
}

#[test]
fn layer_stack_mass() {
    let poly = BoolPolynomial {
        coeffs: vec![
            BoolCoeff::empty(),
            BoolCoeff::singleton(0),
            BoolCoeff::singleton(1),
        ],
    };
    let stack = LayerStack::from_polynomial(&poly);
    assert_eq!(stack.layer_cake_mass(), 2);
    assert_eq!(stack.l1_area(), 2);
    assert_eq!(stack.min_area(), 1);
    assert_eq!(stack.max_area(), 1);
    assert_eq!(stack.union_support().len(), 2);
}

#[test]
fn suffix_envelope_image_theorem() {
    let env = SuffixEnvelope::from_deposits(&[4u32, 2, 1]).unwrap();
    assert_eq!(env.gamma, vec![7, 3, 1]);
    // image theorem: reconstruction equals the original support chain
    assert_eq!(env.reconstruct(), env.gamma);
    assert_eq!(env.r(0), Some(2));
    assert_eq!(env.r(2), Some(0));
    // descending image count (depth+1)^k, spot-checked
    assert_eq!(image_count(2, 4), Some(25));
}

#[test]
fn lean_evidence_registry() {
    let reg = default_registry();
    assert!(reg.lookup("carry_identity").is_some());
    assert!(reg.lookup("syzygy_roundtrip").is_some());
    assert!(reg.lookup("nonexistent").is_none());
    assert!(!reg.is_empty());
}

#[test]
fn product_fibre_identity_small() {
    // (2^3 − 1)^2 = 7² = 49, read off the live fixed_union_count(k, depth)
    assert_eq!(fixed_union_count(2, 3), Some(49));
    // and the k = 0 edge: (2^d − 1)^0 = 1
    assert_eq!(fixed_union_count(0, 5), Some(1));
}
