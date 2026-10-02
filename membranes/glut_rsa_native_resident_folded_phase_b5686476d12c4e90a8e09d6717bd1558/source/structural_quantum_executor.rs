//! Source-bound structural execution contract for the quantum phase landing.
use crate::fixed_point_quantum_phase::structural::Circuit;
use crate::fixed_point_quantum_readout::QuantumWindingPreimage;
use crate::hadamard_gate::Tape;

/// These operators are coherent register operations. An executor must implement
/// their action, not obtain a period first and attach it to the program.
pub enum Operation<'a> {
    Uniform {
        width: usize,
        boundary: &'a Circuit,
    },
    WorkUnit {
        width: usize,
        bit_flip: &'a Circuit,
    },
    ControlledModularPowers {
        controls: usize,
        work: usize,
        base: &'a [char],
        modulus: &'a [char],
    },
    InverseFourier {
        width: usize,
    },
    MeasurePhase {
        width: usize,
    },
}

/// A register program carries one opaque N-dependent landing. No amplitudes,
/// basis states, period, factors or per-nesting-level objects are materialized.
pub struct Program {
    preimage: QuantumWindingPreimage,
    hadamard: Circuit,
    bit_flip: Circuit,
}
impl Program {
    pub(crate) fn prepare(preimage: QuantumWindingPreimage) -> Result<Self, &'static str> {
        Ok(Self {
            preimage,
            hadamard: Circuit::prepare("H")?,
            bit_flip: Circuit::prepare("X")?,
        })
    }
    /// The same resident N supplies the direct factor-pair phase oracle.
    /// Its factor registers differ from the order-estimation registers.
    pub fn factor_phase_oracle(
        &self,
    ) -> Result<crate::fixed_point_quantum_phase::factor_oracle::FactorPhaseOracle<'_>, &'static str>
    {
        crate::fixed_point_quantum_phase::factor_oracle::FactorPhaseOracle::from_n(self.n())
    }
    /// Exact CPU coherent execution and inverse-CDF pair measurement. Rounds
    /// and the measurement quantile are prepared IMASM tapes. The register
    /// retains shared decision branches, with no dense amplitude array.
    pub fn measure_factor_pair(
        &self,
        rounds: &[char],
        quantile: &[char],
        denominator: &[char],
    ) -> Result<Option<crate::factor_extract::FactorCarrier>, &'static str> {
        use crate::fixed_point_quantum_phase::factor_oracle::{Control, GateSink};
        use crate::fixed_point_quantum_phase::register::FoldedRegister;
        use crate::morphism_factor::{one, sub, trim, zero};
        use crate::vox::{CLINK, EVALF, EVALT};
        if rounds.is_empty() || rounds.iter().any(|&c| c != EVALT && c != EVALF) {
            return Err("factor amplification rounds are not an IMASM numeral");
        }
        let oracle = self.factor_phase_oracle()?;
        let layout = oracle.layout();
        let mut register = FoldedRegister::zero(layout.cells);
        for wire in layout.p.clone().chain(layout.q.clone()) {
            register.hadamard(wire)?;
        }
        register.fold();
        let zeros: alloc::vec::Vec<_> = layout
            .p
            .clone()
            .chain(layout.q.clone())
            .map(|cell| Control { cell, value: EVALT })
            .collect();
        let mut remaining = rounds.to_vec();
        while !zero(&remaining) {
            oracle.lower(&mut register)?;
            for wire in layout.p.clone().chain(layout.q.clone()) {
                register.hadamard(wire)?;
            }
            register.phase_flip(&zeros)?;
            for wire in layout.p.clone().chain(layout.q.clone()) {
                register.hadamard(wire)?;
            }
            register.fold();
            remaining = sub(&remaining, &one());
        }
        let measured = register.measure(quantile, denominator)?;
        if measured[layout.product.start..]
            .iter()
            .any(|&bit| bit != EVALT)
        {
            return Err("factor oracle left a nonzero work register");
        }
        let p = trim(measured[layout.p.clone()].to_vec());
        let q = trim(measured[layout.q.clone()].to_vec());
        if !crate::trace_algebra::witness_valid_tape(self.n(), &p, &q) {
            return Ok(None);
        }
        let trace = [crate::router_marks::GStep {
            repr: CLINK,
            judgment: crate::router_marks::M_T,
            recognised: crate::router_marks::M_T,
            next: crate::router_marks::M_FIX,
            applied_word: self.fixation_word().to_vec(),
        }];
        crate::factor_extract::FactorCarrier::from_trace(self.n(), &p, &q, &trace)
            .map(Some)
            .map_err(|_| "measured factor pair failed carrier closure")
    }
    pub fn n(&self) -> &[char] {
        self.preimage.n()
    }
    pub fn base(&self) -> &[char] {
        self.preimage.base()
    }
    pub fn phase_width(&self) -> usize {
        self.preimage.width()
    }
    pub fn work_width(&self) -> usize {
        self.n().len()
    }
    pub fn denominator(&self) -> &[char] {
        self.preimage.precision_denominator()
    }
    pub fn fixation_word(&self) -> &[char] {
        self.preimage.fixation_word()
    }
    pub fn operations(&self) -> [Operation<'_>; 5] {
        let width = self.phase_width();
        [
            Operation::Uniform {
                width,
                boundary: &self.hadamard,
            },
            Operation::WorkUnit {
                width: self.work_width(),
                bit_flip: &self.bit_flip,
            },
            Operation::ControlledModularPowers {
                controls: width,
                work: self.work_width(),
                base: self.base(),
                modulus: self.n(),
            },
            Operation::InverseFourier { width },
            Operation::MeasurePhase { width },
        ]
    }
    /// Control k applies the resident modular permutation to power 2^k.
    /// The exponent is carried as winding, without evaluating a^2^k here.
    pub fn controlled_power_windings(&self) -> core::ops::Range<usize> {
        0..self.phase_width()
    }
    pub(crate) fn into_preimage(self) -> QuantumWindingPreimage {
        self.preimage
    }
}

