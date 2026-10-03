//! Quantum-native fixed-point membrane architecture.
//!
//! This layer deliberately contains no factor search. The resident object is
//! one collapsed IMASM hypernest together with the N-dependent modular phase
//! operator and the exact Hadamard/spectral boundaries already implemented by
//! Vox. Classical factor geometries do not belong here.
//!
//! Hypernest depth is winding, but execution is depth-invariant: one canonical
//! carrier word executes once while the carried depth records the complete
//! ancestry containment.

use crate::fixed_point_hypernest::CollapsedHypernest;
use crate::hadamard_gate::{
    hadamard_character, FixedPointSpectralConstruction, HadamardCarrier, Tape,
};
#[cfg(test)]
use crate::hadamard_gate::SpectralWinding;

#[derive(Clone, PartialEq, Debug)]
pub struct FixedPointQuantumMembrane {
    topology: CollapsedHypernest,
    spectral: FixedPointSpectralConstruction,
}

impl FixedPointQuantumMembrane {
    pub fn from_n(n: &[char]) -> Result<Self, &'static str> {
        Self::from_n_with_base(n, &[crate::vox::EVALT, crate::vox::EVALF])
    }

    /// Build the resident phase membrane with a selected coprime modular base.
    /// Both source and base remain numerator tapes through phase execution.
    pub fn from_n_with_base(n: &[char], base: &[char]) -> Result<Self, &'static str> {
        let spectral = HadamardCarrier::new(n)?
            .fixed_point_spectral_construction_with_base(base)?;
        let depth = n.len().saturating_sub(3).max(1);
        let topology = CollapsedHypernest::from_depth(depth)?;
        let audit = topology.audit();
        if audit.winding != depth
            || audit.live_clears != 1
            || audit.exposed != 0
            || !audit.banked
            || !audit.closed
            || topology.execution_ticks() != 1
        {
            return Err("quantum fixed-point collapsed hypernest invariant failed");
        }
        Ok(Self { topology, spectral })
    }

    pub fn n(&self) -> &[char] { self.spectral.n() }
    pub fn base(&self) -> &[char] { self.spectral.base() }
    pub fn topology(&self) -> &CollapsedHypernest { &self.topology }
    pub fn executed_word(&self) -> &[char] { self.topology.word() }
    pub fn resident_winding(&self) -> usize { self.topology.winding() }
    pub fn execution_ticks(&self) -> usize { self.topology.execution_ticks() }

    /// N-dependent modular phase operator. The exponent addresses one coherent
    /// branch directly; this does not expose or accept a measured k/M sample.
    pub fn modular_phase(&self, exponent: &[char]) -> Result<Tape, &'static str> {
        self.spectral.modular_branch(exponent)
    }

    /// Raw phase-coordinate construction is deliberately test-only here. The
    /// production quantum membrane can only reach phase descent through the
    /// opaque pair-before-advance landing boundary.
    #[cfg(test)]
    fn winding(&self, raw_numerator: &[char], denominator: &[char]) -> Result<SpectralWinding, &'static str> {
        self.spectral.winding(raw_numerator, denominator)
    }

    #[cfg(test)]
    fn split_fuse(&self, raw_numerator: &[char], denominator: &[char]) -> Result<SpectralWinding, &'static str> {
        let winding = self.winding(raw_numerator, denominator)?;
        let fused = winding.clone().delta().mu()?;
        if fused != winding {
            return Err("quantum spectral split/fuse changed the resident winding");
        }
        Ok(fused)
    }

    pub fn hadamard_sign(&self, source: &[char], target: &[char]) -> Result<i8, &'static str> {
        hadamard_character(source, target)
    }

    #[cfg(test)]
    fn fixed_point_sign(&self, raw_numerator: &[char], denominator: &[char]) -> Result<Option<i8>, &'static str> {
        Ok(self.split_fuse(raw_numerator, denominator)?.fixed_point_sign())
    }

    /// Crate-internal escape hatch used only by the sealed readout/descent
    /// module. External callers cannot unwrap the resident spectral object and
    /// inject their own raw phase coordinate.
    pub(crate) fn into_spectral(self) -> FixedPointSpectralConstruction { self.spectral }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::morphism_factor::tape_u64;

    #[test]
    fn resident_topology_is_a_collapsed_hypercarrier() {
        for n in [17u64, 257, 65_537, 4_294_967_291] {
            let tape = tape_u64(n);
            let membrane = FixedPointQuantumMembrane::from_n(&tape).unwrap();
            let expected = tape.len().saturating_sub(3).max(1);
            assert_eq!(membrane.topology().depth(), expected);
            assert_eq!(membrane.resident_winding(), expected);
            assert_eq!(membrane.execution_ticks(), 1);
            let audit = membrane.topology().audit();
            assert_eq!(audit.live_clears, 1);
            assert_eq!(audit.exposed, 0);
            assert!(audit.banked);
            assert!(audit.closed);
        }
    }

    #[test]
    fn runtime_word_is_depth_invariant_while_winding_changes() {
        let a = FixedPointQuantumMembrane::from_n(&tape_u64(257)).unwrap();
        let b = FixedPointQuantumMembrane::from_n(&tape_u64(65_537)).unwrap();
        assert_ne!(a.resident_winding(), b.resident_winding());
        assert_eq!(a.executed_word(), b.executed_word());
        assert_eq!(a.execution_ticks(), 1);
        assert_eq!(b.execution_ticks(), 1);
    }

    #[test]
    fn coherent_modular_phase_is_basis_addressed_not_orbit_walked() {
        let membrane = FixedPointQuantumMembrane::from_n(&tape_u64(257)).unwrap();
        let e3 = membrane.modular_phase(&tape_u64(3)).unwrap();
        let e11 = membrane.modular_phase(&tape_u64(11)).unwrap();
        assert_ne!(e3, e11);
        assert_eq!(membrane.modular_phase(&tape_u64(3)).unwrap(), e3);
    }

    #[test]
    fn spectral_delta_mu_is_exact_inside_the_collapsed_hypernest() {
        let membrane = FixedPointQuantumMembrane::from_n(&tape_u64(257)).unwrap();
        let before = membrane.winding(&tape_u64(29), &tape_u64(16)).unwrap();
        let after = membrane.split_fuse(&tape_u64(29), &tape_u64(16)).unwrap();
        assert_eq!(before, after);
    }

    #[test]
    fn only_true_z2_fixed_points_reach_the_hadamard_edge() {
        let membrane = FixedPointQuantumMembrane::from_n(&tape_u64(257)).unwrap();
        assert_eq!(membrane.fixed_point_sign(&tape_u64(0), &tape_u64(16)).unwrap(), Some(1));
        assert_eq!(membrane.fixed_point_sign(&tape_u64(8), &tape_u64(16)).unwrap(), Some(-1));
        assert_eq!(membrane.fixed_point_sign(&tape_u64(5), &tape_u64(16)).unwrap(), None);
    }

    #[test]
    fn selected_base_stays_bound_to_a_128_bit_semiprime_phase_program() {
        let n = crate::morphism_factor::decimal_to_tape(
            "296650821743515430283258444261036507151",
        )
        .unwrap();
        let base = tape_u64(3);
        let program = FixedPointQuantumMembrane::from_n_with_base(&n, &base)
            .unwrap()
            .prepare_structural_execution()
            .unwrap();
        assert_eq!(program.n(), n);
        assert_eq!(program.base(), base);
        assert_eq!(program.phase_width(), 256);
    }
}
