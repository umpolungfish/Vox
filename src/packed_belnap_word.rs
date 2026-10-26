//! Parallel FOUR arithmetic on truth and falsehood evidence rails.
//! Neither and both retain their distinct rail pairs throughout each gate.
use alloc::{vec,vec::Vec};
use core::cmp::Ordering;
use crate::belnap::B4;

#[derive(Clone,Copy)]
struct Rails { t:u64,f:u64 }
impl Rails {
    const ZERO:Self=Self{t:0,f:u64::MAX};
    fn broadcast(value:B4)->Self {
        Self{t:0u64.wrapping_sub((value as u8&1) as u64),
            f:0u64.wrapping_sub(((value as u8>>1)&1) as u64)}
    }
    fn and(self,b:Self)->Self {Self{t:self.t&b.t,f:self.f|b.f}}
    fn or(self,b:Self)->Self {Self{t:self.t|b.t,f:self.f&b.f}}
    fn not(self)->Self {Self{t:self.f,f:self.t}}
    fn xor(self,b:Self)->Self {self.and(b.not()).or(self.not().and(b))}
    fn cell(self,at:usize)->B4 {B4::from_u8((((self.t>>at)&1)|(((self.f>>at)&1)<<1)) as u8)}
    fn set(&mut self,at:usize,value:B4) {
        let mask=1u64<<at;
        self.t=(self.t&!mask)|(((value as u64)&1)<<at);
        self.f=(self.f&!mask)|((((value as u64)>>1)&1)<<at);
    }
    fn shift(self,by:usize,pad:B4)->Self {
        let mask=(1u64<<by)-1;let fill=Self::broadcast(pad);
        Self{t:(self.t<<by)|(fill.t&mask),f:(self.f<<by)|(fill.f&mask)}
    }
    /// Compose carry maps c -> g OR (p AND c), in the FDE truth lattice.
    fn carries(mut g:Self,mut p:Self,input:B4)->Self {
        for distance in [1,2,4,8,16,32] {
            g=g.or(p.and(g.shift(distance,B4::F)));
            p=p.and(p.shift(distance,B4::T));
        }
        g.or(p.and(Self::broadcast(input)))
    }
    fn add(self,b:Self,input:B4)->(Self,B4) {
        let pair=self.xor(b);
        let carries=Self::carries(self.and(b),pair,input);
        (pair.xor(carries.shift(1,input)),carries.cell(63))
    }
    fn sub(self,b:Self,input:B4)->(Self,B4,Self) {
        let borrows=Self::carries(self.not().and(b),self.not().or(b),input);
        (self.xor(b).xor(borrows.shift(1,input)),borrows.cell(63),borrows)
    }
}

