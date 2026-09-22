//! Structural quantum phase-estimation register for the fixed-point membrane.
//!
//! Nothing in this module searches a period or a factor.  The register width is
//! determined only by the resident IMASM numeral width of N.  Each controlled
//! modular phase is addressed independently by the basis exponent 2^j, and the
//! inverse-QFT is retained as an explicit IMASM gate topology.  No amplitude
//! vector and no modular orbit are materialized.

use alloc::vec;
use alloc::vec::Vec;

use crate::fixed_point_quantum_membrane::FixedPointQuantumMembrane;
use crate::hadamard_gate::Tape;
use crate::vox::{AFWD, CLINK, EVALF, EVALT, FFUSE, FSPLIT, IFIX, IMSCRIB};

/// Pair first, then advance.  This ordering is intentional: split/link creates
/// the resident pair before the punctum transport acts on it.
pub const CONTROLLED_MODULAR_PHASE_WORD: [char; 6] =
    [FSPLIT, CLINK, AFWD, IMSCRIB, IFIX, FFUSE];
pub const INVERSE_QFT_HADAMARD_WORD: [char; 4] = [FSPLIT, EVALT, EVALF, FFUSE];
pub const INVERSE_QFT_PHASE_WORD: [char; 4] = [FSPLIT, CLINK, IMSCRIB, FFUSE];
pub const INVERSE_QFT_SWAP_WORD: [char; 1] = [CLINK];

fn power_of_two(bit: usize) -> Tape {
    let mut tape = vec![EVALT; bit + 1];
    tape[bit] = EVALF;
    tape
}

#[derive(Clone, PartialEq, Debug)]
pub struct ControlledModularPhase {
    control: usize,
    exponent: Tape,
    phase: Tape,
    operator_word: Vec<char>,
}

impl ControlledModularPhase {
    pub fn control(&self) -> usize { self.control }
    pub fn exponent(&self) -> &[char] { &self.exponent }
    pub fn phase(&self) -> &[char] { &self.phase }
    pub fn operator_word(&self) -> &[char] { &self.operator_word }
}

#[derive(Clone, PartialEq, Debug)]
pub enum InverseQftGate {
    ControlledPhase {
        control: usize,
        target: usize,
        distance: usize,
        operator_word: Vec<char>,
    },
    Hadamard {
        wire: usize,
        operator_word: Vec<char>,
    },
    Swap {
        left: usize,
        right: usize,
        operator_word: Vec<char>,
    },
}

impl InverseQftGate {
    pub fn operator_word(&self) -> &[char] {
        match self {
            Self::ControlledPhase { operator_word, .. }
            | Self::Hadamard { operator_word, .. }
            | Self::Swap { operator_word, .. } => operator_word,
        }
    }
}

/// A phase-estimation circuit description resident in the membrane.
///
/// `denominator` is exactly M=2^width as an IMASM numeral tape.  There is no
/// phase sample field: a sample is a later measurement product, not an input or
/// a compile-time hint.
#[derive(Clone, PartialEq, Debug)]
pub struct QuantumPhaseRegister {
    width: usize,
    denominator: Tape,
    controls: Vec<ControlledModularPhase>,
    inverse_qft: Vec<InverseQftGate>,
}

impl QuantumPhaseRegister {
    pub fn width(&self) -> usize { self.width }
    pub fn denominator(&self) -> &[char] { &self.denominator }
    pub fn controls(&self) -> &[ControlledModularPhase] { &self.controls }
    pub fn inverse_qft(&self) -> &[InverseQftGate] { &self.inverse_qft }

    pub fn hadamard_count(&self) -> usize {
        self.inverse_qft.iter().filter(|g| matches!(g, InverseQftGate::Hadamard { .. })).count()
    }

    pub fn controlled_phase_count(&self) -> usize {
        self.inverse_qft.iter().filter(|g| matches!(g, InverseQftGate::ControlledPhase { .. })).count()
    }

    pub fn swap_count(&self) -> usize {
        self.inverse_qft.iter().filter(|g| matches!(g, InverseQftGate::Swap { .. })).count()
    }
}

impl FixedPointQuantumMembrane {
    /// Build the structural phase-estimation register.
    ///
    /// Two control cells per resident N cell provide the conventional exact
    /// rational-readout precision budget without inspecting N's factors or an
    /// order.  All controlled modular phases are independent basis-addressed
    /// evaluations of a^(2^j) mod N.
    pub fn phase_estimation_register(&self) -> Result<QuantumPhaseRegister, &'static str> {
        let width = self.n().len().saturating_mul(2).max(2);
        let denominator = power_of_two(width);
        let mut controls = Vec::with_capacity(width);

