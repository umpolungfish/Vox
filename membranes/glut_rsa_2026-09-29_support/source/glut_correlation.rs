//! Shared glyph-valued multiplier gates and retained conflict implications.
//! Factor, product and carry cells refer to the same vertices across columns.
use super::{bit, mf, trim, ONE, ZERO};
use alloc::{vec, vec::Vec};

#[derive(Clone, Copy, PartialEq, Eq)]
struct Literal {
    cell: usize,
    mark: char,
}
impl Literal {
    fn opposite(self) -> Self {
        Self {
            cell: self.cell,
            mark: if self.mark == ONE { ZERO } else { ONE },
        }
    }
    fn slot(self) -> usize {
        self.cell * 2 + usize::from(self.mark == ONE)
    }
}
struct Gate {
    cells: Vec<usize>,
    rows: &'static [&'static [char]],
}
const AND: &[&[char]] = &[
    &[ZERO, ZERO, ZERO],
    &[ZERO, ONE, ZERO],
    &[ONE, ZERO, ZERO],
    &[ONE, ONE, ONE],
];
const NOT: &[&[char]] = &[&[ZERO, ONE], &[ONE, ZERO]];
const ADD: &[&[char]] = &[
    &[ZERO, ZERO, ZERO, ZERO, ZERO],
    &[ZERO, ZERO, ONE, ONE, ZERO],
    &[ZERO, ONE, ZERO, ONE, ZERO],
    &[ZERO, ONE, ONE, ZERO, ONE],
    &[ONE, ZERO, ZERO, ONE, ZERO],
    &[ONE, ZERO, ONE, ZERO, ONE],
    &[ONE, ONE, ZERO, ZERO, ONE],
    &[ONE, ONE, ONE, ONE, ONE],
];

