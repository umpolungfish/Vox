//! Collapsed fixed-point spectral membrane.
//!
//! The repaired 25-op boundary exposes only three fixed windings. A compiled
//! tower must not recover larger windings by walking TANCH one step at a time.
//! This module instead compiles the spectral observation into nested split/fuse
//! membranes over exponent coordinates. The nested tree shares only structural
//! parent phases: no successive modular-orbit state is advanced to discover an
//! order, and TANCH is never iterated looking for closure.
//!
//! The membrane is affine at its public boundary: `(N, membrane) -> (p, q)`.
//! N, the spectral carrier, the fused winding, and the membrane are all consumed.

use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

use core::cmp::Ordering;

use crate::factor_extract::extract;
use crate::hadamard_factor_bridge::HadamardDescent;
use crate::hadamard_gate::{FixedPointSpectralConstruction, HadamardCarrier, Tape};
use crate::morphism_factor::{add, cmp, modulo, mul, one, sub};
use crate::vox::{EVALF, EVALT};

#[derive(Clone, PartialEq, Debug)]
pub struct FixedPointFactorPair {
    pub p: Tape,
    pub q: Tape,
}

/// Exact folded storage for a spectral phase deposit wider than one host limb.
/// Arithmetic never consumes this representation; it is only the key used by
/// the resident split/fuse table.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
enum PackedPhase {
    OneLimb(u64),
    Many(Box<[u64]>),
}

fn pack_phase(tape: &[char]) -> PackedPhase {
    let limbs = (tape.len() + 63) / 64;
    if limbs <= 1 {
        return PackedPhase::OneLimb(pack_one_limb(tape));
    }

    let mut out = alloc::vec![0u64; limbs];
    for (bit, &mark) in tape.iter().enumerate() {
        if mark == EVALF {
            out[bit / 64] |= 1u64 << (bit % 64);
        }
    }
    PackedPhase::Many(out.into_boxed_slice())
}

fn pack_one_limb(tape: &[char]) -> u64 {
    let mut value = 0u64;
    for (bit, &mark) in tape.iter().take(64).enumerate() {
        if mark == EVALF {
            value |= 1u64 << bit;
        }
    }
    value
}

/// Resident inner deposits. Up through 64-bit N the phase itself is one limb,
/// so use a compact open-addressed table instead of allocating one B-tree node
/// per branch. Wider membranes retain exact arbitrary-width packed keys.
enum PhaseDeposits {
    OneLimb {
        keys: Vec<u64>,
        ordinals_plus_one: Vec<u32>,
        mask: usize,
    },
    Wide(BTreeMap<PackedPhase, usize>),
}

impl PhaseDeposits {
    fn new(n: &[char], branches: usize) -> Result<Self, String> {
        if n.len() <= 64 {
            let wanted = branches
                .checked_mul(2)
                .ok_or_else(|| String::from("compiled membrane deposit capacity overflow"))?;
            let capacity = wanted
                .checked_next_power_of_two()
                .ok_or_else(|| String::from("compiled membrane deposit capacity overflow"))?;
            let mut keys = Vec::new();
            let mut ordinals_plus_one = Vec::new();
            keys.try_reserve_exact(capacity)
                .map_err(|_| String::from("compiled membrane phase table allocation failed"))?;
            ordinals_plus_one
                .try_reserve_exact(capacity)
                .map_err(|_| String::from("compiled membrane ordinal table allocation failed"))?;
            keys.resize(capacity, 0);
            ordinals_plus_one.resize(capacity, 0);
            Ok(Self::OneLimb {
                keys,
                ordinals_plus_one,
                mask: capacity - 1,
            })
        } else {
            Ok(Self::Wide(BTreeMap::new()))
        }
    }

    fn slot(key: u64, mask: usize) -> usize {
        // SplitMix64 finalizer: deterministic, cheap, and independent of host
        // hash-map state. It is only a physical address in the membrane table.
        let mut x = key;
        x ^= x >> 30;
        x = x.wrapping_mul(0xbf58_476d_1ce4_e5b9);
        x ^= x >> 27;
        x = x.wrapping_mul(0x94d0_49bb_1331_11eb);
        x ^= x >> 31;
        (x as usize) & mask
    }

