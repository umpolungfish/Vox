//! Acceptance tests for the SuffixEnvelope Core-Numeral axis.
//!
//! Image theorem, product fibre, fixed-union count `(2^d − 1)^k`, and the
//! deepest-occurrence polynomial `M_{k,d}(y)`. The chain is taken from the
//! phase-word restore unions and from frame-sweep windows.

use vox::factor_2adic::frame_sweep;
use vox::provenance_envelope::restored_support_ladder;
use vox::suffix_envelope::{
    fixed_union_count, image_count, mass_polynomial, parametrized_fibre, SuffixEnvelope,
};
use vox::vox::{EVALF, EVALT};

#[test]
fn image_theorem_counts_descending_chains() {
    for k in 0..8u32 {
        for depth in 0..9u32 {
            let expected = u128::from(depth + 1).pow(k);
            assert_eq!(image_count(k, depth), Some(expected), "k={k} d={depth}");
        }
    }
}

#[test]
fn fixed_union_count_is_two_to_the_d_minus_one_to_the_k() {
    for k in 0..6u32 {
        for depth in 0..12u32 {
            let expected = ((1u128 << depth) - 1).pow(k);
            assert_eq!(
                fixed_union_count(k, depth),
                Some(expected),
                "k={k} d={depth}"
            );
        }
    }
}

#[test]
fn deepest_occurrence_polynomial_matches_closed_form() {
    assert_eq!(mass_polynomial(0, 4), Some(vec![1]));
    assert_eq!(mass_polynomial(1, 1), Some(vec![1, 1]));
    assert_eq!(mass_polynomial(1, 3), Some(vec![1, 1, 2, 4]));
    assert_eq!(mass_polynomial(2, 1), Some(vec![1, 2, 1]));
    assert_eq!(mass_polynomial(2, 2), Some(vec![1, 2, 5, 4, 4]));

    for k in 0..5u32 {
        for depth in 0..7u32 {
            let poly = mass_polynomial(k, depth).expect("polynomial");
            let sum: u128 = poly.iter().sum();
            assert_eq!(sum, 1u128 << (k * depth), "total sequences k={k} d={depth}");
            assert_eq!(poly.len(), (k * depth) as usize + 1);
        }
    }
}

#[test]
fn product_fibre_matches_free_bit_parametrization() {
    let deposits = [4u32, 2, 1];
    let envelope = SuffixEnvelope::from_deposits(&deposits).unwrap();
    assert_eq!(envelope.gamma, [7, 3, 1]);
    assert_eq!(envelope.gamma, restored_support_ladder(&deposits));
    assert_eq!(envelope.fibre_count, 1u128 << (2 + 1));
    assert_eq!(
        parametrized_fibre(&envelope.fibre_param()),
        Some(envelope.fibre_count)
    );
    assert_eq!(envelope.reconstruct(), envelope.gamma);
    assert_eq!(envelope.gamma_i(0), Some(7));
    assert_eq!(envelope.gamma_i(1), Some(3));
    assert_eq!(envelope.gamma_i(2), Some(1));
    assert_eq!(envelope.gamma_i(3), None);
    assert_eq!(envelope.r(0), Some(2));
    assert_eq!(envelope.r(1), Some(1));
    assert_eq!(envelope.r(2), Some(0));

    let over_same = SuffixEnvelope::from_deposits(&[3, 2]).unwrap();
    let other = SuffixEnvelope::from_deposits(&[1, 2]).unwrap();
    assert_eq!(over_same.gamma, other.gamma);
    assert_eq!(over_same.fibre_count, 2);
    assert!(SuffixEnvelope::try_from_ladder(&[1, 3]).is_none());
}

#[test]
fn phase_word_restore_unions_are_the_suffix_chain() {
    let from_word = SuffixEnvelope::from_phase_word("∈⊤∈⊥∋∋").unwrap();
    let from_deposits = SuffixEnvelope::from_deposits(&[1, 2]).unwrap();
    assert_eq!(from_word, from_deposits);
    assert_eq!(from_word.gamma, vec![3, 2]);
    assert_eq!(from_word.r(0), Some(0));
    assert_eq!(from_word.r(1), Some(1));

    let nested = SuffixEnvelope::from_phase_word("∈∈⊤∋≺∋").unwrap();
    assert_eq!(nested.gamma, vec![1, 1]);
    assert_eq!(nested.fibre_count, 2);

    let both_lanes = SuffixEnvelope::from_phase_word("∈⊞∋").unwrap();
    assert_eq!(both_lanes.gamma_i(0), Some(12));
    assert_eq!(both_lanes.fibre_count, 1);

    assert!(SuffixEnvelope::from_phase_word("").is_err());
    assert!(SuffixEnvelope::from_phase_word("⊡?").is_err());
    let open = SuffixEnvelope::from_phase_word("∈⊤").unwrap();
    assert!(open.gamma.is_empty());
    assert_eq!(open.fibre_count, 1);
}

#[test]
fn frame_sweep_windows_feed_the_envelope() {
    let tape = [EVALF, EVALT, EVALF, EVALT];
    let envelope = SuffixEnvelope::from_frame_sweep(&tape, 2).unwrap();
    assert_eq!(envelope.gamma, vec![1, 1]);
    assert_eq!(envelope.reconstruct(), envelope.gamma);
    assert_eq!(
        parametrized_fibre(&envelope.fibre_param()),
        Some(envelope.fibre_count)
    );

    let sweep = frame_sweep(&tape);
    let window = sweep.iter().find(|sweep| sweep.window == 2).unwrap();
    assert_eq!(window.reconstruct(), tape);

    assert!(SuffixEnvelope::from_frame_sweep(&tape, 1).is_err());
    assert!(SuffixEnvelope::from_frame_sweep(&tape, 9).is_err());
}