/// One native residue coordinate, with both FOUR evidence rails resident.
/// Numerical modular operations use an explicit checked image and return
/// both rails. Neither and both retain their original values on rejection.
#[derive(Clone,Copy)]
pub(crate) struct BelnapCoordinate64 { rails:Rails }
impl BelnapCoordinate64 {
    pub(crate) fn from_rails(t:u64,f:u64)->Self{Self{rails:Rails{t,f}}}
    pub(crate) fn from_definite_u64(t:u64)->Self{Self::from_rails(t,!t)}
    pub(crate) fn definite_u64(&self)->Result<u64,&'static str>{
        if self.rails.t^self.rails.f!=u64::MAX{return Err("coordinate retains neither or both evidence");}
        Ok(self.rails.t)
    }
    fn images(&self,b:&Self,modulus:&Self)->Result<(u64,u64,u64),&'static str>{
        let a=self.definite_u64()?;let b=b.definite_u64()?;let m=modulus.definite_u64()?;
        if m==0{return Err("coordinate modulus is zero");}Ok((a,b,m))
    }
    pub(crate) fn add_mod(&self,b:&Self,modulus:&Self)->Result<Self,&'static str>{
        let (a,b,m)=self.images(b,modulus)?;
        Ok(Self::from_definite_u64(((a as u128+b as u128)%m as u128) as u64))
    }
    pub(crate) fn multiply_mod(&self,b:&Self,modulus:&Self)->Result<Self,&'static str>{
        let (a,b,m)=self.images(b,modulus)?;
        Ok(Self::from_definite_u64((a as u128*b as u128%m as u128) as u64))
    }
    pub(crate) fn subtract_mod(&self,b:&Self,modulus:&Self)->Result<Self,&'static str>{
        let (a,b,m)=self.images(b,modulus)?;
        let value=(a as u128%m as u128+m as u128-b as u128%m as u128)%m as u128;
        Ok(Self::from_definite_u64(value as u64))
    }
    /// Euclidean inverse within the checked numerical restriction. Keep each
    /// remainder and coefficient on both rails, without allocating tapes.
    pub(crate) fn inverse_mod(&self,modulus:&Self)->Result<Option<Self>,&'static str>{
        let (a,_,m)=self.images(self,modulus)?;
        if m<2{return Ok(None);}
        let mut r=*modulus;
        let mut next_r=Self::from_definite_u64(a%m);
        let mut t=Self::from_definite_u64(0);
        let mut next_t=Self::from_definite_u64(1);
        while next_r.definite_u64()?!=0 {
            let divisor=next_r.definite_u64()?;
            let dividend=r.definite_u64()?;
            let quotient=Self::from_definite_u64(dividend/divisor);
            let remainder=Self::from_definite_u64(dividend%divisor);
            let coefficient=t.subtract_mod(&quotient.multiply_mod(&next_t,modulus)?,modulus)?;
            r=next_r;next_r=remainder;t=next_t;next_t=coefficient;
        }
        Ok(if r.definite_u64()?==1{Some(t)}else{None})
    }
}

