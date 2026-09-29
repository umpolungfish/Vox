//! Suffix-envelope axis of Core Numeral.
//!
//! A suffix envelope is the descending chain `Γ` of restored supports, the
//! deepest-occurrence vector `r(x)`, the reconstruction of `Γ` from `r`, and
//! the exact fibre over that chain.
//!
//! The chain is read from the existing anchors, not recomputed beside them:
//! deposit schedules go through `provenance_envelope::restored_support_ladder`,
//! phase words through `phase_word::execute` restore unions, and tapes through
//! `factor_2adic::frame_sweep` window groups.

use crate::factor_2adic::frame_sweep;
use crate::phase_word::execute;
use crate::provenance_envelope::{
    is_descending, restored_support_ladder, suffix_fibre_size, LaneSupport,
};
use crate::vox::EVALF;
use alloc::vec::Vec;

/// Descending restored-support chain and the data determined by it.
///
/// `gamma[i]` is `Γ_i`, outermost frame first, so `Γ_i ⊇ Γ_{i+1}`.
/// `deepest[x]` is `r(x)`, the deepest frame index of atom `x`, or `None`
/// when `x` lies outside `Γ_0`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SuffixEnvelope {
    pub gamma: Vec<LaneSupport>,
    pub deepest: Vec<Option<u32>>,
    pub fibre_count: u128,
}

/// Free-bit parametrization of the deposit sequences lying over `Γ`.
///
/// Frame `i` (outermost first) must deposit every bit of `forced[i]` and may
/// deposit any subset of `free[i]`. The last frame is forced entirely.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FibreParam {
    pub forced: Vec<LaneSupport>,
    pub free: Vec<LaneSupport>,
}

impl SuffixEnvelope {
    /// Envelope of a deposit schedule, outermost frame first.
    pub fn from_deposits(deposits: &[LaneSupport]) -> Option<Self> {
        Self::from_gamma(restored_support_ladder(deposits))
    }

    /// Envelope of an already-restored chain. `None` when the chain is not
    /// descending or the fibre count does not fit in `u128`.
    pub fn try_from_ladder(gamma: &[LaneSupport]) -> Option<Self> {
        Self::from_gamma(gamma.iter().copied().collect())
    }

    /// Envelope of the restore unions recorded by `phase_word::execute`.
    ///
    /// Those unions are pushed innermost-first; the chain stored here is the
    /// same sequence read outermost-first.
    pub fn from_phase_word(word: &str) -> Result<Self, &'static str> {
        let readout = execute(word, &()).map_err(|_| "invalid phase execution word")?;
        let supports: Vec<LaneSupport> = readout
            .restore_supports
            .iter()
            .map(|support| LaneSupport::from(*support))
            .collect();
        Self::from_innermost_supports(&supports)
            .ok_or("phase restore unions are not a suffix envelope")
    }

    /// Envelope of one frame-sweep window.
    ///
    /// Each group of `window` tape marks is one frame deposit. Bit `i` of that
    /// deposit is set when the group's `i`-th mark is `⊥`.
    pub fn from_frame_sweep(tape: &[char], window: usize) -> Result<Self, &'static str> {
        if !(2..=8).contains(&window) {
            return Err("frame sweep windows are 2..=8");
        }
        let sweeps = frame_sweep(tape);
        let sweep = sweeps
            .iter()
            .find(|sweep| sweep.window == window)
            .ok_or("frame sweep did not produce the requested window")?;
        let mut deposits = Vec::with_capacity(sweep.groups.len());
        for group in &sweep.groups {
            deposits.push(group_mask(group)?);
        }
        Self::from_deposits(&deposits).ok_or("frame-sweep fibre does not fit in u128")
    }

    /// `Γ_i`, outermost index 0.
    pub fn gamma_i(&self, index: usize) -> Option<LaneSupport> {
        self.gamma.get(index).copied()
    }

    /// Deepest occurrence `r(x)`.
    pub fn r(&self, atom: u32) -> Option<u32> {
        self.deepest.get(atom as usize).copied().flatten()
    }

    /// Rebuild `Γ` from `r`: atom `x` belongs to `Γ_0, …, Γ_{r(x)}`.
    pub fn reconstruct(&self) -> Vec<LaneSupport> {
        let mut gamma = vec![0; self.gamma.len()];
        for (atom, depth) in self.deepest.iter().enumerate() {
            if let Some(depth) = *depth {
                for index in 0..=depth as usize {
                    if index < gamma.len() && atom < 32 {
                        gamma[index] |= 1u32 << atom;
                    }
                }
            }
        }
        gamma
    }

    /// Forced and free bits of every deposit over this chain.
    pub fn fibre_param(&self) -> FibreParam {
        let mut forced = Vec::with_capacity(self.gamma.len());
        let mut free = Vec::with_capacity(self.gamma.len());
        for (index, support) in self.gamma.iter().enumerate() {
            let next = self.gamma.get(index + 1).copied().unwrap_or(0);
            forced.push(support & !next);
            free.push(next);
        }
        FibreParam { forced, free }
    }

    fn from_innermost_supports(supports: &[LaneSupport]) -> Option<Self> {
        let gamma = supports.iter().rev().copied().collect();
        Self::from_gamma(gamma)
    }

    fn from_gamma(gamma: Vec<LaneSupport>) -> Option<Self> {
        if !is_descending(&gamma) {
            return None;
        }
        let fibre_count = suffix_fibre_size(&gamma)?;
        let width = gamma.iter().fold(0u32, |width, support| {
            width.max(32 - support.leading_zeros())
        });
        let mut deepest = vec![None; width as usize];
        for (index, support) in gamma.iter().enumerate() {
            let mut bits = *support;
            while bits != 0 {
                let atom = bits.trailing_zeros() as usize;
                deepest[atom] = Some(index as u32);
                bits &= bits - 1;
            }
        }
        Some(Self {
            gamma,
            deepest,
            fibre_count,
        })
    }
}

