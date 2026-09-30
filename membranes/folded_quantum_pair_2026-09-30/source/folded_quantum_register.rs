//! Exact real-amplitude coherent register over shared decision branches.
//! H is represented without its common sqrt(2) scale. Born ratios divide by
//! the actual norm, so that common scale cancels. No basis table is allocated.
use crate::fixed_point_quantum_phase::factor_oracle::{Control, GateSink};
use crate::morphism_factor::{add, cmp, divmod, mul, one, sub, trim, zero};
use crate::vox::{EVALF, EVALT};
use alloc::{collections::BTreeMap, vec::Vec};
use core::cmp::Ordering;
type Tape = Vec<char>;
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Amplitude {
    sign: char,
    magnitude: Tape,
}
impl Amplitude {
    fn new(sign: char, magnitude: Tape) -> Self {
        let magnitude = trim(magnitude);
        Self {
            sign: if zero(&magnitude) { EVALT } else { sign },
            magnitude,
        }
    }
    fn neg(&self) -> Self {
        Self::new(
            if self.sign == EVALT { EVALF } else { EVALT },
            self.magnitude.clone(),
        )
    }
    fn plus(&self, b: &Self) -> Self {
        if self.sign == b.sign {
            Self::new(self.sign, add(&self.magnitude, &b.magnitude))
        } else {
            match cmp(&self.magnitude, &b.magnitude) {
                Ordering::Less => Self::new(b.sign, sub(&b.magnitude, &self.magnitude)),
                _ => Self::new(self.sign, sub(&self.magnitude, &b.magnitude)),
            }
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Node {
    Leaf(Amplitude),
    Branch {
        wire: usize,
        low: usize,
        high: usize,
    },
}
struct Arena {
    nodes: Vec<Node>,
    unique: BTreeMap<Node, usize>,
}
impl Arena {
    fn new() -> Self {
        Self {
            nodes: Vec::new(),
            unique: BTreeMap::new(),
        }
    }
    fn intern(&mut self, node: Node) -> usize {
        if let Some(&id) = self.unique.get(&node) {
            return id;
        }
        let id = self.nodes.len();
        self.nodes.push(node.clone());
        self.unique.insert(node, id);
        id
    }
    fn branch(&mut self, wire: usize, low: usize, high: usize) -> usize {
        if low == high {
            low
        } else {
            self.intern(Node::Branch { wire, low, high })
        }
    }
    fn top(&self, id: usize) -> Option<usize> {
        match self.nodes[id] {
            Node::Branch { wire, .. } => Some(wire),
            _ => None,
        }
    }
    fn split(&self, id: usize, at: usize) -> (usize, usize) {
        match self.nodes[id] {
            Node::Branch { wire, low, high } if wire == at => (low, high),
            _ => (id, id),
        }
    }
    fn negate(&mut self, id: usize, memo: &mut BTreeMap<usize, usize>) -> usize {
        if let Some(&r) = memo.get(&id) {
            return r;
        }
        let r = match self.nodes[id].clone() {
            Node::Leaf(a) => self.intern(Node::Leaf(a.neg())),
            Node::Branch { wire, low, high } => {
                let low = self.negate(low, memo);
                let high = self.negate(high, memo);
                self.branch(wire, low, high)
            }
        };
        memo.insert(id, r);
        r
    }
    fn sum(&mut self, a: usize, b: usize, memo: &mut BTreeMap<(usize, usize), usize>) -> usize {
        if let Some(&r) = memo.get(&(a, b)) {
            return r;
        }
        let r = match (self.top(a), self.top(b)) {
            (None, None) => {
                let (Node::Leaf(x), Node::Leaf(y)) = (&self.nodes[a], &self.nodes[b]) else {
                    unreachable!()
                };
                let z = x.plus(y);
                self.intern(Node::Leaf(z))
            }
            (x, y) => {
                let wire = x.into_iter().chain(y).min().unwrap();
                let (al, ah) = self.split(a, wire);
                let (bl, bh) = self.split(b, wire);
                let low = self.sum(al, bl, memo);
                let high = self.sum(ah, bh, memo);
                self.branch(wire, low, high)
            }
        };
        memo.insert((a, b), r);
        r
    }
    fn flip(&mut self, id: usize, target: usize, memo: &mut BTreeMap<usize, usize>) -> usize {
        if let Some(&r) = memo.get(&id) {
            return r;
        }
        let r = match self.nodes[id].clone() {
            Node::Branch { wire, low, high } if wire == target => self.branch(wire, high, low),
            Node::Branch { wire, low, high } if wire < target => {
                let low = self.flip(low, target, memo);
                let high = self.flip(high, target, memo);
                self.branch(wire, low, high)
            }
            _ => id,
        };
        memo.insert(id, r);
        r
    }
    fn conditional(
        &mut self,
        a: usize,
        b: usize,
        controls: &[Control],
        at: usize,
        memo: &mut BTreeMap<(usize, usize, usize), usize>,
    ) -> usize {
        if at == controls.len() {
            return b;
        }
        if a == b {
            return a;
        }
        if let Some(&r) = memo.get(&(a, b, at)) {
            return r;
        }
        let c = controls[at];
        let wire = self
            .top(a)
            .into_iter()
            .chain(self.top(b))
            .chain(core::iter::once(c.cell))
            .min()
            .unwrap();
        let (al, ah) = self.split(a, wire);
        let (bl, bh) = self.split(b, wire);
        let (low, high) = if wire == c.cell {
            if c.value == EVALT {
                (self.conditional(al, bl, controls, at + 1, memo), ah)
            } else {
                (al, self.conditional(ah, bh, controls, at + 1, memo))
            }
        } else {
            (
                self.conditional(al, bl, controls, at, memo),
                self.conditional(ah, bh, controls, at, memo),
            )
        };
        let r = self.branch(wire, low, high);
        memo.insert((a, b, at), r);
        r
    }
    fn hadamard(&mut self, id: usize, target: usize, memo: &mut BTreeMap<usize, usize>) -> usize {
        if let Some(&r) = memo.get(&id) {
            return r;
        }
        let r = if let Node::Branch { wire, low, high } = self.nodes[id].clone() {
            if wire < target {
                let low = self.hadamard(low, target, memo);
                let high = self.hadamard(high, target, memo);
                self.branch(wire, low, high)
            } else {
                self.hadamard_boundary(id, target)
            }
        } else {
            self.hadamard_boundary(id, target)
        };
        memo.insert(id, r);
        r
    }
    fn hadamard_boundary(&mut self, id: usize, target: usize) -> usize {
        let (low, high) = self.split(id, target);
        let plus = self.sum(low, high, &mut BTreeMap::new());
        let neg = self.negate(high, &mut BTreeMap::new());
        let minus = self.sum(low, neg, &mut BTreeMap::new());
        self.branch(target, plus, minus)
    }
    fn copy(&self, id: usize, out: &mut Arena, memo: &mut BTreeMap<usize, usize>) -> usize {
        if let Some(&r) = memo.get(&id) {
            return r;
        }
        let r = match &self.nodes[id] {
            Node::Leaf(a) => out.intern(Node::Leaf(a.clone())),
            &Node::Branch { wire, low, high } => {
                let low = self.copy(low, out, memo);
                let high = self.copy(high, out, memo);
                out.branch(wire, low, high)
            }
        };
        memo.insert(id, r);
        r
    }
    fn mass(
        &self,
        id: usize,
        depth: usize,
        cells: usize,
        memo: &mut BTreeMap<(usize, usize), Tape>,
    ) -> Tape {
        if let Some(r) = memo.get(&(id, depth)) {
            return r.clone();
        }
        let (value, shift) = match &self.nodes[id] {
            Node::Leaf(a) => (mul(&a.magnitude, &a.magnitude), cells - depth),
            &Node::Branch { wire, low, high } => {
                let lo = self.mass(low, wire + 1, cells, memo);
                let hi = self.mass(high, wire + 1, cells, memo);
                (add(&lo, &hi), wire - depth)
            }
        };
        let r = if zero(&value) {
            value
        } else {
            core::iter::repeat_n(EVALT, shift).chain(value).collect()
        };
        memo.insert((id, depth), r.clone());
        r
    }
}
/// Shared branches fold whenever their amplitudes agree. Workspace checkpoints
/// discard unreachable construction nodes. Source width chooses no state cap.
pub struct FoldedRegister {
    arena: Arena,
    root: usize,
    cells: usize,
}
impl FoldedRegister {
    pub fn zero(cells: usize) -> Self {
        let mut arena = Arena::new();
        let z = arena.intern(Node::Leaf(Amplitude::new(EVALT, vec![EVALT])));
        let mut root = arena.intern(Node::Leaf(Amplitude::new(EVALT, one())));
        for wire in (0..cells).rev() {
            root = arena.branch(wire, root, z);
        }
        Self { arena, root, cells }
    }
    pub fn retained_nodes(&self) -> usize {
        self.arena.nodes.len()
    }
    pub fn fold(&mut self) {
        let mut arena = Arena::new();
        self.root = self.arena.copy(self.root, &mut arena, &mut BTreeMap::new());
        self.arena = arena;
    }
    pub fn hadamard(&mut self, wire: usize) -> Result<(), &'static str> {
        if wire >= self.cells {
            return Err("Hadamard wire outside register");
        }
        self.root = self.arena.hadamard(self.root, wire, &mut BTreeMap::new());
        Ok(())
    }
    fn controls(&self, controls: &[Control]) -> Result<Vec<Control>, &'static str> {
        if controls
            .iter()
            .any(|c| c.cell >= self.cells || (c.value != EVALT && c.value != EVALF))
        {
            return Err("malformed coherent gate control");
        }
        let mut out = controls.to_vec();
        out.sort_by_key(|c| c.cell);
        if out.windows(2).any(|w| w[0].cell == w[1].cell) {
            return Err("duplicate coherent gate control");
        }
        Ok(out)
    }
    /// An exact Born inverse-CDF measurement at a supplied IMASM fraction.
    /// The caller supplies the quantile; this CPU instrument supplies no claim
    /// of device entropy. The complete register collapses to the selected basis.
    pub fn measure(
        &mut self,
        numerator: &[char],
        denominator: &[char],
    ) -> Result<Tape, &'static str> {
        if [numerator, denominator]
            .iter()
            .any(|t| t.is_empty() || t.iter().any(|&c| c != EVALT && c != EVALF))
            || zero(denominator)
            || cmp(numerator, denominator) != Ordering::Less
        {
            return Err("invalid Born quantile");
        }
        let mut memo = BTreeMap::new();
        let mass = self.arena.mass(self.root, 0, self.cells, &mut memo);
        if zero(&mass) {
            return Err("coherent register has zero norm");
        }
        let (mut rank, _) = divmod(&mul(&mass, numerator), denominator);
        let mut id = self.root;
        let mut result = Vec::new();
        for wire in 0..self.cells {
            let (low, high) = self.arena.split(id, wire);
            let low_mass = self.arena.mass(low, wire + 1, self.cells, &mut memo);
            if cmp(&rank, &low_mass) == Ordering::Less {
                result.push(EVALT);
                id = low;
            } else {
                rank = sub(&rank, &low_mass);
                result.push(EVALF);
                id = high;
            }
        }
        *self = Self::zero(self.cells);
        for (wire, &bit) in result.iter().enumerate() {
            if bit == EVALF {
                self.toggle(&[], wire)?;
            }
        }
        self.fold();
        Ok(result)
    }
}
impl GateSink for FoldedRegister {
    fn toggle(&mut self, controls: &[Control], target: usize) -> Result<(), &'static str> {
        if target >= self.cells || controls.iter().any(|c| c.cell == target) {
            return Err("invalid coherent bit-flip target");
        }
        let controls = self.controls(controls)?;
        let flipped = self.arena.flip(self.root, target, &mut BTreeMap::new());
        self.root = self
            .arena
            .conditional(self.root, flipped, &controls, 0, &mut BTreeMap::new());
        Ok(())
    }
    fn phase_flip(&mut self, controls: &[Control]) -> Result<(), &'static str> {
        let controls = self.controls(controls)?;
        let reversed = self.arena.negate(self.root, &mut BTreeMap::new());
        self.root = self
            .arena
            .conditional(self.root, reversed, &controls, 0, &mut BTreeMap::new());
        Ok(())
    }
    fn checkpoint(&mut self) -> Result<(), &'static str> {
        self.fold();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn entangled_measurement_preserves_joint_correlations() {
        for (quantile, expected) in [
            (vec![EVALT], vec![EVALT, EVALT]),
            (one(), vec![EVALF, EVALF]),
        ] {
            let mut r = FoldedRegister::zero(2);
            r.hadamard(0).unwrap();
            r.toggle(
                &[Control {
                    cell: 0,
                    value: EVALF,
                }],
                1,
            )
            .unwrap();
            let measured = r.measure(&quantile, &vec![EVALT, EVALF]).unwrap();
            assert_eq!(measured, expected);
            assert_eq!(r.measure(&one(), &vec![EVALT, EVALF]).unwrap(), expected);
        }
    }
    #[test]
    fn phase_changes_interference_and_hadamard_inverse_closes() {
        let mut r = FoldedRegister::zero(1);
        r.hadamard(0).unwrap();
        r.phase_flip(&[Control {
            cell: 0,
            value: EVALF,
        }])
        .unwrap();
        r.hadamard(0).unwrap();
        assert_eq!(r.measure(&one(), &vec![EVALT, EVALF]).unwrap(), vec![EVALF]);
        let mut r = FoldedRegister::zero(1);
        r.hadamard(0).unwrap();
        r.hadamard(0).unwrap();
        assert_eq!(r.measure(&one(), &vec![EVALT, EVALF]).unwrap(), vec![EVALT]);
    }
    #[test]
    fn uniform_register_folds_without_basis_expansion() {
        let mut r = FoldedRegister::zero(64);
        for wire in 0..64 {
            r.hadamard(wire).unwrap();
            r.fold();
        }
        assert_eq!(r.retained_nodes(), 1);
    }
    #[test]
    fn coherent_gate_rejects_alias_and_invalid_quantile() {
        let mut r = FoldedRegister::zero(2);
        assert!(r
            .toggle(
                &[Control {
                    cell: 0,
                    value: EVALF
                }],
                0
            )
            .is_err());
        assert!(r
            .phase_flip(&[Control {
                cell: 0,
                value: 'H'
            }])
            .is_err());
        assert!(r.measure(&one(), &one()).is_err());
    }
}
