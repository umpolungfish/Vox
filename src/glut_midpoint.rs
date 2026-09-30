//! Shared square cells for a² = N + b². The diagonal xᵢ² is xᵢ;
//! the two off-diagonal copies become one adjacent-column contribution.
use super::*;
pub(super) const EQUAL: &[&[char]] = &[&[ZERO, ZERO], &[ONE, ONE]];
impl Correlation {
    pub fn new_midpoint(n: &[char], width: usize, root: &[char]) -> Self {
        let mut s = Self::blank(width < 2);
        s.conjugate = true;
        let gap_width = width.saturating_sub(2);
        s.q = if gap_width == 0 {
            vec![0]
        } else {
            (0..gap_width)
                .map(|i| {
                    if i == 0 {
                        usize::from(bit(n, 1) == 1)
                    } else {
                        s.cell()
                    }
                })
                .collect()
        };
        if gap_width == 0 && bit(n, 1) == 1 {
            s.empty = true;
        }
        s.p = (0..width)
            .map(|i| {
                if i == 0 {
                    usize::from(bit(n, 1) == 0)
                } else if i + 1 == width {
                    1
                } else {
                    s.cell()
                }
            })
            .collect();
        if width == 1 && s.p[0] != 1 {
            s.empty = true;
        }
        let a = s.p.clone();
        let b = s.q.clone();
        let left = s.square_columns(&a, width * 2 + 1);
        let mut right = s.square_columns(&b, width * 2 + 1);
        for (k, &mark) in n.iter().enumerate() {
            if mark == ONE {
                right[k].push(1);
            }
        }
        let left = s.reduce_columns(left);
        let right = s.reduce_columns(right);
        for (&a, &b) in left.iter().zip(&right) {
            if a == b {
                continue;
            }
            if a <= 1 {
                if !s.assign(
                    Literal {
                        cell: b,
                        mark: if a == 1 { ONE } else { ZERO },
                    },
                    None,
                ) {
                    s.empty = true;
                }
            } else if b <= 1 {
                if !s.assign(
                    Literal {
                        cell: a,
                        mark: if b == 1 { ONE } else { ZERO },
                    },
                    None,
                ) {
                    s.empty = true;
                }
            } else {
                s.gate(vec![a, b], EQUAL);
            }
        }
        let lower = if mf::mul(root, root) == n {
            root.to_vec()
        } else {
            mf::add(root, &[ONE])
        };
        s.bound(&a, &lower, true);
        s.connect();
        s
    }
    fn square_columns(&mut self, cells: &[usize], extent: usize) -> Vec<Vec<usize>> {
        let mut columns = vec![Vec::new(); extent];
        for (i, &a) in cells.iter().enumerate() {
            if a != 0 {
                columns[2 * i].push(a);
            }
            for (j, &b) in cells.iter().enumerate().skip(i + 1) {
                let product = self.and(a, b);
                if product != 0 {
                    columns[i + j + 1].push(product);
                }
            }
        }
        columns
    }
    fn reduce_columns(&mut self, mut columns: Vec<Vec<usize>>) -> Vec<usize> {
        for k in 0..columns.len() - 1 {
            while columns[k].len() >= 3 {
                let a = columns[k].pop().unwrap();
                let b = columns[k].pop().unwrap();
                let c = columns[k].pop().unwrap();
                let (sum, carry) = self.adder(a, b, c);
                if sum != 0 {
                    columns[k].push(sum);
                }
                if carry != 0 {
                    columns[k + 1].push(carry);
                }
            }
        }
        let mut result = Vec::new();
        let mut carry = 0;
        for column in columns {
            let (sum, next) = self.adder(
                column.first().copied().unwrap_or(0),
                column.get(1).copied().unwrap_or(0),
                carry,
            );
            result.push(sum);
            carry = next;
        }
        if !self.assign(
            Literal {
                cell: carry,
                mark: ZERO,
            },
            None,
        ) {
            self.empty = true;
        }
        result
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn shared_square_coordinates_match_their_entire_small_domain() {
        for n in (3..=1023u64).step_by(2) {
            let tape = mf::tape_u64(n);
            let width = tape.len().div_ceil(2);
            let expected = (1u64 << (width - 1)..1u64 << width).any(|a| {
                (0..1u64 << width.saturating_sub(2))
                    .any(|b| a * a >= b * b && a * a - b * b == n && a - b > 1)
            });
            let mut graph = Correlation::new_midpoint(&tape, width, &mf::isqrt(&tape));
            loop {
                match graph.advance() {
                    Step::Running => {}
                    Step::Empty => {
                        assert!(!expected, "n={n}");
                        break;
                    }
                    Step::Closed(p, q) => {
                        assert!(expected, "n={n}");
                        assert!(crate::trace_algebra::witness_valid(&tape, &p, &q));
                        break;
                    }
                }
            }
        }
    }
}
