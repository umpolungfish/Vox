//! Hypernested phase register for the fixed-point spectral membrane.
//!
//! The register is not expanded into basis addresses, controls, Fourier gates,
//! measurement wires, or one carrier object per nesting level. Proper nesting
//! collapses to one canonical process word plus a carried depth. That depth is
//! the integer winding itself: every embedding adds one `⊡`, while the single
//! live `≺` remains banked through the complete ancestry.
//!
//! A depth-t collapsed hypernest carries the complete `2^t` phase denominator
//! with one runtime collapse tick. The resident modulus and base travel with the
//! collapsed register so the landing boundary remains genuinely N-dependent.

use alloc::vec;

use crate::fixed_point_hypernest::CollapsedHypernest;
use crate::fixed_point_quantum_membrane::FixedPointQuantumMembrane;
use crate::hadamard_gate::Tape;
use crate::vox::{
    verdict, AFWD, AREV, CLINK, EVALF, EVALT, FFUSE, FSPLIT, IFIX, IMSCRIB, TANCH,
    VINIT,
};

/// Canonical IMASM numeral tape for 2^bit, LSB first.
pub(crate) fn power_of_two(bit: usize) -> Tape {
    let mut tape = vec![EVALT; bit + 1];
    tape[bit] = EVALF;
    tape
}

/// Properly nested pair-before-advance landing.
///
/// This is an enclosure, not two complete words concatenated.  The outer hold
/// opens first, the inner readout payload is spliced directly against its fuse,
/// the outer fuse closes after it, and there is exactly one terminal fixation
/// after both fuses.  `CLINK` therefore forms the spectral pair before `AFWD`
/// transports it, while the one live `AREV` remains banked through both frames.
pub const PAIR_BEFORE_ADVANCE_READOUT_WORD: [char; 14] = [
    VINIT,
    FSPLIT,
    EVALT,
    IMSCRIB,
    FSPLIT,
    EVALT,
    AREV,
    EVALF,
    CLINK,
    AFWD,
    FFUSE,
    FFUSE,
    IFIX,
    TANCH,
];

/// One coherent phase register represented by one collapsed hypernest.
///
/// `width == t` means both winding `t` and denominator `M = 2^t`. Runtime
/// storage remains one carrier word plus the depth register and the resident
/// modular relation `(a, N)`.
#[derive(Clone, PartialEq, Debug)]
pub struct QuantumPhaseRegister {
    width: usize,
    denominator: Tape,
    n: Tape,
    base: Tape,
    hypernest: CollapsedHypernest,
}

impl QuantumPhaseRegister {
    pub fn width(&self) -> usize { self.width }
    pub fn denominator(&self) -> &[char] { &self.denominator }
    pub fn n(&self) -> &[char] { &self.n }
    pub fn base(&self) -> &[char] { &self.base }
    pub fn hypernest(&self) -> &CollapsedHypernest { &self.hypernest }

    /// Consume the coherent register into its one fixed-point measurement
    /// boundary. No per-depth execution objects are created or retained.
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
        if self.n.is_empty() || self.base.is_empty() {
            return Err("quantum phase register lost its resident modular relation");
        }
        Ok(PhaseMeasurementProgram {
            width: self.width,
            denominator: self.denominator,
            n: self.n,
            base: self.base,
            hypernest: self.hypernest,
        })
    }
}

/// The one-shot fixed-point boundary immediately before the winding preimage is
/// fixed. The complete represented denominator is carried by hypernest winding,
/// while `(a, N)` remains resident and opaque.
#[derive(Clone, PartialEq, Debug)]
pub struct PhaseMeasurementProgram {
    width: usize,
    denominator: Tape,
    n: Tape,
    base: Tape,
    hypernest: CollapsedHypernest,
}

impl PhaseMeasurementProgram {
    pub fn width(&self) -> usize { self.width }
    pub fn denominator(&self) -> &[char] { &self.denominator }
    pub fn n(&self) -> &[char] { &self.n }
    pub fn base(&self) -> &[char] { &self.base }
    pub fn hypernest(&self) -> &CollapsedHypernest { &self.hypernest }
    pub fn fixation_word(&self) -> &[char] { self.hypernest.word() }
    pub fn execution_ticks(&self) -> usize { self.hypernest.execution_ticks() }

