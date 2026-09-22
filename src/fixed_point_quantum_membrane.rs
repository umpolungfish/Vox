//! Quantum-native fixed-point membrane architecture.
//!
//! This layer deliberately contains no factor search. The resident object is
//! the properly nested IMASM tower together with the N-dependent modular phase
//! operator and the exact Hadamard/spectral boundaries already implemented by
//! Vox. Classical factor geometries do not belong here.

use alloc::vec::Vec;

use crate::fixed_point_imasm::NestedImasmTower;
use crate::hadamard_gate::{
    hadamard_character, FixedPointSpectralConstruction, HadamardCarrier,
    SpectralWinding, Tape,
};

#[derive(Clone, PartialEq, Debug)]
pub struct FixedPointQuantumMembrane {
    topology: NestedImasmTower,
    dissolved: Vec<char>,
    spectral: FixedPointSpectralConstruction,
}

impl FixedPointQuantumMembrane {
    pub fn from_n(n: &[char]) -> Result<Self, &'static str> {
        let spectral = HadamardCarrier::new(n)?.fixed_point_spectral_construction()?;
        let depth = n.len().saturating_sub(3).max(1);
        let topology = NestedImasmTower::from_depth(depth)?;
        if !topology.banking_audit().banked {
            return Err("quantum fixed-point topology leaves the reversal exposed");
        }
        let dissolved = topology.dissolved_word();
        Ok(Self { topology, dissolved, spectral })
    }

    pub fn n(&self) -> &[char] { self.spectral.n() }
    pub fn base(&self) -> &[char] { self.spectral.base() }
    pub fn topology(&self) -> &NestedImasmTower { &self.topology }
    pub fn dissolved_word(&self) -> &[char] { &self.dissolved }

    pub fn modular_phase(&self, exponent: &[char]) -> Result<Tape, &'static str> {
        self.spectral.modular_branch(exponent)
    }

    pub fn winding(&self, raw_numerator: &[char], denominator: &[char]) -> Result<SpectralWinding, &'static str> {
        self.spectral.winding(raw_numerator, denominator)
    }

    pub fn split_fuse(&self, raw_numerator: &[char], denominator: &[char]) -> Result<SpectralWinding, &'static str> {
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

    pub fn fixed_point_sign(&self, raw_numerator: &[char], denominator: &[char]) -> Result<Option<i8>, &'static str> {
        Ok(self.split_fuse(raw_numerator, denominator)?.fixed_point_sign())
    }

    pub fn into_spectral(self) -> FixedPointSpectralConstruction { self.spectral }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::morphism_factor::tape_u64;

    #[test]
    fn nesting_is_a_property_of_the_resident_register_width() {
        for n in [17u64, 257, 65_537, 4_294_967_291] {
            let tape = tape_u64(n);
            let membrane = FixedPointQuantumMembrane::from_n(&tape).unwrap();
            assert_eq!(membrane.topology().max_depth(), tape.len().saturating_sub(3).max(1));
            assert!(membrane.topology().banking_audit().banked);
            assert_eq!(membrane.topology().banking_audit().exposed, 0);
        }
    }

    #[test]
    fn nested_dissolution_does_not_depend_on_factor_relations() {
        let a = FixedPointQuantumMembrane::from_n(&tape_u64(257)).unwrap();
        let b = FixedPointQuantumMembrane::from_n(&tape_u64(65_537)).unwrap();
        assert_ne!(a.topology().max_depth(), b.topology().max_depth());
        assert_eq!(a.dissolved_word(), b.dissolved_word());
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
    fn spectral_delta_mu_is_exact_inside_the_nested_membrane() {
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
}
