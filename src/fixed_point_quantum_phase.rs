//! Structural quantum phase-estimation register for the fixed-point membrane.
//!
//! Nothing in this module searches a period or a factor. Register width is
//! determined only by the resident IMASM numeral width of N. Controlled modular
//! phases are addressed independently by the basis exponent 2^j, and every
//! quantum operation carries explicit matched `∈ ... ∋` nesting. The inverse-QFT
//! and final measurement boundary are IMASM topology, not host-side amplitude
//! simulation.

use alloc::vec;
use alloc::vec::Vec;

use crate::fixed_point_imasm::FrameEdge;
use crate::fixed_point_quantum_membrane::FixedPointQuantumMembrane;
use crate::hadamard_gate::Tape;
use crate::vox::{
    AFWD, CLINK, EVALF, EVALT, FFUSE, FSPLIT, IFIX, IMSCRIB, TANCH, VINIT,
};

/// Pair first, then advance. The pair/link exists before punctum transport.
pub const CONTROLLED_MODULAR_PHASE_WORD: [char; 6] =
    [FSPLIT, CLINK, AFWD, IMSCRIB, IFIX, FFUSE];
pub const INVERSE_QFT_HADAMARD_WORD: [char; 4] = [FSPLIT, EVALT, EVALF, FFUSE];
pub const INVERSE_QFT_PHASE_WORD: [char; 4] = [FSPLIT, CLINK, IMSCRIB, FFUSE];
pub const INVERSE_QFT_SWAP_WORD: [char; 1] = [CLINK];
/// Measurement is also pair-first: expose both Boolean-core arms, fuse, fix.
pub const PHASE_MEASUREMENT_WORD: [char; 6] = [FSPLIT, CLINK, EVALT, EVALF, FFUSE, IFIX];

pub(crate) fn power_of_two(bit: usize) -> Tape {
    let mut tape = vec![EVALT; bit + 1];
    tape[bit] = EVALF;
    tape
}

/// One operation with actual topology, not merely a glued glyph word.
#[derive(Clone, PartialEq, Debug)]
pub struct NestedQuantumGate {
    body: Vec<char>,
    word: Vec<char>,
    frames: Vec<FrameEdge>,
    outer_depth: usize,
    max_depth: usize,
}

impl NestedQuantumGate {
    pub(crate) fn around(body: &[char], outer_depth: usize) -> Result<Self, &'static str> {
        if outer_depth == 0 {
            return Err("quantum gate requires at least one enclosing IMASM frame");
        }
        let mut word = Vec::with_capacity(body.len() + outer_depth * 2 + 3);
        word.push(VINIT);
        for _ in 0..outer_depth { word.push(FSPLIT); }
        word.extend_from_slice(body);
        for _ in 0..outer_depth { word.push(FFUSE); }
        word.push(IFIX);
        word.push(TANCH);

        let mut stack: Vec<(usize, usize)> = Vec::new();
        let mut frames = Vec::new();
        let mut max_depth = 0usize;
        for (at, &mark) in word.iter().enumerate() {
            match mark {
                FSPLIT => {
                    let depth = stack.len() + 1;
                    max_depth = max_depth.max(depth);
                    stack.push((at, depth));
                }
                FFUSE => {
                    let (open, depth) = stack.pop().ok_or("quantum gate has an unmatched fuse")?;
                    frames.push(FrameEdge { open, close: at, depth });
                }
                _ => {}
            }
        }
        if !stack.is_empty() {
            return Err("quantum gate has an unmatched split");
        }
        frames.sort_by_key(|edge| edge.depth);
        Ok(Self { body: body.to_vec(), word, frames, outer_depth, max_depth })
    }

    pub fn body(&self) -> &[char] { &self.body }
    pub fn word(&self) -> &[char] { &self.word }
    pub fn frames(&self) -> &[FrameEdge] { &self.frames }
    pub fn outer_depth(&self) -> usize { self.outer_depth }
    pub fn max_depth(&self) -> usize { self.max_depth }

    /// Collapse repeated outer scale frames to one retained membrane frame.
    /// The operator body's own internal topology is untouched.
    pub fn dissolved_word(&self) -> Vec<char> {
        let mut out = Vec::with_capacity(self.body.len() + 5);
        out.push(VINIT);
        out.push(FSPLIT);
        out.extend_from_slice(&self.body);
        out.push(FFUSE);
        out.push(IFIX);
        out.push(TANCH);
        out
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct ControlledModularPhase {
    control: usize,
    exponent: Tape,
    phase: Tape,
    gate: NestedQuantumGate,
}

impl ControlledModularPhase {
    pub fn control(&self) -> usize { self.control }
    pub fn exponent(&self) -> &[char] { &self.exponent }
    pub fn phase(&self) -> &[char] { &self.phase }
    pub fn operator_word(&self) -> &[char] { self.gate.body() }
    pub fn nested_gate(&self) -> &NestedQuantumGate { &self.gate }
}

#[derive(Clone, PartialEq, Debug)]
pub enum InverseQftGate {
    ControlledPhase { control: usize, target: usize, distance: usize, gate: NestedQuantumGate },
    Hadamard { wire: usize, gate: NestedQuantumGate },
    Swap { left: usize, right: usize, gate: NestedQuantumGate },
}

impl InverseQftGate {
    pub fn operator_word(&self) -> &[char] { self.nested_gate().body() }
    pub fn nested_gate(&self) -> &NestedQuantumGate {
        match self {
            Self::ControlledPhase { gate, .. }
            | Self::Hadamard { gate, .. }
            | Self::Swap { gate, .. } => gate,
        }
    }
}

/// Structural QPE state before measurement. There is deliberately no sample.
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

    /// Consume the coherent register into the one-shot measurement membrane.
    pub fn into_measurement_program(self) -> Result<PhaseMeasurementProgram, &'static str> {
        let mut measurements = Vec::with_capacity(self.width);
        for wire in 0..self.width {
            measurements.push(NestedQuantumGate::around(&PHASE_MEASUREMENT_WORD, wire + 1)?);
        }
        Ok(PhaseMeasurementProgram {
            width: self.width,
            denominator: self.denominator,
            controls: self.controls,
            inverse_qft: self.inverse_qft,
            measurements,
        })
    }
}

