//! Exact basis action ported from G-mOMonadOS fibonacci_qc::pauli_x.
//! Its off-diagonal unit entries exchange the two basis marks. Controls and
//! values remain IMASM cells; composing turns is addition modulo two.
use super::{ONE, ZERO};

#[inline]
pub(super) fn controlled_x(value: char, turn: char) -> char {
    if turn == ZERO { value }
    else if value == ZERO { ONE }
    else { ZERO }
}
