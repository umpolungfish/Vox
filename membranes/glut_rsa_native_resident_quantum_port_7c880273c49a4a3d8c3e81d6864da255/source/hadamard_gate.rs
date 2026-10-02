//! Tape-native symbolic Hadamard and fixed-point spectral construction.
//!
//! This module is upstream of passive factor extraction.  The initial carrier
//! owns only N; p and q are not fields and cannot be smuggled into the seed.
//! A Hadamard boundary is represented by its exact Walsh character
//! `(-1)^<x,y>` over IMASM numeral tapes.  No 2^k matrix is materialized and no
//! tape is decoded to a host integer.
//!
//! The fixed-point spectral layer adds the structure that the bare Walsh row
//! does not contain by itself:
//!
//! * an N-dependent modular phase operator, evaluated symbolically on a basis
//!   exponent without walking an orbit;
//! * an exact cyclic winding `laps + residue / denominator`;
//! * the residual phase involution w -> -w (mod 1), whose fixed points are the
//!   order-two phases 0 and 1/2;
//! * an explicit split/fuse boundary with `mu(delta(w)) = w`.
//!
//! None of these operations discovers a factor or a period by search.  They are
//! the exact structural substrate on which an observation boundary can act.
//! The returned +/-1 coefficient is an amplitude sign, not a FOUR judgment.
//! FOUR remains responsible for judging the exposed carrier relationship.

use alloc::vec;
use alloc::vec::Vec;

use crate::morphism_factor::{add, divmod, modulo, mul, one, sub, trim, zero};
use crate::vox::{EVALF, EVALT};

pub type Tape = Vec<char>;

fn validate_numeral_tape(tape: &[char]) -> Result<(), &'static str> {
    if tape.is_empty() {
        return Err("Hadamard numeral tape is empty");
    }
    if tape.iter().any(|&mark| mark != EVALT && mark != EVALF) {
        return Err("Hadamard numeral tape contains a non-numeral mark");
    }
    Ok(())
}

/// Exact Walsh/Hadamard character on arbitrary-width numeral tapes.
/// Missing high cells are EVALT/zero; no host-width conversion occurs.
pub fn hadamard_character(source: &[char], target: &[char]) -> Result<i8, &'static str> {
    validate_numeral_tape(source)?;
    validate_numeral_tape(target)?;

    let width = source.len().max(target.len());
    let mut odd = false;
    for i in 0..width {
        let a = source.get(i).copied().unwrap_or(EVALT) == EVALF;
        let b = target.get(i).copied().unwrap_or(EVALT) == EVALF;
        odd ^= a && b;
    }
    Ok(if odd { -1 } else { 1 })
}

/// Tape-native XOR for composing Hadamard characters.  This is the additive
/// boundary law over F2, expressed without decoding either tape.
pub fn xor_tape(left: &[char], right: &[char]) -> Result<Tape, &'static str> {
    validate_numeral_tape(left)?;
    validate_numeral_tape(right)?;

    let width = left.len().max(right.len());
    let mut out = Vec::with_capacity(width);
    for i in 0..width {
        let a = left.get(i).copied().unwrap_or(EVALT) == EVALF;
        let b = right.get(i).copied().unwrap_or(EVALT) == EVALF;
        out.push(if a ^ b { EVALF } else { EVALT });
    }
    Ok(trim(out))
}

/// Tape-native modular exponentiation for the phase operator.  The exponent is
/// consumed LSB-first directly from its numeral tape; no host-width integer is
/// reconstructed and no successive orbit is walked to discover an order.
pub fn modular_phase_power(
    base: &[char],
    exponent: &[char],
    n: &[char],
) -> Result<Tape, &'static str> {
    validate_numeral_tape(base)?;
    validate_numeral_tape(exponent)?;
    validate_numeral_tape(n)?;
    if zero(n) {
        return Err("modular phase modulus is zero");
    }

    let mut result = one();
    let mut power = modulo(base, n);
    for &cell in exponent {
        if cell == EVALF {
            result = modulo(&mul(&result, &power), n);
        }
        power = modulo(&mul(&power, &power), n);
    }
    Ok(result)
}

