//! DQI syndrome elimination nested inside the GLUT shared-cell membrane.
//! Ported from G-mOMonadOS src/dqi.rs build_rows/eliminate. Sorted sparse
//! cell addresses replace machine limbs; the syndrome stays in IMASM marks.
//! Free columns remain resident and are never assigned a selected zero value.
use super::*;
pub(super) struct ParityRow {
    pub cells: Vec<usize>,
    pub mark: char,
}
fn xor_mark(left: char, right: char) -> char {
    quantum::controlled_x(left, right)
}
fn xor_cells(left: &[usize], right: &[usize]) -> Vec<usize> {
    let mut result = Vec::new();
    let (mut i, mut j) = (0, 0);
    while i < left.len() || j < right.len() {
        match (left.get(i), right.get(j)) {
            (Some(&a), Some(&b)) if a == b => { i += 1; j += 1; }
            (Some(&a), Some(&b)) if a < b => { result.push(a); i += 1; }
            (Some(_), Some(&b)) => { result.push(b); j += 1; }
            (Some(&a), None) => { result.push(a); i += 1; }
            (None, Some(&b)) => { result.push(b); j += 1; }
            (None, None) => break,
        }
    }
    result
}
/// Streaming echelon form of DQI's XOR row elimination. Rows grow with the
/// actual shared cells. Equal columns cancel before a pivot is retained.
#[derive(Default)]
struct DqiBasis {
    rows: BTreeMap<usize, ParityRow>,
    inconsistent: bool,
}
impl DqiBasis {
    fn insert(&mut self, mut row: ParityRow) {
        while let Some(&pivot) = row.cells.last() {
            let Some(other) = self.rows.get(&pivot) else {
                self.rows.insert(pivot, row);
                return;
            };
            row.cells = xor_cells(&row.cells, &other.cells);
            row.mark = xor_mark(row.mark, other.mark);
        }
        self.inconsistent |= row.mark == ONE;
    }
}
impl Correlation {
    #[cfg(test)]
    pub(super) fn connect_parity(&mut self) {
        self.connect_parity_observed(&mut |_, _, _| {});
    }
    pub(super) fn connect_parity_observed(&mut self, observe: &mut impl FnMut(char, usize, usize)) {
        assert!(self.boundaries.is_empty(), "parity preparation requires the source root");
        let mut basis = DqiBasis::default();
        for (index, gate) in self.gates.iter().enumerate() {
            if index % self.source.len().max(1) == 0 {
                observe('∈', self.gate_count(), self.cell_count());
            }
            let (slots, mut mark) = match gate.cells.len() {
                5 => (&gate.cells[..4], ZERO),
                2 => (&gate.cells[..], if gate.supports[1][0] == 0b10 { ONE } else { ZERO }),
                _ => continue,
            };
            let mut cells = Vec::new();
            for &cell in slots {
                if let Some(value) = self.values[cell] { mark = xor_mark(mark, value); }
                else {
                    match cells.binary_search(&cell) {
                        Ok(i) => { cells.remove(i); }
                        Err(i) => cells.insert(i, cell),
                    }
                }
            }
            basis.insert(ParityRow { cells, mark });
        }
        self.empty |= basis.inconsistent;
        observe('⊞', self.gate_count(), self.cell_count());
        let mut rows: Vec<_> = basis.rows.into_values().collect();
        let mut occurs: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
        for (index, row) in rows.iter().enumerate() {
            for &cell in &row.cells { occurs.entry(cell).or_default().push(index); }
        }
        // Unit pins and two-cell phase edges substitute across the combined
        // shared frames without widening a target row. Longer echelon rows
        // remain resident constraints and propagate through their incidence
        // bank; expanding them into every higher row creates dense fill-in.
        observe('⋈', self.gate_count(), self.cell_count());
        for index in 0..rows.len() {
            if index % self.source.len().max(1) == 0 {
                observe('≺', self.gate_count(), self.cell_count());
            }
            let pivot = *rows[index].cells.last().unwrap();
            if !matches!(rows[index].cells.as_slice(), [_] | [_, _]) { continue; }
            let source = rows[index].cells.clone();
            let mark = rows[index].mark;
            for target in occurs.remove(&pivot).unwrap_or_default() {
                if target <= index || rows[target].cells.binary_search(&pivot).is_err() { continue; }
                for &cell in &source {
                    if cell != pivot && rows[target].cells.binary_search(&cell).is_err() {
                        occurs.entry(cell).or_default().push(target);
                    }
                }
                rows[target].cells = xor_cells(&rows[target].cells, &source);
                rows[target].mark = xor_mark(rows[target].mark, mark);
            }
        }
        observe('⊤', self.gate_count(), self.cell_count());
        self.parity_state = rows.iter().map(|row| (
            row.cells.len(), row.cells.iter().fold(0, |a, &b| a ^ b), row.mark
        )).collect();
        self.parity_pairs = self.parity_state.iter().enumerate()
            .filter_map(|(index, state)| (state.0 == 2).then_some(index)).collect();
        self.parity = rows;
        self.parity_offsets = vec![0; self.values.len()+1];
        for row in &self.parity {
            for &cell in &row.cells { self.parity_offsets[cell+1] += 1; }
        }
        for i in 1..self.parity_offsets.len() { self.parity_offsets[i] += self.parity_offsets[i-1]; }
        self.parity_incidence = vec![0; *self.parity_offsets.last().unwrap()];
        let mut next = self.parity_offsets[..self.values.len()].to_vec();
        for (index, row) in self.parity.iter().enumerate() {
            for &cell in &row.cells { self.parity_incidence[next[cell]] = index; next[cell] += 1; }
        }
        // Singleton rows ascend immediately. Their assigned cells then route
        // through both the original gates and the folded incidence relation.
        for index in 0..self.parity.len() {
            if self.parity_step(index).is_err() { self.empty = true; }
        }
    }
    /// Two-free-cell syndrome rows are phase edges. Close their relative
    /// turns together so an odd loop rejects the branch before another split.
    /// The XOR of the actual loop rows supplies the conflict ancestry.
    pub(super) fn parity_holonomy(&mut self) -> Result<(), Vec<Literal>> {
        use alloc::collections::BTreeSet;
        let mut edges: BTreeMap<usize, Vec<(usize, char, usize)>> = BTreeMap::new();
        for &index in &self.parity_pairs {
            self.work += 1;
            let (_, pair, mark) = self.parity_state[index];
            let a = *self.parity[index].cells.iter()
                .find(|&&cell| self.values[cell].is_none()).unwrap();
            let b = pair ^ a;
            edges.entry(a).or_default().push((b, mark, index));
            edges.entry(b).or_default().push((a, mark, index));
        }
        let mut turns = BTreeMap::new();
        let mut ancestry: BTreeMap<usize, (usize, Option<usize>)> = BTreeMap::new();
        for &start in edges.keys() {
            if turns.contains_key(&start) { continue; }
            turns.insert(start, ZERO);
            ancestry.insert(start, (start, None));
            let mut pending = vec![start];
            while let Some(cell) = pending.pop() {
                for &(next, turn, row) in &edges[&cell] {
                    self.work += 1;
                    let carried = quantum::controlled_x(turns[&cell], turn);
                    if let Some(&previous) = turns.get(&next) {
                        if previous == carried { continue; }
                        let mut loop_rows = BTreeSet::new();
                        loop_rows.insert(row);
                        for mut at in [cell, next] {
                            while let (parent, Some(edge)) = ancestry[&at] {
                                if !loop_rows.insert(edge) { loop_rows.remove(&edge); }
                                at = parent;
                            }
                        }
                        let mut support = BTreeSet::new();
                        for edge in loop_rows {
                            for &source in &self.parity[edge].cells {
                                if !support.insert(source) { support.remove(&source); }
                            }
                        }
                        let mut reason = Vec::new();
                        for source in support {
                            let mark = self.values[source].expect("phase loop cancels free cells");
                            if self.levels[source] != 0 {
                                reason.push(Literal { cell: source, mark }.opposite());
                            }
                        }
                        return Err(reason);
                    }
                    turns.insert(next, carried);
                    ancestry.insert(next, (cell, Some(row)));
                    pending.push(next);
                }
            }
        }
        Ok(())
    }
    pub(super) fn parity_assignment(&mut self, lit: Literal, retreat: bool) {
        if self.parity_offsets.len() <= lit.cell + 1 { return; }
        for slot in self.parity_offsets[lit.cell]..self.parity_offsets[lit.cell + 1] {
            let index = self.parity_incidence[slot];
            let state = &mut self.parity_state[index];
            if state.0 == 2 { self.parity_pairs.remove(&index); }
            if retreat { state.0 += 1; } else { state.0 -= 1; }
            state.1 ^= lit.cell;
            state.2 = xor_mark(state.2, lit.mark);
            if state.0 == 2 { self.parity_pairs.insert(index); }
        }
    }
    pub(super) fn parity_step(&mut self, index: usize) -> Result<(), Vec<Literal>> {
        let (free_count, free_cell, mark) = self.parity_state[index];
        if free_count > 1 || free_count == 0 && mark == ZERO { return Ok(()); }
        let mut premise = Vec::new();
        for &cell in &self.parity[index].cells {
            if let Some(value) = self.values[cell] {
                premise.push(Literal { cell, mark: value }.opposite());
            }
        }
        if free_count == 0 { return Err(premise); }
        let lit = Literal { cell: free_cell, mark };
        premise.push(lit);
        if !self.assign(lit, Some(premise.clone())) { return Err(premise); }
        Ok(())
    }

}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn resident_64_edge_phase_loop_returns_its_pins_and_rewinds() {
        let mut graph = Correlation::blank(false);
        let edges = 64;
        let cells: Vec<_> = (0..edges).map(|_| graph.cell()).collect();
        let pins: Vec<_> = (0..edges).map(|_| graph.cell()).collect();
        for index in 0..edges {
            let (sum, _) = graph.adder(cells[index], cells[(index + 1) % edges], pins[index]);
            assert!(graph.assign(Literal { cell: sum, mark: ZERO }, None));
        }
        graph.connect_parity();
        assert!(!graph.empty);
        assert!(graph.parity_holonomy().is_ok());
        graph.boundaries.push(graph.trail.len());
        for (index, &cell) in pins.iter().enumerate() {
            let mark = if index + 1 == edges { ONE } else { ZERO };
            assert!(graph.assign(Literal { cell, mark }, None));
        }
        let reason = graph.parity_holonomy().expect_err("odd relative phase must return");
        assert_eq!(reason.len(), pins.len());
        for &cell in &pins {
            assert!(reason.contains(&Literal { cell, mark: graph.values[cell].unwrap() }.opposite()));
        }
        graph.retreat(0);
        assert!(graph.parity_holonomy().is_ok());
        for (row, &(free, address, residual)) in graph.parity.iter().zip(&graph.parity_state) {
            let mut expected = (0, 0, row.mark);
            for &cell in &row.cells {
                if let Some(mark) = graph.values[cell] { expected.2 = xor_mark(expected.2, mark); }
                else { expected.0 += 1; expected.1 ^= cell; }
            }
            assert_eq!((free, address, residual), expected);
        }
    }
    #[test]
    fn parity_closes_an_unassigned_odd_not_cycle() {
        let mut graph = Correlation::blank(false);
        let a=graph.cell(); let b=graph.cell(); let c=graph.cell();
        graph.gate(vec![a,b], NOT); graph.gate(vec![b,c], NOT); graph.gate(vec![c,a], NOT);
        graph.connect_parity(); assert!(graph.empty);
    }
    #[test]
    fn parity_folds_equal_inputs_into_the_shared_sum_cell() {
        let mut graph = Correlation::blank(false);
        let a=graph.cell(); let b=graph.cell();
        let (sum, carry)=graph.adder(a,b,0);
        graph.gate(vec![a,b], midpoint::EQUAL);
        graph.connect_parity();
        assert!(!graph.empty); assert_eq!(graph.values[sum], Some(ZERO));
        assert_eq!(graph.values[a], None); assert_eq!(graph.values[carry], None);
    }
}
