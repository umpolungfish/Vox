//! Collapsed runtime form of an ancestry-contained IMASM hypernest.
//!
//! The diagnostic expansion of a depth-d hypernest contains d complete carriers,
//! but execution does not materialize or traverse those copies. The fixed-point
//! nesting rule carries depth as the winding register while one canonical carrier
//! word executes once. Runtime cost is therefore depth-invariant: one process
//! word, one banked live clear, one collapse tick.

use crate::fixed_point_imasm::{HypernestAudit, HypernestedImasmCarrier};
use crate::vox::{
    verdict, AFWD, AREV, CLINK, EVALF, EVALT, FFUSE, FSPLIT, IFIX, IMSCRIB, TANCH,
    VINIT,
};

/// The single process word executed by a collapsed hypernest.
///
/// Full recursive containment is represented by `depth`, not by copying this
/// word once per level at runtime.
pub const COLLAPSED_HYPERNEST_WORD: [char; 11] = [
    VINIT, IMSCRIB, FSPLIT, AFWD, EVALT, AREV, EVALF, CLINK, FFUSE, IFIX, TANCH,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CollapsedHypernest {
    depth: usize,
}

impl CollapsedHypernest {
    pub fn from_depth(depth: usize) -> Result<Self, &'static str> {
        if depth == 0 {
            return Err("collapsed hypernest requires at least one carrier level");
        }
        let collapsed = Self { depth };
        let audit = collapsed.audit();
        if audit.live_clears != 1
            || audit.exposed != 0
            || !audit.banked
            || !audit.closed
            || audit.winding != depth
        {
            return Err("collapsed hypernest process invariant failed");
        }
        Ok(collapsed)
    }

    /// Number of complete carrier embeddings represented by this membrane.
    pub fn depth(&self) -> usize { self.depth }

    /// Hypernest instrumentation identifies integer winding with carrier depth.
    pub fn winding(&self) -> usize { self.depth }

    /// The runtime executes one canonical carrier irrespective of depth.
    pub fn word(&self) -> &'static [char] { &COLLAPSED_HYPERNEST_WORD }

    /// Fixed-point nesting collapses all enclosing copies in one composition.
    pub fn execution_ticks(&self) -> usize { 1 }

    /// Read the runtime invariants from the canonical process word. Depth
    /// contributes only the winding register; closure/banking are measured from
    /// the word that actually executes.
    pub fn audit(&self) -> HypernestAudit {
        let word = self.word();
        let split = word.iter().position(|&mark| mark == FSPLIT);
        let fuse = word.iter().position(|&mark| mark == FFUSE);
        let clears: alloc::vec::Vec<usize> = word
            .iter()
            .enumerate()
            .filter_map(|(at, &mark)| (mark == AREV).then_some(at))
            .collect();
        let live_clears = clears.len();
        let banked = match (split, fuse, clears.as_slice()) {
            (Some(open), Some(close), [clear]) => open < *clear && *clear < close,
            _ => false,
        };
        HypernestAudit {
            winding: self.depth,
            live_clears,
            exposed: if banked { 0 } else { live_clears },
            banked,
            closed: verdict(word) == 'T',
        }
    }

    /// Expand only for pairing/ancestry instrumentation. Production execution
    /// must use the collapsed representation above.
    #[cfg(test)]
    pub fn diagnostic_expansion(&self) -> Result<HypernestedImasmCarrier, &'static str> {
        HypernestedImasmCarrier::from_depth(self.depth)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vox::pairing;

    #[test]
    fn runtime_storage_and_ticks_are_depth_invariant() {
        let one = CollapsedHypernest::from_depth(1).unwrap();
        let huge = CollapsedHypernest::from_depth(4096).unwrap();
        assert_eq!(one.word(), huge.word());
        assert_eq!(one.word().len(), COLLAPSED_HYPERNEST_WORD.len());
        assert_eq!(one.execution_ticks(), 1);
        assert_eq!(huge.execution_ticks(), 1);
        assert_eq!(one.winding(), 1);
        assert_eq!(huge.winding(), 4096);
    }

    #[test]
    fn collapsed_audit_measures_the_free_lunch_from_the_executed_word() {
        for depth in [1usize, 2, 3, 64, 1024] {
            let runtime = CollapsedHypernest::from_depth(depth).unwrap();
            let audit = runtime.audit();
            assert_eq!(runtime.word().iter().filter(|&&m| m == AREV).count(), 1);
            assert_eq!(audit.winding, depth);
            assert_eq!(audit.live_clears, 1);
            assert_eq!(audit.exposed, 0);
            assert!(audit.banked);
            assert!(audit.closed);
        }
    }

    #[test]
    fn diagnostic_expansion_matches_ancestry_without_becoming_runtime_state() {
        let runtime = CollapsedHypernest::from_depth(3).unwrap();
        let expanded = runtime.diagnostic_expansion().unwrap();
        assert_eq!(expanded.winding(), runtime.winding());
        let (_regions, unanswered, unopened) = pairing(expanded.word());
        assert!(unanswered.is_empty());
        assert!(unopened.is_empty());
        assert!(expanded.word().len() > runtime.word().len());
    }
}