/// Exact cyclic winding carried without floating point.
///
/// `raw_numerator / denominator` is retained as
/// `laps + residue / denominator`.  The integer lap count is deliberately kept
/// separate from the residual phase: Fourier phase only sees the residue mod 1,
/// while the fixed-point construction also needs to know whether a nonzero
/// integer winding was present before that quotient was forgotten.
#[derive(Clone, PartialEq, Debug)]
pub struct SpectralWinding {
    laps: Tape,
    residue: Tape,
    denominator: Tape,
}

impl SpectralWinding {
    pub fn new(raw_numerator: &[char], denominator: &[char]) -> Result<Self, &'static str> {
        validate_numeral_tape(raw_numerator)?;
        validate_numeral_tape(denominator)?;
        if zero(denominator) {
            return Err("spectral winding denominator is zero");
        }

        let denominator = trim(denominator.to_vec());
        let (laps, residue) = divmod(raw_numerator, &denominator);
        Ok(Self {
            laps: trim(laps),
            residue: trim(residue),
            denominator,
        })
    }

    pub fn laps(&self) -> &[char] {
        &self.laps
    }

    pub fn residue(&self) -> &[char] {
        &self.residue
    }

    pub fn denominator(&self) -> &[char] {
        &self.denominator
    }

    pub fn has_nonzero_integer_winding(&self) -> bool {
        !zero(&self.laps)
    }

    /// Residual phase conjugation w -> -w (mod 1).  The integer lap count is
    /// retained as topological metadata; the involution acts on the cyclic
    /// phase fibre seen by the spectral/Hadamard boundary.
    pub fn z2_involution(&self) -> Self {
        let residue = if zero(&self.residue) {
            self.residue.clone()
        } else {
            sub(&self.denominator, &self.residue)
        };
        Self {
            laps: self.laps.clone(),
            residue: trim(residue),
            denominator: self.denominator.clone(),
        }
    }

    /// Fixed points of residual conjugation are exactly the self-inverse phases:
    /// 0, and 1/2 when the denominator admits it.  The arithmetic statement is
    /// simply 2*residue = 0 mod denominator.
    pub fn is_z2_fixed_point(&self) -> bool {
        let twice = add(&self.residue, &self.residue);
        zero(&modulo(&twice, &self.denominator))
    }

    /// Retract only an actual Z2 fixed point to a real Hadamard sign.  General
    /// cyclic phases intentionally return None rather than being rounded into a
    /// false +/-1 character.
    pub fn fixed_point_sign(&self) -> Option<i8> {
        if !self.is_z2_fixed_point() {
            None
        } else if zero(&self.residue) {
            Some(1)
        } else {
            Some(-1)
        }
    }

    /// Delta duplicates the complete spectral label.  This is basis-state copy,
    /// not an amplitude clone: it is the explicit split boundary used by the
    /// fixed-point grammar.
    pub fn delta(&self) -> SpectralSplit {
        SpectralSplit {
            left: self.clone(),
            right: self.clone(),
        }
    }
}

/// Two legs of the exact spectral split.  `mu` only fuses equal legs, making
/// `mu(delta(w)) = w` an executable identity rather than a comment.
#[derive(Clone, PartialEq, Debug)]
pub struct SpectralSplit {
    left: SpectralWinding,
    right: SpectralWinding,
}

impl SpectralSplit {
    pub fn left(&self) -> &SpectralWinding {
        &self.left
    }

    pub fn right(&self) -> &SpectralWinding {
        &self.right
    }

    pub fn mu(self) -> Result<SpectralWinding, &'static str> {
        if self.left == self.right {
            Ok(self.left)
        } else {
            Err("spectral split legs do not fuse")
        }
    }
}

/// Current whole object at the Hadamard boundary.
///
/// `new` seeds the production path with N and the zero source only.  A prior
/// phase/modular boundary can write a new source by consuming this object with
/// `imscribe_source`; the resulting object is then the one read by `sign_at`.
#[derive(Clone, PartialEq, Debug)]
pub struct HadamardCarrier {
    n: Tape,
    source: Tape,
}

