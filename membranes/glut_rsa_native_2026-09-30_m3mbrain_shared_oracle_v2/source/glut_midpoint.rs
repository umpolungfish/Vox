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
    pub fn new_nested(n: &[char], width: usize, root: &[char]) -> Self {
        let mut s = Self::new_midpoint(n, width, root);
        let midpoint = s.p.clone();
        let gap = s.q.clone();
        let p: Vec<_> = (0..width)
            .map(|i| if i == 0 { 1 } else { s.cell() })
            .collect();
        let q: Vec<_> = (0..width + 1)
            .map(|i| if i == 0 { 1 } else { s.cell() })
            .collect();
        // Descend through p+b=a and a+b=q; every adder propagates back
        // through the same cells on the ascending side of the boundary.
        s.sum_into(&p, &gap, &midpoint);
        s.sum_into(&midpoint, &gap, &q);
        let lower = s.bind_factor_phase(n, &p, &q, root);
        s.p = p;
        s.q = q;
        s.conjugate = false;
        s.source = n.to_vec();
        s.factor_root = root.to_vec();
        s.cofactor_floor = lower;
        s.connect();
        s
    }
    pub(super) fn equal_cells(&mut self, left: usize, right: usize) {
        if left == right {
            return;
        }
        if left <= 1 || right <= 1 {
            let (cell, constant) = if left <= 1 {
                (right, left)
            } else {
                (left, right)
            };
            if !self.assign(
                Literal {
                    cell,
                    mark: if constant == 1 { ONE } else { ZERO },
                },
                None,
            ) {
                self.empty = true;
            }
        } else {
            self.gate(vec![left, right], EQUAL);
        }
    }
    fn sum_into(&mut self, left: &[usize], right: &[usize], result: &[usize]) {
        let extent = left.len().max(right.len()).max(result.len()) + 1;
        let mut carry = 0;
        for i in 0..extent {
            let (sum, next) = self.adder(
                left.get(i).copied().unwrap_or(0),
                right.get(i).copied().unwrap_or(0),
                carry,
            );
            self.equal_cells(sum, result.get(i).copied().unwrap_or(0));
            carry = next;
        }
        self.equal_cells(carry, 0);
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
    pub(super) fn reduce_columns(&mut self, mut columns: Vec<Vec<usize>>) -> Vec<usize> {
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
    fn conflict_analysis_discards_newer_irrelevant_decisions() {
        let mut graph=Correlation::blank(false);
        let a=graph.cell(); let newer=graph.cell();
        graph.p=vec![a,newer]; graph.q=vec![1]; graph.connect();
        let a=graph.p[0]; let newer=graph.p[1];
        graph.boundaries.push(graph.trail.len());
        graph.assign(Literal {cell:a,mark:ONE},None);
        graph.boundaries.push(graph.trail.len());
        graph.assign(Literal {cell:newer,mark:ONE},None);
        let clause=vec![Literal {cell:a,mark:ZERO},Literal {cell:1,mark:ZERO}];
        for lit in &clause { graph.watches.entry(lit.slot()).or_default().push(0); }
        graph.learned.push(clause);
        assert!(matches!(graph.advance(),Step::Running));
        assert_eq!(graph.values[a],Some(ZERO)); assert_eq!(graph.levels[a],0);
        assert_eq!(graph.boundaries.len(),1);
    }
    #[test]
    fn support_fold_clauses_hold_for_independent_factor_witnesses() {
        for n in (9..=255u64).step_by(2) {
            let tape=mf::tape_u64(n); let width=tape.len().div_ceil(2);
            let witnesses: Vec<_> = (3..=n).step_by(2).filter_map(|p| {
                if n%p!=0 { return None; }
                let q=n/p;
                let a=(p+q)/2; let b=q.saturating_sub(p)/2;
                (p<=q && a>=1<<(width-1) && a<1<<width && b<1<<width.saturating_sub(2))
                    .then_some((p,q))
            }).collect();
            if witnesses.is_empty() { continue; }
            for i in 0..width { for j in 0..=width { for pattern in 0..4 {
                let mut graph=Correlation::new_nested(&tape,width,&mf::isqrt(&tape));
                graph.boundaries.push(graph.trail.len());
                if !graph.assign(Literal {cell:graph.p[i],mark:if pattern&1==0 {ZERO}else{ONE}},None)
                    || !graph.assign(Literal {cell:graph.q[j],mark:if pattern&2==0 {ZERO}else{ONE}},None) {
                    continue;
                }
                let before=graph.trail.len();
                let mut clauses=Vec::new();
                if let Err(clause)=graph.fold_support() { clauses.push(clause); }
                for lit in &graph.trail[before..] {
                    if let Some(reason)=&graph.reasons[lit.cell] { clauses.push(reason.clone()); }
                }
                for &(p,q) in &witnesses {
                    let mut row=BTreeMap::from([(0,ZERO),(1,ONE)]);
                    for (&cell,mark) in graph.p.iter().enumerate().map(|(k,cell)|(cell,if p>>k&1==0 {ZERO}else{ONE}))
                        .chain(graph.q.iter().enumerate().map(|(k,cell)|(cell,if q>>k&1==0 {ZERO}else{ONE}))) {
                        if let Some(old)=row.insert(cell,mark) { assert_eq!(old,mark); }
                    }
                    for clause in &clauses {
                        assert!(clause.iter().any(|lit| row[&lit.cell]==lit.mark),"n={n}, p={p}, q={q}");
                    }
                }
            }}}
        }
    }
    #[test]
    fn shared_square_coordinates_match_their_entire_small_domain() {
        for n in (3..=1023u64).step_by(2) {
            let tape = mf::tape_u64(n);
            let width = tape.len().div_ceil(2);
            let expected = (1u64 << (width - 1)..1u64 << width).any(|a| {
                (0..1u64 << width.saturating_sub(2))
                    .any(|b| a * a >= b * b && a * a - b * b == n && a - b > 1)
            });
            for mut graph in [
                Correlation::new_midpoint(&tape, width, &mf::isqrt(&tape)),
                Correlation::new_nested(&tape, width, &mf::isqrt(&tape)),
            ] {
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
}
