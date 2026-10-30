//! Structural square-congruence phase closure nested in the GLUT membrane.
use super::{mf, ONE, ZERO};
use alloc::vec::Vec;
#[path = "glut_square_phase.rs"]
mod phase;
pub(super) enum Step {
    Running,
    Divisor(Vec<char>),
    Empty,
}
pub(super) struct SquareFold {
    phase: phase::SquarePhase,
    pub advances: usize,
}
impl SquareFold {
    pub fn bind_root_frame(&mut self,p:crate::belnap_word::BelnapWord,q:crate::belnap_word::BelnapWord,
        axes:[usize;12],clock:crate::belnap_word::BelnapWord,evidence:crate::belnap::B4)->Result<(),alloc::string::String>{
        self.phase.bind_root_frame(p,q,axes,clock,evidence)
    }
    pub fn polynomial_actions(&self)->Vec<serde_json::Value>{self.phase.polynomial_actions()}
    pub fn bind_torus_winding(&mut self, step: usize) -> Result<(), alloc::string::String> {
        self.phase.bind_torus_winding(step)
    }
    pub fn torus_observations(&self) -> &[crate::glut_system::SquareTorusObservation] {
        self.phase.torus_observations()
    }
    pub fn new(n: &[char], root: &[char]) -> Self {
        Self {
            phase: phase::SquarePhase::new(n, root),
            advances: 0,
        }
    }
    pub fn cells(&self) -> usize {
        self.phase.cells()
    }
    pub fn work_extent(&self) -> usize {
        self.phase.work_extent()
    }
    pub fn folded_extent(&self) -> usize {
        self.phase.folded_extent()
    }
    #[cfg(feature = "root-work-profile")]
    pub fn root_work(&self) -> (usize, usize, usize, usize) {
        self.phase.root_work()
    }
    pub fn advance_observed(&mut self, observe: impl FnMut(char, usize, usize, usize)) -> Step {
        if mf::cmp(self.phase.source(), &[ONE]) != core::cmp::Ordering::Greater {
            return Step::Empty;
        }
        self.advances += 1;
        if let Some(divisor) = self.phase.advance(observe) {
            assert!(
                mf::cmp(&divisor, &[ONE]).is_gt()
                    && mf::cmp(&divisor, self.phase.source()).is_lt(),
                "square phase returned an improper divisor"
            );
            Step::Divisor(divisor)
        } else {
            Step::Running
        }
    }
}
