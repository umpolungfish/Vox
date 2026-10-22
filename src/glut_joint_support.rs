//! Exact split/fuse of two elementary supports along their shared cells.
//! The source gates stay resident. A reusable joint table returns implications
//! to their existing cells; no factor candidate or free orientation is inserted.
use super::*;
use alloc::collections::BTreeSet;

pub(super) struct JointSupport {
    parents: (usize, usize),
    cells: Vec<usize>,
    supports: Vec<[u64; 2]>,
    all: u64,
}

impl JointSupport {
    fn new(left: usize, right: usize, gates: &[Gate]) -> Self {
        let a = &gates[left];
        let b = &gates[right];
        let mut cells = a.cells.clone();
        cells.extend_from_slice(&b.cells);
        cells.sort_unstable();
        cells.dedup();
        let mut supports = vec![[0u64; 2]; cells.len()];
        let mut count = 0;
        for x in 0..8 {
            if a.all & (1 << x) == 0 { continue; }
            for y in 0..8 {
                if b.all & (1 << y) == 0 { continue; }
                let mut marks = vec![None; cells.len()];
                let mut valid = true;
                for (gate, row) in [(a, x), (b, y)] {
                    for (slot, &cell) in gate.cells.iter().enumerate() {
                        let at = cells.binary_search(&cell).unwrap();
                        let mark = usize::from(gate.supports[slot][1] & (1 << row) != 0);
                        if marks[at].is_some_and(|old| old != mark) { valid = false; }
                        marks[at] = Some(mark);
                    }
                }
                if !valid { continue; }
                for (slot, mark) in marks.into_iter().enumerate() {
                    supports[slot][mark.unwrap()] |= 1u64 << count;
                }
                count += 1;
            }
        }
        let all = if count == 64 { u64::MAX } else { (1u64 << count) - 1 };
        Self { parents: (left, right), cells, supports, all }
    }
}

impl Correlation {
    pub(super) fn connect_joint_support(&mut self) {
        // At least two shared nonconstant cells are required. Joining through
        // only one cell adds no unary information beyond elementary support.
        let mut seams: BTreeMap<(usize, usize), Vec<usize>> = BTreeMap::new();
        let mut pairs = BTreeSet::new();
        for (index, gate) in self.gates.iter().enumerate() {
            let mut cells: Vec<_> = gate.cells.iter().copied().filter(|&c| c > 1).collect();
            cells.sort_unstable();
            cells.dedup();
            for (i, &a) in cells.iter().enumerate() {
                for &b in &cells[i + 1..] {
                    let previous = seams.entry((a, b)).or_default();
                    for &other in previous.iter() { pairs.insert((other, index)); }
                    previous.push(index);
                }
            }
        }
        self.joints = pairs.into_iter().map(|(a, b)| JointSupport::new(a, b, &self.gates)).collect();
        self.joint_offsets = vec![0; self.values.len() + 1];
        for joint in &self.joints {
            for &cell in &joint.cells { self.joint_offsets[cell + 1] += 1; }
        }
        for i in 1..self.joint_offsets.len() { self.joint_offsets[i] += self.joint_offsets[i - 1]; }
        self.joint_incidence = vec![0; *self.joint_offsets.last().unwrap()];
        let mut next = self.joint_offsets[..self.values.len()].to_vec();
        for (index, joint) in self.joints.iter().enumerate() {
            for &cell in &joint.cells {
                self.joint_incidence[next[cell]] = index;
                next[cell] += 1;
            }
        }
    }

    pub(super) fn joint_step(&mut self, index: usize) -> Result<(), Vec<Literal>> {
        let joint = &self.joints[index];
        debug_assert!(joint.parents.0 < joint.parents.1);
        let mut viable = joint.all;
        let mut reason = Vec::new();
        for (slot, &cell) in joint.cells.iter().enumerate() {
            if let Some(mark) = self.values[cell] {
                viable &= joint.supports[slot][usize::from(mark == ONE)];
                reason.push(Literal { cell, mark }.opposite());
            }
        }
        self.joint_returns += 1;
        self.work += joint.cells.len() * 2;
        if viable == 0 { return Err(reason); }
        let forced: Vec<_> = joint.cells.iter().enumerate().filter_map(|(slot, &cell)| {
            if self.values[cell].is_some() { return None; }
            let mark = if viable & joint.supports[slot][0] == 0 { ONE }
                else if viable & joint.supports[slot][1] == 0 { ZERO }
                else { return None; };
            Some(Literal { cell, mark })
        }).collect();
        for lit in forced {
            let mut ancestry = reason.clone();
            ancestry.push(lit);
            if !self.assign(lit, Some(ancestry.clone())) { return Err(ancestry); }
            self.joint_forced += 1;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_202_joint_returns_preserve_exact_parent_supports() {
        extern crate std;
        let n = crate::godel_calculus::Nat::from_decimal(
            "4479106148067194240380817417142593728490011937516752269508037").unwrap();
        let tape = mf::parse_numeral(&crate::godel_calculus::encode_cell_binary(&n)).unwrap();
        assert_eq!(tape.len(), 202);
        let mut graph = Correlation::new_nested(&tape, tape.len().div_ceil(2), &mf::isqrt(&tape));
        assert!(!graph.joints.is_empty());
        for joint in &graph.joints {
            for assignment in 0..1usize << joint.cells.len() {
                let mut viable = joint.all;
                for slot in 0..joint.cells.len() {
                    viable &= joint.supports[slot][(assignment >> slot) & 1];
                }
                let accepted = [joint.parents.0, joint.parents.1].into_iter().all(|index| {
                    let gate = &graph.gates[index];
                    let mut rows = gate.all;
                    for (slot, cell) in gate.cells.iter().enumerate() {
                        let at = joint.cells.binary_search(cell).unwrap();
                        rows &= gate.supports[slot][(assignment >> at) & 1];
                    }
                    rows != 0
                });
                assert_eq!(viable != 0, accepted);
            }
        }
        let source = graph.source.clone();
        for _ in 0..8 {
            match graph.advance() {
                Step::Running => {},
                Step::Closed(p,q) => { assert_eq!(mf::mul(&p,&q), tape); break; },
                Step::Empty => panic!("source-bound relation lost all alternatives"),
            }
        }
        assert_eq!(graph.source, source);
        assert!(graph.joint_returns > 0);
        std::println!("bits=202 joints={} returns={} forced={} decisions={} conflicts={}",
            graph.joints.len(), graph.joint_returns, graph.joint_forced, graph.decisions, graph.conflicts);
    }
}
