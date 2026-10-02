//! Exact basis action ported from G-mOMonadOS fibonacci_qc::pauli_x.
//! Its off-diagonal unit entries exchange the two basis marks. Controls and
//! values remain IMASM cells; composing turns is addition modulo two.
use super::*;
use crate::fixed_point_quantum_phase::{
    factor_oracle::{Control, FactorPhaseOracle, GateSink},
    register::FoldedRegister,
};

#[inline]
pub(super) fn controlled_x(value: char, turn: char) -> char {
    if turn == ZERO {
        value
    } else if value == ZERO {
        ONE
    } else {
        ZERO
    }
}

struct ControlledFactorPhase<'a> {
    register: &'a mut FoldedRegister,
    control: usize,
}

impl GateSink for ControlledFactorPhase<'_> {
    fn toggle(&mut self, controls: &[Control], target: usize) -> Result<(), &'static str> {
        self.register.toggle(controls, target)
    }

    fn phase_flip(&mut self, controls: &[Control]) -> Result<(), &'static str> {
        let mut controlled = controls.to_vec();
        controlled.push(Control {
            cell: self.control,
            value: ONE,
        });
        self.register.phase_flip(&controlled)
    }
}

pub(crate) fn factor_phase_closes(n: &[char], p: &[char], q: &[char]) -> bool {
    let Ok(oracle) = FactorPhaseOracle::from_n(n) else {
        return false;
    };
    let layout = oracle.layout();
    if p.len() > layout.p.len() || q.len() > layout.q.len() {
        return false;
    }
    let control = layout.cells;
    let mut state = FoldedRegister::zero(control + 1);
    for (range, factor) in [(layout.p.clone(), p), (layout.q.clone(), q)] {
        for (offset, &bit) in factor.iter().enumerate() {
            if bit != ZERO && bit != ONE {
                return false;
            }
            if bit == ONE && state.toggle(&[], range.start + offset).is_err() {
                return false;
            }
        }
    }
    if state.hadamard(control).is_err() {
        return false;
    }
    let mut controlled = ControlledFactorPhase {
        register: &mut state,
        control,
    };
    if oracle.lower(&mut controlled).is_err() || state.hadamard(control).is_err() {
        return false;
    }
    let Ok(measured) = state.measure(&[ZERO], &[ONE]) else {
        return false;
    };
    let pair_matches = |range: core::ops::Range<usize>, factor: &[char]| {
        measured.get(range).is_some_and(|bits| {
            bits.iter().enumerate().all(|(index, &bit)| {
                factor.get(index).copied().unwrap_or(ZERO) == bit
            })
        })
    };
    measured.get(control) == Some(&ONE)
        && pair_matches(layout.p.clone(), p)
        && pair_matches(layout.q.clone(), q)
        && measured[layout.product.clone()].iter().all(|&bit| bit == ZERO)
        && measured[layout.row.clone()].iter().all(|&bit| bit == ZERO)
        && measured[layout.carry] == ZERO
        && measured[layout.nontrivial_p] == ZERO
        && measured[layout.nontrivial_q] == ZERO
}

impl Correlation {
    /// Select the marked support of the factor-specific sign character.
    /// The marked sector is exactly p*q=N with proper source-bound factors.
    /// This oracle uses the resident product/carry gates shared with the
    /// midpoint and gap; it never expands a separate pair register.
    pub(super) fn bind_factor_phase(
        &mut self,
        n: &[char],
        p: &[usize],
        q: &[usize],
        root: &[char],
    ) -> Vec<char> {
        let mut columns = vec![Vec::new(); p.len() + q.len() + 1];
        for (i, &left) in p.iter().enumerate() {
            for (j, &right) in q.iter().enumerate() {
                let cell = self.and(left, right);
                if cell != 0 {
                    columns[i + j].push(cell);
                }
            }
        }
        let product = self.reduce_columns(columns);
        for (i, &cell) in product.iter().enumerate() {
            self.equal_cells(cell, usize::from(bit(n, i) == 1));
        }
        self.bound(p, &[ONE, ONE], true);
        self.bound(p, root, false);
        let (mut lower, remainder) = mf::divmod(n, root);
        if remainder != vec![ZERO] {
            lower = mf::add(&lower, &[ONE]);
        }
        self.bound(q, &lower, true);
        lower
    }
}
