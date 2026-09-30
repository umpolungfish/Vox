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
    pub fn advance(&mut self) -> Step {self.advance_observed(|_,_,_,_|{})}
    pub fn advance_observed(&mut self, observe: impl FnMut(char,usize,usize,usize)) -> Step {
        if mf::cmp(self.phase.source(), &[ONE]) != core::cmp::Ordering::Greater {return Step::Empty;}
        self.advances+=1;
        if let Some((mut p,mut q))=self.phase.advance(observe) {
            if mf::cmp(&p,&q)==core::cmp::Ordering::Greater {core::mem::swap(&mut p,&mut q);}
            assert!(crate::trace_algebra::witness_valid(self.phase.source(),&p,&q));
            Step::Closed(p,q)
        } else {Step::Running}
    }
}