    fn insert(&mut self, phase: &[char], ordinal: usize) -> Result<(), String> {
        match self {
            Self::OneLimb {
                keys,
                ordinals_plus_one,
                mask,
            } => {
                let stored = u32::try_from(
                    ordinal
                        .checked_add(1)
                        .ok_or_else(|| String::from("compiled membrane branch address overflow"))?,
                )
                .map_err(|_| String::from("compiled membrane one-limb branch address overflow"))?;
                let key = pack_one_limb(phase);
                let mut slot = Self::slot(key, *mask);
                loop {
                    if ordinals_plus_one[slot] == 0 || keys[slot] == key {
                        keys[slot] = key;
                        ordinals_plus_one[slot] = stored;
                        return Ok(());
                    }
                    slot = (slot + 1) & *mask;
                }
            }
            Self::Wide(map) => {
                map.insert(pack_phase(phase), ordinal);
                Ok(())
            }
        }
    }

    fn get(&self, phase: &[char]) -> Option<usize> {
        match self {
            Self::OneLimb {
                keys,
                ordinals_plus_one,
                mask,
            } => {
                let key = pack_one_limb(phase);
                let mut slot = Self::slot(key, *mask);
                loop {
                    let stored = ordinals_plus_one[slot];
                    if stored == 0 {
                        return None;
                    }
                    if keys[slot] == key {
                        return Some((stored - 1) as usize);
                    }
                    slot = (slot + 1) & *mask;
                }
            }
            Self::Wide(map) => map.get(&pack_phase(phase)).copied(),
        }
    }
}

fn branch_ordinal_tape(mut ordinal: usize) -> Tape {
    if ordinal == 0 {
        return alloc::vec![EVALT];
    }
    let mut out = Vec::new();
    while ordinal != 0 {
        out.push(if ordinal & 1 == 1 { EVALF } else { EVALT });
        ordinal >>= 1;
    }
    out
}

fn phase_product(left: &[char], right: &[char], n: &[char]) -> Tape {
    modulo(&mul(left, right), n)
}

/// a^(2^k), produced by the nested doubling spine once per membrane scale.
fn binary_phase_jumps(seed: Tape, levels: usize, n: &[char]) -> Vec<Tape> {
    let mut jumps = Vec::with_capacity(levels + 1);
    jumps.push(seed);
    for k in 0..levels {
        let next = phase_product(&jumps[k], &jumps[k], n);
        jumps.push(next);
    }
    jumps
}

fn populate_inner(
    phase: &[char],
    start: usize,
    span: usize,
    jumps: &[Tape],
    n: &[char],
    deposits: &mut PhaseDeposits,
) -> Result<(), String> {
    if span == 1 {
        return deposits.insert(phase, start);
    }
    let half = span / 2;
    populate_inner(phase, start, half, jumps, n, deposits)?;
    let jump_index = half.trailing_zeros() as usize;
    let right_phase = phase_product(phase, &jumps[jump_index], n);
    populate_inner(
        &right_phase,
        start + half,
        half,
        jumps,
        n,
        deposits,
    )
}

fn consider_outer_deposit(
    spectral: &FixedPointSpectralConstruction,
    deposits: &PhaseDeposits,
    phase: &[char],
    i: usize,
    m_tape: &[char],
    best: &mut Option<Tape>,
) -> Result<(), String> {
    if i == 0 {
        return Ok(());
    }
    let Some(inner_ordinal) = deposits.get(phase) else {
        return Ok(());
    };

    let exponent = mul(&branch_ordinal_tape(i), m_tape);
    let inner_exponent = branch_ordinal_tape(inner_ordinal);
    if cmp(&exponent, &inner_exponent) != Ordering::Greater {
        return Ok(());
    }
    let winding = sub(&exponent, &inner_exponent);
    let closure = spectral.modular_branch(&winding).map_err(String::from)?;
    if cmp(&closure, &one()) != Ordering::Equal {
        return Ok(());
    }

    if best
        .as_ref()
        .map(|current| cmp(&winding, current) == Ordering::Less)
        .unwrap_or(true)
    {
        *best = Some(winding);
    }
    Ok(())
}

fn scan_outer(
    spectral: &FixedPointSpectralConstruction,
    deposits: &PhaseDeposits,
    phase: &[char],
    start: usize,
    span: usize,
    jumps: &[Tape],
    m_tape: &[char],
    best: &mut Option<Tape>,
) -> Result<(), String> {
    if span == 1 {
        return consider_outer_deposit(spectral, deposits, phase, start, m_tape, best);
    }
    let half = span / 2;
    scan_outer(
        spectral, deposits, phase, start, half, jumps, m_tape, best,
    )?;
    let jump_index = half.trailing_zeros() as usize;
    let right_phase = phase_product(phase, &jumps[jump_index], spectral.n());
    scan_outer(
        spectral,
        deposits,
        &right_phase,
        start + half,
        half,
        jumps,
        m_tape,
        best,
    )
}

