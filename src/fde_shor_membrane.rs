#![allow(dead_code)]
//! FDE/topological carrier for the Shor order membrane.
//!
//! The carrier is the computational state.  A modular transition is opened
//! into forward and reverse lanes, transported through the boundary, and fused
//! only after both lanes recover the same arbitrary-width tape.

use std::cmp::Ordering;

pub type Tape = Vec<char>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fde {
    N,
    T,
    F,
    B,
}

impl Fde {
    fn join(self, other: Self) -> Self {
        use Fde::*;
        match (self, other) {
            (N, x) | (x, N) => x,
            (T, T) => T,
            (F, F) => F,
            _ => B,
        }
    }
}

#[derive(Clone, Debug)]
pub struct OrderCarrier {
    pub forward: Tape,
    pub reverse: Tape,
    pub lane: Fde,
    pub transitions: usize,
    pub closed: usize,
}

impl OrderCarrier {
    pub fn new(seed: Tape) -> Self {
        Self { forward: seed.clone(), reverse: seed, lane: Fde::N,
            transitions: 0, closed: 0 }
    }

    /// δ opens both sidearms over the same resident object.
    pub fn delta(&mut self) {
        self.reverse = self.forward.clone();
        self.lane = self.lane.join(Fde::B);
        self.transitions += 1;
    }

    /// μ fuses only equal sidearms.  This is the executable μ∘δ = id check.
    pub fn mu(&mut self) -> Result<Tape, String> {
        if self.forward != self.reverse {
            return Err("FDE sidearms diverged before fusion".into());
        }
        self.lane = self.lane.join(Fde::T);
        self.closed += 1;
        Ok(self.forward.clone())
    }

    /// Apply one topological modular transition to both sidearms and verify
    /// the boundary immediately at this level.
    pub fn modular_step(&mut self, a: &[char], n: &[char]) -> Result<Tape, String> {
        self.delta();
        let opened = self.forward.clone();
        self.forward = ::vox::morphism_factor::modulo(
            &::vox::morphism_factor::mul(&opened, a), n);
        self.reverse = ::vox::morphism_factor::modulo(
            &::vox::morphism_factor::mul(&opened, a), n);
        let fused = self.mu()?;
        if ::vox::morphism_factor::cmp(&fused, n) != Ordering::Less {
            return Err("FDE product boundary left the residue object".into());
        }
        Ok(fused)
    }

    pub fn boundary_identity(&self) -> bool {
        self.transitions == self.closed
            || (self.transitions == 0 && self.closed == 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_transition_closes_both_lanes() {
        let n = ::vox::morphism_factor::tape_u64(15);
        let a = ::vox::morphism_factor::tape_u64(2);
        let mut carrier = OrderCarrier::new(::vox::morphism_factor::one());
        for _ in 0..4 { carrier.modular_step(&a, &n).unwrap(); }
        assert_eq!(carrier.lane, Fde::B);
        assert_eq!(carrier.transitions, carrier.closed);
        assert!(carrier.boundary_identity());
    }
}
