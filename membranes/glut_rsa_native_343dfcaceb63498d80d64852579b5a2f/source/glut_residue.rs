//! Residual factor views inside the resident square-phase membrane.
//! The whole tape survives above a folded word. Word-sized views use the
//! multiplicative orbit from meta_router, retaining every exact factor.
use crate::morphism_factor as mf;
use alloc::{vec, vec::Vec};
type Tape=Vec<char>;

fn gcd(mut a:u64,mut b:u64)->u64 {while b!=0 {(a,b)=(b,a%b);}a}
fn multiply(a:u64,b:u64,n:u64)->u64 {
    ((u128::from(a)*u128::from(b))%u128::from(n)) as u64
}
fn power(mut a:u64,mut exponent:u64,n:u64)->u64 {
    let mut value=1;
    while exponent!=0 {
        if exponent&1!=0 {value=multiply(value,a,n);}
        a=multiply(a,a,n);exponent>>=1;
    }
    value
}
// The same witness bases as meta_router's folded-word primality view.
fn prime(n:u64)->bool {
    if n<2 {return false;}
    let bases=[2,3,5,7,11,13,17,19,23,29,31,37];
    for p in bases {if n%p==0 {return n==p;}}
    let shift=(n-1).trailing_zeros();
    let odd=(n-1)>>shift;
    'base:for a in bases {
        let mut x=power(a%n,odd,n);
        if x==1 || x==n-1 {continue;}
        for _ in 1..shift {
            x=multiply(x,x,n);
            if x==n-1 {continue 'base;}
        }
        return false;
    }
    true
}
fn split(n:u64)->u64 {
    if n&1==0 {return 2;}
    let batch=(u64::BITS-n.leading_zeros()) as usize;
    let mut seed=mf::one();
    loop {
        let c=mf::FoldedTape::new(&seed).remainder_word(n);
        seed=mf::add(&seed,&mf::one());
        let next=|x:u64| ((u128::from(x)*u128::from(x)+u128::from(c))%u128::from(n)) as u64;
        let (mut y,mut extent,mut product,mut divisor,mut x,mut previous)=(2,1usize,1,1,0,0);
        while divisor==1 {
            x=y;
            for _ in 0..extent {y=next(y);}
            let mut visited=0;
            while visited<extent && divisor==1 {
                previous=y;
                let work=batch.min(extent-visited);
                for _ in 0..work {
                    y=next(y);product=multiply(product,x.abs_diff(y),n);
                }
                divisor=gcd(product,n);visited+=work;
            }
            let Some(expanded)=extent.checked_mul(2) else {break;};
            extent=expanded;
        }
        if divisor==n {
            loop {
                previous=next(previous);divisor=gcd(x.abs_diff(previous),n);
                if divisor>1 || previous==x {break;}
            }
        }
        if divisor>1 && divisor<n {return divisor;}
    }
}

pub(super) fn factors(source:&[char])->Vec<Tape> {
    if source.len()>u64::BITS as usize {return vec![source.to_vec()];}
    let mut value=0u64;
    for &mark in source.iter().rev() {value=(value<<1)|u64::from(mark=='⊥');}
    assert!(value>0);
    let mut pending=vec![value];let mut out=Vec::new();
    while let Some(value)=pending.pop() {
        if value==1 {continue;}
        if prime(value) {out.push(mf::tape_u64(value));}
        else {
            let divisor=split(value);
            assert_eq!(value%divisor,0);
            pending.push(divisor);pending.push(value/divisor);
        }
    }
    out.sort_unstable_by(|a,b|mf::cmp(a,b));
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn residual_views_reconstruct_all_words_of_the_random_2048_source() {
        let source=mf::parse_numeral(include_str!("../tests/fixtures/random_rsa_2048.imasm").trim()).unwrap();
        for chunk in source.chunks(u64::BITS as usize) {
            let source=mf::trim(chunk.to_vec());
            let views=factors(&source);
            let product=views.iter().fold(mf::one(),|value,factor|mf::mul(&value,factor));
            assert_eq!(product,source);
            for factor in views {assert!(mf::miller_rabin(&factor));}
        }
        assert_eq!(factors(&source),vec![source]);
    }
}
