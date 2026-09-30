//! Sparse parity elimination across shared carry and equality frames.
use super::*;
pub(super) struct ParityRow {
    pub cells: Vec<usize>,
    pub mark: char,
}
fn xor_mark(left: char, right: char) -> char {
    if left == right { ZERO } else { ONE }
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
impl Correlation {
    pub(super) fn connect_parity(&mut self) {
        assert!(self.boundaries.is_empty(), "parity preparation requires the source root");
        let mut basis: BTreeMap<usize, ParityRow> = BTreeMap::new();
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
            while let Some(&pivot) = cells.last() {
                if let Some(row) = basis.get(&pivot) {
                    cells = xor_cells(&cells, &row.cells);
                    mark = xor_mark(mark, row.mark);
                } else { break; }
            }
            if let Some(&pivot) = cells.last() { basis.insert(pivot, ParityRow { cells, mark }); }
            else if mark == ONE { self.empty = true; }
        }
        let mut rows: Vec<_> = basis.into_values().collect();
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
    pub(super) fn parity_step(&mut self, index: usize) -> Result<(), Vec<Literal>> {
        let row = &self.parity[index];
        let mut mark = row.mark;
        let mut free = None;
        let mut free_count = 0;
        let mut premise = Vec::new();
        for &cell in &row.cells {
            if let Some(value) = self.values[cell] {
                mark = xor_mark(mark, value);
                premise.push(Literal { cell, mark: value }.opposite());
            } else { free = Some(cell); free_count += 1; }
        }
        if free_count == 0 {
            if mark == ONE { return Err(premise); }
        } else if free_count == 1 {
            let lit = Literal { cell: free.unwrap(), mark };
            premise.push(lit);
            if !self.assign(lit, Some(premise.clone())) { return Err(premise); }
        }
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
