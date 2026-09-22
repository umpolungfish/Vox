//! One-shot readout boundary for the collapsed hypernested fixed-point membrane.
//!
//! A `QuantumPhaseSample` has no production constructor that accepts a host
//! numerator. The collapsed hypernest carries the QPE precision denominator
//! `M = 2^t`; it does not by itself claim the multiplicative period `r`.
//! The resident membrane first collapses to an opaque `QuantumWindingPreimage`
//! at the explicit pair-before-advance landing. Only that exact preimage can be
//! consumed by the internal measurement fixation seam, so an arbitrary `k/M`
//! cannot be attached to a generic landing after the fact. The existing
//! Hadamard bridge may infer a period from a genuine fixed phase sample `k/M`;
//! this module never relabels `M` as that period and never walks an orbit.

use core::cmp::Ordering;

use alloc::vec::Vec;

use crate::fixed_point_quantum_membrane::FixedPointQuantumMembrane;
use crate::fixed_point_quantum_phase::{power_of_two, PhaseLandingProgram};
use crate::fixed_point_quantum_relation::resident_landing_word;
use crate::hadamard_factor_bridge::HadamardDescent;
use crate::hadamard_gate::Tape;
use crate::morphism_factor::cmp;
use crate::vox::{verdict, AFWD, CLINK, EVALF, EVALT};

/// Opaque one-shot winding preimage produced from N alone by the resident
/// collapsed membrane.
///
/// This object is deliberately not `Clone`: the pair-before-advance landing is
/// a consumable measurement boundary, not reusable metadata. It contains no
/// measured numerator, no multiplicative period, no factor, and no orbit cursor.
/// The only denominator present here is the QPE precision denominator `M=2^t`.
#[derive(PartialEq, Debug)]
pub struct QuantumWindingPreimage {
    landing: PhaseLandingProgram,
    fixation_word: Vec<char>,
}

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

impl QuantumWindingPreimage {
    /// Collapse one already-formed landing into the unique resident preimage.
    /// All structural checks happen before any measurement result can exist.
    fn from_landing(landing: PhaseLandingProgram) -> Result<Self, &'static str> {
        let fixation_word = resident_landing_word(&landing)?;
        if verdict(&fixation_word) != 'T' {
            return Err("resident quantum phase landing does not close");
        }
        let link = fixation_word
            .iter()
            .position(|&mark| mark == CLINK)
            .ok_or("quantum phase landing lost its spectral pair")?;
        let advance = fixation_word
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
        if landing.denominator() != power_of_two(width).as_slice() {
            return Err("quantum phase landing precision denominator changed");
        }

        Ok(Self { landing, fixation_word })
    }

    pub fn width(&self) -> usize { self.landing.width() }

    /// QPE precision denominator represented by the collapsed hypernest.
    pub fn precision_denominator(&self) -> &[char] { self.landing.denominator() }

    pub fn n(&self) -> &[char] { self.landing.n() }
    pub fn base(&self) -> &[char] { self.landing.base() }
    pub fn fixation_word(&self) -> &[char] { &self.fixation_word }
    pub fn execution_ticks(&self) -> usize { self.landing.execution_ticks() }
}

