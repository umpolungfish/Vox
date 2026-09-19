//! Native restored-support provenance for nested frames.
//!
//! A deposit schedule is ordered outermost-first. Each frame carries a lane-support
//! bitmask. Restoration pops frames innermost-first; the support restored across a
//! boundary is the union accumulated so far. Reading those restored supports back
//! outermost-first gives the suffix envelope used by the exact fibre theorems.
//!
//! This module is no_std + alloc and contains the machine-side seam only. Counting
//! theorems live above it; they consume this ladder rather than reimplementing the
//! restoration rule.

use alloc::vec::Vec;

pub type LaneSupport = u32;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct RestoredSupport {
    /// Deposits as entered, outermost frame first.
    pub deposits: Vec<LaneSupport>,
    /// Support visible when each frame is restored, outermost frame first.
    pub ladder: Vec<LaneSupport>,
}

/// Resident frame-restoration machine. Frames are entered outermost-first and
/// restored innermost-first. Every restoration unions that frame's deposit into
/// the support carried outward.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct FrameRestoreMachine {
    deposits: Vec<LaneSupport>,
    next: usize,
    restored: LaneSupport,
    ladder: Vec<LaneSupport>,
}

impl FrameRestoreMachine {
    pub fn new(deposits: &[LaneSupport]) -> Self {
        Self {
            deposits: deposits.to_vec(),
            next: deposits.len(),
            restored: 0,
            ladder: vec![0; deposits.len()],
        }
    }

    /// Restore one frame. The returned pair is `(outer_index, restored_support)`.
    pub fn restore_one(&mut self) -> Option<(usize, LaneSupport)> {
        if self.next == 0 {
            return None;
        }
        self.next -= 1;
        self.restored |= self.deposits[self.next];
        self.ladder[self.next] = self.restored;
        Some((self.next, self.restored))
    }

    pub fn run(mut self) -> RestoredSupport {
        while self.restore_one().is_some() {}
        RestoredSupport {
            deposits: self.deposits,
            ladder: self.ladder,
        }
    }
}

/// Execute the frame-restoration machine and retain both the original deposits
/// and the restored-support ladder.
pub fn restored_support(deposits: &[LaneSupport]) -> RestoredSupport {
    FrameRestoreMachine::new(deposits).run()
}

/// The native suffix envelope: support restored at each frame boundary,
/// outermost-first.
pub fn restored_support_ladder(deposits: &[LaneSupport]) -> Vec<LaneSupport> {
    restored_support(deposits).ladder
}

/// Every restored-support ladder is descending under set inclusion.
pub fn is_descending(ladder: &[LaneSupport]) -> bool {
    ladder.windows(2).all(|w| w[0] & w[1] == w[1])
}

/// Exact fibre size for a valid restored-support ladder:
///
/// `|fibre(G)| = 2^(sum_{i>=1} popcount(G_i))`.
///
/// Returns `None` for a non-descending ladder or when the exact count does not fit
/// in `u128`; the machine ladder itself is not width-limited by this counting view.
pub fn suffix_fibre_size(ladder: &[LaneSupport]) -> Option<u128> {
    if !is_descending(ladder) {
        return None;
    }
    let exponent = ladder
        .iter()
        .skip(1)
        .try_fold(0u32, |acc, x| acc.checked_add(x.count_ones()))?;
    1u128.checked_shl(exponent)
}

/// Union visible at the outer boundary after all nested frames have restored.
pub fn restored_union(ladder: &[LaneSupport]) -> LaneSupport {
    ladder.first().copied().unwrap_or(0)
}
