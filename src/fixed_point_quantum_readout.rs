//! One-shot readout boundary for the quantum fixed-point membrane.
//!
//! A `QuantumPhaseSample` cannot be constructed by callers.  It is a product of
//! consuming a complete `PhaseMeasurementProgram`; the eventual membrane
//! executor is the only crate layer allowed to invoke `fix_from_collapse`.
//! Downstream continued-fraction/Hadamard descent therefore consumes a genuine
//! measurement object rather than an arbitrary host-supplied k/M pair.

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
    /// The quantum membrane executor must consume the measurement program and
    /// call this at its IFIX boundary.
    pub(crate) fn fix_from_collapse(
        program: PhaseMeasurementProgram,
        numerator: Tape,
    ) -> Result<Self, &'static str> {
        if !valid_tape(&numerator) {
            return Err("quantum phase collapse produced a malformed numeral");
        }
        let width = program.width();
        if program.measurements().len() != width {
            return Err("quantum phase collapse lost a measurement wire");
        }
        let denominator = program.denominator().to_vec();
        if denominator != power_of_two(width) {
            return Err("quantum phase collapse denominator changed");
        }
        if cmp(&numerator, &denominator) != Ordering::Less {
            return Err("quantum phase collapse lies outside its Fourier register");
        }

        let first = program
            .measurements()
            .first()
            .ok_or("quantum phase collapse has no measurement gates")?
            .dissolved_word();
        if program
            .measurements()
            .iter()
            .any(|gate| gate.dissolved_word() != first)
        {
            return Err("quantum phase measurement gates do not collapse to one membrane");
        }
        if first.first() != Some(&VINIT) || first.last() != Some(&TANCH) {
            return Err("quantum phase fixation is not a closed IMASM membrane");
        }

        // `program` is consumed here.  Controls, inverse-QFT and measurement
        // frames do not survive beside the one fixed readout.
        Ok(Self { numerator, denominator, fixation_word: first })
    }

    pub fn numerator(&self) -> &[char] { &self.numerator }
    pub fn denominator(&self) -> &[char] { &self.denominator }
    pub fn fixation_word(&self) -> &[char] { &self.fixation_word }
}

impl FixedPointQuantumMembrane {
    /// Consume both the resident membrane and one opaque quantum measurement.
    ///
    /// This boundary performs only the standard phase-sample readout already in
    /// the Hadamard bridge: continued fractions certify one denominator against
    /// the resident modular relation, then the order-two descent is attempted.
    /// It does not repeat a measurement, scan bases, walk an orbit, or search
    /// factor candidates.
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
    fn collapse_consumes_program_into_only_k_over_m_and_fixation() {
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
    fn collapse_rejects_non_numeral_or_out_of_register_readout() {
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
}
