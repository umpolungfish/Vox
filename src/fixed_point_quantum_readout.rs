//! One-shot readout boundary for the collapsed hypernested fixed-point membrane.
//!
//! A `QuantumPhaseSample` has no production constructor that accepts a host
//! numerator. The complete denominator is carried by hypernest winding while one
//! canonical carrier executes once. Only the membrane executor may eventually
//! mint the opaque fixed winding preimage consumed by Hadamard descent.

use core::cmp::Ordering;

use alloc::vec::Vec;

use crate::fixed_point_quantum_membrane::FixedPointQuantumMembrane;
use crate::fixed_point_quantum_phase::{power_of_two, PhaseMeasurementProgram};
use crate::hadamard_factor_bridge::HadamardDescent;
use crate::hadamard_gate::Tape;
use crate::morphism_factor::cmp;
use crate::vox::{verdict, EVALF, EVALT};

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
    /// Test-only seam for validating the readout boundary. Production code has
    /// no path that accepts an arbitrary k/M pair; the real executor must create
    /// this object from the N-dependent collapsed membrane itself.
    #[cfg(test)]
    fn fix_from_collapse_for_test(
        program: PhaseMeasurementProgram,
        numerator: Tape,
    ) -> Result<Self, &'static str> {
        Self::from_executor_landing(program, numerator)
    }

    /// Private mint used by the eventual membrane executor. Keeping this private
    /// prevents every other production module from injecting a phase sample.
    fn from_executor_landing(
        program: PhaseMeasurementProgram,
        numerator: Tape,
    ) -> Result<Self, &'static str> {
        if !valid_tape(&numerator) {
            return Err("quantum phase collapse produced a malformed numeral");
        }
        let width = program.width();
        let audit = program.hypernest().audit();
        if program.hypernest().depth() != width || audit.winding != width {
            return Err("quantum phase collapse lost hypernest winding");
        }
        if program.execution_ticks() != 1 {
            return Err("quantum phase collapse expanded the hypernest at runtime");
        }
        if audit.live_clears != 1 || !audit.banked || audit.exposed != 0 {
            return Err("quantum phase collapse exposed its unique live clear");
        }
        if !audit.closed {
            return Err("quantum phase collapse lost control-flow closure");
        }

        let denominator = program.denominator().to_vec();
        if denominator != power_of_two(width) {
            return Err("quantum phase collapse denominator changed");
        }
        if cmp(&numerator, &denominator) != Ordering::Less {
            return Err("quantum phase collapse lies outside its hypernested denominator");
        }

        let fixation_word = program.fixation_word().to_vec();
        if verdict(&fixation_word) != 'T' {
            return Err("quantum phase fixation is not a closed hypercarrier");
        }

        Ok(Self { numerator, denominator, fixation_word })
    }

    pub fn numerator(&self) -> &[char] { &self.numerator }
    pub fn denominator(&self) -> &[char] { &self.denominator }
    pub fn fixation_word(&self) -> &[char] { &self.fixation_word }
}

impl FixedPointQuantumMembrane {
    /// Consume the resident membrane and one opaque fixed winding preimage.
    ///
    /// This boundary only interprets an already-produced quantum readout. It
    /// never repeats measurement, scans bases, walks a modular orbit, or searches
    /// factor candidates.
    pub fn descend_quantum_measurement(
        self,
        sample: QuantumPhaseSample,
    ) -> HadamardDescent {
        let width = self.n().len().saturating_mul(2).max(2);
        if sample.denominator != power_of_two(width) {
            return HadamardDescent::F;
        }
        if verdict(&sample.fixation_word) != 'T' {
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
    use crate::fixed_point_hypernest::COLLAPSED_HYPERNEST_WORD;
    use crate::fixed_point_quantum_membrane::FixedPointQuantumMembrane;
    use crate::morphism_factor::tape_u64;

    #[test]
    fn test_seam_consumes_collapsed_denominator_into_k_over_m_and_fixation() {
        let membrane = FixedPointQuantumMembrane::from_n(&tape_u64(257)).unwrap();
        let program = membrane
            .phase_estimation_register()
            .unwrap()
            .into_measurement_program()
            .unwrap();
        let width = program.width();
        let sample = QuantumPhaseSample::fix_from_collapse_for_test(program, tape_u64(5)).unwrap();
        assert_eq!(sample.numerator(), tape_u64(5).as_slice());
        assert_eq!(sample.denominator(), power_of_two(width).as_slice());
        assert_eq!(sample.fixation_word(), COLLAPSED_HYPERNEST_WORD.as_slice());
        assert_eq!(verdict(sample.fixation_word()), 'T');
    }

    #[test]
    fn test_seam_rejects_non_numeral_or_out_of_denominator_readout() {
        let membrane = FixedPointQuantumMembrane::from_n(&tape_u64(257)).unwrap();
        let program = membrane
            .phase_estimation_register()
            .unwrap()
            .into_measurement_program()
            .unwrap();
        let width = program.width();
        assert!(QuantumPhaseSample::fix_from_collapse_for_test(program, power_of_two(width)).is_err());

        let membrane = FixedPointQuantumMembrane::from_n(&tape_u64(257)).unwrap();
        let program = membrane
            .phase_estimation_register()
            .unwrap()
            .into_measurement_program()
            .unwrap();
        assert!(QuantumPhaseSample::fix_from_collapse_for_test(program, vec!['⊙']).is_err());
    }

    #[test]
    fn fixation_word_stays_constant_while_denominator_winding_grows() {
        for n in [257u64, 65_537, 4_294_967_291] {
            let membrane = FixedPointQuantumMembrane::from_n(&tape_u64(n)).unwrap();
            let program = membrane
                .phase_estimation_register()
                .unwrap()
                .into_measurement_program()
                .unwrap();
            let width = program.width();
            let sample = QuantumPhaseSample::fix_from_collapse_for_test(program, tape_u64(1)).unwrap();
            assert_eq!(sample.fixation_word(), COLLAPSED_HYPERNEST_WORD.as_slice());
            assert_eq!(sample.fixation_word().len(), COLLAPSED_HYPERNEST_WORD.len());
            assert_eq!(sample.denominator(), power_of_two(width).as_slice());
        }
    }
}
