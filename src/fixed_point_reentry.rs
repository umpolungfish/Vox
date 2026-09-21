//! Explicit one-step re-entry for the fixed-point spectral tower.
//!
//! The repaired boundary terminates at V3 with winding 4.  The source grammar
//! defines TANCH as a self-referential closure whose scheduler re-enters the
//! scale recursion V_n -> V_{n+1}: AFWD consumes one re-entry marker, CLINK
//! applies the frame collapse, and IFIX permanently records the next integer
//! winding.  This module makes exactly one such re-entry executable at a time.
//!
//! Nothing here searches for a multiplicative order.  `reenter_once` advances
//! one scale because the caller explicitly requested one scale.  The instant
//! non-walking read in `fixed_point_protocol` does not call this method and does
//! not loop over this tower looking for a closing modular phase.

use alloc::vec::Vec;

use crate::fixed_point_protocol::FixedWindingDeposit;
use crate::hadamard_gate::{FixedPointSpectralConstruction, Tape};
use crate::morphism_factor::{add, cmp, one, sub};
use crate::vox::{AFWD, CLINK, ENGAGR, IFIX, IMSCRIB, TANCH};

/// The scale-preserving tail re-entered after the terminal fixed-point anchor.
/// It is the V3 tail generalized by the source scheduler to V_n -> V_{n+1}.
pub const FIXED_POINT_REENTRY_WORD: [char; 6] = [AFWD, CLINK, IMSCRIB, ENGAGR, IFIX, TANCH];

/// One anchored scale of the re-entry tower.
///
/// `scale` is the index n in V_n and `fixed.winding` is n+1.  Both remain
/// arbitrary-width numeral tapes, so re-entry does not introduce a host-word
/// bound into the spectral construction.
#[derive(Clone, PartialEq, Debug)]
pub struct FixedPointReentryState {
    scale: Tape,
    fixed: FixedWindingDeposit,
}

impl FixedPointReentryState {
    pub fn scale(&self) -> &[char] {
        &self.scale
    }

    pub fn winding(&self) -> &[char] {
        &self.fixed.winding
    }

    pub fn modular_phase(&self) -> &[char] {
        &self.fixed.modular_phase
    }

    pub fn closes_modular_phase(&self) -> bool {
        self.fixed.closes_modular_phase
    }

    pub fn fixed(&self) -> &FixedWindingDeposit {
        &self.fixed
    }
}

impl FixedPointSpectralConstruction {
    /// Recover the terminal anchored state of the repaired boundary: V3, w=4.
    /// This executes the boundary once and takes its final IFIX deposit; it does
    /// not inspect later windings.
    pub fn reentry_anchor(&self) -> Result<FixedPointReentryState, &'static str> {
        let measurement = self.run_boundary_measurement()?;
        let fixed = measurement
            .fixed
            .last()
            .cloned()
            .ok_or("fixed-point boundary has no terminal IFIX")?;
        let scale = sub(&fixed.winding, &one());
        Ok(FixedPointReentryState { scale, fixed })
    }

    /// Execute exactly one scale recursion V_n -> V_{n+1}.
    ///
    /// The resulting winding is w+1 and its modular phase is evaluated directly
    /// by tape-native square-and-multiply.  There is deliberately no loop and no
    /// branch that keeps re-entering until a closing phase is found.
    pub fn reenter_once(
        &self,
        state: &FixedPointReentryState,
    ) -> Result<FixedPointReentryState, &'static str> {
        let expected_winding = add(&state.scale, &one());
        if cmp(&expected_winding, &state.fixed.winding) != core::cmp::Ordering::Equal {
            return Err("fixed-point re-entry scale/winding invariant is broken");
        }

        let scale = add(&state.scale, &one());
        let winding = add(&state.fixed.winding, &one());
        let modular_phase = self.modular_branch(&winding)?;
        let closes_modular_phase = cmp(&modular_phase, &one()) == core::cmp::Ordering::Equal;

        Ok(FixedPointReentryState {
            scale,
            fixed: FixedWindingDeposit {
                winding,
                modular_phase,
                closes_modular_phase,
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hadamard_gate::HadamardCarrier;
    use crate::morphism_factor::tape_u64;

    #[test]
    fn reentry_word_is_the_exact_scale_tail() {
        let expected: Vec<char> = "≻⋈⊙⊞⊡⊣".chars().collect();
        assert_eq!(FIXED_POINT_REENTRY_WORD.as_slice(), expected.as_slice());
    }

    #[test]
    fn repaired_boundary_anchors_reentry_at_v3_winding_four() {
        let spectral = HadamardCarrier::new(&tape_u64(15))
            .unwrap()
            .fixed_point_spectral_construction()
            .unwrap();
        let anchor = spectral.reentry_anchor().unwrap();

        assert_eq!(anchor.scale(), tape_u64(3).as_slice());
        assert_eq!(anchor.winding(), tape_u64(4).as_slice());
        assert_eq!(anchor.modular_phase(), tape_u64(1).as_slice());
        assert!(anchor.closes_modular_phase());
    }

    #[test]
    fn one_reentry_advances_exactly_one_scale_and_one_winding() {
        let spectral = HadamardCarrier::new(&tape_u64(21))
            .unwrap()
            .fixed_point_spectral_construction()
            .unwrap();
        let anchor = spectral.reentry_anchor().unwrap();
        let next = spectral.reenter_once(&anchor).unwrap();

        assert_eq!(anchor.scale(), tape_u64(3).as_slice());
        assert_eq!(anchor.winding(), tape_u64(4).as_slice());
        assert_eq!(anchor.modular_phase(), tape_u64(16).as_slice());
        assert_eq!(next.scale(), tape_u64(4).as_slice());
        assert_eq!(next.winding(), tape_u64(5).as_slice());
        assert_eq!(next.modular_phase(), tape_u64(11).as_slice());
        assert!(!next.closes_modular_phase());
    }

    #[test]
    fn explicit_reentry_does_not_become_a_hidden_order_walk() {
        // ord_21(2)=6.  Two explicit structural re-entries can expose winding 6,
        // but the instant read remains restricted to the repaired boundary's
        // own IFIX deposits at 2, 3, and 4 and therefore still returns None.
        let spectral = HadamardCarrier::new(&tape_u64(21))
            .unwrap()
            .fixed_point_spectral_construction()
            .unwrap();
        let v3 = spectral.reentry_anchor().unwrap();
        let v4 = spectral.reenter_once(&v3).unwrap();
        let v5 = spectral.reenter_once(&v4).unwrap();

        assert_eq!(v5.scale(), tape_u64(5).as_slice());
        assert_eq!(v5.winding(), tape_u64(6).as_slice());
        assert_eq!(v5.modular_phase(), tape_u64(1).as_slice());
        assert!(v5.closes_modular_phase());
        assert_eq!(spectral.instant_non_walking_read().unwrap(), None);
    }
}
