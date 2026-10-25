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

pub(super) struct PackedWord { words:Vec<Rails>,len:usize }
impl PackedWord {
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
    pub(super) fn compare(&self,b:&Self)->Result<Ordering,&'static str> {
        if !self.definite()||!b.definite(){return Err("ordering awaits definite Belnap numerical cells");}
        Ok(self.compare_definite(b))
    }
    pub(super) fn add(&self,b:&Self)->Self {
        let mut words=Vec::with_capacity(self.words.len().max(b.words.len())+1);
        let mut carry=B4::F;
        for at in 0..self.words.len().max(b.words.len()) {
            let (sum,next)=self.word(at).add(b.word(at),carry);words.push(sum);carry=next;
        }
        let mut last=Rails::ZERO;last.set(0,carry);words.push(last);
        Self::normalize(words,self.len.max(b.len)+1)
    }
    pub(super) fn subtract(&self,b:&Self)->(Self,B4) {
        let width=self.len.max(b.len);
        if width==0{return (Self::from_cells(&[]),B4::F);}
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
    pub(super) fn multiply(&self,b:&Self)->Self {
        let mut product=Self::from_cells(&[]);
        for at in 0..b.len {
            let value=b.cell(at);if value==B4::F{continue;}
            let rail=Rails::broadcast(value);
            let row=Self::normalize(self.words.iter().map(|&word|word.and(rail)).collect(),self.len).shift(at);
            product=product.add(&row);
        }
        product
    }
    pub(super) fn divide(&self,b:&Self)->Result<(Self,Self),&'static str> {
        if !self.definite()||!b.definite(){return Err("division awaits definite Belnap numerical cells");}
        if b.len==0{return Err("Belnap numerical division by zero");}
        // Division by a single occupied numerical position is exactly a
        // rail cut. Keep both rails; no carry network has to be executed.
        if b.words.iter().map(|word|word.t.count_ones()).sum::<u32>()==1 {
            let cut=b.len-1;
            let remainder=Self::normalize(self.words.iter().take(cut.div_ceil(64)).copied().collect(),cut.min(self.len));
            return Ok((self.lower(cut),remainder));
        }
        match self.compare_definite(b) {
            Ordering::Less=>return Ok((Self::from_cells(&[]),Self::normalize(self.words.clone(),self.len))),
            Ordering::Equal=>return Ok((Self::from_cells(&[B4::T]),Self::from_cells(&[]))),
            Ordering::Greater=>{},
        }
        let mut quotient=vec![B4::F;self.len];let mut remainder=Self::from_cells(&[]);
        for at in (0..self.len).rev() {
            remainder=remainder.shift(1);
            if remainder.words.is_empty(){remainder.words.push(Rails::ZERO);}
            remainder.words[0].set(0,self.cell(at));
            remainder=Self::normalize(remainder.words,remainder.len.max(1));
            if remainder.compare_definite(b)!=Ordering::Less {
                let (next,borrow)=remainder.subtract(b);
                debug_assert_eq!(borrow,B4::F);remainder=next;quotient[at]=B4::T;
            }
        }
        Ok((Self::from_cells(&quotient),remainder))
    }
}