pub(crate) struct PackedWord { words:Vec<Rails>,len:usize }
impl PackedWord {
    /// Enter the explicit definite numeral boundary directly into both rails.
    pub(crate) fn from_parity(cells:&[char])->Self {
        let mut words=vec![Rails::ZERO;cells.len().div_ceil(64)];
        for (at,&mark) in cells.iter().enumerate() {
            words[at/64].set(at%64,B4::from_parity(mark));
        }
        Self::normalize(words,cells.len())
    }
    pub(crate) fn from_definite_u64(t:u64)->Self {
        Self::normalize(vec![Rails{t,f:!t}],64)
    }
    pub(crate) fn to_parity(&self)->Result<Vec<char>,&'static str>{
        if !self.definite(){return Err("numeral return awaits both definite evidence rails");}
        if self.len==0{return Ok(vec!['⊤']);}
        Ok((0..self.len).map(|i|if self.cell(i)==B4::T{'⊥'}else{'⊤'}).collect())
    }
    pub(crate) fn definite_u64(&self)->Result<u64,&'static str>{
        if !self.definite(){return Err("coordinate return awaits both definite evidence rails");}
        if self.len>64{return Err("coordinate exceeds its retained width");}
        Ok(self.word(0).t)
    }
    pub(super) fn from_cells(cells:&[B4])->Self {
        let mut words=vec![Rails::ZERO;cells.len().div_ceil(64)];
        for (at,&value) in cells.iter().enumerate(){words[at/64].set(at%64,value);}
        Self::normalize(words,cells.len())
    }
    fn normalize(mut words:Vec<Rails>,len:usize)->Self {
        words.truncate(len.div_ceil(64));
        if len%64!=0 {if let Some(last)=words.last_mut(){
            let mask=(1u64<<(len%64))-1;last.t&=mask;last.f|=!mask;
        }}
        while words.last().is_some_and(|w|w.t==0&&w.f==u64::MAX){words.pop();}
        let len=words.last().map_or(0,|w|(words.len()-1)*64+64-(w.t|!w.f).leading_zeros() as usize);
        Self{words,len}
    }
    fn word(&self,at:usize)->Rails {self.words.get(at).copied().unwrap_or(Rails::ZERO)}
    fn cell(&self,at:usize)->B4 {self.word(at/64).cell(at%64)}
    pub(super) fn cells(&self)->Vec<B4> {(0..self.len).map(|at|self.cell(at)).collect()}
    fn definite(&self)->bool {self.words.iter().all(|w|w.t^w.f==u64::MAX)}
    fn compare_definite(&self,b:&Self)->Ordering {
        match self.len.cmp(&b.len) {
            Ordering::Equal=>{
                for at in (0..self.words.len()).rev(){
                    let order=self.words[at].t.cmp(&b.words[at].t);
                    if order!=Ordering::Equal{return order;}
                }
                Ordering::Equal
            },order=>order,
        }
    }
    pub(crate) fn compare(&self,b:&Self)->Result<Ordering,&'static str> {
        if !self.definite()||!b.definite(){return Err("ordering awaits definite Belnap numerical cells");}
        Ok(self.compare_definite(b))
    }
    pub(crate) fn add(&self,b:&Self)->Self {
        // The definite numerical restriction is checked before using its
        // native word image. Original evidence operands remain unchanged.
        if self.definite() && b.definite() {
            let mut words=Vec::with_capacity(self.words.len().max(b.words.len())+1);
            let mut carry=0u128;
            for at in 0..self.words.len().max(b.words.len()) {
                let sum=self.word(at).t as u128+b.word(at).t as u128+carry;
                let t=sum as u64;words.push(Rails{t,f:!t});carry=sum>>64;
            }
            // Retain the high zero word until logical-width normalization:
            // a64k-bit operand can have a64k+1-bit result width with no carry.
            words.push(Rails{t:carry as u64,f:!(carry as u64)});
            return Self::normalize(words,self.len.max(b.len)+1);
        }
        let mut words=Vec::with_capacity(self.words.len().max(b.words.len())+1);
        let mut carry=B4::F;
        for at in 0..self.words.len().max(b.words.len()) {
            let (sum,next)=self.word(at).add(b.word(at),carry);words.push(sum);carry=next;
        }
        let mut last=Rails::ZERO;last.set(0,carry);words.push(last);
        Self::normalize(words,self.len.max(b.len)+1)
    }
    pub(crate) fn subtract(&self,b:&Self)->(Self,B4) {
        let width=self.len.max(b.len);
        if width==0{return (Self::from_cells(&[]),B4::F);}
        if self.definite() && b.definite() {
            let mut words=Vec::with_capacity(width.div_ceil(64));let mut borrow=false;
            for at in 0..width.div_ceil(64) {
                let (difference,first)=self.word(at).t.overflowing_sub(b.word(at).t);
                let (t,second)=difference.overflowing_sub(u64::from(borrow));
                words.push(Rails{t,f:!t});borrow=first||second;
            }
            return (Self::normalize(words,width),if borrow{B4::T}else{B4::F});
        }
        let mut words=Vec::with_capacity(width.div_ceil(64));let mut borrow=B4::F;let mut terminal=B4::F;
        for at in 0..width.div_ceil(64) {
            let (difference,next,borrows)=self.word(at).sub(b.word(at),borrow);
            words.push(difference);borrow=next;
            if at==width.div_ceil(64)-1{terminal=borrows.cell((width-1)%64);}
        }
        (Self::normalize(words,width),terminal)
    }
    fn shift(&self,by:usize)->Self {
        if self.len==0{return Self::from_cells(&[]);}
        let whole=by/64;let bits=by%64;let len=self.len+by;
        let mut words=vec![Rails::ZERO;len.div_ceil(64)];
        for (at,out) in words.iter_mut().enumerate().skip(whole) {
            let here=self.word(at-whole);
            *out=if bits==0{here}else{
                let previous=if at==whole{Rails::ZERO}else{self.word(at-whole-1)};
                Rails{t:(here.t<<bits)|(previous.t>>(64-bits)),f:(here.f<<bits)|(previous.f>>(64-bits))}
            };
        }
        Self::normalize(words,len)
    }
    fn lower(&self,bits:usize)->Self {
        if bits>=self.len{return Self::from_cells(&[]);}
        let whole=bits/64;let offset=bits%64;let len=self.len-bits;
        let words=(0..len.div_ceil(64)).map(|at|{
            let here=self.word(at+whole);
            if offset==0{here}else{
                let next=self.word(at+whole+1);
                Rails{t:(here.t>>offset)|(next.t<<(64-offset)),
                    f:(here.f>>offset)|(next.f<<(64-offset))}
            }
        }).collect();
        Self::normalize(words,len)
    }
    pub(crate) fn multiply(&self,b:&Self)->Self {
        if self.definite() && b.definite() {
            let mut product=vec![0u64;self.words.len()+b.words.len()];
            for (i,a) in self.words.iter().enumerate() {
                let mut carry=0u128;
                for (j,b) in b.words.iter().enumerate() {
                    let value=a.t as u128*b.t as u128+product[i+j] as u128+carry;
                    product[i+j]=value as u64;carry=value>>64;
                }
                if !b.words.is_empty(){product[i+b.words.len()]=carry as u64;}
            }
            return Self::normalize(product.into_iter().map(|t|Rails{t,f:!t}).collect(),self.len+b.len);
        }
        let mut product=Self::from_cells(&[]);
        for at in 0..b.len {
            let value=b.cell(at);if value==B4::F{continue;}
            let rail=Rails::broadcast(value);
            let row=Self::normalize(self.words.iter().map(|&word|word.and(rail)).collect(),self.len).shift(at);
            product=product.add(&row);
        }
        product
    }
    pub(crate) fn divide(&self,b:&Self)->Result<(Self,Self),&'static str> {
        if !self.definite()||!b.definite(){return Err("division awaits definite Belnap numerical cells");}
        if b.len==0{return Err("Belnap numerical division by zero");}
        // Division by a single occupied numerical position is exactly a
        // rail cut. Keep both rails; no carry network has to be executed.
        if b.words.iter().map(|word|word.t.count_ones()).sum::<u32>()==1 {
            let cut=b.len-1;
            let remainder=Self::normalize(self.words.iter().take(cut.div_ceil(64)).copied().collect(),cut.min(self.len));
            return Ok((self.lower(cut),remainder));
        }
        if b.words.len()==1 {
            let divisor=b.words[0].t as u128;
            let mut quotient=vec![Rails::ZERO;self.words.len()];let mut remainder=0u128;
            for at in (0..self.words.len()).rev() {
                let value=(remainder<<64)|self.words[at].t as u128;
                let t=(value/divisor) as u64;quotient[at]=Rails{t,f:!t};remainder=value%divisor;
            }
            let t=remainder as u64;
            return Ok((Self::normalize(quotient,self.len),Self::normalize(vec![Rails{t,f:!t}],64)));
        }
        match self.compare_definite(b) {
            Ordering::Less=>return Ok((Self::from_cells(&[]),Self::normalize(self.words.clone(),self.len))),
            Ordering::Equal=>return Ok((Self::from_cells(&[B4::T]),Self::from_cells(&[]))),
            Ordering::Greater=>{},
        }
        // This restriction was checked above on both complete evidence rails.
        // Use the existing normalized whole-limb division, then reconstruct
        // complementary rails. The immutable source evidence remains resident.
        let n:Vec<u64>=self.words.iter().map(|word|word.t).collect();
        let d:Vec<u64>=b.words.iter().map(|word|word.t).collect();
        let (quotient,remainder)=definite_division(&n,&d);
        let rails=|limbs:Vec<u64>| {
            let len=limbs.len()*64;
            Self::normalize(limbs.into_iter().map(|t|Rails{t,f:!t}).collect(),len)
        };
        Ok((rails(quotient),rails(remainder)))
    }
}