impl QuantumPhaseSample {
    /// Test-only seam for validating fixation of an opaque one-shot preimage.
    /// Production code has no API that accepts an arbitrary k/M pair.
    #[cfg(test)]
    fn fix_preimage_for_test(
        preimage: QuantumWindingPreimage,
        numerator: Tape,
    ) -> Result<Self, &'static str> {
        Self::from_executor_preimage(preimage, numerator)
    }

    /// Private fixation seam reserved for the measurement executor. The opaque
    /// preimage is consumed, so a result can only be attached to the exact
    /// resident `(a,N,M)` object whose pair was formed before advance.
    ///
    /// The numeric outcome is not derived here and this function does not fake a
    /// measurement by scanning the modular orbit, enumerating candidate periods,
    /// or relabelling `M` as `r`.
    fn from_executor_preimage(
        preimage: QuantumWindingPreimage,
        numerator: Tape,
    ) -> Result<Self, &'static str> {
        if !valid_tape(&numerator) {
            return Err("quantum phase landing produced a malformed numeral");
        }

        let QuantumWindingPreimage { landing, fixation_word } = preimage;
        let precision_denominator = landing.denominator().to_vec();
        if cmp(&numerator, &precision_denominator) != Ordering::Less {
            return Err("quantum phase landing lies outside its precision denominator");
        }

        Ok(Self {
            numerator,
            precision_denominator,
            n: landing.n().to_vec(),
            base: landing.base().to_vec(),
            fixation_word,
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
    /// Collapse the complete N-dependent quantum membrane into its one-shot
    /// winding preimage without accepting a phase numerator from the caller.
    ///
    /// This is the executable N -> resident landing -> preimage boundary. It
    /// performs one collapsed hypernest execution and carries `(a,N,M)` through
    /// pair-before-advance fixation. It does not measure `k`, discover `r`, walk
    /// an orbit, scan bases, or search factors.
    pub fn one_shot_winding_preimage(&self) -> Result<QuantumWindingPreimage, &'static str> {
        let landing = self
            .phase_estimation_register()?
            .into_measurement_program()?
            .into_landing_program()?;
        QuantumWindingPreimage::from_landing(landing)
    }

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

    fn preimage_for(n: u64) -> QuantumWindingPreimage {
        FixedPointQuantumMembrane::from_n(&tape_u64(n)).unwrap()
            .one_shot_winding_preimage().unwrap()
    }

    #[test]
    fn n_only_membrane_collapses_to_one_shot_winding_preimage() {
        let membrane = FixedPointQuantumMembrane::from_n(&tape_u64(257)).unwrap();
        let expected_n = membrane.n().to_vec();
        let expected_base = membrane.base().to_vec();
        let expected_width = membrane.n().len() * 2;
        let preimage = membrane.one_shot_winding_preimage().unwrap();

        assert_eq!(preimage.width(), expected_width);
        assert_eq!(preimage.precision_denominator(), power_of_two(expected_width).as_slice());
        assert_eq!(preimage.n(), expected_n.as_slice());
        assert_eq!(preimage.base(), expected_base.as_slice());
        assert_eq!(preimage.execution_ticks(), 1);
        assert_eq!(verdict(preimage.fixation_word()), 'T');
        let link = preimage.fixation_word().iter().position(|&mark| mark == CLINK).unwrap();
        let advance = preimage.fixation_word().iter().position(|&mark| mark == AFWD).unwrap();
        assert!(link < advance);
    }

    #[test]
    fn test_seam_consumes_resident_preimage_into_k_over_m() {
        let preimage = preimage_for(257);
        let width = preimage.width();
        let expected_n = preimage.n().to_vec();
        let expected_base = preimage.base().to_vec();
        let expected_word = preimage.fixation_word().to_vec();
        let sample = QuantumPhaseSample::fix_preimage_for_test(preimage, tape_u64(5)).unwrap();
        assert_eq!(sample.numerator(), tape_u64(5).as_slice());
        assert_eq!(sample.precision_denominator(), power_of_two(width).as_slice());
        assert_eq!(sample.denominator(), sample.precision_denominator());
        assert_eq!(sample.n(), expected_n.as_slice());
        assert_eq!(sample.base(), expected_base.as_slice());
        assert_eq!(sample.fixation_word(), expected_word.as_slice());
        assert_eq!(verdict(sample.fixation_word()), 'T');
    }

    #[test]
    fn test_seam_rejects_non_numeral_or_out_of_denominator_readout() {
        let preimage = preimage_for(257);
        let width = preimage.width();
        assert!(QuantumPhaseSample::fix_preimage_for_test(preimage, power_of_two(width)).is_err());

        let preimage = preimage_for(257);
        assert!(QuantumPhaseSample::fix_preimage_for_test(preimage, vec!['⊙']).is_err());
    }

    #[test]
    fn sample_cannot_cross_resident_modulus_boundary() {
        let preimage = preimage_for(257);
        let sample = QuantumPhaseSample::fix_preimage_for_test(preimage, tape_u64(5)).unwrap();
        let other = FixedPointQuantumMembrane::from_n(&tape_u64(263)).unwrap();
        assert_eq!(other.descend_quantum_measurement(sample), HadamardDescent::F);
    }

    #[test]
    fn fixation_word_contains_the_resident_n_and_base() {
        for n in [257u64, 65_537, 4_294_967_291] {
            let preimage = preimage_for(n);
            let width = preimage.width();
            let expected_n = preimage.n().to_vec();
            let expected_base = preimage.base().to_vec();
            let sample = QuantumPhaseSample::fix_preimage_for_test(preimage, tape_u64(1)).unwrap();
            assert!(sample.fixation_word().windows(expected_n.len()).any(|w| w == expected_n));
            assert!(sample.fixation_word().windows(expected_base.len()).any(|w| w == expected_base));
            assert_eq!(sample.precision_denominator(), power_of_two(width).as_slice());
        }
    }

    #[test]
    fn precision_denominator_is_not_relabelled_as_period() {
        let membrane = FixedPointQuantumMembrane::from_n(&tape_u64(21)).unwrap();
        let preimage = membrane.one_shot_winding_preimage().unwrap();
        let precision_denominator = preimage.precision_denominator().to_vec();

        assert_eq!(precision_denominator, power_of_two(preimage.width()));
        assert_ne!(membrane.modular_phase(&precision_denominator).unwrap(), tape_u64(1));
    }
}
