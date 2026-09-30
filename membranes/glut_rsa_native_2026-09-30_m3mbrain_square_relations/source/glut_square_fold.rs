//! Structural square-congruence phase closure nested in the GLUT membrane.
use super::{mf, trim, ONE, ZERO};
use alloc::vec::Vec;
#[path = "glut_square_phase.rs"]
mod phase;
pub(super) enum Step { Running, Closed(Vec<char>,Vec<char>), Empty }
pub(super) struct SquareFold { phase: phase::SquarePhase, pub advances: usize }
impl SquareFold {
    pub fn new(n: &[char],root: &[char]) -> Self {Self {phase:phase::SquarePhase::new(n,root),advances:0}}
    pub fn cells(&self) -> usize {self.phase.cells()}
    pub fn advance(&mut self) -> Step {
        self.advances+=1;
        if let Some((mut p,mut q))=self.phase.advance() {
            if mf::cmp(&p,&q)==core::cmp::Ordering::Greater {core::mem::swap(&mut p,&mut q);}
            assert!(crate::trace_algebra::witness_valid(self.phase.source(),&p,&q));
            Step::Closed(p,q)
        } else {Step::Running}
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