// Normalized division on the explicitly restricted definite numerical image.
fn definite_division(n:&[u64],d:&[u64])->(Vec<u64>,Vec<u64>) {
    let shift = d[d.len() - 1].leading_zeros();
    let mut v = vec![0u64; d.len()];
    let mut carry = 0u64;
    for (i, &limb) in d.iter().enumerate() {
        v[i] = (limb << shift) | carry;
        carry = if shift == 0 { 0 } else { limb >> (64 - shift) };
    }

    let mut u = vec![0u64; n.len() + 1];
    carry = 0;
    for (i, &limb) in n.iter().enumerate() {
        u[i] = (limb << shift) | carry;
        carry = if shift == 0 { 0 } else { limb >> (64 - shift) };
    }
    u[n.len()] = carry;

    let divisor_len = v.len();
    let top = v[divisor_len - 1] as u128;
    let m = n.len() - divisor_len;
    let mut q = vec![0u64; m + 1];
    for j in (0..=m).rev() {
        let numerator = ((u[j + divisor_len] as u128) << 64) | u[j + divisor_len - 1] as u128;
        let mut qhat = numerator / top;
        let mut rhat = numerator % top;
        if qhat >= (1u128 << 64) {
            qhat = (1u128 << 64) - 1;
            rhat = numerator - qhat * top;
        }
        while rhat < (1u128 << 64)
            && qhat * v[divisor_len - 2] as u128 > (rhat << 64) + u[j + divisor_len - 2] as u128
        {
            qhat -= 1;
            rhat += top;
        }

        let mut borrow = 0u128;
        for i in 0..divisor_len {
            let product = qhat * v[i] as u128 + borrow;
            let low = product as u64;
            borrow = product >> 64;
            let old = u[j + i];
            u[j + i] = old.wrapping_sub(low);
            if old < low {
                borrow += 1;
            }
        }
        let old_high = u[j + divisor_len];
        let underflow = (old_high as u128) < borrow;
        u[j + divisor_len] = old_high.wrapping_sub(borrow as u64);
        if underflow {
            qhat -= 1;
            let mut add_carry = 0u128;
            for i in 0..divisor_len {
                let sum = u[j + i] as u128 + v[i] as u128 + add_carry;
                u[j + i] = sum as u64;
                add_carry = sum >> 64;
            }
            u[j + divisor_len] = u[j + divisor_len].wrapping_add(add_carry as u64);
        }
        q[j] = qhat as u64;
    }

    let mut r = vec![0u64; divisor_len];
    if shift == 0 {
        r.copy_from_slice(&u[..divisor_len]);
    } else {
        for i in 0..divisor_len {
            r[i] = (u[i] >> shift) | (u[i + 1] << (64 - shift));
        }
    }
    (q, r)
}

