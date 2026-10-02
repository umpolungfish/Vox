//! Exact basis action ported from G-mOMonadOS fibonacci_qc::pauli_x.
//! Its off-diagonal unit entries exchange the two basis marks. Controls and
//! values remain IMASM cells; composing turns is addition modulo two.
use super::*;
use crate::fixed_point_quantum_phase::factor_oracle::{Control, FactorPhaseOracle, GateSink};

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

struct FactorPhaseBasis {
    cells: Vec<char>,
    phase: char,
}

impl GateSink for FactorPhaseBasis {
    fn toggle(&mut self, controls: &[Control], target: usize) -> Result<(), &'static str> {
        if target >= self.cells.len() || controls.iter().any(|control| control.cell == target) {
            return Err("factor-phase basis toggle is out of range");
        }
        if controls
            .iter()
            .all(|control| self.cells.get(control.cell) == Some(&control.value))
        {
            self.cells[target] = if self.cells[target] == ZERO {
                ONE
            } else {
                ZERO
            };
        }
        Ok(())
    }

    fn phase_flip(&mut self, controls: &[Control]) -> Result<(), &'static str> {
        if controls
            .iter()
            .any(|control| self.cells.get(control.cell) != Some(&control.value))
        {
            return Ok(());
        }
        self.phase = if self.phase == ZERO { ONE } else { ZERO };
        Ok(())
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
    let mut state = FactorPhaseBasis {
        cells: vec![ZERO; layout.cells],
        phase: ZERO,
    };
    for (range, factor) in [(layout.p.clone(), p), (layout.q.clone(), q)] {
        for (offset, &bit) in factor.iter().enumerate() {
            state.cells[range.start + offset] = bit;
        }
    }
    let p_before = state.cells[layout.p.clone()].to_vec();
    let q_before = state.cells[layout.q.clone()].to_vec();
    if oracle.lower(&mut state).is_err() || state.phase != ONE {
        return false;
    }
    state.cells[layout.p.clone()] == p_before
        && state.cells[layout.q.clone()] == q_before
        && state.cells[layout.product.clone()]
            .iter()
            .all(|&bit| bit == ZERO)
        && state.cells[layout.row.clone()]
            .iter()
            .all(|&bit| bit == ZERO)
        && state.cells[layout.carry] == ZERO
        && state.cells[layout.nontrivial_p] == ZERO
        && state.cells[layout.nontrivial_q] == ZERO
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
