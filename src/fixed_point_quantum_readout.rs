//! One-shot readout boundary for the collapsed hypernested fixed-point membrane.
//!
//! A `QuantumPhaseSample` has no production constructor that accepts a host
//! numerator. The collapsed hypernest carries the QPE precision denominator
//! `M = 2^t`; it does not by itself claim the multiplicative period `r`.
//! A sample can only be minted at the explicit pair-before-advance landing
//! boundary and remains bound to that landing's resident modular relation
//! `(a, N)`. The existing Hadamard bridge may infer a period from a genuine
//! fixed phase sample `k/M`; this module never relabels `M` as that period.

use core::cmp::Ordering;

use alloc::vec::Vec;

use crate::fixed_point_quantum_membrane::FixedPointQuantumMembrane;
use crate::fixed_point_quantum_phase::{power_of_two, PhaseLandingProgram};
use crate::fixed_point_quantum_relation::resident_landing_word;
use crate::hadamard_factor_bridge::HadamardDescent;
use crate::hadamard_gate::Tape;
use crate::morphism_factor::cmp;
use crate::vox::{verdict, AFWD, CLINK, EVALF, EVALT};

#[derive(Clone, PartialEq, Debug)]
pub struct QuantumPhaseSample {
    numerator: Tape,
    precision_denominator: Tape,
    n: Tape,
    base: Tape,
    fixation_word: Vec<char>,
}

fn valid_tape(tape: &[char]) -> bool {
    !tape.is_empty() && tape.iter().all(|&mark| mark == EVALT || mark == EVALF)
}

impl QuantumPhaseSample {
    /// Test-only seam for validating the opaque landing boundary. Production
    /// code has no API that accepts an arbitrary k/M pair.
    #[cfg(test)]
    fn fix_from_landing_for_test(
        landing: PhaseLandingProgram,
        numerator: Tape,
    ) -> Result<Self, &'static str> {
        Self::from_executor_landing(landing, numerator)
    }

    /// Private mint used only by the membrane executor. The landing object is
    /// consumed, so a caller cannot bypass the pair-before-advance ordering and
    /// later attach an unrelated phase sample. The fixation word is rebuilt
    /// from the resident IMASM numerals themselves, not from a generic shell.
    ///
    /// `landing.denominator()` is the QPE precision denominator `M = 2^t`.
    /// It is deliberately stored under that name here so it cannot be confused
    /// with the multiplicative period denominator recovered from a real phase.
    fn from_executor_landing(
        landing: PhaseLandingProgram,
        numerator: Tape,
    ) -> Result<Self, &'static str> {
        if !valid_tape(&numerator) {
            return Err("quantum phase landing produced a malformed numeral");
        }

        let word = resident_landing_word(&landing)?;
        if verdict(&word) != 'T' {
            return Err("resident quantum phase landing does not close");
        }
        let link = word
            .iter()
            .position(|&mark| mark == CLINK)
            .ok_or("quantum phase landing lost its spectral pair")?;
        let advance = word
            .iter()
            .position(|&mark| mark == AFWD)
            .ok_or("quantum phase landing lost its transport")?;
        if link >= advance {
            return Err("quantum phase landing advanced before forming the spectral pair");
        }

        let width = landing.width();
        let audit = landing.hypernest().audit();
        if landing.hypernest().depth() != width || audit.winding != width {
            return Err("quantum phase landing lost hypernest winding");
        }
        if landing.execution_ticks() != 1 {
            return Err("quantum phase landing expanded the hypernest at runtime");
        }
        if audit.live_clears != 1 || !audit.banked || audit.exposed != 0 {
            return Err("quantum phase landing exposed its unique live clear");
        }
        if !audit.closed {
            return Err("quantum phase landing lost control-flow closure");
        }
        if !valid_tape(landing.n()) || !valid_tape(landing.base()) {
            return Err("quantum phase landing lost its resident modular relation");
        }

        let precision_denominator = landing.denominator().to_vec();
        if precision_denominator != power_of_two(width) {
            return Err("quantum phase landing precision denominator changed");
        }
        if cmp(&numerator, &precision_denominator) != Ordering::Less {
            return Err("quantum phase landing lies outside its precision denominator");
        }

        Ok(Self {
            numerator,
            precision_denominator,
            n: landing.n().to_vec(),
            base: landing.base().to_vec(),
            fixation_word: word,
        })
    }

    pub fn numerator(&self) -> &[char] { &self.numerator }

    /// The QPE precision denominator `M = 2^t` represented by hypernest depth.
    /// This value is not, in general, the multiplicative period `r`.
    pub fn precision_denominator(&self) -> &[char] { &self.precision_denominator }

    /// Compatibility alias for the phase-sample denominator `M`.
    /// New code should prefer `precision_denominator()` so `M` is not mistaken
    /// for the period denominator recovered by continued-fraction descent.
    pub fn denominator(&self) -> &[char] { self.precision_denominator() }

    pub fn n(&self) -> &[char] { &self.n }
    pub fn base(&self) -> &[char] { &self.base }
    pub fn fixation_word(&self) -> &[char] { &self.fixation_word }
}