#[cfg(test)]
mod division_tests {
    use super::*;
    #[test]
    fn definite_division_matches_retained_bit_rail_reference() {
        let mut seed=0x9e3779b97f4a7c15u64;
        for width in [128usize,150,175,200,256,300,512,1024] {
            for divisor_width in [65usize,127,128,width/2,width-1,width,width+1] {
                for _ in 0..8 {
                    let mut generate=|size:usize| {
                        let mut cells:Vec<B4>=(0..size).map(|_| {
                            seed^=seed<<13;seed^=seed>>7;seed^=seed<<17;
                            if seed&1==1 {B4::T}else{B4::F}
                        }).collect();
                        cells[size-1]=B4::T;PackedWord::from_cells(&cells)
                    };
                    let n=generate(width);let d=generate(divisor_width);
                    let before=n.cells();let divisor_before=d.cells();
                    let (q,r)=n.divide(&d).unwrap();
                    let mut quotient=vec![B4::F;n.len];let mut remainder=PackedWord::from_cells(&[]);
                    for at in (0..n.len).rev() {
                        remainder=remainder.shift(1);
                        if remainder.words.is_empty(){remainder.words.push(Rails::ZERO);}
                        remainder.words[0].set(0,n.cell(at));
                        remainder=PackedWord::normalize(remainder.words,remainder.len.max(1));
                        if remainder.compare_definite(&d)!=Ordering::Less {
                            let (next,borrow)=remainder.subtract(&d);
                            assert_eq!(borrow,B4::F);remainder=next;quotient[at]=B4::T;
                        }
                    }
                    assert_eq!(q.cells(),PackedWord::from_cells(&quotient).cells());
                    assert_eq!(r.cells(),remainder.cells());
                    assert_eq!(n.cells(),before);assert_eq!(d.cells(),divisor_before);
                    assert!(q.definite()&&r.definite());
                }
            }
        }
    }
}
