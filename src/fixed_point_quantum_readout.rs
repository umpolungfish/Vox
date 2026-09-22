//! One-shot readout boundary for the properly nested fixed-point membrane.
//!
//! A `QuantumPhaseSample` cannot be constructed by callers.  It is the fixed
//! winding preimage produced when one complete nested denominator collapses at
//! IFIX.  The denominator is carried by frame depth and dissolves to one retained
//! membrane; callers never supply an independent k/M pair at the public boundary.

use core::cmp::Ordering;

use alloc::vec::Vec;

use crate::fixed_point_quantum_membrane::FixedPointQuantumMembrane;
use crate::fixed_point_quantum_phase::{power_of_two, PhaseMeasurementProgram};
use crate::hadamard_factor_bridge::HadamardDescent;
use crate::hadamard_gate::Tape;
use crate::morphism_factor::cmp;
use crate::vox::{EVALF, EVALT, TANCH, VINIT};

#[derive(Clone, PartialEq, Debug)]
pub struct QuantumPhaseSample {
    numerator: Tape,
    denominator: Tape,
    fixation_word: Vec<char>,
}

fn valid_tape(tape: &[char]) -> bool {
    !tape.is_empty() && tape.iter().all(|&mark| mark == EVALT || mark == EVALF)
}

impl QuantumPhaseSample {
    /// Crate-private on purpose: external callers cannot inject a phase sample.
    /// The nested fixed-point executor must consume the complete denominator and
    /// call this exactly at the IFIX boundary that fixes its winding preimage.
    pub(crate) fn fix_from_collapse(
        program: PhaseMeasurementProgram,
        numerator: Tape,
    ) -> Result<Self, &'static str> {
        if !valid_tape(&numerator) {
            return Err("quantum phase collapse produced a malformed numeral");
        }
        let width = program.width();
        if program.tower().max_depth() != width {
            return Err("quantum phase collapse lost denominator nesting depth");
        }
        if !program.tower().banking_audit().banked {
            return Err("quantum phase collapse exposed its reversal");
        }
        let denominator = program.denominator().to_vec();
        if denominator != power_of_two(width) {
            return Err("quantum phase collapse denominator changed");
        }
        if cmp(&numerator, &denominator) != Ordering::Less {
            return Err("quantum phase collapse lies outside its nested denominator");
        }

        let fixation_word = program.dissolved_word();
        if fixation_word.first() != Some(&VINIT) || fixation_word.last() != Some(&TANCH) {
            return Err("quantum phase fixation is not a closed IMASM membrane");
        }

        // `program` is consumed here. The complete depth-t bulk has dissolved to
        // this one fixed boundary; no per-depth host execution objects survive.
        Ok(Self { numerator, denominator, fixation_word })
    }

    pub fn numerator(&self) -> &[char] { &self.numerator }
    pub fn denominator(&self) -> &[char] { &self.denominator }
    pub fn fixation_word(&self) -> &[char] { &self.fixation_word }
}

impl FixedPointQuantumMembrane {
    /// Consume both the resident membrane and one opaque fixed winding preimage.
    ///
    /// This boundary performs only the standard phase-sample readout already in
    /// the Hadamard bridge: continued fractions certify one denominator against
    /// the resident modular relation, then the order-two descent is attempted.
    /// It does not repeat a readout, scan bases, walk an orbit, or search factor
    /// candidates.
    pub fn descend_quantum_measurement(
        self,
        sample: QuantumPhaseSample,
    ) -> HadamardDescent {
        let width = self.n().len().saturating_mul(2).max(2);
        if sample.denominator != power_of_two(width) {
            return HadamardDescent::F;
        }
        let base = self.base().to_vec();
        let spectral = self.into_spectral();
        let carrier = spectral.into_carrier();
        carrier.descend_phase_sample(&base, &sample.numerator, &sample.denominator)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixed_point_quantum_membrane::FixedPointQuantumMembrane;
    use crate::morphism_factor::tape_u64;

    #[test]
    fn collapse_consumes_nested_denominator_into_only_k_over_m_and_fixation() {
        let membrane = FixedPointQuantumMembrane::from_n(&tape_u64(257)).unwrap();
        let program = membrane
            .phase_estimation_register()
            .unwrap()
            .into_measurement_program()
            .unwrap();
        let width = program.width();
        let sample = QuantumPhaseSample::fix_from_collapse(program, tape_u64(5)).unwrap();
        assert_eq!(sample.numerator(), tape_u64(5).as_slice());
        assert_eq!(sample.denominator(), power_of_two(width).as_slice());
        assert_eq!(sample.fixation_word().first(), Some(&VINIT));
        assert_eq!(sample.fixation_word().last(), Some(&TANCH));
    }

    #[test]
    fn collapse_rejects_non_numeral_or_out_of_denominator_readout() {
        let membrane = FixedPointQuantumMembrane::from_n(&tape_u64(257)).unwrap();
        let program = membrane
            .phase_estimation_register()
            .unwrap()
            .into_measurement_program()
            .unwrap();
        let width = program.width();
        assert!(QuantumPhaseSample::fix_from_collapse(program, power_of_two(width)).is_err());

        let membrane = FixedPointQuantumMembrane::from_n(&tape_u64(257)).unwrap();
        let program = membrane
            .phase_estimation_register()
            .unwrap()
            .into_measurement_program()
            .unwrap();
        assert!(QuantumPhaseSample::fix_from_collapse(program, vec!['⊙']).is_err());
    }

    #[test]
    fn collapse_keeps_only_one_dissolved_fixed_point_boundary() {
        let membrane = FixedPointQuantumMembrane::from_n(&tape_u64(257)).unwrap();
        let program = membrane
            .phase_estimation_register()
            .unwrap()
            .into_measurement_program()
            .unwrap();
        let expected = program.dissolved_word();
        let sample = QuantumPhaseSample::fix_from_collapse(program, tape_u64(5)).unwrap();
        assert_eq!(sample.fixation_word(), expected.as_slice());
    }
}
