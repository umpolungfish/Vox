//! Residual factor views inside the resident square-phase membrane.
//! The whole tape and its residual views use the same Belnap-cell arithmetic.
//! The multiplicative orbit retains every exact factor in its containing frame.
use crate::morphism_factor as mf;
use alloc::{vec, vec::Vec};
type Tape = Vec<char>;

// The residual view uses the same source-bound Belnap arithmetic as its
// containing phase. Loop extents are structural addresses, not numeral cells.
fn split(n: &[char]) -> Tape {
    if n.first()==Some(&'⊤') { return mf::two(); }
    let batch=n.len();
    let mut seed=mf::one();
    loop {
        let c=mf::modulo(&seed,n);
        seed=mf::add(&seed,&mf::one());
        let next=|x:&[char]|mf::mul_mod_add(x,x,&c,n);
        let (mut y,mut extent,mut product,mut divisor,mut x,mut previous)=
            (mf::two(),1usize,mf::one(),mf::one(),vec!['⊤'],vec!['⊤']);
        while divisor==mf::one() {
            x=y.clone();
            for _ in 0..extent { y=next(&y); }
            let mut visited=0;
            while visited<extent && divisor==mf::one() {
                previous=y.clone();
                let work=batch.min(extent-visited);
                for _ in 0..work {
                    y=next(&y);
                    let difference=if mf::cmp(&x,&y).is_lt(){mf::sub(&y,&x)}else{mf::sub(&x,&y)};
                    product=mf::mul_mod(&product,&difference,n);
                }
                divisor=mf::gcd(product.clone(),n.to_vec());
                visited+=work;
            }
            let Some(expanded)=extent.checked_mul(2) else {break;};
            extent=expanded;
        }
        if divisor==n {
            loop {
                previous=next(&previous);
                let difference=if mf::cmp(&x,&previous).is_lt(){mf::sub(&previous,&x)}else{mf::sub(&x,&previous)};
                divisor=mf::gcd(difference,n.to_vec());
                if mf::cmp(&divisor,&mf::one()).is_gt() || previous==x {break;}
            }
        }
        if mf::cmp(&divisor,&mf::one()).is_gt() && mf::cmp(&divisor,n).is_lt() {return divisor;}
    }
}

pub(super) fn factors(source: &[char]) -> Vec<Tape> {
    if source.len()>u64::BITS as usize {return vec![source.to_vec()];}
    assert!(!mf::zero(source));
    let mut pending=vec![source.to_vec()];
    let mut out=Vec::new();
    while let Some(value)=pending.pop() {
        if value==mf::one() {continue;}
        if mf::miller_rabin(&value) {out.push(value);}
        else {
            let divisor=split(&value);
            let (quotient,remainder)=mf::divmod(&value,&divisor);
            assert!(mf::zero(&remainder));
            pending.push(divisor);pending.push(quotient);
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
        let source =
            mf::parse_numeral(include_str!("../tests/fixtures/random_rsa_2048.imasm").trim())
                .unwrap();
        for chunk in source.chunks(u64::BITS as usize) {
            let source = mf::trim(chunk.to_vec());
            let views = factors(&source);
            let product = views
                .iter()
                .fold(mf::one(), |value, factor| mf::mul(&value, factor));
            assert_eq!(product, source);
            for factor in views {
                assert!(mf::miller_rabin(&factor));
            }
        }
        assert_eq!(factors(&source), vec![source]);
    }
}
