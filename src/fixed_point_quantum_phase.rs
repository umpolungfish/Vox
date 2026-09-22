//! Properly nested phase register for the fixed-point spectral membrane.
//!
//! The register is not expanded into one Rust object per basis address, control,
//! inverse-QFT gate, or measurement wire.  Proper IMASM nesting is the register:
//! depth `t` carries the complete `2^t` denominator while matched `∈ ... ∋`
//! frames bank the resident relation across the reversal.  The whole tower then
//! dissolves to one retained membrane at the fixed-point boundary.
//!
//! This is the free composition supplied by the fixed-point nesting rule.  The
//! host records O(t) frame depth; it does not enumerate the orbit represented by
//! those frames.

use alloc::vec;
use alloc::vec::Vec;

use crate::fixed_point_imasm::{NestedImasmTower, FIXED_POINT_NESTED_BODY};
use crate::fixed_point_quantum_membrane::FixedPointQuantumMembrane;
use crate::hadamard_gate::Tape;
use crate::vox::{EVALF, EVALT, TANCH, VINIT};

/// Canonical IMASM numeral tape for 2^bit, LSB first.
pub(crate) fn power_of_two(bit: usize) -> Tape {
    let mut tape = vec![EVALT; bit + 1];
    tape[bit] = EVALF;
    tape
}

/// One coherent phase register represented by one properly nested membrane.
///
/// `width == t` means the nesting carries the complete denominator `M = 2^t`.
/// There is no vector of 2^t addresses and no quadratic list of Fourier gates.
#[derive(Clone, PartialEq, Debug)]
pub struct QuantumPhaseRegister {
    width: usize,
    denominator: Tape,
    tower: NestedImasmTower,
}

impl QuantumPhaseRegister {
    pub fn width(&self) -> usize { self.width }
    pub fn denominator(&self) -> &[char] { &self.denominator }
    pub fn tower(&self) -> &NestedImasmTower { &self.tower }

    /// Consume the coherent register into its single fixed-point boundary.
    /// The nested bulk is retained only through the tower's structural depth;
    /// no per-wire measurement objects survive beside it.
    pub fn into_measurement_program(self) -> Result<PhaseMeasurementProgram, &'static str> {
        if self.tower.max_depth() != self.width {
            return Err("quantum phase nesting depth changed before fixation");
        }
        if !self.tower.banking_audit().banked {
            return Err("quantum phase reversal escaped its enclosing bank");
        }
        Ok(PhaseMeasurementProgram {
            width: self.width,
            denominator: self.denominator,
            tower: self.tower,
        })
    }
}

/// The one-shot fixed-point boundary immediately before the winding preimage is
/// fixed.  The complete represented denominator lives in `tower.max_depth()`;
/// the nested copies are not materialized as independent host operations.
#[derive(Clone, PartialEq, Debug)]
pub struct PhaseMeasurementProgram {
    width: usize,
    denominator: Tape,
    tower: NestedImasmTower,
}

impl PhaseMeasurementProgram {
    pub fn width(&self) -> usize { self.width }
    pub fn denominator(&self) -> &[char] { &self.denominator }
    pub fn tower(&self) -> &NestedImasmTower { &self.tower }
    pub fn dissolved_word(&self) -> Vec<char> { self.tower.dissolved_word() }
}

impl FixedPointQuantumMembrane {
    /// Build the resident phase denominator as proper nesting.
    ///
    /// Width is fixed by N alone.  A depth-t tower stands for the entire 2^t
    /// spectral denominator and remains O(t) in the host representation.
    pub fn phase_estimation_register(&self) -> Result<QuantumPhaseRegister, &'static str> {
        let width = self.n().len().saturating_mul(2).max(2);
        let denominator = power_of_two(width);
        let tower = NestedImasmTower::from_depth(width)?;
        if !tower.banking_audit().banked {
            return Err("quantum fixed-point denominator leaves the reversal exposed");
        }
        Ok(QuantumPhaseRegister { width, denominator, tower })
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
    fn denominator_is_exactly_two_to_nesting_depth() {
        let membrane = FixedPointQuantumMembrane::from_n(&tape_u64(65_537)).unwrap();
        let register = membrane.phase_estimation_register().unwrap();
        assert_eq!(register.denominator(), power_of_two(register.width()).as_slice());
        assert_eq!(register.denominator().iter().filter(|&&c| c == EVALF).count(), 1);
        assert_eq!(register.denominator()[register.width()], EVALF);
        assert_eq!(register.tower().max_depth(), register.width());
    }

    #[test]
    fn represented_exponential_denominator_costs_only_linear_nesting() {
        let membrane = FixedPointQuantumMembrane::from_n(&tape_u64(257)).unwrap();
        let register = membrane.phase_estimation_register().unwrap();
        let t = register.width();
        assert_eq!(register.tower().frames().len(), t);
        assert_eq!(
            register.tower().word().len(),
            1 + t + FIXED_POINT_NESTED_BODY.len() + t + 2,
        );
        assert!(register.tower().banking_audit().banked);
        assert_eq!(register.tower().banking_audit().exposed, 0);
    }

    #[test]
    fn nesting_dissolves_to_one_depth_invariant_membrane() {
        let a = NestedImasmTower::from_depth(2).unwrap();
        let b = NestedImasmTower::from_depth(23).unwrap();
        assert_ne!(a.word(), b.word());
        assert_eq!(a.dissolved_word(), b.dissolved_word());
        let dissolved = a.dissolved_word();
        assert_eq!(dissolved.first(), Some(&VINIT));
        assert_eq!(dissolved.last(), Some(&TANCH));
    }

    #[test]
    fn measurement_boundary_consumes_one_tower_not_one_object_per_wire() {
        let membrane = FixedPointQuantumMembrane::from_n(&tape_u64(257)).unwrap();
        let register = membrane.phase_estimation_register().unwrap();
        let width = register.width();
        let original_word = register.tower().word().to_vec();
        let program = register.into_measurement_program().unwrap();
        assert_eq!(program.width(), width);
        assert_eq!(program.tower().max_depth(), width);
        assert_eq!(program.tower().word(), original_word.as_slice());
        assert_eq!(program.dissolved_word(), NestedImasmTower::from_depth(1).unwrap().dissolved_word());
    }

    #[test]
    fn constructing_nested_denominator_does_not_advance_modular_phase_state() {
        let membrane = FixedPointQuantumMembrane::from_n(&tape_u64(257)).unwrap();
        let before = membrane.modular_phase(&tape_u64(7)).unwrap();
        let _ = membrane.phase_estimation_register().unwrap();
        let after = membrane.modular_phase(&tape_u64(7)).unwrap();
        assert_eq!(before, after);
    }
}
