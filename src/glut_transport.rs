//! Quotient shared cells by equality transport and expression congruence.
use super::*;

fn representative(parent: &mut [usize], cell: usize) -> usize {
    let mut root = cell;
    while parent[root] != root {
        root = parent[root];
    }
    let mut next = cell;
    while parent[next] != next {
        let old = parent[next];
        parent[next] = root;
        next = old;
    }
    root
}
fn join(parent: &mut [usize], left: usize, right: usize) -> bool {
    let a = representative(parent, left);
    let b = representative(parent, right);
    if a == b {
        return false;
    }
    parent[a.max(b)] = a.min(b);
    true
}
impl Correlation {
    pub(super) fn fold_transport(&mut self) {
        assert!(self.boundaries.is_empty() && self.learned.is_empty());
        let mut parent: Vec<_> = (0..self.values.len()).collect();
        let mut edges = vec![Vec::new(); self.values.len()];
        for gate in &self.gates {
            if let &[a, b] = gate.cells.as_slice() {
                let turn = if gate.supports[1][0] == 0b10 {
                    ONE
                } else {
                    ZERO
                };
                edges[a].push((b, turn));
                edges[b].push((a, turn));
            }
        }
        let mut orientation = vec![None; self.values.len()];
        for start in 0..self.values.len() {
            if orientation[start].is_some() || edges[start].is_empty() {
                continue;
            }
            orientation[start] = Some(ZERO);
            let mut pending = vec![start];
            let mut members = Vec::new();
            let mut sides = [None, None];
            let mut anchor = None;
            while let Some(cell) = pending.pop() {
                let turn = orientation[cell].unwrap();
                let slot = usize::from(turn == ONE);
                if let Some(other) = sides[slot] {
                    join(&mut parent, cell, other);
                } else {
                    sides[slot] = Some(cell);
                }
                members.push(cell);
                if let Some(mark) = self.values[cell] {
                    let transported = quantum::controlled_x(mark, turn);
                    if anchor.is_some_and(|other| other != transported) {
                        self.empty = true;
                    }
                    anchor = Some(transported);
                }
                for &(next, edge) in &edges[cell] {
                    let transported = quantum::controlled_x(turn, edge);
                    if let Some(existing) = orientation[next] {
                        if existing != transported {
                            self.empty = true;
                        }
                    } else {
                        orientation[next] = Some(transported);
                        pending.push(next);
                    }
                }
            }
            // Relative transport never prepares a free component to a chosen
            // truth value. Only a source-root pin fixes its orientation.
            if let Some(mark) = anchor {
                for cell in members {
                    let value = usize::from(mark != orientation[cell].unwrap());
                    join(&mut parent, cell, value);
                }
            }
        }
        loop {
            let mut changed = false;
            let mut expressions: BTreeMap<Vec<usize>, Vec<usize>> = BTreeMap::new();
            let mut opposite = BTreeMap::new();
            for gate in &self.gates {
                if let &[a, b] = gate.cells.as_slice() {
                    if gate.supports[1][0] == 0b10 {
                        let a = representative(&mut parent, a);
                        let b = representative(&mut parent, b);
                        opposite.insert(a, b);
                        opposite.insert(b, a);
                    }
                }
            }
            for gate in &self.gates {
                let cells: Vec<_> = gate
                    .cells
                    .iter()
                    .map(|&cell| representative(&mut parent, cell))
                    .collect();
                if cells.len() == 2 && gate.supports[1][0] == 0b01 {
                    changed |= join(&mut parent, cells[0], cells[1]);
                    continue;
                }
                match cells.as_slice() {
                    &[a, b, out] => {
                        let folded = if a == 0 || b == 0 || opposite.get(&a) == Some(&b) {
                            Some(0)
                        } else if a == b || b == 1 {
                            Some(a)
                        } else if a == 1 {
                            Some(b)
                        } else {
                            None
                        };
                        if let Some(value) = folded {
                            changed |= join(&mut parent, out, value);
                        }
                    }
                    &[a, out] => {
                        if a <= 1 {
                            changed |= join(&mut parent, out, 1 - a);
                        }
                        if a == out {
                            self.empty = true;
                        }
                    }
                    &[a, b, c, sum, carry] => {
                        let repeated = if a == b {
                            Some((c, a))
                        } else if a == c {
                            Some((b, a))
                        } else if b == c {
                            Some((a, b))
                        } else {
                            None
                        };
                        if let Some((bit, next)) = repeated {
                            changed |= join(&mut parent, sum, bit);
                            changed |= join(&mut parent, carry, next);
                        } else {
                            let remaining = if opposite.get(&a) == Some(&b) {
                                Some(c)
                            } else if opposite.get(&a) == Some(&c) {
                                Some(b)
                            } else if opposite.get(&b) == Some(&c) {
                                Some(a)
                            } else {
                                None
                            };
                            if let Some(bit) = remaining {
                                changed |= join(&mut parent, carry, bit);
                                let complement = if bit <= 1 {
                                    Some(1 - bit)
                                } else {
                                    opposite.get(&bit).copied()
                                };
                                if let Some(value) = complement {
                                    changed |= join(&mut parent, sum, value);
                                }
                            }
                        }
                    }
                    _ => unreachable!(),
                }
                let inputs = match cells.len() {
                    2 => 1,
                    3 => 2,
                    5 => 3,
                    _ => unreachable!(),
                };
                let mut key = cells[..inputs].to_vec();
                key.sort_unstable();
                key.insert(0, cells.len());
                if let Some(outputs) = expressions.get(&key) {
                    for (&left, &right) in outputs.iter().zip(&cells[inputs..]) {
                        changed |= join(&mut parent, left, right);
                    }
                } else {
                    expressions.insert(key, cells[inputs..].to_vec());
                }
            }
            if !changed {
                break;
            }
        }
        for cell in 0..parent.len() {
            representative(&mut parent, cell);
        }
        // Keep the quotient itself, rather than leaving the old vertices as
        // holes in every value, reason and incidence tape.
        if parent[0] == parent[1] {
            self.empty = true;
            return;
        }
        let mut retained = alloc::collections::BTreeSet::from([0, 1]);
        for gate in &self.gates {
            if gate.cells.len() == 2 && gate.supports[1][0] == 0b01 {
                continue;
            }
            for &cell in &gate.cells {
                retained.insert(parent[cell]);
            }
        }
        for &cell in self.p.iter().chain(&self.q) {
            retained.insert(parent[cell]);
        }
        let mut dense = vec![None; parent.len()];
        for (index, root) in retained.iter().copied().enumerate() {
            dense[root] = Some(index);
        }
        let mut values = vec![None; retained.len()];
        for (cell, &value) in self.values.iter().enumerate() {
            if let (Some(mark), Some(root)) = (value, dense[parent[cell]]) {
                if values[root].is_some_and(|other| other != mark) {
                    self.empty = true;
                } else {
                    values[root] = Some(mark);
                }
            }
        }
        self.values = values;
        self.trail.clear();
        self.cursor = 0;
        self.reasons.resize(self.values.len(), None);
        self.levels.resize(self.values.len(), 0);
        self.reasons.fill(None);
        self.levels.fill(0);
        for (cell, &value) in self.values.iter().enumerate() {
            if let Some(mark) = value {
                self.trail.push(Literal { cell, mark });
            }
        }
        for cell in self.p.iter_mut().chain(&mut self.q) {
            *cell = dense[parent[*cell]].unwrap();
        }
        self.activity.clear();
        self.priority.clear();
        self.products.clear();
        self.complements.clear();
        self.sums.clear();
        self.parity.clear();
        self.parity_state.clear();
        self.parity_pairs.clear();
        self.parity_offsets.clear();
        self.parity_incidence.clear();
        let mut seen = alloc::collections::BTreeSet::new();
        self.gates.retain_mut(|gate| {
            if gate.cells.len() == 2 && gate.supports[1][0] == 0b01 {
                return false;
            }
            for cell in &mut gate.cells {
                *cell = dense[parent[*cell]].unwrap();
            }
            seen.insert(gate.cells.clone())
        });
        for gate in &self.gates {
            match gate.cells.as_slice() {
                &[a, b, out] => {
                    self.products.insert((a.min(b), a.max(b)), out);
                }
                &[a, out] => {
                    self.complements.insert(a, out);
                    self.complements.insert(out, a);
                }
                &[a, b, c, sum, carry] => {
                    let mut key = [a, b, c];
                    key.sort_unstable();
                    self.sums.insert(key, (sum, carry));
                }
                _ => unreachable!(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn equality_transport_identifies_distant_products() {
        let mut graph = Correlation::blank(false);
        let a = graph.cell();
        let b = graph.cell();
        let c = graph.cell();
        let left = graph.and(a, c);
        let right = graph.and(b, c);
        graph.gate(vec![a, b], midpoint::EQUAL);
        graph.assign(
            Literal {
                cell: left,
                mark: ONE,
            },
            None,
        );
        graph.p = vec![right];
        graph.connect();
        assert!(!graph.empty);
        assert_eq!(graph.values[graph.p[0]], Some(ONE));
        assert_eq!(graph.products.len(), 1);
    }
    #[test]
    fn equality_loop_rejects_inconsistent_root_transport() {
        let mut graph = Correlation::blank(false);
        let a = graph.cell();
        let b = graph.cell();
        let c = graph.cell();
        graph.gate(vec![a, b], midpoint::EQUAL);
        graph.gate(vec![b, c], midpoint::EQUAL);
        graph.assign(
            Literal {
                cell: a,
                mark: ZERO,
            },
            None,
        );
        graph.assign(Literal { cell: c, mark: ONE }, None);
        graph.connect();
        assert!(graph.empty);
    }
    #[test]
    fn complement_transport_closes_even_loops_without_pinning_them() {
        let mut graph = Correlation::blank(false);
        let a = graph.cell();
        let b = graph.cell();
        let c = graph.cell();
        let d = graph.cell();
        for (left, right) in [(a, b), (b, c), (c, d), (d, a)] {
            graph.gate(vec![left, right], NOT);
        }
        graph.p = vec![a, b, c, d];
        graph.connect();
        assert!(!graph.empty);
        assert_eq!(graph.p[0], graph.p[2]);
        assert_eq!(graph.p[1], graph.p[3]);
        assert_ne!(graph.p[0], graph.p[1]);
        for &cell in &graph.p {
            assert_eq!(graph.values[cell], None);
        }
    }
    #[test]
    fn odd_complement_holonomy_rejects_an_unpinned_loop() {
        let mut graph = Correlation::blank(false);
        let a = graph.cell();
        let b = graph.cell();
        let c = graph.cell();
        for (left, right) in [(a, b), (b, c), (c, a)] {
            graph.gate(vec![left, right], NOT);
        }
        graph.connect();
        assert!(graph.empty);
        assert_eq!(graph.decisions, 0);
    }
    #[test]
    fn transported_complement_product_folds_without_choosing_its_input() {
        let mut graph = Correlation::blank(false);
        let a = graph.cell();
        let b = graph.not(a);
        let product = graph.and(a, b);
        graph.p = vec![a, b, product];
        graph.connect();
        assert!(!graph.empty);
        assert_eq!(graph.p[2], 0);
        assert_eq!(graph.values[graph.p[0]], None);
        assert_eq!(graph.values[graph.p[1]], None);
    }
}
