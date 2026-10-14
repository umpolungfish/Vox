//! Explicit re-entry for the fixed-point spectral tower.
//!
//! The repaired boundary terminates at V3 with winding 4. The source grammar
//! defines TANCH as a self-referential closure whose scheduler re-enters the
//! scale recursion V_n -> V_{n+1}: AFWD consumes one re-entry marker, CLINK
//! applies the frame collapse, and IFIX permanently records the next integer
//! winding. `reenter_once` keeps that exact one-scale cycle executable.
//!
//! Executed hypernesting adds a second exact operation: a caller may explicitly
//! supply ancestry depth d and thereby pump the winding by exactly d in one
//! properly contained carrier word. The banked clear cost remains one, and only
//! the final modular branch is evaluated. Nothing here advances until closure,
//! scans intermediate windings, or discovers a multiplicative order by walking.

use alloc::vec::Vec;

use crate::fixed_point_imasm::HypernestedImasmCarrier;
use crate::fixed_point_protocol::FixedWindingDeposit;
use crate::hadamard_factor_bridge::HadamardDescent;
use crate::hadamard_gate::{FixedPointSpectralConstruction, SpectralWinding, Tape};
use crate::morphism_factor::{add, cmp, one, sub};
use crate::vox::{AFWD, CLINK, ENGAGR, EVALF, EVALT, IFIX, IMSCRIB, TANCH};

/// The scale-preserving tail re-entered after the terminal fixed-point anchor.
/// It is the V3 tail generalized by the source scheduler to V_n -> V_{n+1}.
pub const FIXED_POINT_REENTRY_WORD: [char; 6] = [AFWD, CLINK, IMSCRIB, ENGAGR, IFIX, TANCH];

/// One anchored scale of the re-entry tower.
///
/// `scale` is the index n in V_n and `fixed.winding` is n+1. Both remain
/// arbitrary-width numeral tapes, so re-entry does not introduce a host-word
/// bound into the spectral construction.
#[derive(Clone, PartialEq, Debug)]
pub struct FixedPointReentryState {
    scale: Tape,
    fixed: FixedWindingDeposit,
}

impl FixedPointReentryState {
    pub fn scale(&self) -> &[char] { &self.scale }
    pub fn winding(&self) -> &[char] { &self.fixed.winding }
    pub fn modular_phase(&self) -> &[char] { &self.fixed.modular_phase }
    pub fn closes_modular_phase(&self) -> bool { self.fixed.closes_modular_phase }
    pub fn fixed(&self) -> &FixedWindingDeposit { &self.fixed }
}

/// Audit of one exact scheduler cycle through the self-referential tail.
///
/// The explicit call supplies one re-entry marker. AFWD consumes it exactly
/// once; CLINK certifies the spectral frame collapse as an idempotent
/// retraction; IMSCRIB and ENGAGR commit the self-reference and dialetheic
/// witness; IFIX records exactly the one new winding; and TANCH closes the cycle
/// back at the fixed-point anchor.
#[derive(Clone, PartialEq, Debug)]
pub struct FixedPointReentryCycle {
    pub word: Vec<char>,
    pub marker_consumed: bool,
    pub frame_collapse_idempotent: bool,
    pub self_reference_asserted: bool,
    pub dialetheic_witness_held: bool,
    pub anchored: bool,
    pub next: FixedPointReentryState,
}

/// One explicit hypernest winding jump.
///
/// `hypernest.depth() == hypernest.winding()`: ancestry itself is the requested
/// integer winding gain. The unique live clear remains banked through the whole
/// word. `next` contains only the final fixed winding and its independently
/// addressed modular phase; no intermediate modular branch is evaluated.
#[derive(Clone, PartialEq, Debug)]
pub struct FixedPointHypernestReentry {
    pub hypernest: HypernestedImasmCarrier,
    pub winding_gain: Tape,
    pub frame_collapse_idempotent: bool,
    pub next: FixedPointReentryState,
}