/// The complete one-shot quantum program immediately before collapse.
///
/// This object still contains no measured numerator and no factor/order fields.
/// It owns the controlled modular phase register, inverse-QFT and measurement
/// topology so an executor cannot reuse them after producing one measurement.
#[derive(Clone, PartialEq, Debug)]
pub struct PhaseMeasurementProgram {
    width: usize,
    denominator: Tape,
    controls: Vec<ControlledModularPhase>,
    inverse_qft: Vec<InverseQftGate>,
    measurements: Vec<NestedQuantumGate>,
}

impl PhaseMeasurementProgram {
    pub fn width(&self) -> usize { self.width }
    pub fn denominator(&self) -> &[char] { &self.denominator }
    pub fn controls(&self) -> &[ControlledModularPhase] { &self.controls }
    pub fn inverse_qft(&self) -> &[InverseQftGate] { &self.inverse_qft }
    pub fn measurements(&self) -> &[NestedQuantumGate] { &self.measurements }
}

impl FixedPointQuantumMembrane {
    /// Build the nested structural phase-estimation register.
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
                gate: NestedQuantumGate::around(&CONTROLLED_MODULAR_PHASE_WORD, control + 1)?,
            });
        }

        let mut inverse_qft = Vec::new();
        for target in 0..width {
            for control in (target + 1)..width {
                let distance = control - target;
                inverse_qft.push(InverseQftGate::ControlledPhase {
                    control,
                    target,
                    distance,
                    gate: NestedQuantumGate::around(&INVERSE_QFT_PHASE_WORD, distance)?,
                });
            }
            inverse_qft.push(InverseQftGate::Hadamard {
                wire: target,
                gate: NestedQuantumGate::around(&INVERSE_QFT_HADAMARD_WORD, target + 1)?,
            });
        }
        for left in 0..(width / 2) {
            inverse_qft.push(InverseQftGate::Swap {
                left,
                right: width - 1 - left,
                gate: NestedQuantumGate::around(&INVERSE_QFT_SWAP_WORD, 1)?,
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
    fn controls_are_power_of_two_addresses_with_real_nested_edges() {
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
            assert_eq!(control.nested_gate().outer_depth(), j + 1);
            assert!(control.nested_gate().frames().len() >= j + 1);
        }
    }

    #[test]
    fn repeated_scale_nesting_dissolves_without_changing_gate_body() {
        let shallow = NestedQuantumGate::around(&CONTROLLED_MODULAR_PHASE_WORD, 1).unwrap();
        let deep = NestedQuantumGate::around(&CONTROLLED_MODULAR_PHASE_WORD, 23).unwrap();
        assert_ne!(shallow.word(), deep.word());
        assert_ne!(shallow.max_depth(), deep.max_depth());
        assert_eq!(shallow.dissolved_word(), deep.dissolved_word());
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
    fn inverse_qft_is_explicit_nested_polynomial_topology() {
        let membrane = FixedPointQuantumMembrane::from_n(&tape_u64(257)).unwrap();
        let register = membrane.phase_estimation_register().unwrap();
        let t = register.width();
        assert_eq!(register.hadamard_count(), t);
        assert_eq!(register.controlled_phase_count(), t * (t - 1) / 2);
        assert_eq!(register.swap_count(), t / 2);
        assert_eq!(register.inverse_qft().len(), t + t * (t - 1) / 2 + t / 2);
        assert!(register.inverse_qft().iter().all(|g| {
            !g.operator_word().is_empty()
                && !g.nested_gate().frames().is_empty()
                && g.nested_gate().word().first() == Some(&VINIT)
                && g.nested_gate().word().last() == Some(&TANCH)
        }));
    }

    #[test]
    fn measurement_program_consumes_register_and_nests_every_wire() {
        let membrane = FixedPointQuantumMembrane::from_n(&tape_u64(257)).unwrap();
        let program = membrane.phase_estimation_register().unwrap().into_measurement_program().unwrap();
        assert_eq!(program.measurements().len(), program.width());
        for (wire, gate) in program.measurements().iter().enumerate() {
            assert_eq!(gate.outer_depth(), wire + 1);
            assert_eq!(gate.body(), PHASE_MEASUREMENT_WORD);
            assert!(!gate.frames().is_empty());
        }
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