/// The backend executes all operations and returns one LSB-first IMASM phase
/// numerator. Width and source stay bound to the program, not backend metadata.
/// No observation is printed by the membrane before verified readout.
pub trait Executor {
    fn execute_and_measure(&mut self, program: &Program) -> Result<Tape, &'static str>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixed_point_quantum_membrane::FixedPointQuantumMembrane;
    use crate::fixed_point_quantum_phase::structural::Gate;
    use crate::vox::{EVALF, EVALT};
    use alloc::vec;

    fn numeral(mut n: u64) -> Tape {
        let mut tape = alloc::vec::Vec::new();
        while n != 0 {
            tape.push(if n & 1 == 1 { EVALF } else { EVALT });
            n >>= 1;
        }
        if tape.is_empty() {
            tape.push(EVALT);
        }
        tape
    }

    #[test]
    fn resident_register_operations_preserve_source_and_order() {
        let n = numeral(15);
        let membrane = FixedPointQuantumMembrane::from_n(&n).unwrap();
        let program = membrane.prepare_structural_execution().unwrap();
        assert_eq!(program.n(), n);
        assert_eq!(program.work_width(), 4);
        assert_eq!(program.factor_phase_oracle().unwrap().n(), program.n());
        assert_eq!(program.phase_width(), 8);
        assert_eq!(program.denominator(), numeral(256));
        let operations = program.operations();
        match &operations[0] {
            Operation::Uniform { width, boundary } => {
                assert_eq!(*width, 8);
                assert_eq!(boundary.gates, vec![Gate::Hadamard]);
            }
            _ => panic!("missing uniform preparation"),
        }
        match &operations[1] {
            Operation::WorkUnit { width, bit_flip } => {
                assert_eq!(*width, 4);
                assert_eq!(bit_flip.gates, vec![Gate::BitFlip]);
            }
            _ => panic!("missing work-unit preparation"),
        }
        match &operations[2] {
            Operation::ControlledModularPowers {
                controls,
                work,
                base,
                modulus,
            } => {
                assert_eq!((*controls, *work), (8, 4));
                assert_eq!(*base, program.base());
                assert_eq!(*modulus, n);
            }
            _ => panic!("missing controlled modular permutation"),
        }
        assert!(matches!(
            operations[3],
            Operation::InverseFourier { width: 8 }
        ));
        assert!(matches!(
            operations[4],
            Operation::MeasurePhase { width: 8 }
        ));
        assert_eq!(program.controlled_power_windings(), 0..8);
    }

