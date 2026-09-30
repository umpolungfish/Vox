//! Correlated sum/difference support for the same odd product relation.
//! The midpoint and half-gap are numeral tapes; adjacent midpoint advances
//! update the residual square by its exact carry difference.
use super::{bit, mf, trim, ONE, ZERO};
use alloc::{vec, vec::Vec};
pub(super) enum Step {
    Running,
    Closed(Vec<char>, Vec<char>),
    Empty,
}
pub(super) struct SquareFold {
    n: Vec<char>,
    midpoint: Vec<char>,
    residual: Vec<char>,
    root: Vec<char>,
    root_square: Vec<char>,
    unit_midpoint: Vec<char>,
    pub advances: usize,
}
impl SquareFold {
    pub fn new(n: &[char], root: &[char]) -> Self {
        let mut midpoint = root.to_vec();
        if mf::mul(root, root) != n {
            midpoint = mf::add(&midpoint, &[ONE]);
        }
        // Odd squares are one modulo four: the midpoint parity follows N.
        if bit(&midpoint, 0) != (1 ^ bit(n, 1)) {
            midpoint = mf::add(&midpoint, &[ONE]);
        }
        let residual = mf::sub(&mf::mul(&midpoint, &midpoint), n);
        let unit_midpoint = trim(mf::add(n, &[ONE])[1..].to_vec());
        Self {
            n: n.to_vec(),
            midpoint,
            residual,
            root: vec![ZERO],
            root_square: vec![ZERO],
            unit_midpoint,
            advances: 0,
        }
    }
    pub fn cells(&self) -> usize {
        self.midpoint.len() + self.residual.len() + self.root.len() + self.root_square.len()
    }
    fn square_possible(&self) -> bool {
        let Some(valuation) = self.residual.iter().position(|&mark| mark == ONE) else {
            return true;
        };
        // A square has even valuation and its odd residue is one modulo eight.
        valuation % 2 == 0
            && bit(&self.residual, valuation + 1) == 0
            && bit(&self.residual, valuation + 2) == 0
    }
    fn root_at_residual(&mut self) {
        if self.residual == vec![ZERO] {
            self.root = vec![ZERO];
            self.root_square = vec![ZERO];
            return;
        }
        let mut denominator = vec![ONE];
        denominator.extend_from_slice(&self.root);
        let delta = mf::sub(&self.residual, &self.root_square);
        let (mut rise, remainder) = mf::divmod(&delta, &denominator);
        if remainder != vec![ZERO] {
            rise = mf::add(&rise, &[ONE]);
        }
        let mut upper = mf::add(&self.root, &rise);
        // The adjacent residual rise supplies an upper root bound. Newton's
        // descending tape fold returns its floor without a root-step quota.
        loop {
            let quotient = mf::divmod(&self.residual, &upper).0;
            let next = trim(mf::add(&upper, &quotient)[1..].to_vec());
            if mf::cmp(&next, &upper) != core::cmp::Ordering::Less {
                break;
            }
            upper = next;
        }
        self.root = upper;
        self.root_square = mf::mul(&self.root, &self.root);
    }
    pub fn advance(&mut self) -> Step {
        if mf::cmp(&self.midpoint, &self.unit_midpoint) != core::cmp::Ordering::Less {
            return Step::Empty;
        }
        self.advances += 1;
        if self.square_possible() {
            self.root_at_residual();
            if self.root_square == self.residual {
                let p = mf::sub(&self.midpoint, &self.root);
                let q = mf::add(&self.midpoint, &self.root);
                assert!(crate::trace_algebra::witness_valid(&self.n, &p, &q));
                return Step::Closed(p, q);
            }
        }
        // (a+2)^2-a^2=4(a+1), with both carries transported on tapes.
        let mut rise = vec![ZERO, ZERO];
        rise.extend(mf::add(&self.midpoint, &[ONE]));
        self.residual = mf::add(&self.residual, &rise);
        self.midpoint = mf::add(&self.midpoint, &[ZERO, ONE]);
        Step::Running
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn correlated_square_support_matches_small_odd_product_census() {
        for n in (1..=511u64).step_by(2) {
            let tape = mf::tape_u64(n);
            let mut fold = SquareFold::new(&tape, &mf::isqrt(&tape));
            let expected = (3..=n).any(|p| p * p <= n && n % p == 0);
            loop {
                match fold.advance() {
                    Step::Running => {}
                    Step::Empty => {
                        assert!(!expected, "n={n}");
                        break;
                    }
                    Step::Closed(p, q) => {
                        assert!(expected);
                        assert!(crate::trace_algebra::witness_valid(&tape, &p, &q));
                        break;
                    }
                }
            }
        }
    }
}
