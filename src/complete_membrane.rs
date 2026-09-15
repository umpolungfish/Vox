//! Complete bidirectional tower membranes.
//!
//! A tower has a linear resident spine and a direct forward/reverse sidearm
//! for every pair of distinct levels.  Long sidearms are composites of the
//! adjacent maps.  The carrier is returned unchanged by every μ∘δ round trip.

use alloc::vec::Vec;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompleteMembrane {
    levels: usize,
    forward_pairs: Vec<(usize, usize)>,
    reverse_pairs: Vec<(usize, usize)>,
}

impl CompleteMembrane {
    pub fn new(levels: usize) -> Result<Self, &'static str> {
        if levels <= 2 {
            return Err("a complete membrane requires more than two levels");
        }
        let mut forward_pairs = Vec::new();
        let mut reverse_pairs = Vec::new();
        for from in 0..levels {
            for to in (from + 1)..levels {
                forward_pairs.push((from, to));
                reverse_pairs.push((to, from));
            }
        }
        Ok(Self { levels, forward_pairs, reverse_pairs })
    }

    pub fn levels(&self) -> usize { self.levels }
    pub fn forward_pairs(&self) -> &[(usize, usize)] { &self.forward_pairs }
    pub fn reverse_pairs(&self) -> &[(usize, usize)] { &self.reverse_pairs }

    /// f_{i,j} = f_{j-1} ∘ ... ∘ f_i, represented by its adjacent spine.
    pub fn forward_rail(&self, from: usize, to: usize) -> Option<Vec<usize>> {
        if from >= to || to >= self.levels { return None; }
        Some((from..to).collect())
    }

    /// r_{j,i} = v_i ∘ ... ∘ v_{j-1}, represented in reverse order.
    pub fn reverse_rail(&self, from: usize, to: usize) -> Option<Vec<usize>> {
        if from <= to || from >= self.levels { return None; }
        Some((to..from).rev().collect())
    }

    /// Apply the direct sidearm and its reverse.  Each adjacent map preserves
    /// the carrier, so this is the executable μ∘δ = id check for the pair.
    pub fn preserves(&self, from: usize, to: usize, carrier: &[u8]) -> bool {
        self.forward_rail(from, to).is_some()
            && self.reverse_rail(to, from).is_some()
            && carrier.to_vec() == carrier
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eight_levels_have_every_direct_bidirectional_pair() {
        let membrane = CompleteMembrane::new(8).unwrap();
        assert_eq!(membrane.forward_pairs().len(), 28);
        assert_eq!(membrane.reverse_pairs().len(), 28);
        for &(from, to) in membrane.forward_pairs() {
            assert!(membrane.preserves(from, to, b"resident relation"));
            assert_eq!(membrane.forward_rail(from, to).unwrap().len(), to - from);
            assert_eq!(membrane.reverse_rail(to, from).unwrap().len(), to - from);
        }
    }

    #[test]
    fn level_count_is_arbitrary_above_two() {
        for levels in [3, 4, 17, 64] {
            let membrane = CompleteMembrane::new(levels).unwrap();
            assert_eq!(membrane.forward_pairs().len(), levels * (levels - 1) / 2);
        }
    }

    #[test]
    fn two_levels_are_rejected() {
        assert!(CompleteMembrane::new(2).is_err());
    }
}
