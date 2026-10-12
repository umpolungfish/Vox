//! Lean Evidence registry
//!
//! Maps theorem identifiers to build witnesses.
//! Never substitutes for an executable check or a native measurement
//! (daplan.md §8 evidence-substitution risk control): evidence-level tag only.

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

/// A single kernel-checked citation.
#[derive(Clone, Debug)]
pub struct LeanWitness {
    pub theorem_id: String,
    pub build_hash: String,
    pub statement: String,
}

/// Global registry (process-local).
#[derive(Clone, Debug, Default)]
pub struct LeanEvidenceRegistry {
    map: BTreeMap<String, LeanWitness>,
}

impl LeanEvidenceRegistry {
    pub fn new() -> Self {
        Self {
            map: BTreeMap::new(),
        }
    }

    pub fn register(&mut self, id: &str, hash: &str, statement: &str) {
        self.map.insert(
            id.into(),
            LeanWitness {
                theorem_id: id.into(),
                build_hash: hash.into(),
                statement: statement.into(),
            },
        );
    }

    pub fn lookup(&self, id: &str) -> Option<&LeanWitness> {
        self.map.get(id)
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Convenience: the carry-identity theorem cited by the membrane certificate.
    pub fn register_carry_identity(&mut self) {
        self.register(
            "carry_identity",
            "kernel-checked-placeholder",
            "popcount(p)·popcount(q) − popcount(N) = Σ carry values",
        );
    }

    pub fn register_syzygy(&mut self) {
        self.register(
            "syzygy_roundtrip",
            "kernel-checked-placeholder",
            "Λ(Γ(D(p),D(q))) = (D(p),D(q)) ∧ encode(μ(Λ(D))) = D(N)",
        );
    }

    pub fn all_ids(&self) -> Vec<&str> {
        self.map.keys().map(|s| s.as_str()).collect()
    }
}

/// Module-level default registry used by Verification.
pub fn default_registry() -> LeanEvidenceRegistry {
    let mut r = LeanEvidenceRegistry::new();
    r.register_carry_identity();
    r.register_syzygy();
    r
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_registry_holds_the_two_citations() {
        let reg = default_registry();
        assert_eq!(reg.len(), 2);
        assert!(reg.lookup("carry_identity").is_some());
        assert!(reg.lookup("syzygy_roundtrip").is_some());
        assert!(reg.lookup("nonexistent").is_none());
    }
}
