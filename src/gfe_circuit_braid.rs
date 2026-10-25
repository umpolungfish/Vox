//! Source-bound QFT plus Fibonacci-QC braid carried into the GFE resident.
//!
//! The compiled operator is the 59-generator `QFT1+HTSX` word retained by the
//! G-mOMonadOS compiler. Its synthesized 2x2 unitary acts on the source phase
//! coordinate before FactorSynthesis prepares its branch priorities. The
//! resulting complex amplitudes and the induced priority order travel with the
//! same GFE resident through bind, re-entry and terminal readout.

use alloc::{string::String, vec::Vec};

const OPERATOR: [(f64, f64); 4] = [
    (-0.07042358424399570, 0.15249544690176758),
    (-0.73126997987826770, 0.66108235038101892),
    (-0.85470152839619384, 0.49119339858518468),
    (0.16679387281745811, -0.01985312403866429),
];
const BRAID_WORD: [i8; 59] = [
    -2, -3, 2, 1, 1, 2, 2, -1, 2, 2, -1, -2, -2, 1, 1, -2, -2, -1, -1, -1,
    2, 2, -1, -1, 2, 2, 1, 1, 1, -2, -2, 1, 1, -2, -2, 1, 1, 2, -1, -1,
    2, 2, -1, -1, 2, -1, -1, 2, 2, -1, -1, 2, 1, 1, -2, -2, 1, 1, -2,
];

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ComplexAmplitude {
    pub re: f64,
    pub im: f64,
}

impl ComplexAmplitude {
    fn zero() -> Self { Self { re: 0.0, im: 0.0 } }
    fn norm_sqr(self) -> f64 { self.re * self.re + self.im * self.im }
}

#[derive(Clone, Debug, PartialEq)]
pub struct GfeCircuitBraidState {
    pub circuit: &'static str,
    pub braid_word: Vec<i32>,
    pub braid_order: Vec<usize>,
    pub inverse_braid_word: Vec<i32>,
    pub operator: [(f64, f64); 4],
    pub inverse_recovery_residual: f64,
    pub source_phase_bin: i64,
    pub input_label: usize,
    pub input: [ComplexAmplitude; 4],
    pub output: [ComplexAmplitude; 4],
    pub output_weights: [f64; 4],
    pub unitary_residual: f64,
    pub norm_residual: f64,
    pub torus_axis_order: [usize; 12],
    pub axis_order: [usize; 12],
    pub priority_order_changed: bool,
}

impl GfeCircuitBraidState {
    pub fn prepare(
        source_phase_bin: i64,
        source_log: crate::pwoat_geometry::Log2Coordinate,
        torus_axes: [usize; 12],
    ) -> Result<Self, String> {
        let original: Vec<i32> = BRAID_WORD.iter().map(|&g| i32::from(g)).collect();
        let order = crate::ordering::selected("braid", original.len())?;
        let braid_word = crate::ordering::selected_braid(&original)?;
        let inverse_braid_word: Vec<i32> = braid_word.iter().rev().map(|g| -g).collect();
        let operator = if braid_word == original { OPERATOR } else { evaluate_word(&braid_word) };
        let inverse_operator = evaluate_word(&inverse_braid_word);
        let unitary_residual = unitary_residual(&operator);
        if !unitary_residual.is_finite() || unitary_residual > 1e-12 {
            return Err(String::from("compiled QFT plus QC braid failed its unitary closure"));
        }

        // The source's integer exponent and leading mantissa address the two
        // phase-register coordinates. Both are derived from N alone.
        let phase_bit = source_log.integer_exponent & 1;
        let mantissa_bit = (source_log.leading_mantissa & 1) as usize;
        let input_label = phase_bit * 2 + mantissa_bit;
        let mut input = [ComplexAmplitude::zero(); 4];
        input[input_label] = ComplexAmplitude { re: 1.0, im: 0.0 };

        // Apply U on the first phase coordinate and retain the second as its
        // spectator. Outcome order is little-endian: label = 2*p + q.
        let mut output = [ComplexAmplitude::zero(); 4];
        for q in 0..2 {
            let a = input[q];
            let b = input[2 + q];
            output[q] = multiply_add(operator[0], a, operator[1], b);
            output[2 + q] = multiply_add(operator[2], a, operator[3], b);
        }
        let mut inverse_recovery_residual = 0.0f64;
        for q in 0..2 {
            let recovered = [multiply_add(inverse_operator[0], output[q], inverse_operator[1], output[2+q]),
                multiply_add(inverse_operator[2], output[q], inverse_operator[3], output[2+q])];
            for (i, actual) in recovered.iter().enumerate() {
                let expected = input[2*i+q];
                inverse_recovery_residual = inverse_recovery_residual.max((actual.re-expected.re).abs()).max((actual.im-expected.im).abs());
            }
        }
        if inverse_recovery_residual > 1e-12 { return Err("selected braid inverse failed state recovery".into()); }
        let input_norm: f64 = input.iter().map(|a| a.norm_sqr()).sum();
        let output_weights = core::array::from_fn(|i| output[i].norm_sqr());
        let output_norm: f64 = output_weights.iter().sum();
        let norm_residual = (output_norm - input_norm).abs();
        if norm_residual > 1e-12 {
            return Err(String::from("compiled QFT plus QC braid changed source-state norm"));
        }

        // The transformed amplitudes order the live GFE branch axes. Ties
        // retain the singularity braid's order, preserving its ancestry.
        let mut axis_order = torus_axes;
        axis_order.sort_by(|left, right| {
            output_weights[*right % 4].partial_cmp(&output_weights[*left % 4])
                .unwrap_or(core::cmp::Ordering::Equal)
        });
        let axis_selection = crate::ordering::selected("axes", axis_order.len())?;
        let ordered_axes = axis_selection.apply(&axis_order)?;
        axis_order.copy_from_slice(&ordered_axes);
        let priority_order_changed = axis_order != torus_axes;
        if !priority_order_changed && crate::ordering::setting("IG_ORDER_SITE").is_none() {
            return Err(String::from("QFT plus QC amplitudes did not transform the torus priority order"));
        }
        Ok(Self {
            circuit: "QFT1+HTSX",
            braid_word,
            braid_order: order.indices,
            inverse_braid_word,
            operator,
            inverse_recovery_residual,
            source_phase_bin,
            input_label,
            input,
            output,
            output_weights,
            unitary_residual,
            norm_residual,
            torus_axis_order: torus_axes,
            axis_order,
            priority_order_changed,
        })
    }
}

