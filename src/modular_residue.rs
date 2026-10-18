//! Exact skip-ahead on a certified modular orbit.
use alloc::string::String;
use alloc::vec::Vec;
use num_bigint::BigUint;
use num_traits::{One, Zero, ToPrimitive};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModularResidueStep {
    pub tick: BigUint,
    pub period: BigUint,
    pub residue: BigUint,
    pub value: BigUint,
}

/// A source-bound period certificate. Fields stay private so a caller cannot
/// reuse the certificate with another base, modulus, or period.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CertifiedModularOrbit {
    base: BigUint,
    modulus: BigUint,
    period: BigUint,
}

impl CertifiedModularOrbit {
    pub fn certify_tapes(base: &[char], modulus: &[char], period: &[char]) -> Result<Self, String> {
        Self::certify(&tape_value(base)?, &tape_value(modulus)?, &tape_value(period)?)
    }

    pub fn step_tape(&self, tick: &[char]) -> Result<Vec<char>, String> {
        Ok(value_tape(&self.step(&tape_value(tick)?).value))
    }

    pub fn certify(base: &BigUint, modulus: &BigUint, period: &BigUint) -> Result<Self, String> {
        if *modulus < BigUint::from(2u8) || period.is_zero() {
            return Err("modular residue step requires modulus >= 2 and a positive period".into());
        }
        let base = base % modulus;
        if base.modpow(period, modulus) != BigUint::one() {
            return Err("modular residue step rejected an uncertified period".into());
        }
        Ok(Self { base, modulus: modulus.clone(), period: period.clone() })
    }

    pub fn base(&self) -> &BigUint { &self.base }
    pub fn modulus(&self) -> &BigUint { &self.modulus }
    pub fn period(&self) -> &BigUint { &self.period }

    /// Evaluate only the residue, with no orbit walk or recertification.
    /// A certified period need not be the minimal period.
    pub fn step(&self, tick: &BigUint) -> ModularResidueStep {
        let residue = tick % &self.period;
        let value = self.base.modpow(&residue, &self.modulus);
        ModularResidueStep { tick: tick.clone(), period: self.period.clone(), residue, value }
    }
}

pub fn certified_modular_residue_step(
    tick: &BigUint, base: &BigUint, modulus: &BigUint, period: &BigUint,
) -> Result<ModularResidueStep, String> {
    Ok(CertifiedModularOrbit::certify(base, modulus, period)?.step(tick))
}

pub fn certified_modular_residue_u64_step(
    tick: u64, base: u64, modulus: u64, period: u64,
) -> Result<u64, String> {
    let step = certified_modular_residue_step(&tick.into(), &base.into(), &modulus.into(), &period.into())?;
    step.value.to_u64().ok_or_else(|| "modular residue exceeds its native modulus".into())
}

pub fn certified_modular_residue_tape_step(
    tick: &[char], base: &[char], modulus: &[char], period: &[char],
) -> Result<Vec<char>, String> {
    CertifiedModularOrbit::certify_tapes(base, modulus, period)?.step_tape(tick)
}

fn tape_value(tape: &[char]) -> Result<BigUint, String> {
    use crate::vox::{EVALT, EVALF};
    let mut bytes = alloc::vec![0u8; (tape.len() + 7) / 8];
    for (index, mark) in tape.iter().enumerate() {
        match *mark {
            EVALF => bytes[index / 8] |= 1 << (index % 8),
            EVALT => {},
            _ => return Err("modular residue requires a binary numeral tape".into()),
        }
    }
    Ok(BigUint::from_bytes_le(&bytes))
}

