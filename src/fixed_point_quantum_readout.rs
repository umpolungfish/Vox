//! One-shot readout boundary for the hypernested fixed-point membrane.
//!
//! A `QuantumPhaseSample` cannot be constructed by callers. It is the fixed
//! winding preimage produced when one complete hypernested denominator reaches
//! IFIX. The denominator is carried by integer winding (`⊡` count), while the
//! unique live clear remains banked through every enclosing carrier.

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
    /// Crate-private on purpose: external callers cannot inject a phase sample.
    /// The hypernested fixed-point executor must consume the complete denominator
    /// and call this exactly at the IFIX boundary that fixes its winding preimage.
    pub(crate) fn fix_from_collapse(
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
    use crate::fixed_point_quantum_membrane::FixedPointQuantumMembrane;
    use crate::morphism_factor::tape_u64;
    use crate::vox::IFIX;

    #[test]
    fn collapse_consumes_hypernested_denominator_into_k_over_m_and_fixation() {
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
        assert_eq!(sample.fixation_word().iter().filter(|&&c| c == IFIX).count(), width);
        assert_eq!(verdict(sample.fixation_word()), 'T');
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
    fn collapse_retains_full_winding_word_without_expanding_branches() {
        let membrane = FixedPointQuantumMembrane::from_n(&tape_u64(257)).unwrap();
        let program = membrane
            .phase_estimation_register()
            .unwrap()
            .into_measurement_program()
            .unwrap();
        let expected = program.fixation_word().to_vec();
        let width = program.width();
        let sample = QuantumPhaseSample::fix_from_collapse(program, tape_u64(5)).unwrap();
        assert_eq!(sample.fixation_word(), expected.as_slice());
        assert_eq!(sample.fixation_word().len(), 7 * width + 4);
    }
}