/// The executable image of the properly nested fixed-point tower.
pub struct CompiledFixedPointMembrane;

impl CompiledFixedPointMembrane {
    pub const fn compiled() -> Self {
        Self
    }

    pub fn consume(self, n: Tape) -> Result<FixedPointFactorPair, String> {
        if cmp(&n, &one()) != Ordering::Greater {
            return Err(String::from("fixed-point membrane requires N > 1"));
        }

        let spectral = HadamardCarrier::new(&n)
            .map_err(String::from)?
            .fixed_point_spectral_construction()
            .map_err(String::from)?;

        let winding = collapse_nested_tower(&spectral)?;
        let base = spectral.base().to_vec();
        let carrier = spectral.into_carrier();

        let factor_carrier = match carrier.descend_phase_order(&base, &winding) {
            HadamardDescent::T(carrier) => carrier,
            HadamardDescent::B(_) => {
                return Err(String::from("collapsed membrane produced only a productive fork"));
            }
            HadamardDescent::N(_) => {
                return Err(String::from("collapsed membrane fixed a nonproductive winding"));
            }
            HadamardDescent::F => {
                return Err(String::from("collapsed fixed-point membrane failed"));
            }
        };

        let readout = extract(&factor_carrier)?;
        Ok(FixedPointFactorPair {
            p: readout.p.0,
            q: readout.q.0,
        })
    }
}

/// Collapse successively nested split/fuse membranes until one scale contains
/// the complete return winding. Each scale has m baby deposits and m giant
/// deposits, with m a power of two. The tree itself composes child phases from
/// parent phases, so it performs one modular product per structural right edge
/// instead of recomputing every exponent by square-and-multiply.
///
/// This is not an order walk: there is no carried `a^e -> a^(e+1)` state and no
/// closure-driven TANCH iteration. Every scale is a complete nested membrane;
/// a failed smaller scale is consumed before the next compiled scale opens.
fn collapse_nested_tower(
    spectral: &FixedPointSpectralConstruction,
) -> Result<Tape, String> {
    const FIRST_SCALE: usize = 256;

    let n = spectral.n();
    let identity = one();
    let base_phase = spectral.modular_branch(&identity).map_err(String::from)?;
    let mut m = FIRST_SCALE;

    loop {
        let levels = m.trailing_zeros() as usize;
        let m_tape = branch_ordinal_tape(m);

        // Inner split: leaves are j=0..m-1 in ascending order. Repeated phase
        // deposits overwrite earlier ordinals, retaining the nearest inner leg.
        let inner_jumps = binary_phase_jumps(base_phase.clone(), levels, n);
        let mut deposits = PhaseDeposits::new(n, m)?;
        populate_inner(&identity, 0, m, &inner_jumps, n, &mut deposits)?;

        // Outer split: g=a^m. Its nested leaves are g^i=a^(i*m), again without
        // a linear orbit lane. Scan i=0..m-1, then the closing i=m boundary.
        let giant_seed = inner_jumps[levels].clone();
        let giant_jumps = binary_phase_jumps(giant_seed, levels, n);
        let mut best = None;
        scan_outer(
            spectral,
            &deposits,
            &identity,
            0,
            m,
            &giant_jumps,
            &m_tape,
            &mut best,
        )?;
        consider_outer_deposit(
            spectral,
            &deposits,
            &giant_jumps[levels],
            m,
            &m_tape,
            &mut best,
        )?;

        if let Some(winding) = best {
            return Ok(winding);
        }

        // If m^2 already covers N, every multiplicative order below N was in
        // this membrane. A missing fuse is therefore a structural failure.
        let covered = mul(&m_tape, &m_tape);
        if cmp(&covered, n) != Ordering::Less {
            return Err(String::from(
                "nested spectral membrane covered N without a fixed return winding",
            ));
        }

        m = m
            .checked_mul(2)
            .ok_or_else(|| String::from("compiled membrane scale exceeds host address space"))?;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::morphism_factor::decimal_to_tape;

    #[test]
    fn compiled_membrane_consumes_the_31_bit_semiprime() {
        let n = decimal_to_tape("2031940837").unwrap();
        let pair = CompiledFixedPointMembrane::compiled().consume(n).unwrap();
        let p = decimal_to_tape("54559").unwrap();
        let q = decimal_to_tape("37243").unwrap();
        let direct = pair.p == p && pair.q == q;
        let swapped = pair.p == q && pair.q == p;
        assert!(direct || swapped);
    }
}