    /// Consume the measurement boundary into one properly nested landing.
    /// The inner readout has no independent VINIT/TANCH/IFIX interface: its
    /// payload is enclosed by the outer frame and the only fixation is terminal.
    /// The denominator, resident modular relation, and winding move together.
    pub fn into_landing_program(self) -> Result<PhaseLandingProgram, &'static str> {
        let word = &PAIR_BEFORE_ADVANCE_READOUT_WORD;
        let link = word
            .iter()
            .position(|&mark| mark == CLINK)
            .ok_or("quantum landing has no pair operation")?;
        let advance = word
            .iter()
            .position(|&mark| mark == AFWD)
            .ok_or("quantum landing has no transport operation")?;
        let clear = word
            .iter()
            .position(|&mark| mark == AREV)
            .ok_or("quantum landing has no live clear")?;
        let outer_split = word
            .iter()
            .position(|&mark| mark == FSPLIT)
            .ok_or("quantum landing has no outer bank")?;
        let inner_split = word
            .iter()
            .enumerate()
            .filter_map(|(at, &mark)| (mark == FSPLIT).then_some(at))
            .nth(1)
            .ok_or("quantum landing has no nested bank")?;
        let inner_fuse = word
            .iter()
            .position(|&mark| mark == FFUSE)
            .ok_or("quantum landing has no inner fuse")?;
        let outer_fuse = word
            .iter()
            .rposition(|&mark| mark == FFUSE)
            .ok_or("quantum landing has no outer fuse")?;
        let fixation = word
            .iter()
            .position(|&mark| mark == IFIX)
            .ok_or("quantum landing has no terminal fixation")?;

        if link >= advance {
            return Err("quantum landing advances before forming the spectral pair");
        }
        if word.iter().filter(|&&mark| mark == FSPLIT).count() != 2
            || word.iter().filter(|&&mark| mark == FFUSE).count() != 2
            || word.iter().filter(|&&mark| mark == IFIX).count() != 1
            || word.iter().filter(|&&mark| mark == VINIT).count() != 1
            || word.iter().filter(|&&mark| mark == TANCH).count() != 1
        {
            return Err("quantum landing is juxtaposed words instead of one nested carrier");
        }
        if !(outer_split < inner_split
            && inner_split < clear
            && clear < inner_fuse
            && inner_fuse < outer_fuse
            && outer_fuse < fixation)
        {
            return Err("quantum landing lost enclosure or banked-clear ordering");
        }
        if verdict(word) != 'T' {
            return Err("quantum landing boundary does not close");
        }
        if self.hypernest.winding() != self.width || self.hypernest.execution_ticks() != 1 {
            return Err("quantum landing lost collapsed hypernest winding");
        }
        if self.n.is_empty() || self.base.is_empty() {
            return Err("quantum landing lost its resident modular relation");
        }
        Ok(PhaseLandingProgram {
            width: self.width,
            denominator: self.denominator,
            n: self.n,
            base: self.base,
            hypernest: self.hypernest,
        })
    }
}

/// Opaque landing boundary immediately before the N-dependent winding preimage
/// is minted. It contains the resident `(a, N)` relation and denominator, but no
/// numerator, period, factor, or orbit cursor.
#[derive(Clone, PartialEq, Debug)]
pub struct PhaseLandingProgram {
    width: usize,
    denominator: Tape,
    n: Tape,
    base: Tape,
    hypernest: CollapsedHypernest,
}

impl PhaseLandingProgram {
    pub fn width(&self) -> usize { self.width }
    pub fn denominator(&self) -> &[char] { &self.denominator }
    pub fn n(&self) -> &[char] { &self.n }
    pub fn base(&self) -> &[char] { &self.base }
    pub fn hypernest(&self) -> &CollapsedHypernest { &self.hypernest }
    pub fn readout_word(&self) -> &'static [char] { &PAIR_BEFORE_ADVANCE_READOUT_WORD }
    pub fn execution_ticks(&self) -> usize { self.hypernest.execution_ticks() }
}