/// `|fibre(Γ)|` from the free-bit parametrization, `Π_i 2^{popcount(free_i)}`.
pub fn parametrized_fibre(param: &FibreParam) -> Option<u128> {
    let mut count = 1u128;
    for free in &param.free {
        count = count.checked_mul(1u128.checked_shl(free.count_ones())?)?;
    }
    Some(count)
}

/// Image theorem: descending chains of length `depth` on a `k`-set.
///
/// Each atom chooses a deepest index in `-1, 0, …, depth-1`, so there are
/// `(depth + 1)^k` chains.
pub fn image_count(k: u32, depth: u32) -> Option<u128> {
    u128::from(depth)
        .checked_add(1)
        .and_then(|base| base.checked_pow(k))
}

/// Deposit sequences whose outer support is a fixed `k`-set:
/// `(2^depth − 1)^k`.
pub fn fixed_union_count(k: u32, depth: u32) -> Option<u128> {
    if depth >= 128 {
        return None;
    }
    let base = (1u128 << depth) - 1;
    base.checked_pow(k)
}

/// Deepest-occurrence polynomial `M_{k,depth}(y)`.
///
/// ```text
/// M_{k,d}(y) = (1 + Σ_{r=1}^{d} 2^{r-1} y^r)^k
/// ```
///
/// Coefficient `r` counts deposit sequences whose suffix mass is `r`.
/// The mass of a chain is `Σ_x (r(x) + 1)` over atoms that occur.
pub fn mass_polynomial(k: u32, depth: u32) -> Option<Vec<u128>> {
    if depth >= 128 {
        return None;
    }
    let mut base = Vec::with_capacity(depth as usize + 1);
    base.push(1u128);
    for r in 1..=depth {
        base.push(1u128.checked_shl(r - 1)?);
    }
    let mut poly = vec![1u128];
    for _ in 0..k {
        let mut next = vec![0u128; poly.len() + depth as usize];
        for (i, &left) in poly.iter().enumerate() {
            for (j, &right) in base.iter().enumerate() {
                let product = left.checked_mul(right)?;
                next[i + j] = next[i + j].checked_add(product)?;
            }
        }
        poly = next;
    }
    Some(poly)
}

fn group_mask(group: &[char]) -> Result<LaneSupport, &'static str> {
    let mut mask = 0u32;
    for (index, mark) in group.iter().enumerate() {
        if *mark == EVALF {
            if index >= 32 {
                return Err("frame-sweep group wider than a lane word");
            }
            mask |= 1u32 << index;
        }
    }
    Ok(mask)
}
