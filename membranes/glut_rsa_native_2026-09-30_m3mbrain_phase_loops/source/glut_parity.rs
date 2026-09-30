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
    pub(super) fn connect_parity(&mut self) {
        assert!(self.boundaries.is_empty(), "parity preparation requires the source root");
        let mut basis = DqiBasis::default();
        for gate in &self.gates {
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
        let mut rows: Vec<_> = basis.rows.into_values().collect();
        let mut occurs: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
        for (index, row) in rows.iter().enumerate() {
            for &cell in &row.cells { occurs.entry(cell).or_default().push(index); }
        }
        // Ascend through the same adjacent pivots. A lower equality reduces
        // every higher row referring to it, including distant sum frames.
        for index in 0..rows.len() {
            let pivot = *rows[index].cells.last().unwrap();
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
        self.parity_state = rows.iter().map(|row| (
            row.cells.len(), row.cells.iter().fold(0, |a, &b| a ^ b), row.mark
        )).collect();
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
        for (index, &(free, pair, mark)) in self.parity_state.iter().enumerate() {
            self.work += 1;
            if free != 2 { continue; }
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
            let state = &mut self.parity_state[self.parity_incidence[slot]];
            if retreat { state.0 += 1; } else { state.0 -= 1; }
            state.1 ^= lit.cell;
            state.2 = xor_mark(state.2, lit.mark);
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