impl FixedPointQuantumMembrane {
    /// Consume the resident membrane and one opaque fixed phase sample `k/M`.
    ///
    /// This boundary only interprets an already-produced quantum readout. The
    /// sample must have been fixed from this exact resident `(a, N)` relation.
    /// `M` is the precision denominator; the existing Hadamard bridge may infer
    /// a candidate period from `k/M`. This function never repeats measurement,
    /// scans bases, walks a modular orbit, or searches factor candidates.
    pub fn descend_quantum_measurement(
        self,
        sample: QuantumPhaseSample,
    ) -> HadamardDescent {
        let width = self.n().len().saturating_mul(2).max(2);
        if sample.precision_denominator != power_of_two(width) {
            return HadamardDescent::F;
        }
        if sample.n.as_slice() != self.n() || sample.base.as_slice() != self.base() {
            return HadamardDescent::F;
        }
        if verdict(&sample.fixation_word) != 'T' {
            return HadamardDescent::F;
        }
        let base = sample.base.clone();
        let spectral = self.into_spectral();
        let carrier = spectral.into_carrier();
        carrier.descend_phase_sample(&base, &sample.numerator, &sample.precision_denominator)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixed_point_quantum_membrane::FixedPointQuantumMembrane;
    use crate::morphism_factor::tape_u64;

    fn landing_for(n: u64) -> PhaseLandingProgram {
        FixedPointQuantumMembrane::from_n(&tape_u64(n)).unwrap()
            .phase_estimation_register().unwrap()
            .into_measurement_program().unwrap()
            .into_landing_program().unwrap()
    }

    #[test]
    fn test_seam_consumes_resident_pair_before_advance_landing_into_k_over_m() {
        let landing = landing_for(257);
        let width = landing.width();
        let expected_n = landing.n().to_vec();
        let expected_base = landing.base().to_vec();
        let expected_word = resident_landing_word(&landing).unwrap();
        let sample = QuantumPhaseSample::fix_from_landing_for_test(landing, tape_u64(5)).unwrap();
        assert_eq!(sample.numerator(), tape_u64(5).as_slice());
        assert_eq!(sample.precision_denominator(), power_of_two(width).as_slice());
        assert_eq!(sample.denominator(), sample.precision_denominator());
        assert_eq!(sample.n(), expected_n.as_slice());
        assert_eq!(sample.base(), expected_base.as_slice());
        assert_eq!(sample.fixation_word(), expected_word.as_slice());
        assert_eq!(verdict(sample.fixation_word()), 'T');
        let link = sample.fixation_word().iter().position(|&mark| mark == CLINK).unwrap();
        let advance = sample.fixation_word().iter().position(|&mark| mark == AFWD).unwrap();
        assert!(link < advance);
    }

    #[test]
    fn test_seam_rejects_non_numeral_or_out_of_denominator_readout() {
        let landing = landing_for(257);
        let width = landing.width();
        assert!(QuantumPhaseSample::fix_from_landing_for_test(landing, power_of_two(width)).is_err());

        let landing = landing_for(257);
        assert!(QuantumPhaseSample::fix_from_landing_for_test(landing, vec!['⊙']).is_err());
    }

    #[test]
    fn sample_cannot_cross_resident_modulus_boundary() {
        let landing = landing_for(257);
        let sample = QuantumPhaseSample::fix_from_landing_for_test(landing, tape_u64(5)).unwrap();
        let other = FixedPointQuantumMembrane::from_n(&tape_u64(263)).unwrap();
        assert_eq!(other.descend_quantum_measurement(sample), HadamardDescent::F);
    }

    #[test]
    fn fixation_word_contains_the_resident_n_and_base() {
        for n in [257u64, 65_537, 4_294_967_291] {
            let landing = landing_for(n);
            let width = landing.width();
            let expected_n = landing.n().to_vec();
            let expected_base = landing.base().to_vec();
            let sample = QuantumPhaseSample::fix_from_landing_for_test(landing, tape_u64(1)).unwrap();
            assert!(sample.fixation_word().windows(expected_n.len()).any(|w| w == expected_n));
            assert!(sample.fixation_word().windows(expected_base.len()).any(|w| w == expected_base));
            assert_eq!(sample.precision_denominator(), power_of_two(width).as_slice());
        }
    }

    #[test]
    fn precision_denominator_is_not_relabelled_as_period() {
        let membrane = FixedPointQuantumMembrane::from_n(&tape_u64(21)).unwrap();
        let landing = membrane
            .phase_estimation_register().unwrap()
            .into_measurement_program().unwrap()
            .into_landing_program().unwrap();
        let width = landing.width();
        let precision_denominator = landing.denominator().to_vec();

        assert_eq!(precision_denominator, power_of_two(width));
        assert_ne!(membrane.modular_phase(&precision_denominator).unwrap(), tape_u64(1));
    }
}