pub(super) enum Step {
    Running,
    Closed(Vec<char>, Vec<char>),
    Empty,
}
pub(super) struct Correlation {
    gates: Vec<Gate>,
    adjacent: Vec<Vec<usize>>,
    values: Vec<Option<char>>,
    levels: Vec<usize>,
    reasons: Vec<Option<Vec<Literal>>>,
    activity: Vec<f64>,
    bump: f64,
    trail: Vec<Literal>,
    boundaries: Vec<usize>,
    cursor: usize,
    learned: Vec<Vec<Literal>>,
    watches: Vec<Vec<usize>>,
    p: Vec<usize>,
    q: Vec<usize>,
    empty: bool,
    pub decisions: usize,
    pub conflicts: usize,
}
impl Correlation {
    pub fn new(n: &[char], width: usize, upper: &[char], lower: &[char]) -> Self {
        let mut s = Self {
            gates: Vec::new(),
            adjacent: Vec::new(),
            values: Vec::new(),
            levels: Vec::new(),
            reasons: Vec::new(),
            activity: Vec::new(),
            bump: 1.0,
            trail: Vec::new(),
            boundaries: Vec::new(),
            cursor: 0,
            learned: Vec::new(),
            watches: Vec::new(),
            p: Vec::new(),
            q: Vec::new(),
            empty: width < 2,
            decisions: 0,
            conflicts: 0,
        };
        let zero = s.cell();
        let one = s.cell();
        assert_eq!((zero, one), (0, 1));
        s.assign(
            Literal {
                cell: zero,
                mark: ZERO,
            },
            None,
        );
        s.assign(
            Literal {
                cell: one,
                mark: ONE,
            },
            None,
        );
        s.p = (0..width)
            .map(|i| {
                if i == 0 || i + 1 == width {
                    one
                } else {
                    s.cell()
                }
            })
            .collect();
        s.q = (0..width)
            .map(|i| {
                if i == 0 || i + 1 == width {
                    one
                } else {
                    s.cell()
                }
            })
            .collect();
        let mut columns = vec![Vec::new(); width * 2 + 1];
        for i in 0..width {
            for j in 0..width {
                let product = s.and(s.p[i], s.q[j]);
                if product != zero {
                    columns[i + j].push(product);
                }
            }
        }
        // Carry-save folding transports each carry to the adjacent column.
        for k in 0..columns.len() - 1 {
            while columns[k].len() >= 3 {
                let a = columns[k].pop().unwrap();
                let b = columns[k].pop().unwrap();
                let c = columns[k].pop().unwrap();
                let (sum, carry) = s.adder(a, b, c);
                if sum != zero {
                    columns[k].push(sum);
                }
                if carry != zero {
                    columns[k + 1].push(carry);
                }
            }
        }
        let mut carry = zero;
        for (k, column) in columns.iter().enumerate() {
            let (sum, next) = s.adder(
                column.first().copied().unwrap_or(zero),
                column.get(1).copied().unwrap_or(zero),
                carry,
            );
            if !s.assign(
                Literal {
                    cell: sum,
                    mark: if bit(n, k) == 1 { ONE } else { ZERO },
                },
                None,
            ) {
                s.empty = true;
            }
            carry = next;
        }
        if !s.assign(
            Literal {
                cell: carry,
                mark: ZERO,
            },
            None,
        ) {
            s.empty = true;
        }
        let p = s.p.clone();
        let q = s.q.clone();
        s.bound(&p, upper, false);
        s.bound(&q, lower, true);
        s
    }
    pub fn gate_count(&self) -> usize {
        self.gates.len()
    }
    pub fn cell_count(&self) -> usize {
        self.values.len()
    }
    fn cell(&mut self) -> usize {
        let index = self.values.len();
        self.values.push(None);
        self.adjacent.push(Vec::new());
        self.levels.push(0);
        self.reasons.push(None);
        self.activity.push(0.0);
        self.watches.push(Vec::new());
        self.watches.push(Vec::new());
        index
    }
    fn gate(&mut self, cells: Vec<usize>, rows: &'static [&'static [char]]) {
        let index = self.gates.len();
        for (slot, &cell) in cells.iter().enumerate() {
            if !cells[..slot].contains(&cell) {
                self.adjacent[cell].push(index);
            }
        }
        self.gates.push(Gate { cells, rows });
    }
    fn and(&mut self, a: usize, b: usize) -> usize {
        if a == 0 || b == 0 {
            0
        } else if a == 1 {
            b
        } else if b == 1 || a == b {
            a
        } else {
            let out = self.cell();
            self.gate(vec![a, b, out], AND);
            out
        }
    }
    fn not(&mut self, a: usize) -> usize {
        if a == 0 {
            1
        } else if a == 1 {
            0
        } else {
            let out = self.cell();
            self.gate(vec![a, out], NOT);
            out
        }
    }
    fn adder(&mut self, a: usize, b: usize, c: usize) -> (usize, usize) {
        if a == b {
            return (c, a);
        }
        if a == c {
            return (b, a);
        }
        if b == c {
            return (a, b);
        }
        if (a == 0 && b == 1) || (a == 1 && b == 0) {
            return (self.not(c), c);
        }
        if (a == 0 && c == 1) || (a == 1 && c == 0) {
            return (self.not(b), b);
        }
        if (b == 0 && c == 1) || (b == 1 && c == 0) {
            return (self.not(a), a);
        }
        let sum = self.cell();
        let carry = self.cell();
        self.gate(vec![a, b, c, sum, carry], ADD);
        (sum, carry)
    }
    fn bound(&mut self, cells: &[usize], value: &[char], lower: bool) {
        let mut equal = 1;
        for k in (0..cells.len()).rev() {
            let mark = bit(value, k) == 1;
            let mismatch = if mark { self.not(cells[k]) } else { cells[k] };
            if mark == lower {
                let forbidden = self.and(equal, mismatch);
                if !self.assign(
                    Literal {
                        cell: forbidden,
                        mark: ZERO,
                    },
                    None,
                ) {
                    self.empty = true;
                }
            }
            let matching = if mark { cells[k] } else { self.not(cells[k]) };
            equal = self.and(equal, matching);
        }
    }
    fn assign(&mut self, lit: Literal, reason: Option<Vec<Literal>>) -> bool {
        if let Some(mark) = self.values[lit.cell] {
            return mark == lit.mark;
        }
        self.values[lit.cell] = Some(lit.mark);
        self.levels[lit.cell] = self.boundaries.len();
        self.reasons[lit.cell] = reason;
        self.trail.push(lit);
        true
    }
    fn satisfied(&self, lit: Literal) -> Option<bool> {
        self.values[lit.cell].map(|mark| mark == lit.mark)
    }
    fn gate_step(&mut self, index: usize) -> Result<(), Vec<Literal>> {
        let gate = &self.gates[index];
        let mut support = vec![None; gate.cells.len()];
        let mut any = false;
        for row in gate.rows {
            let fits = gate.cells.iter().enumerate().all(|(i, &cell)| {
                self.values[cell].is_none_or(|mark| mark == row[i])
                    && gate.cells[..i]
                        .iter()
                        .enumerate()
                        .all(|(j, &previous)| previous != cell || row[j] == row[i])
            });
            if !fits {
                continue;
            }
            if !any {
                for (out, &mark) in support.iter_mut().zip(row.iter()) {
                    *out = Some(mark);
                }
            } else {
                for (out, &mark) in support.iter_mut().zip(row.iter()) {
                    if *out != Some(mark) {
                        *out = None;
                    }
                }
            }
            any = true;
        }
        let mut premise = Vec::new();
        for &cell in &gate.cells {
            if let Some(mark) = self.values[cell] {
                let lit = Literal { cell, mark }.opposite();
                if !premise.contains(&lit) {
                    premise.push(lit);
                }
            }
        }
        if !any {
            return Err(premise);
        }
        let forced: Vec<_> = gate
            .cells
            .iter()
            .copied()
            .zip(support)
            .filter_map(|(cell, mark)| {
                mark.filter(|_| self.values[cell].is_none())
                    .map(|mark| Literal { cell, mark })
            })
            .collect();
        for lit in forced {
            let mut reason = premise.clone();
            reason.push(lit);
            if !self.assign(lit, Some(reason.clone())) {
                return Err(reason);
            }
        }
        Ok(())
    }
    fn propagate(&mut self) -> Result<(), Vec<Literal>> {
        while self.cursor < self.trail.len() {
            let assigned = self.trail[self.cursor];
            self.cursor += 1;
            for slot in 0..self.adjacent[assigned.cell].len() {
                let gate = self.adjacent[assigned.cell][slot];
                self.gate_step(gate)?;
            }
            let false_lit = assigned.opposite();
            let key = false_lit.slot();
            let mut waiting = core::mem::take(&mut self.watches[key]);
            while let Some(index) = waiting.pop() {
                if self.learned[index][0] == false_lit {
                    self.learned[index].swap(0, 1);
                }
                let other = self.learned[index][0];
                if self.satisfied(other) == Some(true) {
                    self.watches[key].push(index);
                    continue;
                }
                let replacement = (2..self.learned[index].len())
                    .find(|&i| self.satisfied(self.learned[index][i]) != Some(false));
                if let Some(position) = replacement {
                    self.learned[index].swap(1, position);
                    self.watches[self.learned[index][1].slot()].push(index);
                } else {
                    self.watches[key].push(index);
                    let reason = self.learned[index].clone();
                    if self.satisfied(other) == Some(false)
                        || !self.assign(other, Some(reason.clone()))
                    {
                        self.watches[key].extend(waiting);
                        return Err(reason);
                    }
                }
            }
        }
        Ok(())
    }
    fn analyze(&mut self, mut conflict: Vec<Literal>) -> (Vec<Literal>, usize) {
        let level = self.boundaries.len();
        let mut seen = vec![false; self.values.len()];
        let mut learned = Vec::new();
        let mut pending = 0;
        let mut cursor = self.trail.len();
        let mut resolved = None;
        loop {
            for lit in conflict {
                if Some(lit.cell) == resolved || seen[lit.cell] || self.levels[lit.cell] == 0 {
                    continue;
                }
                seen[lit.cell] = true;
                self.activity[lit.cell] += self.bump;
                if self.levels[lit.cell] == level {
                    pending += 1;
                } else {
                    learned.push(lit);
                }
            }
            let chosen = loop {
                cursor -= 1;
                let chosen = self.trail[cursor];
                if seen[chosen.cell] {
                    break chosen;
                }
            };
            seen[chosen.cell] = false;
            pending -= 1;
            if pending == 0 {
                learned.insert(0, chosen.opposite());
                break;
            }
            resolved = Some(chosen.cell);
            conflict = self.reasons[chosen.cell]
                .clone()
                .expect("shared implication ancestry");
        }
        if learned.len() > 1 {
            let position = (1..learned.len())
                .max_by_key(|&i| self.levels[learned[i].cell])
                .unwrap();
            learned.swap(1, position);
        }
        let target = learned.get(1).map_or(0, |lit| self.levels[lit.cell]);
        self.bump *= 1.05;
        if self.bump > 1e100 {
            for value in &mut self.activity {
                *value *= 1e-100;
            }
            self.bump *= 1e-100;
        }
        (learned, target)
    }
    fn retreat(&mut self, level: usize) {
        if self.boundaries.len() <= level {
            return;
        }
        let end = self.boundaries[level];
        for lit in self.trail.drain(end..) {
            self.values[lit.cell] = None;
            self.levels[lit.cell] = 0;
            self.reasons[lit.cell] = None;
        }
        self.boundaries.truncate(level);
        self.cursor = self.trail.len();
    }
    pub fn advance(&mut self) -> Step {
        if self.empty {
            return Step::Empty;
        }
        loop {
            if let Err(conflict) = self.propagate() {
                self.conflicts += 1;
                if self.boundaries.is_empty() {
                    self.empty = true;
                    return Step::Empty;
                }
                let (learned, target) = self.analyze(conflict);
                self.retreat(target);
                let forced = learned[0];
                if learned.len() > 1 {
                    let index = self.learned.len();
                    self.watches[learned[0].slot()].push(index);
                    self.watches[learned[1].slot()].push(index);
                    self.learned.push(learned.clone());
                }
                if !self.assign(forced, Some(learned)) {
                    self.empty = true;
                    return Step::Empty;
                }
                continue;
            }
            // Internal gate cells propagate from the factor support. Branch
            // on that support, so a carry guess cannot detach from its factors.
            let next = self
                .p
                .iter()
                .chain(&self.q)
                .copied()
                .filter(|&i| self.values[i].is_none())
                .max_by(|&a, &b| {
                    self.activity[a]
                        .total_cmp(&self.activity[b])
                        .then_with(|| b.cmp(&a))
                });
            if let Some(cell) = next {
                self.boundaries.push(self.trail.len());
                self.decisions += 1;
                self.assign(Literal { cell, mark: ONE }, None);
                return Step::Running;
            }
            let read = |cells: &[usize]| {
                trim(
                    cells
                        .iter()
                        .map(|&cell| self.values[cell].unwrap())
                        .collect(),
                )
            };
            let mut p = read(&self.p);
            let mut q = read(&self.q);
            if mf::cmp(&p, &q) == core::cmp::Ordering::Greater {
                core::mem::swap(&mut p, &mut q);
            }
            return Step::Closed(p, q);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn shared_multiplier_matches_every_small_balanced_product() {
        for n in (9..=255u64).step_by(2) {
            let tape = mf::tape_u64(n);
            let width = tape.len().div_ceil(2);
            let upper = mf::isqrt(&tape);
            let lower = super::super::ceiling(&tape, &upper);
            let expected = (1u64 << (width - 1)..1u64 << width).any(|p| {
                p > 1 && n % p == 0 && (1u64 << (width - 1)..1u64 << width).contains(&(n / p))
            });
            let mut circuit = Correlation::new(&tape, width, &upper, &lower);
            loop {
                match circuit.advance() {
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