        for control in 0..width {
            let exponent = power_of_two(control);
            let phase = self.modular_phase(&exponent)?;
            controls.push(ControlledModularPhase {
                control,
                exponent,
                phase,
                operator_word: CONTROLLED_MODULAR_PHASE_WORD.to_vec(),
            });
        }

        // Inverse-QFT topology.  The graph is polynomial in register width; no
        // state-vector simulation is performed here.
        let mut inverse_qft = Vec::new();
        for target in 0..width {
            for control in (target + 1)..width {
                inverse_qft.push(InverseQftGate::ControlledPhase {
                    control,
                    target,
                    distance: control - target,
                    operator_word: INVERSE_QFT_PHASE_WORD.to_vec(),
                });
            }
            inverse_qft.push(InverseQftGate::Hadamard {
                wire: target,
                operator_word: INVERSE_QFT_HADAMARD_WORD.to_vec(),
            });
        }
        for left in 0..(width / 2) {
            inverse_qft.push(InverseQftGate::Swap {
                left,
                right: width - 1 - left,
                operator_word: INVERSE_QFT_SWAP_WORD.to_vec(),
            });
        }

        Ok(QuantumPhaseRegister { width, denominator, controls, inverse_qft })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::morphism_factor::tape_u64;

    #[test]
    fn phase_width_depends_only_on_resident_n_width() {
        let a = FixedPointQuantumMembrane::from_n(&tape_u64(257)).unwrap();
        let b = FixedPointQuantumMembrane::from_n(&tape_u64(263)).unwrap();
        let ra = a.phase_estimation_register().unwrap();
        let rb = b.phase_estimation_register().unwrap();
        assert_eq!(ra.width(), rb.width());
        assert_eq!(ra.width(), 2 * a.n().len());
    }

    #[test]
    fn controls_are_exact_power_of_two_basis_addresses() {
        let membrane = FixedPointQuantumMembrane::from_n(&tape_u64(257)).unwrap();
        let register = membrane.phase_estimation_register().unwrap();
        for (j, control) in register.controls().iter().enumerate() {
            assert_eq!(control.control(), j);
            assert_eq!(control.exponent(), power_of_two(j).as_slice());
            assert_eq!(control.phase(), membrane.modular_phase(control.exponent()).unwrap().as_slice());
            assert_eq!(control.operator_word(), CONTROLLED_MODULAR_PHASE_WORD);
            assert_eq!(control.operator_word()[0], FSPLIT);
            assert_eq!(control.operator_word()[1], CLINK);
            assert_eq!(control.operator_word()[2], AFWD);
        }
    }

    #[test]
    fn denominator_is_exactly_two_to_register_width() {
        let membrane = FixedPointQuantumMembrane::from_n(&tape_u64(65_537)).unwrap();
        let register = membrane.phase_estimation_register().unwrap();
        assert_eq!(register.denominator(), power_of_two(register.width()).as_slice());
        assert_eq!(register.denominator().iter().filter(|&&c| c == EVALF).count(), 1);
        assert_eq!(register.denominator()[register.width()], EVALF);
    }

    #[test]
    fn inverse_qft_is_explicit_polynomial_topology() {
        let membrane = FixedPointQuantumMembrane::from_n(&tape_u64(257)).unwrap();
        let register = membrane.phase_estimation_register().unwrap();
        let t = register.width();
        assert_eq!(register.hadamard_count(), t);
        assert_eq!(register.controlled_phase_count(), t * (t - 1) / 2);
        assert_eq!(register.swap_count(), t / 2);
        assert_eq!(register.inverse_qft().len(), t + t * (t - 1) / 2 + t / 2);
        assert!(register.inverse_qft().iter().all(|g| !g.operator_word().is_empty()));
    }

    #[test]
    fn constructing_register_does_not_advance_hidden_phase_state() {
        let membrane = FixedPointQuantumMembrane::from_n(&tape_u64(257)).unwrap();
        let before = membrane.modular_phase(&tape_u64(7)).unwrap();
        let _ = membrane.phase_estimation_register().unwrap();
        let after = membrane.modular_phase(&tape_u64(7)).unwrap();
        assert_eq!(before, after);
    }
}
