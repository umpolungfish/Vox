//! Quotient shared cells by equality transport and expression congruence.
use super::*;

fn representative(parent: &mut [usize], cell: usize) -> usize {
    let mut root = cell;
    while parent[root] != root { root = parent[root]; }
    let mut next = cell;
    while parent[next] != next {
        let old = parent[next]; parent[next] = root; next = old;
    }
    root
}
fn join(parent: &mut [usize], left: usize, right: usize) -> bool {
    let a = representative(parent, left);
    let b = representative(parent, right);
    if a == b { return false; }
    parent[a.max(b)] = a.min(b);
    true
}
impl Correlation {
    pub(super) fn fold_transport(&mut self) {
        assert!(self.boundaries.is_empty() && self.learned.is_empty());
        let mut parent: Vec<_> = (0..self.values.len()).collect();
        loop {
            let mut changed = false;
            let mut expressions: BTreeMap<Vec<usize>, Vec<usize>> = BTreeMap::new();
            for gate in &self.gates {
                let cells: Vec<_> = gate.cells.iter()
                    .map(|&cell| representative(&mut parent, cell)).collect();
                if cells.len() == 2 && gate.supports[1][0] == 0b01 {
                    changed |= join(&mut parent, cells[0], cells[1]);
                    continue;
                }
                let inputs = match cells.len() { 2 => 1, 3 => 2, 5 => 3, _ => unreachable!() };
                let mut key = cells[..inputs].to_vec();
                key.sort_unstable();
                key.insert(0, cells.len());
                if let Some(outputs) = expressions.get(&key) {
                    for (&left, &right) in outputs.iter().zip(&cells[inputs..]) {
                        changed |= join(&mut parent, left, right);
                    }
                } else { expressions.insert(key, cells[inputs..].to_vec()); }
            }
            if !changed { break; }
        }
        for cell in 0..parent.len() { representative(&mut parent, cell); }
        let mut values = vec![None; self.values.len()];
        for (cell, &value) in self.values.iter().enumerate() {
            if let Some(mark) = value {
                let root = parent[cell];
                if values[root].is_some_and(|other| other != mark) { self.empty = true; }
                else { values[root] = Some(mark); }
            }
        }
        // Both constant vertices must retain their distinct truth marks.
        if parent[0] == parent[1] { self.empty = true; return; }
        self.values = values;
        self.trail.clear(); self.cursor = 0;
        self.reasons.fill(None); self.levels.fill(0);
        for (cell, &value) in self.values.iter().enumerate() {
            if let Some(mark) = value { self.trail.push(Literal { cell, mark }); }
        }
        for cell in self.p.iter_mut().chain(&mut self.q) { *cell = parent[*cell]; }
        self.products.clear(); self.complements.clear(); self.sums.clear();
        self.parity.clear(); self.parity_offsets.clear(); self.parity_incidence.clear();
        let mut seen = alloc::collections::BTreeSet::new();
        self.gates.retain_mut(|gate| {
            for cell in &mut gate.cells { *cell = parent[*cell]; }
            if gate.cells.len() == 2 && gate.supports[1][0] == 0b01 { return false; }
            seen.insert(gate.cells.clone())
        });
        for gate in &self.gates {
            match gate.cells.as_slice() {
                &[a,b,out] => { self.products.insert((a.min(b),a.max(b)),out); }
                &[a,out] => { self.complements.insert(a,out); self.complements.insert(out,a); }
                &[a,b,c,sum,carry] => {
                    let mut key=[a,b,c]; key.sort_unstable(); self.sums.insert(key,(sum,carry));
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
        let a=graph.cell(); let b=graph.cell(); let c=graph.cell();
        let left=graph.and(a,c); let right=graph.and(b,c);
        graph.gate(vec![a,b], midpoint::EQUAL);
        graph.assign(Literal {cell:left,mark:ONE},None);
        graph.p=vec![right]; graph.connect();
        assert!(!graph.empty);
        assert_eq!(graph.values[graph.p[0]],Some(ONE));
        assert_eq!(graph.products.len(),1);
    }
    #[test]
    fn equality_loop_rejects_inconsistent_root_transport() {
        let mut graph=Correlation::blank(false);
        let a=graph.cell(); let b=graph.cell(); let c=graph.cell();
        graph.gate(vec![a,b],midpoint::EQUAL);
        graph.gate(vec![b,c],midpoint::EQUAL);
        graph.assign(Literal {cell:a,mark:ZERO},None);
        graph.assign(Literal {cell:c,mark:ONE},None);
        graph.connect(); assert!(graph.empty);
    }
}
