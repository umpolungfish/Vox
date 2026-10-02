//! Exact basis action ported from G-mOMonadOS fibonacci_qc::pauli_x.
//! Its off-diagonal unit entries exchange the two basis marks. Controls and
//! values remain IMASM cells; composing turns is addition modulo two.
use super::*;

#[inline]
pub(super) fn controlled_x(value: char, turn: char) -> char {
    if turn == ZERO { value }
    else if value == ZERO { ONE }
    else { ZERO }
}

impl Correlation {
    /// Select the marked support of the factor-specific sign character.
    /// The marked sector is exactly p*q=N with proper source-bound factors.
    /// This oracle uses the resident product/carry gates shared with the
    /// midpoint and gap; it never expands a separate pair register.
    pub(super) fn bind_factor_phase(
        &mut self, n: &[char], p: &[usize], q: &[usize], root: &[char],
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