/// N-only symbolic program for the missing fixed-point spectral layer.
///
/// The canonical first modular generator is 2.  This object does not claim that
/// base 2 is always a productive factoring base, nor does it synthesize a phase
/// measurement.  Its purpose is narrower and exact: starting from N alone, it
/// creates the N-dependent modular operator that the formerly uniform Hadamard
/// seed lacked, while keeping phase observation as an explicit later boundary.
#[derive(Clone, PartialEq, Debug)]
pub struct FixedPointSpectralConstruction {
    carrier: HadamardCarrier,
    base: Tape,
}

impl FixedPointSpectralConstruction {
    pub fn n(&self) -> &[char] {
        self.carrier.n()
    }

    pub fn base(&self) -> &[char] {
        &self.base
    }

    pub fn carrier(&self) -> &HadamardCarrier {
        &self.carrier
    }

    pub fn into_carrier(self) -> HadamardCarrier {
        self.carrier
    }

    /// Evaluate one basis branch of the controlled modular phase operator.
    /// The full spectrum remains symbolic; this method does not enumerate
    /// exponents or search for a closing period.
    pub fn modular_branch(&self, exponent: &[char]) -> Result<Tape, &'static str> {
        modular_phase_power(&self.base, exponent, self.carrier.n())
    }

    pub fn winding(
        &self,
        raw_numerator: &[char],
        denominator: &[char],
    ) -> Result<SpectralWinding, &'static str> {
        SpectralWinding::new(raw_numerator, denominator)
    }
}

