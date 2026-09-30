//! Shared glyph-valued multiplier gates and retained conflict implications.
//! Factor, product and carry cells refer to the same vertices across columns.
use super::{bit, mf, trim, ONE, ZERO};
#[path = "glut_midpoint.rs"]
mod midpoint;
#[path = "glut_parity.rs"]
mod parity;
use alloc::{collections::BTreeMap, vec, vec::Vec};

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
    supports: &'static [[u8; 2]],
    all: u8,
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

// These are the complete truth supports of the elementary gates. A mask
// packs a finite gate table; it places no bound on multiplier or graph extent.
const AND_SUPPORT: &[[u8; 2]] = &[[0b0011, 0b1100], [0b0101, 0b1010], [0b0111, 0b1000]];
const EQUAL_SUPPORT: &[[u8; 2]] = &[[0b01, 0b10], [0b01, 0b10]];
const NOT_SUPPORT: &[[u8; 2]] = &[[0b01, 0b10], [0b10, 0b01]];
const ADD_SUPPORT: &[[u8; 2]] = &[
    [0x0f, 0xf0],
    [0x33, 0xcc],
    [0x55, 0xaa],
    [0x69, 0x96],
    [0x17, 0xe8],
];

pub(super) enum Step {
    Running,
    Closed(Vec<char>, Vec<char>),
    Empty,
}
pub(super) struct Correlation {
    gates: Vec<Gate>,
    parity: Vec<parity::ParityRow>,
    parity_offsets: Vec<usize>,
    parity_incidence: Vec<usize>,
    adjacent_offsets: Vec<usize>,
    adjacent_gates: Vec<usize>,
    values: Vec<Option<char>>,
    levels: Vec<usize>,
    reasons: Vec<Option<Vec<Literal>>>,
    activity: BTreeMap<usize, Vec<char>>,
    priority: BTreeMap<usize, usize>,
    bump: Vec<char>,
    trail: Vec<Literal>,
    boundaries: Vec<usize>,
    cursor: usize,
    learned: Vec<Vec<Literal>>,
    watches: BTreeMap<usize, Vec<usize>>,
    p: Vec<usize>,
    q: Vec<usize>,
    empty: bool,
    conjugate: bool,
    pub decisions: usize,
    pub conflicts: usize,
}
impl Correlation {
    fn blank(empty: bool) -> Self {
        let mut s = Self {
            gates: Vec::new(),
            parity: Vec::new(),
            parity_offsets: Vec::new(),
            parity_incidence: Vec::new(),
            adjacent_offsets: Vec::new(),
            adjacent_gates: Vec::new(),
            values: Vec::new(),
            levels: Vec::new(),
            reasons: Vec::new(),
            activity: BTreeMap::new(),
            bump: vec![ONE],
            priority: BTreeMap::new(),
            trail: Vec::new(),
            boundaries: Vec::new(),
            cursor: 0,
            learned: Vec::new(),
            watches: BTreeMap::new(),
            p: Vec::new(),
            q: Vec::new(),
            empty,
            conjugate: false,
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
        s
    }
    #[cfg(test)]
    pub fn new(n: &[char], width: usize, upper: &[char], lower: &[char]) -> Self {
        let mut s = Self::blank(width < 2);
        let zero = 0;
        let one = 1;
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
        s.connect();
        s
    }
    fn connect(&mut self) {
        for &cell in self.p.iter().chain(&self.q) {
            self.activity.insert(cell, vec![ZERO]);
        }
        for cells in [&self.p, &self.q] {
            for (i, &cell) in cells.iter().enumerate() {
                let inside = i.saturating_sub(1).min(cells.len().saturating_sub(i + 2));
                self.priority
                    .insert(cell, 2 * inside + usize::from(i > cells.len() / 2));
            }
        }
        // A compact incidence fold replaces one heap vector per graph cell.
        self.adjacent_offsets = vec![0; self.values.len() + 1];
        for gate in &self.gates {
            for &cell in &gate.cells {
                self.adjacent_offsets[cell + 1] += 1;
            }
        }
        for i in 1..self.adjacent_offsets.len() {
            self.adjacent_offsets[i] += self.adjacent_offsets[i - 1];
        }
        self.adjacent_gates = vec![0; *self.adjacent_offsets.last().unwrap()];
        let mut next = self.adjacent_offsets[..self.values.len()].to_vec();
        for (index, gate) in self.gates.iter().enumerate() {
            for &cell in &gate.cells {
                self.adjacent_gates[next[cell]] = index;
                next[cell] += 1;
            }
        }
        if option_env!("GLUT_PARITY_FOLD_WORD") == Some("⊥") { self.connect_parity(); }
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
        self.levels.push(0);
        self.reasons.push(None);
        index
    }
    fn gate(&mut self, cells: Vec<usize>, rows: &'static [&'static [char]]) {
        let supports = match rows.len() {
            2 => {
                if rows[0][1] == ZERO {
                    EQUAL_SUPPORT
                } else {
                    NOT_SUPPORT
                }
            }
            4 => AND_SUPPORT,
            8 => ADD_SUPPORT,
            _ => unreachable!("elementary gate table"),
        };
        let all = ((1u16 << rows.len()) - 1) as u8;
        self.gates.push(Gate {
            cells,
            supports,
            all,
        });
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
        let mut viable = gate.all;
        for (slot, &cell) in gate.cells.iter().enumerate() {
            if let Some(mark) = self.values[cell] {
                viable &= gate.supports[slot][usize::from(mark == ONE)];
            }
        }
        let mut forced = Vec::new();
        if viable != 0 {
            for (slot, &cell) in gate.cells.iter().enumerate() {
                if self.values[cell].is_some() {
                    continue;
                }
                let mark = if viable & gate.supports[slot][0] == 0 {
                    Some(ONE)
                } else if viable & gate.supports[slot][1] == 0 {
                    Some(ZERO)
                } else {
                    None
                };
                if let Some(mark) = mark {
                    forced.push(Literal { cell, mark });
                }
            }
            if forced.is_empty() {
                return Ok(());
            }
        }
        if viable == 0 {
            return Err(self.gate_reason(index, None));
        }
        for lit in forced {
            let reason = self.gate_reason(index, Some(lit));
            if !self.assign(lit, Some(reason.clone())) {
                return Err(reason);
            }
        }
        Ok(())
    }
    fn gate_reason(&self, index: usize, conclusion: Option<Literal>) -> Vec<Literal> {
        let gate = &self.gates[index];
        let mut premise = Vec::new();
        for &cell in &gate.cells {
            if cell <= 1 {
                continue;
            }
            if let Some(mark) = self.values[cell] {
                let literal = Literal { cell, mark }.opposite();
                if !premise.contains(&literal) {
                    premise.push(literal);
                }
            }
        }
        let excludes_opposite = |premise: &[Literal]| {
            let mut support = gate.all;
            for (slot, &cell) in gate.cells.iter().enumerate() {
                let condition = if cell <= 1 {
                    self.values[cell]
                } else {
                    premise
                        .iter()
                        .find(|lit| lit.cell == cell)
                        .map(|lit| lit.opposite().mark)
                };
                if let Some(mark) = condition {
                    support &= gate.supports[slot][usize::from(mark == ONE)];
                }
                if let Some(lit) = conclusion {
                    if lit.cell == cell {
                        support &= gate.supports[slot][usize::from(lit.opposite().mark == ONE)];
                    }
                }
            }
            support == 0
        };
        let mut position = 0;
        while position < premise.len() {
            let removed = premise.remove(position);
            if !excludes_opposite(&premise) {
                premise.insert(position, removed);
                position += 1;
            }
        }
        if let Some(lit) = conclusion {
            premise.push(lit);
        }
        premise
    }
    fn propagate(&mut self) -> Result<(), Vec<Literal>> {
        while self.cursor < self.trail.len() {
            let assigned = self.trail[self.cursor];
            self.cursor += 1;
            for slot in
                self.adjacent_offsets[assigned.cell]..self.adjacent_offsets[assigned.cell + 1]
            {
                let gate = self.adjacent_gates[slot];
                self.gate_step(gate)?;
            }
            if !self.parity_offsets.is_empty() {
                for slot in self.parity_offsets[assigned.cell]..self.parity_offsets[assigned.cell+1] {
                    let row = self.parity_incidence[slot];
                    self.parity_step(row)?;
                }
            }
            let false_lit = assigned.opposite();
            let key = false_lit.slot();
            let mut waiting = self.watches.remove(&key).unwrap_or_default();
            while let Some(index) = waiting.pop() {
                if self.learned[index][0] == false_lit {
                    self.learned[index].swap(0, 1);
                }
                let other = self.learned[index][0];
                if self.satisfied(other) == Some(true) {
                    self.watches.entry(key).or_default().push(index);
                    continue;
                }
                let replacement = (2..self.learned[index].len())
                    .find(|&i| self.satisfied(self.learned[index][i]) != Some(false));
                if let Some(position) = replacement {
                    self.learned[index].swap(1, position);
                    self.watches
                        .entry(self.learned[index][1].slot())
                        .or_default()
                        .push(index);
                } else {
                    self.watches.entry(key).or_default().push(index);
                    let reason = self.learned[index].clone();
                    if self.satisfied(other) == Some(false)
                        || !self.assign(other, Some(reason.clone()))
                    {
                        self.watches.entry(key).or_default().extend(waiting);
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
                if let Some(score) = self.activity.get_mut(&lit.cell) {
                    *score = mf::add(score, &self.bump);
                }
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
        // Conflict ranks are dynamic numeral tapes too. Later conflicts add
        // a larger ordinal without floating-point thresholds or rescaling.
        self.bump = mf::add(&self.bump, &[ONE]);
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
                    self.watches
                        .entry(learned[0].slot())
                        .or_default()
                        .push(index);
                    self.watches
                        .entry(learned[1].slot())
                        .or_default()
                        .push(index);
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
                    mf::cmp(&self.activity[&a], &self.activity[&b])
                        .then_with(|| self.priority[&b].cmp(&self.priority[&a]))
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
            if self.conjugate {
                let a = p;
                let b = q;
                p = mf::sub(&a, &b);
                q = mf::add(&a, &b);
            }
            if mf::cmp(&p, &q) == core::cmp::Ordering::Greater {
                core::mem::swap(&mut p, &mut q);
            }
            return Step::Closed(p, q);
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn folded_gate_explanations_are_valid_for_every_partial_assignment() {
        for rows in [AND, NOT, ADD, midpoint::EQUAL] {
            let width = rows[0].len();
            for pattern in 0..3usize.pow(width as u32) {
                let mut graph = Correlation::blank(false);
                let cells: Vec<_> = (0..width).map(|_| graph.cell()).collect();
                graph.gate(cells.clone(), rows);
                let mut pattern = pattern;
                for &cell in &cells {
                    match pattern % 3 {
                        1 => {
                            graph.assign(Literal { cell, mark: ZERO }, None);
                        }
                        2 => {
                            graph.assign(Literal { cell, mark: ONE }, None);
                        }
                        _ => {}
                    }
                    pattern /= 3;
                }
                let mut clauses = Vec::new();
                if let Err(reason) = graph.gate_step(0) {
                    clauses.push(reason);
                }
                for &cell in &cells {
                    if let Some(reason) = &graph.reasons[cell] {
                        clauses.push(reason.clone());
                    }
                }
                for clause in clauses {
                    assert!(rows
                        .iter()
                        .all(|row| clause.iter().any(|lit| row[lit.cell - 2] == lit.mark)));
                    // Every retained premise is required by this local proof.
                    let premises = if clause
                        .last()
                        .is_some_and(|lit| graph.reasons[lit.cell].is_some())
                    {
                        clause.len() - 1
                    } else {
                        clause.len()
                    };
                    for omit in 0..premises {
                        assert!(rows.iter().any(|row| !clause
                            .iter()
                            .enumerate()
                            .any(|(i, lit)| i != omit && row[lit.cell - 2] == lit.mark)));
                    }
                }
            }
        }
    }

    use super::*;
    #[test]
    fn local_support_masks_preserve_every_partial_gate_assignment() {
        for rows in [NOT, midpoint::EQUAL, AND, ADD] {
            let arity = rows[0].len();
            for pattern in 0..3usize.pow(arity as u32) {
                let mut s =
                    Correlation::new(&mf::tape_u64(9), 2, &mf::tape_u64(3), &mf::tape_u64(3));
                let cells: Vec<_> = (0..arity).map(|_| s.cell()).collect();
                let mut digits = pattern;
                for &cell in &cells {
                    let mark = match digits % 3 {
                        0 => None,
                        1 => Some(ZERO),
                        _ => Some(ONE),
                    };
                    digits /= 3;
                    if let Some(mark) = mark {
                        s.assign(Literal { cell, mark }, None);
                    }
                }
                let valid: Vec<_> = rows
                    .iter()
                    .filter(|row| {
                        cells
                            .iter()
                            .enumerate()
                            .all(|(i, &cell)| s.values[cell].is_none_or(|mark| mark == row[i]))
                    })
                    .collect();
                let index = s.gates.len();
                s.gate(cells.clone(), rows);
                let result = s.gate_step(index);
                assert_eq!(
                    result.is_ok(),
                    !valid.is_empty(),
                    "arity={arity} pattern={pattern}"
                );
                for row in valid {
                    assert!(cells
                        .iter()
                        .enumerate()
                        .all(|(i, &cell)| s.values[cell].is_none_or(|mark| mark == row[i])));
                }
            }
        }
    }

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
