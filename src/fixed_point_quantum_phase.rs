//! Hypernested phase register for the fixed-point spectral membrane.
//!
//! The register is not expanded into basis addresses, controls, Fourier gates,
//! measurement wires, or one carrier object per nesting level.  Proper nesting
//! collapses to one canonical process word plus a carried depth.  That depth is
//! the integer winding itself: every embedding adds one `⊡`, while the single
//! live `≺` remains banked through the complete ancestry.
//!
//! A depth-t collapsed hypernest carries the complete `2^t` phase denominator
//! with one runtime collapse tick.  No modular orbit is enumerated to construct it.

use alloc::vec;

use crate::fixed_point_hypernest::CollapsedHypernest;
use crate::fixed_point_quantum_membrane::FixedPointQuantumMembrane;
use crate::hadamard_gate::Tape;
use crate::vox::{EVALF, EVALT};

/// Canonical IMASM numeral tape for 2^bit, LSB first.
pub(crate) fn power_of_two(bit: usize) -> Tape {
    let mut tape = vec![EVALT; bit + 1];
    tape[bit] = EVALF;
    tape
}

/// One coherent phase register represented by one collapsed hypernest.
///
/// `width == t` means both winding `t` and denominator `M = 2^t`.  Runtime
/// storage remains one carrier word plus the depth register.
#[derive(Clone, PartialEq, Debug)]
pub struct QuantumPhaseRegister {
    width: usize,
    denominator: Tape,
    hypernest: CollapsedHypernest,
}

impl QuantumPhaseRegister {
    pub fn width(&self) -> usize { self.width }
    pub fn denominator(&self) -> &[char] { &self.denominator }
    pub fn hypernest(&self) -> &CollapsedHypernest { &self.hypernest }

    /// Consume the coherent register into its one fixed-point measurement
    /// boundary.  No per-depth execution objects are created or retained.
    pub fn into_measurement_program(self) -> Result<PhaseMeasurementProgram, &'static str> {
        if self.hypernest.depth() != self.width || self.hypernest.winding() != self.width {
            return Err("quantum phase winding changed before fixation");
        }
        if self.hypernest.execution_ticks() != 1 {
            return Err("quantum phase hypernest failed to collapse in one composition");
        }
        let audit = self.hypernest.audit();
        if audit.live_clears != 1 || !audit.banked || audit.exposed != 0 {
            return Err("quantum phase clear escaped its hypernest bank");
        }
        if !audit.closed {
            return Err("quantum phase hypernest lost control-flow closure");
        }
        Ok(PhaseMeasurementProgram {
            width: self.width,
            denominator: self.denominator,
            hypernest: self.hypernest,
        })
    }
}

/// The one-shot fixed-point boundary immediately before the winding preimage is
/// fixed.  The complete represented denominator is carried by hypernest winding.
#[derive(Clone, PartialEq, Debug)]
pub struct PhaseMeasurementProgram {
    width: usize,
    denominator: Tape,
    hypernest: CollapsedHypernest,
}

impl PhaseMeasurementProgram {
    pub fn width(&self) -> usize { self.width }
    pub fn denominator(&self) -> &[char] { &self.denominator }
    pub fn hypernest(&self) -> &CollapsedHypernest { &self.hypernest }
    pub fn fixation_word(&self) -> &[char] { self.hypernest.word() }
    pub fn execution_ticks(&self) -> usize { self.hypernest.execution_ticks() }
}

impl FixedPointQuantumMembrane {
    /// Build the resident phase denominator as a collapsed hypernested winding.
    ///
    /// Width is fixed by N alone.  Depth t is winding t and carries denominator
    /// 2^t while execution stays one process word / one collapse tick.
    pub fn phase_estimation_register(&self) -> Result<QuantumPhaseRegister, &'static str> {
        let width = self.n().len().saturating_mul(2).max(2);
        let denominator = power_of_two(width);
        let hypernest = CollapsedHypernest::from_depth(width)?;
        let audit = hypernest.audit();
        if audit.winding != width
            || audit.live_clears != 1
            || !audit.banked
            || !audit.closed
            || hypernest.execution_ticks() != 1
        {
            return Err("quantum fixed-point collapsed hypernest invariant failed");
        }
        Ok(QuantumPhaseRegister { width, denominator, hypernest })
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
    fn denominator_is_two_to_hypernest_winding() {
        let membrane = FixedPointQuantumMembrane::from_n(&tape_u64(65_537)).unwrap();
        let register = membrane.phase_estimation_register().unwrap();
        assert_eq!(register.denominator(), power_of_two(register.width()).as_slice());
        assert_eq!(register.denominator().iter().filter(|&&c| c == EVALF).count(), 1);
        assert_eq!(register.denominator()[register.width()], EVALF);
        assert_eq!(register.hypernest().depth(), register.width());
        assert_eq!(register.hypernest().winding(), register.width());
    }

    #[test]
    fn represented_exponential_denominator_has_constant_runtime_shape() {
        let membrane = FixedPointQuantumMembrane::from_n(&tape_u64(257)).unwrap();
        let register = membrane.phase_estimation_register().unwrap();
        let t = register.width();
        assert_eq!(register.hypernest().word().len(), 11);
        assert_eq!(register.hypernest().execution_ticks(), 1);
        let audit = register.hypernest().audit();
        assert_eq!(audit.winding, t);
        assert_eq!(audit.live_clears, 1);
        assert_eq!(audit.exposed, 0);
        assert!(audit.banked);
        assert!(audit.closed);
    }

    #[test]
    fn measurement_boundary_retains_winding_not_expanded_depth() {
        let membrane = FixedPointQuantumMembrane::from_n(&tape_u64(257)).unwrap();
        let register = membrane.phase_estimation_register().unwrap();
        let width = register.width();
        let word = register.hypernest().word();
        let program = register.into_measurement_program().unwrap();
        assert_eq!(program.width(), width);
        assert_eq!(program.hypernest().depth(), width);
        assert_eq!(program.hypernest().winding(), width);
        assert_eq!(program.fixation_word(), word);
        assert_eq!(program.execution_ticks(), 1);
    }

    #[test]
    fn banking_cost_and_runtime_ticks_are_depth_invariant() {
        for n in [15u64, 257, 65_537] {
            let membrane = FixedPointQuantumMembrane::from_n(&tape_u64(n)).unwrap();
            let register = membrane.phase_estimation_register().unwrap();
            let audit = register.hypernest().audit();
            assert_eq!(audit.live_clears, 1);
            assert_eq!(audit.exposed, 0);
            assert!(audit.banked);
            assert_eq!(register.hypernest().execution_ticks(), 1);
        }
    }

    #[test]
    fn constructing_collapsed_denominator_does_not_advance_modular_phase_state() {
        let membrane = FixedPointQuantumMembrane::from_n(&tape_u64(257)).unwrap();
        let before = membrane.modular_phase(&tape_u64(7)).unwrap();
        let _ = membrane.phase_estimation_register().unwrap();
        let after = membrane.modular_phase(&tape_u64(7)).unwrap();
        assert_eq!(before, after);
    }
}
