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
        Ok(Self {
            levels,
            forward_pairs,
            reverse_pairs,
        })
    }

    pub fn levels(&self) -> usize {
        self.levels
    }
    pub fn forward_pairs(&self) -> &[(usize, usize)] {
        &self.forward_pairs
    }
    pub fn reverse_pairs(&self) -> &[(usize, usize)] {
        &self.reverse_pairs
    }

    /// f_{i,j} = f_{j-1} ∘ ... ∘ f_i, represented by its adjacent spine.
    pub fn forward_rail(&self, from: usize, to: usize) -> Option<Vec<usize>> {
        if from >= to || to >= self.levels {
            return None;
        }
        Some((from..to).collect())
    }

    /// r_{j,i} = v_i ∘ ... ∘ v_{j-1}, represented in reverse order.
    pub fn reverse_rail(&self, from: usize, to: usize) -> Option<Vec<usize>> {
        if from <= to || from >= self.levels {
            return None;
        }
        Some((to..from).rev().collect())
    }

    /// Apply the direct sidearm and its reverse on a bit tape. Each adjacent
    /// map applies the bitwise involution; the reverse rail applies the same
    /// involution in reverse edge order. Their composition must recover the
    /// input tape.
    pub fn preserves(&self, from: usize, to: usize, carrier: &[u8]) -> bool {
        let (Some(forward), Some(reverse)) =
            (self.forward_rail(from, to), self.reverse_rail(to, from))
        else {
            return false;
        };
        let mut resident = carrier.to_vec();
        for _edge in forward {
            resident.iter_mut().for_each(|cell| *cell = !*cell);
        }
        for _edge in reverse {
            resident.iter_mut().for_each(|cell| *cell = !*cell);
        }
        resident == carrier
    }

    /// Apply the direct sidearm and its reverse on an IMASM numeral bit-tape.
    pub fn preserves_tape(&self, from: usize, to: usize, tape: &[char]) -> bool {
        if tape.iter().any(|cell| !matches!(cell, '⊥' | '⊤')) {
            return false;
        }
        let (Some(forward), Some(reverse)) =
            (self.forward_rail(from, to), self.reverse_rail(to, from))
        else {
            return false;
        };
        let mut resident = tape.to_vec();
        for _edge in forward {
            flip_numeral_cells(&mut resident);
        }
        for _edge in reverse {
            flip_numeral_cells(&mut resident);
        }
        resident == tape
    }

    /// Evaluate all-to-all Frobenius closure (μ∘δ = id) across all pairs in the tower.
    pub fn all_pairs_preserve_tape(&self, tape: &[char]) -> bool {
        for &(from, to) in &self.forward_pairs {
            if !self.preserves_tape(from, to, tape) {
                return false;
            }
        }
        true
    }

    /// Full Nester Tower audit: returns (closed, total_bidirectional_pairs).
    pub fn evaluate_tower_closure(&self, tape: &[char]) -> (bool, usize) {
        let ok = self.all_pairs_preserve_tape(tape);
        (ok, self.forward_pairs.len())
    }
}

fn flip_numeral_cells(tape: &mut [char]) {
    for cell in tape {
        *cell = match *cell {
            '⊥' => '⊤',
            '⊤' => '⊥',
            other => other,
        };
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
            let mut transformed = b"resident relation".to_vec();
            transformed.iter_mut().for_each(|cell| *cell = !*cell);
            assert_ne!(transformed, b"resident relation");
            assert_eq!(membrane.forward_rail(from, to).unwrap().len(), to - from);
            assert_eq!(membrane.reverse_rail(to, from).unwrap().len(), to - from);
        }
        let tape: Vec<char> = "⊥⊤⊤⊥⊥⊤⊤⊥".chars().collect();
        assert!(membrane.all_pairs_preserve_tape(&tape));

        let mut resident = b"resident relation".to_vec();
        for _edge in membrane.forward_rail(0, 1).unwrap() {
            resident.iter_mut().for_each(|cell| *cell = !*cell);
        }
        assert_ne!(resident, b"resident relation");
        for _edge in membrane.reverse_rail(1, 0).unwrap() {
            resident.iter_mut().for_each(|cell| *cell = !*cell);
        }
        assert_eq!(resident, b"resident relation");
    }

    #[test]
    fn level_count_is_arbitrary_above_two() {
        for levels in [3, 4, 17, 64] {
            let membrane = CompleteMembrane::new(levels).unwrap();
            assert_eq!(membrane.forward_pairs().len(), levels * (levels - 1) / 2);
        }
    }

    #[test]
    fn all_register_pairs_close_on_a_65536_cell_tape() {
        let membrane = CompleteMembrane::new(16).unwrap();
        let tape: Vec<char> = (0..65_536)
            .map(|i| if i % 3 == 0 { '⊥' } else { '⊤' })
            .collect();
        let (closed, pairs) = membrane.evaluate_tower_closure(&tape);
        assert!(closed);
        assert_eq!(pairs, 120);
        assert!(!membrane.preserves_tape(0, 1, &['x']));
    }

    #[test]
    fn two_levels_are_rejected() {
        assert!(CompleteMembrane::new(2).is_err());
    }
}