impl FixedPointQuantumMembrane {
    /// Build the resident phase denominator as a collapsed hypernested winding.
    ///
    /// Width is fixed by N alone. Depth t is winding t and carries denominator
    /// 2^t while execution stays one process word / one collapse tick. The same
    /// N and canonical base remain attached all the way to the landing boundary.
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
        Ok(QuantumPhaseRegister {
            width,
            denominator,
            n: self.n().to_vec(),
            base: self.base().to_vec(),
            hypernest,
        })
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
    fn resident_modular_relation_survives_to_landing() {
        let membrane = FixedPointQuantumMembrane::from_n(&tape_u64(257)).unwrap();
        let expected_n = membrane.n().to_vec();
        let expected_base = membrane.base().to_vec();
        let landing = membrane
            .phase_estimation_register().unwrap()
            .into_measurement_program().unwrap()
            .into_landing_program().unwrap();
        assert_eq!(landing.n(), expected_n.as_slice());
        assert_eq!(landing.base(), expected_base.as_slice());
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
        let expected_n = register.n().to_vec();
        let expected_base = register.base().to_vec();
        let program = register.into_measurement_program().unwrap();
        assert_eq!(program.width(), width);
        assert_eq!(program.hypernest().depth(), width);
        assert_eq!(program.hypernest().winding(), width);
        assert_eq!(program.fixation_word(), word);
        assert_eq!(program.execution_ticks(), 1);
        assert_eq!(program.n(), expected_n.as_slice());
        assert_eq!(program.base(), expected_base.as_slice());
    }

    #[test]
    fn landing_forms_pair_before_advance_without_expanding_the_hypernest() {
        let membrane = FixedPointQuantumMembrane::from_n(&tape_u64(257)).unwrap();
        let landing = membrane
            .phase_estimation_register().unwrap()
            .into_measurement_program().unwrap()
            .into_landing_program().unwrap();
        let word = landing.readout_word();
        let link = word.iter().position(|&mark| mark == CLINK).unwrap();
        let advance = word.iter().position(|&mark| mark == AFWD).unwrap();
        assert!(link < advance);
        assert_eq!(verdict(word), 'T');
        assert_eq!(landing.execution_ticks(), 1);
        assert_eq!(landing.hypernest().winding(), landing.width());
        assert_eq!(word.len(), PAIR_BEFORE_ADVANCE_READOUT_WORD.len());
    }

    #[test]
    fn landing_is_enclosure_not_concatenated_complete_words() {
        let landing = FixedPointQuantumMembrane::from_n(&tape_u64(257)).unwrap()
            .phase_estimation_register().unwrap()
            .into_measurement_program().unwrap()
            .into_landing_program().unwrap();
        let word = landing.readout_word();
        let splits: alloc::vec::Vec<usize> = word.iter().enumerate()
            .filter_map(|(at, &mark)| (mark == FSPLIT).then_some(at)).collect();
        let fuses: alloc::vec::Vec<usize> = word.iter().enumerate()
            .filter_map(|(at, &mark)| (mark == FFUSE).then_some(at)).collect();
        let clear = word.iter().position(|&mark| mark == AREV).unwrap();
        let fix = word.iter().position(|&mark| mark == IFIX).unwrap();

        assert_eq!(splits.len(), 2);
        assert_eq!(fuses.len(), 2);
        assert!(splits[0] < splits[1]);
        assert!(splits[1] < clear && clear < fuses[0]);
        assert!(fuses[0] < fuses[1] && fuses[1] < fix);
        assert_eq!(word.iter().filter(|&&mark| mark == VINIT).count(), 1);
        assert_eq!(word.iter().filter(|&&mark| mark == TANCH).count(), 1);
        assert_eq!(word.iter().filter(|&&mark| mark == IFIX).count(), 1);
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
