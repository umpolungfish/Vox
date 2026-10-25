//! Shared glyph-valued multiplier gates and retained conflict implications.
//! Factor, product and carry cells refer to the same vertices across columns.
use super::{bit, mf, trim, ONE, ZERO};
use crate::belnap::B4;
use crate::belnap_relation::{self as fde, BelnapSet};
#[path = "glut_midpoint.rs"]
mod midpoint;
#[path = "glut_quantum.rs"]
mod quantum;
pub(super) use quantum::factor_phase_closes;
#[path = "glut_parity.rs"]
mod parity;
#[path = "glut_transport.rs"]
mod transport;
#[path = "glut_joint_support.rs"]
mod joint_support;
use alloc::{collections::BTreeMap, string::String, vec, vec::Vec};

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
use super::super::{BelnapConflict,BelnapRelationConflict};
struct Gate {
    cells: Vec<usize>,
    supports: &'static [[u8; 2]],
    all: u8,
    full: &'static fde::BelnapGateTable,
    full_rows: u64,
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

// Numerical projections are compiled from the full FOUR relations.
const AND_PROJECTION:[[u8;2];5]=fde::BelnapGateTable::compile(fde::GateKind::And).numeral_supports();
const NOT_PROJECTION:[[u8;2];5]=fde::BelnapGateTable::compile(fde::GateKind::Not).numeral_supports();
const EQUAL_PROJECTION:[[u8;2];5]=fde::BelnapGateTable::compile(fde::GateKind::Equal).numeral_supports();
const ADD_PROJECTION:[[u8;2];5]=fde::BelnapGateTable::compile(fde::GateKind::Add).numeral_supports();
const AND_SUPPORT:&[[u8;2]]=&AND_PROJECTION;
const NOT_SUPPORT:&[[u8;2]]=&NOT_PROJECTION;
const EQUAL_SUPPORT:&[[u8;2]]=&EQUAL_PROJECTION;
const ADD_SUPPORT:&[[u8;2]]=&ADD_PROJECTION;

pub(super) enum Step {
    Running,
    Closed(Vec<char>, Vec<char>),
    Empty,
}
pub(super) struct Correlation {
    gates: Vec<Gate>,
    products: BTreeMap<(usize, usize), usize>,
    complements: BTreeMap<usize, usize>,
    sums: BTreeMap<[usize; 3], (usize, usize)>,
    parity: Vec<parity::ParityRow>,
    parity_state: Vec<(usize, usize, char)>,
    parity_pairs: alloc::collections::BTreeSet<usize>,
    parity_offsets: Vec<usize>,
    parity_incidence: Vec<usize>,
    adjacent_offsets: Vec<usize>,
    adjacent_gates: Vec<usize>,
    values: Vec<B4>,
    held_conflicts: Vec<BelnapConflict>,
    held_relations: Vec<BelnapRelationConflict>,
    levels: Vec<usize>,
    reasons: Vec<Option<Vec<Literal>>>,
    activity: BTreeMap<usize, Vec<char>>,
    priority: BTreeMap<usize, usize>,
    catalyst_inverse: Option<BTreeMap<usize, usize>>,
    bump: Vec<char>,
    trail: Vec<Literal>,
    boundaries: Vec<usize>,
    cursor: usize,
    learned: Vec<Vec<Literal>>,
    watches: BTreeMap<usize, Vec<usize>>,
    p: Vec<usize>,
    q: Vec<usize>,
    admission: Vec<(usize, usize, usize)>,
    midpoint: Vec<usize>,
    gap: Vec<usize>,
    source: Vec<char>,
    factor_root: Vec<char>,
    cofactor_floor: Vec<char>,
    empty: bool,
    conjugate: bool,
    pub decisions: usize,
    pub conflicts: usize,
    pub work: usize,
    pub fold_calls: usize,
    pub fold_rejected: usize,
    pub fold_forced: usize,
    pub interval_rejected: usize,
    pub carry_rejected: usize,
    pub reason_original: usize,
    pub reason_folded: usize,
    frame_bank: super::frame_elimination::FrameBank,
    support_p: Vec<Option<char>>,
    support_q: Vec<Option<char>>,
    joints: Vec<joint_support::JointSupport>,
    joint_offsets: Vec<usize>,
    joint_incidence: Vec<usize>,
    pub joint_returns: usize,
    pub joint_forced: usize,
}
impl Correlation {
    pub(super) fn held_relations(&self) -> &[BelnapRelationConflict] { &self.held_relations }
    pub(super) fn held_conflicts(&self) -> &[BelnapConflict] { &self.held_conflicts }
    pub(super) fn bind_midpoint_floor(&mut self, floor: &[char]) {
        let midpoint = self.midpoint.clone();
        self.bound(&midpoint, floor, true);
    }
    pub(super) fn bind_factor_pair(&mut self, p:&[char], q:&[char]) -> Result<(),String> {
        for (cells,value) in [(self.p.clone(),p),(self.q.clone(),q)] {
            if value.iter().any(|&mark|mark!=ZERO&&mark!=ONE) {
                return Err("dialectic factor closure contains a non-parity mark".into());
            }
            if value.iter().skip(cells.len()).any(|&mark|mark==ONE) {
                return Err("dialectic factor closure exceeds the resident factor lane".into());
            }
            for (at,&cell) in cells.iter().enumerate() {
                let mark=bit(value,at)==1;
                if !self.assign(Literal{cell,mark:if mark {ONE}else{ZERO}},None) {
                    return Err("dialectic factor closure contradicts the shared arithmetic graph".into());
                }
            }
        }
        self.propagate().map_err(|_|String::from("dialectic factor closure empties the shared arithmetic graph"))
    }
    pub(super) fn factor_support(&self) -> crate::dialectic_reentry::FactorSupport {
        crate::dialectic_reentry::FactorSupport {
            source:self.source.clone(),
            p:self.p.iter().map(|&cell|self.values[cell].parity()).collect(),
            q:self.q.iter().map(|&cell|self.values[cell].parity()).collect(),
            depth:self.boundaries.len(),
        }
    }
    pub(super) fn admission_only(n:&[char])->Self {
        let mut s=Self::blank(n.len()<2);
        let width=n.len().div_ceil(2);
        s.p=(0..width).map(|i|if i==0 {1}else{s.cell()}).collect();
        s.q=(0..width+1).map(|i|if i==0 {1}else{s.cell()}).collect();
        s.bind_admission(n);s.connect_observed(&mut |_,_,_|{});s
    }
    pub(super) fn into_device_relation(self)->super::super::DeviceCorrelation {
        let mut relation=super::super::DeviceCorrelation {
            belnap_prepared_bank:Vec::new(),belnap_preparation:super::super::BelnapPropagation::pending(),
            gates:self.gates.into_iter().map(|gate|super::super::DeviceCorrelationGate {
                full_kind:gate.full.kind,full_supports:gate.full.supports[..gate.full.width].to_vec(),
                full_rows:gate.full_rows,cells:gate.cells,supports:gate.supports.to_vec(),all:gate.all,
            }).collect(),
            belnap_domains:self.values.iter().map(|&value| if value==B4::N {BelnapSet::NUMERAL}else{BelnapSet::singleton(value)}).collect(),
            masks:self.values.into_iter().map(|value|match value {
                B4::F=>1,B4::T=>2,B4::N=>3,B4::B=>0,
            }).collect(),
            p:self.p,q:self.q,empty:self.empty,
            admission:self.admission,
            midpoint:self.midpoint,gap:self.gap,
            parity:self.parity.into_iter().map(|row|(row.cells,row.mark==ONE)).collect(),
        };
        relation.belnap_preparation=relation.propagate_belnap();
        relation
    }
    fn blank(empty: bool) -> Self {
        let mut s = Self {
            gates: Vec::new(),
            products: BTreeMap::new(),
            complements: BTreeMap::new(),
            sums: BTreeMap::new(),
            parity: Vec::new(),
            parity_state: Vec::new(),
            parity_pairs: alloc::collections::BTreeSet::new(),
            parity_offsets: Vec::new(),
            parity_incidence: Vec::new(),
            adjacent_offsets: Vec::new(),
            adjacent_gates: Vec::new(),
            values: Vec::new(),
            held_conflicts: Vec::new(),
            held_relations: Vec::new(),
            levels: Vec::new(),
            reasons: Vec::new(),
            activity: BTreeMap::new(),
            bump: vec![ONE],
            priority: BTreeMap::new(),
            catalyst_inverse: None,
            trail: Vec::new(),
            boundaries: Vec::new(),
            cursor: 0,
            learned: Vec::new(),
            watches: BTreeMap::new(),
            p: Vec::new(),
            q: Vec::new(),
            admission: Vec::new(),
            midpoint: Vec::new(),
            gap: Vec::new(),
            source: Vec::new(),
            factor_root: Vec::new(),
            cofactor_floor: Vec::new(),
            empty,
            conjugate: false,
            decisions: 0,
            conflicts: 0,
            work: 0,
            fold_calls: 0,
            fold_rejected: 0,
            fold_forced: 0,
            interval_rejected: 0,
            carry_rejected: 0,
            reason_original: 0,
            reason_folded: 0,
            frame_bank: super::frame_elimination::FrameBank::default(),
            support_p: Vec::new(),
            support_q: Vec::new(),
            joints: Vec::new(),
            joint_offsets: Vec::new(),
            joint_incidence: Vec::new(),
            joint_returns: 0,
            joint_forced: 0,
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
        s.connect_observed(&mut |_, _, _| {});
        s
    }
    #[cfg(test)]
    fn connect(&mut self) {
        self.connect_observed(&mut |_, _, _| {});
    }
    fn connect_observed(&mut self, observe: &mut impl FnMut(char, usize, usize)) {
        observe('≺', self.gate_count(), self.cell_count());
        self.fold_transport();
        observe('≻', self.gate_count(), self.cell_count());
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
        // Keep the DQI syndrome connected throughout descent and ascent.
        self.connect_joint_support();
        // Each assignment updates only incident row residues; root substitution
        // remains valid because source-root assignments never retreat.
        observe('⊤', self.gate_count(), self.cell_count());
        if self.propagate().is_err() {
            self.empty = true;
        } else {
            observe('⊥', self.gate_count(), self.cell_count());
            self.connect_parity_observed(observe);
            observe('⋈', self.gate_count(), self.cell_count());
            if self.propagate().is_err() {
                self.empty = true;
            }
        }
    }

    /// Apply MoDoT's twelve-axis order to the actual factor cells. The
    /// permutation changes the resident's next-cell selection while leaving
    /// every product, carry, parity, and support gate attached to its cell.
    pub(super) fn apply_catalyst_order(&mut self, axes: &[usize; 12]) -> Result<(), String> {
        if self.catalyst_inverse.is_some() {
            return Err("catalyst order is already active on this correlation".into());
        }
        let mut seen = [false; 12];
        for &axis in axes {
            if axis >= 12 || seen[axis] {
                return Err("MoDoT catalyst order is not a permutation of twelve axes".into());
            }
            seen[axis] = true;
        }
        let expand = |width: usize| {
            let mut positions = Vec::with_capacity(width);
            for block in 0..width.div_ceil(12) {
                for &axis in axes {
                    let position = block * 12 + axis;
                    if position < width { positions.push(position); }
                }
            }
            positions
        };
        let p_order = expand(self.p.len());
        let q_order = expand(self.q.len());
        if p_order.len() != self.p.len() || q_order.len() != self.q.len() {
            return Err("MoDoT support order did not cover every resident factor cell".into());
        }
        self.catalyst_inverse = Some(self.priority.clone());
        for (rank, position) in p_order.into_iter().enumerate() {
            self.priority.insert(self.p[position], rank * 2);
        }
        for (rank, position) in q_order.into_iter().enumerate() {
            self.priority.insert(self.q[position], rank * 2 + 1);
        }
        Ok(())
    }

    /// Return the original resident decision order after the active
    /// computation, consuming the exact inverse retained by AFWD.
    pub(super) fn return_catalyst_order(&mut self) -> Result<(), String> {
        let original = self.catalyst_inverse.take()
            .ok_or("catalyst inverse has no retained forward order")?;
        if original.len() != self.priority.len() {
            return Err("catalyst inverse lost resident decision coordinates".into());
        }
        self.priority = original;
        Ok(())
    }
    pub fn gate_count(&self) -> usize {
        self.gates.len()
    }
    pub fn cell_count(&self) -> usize {
        self.values.len()
    }
    fn cell(&mut self) -> usize {
        let index = self.values.len();
        self.values.push(B4::N);
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
        let full = match rows.len() {
            2 if rows[0][1] == ZERO => &fde::EQUAL,
            2 => &fde::NOT,
            4 => &fde::AND,
            8 => &fde::ADD,
            _ => unreachable!("elementary gate relation"),
        };
        let full_rows=full.correlated_rows(&cells);
        self.gates.push(Gate { cells, supports:&supports[..full.width], all, full, full_rows });
    }
    fn and(&mut self, a: usize, b: usize) -> usize {
        if a == 0 || b == 0 {
            0
        } else if a == 1 {
            b
        } else if b == 1 || a == b {
            a
        } else {
            let key = (a.min(b), a.max(b));
            if let Some(&out) = self.products.get(&key) {
                return out;
            }
            let out = self.cell();
            self.gate(vec![a, b, out], AND);
            self.products.insert(key, out);
            out
        }
    }
    fn not(&mut self, a: usize) -> usize {
        if a == 0 {
            1
        } else if a == 1 {
            0
        } else {
            if let Some(&out) = self.complements.get(&a) {
                return out;
            }
            let out = self.cell();
            self.gate(vec![a, out], NOT);
            self.complements.insert(a, out);
            self.complements.insert(out, a);
            out
        }
    }
    /// Width alternatives are predicates on these same factor cells. The
    /// disjunction holds the complete source-derived family in one register.
    pub(super) fn bind_admission(&mut self,n:&[char]) {
        let widths=|s:&mut Self,cells:&[usize]| {
            let mut labels=vec![0;cells.len()+1];let mut higher_zero=1;
            for i in (0..cells.len()).rev() {
                labels[i+1]=s.and(higher_zero,cells[i]);
                let negated=s.not(cells[i]);higher_zero=s.and(higher_zero,negated);
            }
            labels
        };
        let p=self.p.clone();let q=self.q.clone();
        let left=widths(self,&p);let right=widths(self,&q);
        let mut none=1;
        for (pw,qw) in super::super::source_factor_geometries(n.len()) {
            let selector=self.and(left.get(pw).copied().unwrap_or(0),right.get(qw).copied().unwrap_or(0));
            self.admission.push((pw,qw,selector));
            let excluded=self.not(selector);none=self.and(none,excluded);
        }
        self.equal_cells(none,0);
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
        let mut key = [a, b, c];
        key.sort_unstable();
        if let Some(&pair) = self.sums.get(&key) {
            return pair;
        }
        let sum = self.cell();
        let carry = self.cell();
        self.gate(vec![a, b, c, sum, carry], ADD);
        self.sums.insert(key, (sum, carry));
        (sum, carry)
    }
    fn bound(&mut self, cells: &[usize], value: &[char], lower: bool) {
        // A bound wider than the register cannot be truncated modulo its
        // width: every represented value is below it. A lower bound empties
        // that frame; an upper bound already holds throughout the frame.
        if trim(value.to_vec()).len() > cells.len() {
            self.empty |= lower;
            return;
        }
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
        let previous = self.values[lit.cell];
        let incoming = B4::from_parity(lit.mark);
        let joined = previous.join(incoming);
        if joined == B4::B {
            // Keep both observations with their grounding. The numerical
            // continuation stays scoped to its existing decision branch;
            // the contradictory return is never converted into that branch.
            self.held_conflicts.push(BelnapConflict {
                cell: lit.cell, evidence: joined, previous, incoming,
                depth: self.boundaries.len(),
                previous_reason: self.reasons[lit.cell].as_ref().map(|r| r.iter().map(|lit| (lit.cell, B4::from_parity(lit.mark))).collect()),
                incoming_reason: reason.as_ref().map(|r| r.iter().map(|lit| (lit.cell, B4::from_parity(lit.mark))).collect()),
            });
            return false;
        }
        if previous != B4::N { return true; }
        self.values[lit.cell] = B4::from_parity(lit.mark);
        self.parity_assignment(lit, false);
        self.levels[lit.cell] = self.boundaries.len();
        self.reasons[lit.cell] = reason;
        self.trail.push(lit);
        true
    }
    fn satisfied(&self, lit: Literal) -> Option<bool> {
        self.values[lit.cell].parity().map(|mark| mark == lit.mark)
    }
    fn gate_step(&mut self, index: usize) -> Result<(), Vec<Literal>> {
        let gate = &self.gates[index];
        self.work += gate.cells.len() * 2;
        // Numeral candidates are an explicit subset of FOUR. Query the full
        // relation and retain common cell identities before selecting a mark.
        let domains:Vec<_>=gate.cells.iter().map(|&cell| match self.values[cell] {
            B4::N=>BelnapSet::NUMERAL,
            value=>BelnapSet::singleton(value),
        }).collect();
        let viable=gate.full.restrict(gate.full_rows,&domains);
        if viable==0 {return Err(self.gate_reason(index,None));}
        let forced:Vec<_>=gate.cells.iter().enumerate().filter_map(|(slot,&cell)| {
            if self.values[cell] != B4::N {return None;}
            match gate.full.domain(viable,slot).only() {
                Some(B4::T)=>Some(Literal{cell,mark:ONE}),
                Some(B4::F)=>Some(Literal{cell,mark:ZERO}),
                _=>None,
            }
        }).collect();
        for lit in forced {
            let reason = self.gate_reason(index, Some(lit));
            if !self.assign(lit, Some(reason.clone())) {
                return Err(reason);
            }
        }
        Ok(())
    }
    fn gate_reason(&mut self, index: usize, conclusion: Option<Literal>) -> Vec<Literal> {
        let gate = &self.gates[index];
        let mut premise = Vec::new();
        for &cell in &gate.cells {
            if cell <= 1 {
                continue;
            }
            if let Some(mark) = self.values[cell].parity() {
                let literal = Literal { cell, mark }.opposite();
                if !premise.contains(&literal) {
                    premise.push(literal);
                }
            }
        }
        let mut examined = 0;
        let mut excludes_opposite = |premise: &[Literal]| {
            examined += gate.cells.len() * (premise.len() + 1);
            let mut support = gate.all;
            for (slot, &cell) in gate.cells.iter().enumerate() {
                let condition = if cell <= 1 {
                    self.values[cell].parity()
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
        self.work += examined;
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
            if !self.joint_offsets.is_empty() {
                for slot in self.joint_offsets[assigned.cell]..self.joint_offsets[assigned.cell + 1] {
                    self.joint_step(self.joint_incidence[slot])?;
                }
            }
            if !self.parity_offsets.is_empty() {
                for slot in
                    self.parity_offsets[assigned.cell]..self.parity_offsets[assigned.cell + 1]
                {
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
        while self.trail.len() > end {
            let lit = self.trail.pop().unwrap();
            self.parity_assignment(lit, true);
            self.values[lit.cell] = B4::N;
            self.levels[lit.cell] = 0;
            self.reasons[lit.cell] = None;
        }
        self.boundaries.truncate(level);
        self.cursor = self.trail.len();
    }
    fn interval_rejects(&self, premise: &[Literal], work: &mut usize) -> bool {
        let mut included: Vec<_> = premise.iter().map(|lit| lit.cell).collect();
        included.sort_unstable();
        let read = |cells: &[usize]| {
            cells
                .iter()
                .map(|&cell| {
                    if self.levels[cell] == 0 || included.binary_search(&cell).is_ok() {
                        self.values[cell].parity()
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
        };
        let p = read(&self.p);
        let q = read(&self.q);
        *work += (p.len() + q.len()) * (included.len().max(1).ilog2() as usize + 1);
        let Some((pmin, pmax)) = super::extrema(&p, &[ONE, ONE], &self.factor_root) else {
            return true;
        };
        let Some((qmin, qmax)) = super::extrema(&q, &self.cofactor_floor, &super::bounds(&q).1)
        else {
            return true;
        };
        *work += pmin.len() * qmin.len() + pmax.len() * qmax.len();
        mf::cmp(&mf::mul(&pmin, &qmin), &self.source) == core::cmp::Ordering::Greater
            || mf::cmp(&mf::mul(&pmax, &qmax), &self.source) == core::cmp::Ordering::Less
            || mf::cmp(&pmin, &qmax) == core::cmp::Ordering::Greater
    }
    fn support_rejects(&self, premise: &[Literal], work: &mut usize) -> bool {
        let mut included: Vec<_> = premise.iter().map(|lit| lit.cell).collect();
        included.sort_unstable();
        let read = |cells: &[usize]| {
            cells
                .iter()
                .map(|&cell| {
                    if self.levels[cell] == 0 || included.binary_search(&cell).is_ok() {
                        self.values[cell].parity()
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
        };
        let p = read(&self.p);
        let q = read(&self.q);
        let mut frame = super::Fold {
            p_range: (vec![ONE, ONE], self.factor_root.clone()),
            q_range: (self.cofactor_floor.clone(), super::bounds(&q).1),
            p,
            q,
        };
        frame.propagate(&self.source, work).is_none()
    }
    fn fold_conflict_reason(&mut self, mut premise: Vec<Literal>) -> Vec<Literal> {
        self.reason_original += premise.len();
        let mut work = 0;
        let interval = self.interval_rejects(&premise, &mut work);
        // A retained conflict excludes every extension of its premises.
        // Let the native support return prove the larger family directly,
        // including source-bound prime rails, before shedding assumptions.
        let support = !interval && self.support_rejects(&premise, &mut work);
        if interval || support {
            if interval {
                self.interval_rejected += 1;
            } else {
                self.carry_rejected += 1;
            }
            // Fold whole assumption blocks first, then their smaller pieces.
            // Every deletion requires a fresh native interval/support return
            // proving that all extensions remain outside the source product.
            let mut width = premise.len();
            while width != 0 {
                let mut position = 0;
                while position < premise.len() {
                    let end = (position + width).min(premise.len());
                    let mut candidate = premise.clone();
                    candidate.drain(position..end);
                    let rejected = if interval {
                        self.interval_rejects(&candidate, &mut work)
                    } else {
                        self.support_rejects(&candidate, &mut work)
                    };
                    if rejected {
                        premise = candidate;
                    } else {
                        position = end;
                    }
                }
                width /= 2;
            }
        }
        self.work += work;
        self.reason_folded += premise.len();
        premise
    }
    fn fold_support(&mut self) -> Result<(), Vec<Literal>> {
        if self.source.is_empty() {
            return Ok(());
        }
        let p: Vec<_> = self.p.iter().map(|&cell| self.values[cell].parity()).collect();
        let q: Vec<_> = self.q.iter().map(|&cell| self.values[cell].parity()).collect();
        if self.support_p == p && self.support_q == q {
            return Ok(());
        }
        self.fold_calls += 1;
        let mut premise = Vec::new();
        for &cell in self.p.iter().chain(&self.q) {
            if self.levels[cell] == 0 {
                continue;
            }
            if let Some(mark) = self.values[cell].parity() {
                let lit = Literal { cell, mark }.opposite();
                if !premise.contains(&lit) {
                    premise.push(lit);
                }
            }
        }
        let mut frame = super::Fold {
            p_range: (vec![ONE, ONE], self.factor_root.clone()),
            q_range: (self.cofactor_floor.clone(), super::bounds(&q).1),
            p,
            q,
        };
        let mut work = 0;
        let supported = frame.propagate(&self.source, &mut work).is_some()
            && super::frame_elimination::eliminate(
                &self.source,
                &mut frame.p,
                &mut frame.q,
                self.fold_calls - 1,
                &mut self.frame_bank,
            )
            .is_some();
        self.work += work;
        if !supported {
            self.fold_rejected += 1;
            return Err(self.fold_conflict_reason(premise));
        }
        let forced: Vec<_> = self
            .p
            .iter()
            .zip(&frame.p)
            .chain(self.q.iter().zip(&frame.q))
            .filter_map(|(&cell, &mark)| {
                mark.filter(|_| self.values[cell].parity().is_none())
                    .map(|mark| Literal { cell, mark })
            })
            .collect();
        for lit in forced {
            if self.values[lit.cell].parity().is_none() {
                self.fold_forced += 1;
            }
            let mut reason = premise.clone();
            reason.push(lit);
            if !self.assign(lit, Some(reason.clone())) {
                return Err(reason);
            }
        }
        self.support_p = frame.p;
        self.support_q = frame.q;
        Ok(())
    }
    pub fn advance(&mut self) -> Step {
        if self.empty {
            return Step::Empty;
        }
        loop {
            let settled = loop {
                if let Err(conflict) = self.propagate() {
                    break Err(conflict);
                }
                if let Err(conflict) = self.parity_holonomy() {
                    break Err(conflict);
                }
                let before = self.trail.len();
                if let Err(conflict) = self.fold_support() {
                    break Err(conflict);
                }
                if self.trail.len() == before {
                    break Ok(());
                }
            };
            if let Err(conflict) = settled {
                self.held_relations.push(BelnapRelationConflict {
                    evidence:B4::B, depth:self.boundaries.len(), decision:self.decisions,
                    support:conflict.iter().map(|lit|(lit.cell,self.values[lit.cell])).collect(),
                    implication:conflict.iter().map(|lit|(lit.cell,B4::from_parity(lit.mark))).collect(),
                });
                self.conflicts += 1;
                let conflict_level = conflict
                    .iter()
                    .map(|lit| self.levels[lit.cell])
                    .max()
                    .unwrap_or(0);
                if conflict_level == 0 {
                    self.empty = true;
                    return Step::Empty;
                }
                if conflict_level < self.boundaries.len() {
                    self.retreat(conflict_level);
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
                .filter(|&i| self.values[i].parity().is_none())
                .max_by(|&a, &b| {
                    mf::cmp(&self.activity[&a], &self.activity[&b])
                        .then_with(|| self.priority[&b].cmp(&self.priority[&a]))
                        .then_with(|| b.cmp(&a))
                });
            if let Some(cell) = next {
                self.boundaries.push(self.trail.len());
                self.decisions += 1;
                // Square frames descend toward the source-root midpoint and
                // small difference support; product frames retain their
                // factor-oriented polarity. Conflicts can reverse either bit.
                let mark = if self.conjugate { ZERO } else { ONE };
                self.assign(Literal { cell, mark }, None);
                return Step::Running;
            }
            let read = |cells: &[usize]| {
                trim(
                    cells
                        .iter()
                        .map(|&cell| self.values[cell].parity().unwrap())
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
            let mut prime_conflict = false;
            for &p_s in &crate::geometer_factor_extractor::SMALL_PRIME_ORBIT_PRIMES {
                let r_n = crate::geometer_factor_extractor::tape_rem_u32(&self.source, p_s);
                if r_n != 0 {
                    let r_p = crate::geometer_factor_extractor::tape_rem_u32(&p, p_s);
                    let r_q = crate::geometer_factor_extractor::tape_rem_u32(&q, p_s);
                    if r_p == 0 || r_q == 0 || (r_p * r_q) % p_s != r_n {
                        prime_conflict = true;
                        break;
                    }
                }
            }
            if prime_conflict {
                let mut premise = Vec::new();
                for &cell in self.p.iter().chain(&self.q) {
                    if self.levels[cell] > 0 {
                        if let Some(mark) = self.values[cell].parity() {
                            let lit = Literal { cell, mark }.opposite();
                            if !premise.contains(&lit) {
                                premise.push(lit);
                            }
                        }
                    }
                }
                self.conflicts += 1;
                let conflict_level = premise.iter().map(|lit| self.levels[lit.cell]).max().unwrap_or(0);
                if conflict_level == 0 {
                    self.empty = true;
                    return Step::Empty;
                }
                self.retreat(conflict_level);
                let (learned, target) = self.analyze(premise);
                self.retreat(target);
                let forced = learned[0];
                if learned.len() > 1 {
                    let index = self.learned.len();
                    self.watches.entry(learned[0].slot()).or_default().push(index);
                    self.watches.entry(learned[1].slot()).or_default().push(index);
                    self.learned.push(learned.clone());
                }
                if !self.assign(forced, Some(learned)) {
                    self.empty = true;
                    return Step::Empty;
                }
                continue;
            }
            return Step::Closed(p, q);
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn belnap_conflict_retains_both_observations_and_branch_recovery() {
        use super::*;
        let mut graph=Correlation::blank(false);
        let cell=graph.cell();
        graph.boundaries.push(graph.trail.len());
        assert!(graph.assign(Literal{cell,mark:ONE},None));
        let reason=vec![Literal{cell,mark:ZERO}];
        assert!(!graph.assign(Literal{cell,mark:ZERO},Some(reason)));
        let held=&graph.held_conflicts()[0];
        assert_eq!(held.evidence,B4::B);
        assert_eq!((held.previous,held.incoming),(B4::T,B4::F));
        assert_eq!(held.depth,1);
        assert_eq!(held.incoming_reason,Some(vec![(cell,B4::F)]));
        graph.retreat(0);
        assert_eq!(graph.values[cell],B4::N);
        assert_eq!(graph.held_conflicts()[0].evidence,B4::B);
        assert!(graph.assign(Literal{cell,mark:ZERO},None));
        assert_eq!(graph.values[cell],B4::F);
        assert_eq!(graph.held_conflicts()[0].evidence,B4::B);
    }
    #[test]
    #[ignore = "explicit RSA-200 internal-relation measurement"]
    fn rsa_200_source_only_product_relation() {
        extern crate std;
        let source = crate::godel_calculus::Nat::from_decimal(
            "1199215083673205158356661431646321119743288893586546308999201",
        ).unwrap();
        let word = crate::godel_calculus::encode_cell_binary(&source);
        let tape = mf::parse_numeral(&word).unwrap();
        assert_eq!(tape.len(), 200);
        let width = tape.len() / 2;
        let upper = vec![ONE; width];
        let mut lower = vec![ZERO; width];
        lower[width - 1] = ONE;
        let started = std::time::Instant::now();
        std::println!("producer=shared-product-cell-relation source={} bits={} factor-width={} source-only=true", source, tape.len(), width);
        let mut graph = Correlation::new(&tape, width, &upper, &lower);
        std::println!("prepared elapsed={:.3} cells={} gates={}", started.elapsed().as_secs_f64(), graph.cell_count(), graph.gate_count());
        let mut last = started.elapsed();
        loop {
            match graph.advance() {
                Step::Running => {
                    let elapsed = started.elapsed();
                    if elapsed.saturating_sub(last).as_secs() >= 5 {
                        std::println!("running elapsed={:.3} decisions={} conflicts={} work={} assigned={} learned={}", elapsed.as_secs_f64(), graph.decisions, graph.conflicts, graph.work, graph.trail.len(), graph.learned.len());
                        last = elapsed;
                    }
                }
                Step::Empty => panic!("the RSA-200 source relation returned empty"),
                Step::Closed(p, q) => {
                    let p_word = mf::emit_numeral(&p);
                    let q_word = mf::emit_numeral(&q);
                    assert!(crate::godel_calculus::check(&p_word, crate::godel_calculus::Operator::Mul, &q_word, &word).unwrap().valid);
                    std::println!("closed elapsed={:.3} p={} q={} decisions={} conflicts={}\np.word={}\nq.word={}", started.elapsed().as_secs_f64(), mf::dec_of(&p), mf::dec_of(&q), graph.decisions, graph.conflicts, p_word, q_word);
                    return;
                }
            }
        }
    }

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
                            .all(|(i, &cell)| s.values[cell].parity().is_none_or(|mark| mark == row[i]))
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
                        .all(|(i, &cell)| s.values[cell].parity().is_none_or(|mark| mark == row[i])));
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