    #[test]
    fn wide_preparation_retains_symbolic_registers() {
        let mut n = vec![EVALT; 2048];
        n[0] = EVALF;
        n[2047] = EVALF;
        let program = FixedPointQuantumMembrane::from_n(&n)
            .unwrap()
            .prepare_structural_execution()
            .unwrap();
        assert_eq!(program.n(), n);
        assert_eq!(program.work_width(), 2048);
        assert_eq!(program.phase_width(), 4096);
        assert_eq!(program.operations().len(), 5);
        assert_eq!(program.controlled_power_windings(), 0..4096);
    }

    struct InjectedControl {
        calls: usize,
        response: Result<Tape, &'static str>,
    }
    impl Executor for InjectedControl {
        fn execute_and_measure(&mut self, program: &Program) -> Result<Tape, &'static str> {
            self.calls += 1;
            assert_eq!(program.n(), numeral(15));
            self.response.clone()
        }
    }

    #[test]
    fn folded_coherent_factor_readout_uses_n_only() {
        let program = FixedPointQuantumMembrane::from_n(&numeral(15))
            .unwrap()
            .prepare_structural_execution()
            .unwrap();
        let carrier = program
            .measure_factor_pair(&numeral(8), &numeral(1), &numeral(2))
            .unwrap()
            .expect("factor pair was not measured");
        let factors = crate::factor_extract::extract(&carrier).unwrap();
        assert!(
            (factors.p.0 == numeral(3) && factors.q.0 == numeral(5))
                || (factors.p.0 == numeral(5) && factors.q.0 == numeral(3))
        );
    }

    #[test]
    fn executor_failure_returns_without_arithmetic_fallback() {
        let mut executor = InjectedControl {
            calls: 0,
            response: Err("executor unavailable"),
        };
        let result = FixedPointQuantumMembrane::from_n(&numeral(15))
            .unwrap()
            .measure_and_descend(&mut executor);
        assert_eq!(result.unwrap_err(), "executor unavailable");
        assert_eq!(executor.calls, 1);
    }

    #[test]
    fn executor_measurement_must_be_an_in_range_imasm_numeral() {
        for response in [vec!['H'], numeral(256)] {
            let mut executor = InjectedControl {
                calls: 0,
                response: Ok(response),
            };
            assert!(FixedPointQuantumMembrane::from_n(&numeral(15))
                .unwrap()
                .measure_and_descend(&mut executor)
                .is_err());
            assert_eq!(executor.calls, 1);
        }
    }

    #[test]
    fn injected_phase_control_reaches_verified_factor_readout() {
        // Injected k/M = 64/256 is a wiring control, not an executed measurement.
        let mut executor = InjectedControl {
            calls: 0,
            response: Ok(numeral(64)),
        };
        let result = FixedPointQuantumMembrane::from_n(&numeral(15))
            .unwrap()
            .measure_and_descend(&mut executor)
            .unwrap();
        match result {
            crate::hadamard_factor_bridge::HadamardDescent::T(carrier) => {
                let factors = crate::factor_extract::extract(&carrier).unwrap();
                let (p, q) = (factors.p.0, factors.q.0);
                assert!(
                    (p == numeral(3) && q == numeral(5)) || (p == numeral(5) && q == numeral(3))
                );
            }
            other => panic!("injected phase control did not descend: {other:?}"),
        }
        assert_eq!(executor.calls, 1);
    }
}
