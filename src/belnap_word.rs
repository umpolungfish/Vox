//! Numerical cells in FOUR. Neither and both remain resident through gates;
//! only an explicit definite boundary can expose a numeral or an ordering.
use alloc::vec::Vec;
#[cfg(test)]
use alloc::vec;
use core::cmp::Ordering;
use crate::belnap::B4;
#[cfg(test)]
use crate::fixed_point_word_arithmetic::{belnap_add_cell, belnap_sub_cell};
#[path="packed_belnap_word.rs"]
mod packed;
pub(crate) use packed::PackedWord;
pub(crate) use packed::BelnapCoordinate64;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BelnapWord(pub Vec<B4>);

impl BelnapWord {
    pub fn new(cells: Vec<B4>) -> Self {
        let mut word = Self(cells);
        while word.0.last() == Some(&B4::F) { word.0.pop(); }
        word
    }
    pub fn from_parity(cells: &[char]) -> Self {
        Self::new(cells.iter().map(|&mark| B4::from_parity(mark)).collect())
    }
    pub fn to_parity(&self) -> Result<Vec<char>, &'static str> {
        let mut cells = Vec::with_capacity(self.0.len());
        for &cell in &self.0 {
            cells.push(match cell {
                B4::F => '⊤', B4::T => '⊥',
                B4::N => return Err("neither evidence remains in the numerical word"),
                B4::B => return Err("both evidence remains in the numerical word"),
            });
        }
        if cells.is_empty() { cells.push('⊤'); }
        Ok(cells)
    }
    pub fn cell(&self, at: usize) -> B4 { self.0.get(at).copied().unwrap_or(B4::F) }
    pub fn add(&self, other: &Self) -> Self {
        Self(PackedWord::from_cells(&self.0).add(&PackedWord::from_cells(&other.0)).cells())
    }
    /// The terminal borrow remains a FOUR value, including neither and both.
    pub fn subtract(&self, other: &Self) -> (Self,B4) {
        let (difference,borrow)=PackedWord::from_cells(&self.0).subtract(&PackedWord::from_cells(&other.0));
        (Self(difference.cells()),borrow)
    }
    pub fn multiply(&self, other: &Self) -> Self {
        Self(PackedWord::from_cells(&self.0).multiply(&PackedWord::from_cells(&other.0)).cells())
    }
    pub fn compare(&self, other: &Self) -> Result<Ordering,&'static str> {
        PackedWord::from_cells(&self.0).compare(&PackedWord::from_cells(&other.0))
    }
    pub fn divide(&self, divisor: &Self) -> Result<(Self,Self),&'static str> {
        let (quotient,remainder)=PackedWord::from_cells(&self.0).divide(&PackedWord::from_cells(&divisor.0))?;
        Ok((Self(quotient.cells()),Self(remainder.cells())))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn packed_numeral_boundaries_preserve_rails_and_wide_residue_values(){
        use crate::morphism_factor as mf;
        let source=mf::decimal_to_tape("1306593903193228327879866132486971792765599839798626833574803").unwrap();
        assert_eq!(source.len(),200);
        let packed=PackedWord::from_parity(&source);
        assert_eq!(packed.to_parity().unwrap(),source);
        let original=packed.cells();
        for modulus in [3u64,31,257,65537,4294967291,u64::MAX] {
            let mut remainder=0u128;
            for &mark in source.iter().rev(){remainder=(2*remainder+u128::from(mark=='⊥'))%u128::from(modulus);}
            let result=packed.divide(&PackedWord::from_definite_u64(modulus)).unwrap().1;
            assert_eq!(result.definite_u64().unwrap(),remainder as u64);
            let coordinate=packed.remainder_coordinate(&BelnapCoordinate64::from_definite_u64(modulus)).unwrap();
            assert_eq!(coordinate.definite_u64().unwrap(),remainder as u64);
            assert_eq!(packed.cells(),original);
            for a in [0,1,modulus-1,u64::MAX] {for b in [0,1,modulus-1,u64::MAX] {
                assert_eq!(mf::residue_mul(a,b,modulus),(a as u128*b as u128%modulus as u128) as u64);
                assert_eq!(mf::residue_add(a,b,modulus),((a as u128+b as u128)%modulus as u128) as u64);
                assert_eq!(mf::residue_sub(a,b,modulus),((a as u128%modulus as u128+modulus as u128-b as u128%modulus as u128)%modulus as u128) as u64);
            }}
            for value in [0,1,2,3,modulus-1,u64::MAX] {
                let inverse=mf::inverse_residue_word(value,modulus);
                let reference=mf::mod_inv(&mf::tape_u64(value),&mf::tape_u64(modulus)).ok();
                assert_eq!(inverse,reference.map(|word|PackedWord::from_parity(&word).definite_u64().unwrap()));
                if let Some(inverse)=inverse {
                    assert_eq!((value as u128*inverse as u128)%modulus as u128,1);
                }
            }
        }
        for value in [B4::N,B4::B] {
            let mut cells=original.clone();cells[70]=value;
            let held=PackedWord::from_cells(&cells);
            assert!(held.to_parity().is_err());
            assert!(held.definite_u64().is_err());
            assert!(held.divide(&PackedWord::from_definite_u64(3)).is_err());
            assert!(held.remainder_coordinate(&BelnapCoordinate64::from_definite_u64(3)).is_err());
            assert_eq!(held.cells(),cells);
        }
        for (t,f) in [(0,0),(u64::MAX,u64::MAX),(1,1),(1,0)] {
            let held=BelnapCoordinate64::from_rails(t,f);
            let definite=BelnapCoordinate64::from_definite_u64(3);
            assert!(held.add_mod(&definite,&definite).is_err());
            assert!(held.multiply_mod(&definite,&definite).is_err());
            assert!(held.subtract_mod(&definite,&definite).is_err());
            assert!(held.inverse_mod(&definite).is_err());
            assert!(packed.remainder_coordinate(&held).is_err());
            assert!(held.definite_u64().is_err());
        }
    }
    fn serial_add(a:&[B4],b:&[B4])->BelnapWord {
        let mut carry=B4::F;let mut out=Vec::new();
        for at in 0..a.len().max(b.len()) {
            let (sum,next)=belnap_add_cell(a.get(at).copied().unwrap_or(B4::F),b.get(at).copied().unwrap_or(B4::F),carry);
            out.push(sum);carry=next;
        }
        out.push(carry);BelnapWord::new(out)
    }
    fn serial_sub(a:&[B4],b:&[B4])->(BelnapWord,B4) {
        let mut borrow=B4::F;let mut out=Vec::new();
        for at in 0..a.len().max(b.len()) {
            let (difference,next)=belnap_sub_cell(a.get(at).copied().unwrap_or(B4::F),b.get(at).copied().unwrap_or(B4::F),borrow);
            out.push(difference);borrow=next;
        }
        (BelnapWord::new(out),borrow)
    }
    fn serial_mul(a:&[B4],b:&[B4])->BelnapWord {
        let mut product=BelnapWord::new(Vec::new());
        for (at,&value) in b.iter().enumerate() {
            let mut row=vec![B4::F;at];row.extend(a.iter().map(|&v|v.truth_and(value)));
            product=serial_add(&product.0,&BelnapWord::new(row).0);
        }
        product
    }
    #[test]
    fn belnap_parallel_dyadic_division_preserves_both_evidence_rails() {
        let n=12362336585629042303u128*17072830917575103563u128;
        let make=|value:u128|BelnapWord::new((0..128).map(|at|if value&(1u128<<at)==0{B4::F}else{B4::T}).collect());
        for cut in [0usize,1,63,64,65,127,128,129] {
            let mut divisor=vec![B4::F;cut+1];divisor[cut]=B4::T;
            let (quotient,remainder)=make(n).divide(&BelnapWord::new(divisor)).unwrap();
            let q=if cut<128{n>>cut}else{0};
            let r=if cut<128{n&((1u128<<cut)-1)}else{n};
            assert_eq!(quotient,make(q));assert_eq!(remainder,make(r));
        }
    }
    #[test]
    fn belnap_parallel_words_match_serial_four_arithmetic_across_boundaries() {
        // Widths exercise both sides of the physical packing boundary while
        // retaining at least128 logical cells in every arithmetic comparison.
        for width in [128usize,129,150,175,200,255,256,257,300,512] {
            for seed in 0..16u64 {
                let mut state=0x9e3779b97f4a7c15u64^seed;
                let mut make=|| {state^=state<<13;state^=state>>7;state^=state<<17;B4::from_u8(state as u8)};
                let a=BelnapWord::new((0..width).map(|_|make()).collect());
                let b=BelnapWord::new((0..width-1).map(|_|make()).collect());
                assert_eq!(a.add(&b),serial_add(&a.0,&b.0),"add width={width} seed={seed}");
                assert_eq!(a.subtract(&b),serial_sub(&a.0,&b.0),"sub width={width} seed={seed}");
                if width<=175 {assert_eq!(a.multiply(&b),serial_mul(&a.0,&b.0),"mul width={width} seed={seed}");}
                assert!(a.divide(&b).is_err());
            }
        }
    }

    use super::*;
    #[test]
    fn belnap_cells_match_dual_rail_fde_for_all_inputs() {
        let values = [B4::N, B4::T, B4::F, B4::B];
        let and = |a: B4,b: B4| { let (at,af)=a.to_wh2(); let (bt,bf)=b.to_wh2(); B4::from_wh2(at && bt, af || bf) };
        let or = |a: B4,b: B4| { let (at,af)=a.to_wh2(); let (bt,bf)=b.to_wh2(); B4::from_wh2(at || bt, af && bf) };
        let xor = |a: B4,b: B4| or(and(a,b.bnot()),and(a.bnot(),b));
        for a in values { for b in values {
            assert_eq!(a.truth_and(b),and(a,b));
            assert_eq!(a.truth_or(b),or(a,b));
            for carry in values {
                let expected_sum = xor(xor(a,b),carry);
                let expected_carry = or(and(a,b),and(xor(a,b),carry));
                assert_eq!(belnap_add_cell(a,b,carry),(expected_sum,expected_carry));
                let expected_borrow=or(and(a.bnot(),or(b,carry)),and(b,carry));
                assert_eq!(belnap_sub_cell(a,b,carry),(expected_sum,expected_borrow));
            }
        }}
    }
    #[test]
    fn belnap_open_and_contradictory_words_survive_arithmetic() {
        for value in [B4::N,B4::B] {
            let word=BelnapWord::new(vec![value;128]);
            let zero=BelnapWord::new(Vec::new());
            let one=BelnapWord::new(vec![B4::T]);
            assert_eq!(word.add(&zero),word);
            assert_eq!(word.multiply(&one),word);
            assert!(word.to_parity().is_err());
            assert!(word.compare(&zero).is_err());
            assert!(word.divide(&one).is_err());
        }
    }
    #[test]
    fn belnap_definite_128_bit_product_and_division() {
        // Independent near-balanced source from the retained production receipt.
        let p=12362336585629042303u128;
        let q=17072830917575103563u128;
        let cells=|n:u128| BelnapWord::new((0..128).map(|at|
            if n & (1u128<<at)==0 {B4::F}else{B4::T}).collect());
        let product=cells(p).multiply(&cells(q));
        assert_eq!(product,cells(p*q));
        assert_eq!(product.divide(&cells(p)).unwrap(),(cells(q),cells(0)));
        assert_eq!(cells(p).add(&cells(q)),cells(p+q));
        assert_eq!(cells(p+q).subtract(&cells(q)),(cells(p),B4::F));
    }
}