impl HadamardCarrier {
    /// Seed from N only.  No factor witness exists in this object.
    pub fn new(n: &[char]) -> Result<Self, &'static str> {
        validate_numeral_tape(n)?;
        Ok(Self {
            n: trim(n.to_vec()),
            source: vec![EVALT],
        })
    }

    pub fn n(&self) -> &[char] {
        &self.n
    }

    pub fn source(&self) -> &[char] {
        &self.source
    }

    /// Bind the N-only seed to the first exact modular phase operator.  The
    /// canonical tape `[0,1]` is the numeral 2 in the crate's LSB-first numeral
    /// convention; no factor, period, or phase sample is supplied here.
    pub fn fixed_point_spectral_construction(
        self,
    ) -> Result<FixedPointSpectralConstruction, &'static str> {
        if zero(&self.n) {
            return Err("fixed-point spectral modulus is zero");
        }
        Ok(FixedPointSpectralConstruction {
            carrier: self,
            base: vec![EVALT, EVALF],
        })
    }

    /// Consume the current whole object and write the source exposed by the
    /// preceding phase boundary into the succeeding Hadamard carrier.
    pub fn imscribe_source(mut self, source: &[char]) -> Result<Self, &'static str> {
        validate_numeral_tape(source)?;
        self.source = trim(source.to_vec());
        Ok(self)
    }

    /// Read one exact coefficient of the symbolic Hadamard row.
    pub fn sign_at(&self, target: &[char]) -> Result<i8, &'static str> {
        hadamard_character(&self.source, target)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::morphism_factor::tape_u64;

    #[test]
    fn n_only_seed_opens_the_uniform_hadamard_row() {
        let n = tape_u64(8051);
        let carrier = HadamardCarrier::new(&n).unwrap();
        assert_eq!(carrier.n(), n.as_slice());
        assert_eq!(carrier.source(), [EVALT]);

        for target in 0u64..16 {
            assert_eq!(carrier.sign_at(&tape_u64(target)).unwrap(), 1);
        }
    }

    #[test]
    fn fixed_point_spectral_construction_binds_real_modular_dynamics_from_n_only() {
        let seed = HadamardCarrier::new(&tape_u64(15)).unwrap();
        let spectral = seed.fixed_point_spectral_construction().unwrap();

        assert_eq!(spectral.n(), tape_u64(15).as_slice());
        assert_eq!(spectral.base(), tape_u64(2).as_slice());
        assert_eq!(spectral.carrier().source(), [EVALT]);
        assert_eq!(spectral.modular_branch(&tape_u64(0)).unwrap(), tape_u64(1));
        assert_eq!(spectral.modular_branch(&tape_u64(1)).unwrap(), tape_u64(2));
        assert_eq!(spectral.modular_branch(&tape_u64(2)).unwrap(), tape_u64(4));
        assert_eq!(spectral.modular_branch(&tape_u64(4)).unwrap(), tape_u64(1));
    }

    #[test]
    fn spectral_delta_mu_is_an_exact_fixed_point_identity() {
        let winding = SpectralWinding::new(&tape_u64(14), &tape_u64(4)).unwrap();
        assert_eq!(winding.laps(), tape_u64(3).as_slice());
        assert_eq!(winding.residue(), tape_u64(2).as_slice());
        assert!(winding.has_nonzero_integer_winding());
        assert!(winding.is_z2_fixed_point());
        assert_eq!(winding.fixed_point_sign(), Some(-1));

        let split = winding.delta();
        assert_eq!(split.left(), &winding);
        assert_eq!(split.right(), &winding);
        assert_eq!(split.mu().unwrap(), winding);
    }

    #[test]
    fn residual_phase_involution_is_z2_without_rounding_general_winding() {
        let third = SpectralWinding::new(&tape_u64(1), &tape_u64(3)).unwrap();
        assert!(!third.is_z2_fixed_point());
        assert_eq!(third.fixed_point_sign(), None);

        let conjugate = third.z2_involution();
        assert_eq!(conjugate.residue(), tape_u64(2).as_slice());
        assert_eq!(conjugate.z2_involution(), third);
    }

    #[test]
    fn integer_winding_survives_the_residual_phase_involution() {
        let winding = SpectralWinding::new(&tape_u64(13), &tape_u64(4)).unwrap();
        assert_eq!(winding.laps(), tape_u64(3).as_slice());
        assert_eq!(winding.residue(), tape_u64(1).as_slice());
        assert!(winding.has_nonzero_integer_winding());

        let conjugate = winding.z2_involution();
        assert_eq!(conjugate.laps(), winding.laps());
        assert_eq!(conjugate.residue(), tape_u64(3).as_slice());
        assert_eq!(conjugate.z2_involution(), winding);
    }

    #[test]
    fn symbolic_gate_matches_the_sylvester_character_without_a_matrix() {
        let n = tape_u64(8051);
        for source in 0u64..16 {
            let carrier = HadamardCarrier::new(&n)
                .unwrap()
                .imscribe_source(&tape_u64(source))
                .unwrap();
            for target in 0u64..16 {
                let expected = if (source & target).count_ones() & 1 == 0 { 1 } else { -1 };
                assert_eq!(carrier.sign_at(&tape_u64(target)).unwrap(), expected);
            }
        }
    }

    #[test]
    fn character_composes_on_tapes_and_second_gate_closes() {
        let n = tape_u64(8051);
        for a in 0u64..16 {
            for b in 0u64..16 {
                let ab = xor_tape(&tape_u64(a), &tape_u64(b)).unwrap();
                for k in 0u64..16 {
                    let target = tape_u64(k);
                    let lhs = hadamard_character(&ab, &target).unwrap();
                    let rhs = hadamard_character(&tape_u64(a), &target).unwrap()
                        * hadamard_character(&tape_u64(b), &target).unwrap();
                    assert_eq!(lhs, rhs, "a={a}, b={b}, k={k}");
                }
            }
        }

        // H H = I up to the unmaterialized normalization: the exact row inner
        // product is 16 on the same basis label and zero on every other label.
        for source in 0u64..16 {
            let carrier = HadamardCarrier::new(&n)
                .unwrap()
                .imscribe_source(&tape_u64(source))
                .unwrap();
            for other in 0u64..16 {
                let mut inner = 0i64;
                for target in 0u64..16 {
                    let y = tape_u64(target);
                    inner += i64::from(carrier.sign_at(&y).unwrap())
                        * i64::from(hadamard_character(&tape_u64(other), &y).unwrap());
                }
                assert_eq!(inner, if source == other { 16 } else { 0 });
            }
        }
    }

    #[test]
    fn character_is_not_limited_to_a_host_word() {
        let mut source = vec![EVALT; 130];
        let mut target = vec![EVALT; 130];
        source[129] = EVALF;
        target[129] = EVALF;
        assert_eq!(hadamard_character(&source, &target).unwrap(), -1);

        target[64] = EVALF;
        assert_eq!(hadamard_character(&source, &target).unwrap(), -1);
        source[64] = EVALF;
        assert_eq!(hadamard_character(&source, &target).unwrap(), 1);
    }
}
