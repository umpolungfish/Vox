//! Finite relations over FOUR. A set of alternatives is not a B4 value.
//! Repeated cell addresses select the same value in every surviving row.
use crate::belnap::B4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BelnapSet(u8);
impl BelnapSet {
    pub const EMPTY: Self = Self(0);
    pub const COMPLETE: Self = Self(15);
    pub const NUMERAL: Self = Self((1 << B4::T as u8) | (1 << B4::F as u8));
    pub const fn singleton(value: B4) -> Self { Self(1 << value as u8) }
    pub const fn from_mask(mask: u8) -> Option<Self> {
        if mask <= 15 { Some(Self(mask)) } else { None }
    }
    pub const fn mask(self) -> u8 { self.0 }
    pub const fn contains(self, value: B4) -> bool { self.0 & (1 << value as u8) != 0 }
    pub const fn union(self, other: Self) -> Self { Self(self.0 | other.0) }
    pub const fn intersection(self, other: Self) -> Self { Self(self.0 & other.0) }
    pub const fn only(self) -> Option<B4> {
        match self.0 { 1 => Some(B4::N), 2 => Some(B4::T), 4 => Some(B4::F), 8 => Some(B4::B), _ => None }
    }
    /// A deliberate numeral boundary. N and B are not discarded from a set.
    pub const fn numeral_projection(self) -> Result<u32, &'static str> {
        if self.0 & !Self::NUMERAL.0 != 0 {
            return Err("numeral projection retains nonclassical Belnap alternatives");
        }
        Ok((if self.contains(B4::F) {1} else {0}) | (if self.contains(B4::T) {2} else {0}))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GateKind { And, Not, Equal, Add }

pub struct BelnapGateTable {
    pub kind: GateKind,
    pub width: usize,
    pub rows: usize,
    pub supports: [[u64; 4]; 5],
}
impl GateKind {
    pub const fn input_count(self) -> usize {
        match self { Self::And => 2, Self::Not | Self::Equal => 1, Self::Add => 3 }
    }
    pub const fn width(self) -> usize {
        match self { Self::And => 3, Self::Not | Self::Equal => 2, Self::Add => 5 }
    }
    pub const fn row(self, index: usize) -> [B4; 5] {
        let a = B4::from_u8((index & 3) as u8);
        let b = B4::from_u8(((index >> 2) & 3) as u8);
        let c = B4::from_u8(((index >> 4) & 3) as u8);
        match self {
            Self::And => [a,b,a.truth_and(b),B4::N,B4::N],
            Self::Not => [a,a.bnot(),B4::N,B4::N,B4::N],
            Self::Equal => [a,a,B4::N,B4::N,B4::N],
            Self::Add => {
                let pair = a.truth_xor(b);
                [a,b,c,pair.truth_xor(c),a.truth_and(b).truth_or(pair.truth_and(c))]
            }
        }
    }
}
impl BelnapGateTable {
    pub const fn compile(kind: GateKind) -> Self {
        let rows = 1 << (2 * kind.input_count());
        let width = kind.width();
        let mut supports = [[0;4];5];
        let mut index = 0;
        while index < rows {
            let row = kind.row(index);
            let mut slot = 0;
            while slot < width {
                supports[slot][row[slot] as usize] |= 1u64 << index;
                slot += 1;
            }
            index += 1;
        }
        Self { kind, width, rows, supports }
    }
    pub const fn all(&self) -> u64 {
        if self.rows == 64 { u64::MAX } else { (1u64 << self.rows)-1 }
    }
    /// Derive the definite slice from the full relation. No second truth table.
    pub const fn numeral_supports(&self) -> [[u8;2];5] {
        let mut supports = [[0;2];5];
        let count = self.kind.input_count();
        let mut index = 0;
        while index < (1 << count) {
            let mut full = 0;
            let mut at = 0;
            while at < count {
                let value = if index & (1 << (count-at-1)) == 0 { B4::F } else { B4::T };
                full |= (value as usize) << (2*at);
                at += 1;
            }
            let row = self.kind.row(full);
            let mut slot = 0;
            while slot < self.width {
                let mark = match row[slot] { B4::F => 0, B4::T => 1, _ => panic!("numeral slice is not closed") };
                supports[slot][mark] |= 1 << index;
                slot += 1;
            }
            index += 1;
        }
        supports
    }
    pub fn correlated_rows(&self, cells: &[usize]) -> u64 {
        assert_eq!(cells.len(),self.width);
        let mut rows = self.all();
        for index in 0..self.rows {
            let row = self.kind.row(index);
            for a in 0..self.width { for b in 0..a {
                if cells[a] == cells[b] && row[a] != row[b] { rows &= !(1u64 << index); }
            }}
        }
        rows
    }
    pub fn restrict(&self, mut rows: u64, domains: &[BelnapSet]) -> u64 {
        assert_eq!(domains.len(),self.width);
        rows &= self.all();
        for (slot,domain) in domains.iter().enumerate() {
            let mut admitted = 0;
            for value in [B4::N,B4::T,B4::F,B4::B] {
                if domain.contains(value) { admitted |= self.supports[slot][value as usize]; }
            }
            rows &= admitted;
        }
        rows
    }
    pub fn domain(&self, rows:u64, slot:usize) -> BelnapSet {
        assert!(slot < self.width);
        let mut result=BelnapSet::EMPTY;
        for value in [B4::N,B4::T,B4::F,B4::B] {
            if rows & self.supports[slot][value as usize] != 0 {
                result=result.union(BelnapSet::singleton(value));
            }
        }
        result
    }
}

pub static AND: BelnapGateTable = BelnapGateTable::compile(GateKind::And);
pub static NOT: BelnapGateTable = BelnapGateTable::compile(GateKind::Not);
pub static EQUAL: BelnapGateTable = BelnapGateTable::compile(GateKind::Equal);
pub static ADD: BelnapGateTable = BelnapGateTable::compile(GateKind::Add);

impl GateKind {
    pub fn table(self) -> &'static BelnapGateTable {
        match self { Self::And=>&AND,Self::Not=>&NOT,Self::Equal=>&EQUAL,Self::Add=>&ADD }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn belnap_relation_distinguishes_evidence_and_powerset_alternatives() {
        assert_ne!(BelnapSet::EMPTY,BelnapSet::singleton(B4::N));
        assert_ne!(BelnapSet::COMPLETE,BelnapSet::singleton(B4::B));
        assert_ne!(BelnapSet::NUMERAL,BelnapSet::singleton(B4::B));
        assert!(BelnapSet::COMPLETE.numeral_projection().is_err());
        assert!(BelnapSet::singleton(B4::N).numeral_projection().is_err());
        assert_eq!(BelnapSet::NUMERAL.numeral_projection(),Ok(3));
    }
    #[test]
    fn belnap_relation_preserves_every_full_four_gate_row() {
        for kind in [GateKind::And,GateKind::Not,GateKind::Equal,GateKind::Add] {
            let table=kind.table();
            for index in 0..table.rows {
                let row=kind.row(index);
                let mut domains=[BelnapSet::COMPLETE;5];
                for at in 0..kind.input_count() {domains[at]=BelnapSet::singleton(row[at]);}
                let retained=table.restrict(table.all(),&domains[..table.width]);
                assert_ne!(retained,0);
                for at in 0..table.width {assert_eq!(table.domain(retained,at).only(),Some(row[at]));}
            }
        }
    }
    #[test]
    fn belnap_relation_fde_contradiction_is_contained_and_aliases_correlate() {
        let rows=AND.restrict(AND.all(),&[BelnapSet::singleton(B4::N),
            BelnapSet::singleton(B4::B),BelnapSet::COMPLETE]);
        assert_eq!(AND.domain(rows,2).only(),Some(B4::F));
        let rows=AND.restrict(AND.all(),&[BelnapSet::singleton(B4::B),
            BelnapSet::singleton(B4::T),BelnapSet::COMPLETE]);
        assert_eq!(AND.domain(rows,2).only(),Some(B4::B));
        let aliases=AND.correlated_rows(&[8,8,9]);
        let rows=AND.restrict(aliases,&[BelnapSet::COMPLETE,BelnapSet::COMPLETE,BelnapSet::singleton(B4::T)]);
        assert_eq!(AND.domain(rows,0).only(),Some(B4::T));
        assert_eq!(AND.domain(rows,1).only(),Some(B4::T));
    }
}
