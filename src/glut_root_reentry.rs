//! Prime-power fibers of the resident square polynomial, before contraction.
use crate::morphism_factor as mf;
use alloc::vec::Vec;

pub(super) struct RootPolynomial {
    a: mf::BelnapResidue,
    b: mf::BelnapResidue,
    c: mf::BelnapResidue,
    negative: bool,
    half: usize,
}
impl RootPolynomial {
    pub(super) fn new(a: &[char], b: &[char], n: &[char], half: usize) -> Self {
        let square = mf::mul(b, b);
        let negative = mf::cmp(&square, n).is_lt();
        let delta = if negative { mf::sub(n, &square) } else { mf::sub(&square, n) };
        let (c, remainder) = mf::divmod(&delta, a);
        assert!(mf::zero(&remainder), "root frame must retain B²=N mod A");
        Self { a: mf::BelnapResidue::new(a), b: mf::BelnapResidue::new(b),
            c: mf::BelnapResidue::new(&c), negative, half }
    }
    /// Q(i)=A(i-half)²+2B(i-half)+(B²-N)/A, reduced exactly.
    fn residue(&self, i: usize, modulus: u64) -> u64 {
        let t=mf::residue_sub(i as u64,self.half as u64,modulus);
        let a=self.a.remainder_word(modulus);
        let b=self.b.remainder_word(modulus);
        let c=self.c.remainder_word(modulus);
        let c=if self.negative {mf::residue_sub(0,c,modulus)}else{c};
        mf::residue_add(mf::residue_mul(mf::residue_add(mf::residue_mul(a,t,modulus),
            mf::residue_mul(2,b,modulus),modulus),t,modulus),c,modulus)
    }
    pub(super) fn linear_root(&self, prime: usize) -> usize {
        let p=prime as u64;
        assert_eq!(self.a.remainder_word(p),0);
        let derivative=mf::residue_mul(2,self.b.remainder_word(p),p);
        let inverse=mf::inverse_residue_word(derivative,p).expect("linear field root is invertible");
        let c=self.residue(self.half,p);
        let displacement=mf::residue_mul(mf::residue_sub(0,c,p),inverse,p);
        mf::residue_add(displacement,self.half as u64,p) as usize
    }
    pub(super) fn lift_roots(&self, prime: usize, roots: &[usize], span: usize)
        -> Vec<(usize, usize)> {
        let p = prime as u64;
        let mut modulus = p;
        let mut roots: Vec<usize> = roots.iter().copied().filter(|&r| r < span).collect();
        roots.sort_unstable();
        roots.dedup();
        let mut out = Vec::new();
        while !roots.is_empty() {
            let Some(next) = modulus.checked_mul(p).and_then(|m| usize::try_from(m).ok()) else { break; };
            let mut lifted = Vec::new();
            for &r in &roots {
                let value = self.residue(r, next as u64);
                assert_eq!(value % modulus, 0, "root re-entry lost its parent fiber");
                let quotient = value / modulus;
                let t=mf::residue_sub(r as u64,self.half as u64,p);
                let derivative=mf::residue_mul(2,mf::residue_add(
                    mf::residue_mul(self.a.remainder_word(p),t,p),self.b.remainder_word(p),p),p);
                if let Some(inverse) = mf::inverse_residue_word(derivative, p) {
                    let digit=mf::residue_mul(mf::residue_sub(0,quotient,p),inverse,p);
                    let child = r as u128 + u128::from(digit) * u128::from(modulus);
                    if child < span as u128 { lifted.push(child as usize); }
                } else if quotient == 0 {
                    // The singular binary fiber can branch. Retain every
                    // child in the active window, including both at p=2.
                    let count = (span - 1 - r) as u128 / u128::from(modulus) + 1;
                    for digit in 0..count.min(p as u128) {
                        lifted.push((r as u128 + digit * u128::from(modulus)) as usize);
                    }
                }
            }
            lifted.sort_unstable();
            lifted.dedup();
            for &r in &lifted { out.push((next, r)); }
            roots = lifted;
            modulus = next as u64;
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn prime_power_marks_match_exact_division_on_200_bit_source() {
        let n = mf::decimal_to_tape("1306593903193228327879866132486971792765599839798626833574803").unwrap();
        assert_eq!(n.len(), 200);
        // A=1 is the enclosing square chart, B=ceil(sqrt(N)).
        let a = mf::one();
        let b = mf::add(&mf::isqrt(&n), &mf::one());
        let half = 128;
        let span = 256;
        let poly = RootPolynomial::new(&a, &b, &n, half);
        let mut repeated = 0;
        for p in [2usize, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31] {
            let roots: Vec<_> = (0..p).filter(|&r| poly.residue(r, p as u64) == 0).collect();
            let lifts = poly.lift_roots(p, &roots, span);
            repeated += lifts.len();
            for index in 0..span {
                let distance = mf::tape_u64(index.abs_diff(half) as u64);
                let x = if index >= half { mf::add(&b, &distance) } else { mf::sub(&b, &distance) };
                let square = mf::mul(&x, &x);
                let delta = if mf::cmp(&square, &n).is_lt() { mf::sub(&n, &square) } else { mf::sub(&square, &n) };
                let folded = mf::BelnapResidue::new(&delta);
                let mut power = p as u64;
                while let Some(next) = power.checked_mul(p as u64) {
                    let marked = lifts.iter().any(|&(m, r)| m == next as usize && index >= r && (index-r) % m == 0);
                    assert_eq!(marked, folded.remainder_word(next) == 0, "prime={p} power={next} index={index}");
                    power = next;
                }
            }
        }
        assert!(repeated > 0);
    }

    #[test]
    fn linear_roots_preserve_both_signed_200_bit_square_charts() {
        let n = mf::decimal_to_tape("1306593903193228327879866132486971792765599839798626833574803").unwrap();
        assert_eq!(n.len(), 200);
        let p = [3usize, 5, 7, 11, 13, 17, 19, 23, 29, 31].into_iter()
            .find(|&p| (1..p).any(|r| (r*r) as u64 % p as u64 == mf::BelnapResidue::new(&n).remainder_word(p as u64)))
            .unwrap();
        let a = mf::tape_u64(p as u64);
        let n_residue = mf::BelnapResidue::new(&n).remainder_word(p as u64);
        let r = (1..p).find(|r| (r*r) as u64 % p as u64 == n_residue).unwrap();
        let low = mf::tape_u64(r as u64);
        let root = mf::isqrt(&n);
        let base = mf::mul(&mf::divmod(&root, &a).0, &a);
        let high = mf::add(&mf::add(&base, &a), &low);
        for b in [low, high] {
            let poly = RootPolynomial::new(&a, &b, &n, 128);
            let linear = poly.linear_root(p);
            assert_eq!(poly.residue(linear, p as u64), 0);
            let lifts = poly.lift_roots(p, &[linear], 256);
            for i in 0..256 {
                assert_eq!((i + p - linear) % p == 0, poly.residue(i, p as u64) == 0);
                for &(modulus, position) in &lifts {
                    if i >= position && (i - position) % modulus == 0 {
                        assert_eq!(poly.residue(i, modulus as u64), 0);
                    }
                }
            }
        }
    }
}