fn value_tape(value: &BigUint) -> Vec<char> {
    use crate::vox::{EVALT, EVALF};
    if value.is_zero() { return alloc::vec![EVALT]; }
    let mut tape = Vec::new();
    for byte in value.to_bytes_le() {
        for bit in 0..8 { tape.push(if (byte >> bit) & 1 == 1 { EVALF } else { EVALT }); }
    }
    while tape.last() == Some(&EVALT) { tape.pop(); }
    tape
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arbitrary_width_ticks_and_nonminimal_periods_preserve_the_orbit() {
        let n = BigUint::from(15u8);
        let base = BigUint::from(17u8);
        let orbit = CertifiedModularOrbit::certify(&base, &n, &BigUint::from(8u8)).unwrap();
        let tick = (BigUint::one() << 2048usize) + BigUint::from(5u8);
        let step = orbit.step(&tick);
        assert_eq!(step.residue, BigUint::from(5u8));
        assert_eq!(step.value, base.modpow(&tick, &n));
        assert_eq!(orbit.step(&BigUint::zero()).value, BigUint::one());
        assert_eq!(orbit.step(&(&tick + orbit.period())).value, step.value);
    }

    #[test]
    fn invalid_boundaries_and_nonunits_cannot_create_certificates() {
        for (base, n, period) in [(2u8,0u8,4u8),(2,1,4),(2,15,0),(3,15,4),(2,15,3)] {
            assert!(CertifiedModularOrbit::certify(&base.into(), &n.into(), &period.into()).is_err());
        }
    }

    #[test]
    fn native_and_tape_surfaces_share_the_arbitrary_width_certificate() {
        use crate::morphism_factor::tape_u64;
        assert_eq!(certified_modular_residue_u64_step(1_000_000_000_000_000_001, 2, 15, 4).unwrap(), 2);
        let orbit = CertifiedModularOrbit::certify_tapes(&tape_u64(2), &tape_u64(15), &tape_u64(4)).unwrap();
        let tick = (BigUint::one() << 2048usize) + BigUint::one();
        assert_eq!(orbit.step_tape(&value_tape(&tick)).unwrap(), tape_u64(2));
        assert!(orbit.step_tape(&['∈']).is_err());
        assert!(CertifiedModularOrbit::certify_tapes(&['∈'], &tape_u64(15), &tape_u64(4)).is_err());
        assert!(certified_modular_residue_u64_step(1,2,15,3).is_err());
    }
}

// Retained u64 period-certified helper.
pub mod u64_period_helper {
use alloc::string::String;
// modular_residue.rs — modular exponentiation with period certification.

fn mod_pow_u64(mut a: u64, mut e: u64, n: u64) -> u64 {
    if n == 1 { return 0; }
    let mut r = 1u64;
    a %= n;
    while e > 0 {
        if e & 1 == 1 {
            r = (r as u128 * a as u128 % n as u128) as u64;
        }
        a = (a as u128 * a as u128 % n as u128) as u64;
        e >>= 1;
    }
    r
}

/// Return `base^exp mod modulus`, certified against `period`:
/// fails if `base^period mod modulus != 1`.
pub fn certified_modular_residue_u64_step(
    exp: u64,
    base: u64,
    modulus: u64,
    period: u64,
) -> Result<u64, String> {
    if modulus == 0 {
        return Err("modulus is zero".into());
    }
    if period == 0 {
        return Err("period is zero".into());
    }
    let check = mod_pow_u64(base, period, modulus);
    if check != 1 {
        return Err(format!(
            "certification failed: {} ^ {} mod {} = {} (expected 1)",
            base, period, modulus, check,
        ));
    }
    Ok(mod_pow_u64(base, exp, modulus))
}

#[cfg(test)]
mod u64_step_tests {
    use super::*;

    #[test]
    fn certifies_known_period() {
        // 2^6 ≡ 1 (mod 21); 2^3 = 8 (mod 21)
        assert_eq!(certified_modular_residue_u64_step(3, 2, 21, 6).unwrap(), 8);
    }

    #[test]
    fn rejects_wrong_period() {
        // 2^5 = 32 ≡ 11 (mod 21), not 1
        assert!(certified_modular_residue_u64_step(2, 2, 21, 5).is_err());
    }

    #[test]
    fn edge_case_n_equals_1() {
        // modulus 1 → result always 0
        assert_eq!(certified_modular_residue_u64_step(5, 3, 1, 1).unwrap(), 0);
    }
}
}