fn multiply_add(
    left: (f64, f64), a: ComplexAmplitude,
    right: (f64, f64), b: ComplexAmplitude,
) -> ComplexAmplitude {
    ComplexAmplitude {
        re: left.0 * a.re - left.1 * a.im + right.0 * b.re - right.1 * b.im,
        im: left.0 * a.im + left.1 * a.re + right.0 * b.im + right.1 * b.re,
    }
}

fn unitary_residual(operator: &[(f64,f64);4]) -> f64 {
    let mut maximum = 0.0f64;
    for left in 0..2 {
        for right in 0..2 {
            let mut re = 0.0;
            let mut im = 0.0;
            for row in 0..2 {
                let a = operator[row * 2 + left];
                let b = operator[row * 2 + right];
                re += a.0 * b.0 + a.1 * b.1;
                im += a.0 * b.1 - a.1 * b.0;
            }
            let expected = if left == right { 1.0 } else { 0.0 };
            maximum = maximum.max((re - expected).abs()).max(im.abs());
        }
    }
    maximum
}

/// Four-strand Fibonacci fusion basis, matching the native compiler's F/R representation.
fn evaluate_word(word: &[i32]) -> [(f64, f64); 4] {
    use num_complex::Complex64 as C;
    let phi = (1.0 + libm::sqrt(5.0)) / 2.0;
    let phase = |angle| C::new(libm::cos(angle), libm::sin(angle));
    let r0 = phase(4.0 * core::f64::consts::PI / 5.0);
    let r1 = phase(7.0 * core::f64::consts::PI / 5.0);
    let (a,b) = (1.0/phi, 1.0/libm::sqrt(phi));
    let diagonal = [r0,C::new(0.0,0.0),C::new(0.0,0.0),r1];
    let middle = [a*a*r0+b*b*r1,a*b*(r0-r1),a*b*(r0-r1),b*b*r0+a*a*r1];
    let mut result = [C::new(1.0,0.0),C::new(0.0,0.0),C::new(0.0,0.0),C::new(1.0,0.0)];
    for &g in word {
        let mut gate = if g.abs()==2 {middle} else {diagonal};
        if g<0 {gate=[gate[0].conj(),gate[2].conj(),gate[1].conj(),gate[3].conj()];}
        result=core::array::from_fn(|i| gate[(i/2)*2]*result[i%2]+gate[(i/2)*2+1]*result[2+i%2]);
    }
    result.map(|z|(z.re,z.im))
}

#[cfg(test)]
mod ordering_tests {
    use super::*;
    #[test]
    fn operator_tracks_word_and_its_true_inverse() {
        let original: Vec<i32> = BRAID_WORD.iter().map(|&g|i32::from(g)).collect();
        let evaluated=evaluate_word(&original);
        for i in 0..4 { assert!((evaluated[i].0-OPERATOR[i].0).abs()<1e-12); assert!((evaluated[i].1-OPERATOR[i].1).abs()<1e-12); }
        let first=evaluate_word(&[1,2]); let second=evaluate_word(&[2,1]);
        assert!(first.iter().zip(second).any(|(a,b)|(a.0-b.0).abs()>0.1 || (a.1-b.1).abs()>0.1));
        for order in crate::ordering::AllOrders::new(3) {
            let word=order.apply(&[1,2,-3]).unwrap();
            let mut roundtrip=word.clone();roundtrip.extend(word.iter().rev().map(|g|-g));
            let actual=evaluate_word(&roundtrip);
            for (i,z) in actual.iter().enumerate() { assert!((z.0-if i==0||i==3 {1.0}else{0.0}).abs()<1e-12);assert!(z.1.abs()<1e-12); }
        }
    }
}