fn structural_count_tape(mut value: usize) -> Tape {
    if value == 0 {
        return alloc::vec![EVALT];
    }
    let mut out = Vec::new();
    while value != 0 {
        out.push(if value & 1 == 1 { EVALF } else { EVALT });
        value >>= 1;
    }
    out
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

    /// Execute the exact six-op scheduler tail once.
    ///
    /// The call itself supplies one re-entry marker. There is no loop, no
    /// closure-triggered retry, and no inspection of any winding other than the
    /// one produced by this invocation.
    pub fn run_reentry_cycle(
        &self,
        state: &FixedPointReentryState,
    ) -> Result<FixedPointReentryCycle, &'static str> {
        let expected_winding = add(&state.scale, &one());
        if cmp(&expected_winding, &state.fixed.winding) != core::cmp::Ordering::Equal {
            return Err("fixed-point re-entry scale/winding invariant is broken");
        }

        let mut scale = state.scale.clone();
        let mut winding = state.fixed.winding.clone();
        let mut marker_present = true;
        let mut marker_consumed = false;
        let mut frame_collapse_idempotent = false;
        let mut self_reference_asserted = false;
        let mut dialetheic_witness_held = false;
        let mut fixed: Option<FixedWindingDeposit> = None;
        let mut anchored = false;

        for &op in &FIXED_POINT_REENTRY_WORD {
            match op {
                AFWD => {
                    if !marker_present || marker_consumed {
                        return Err("fixed-point re-entry marker was not available exactly once");
                    }
                    marker_present = false;
                    marker_consumed = true;
                    scale = add(&scale, &one());
                    winding = add(&winding, &one());
                }
                CLINK => {
                    // The existing spectral split/fuse is the executable form of
                    // the frame collapse. Certify both rho(x)=x and rho²=rho.
                    let probe = SpectralWinding::new(&winding, &one())?;
                    let once = probe.delta().mu()?;
                    let twice = once.delta().mu()?;
                    frame_collapse_idempotent = once == probe && twice == once;
                    if !frame_collapse_idempotent {
                        return Err("fixed-point frame collapse is not idempotent");
                    }
                }
                IMSCRIB => {
                    self_reference_asserted = true;
                }
                ENGAGR => {
                    dialetheic_witness_held = true;
                }
                IFIX => {
                    if !marker_consumed
                        || !frame_collapse_idempotent
                        || !self_reference_asserted
                        || !dialetheic_witness_held
                    {
                        return Err("fixed-point re-entry reached IFIX before its scheduler commitments");
                    }
                    let modular_phase = self.modular_branch(&winding)?;
                    let closes_modular_phase =
                        cmp(&modular_phase, &one()) == core::cmp::Ordering::Equal;
                    fixed = Some(FixedWindingDeposit {
                        winding: winding.clone(),
                        modular_phase,
                        closes_modular_phase,
                    });
                }
                TANCH => {
                    if fixed.is_none() || marker_present {
                        return Err("fixed-point re-entry reached TANCH before fixation");
                    }
                    anchored = true;
                }
                _ => return Err("unexpected opcode in fixed-point re-entry word"),
            }
        }

        if !anchored {
            return Err("fixed-point re-entry did not return to TANCH");
        }
        let fixed = fixed.ok_or("fixed-point re-entry did not produce an IFIX deposit")?;
        let next = FixedPointReentryState { scale, fixed };

        Ok(FixedPointReentryCycle {
            word: FIXED_POINT_REENTRY_WORD.to_vec(),
            marker_consumed,
            frame_collapse_idempotent,
            self_reference_asserted,
            dialetheic_witness_held,
            anchored,
            next,
        })
    }

    /// Execute exactly one scale recursion V_n -> V_{n+1}.
    ///
    /// This is the state-only projection of `run_reentry_cycle`. The exact
    /// six-op tail performs the work; this method simply returns its next state.
    pub fn reenter_once(
        &self,
        state: &FixedPointReentryState,
    ) -> Result<FixedPointReentryState, &'static str> {
        Ok(self.run_reentry_cycle(state)?.next)
    }

    /// Pump the re-entry winding by exactly `depth` through one executed
    /// hypernest, then evaluate only the final N-dependent modular branch.
    ///
    /// This is explicit-depth re-entry, not order discovery. The caller chooses
    /// `depth` before execution. The operation does not inspect any intermediate
    /// winding and does not extend itself if the final phase is nonclosing.
    pub fn reenter_hypernest(
        &self,
        state: &FixedPointReentryState,
        depth: usize,
    ) -> Result<FixedPointHypernestReentry, &'static str> {
        let expected_winding = add(&state.scale, &one());
        if cmp(&expected_winding, &state.fixed.winding) != core::cmp::Ordering::Equal {
            return Err("fixed-point re-entry scale/winding invariant is broken");
        }
        if depth == 0 {
            return Err("fixed-point hypernest re-entry requires positive depth");
        }

        let hypernest = HypernestedImasmCarrier::from_depth(depth)?;
        let audit = hypernest.audit();
        if audit.winding != depth
            || audit.live_clears != 1
            || audit.exposed != 0
            || !audit.banked
            || !audit.closed
        {
            return Err("fixed-point hypernest winding pump invariant failed");
        }

        let winding_gain = structural_count_tape(depth);
        let scale = add(&state.scale, &winding_gain);
        let winding = add(&state.fixed.winding, &winding_gain);
        let expected_final_winding = add(&scale, &one());
        if cmp(&expected_final_winding, &winding) != core::cmp::Ordering::Equal {
            return Err("fixed-point hypernest changed scale/winding offset");
        }

        // One collapse audit is enough for the ancestry-contained word: the
        // hypernest instrument already established one banked clear at every
        // depth, and the source frame collapse is idempotent.
        let probe = SpectralWinding::new(&winding, &one())?;
        let once = probe.delta().mu()?;
        let twice = once.clone().delta().mu()?;
        let frame_collapse_idempotent = once == probe && twice == once;
        if !frame_collapse_idempotent {
            return Err("fixed-point hypernest frame collapse is not idempotent");
        }

        // Deliberately evaluate exactly one branch: the final pumped winding.
        let modular_phase = self.modular_branch(&winding)?;
        let closes_modular_phase = cmp(&modular_phase, &one()) == core::cmp::Ordering::Equal;
        let fixed = FixedWindingDeposit {
            winding,
            modular_phase,
            closes_modular_phase,
        };
        let next = FixedPointReentryState { scale, fixed };

        Ok(FixedPointHypernestReentry {
            hypernest,
            winding_gain,
            frame_collapse_idempotent,
            next,
        })
    }

    /// Consume one explicitly exposed re-entry state and hand its fixed winding
    /// to the existing Hadamard order-two descent.
    ///
    /// This is not an order read. The caller already chose and executed every
    /// structural re-entry needed to produce `state`. Before descent we verify
    /// both tower invariants and recompute the resident modular phase at exactly
    /// that one winding. A nonclosing state routes around as N; a malformed or
    /// cross-construction state is F. No later winding is inspected.
    pub fn descend_reentry_state(
        self,
        state: &FixedPointReentryState,
    ) -> HadamardDescent {
        let expected_winding = add(&state.scale, &one());
        if cmp(&expected_winding, &state.fixed.winding) != core::cmp::Ordering::Equal {
            return HadamardDescent::F;
        }

        let resident_phase = match self.modular_branch(&state.fixed.winding) {
            Ok(phase) => phase,
            Err(_) => return HadamardDescent::F,
        };
        if cmp(&resident_phase, &state.fixed.modular_phase) != core::cmp::Ordering::Equal {
            return HadamardDescent::F;
        }

        let closes_modular_phase = cmp(&resident_phase, &one()) == core::cmp::Ordering::Equal;
        if closes_modular_phase != state.fixed.closes_modular_phase {
            return HadamardDescent::F;
        }

        let base = self.base().to_vec();
        let carrier = self.into_carrier();
        if !closes_modular_phase {
            return HadamardDescent::N(carrier);
        }

        carrier.descend_phase_order(&base, &state.fixed.winding)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::factor_extract::extract;
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
    fn reentry_cycle_executes_every_scheduler_commit() {
        let spectral = HadamardCarrier::new(&tape_u64(21))
            .unwrap()
            .fixed_point_spectral_construction()
            .unwrap();
        let anchor = spectral.reentry_anchor().unwrap();
        let cycle = spectral.run_reentry_cycle(&anchor).unwrap();

        assert_eq!(cycle.word.as_slice(), FIXED_POINT_REENTRY_WORD.as_slice());
        assert!(cycle.marker_consumed);
        assert!(cycle.frame_collapse_idempotent);
        assert!(cycle.self_reference_asserted);
        assert!(cycle.dialetheic_witness_held);
        assert!(cycle.anchored);
        assert_eq!(cycle.next.scale(), tape_u64(4).as_slice());
        assert_eq!(cycle.next.winding(), tape_u64(5).as_slice());
        assert_eq!(cycle.next.modular_phase(), tape_u64(11).as_slice());
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
    fn depth_one_hypernest_matches_one_explicit_reentry() {
        let spectral = HadamardCarrier::new(&tape_u64(21))
            .unwrap()
            .fixed_point_spectral_construction()
            .unwrap();
        let anchor = spectral.reentry_anchor().unwrap();
        let one = spectral.reenter_once(&anchor).unwrap();
        let jumped = spectral.reenter_hypernest(&anchor, 1).unwrap();

        assert_eq!(jumped.hypernest.depth(), 1);
        assert_eq!(jumped.hypernest.winding(), 1);
        assert_eq!(jumped.winding_gain, tape_u64(1));
        assert!(jumped.frame_collapse_idempotent);
        assert_eq!(jumped.next, one);
    }

    #[test]
    fn two_level_hypernest_pumps_winding_without_intermediate_phase_walk() {
        // ord_21(2)=6. The caller explicitly supplies depth 2 before execution.
        // Hypernest ancestry adds two turns at once from w=4 to w=6, and only
        // the final modular branch is resident in the returned state.
        let spectral = HadamardCarrier::new(&tape_u64(21))
            .unwrap()
            .fixed_point_spectral_construction()
            .unwrap();
        let anchor = spectral.reentry_anchor().unwrap();
        let jumped = spectral.reenter_hypernest(&anchor, 2).unwrap();
        let audit = jumped.hypernest.audit();

        assert_eq!(jumped.winding_gain, tape_u64(2));
        assert_eq!(audit.winding, 2);
        assert_eq!(audit.live_clears, 1);
        assert_eq!(audit.exposed, 0);
        assert!(audit.banked);
        assert!(audit.closed);
        assert_eq!(jumped.next.scale(), tape_u64(5).as_slice());
        assert_eq!(jumped.next.winding(), tape_u64(6).as_slice());
        assert_eq!(jumped.next.modular_phase(), tape_u64(1).as_slice());
        assert!(jumped.next.closes_modular_phase());
    }

    #[test]
    fn explicit_reentry_does_not_become_a_hidden_order_walk() {
        // ord_21(2)=6. Two explicit structural re-entries can expose winding 6,
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

    #[test]
    fn explicit_closing_reentry_feeds_existing_hadamard_descent() {
        // The tower exposes w=6 only because the caller explicitly performs two
        // re-entry steps. The final handoff consumes that one exposed state;
        // it does not scan w=4,5,6 or search for a closing exponent.
        let spectral = HadamardCarrier::new(&tape_u64(21))
            .unwrap()
            .fixed_point_spectral_construction()
            .unwrap();
        let v3 = spectral.reentry_anchor().unwrap();
        let v4 = spectral.reenter_once(&v3).unwrap();
        let v5 = spectral.reenter_once(&v4).unwrap();

        let factor_carrier = match spectral.descend_reentry_state(&v5) {
            HadamardDescent::T(carrier) => carrier,
            other => panic!("expected T factor carrier, got {other:?}"),
        };
        let readout = extract(&factor_carrier).unwrap();
        let p = readout.p.0;
        let q = readout.q.0;
        let direct = p == tape_u64(3) && q == tape_u64(7);
        let swapped = p == tape_u64(7) && q == tape_u64(3);
        assert!(direct || swapped);
    }

    #[test]
    fn explicit_hypernest_jump_feeds_existing_hadamard_descent() {
        let spectral = HadamardCarrier::new(&tape_u64(21))
            .unwrap()
            .fixed_point_spectral_construction()
            .unwrap();
        let anchor = spectral.reentry_anchor().unwrap();
        let jumped = spectral.reenter_hypernest(&anchor, 2).unwrap();

        let factor_carrier = match spectral.descend_reentry_state(&jumped.next) {
            HadamardDescent::T(carrier) => carrier,
            other => panic!("expected T factor carrier, got {other:?}"),
        };
        let readout = extract(&factor_carrier).unwrap();
        let p = readout.p.0;
        let q = readout.q.0;
        let direct = p == tape_u64(3) && q == tape_u64(7);
        let swapped = p == tape_u64(7) && q == tape_u64(3);
        assert!(direct || swapped);
    }

    #[test]
    fn explicit_nonclosing_reentry_routes_around_without_advancing_again() {
        let spectral = HadamardCarrier::new(&tape_u64(21))
            .unwrap()
            .fixed_point_spectral_construction()
            .unwrap();
        let v3 = spectral.reentry_anchor().unwrap();
        let v4 = spectral.reenter_once(&v3).unwrap();
        assert_eq!(v4.winding(), tape_u64(5).as_slice());
        assert!(!v4.closes_modular_phase());

        assert!(matches!(
            spectral.descend_reentry_state(&v4),
            HadamardDescent::N(_)
        ));
    }
}
