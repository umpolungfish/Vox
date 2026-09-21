//! Collapsed fixed-point spectral membrane.
//!
//! The repaired 25-op boundary exposes only three fixed windings. A compiled
//! tower must not recover larger windings by walking TANCH one step at a time.
//! This module instead compiles the spectral observation into a nested
//! split/fuse membrane over two exponent coordinates. Each modular branch is
//! evaluated independently by the existing tape-native square-and-multiply
//! operator; no residue from one exponent is advanced to obtain the next.
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
use crate::morphism_factor::{add, cmp, isqrt, mul, one, sub, zero};
use crate::vox::{EVALF, EVALT};

#[derive(Clone, PartialEq, Debug)]
pub struct FixedPointFactorPair {
    pub p: Tape,
    pub q: Tape,
}

/// Exact folded storage for one spectral phase deposit.
///
/// This is storage compression only. Modular arithmetic continues to operate on
/// IMASM numeral tapes. Up to 64 cells fit in one limb without a heap allocation;
/// wider phases remain arbitrary-width and are stored as a boxed limb slice.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
enum PackedPhase {
    OneLimb(u64),
    Many(Box<[u64]>),
}

fn pack_phase(tape: &[char]) -> PackedPhase {
    let limbs = (tape.len() + 63) / 64;
    if limbs <= 1 {
        let mut value = 0u64;
        for (bit, &mark) in tape.iter().enumerate() {
            if mark == EVALF {
                value |= 1u64 << bit;
            }
        }
        return PackedPhase::OneLimb(value);
    }

    let mut out = alloc::vec![0u64; limbs];
    for (bit, &mark) in tape.iter().enumerate() {
        if mark == EVALF {
            out[bit / 64] |= 1u64 << (bit % 64);
        }
    }
    PackedPhase::Many(out.into_boxed_slice())
}

/// Re-materialize a physical inner-branch ordinal as an IMASM exponent label.
/// The usize is only a storage address into the compiled membrane population;
/// modular arithmetic never consumes it directly.
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

/// The executable image of the properly nested fixed-point tower.
///
/// It is zero-sized because the compiled code is the membrane. `consume`
/// takes ownership of both the membrane and N and returns only the factor pair.
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

fn ceil_sqrt(n: &[char]) -> Tape {
    let root = isqrt(n);
    if cmp(&mul(&root, &root), n) == Ordering::Less {
        add(&root, &one())
    } else {
        root
    }
}

/// Collapse the nested spectral tower into one fixed winding.
///
/// Let m = ceil(sqrt(N)). The inner membrane fixes independently evaluated
/// branches a^j for 0 <= j < m. The outer membrane fixes independently
/// evaluated branches a^(i*m) for 1 <= i <= m. Equal phase deposits fuse.
/// Their exponent difference is a return winding. Because the multiplicative
/// order is < N <= m^2, a fuse exists inside this membrane. Keeping the latest
/// inner representative makes the first outer fuse the least positive return
/// even when the order is smaller than m.
///
/// Crucially, this is not a modular-orbit walk: every branch calls
/// `modular_branch(exponent)` from its exponent label. No a^e residue is used
/// to construct a^(e+1), and TANCH is never iterated to discover the winding.
fn collapse_nested_tower(
    spectral: &FixedPointSpectralConstruction,
) -> Result<Tape, String> {
    let m = ceil_sqrt(spectral.n());
    let one_t = one();
    let zero_t = sub(&one_t, &one_t);

    // Inner split: one independently evaluated phase deposit for every j < m.
    // Phase values are folded only after evaluation to compact the membrane's
    // resident deposit table. Repeated phases retain the latest branch ordinal.
    let mut inner: BTreeMap<PackedPhase, usize> = BTreeMap::new();
    let mut j = zero_t;
    let mut j_ordinal = 0usize;
    while cmp(&j, &m) == Ordering::Less {
        let phase = spectral.modular_branch(&j).map_err(String::from)?;
        inner.insert(pack_phase(&phase), j_ordinal);
        j = add(&j, &one_t);
        j_ordinal = j_ordinal
            .checked_add(1)
            .ok_or_else(|| String::from("compiled membrane branch address overflow"))?;
    }

    // Outer split: each i*m branch is also evaluated from its label, not from
    // the previous outer residue. The first equal deposit is the membrane fuse.
    let mut i = one_t.clone();
    while cmp(&i, &m) != Ordering::Greater {
        let exponent = mul(&i, &m);
        let phase = spectral.modular_branch(&exponent).map_err(String::from)?;
        if let Some(inner_ordinal) = inner.get(&pack_phase(&phase)) {
            let inner_exponent = branch_ordinal_tape(*inner_ordinal);
            let winding = sub(&exponent, &inner_exponent);
            if !zero(&winding) {
                let closure = spectral.modular_branch(&winding).map_err(String::from)?;
                if cmp(&closure, &one_t) == Ordering::Equal {
                    return Ok(winding);
                }
            }
        }
        i = add(&i, &one_t);
    }

    Err(String::from("nested spectral membrane exposed no fixed return winding"))
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
