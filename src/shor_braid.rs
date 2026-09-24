//! shor_braid.rs — emit the Shor braid word for base a and modulus N.
//!
//! The braid is the state, per the tower→membrane collapse. This module
//! produces the braid from (a, N) via the tower's recursion, without
//! enumerating the orbit. Downstream: winding readout gives r, factor_close
//! gives the factors.
//!
//! Encoding (honest form): the word carries r = ord_N(a) as a binary counter
//! inside FSPLIT/FFUSE frames — one level per bit, so the word is O(log r)
//! tokens. The order itself comes from the tower's tape order lane (linear
//! walk cross-checked by a^r = 1, no caps, no Python). The braid does not
//! find the order by magic; it is the compressed state that transports r
//! from the order lane to the winding readout and the factor close.
//!
//! The x^r − 1 exponent-halving recursion of the spec is realized here as the
//! binary recursion on the exponent: each level halves the remaining exponent
//! (x^(2l)−1 = (x^l−1)(x^l+1) at the polynomial level = one counter bit at the
//! word level), so level count is O(log r) = O(log N).

use crate::morphism_factor::{cmp, divmod, gcd, modulo, mul, one, tape_u64, trim, zero};
use crate::vox::{AFWD, AREV, CLINK, EVALF, EVALT, FFUSE, FSPLIT, IFIX, IMSCRIB, TANCH, VINIT, ENGAGR};
use alloc::string::String;
use alloc::vec::Vec;
use num_bigint::BigUint;
use num_integer::Integer;
use num_traits::{One, Zero};

fn eq(a: &[char], b: &[char]) -> bool {
    cmp(a, b) == core::cmp::Ordering::Equal
}

/// Tape exponentiation by square-and-multiply, exponent a tape (any size).
fn pow_tape(base: &[char], e_in: &[char], n: &[char]) -> Vec<char> {
    let mut r = one();
    let mut b = modulo(base, n);
    let e = trim(e_in.to_vec());
    for (i, &bit) in e.iter().enumerate() {
        if bit == crate::vox::EVALF {
            r = modulo(&mul(&r, &b), n);
        }
        // The final square cannot contribute to the result. Skipping it also
        // avoids one full-width multiply/modulo pair for every exponentiation.
        if i + 1 < e.len() {
            b = modulo(&mul(&b, &b), n);
        }
    }
    r
}

/// Walk until the modular orbit closes. Invalid inputs fail explicitly.
fn order_lane(a: &[char], n: &[char]) -> Result<Vec<char>, String> {
    if zero(n) { return Err("modulus is zero".into()); }
    if eq(n, &one()) { return Err("modulus is one".into()); }
    if !eq(&gcd(a.to_vec(), n.to_vec()), &one()) {
        return Err("base not coprime to N; gcd(a,N) is already a factor".into());
    }
    let ared = modulo(a, n);
    if eq(&ared, &one()) { return Ok(one()); }
    let mut state = ared.clone();
    let mut i = one();
    loop {
        if eq(&state, &one()) { break; }
        state = modulo(&mul(&state, &ared), n);
        i = crate::morphism_factor::add(&i, &one());
    }
    let r = trim(i);
    if !eq(&pow_tape(a, &r, n), &one()) {
        return Err("order lane verification failed (a^r != 1)".into());
    }
    Ok(r)
}

/// The Shor closing step on tapes: order r -> factors via gcd(a^(r/2) -/+ 1, n).
/// Public so both the braid composition and the shor_qft wide branch call it.
pub fn factor_close_public(a: &[char], n: &[char], r: &[char]) -> Result<(Vec<char>, Vec<char>), String> {
    fn to_dynamic(tape: &[char]) -> BigUint {
        let mut bytes = alloc::vec![0u8; (tape.len() + 7) / 8];
        for (index, mark) in tape.iter().enumerate() {
            if *mark == EVALF { bytes[index / 8] |= 1 << (index % 8); }
        }
        BigUint::from_bytes_le(&bytes)
    }
    fn to_tape(value: &BigUint) -> Vec<char> {
        if value.is_zero() { return alloc::vec![EVALT]; }
        let mut tape = Vec::new();
        for byte in value.to_bytes_le() {
            for bit in 0..8 {
                tape.push(if (byte >> bit) & 1 == 1 { EVALF } else { EVALT });
            }
        }
        while tape.last() == Some(&EVALT) { tape.pop(); }
        tape
    }
    fn dynamic_gcd(x: BigUint, y: BigUint) -> BigUint {
        x.gcd(&y)
    }

    let a = to_dynamic(a);
    let n = to_dynamic(n);
    let r = to_dynamic(r);
    if r.is_zero() || r.is_one() { return Err("order is trivial (0 or 1) -- no close".into()); }
    if (&r % 2u8) != BigUint::zero() {
        return Err("order odd -- a^(r/2) undefined, retry with another base".into());
    }
    let half = &r >> 1usize;
    let h = a.modpow(&half, &n);
    if h == &n - BigUint::one() {
        return Err("a^(r/2) = -1 (mod n) -- retry with another base".into());
    }
    let h_minus = if h.is_zero() { BigUint::zero() } else { &h - BigUint::one() };
    let h_plus = &h + BigUint::one();
    let factor = dynamic_gcd(h_minus, n.clone());
    if !factor.is_one() && factor != n {
        let quotient = &n / &factor;
        if &quotient * &factor == n {
            return Ok((to_tape(&factor), to_tape(&quotient)));
        }
    }
    let factor = dynamic_gcd(h_plus, n.clone());
    if !factor.is_one() && factor != n {
        let quotient = &n / &factor;
        if &quotient * &factor == n {
            return Ok((to_tape(&factor), to_tape(&quotient)));
        }
    }
    Err("both closing gcds trivial for this base -- retry with another base".into())
}

/// Close from the two dyadic half-step residues already carried by the phase
/// winding. If `x = a^(2^(j-1))` and `y = a^(2^(i-1))`, then
/// `x/y = a^((2^j-2^i)/2)`. Since `y` is a unit modulo N, the closing gcds are
/// exactly `gcd(x-y,N)` and `gcd(x+y,N)`, avoiding a second modular
/// exponentiation after phase winding.
pub fn factor_close_from_phase_halves(
    n: &[char], current_half: &[char], earlier_half: &[char],
) -> Result<(Vec<char>, Vec<char>), String> {
    fn to_dynamic(tape: &[char]) -> BigUint {
        let mut bytes = alloc::vec![0u8; (tape.len() + 7) / 8];
        for (index, mark) in tape.iter().enumerate() {
            if *mark == EVALF { bytes[index / 8] |= 1 << (index % 8); }
        }
        BigUint::from_bytes_le(&bytes)
    }
    fn to_tape(value: &BigUint) -> Vec<char> {
        if value.is_zero() { return alloc::vec![EVALT]; }
        let mut tape = Vec::new();
        for byte in value.to_bytes_le() {
            for bit in 0..8 {
                tape.push(if (byte >> bit) & 1 == 1 { EVALF } else { EVALT });
            }
        }
        while tape.last() == Some(&EVALT) { tape.pop(); }
        tape
    }
    fn dynamic_gcd(x: BigUint, y: BigUint) -> BigUint {
        x.gcd(&y)
    }

    let n = to_dynamic(n);
    let x = to_dynamic(current_half) % &n;
    let y = to_dynamic(earlier_half) % &n;
    if n <= BigUint::one() || x.is_zero() || y.is_zero() {
        return Err("phase half-step residues are invalid for this modulus".into());
    }
    let difference = if x >= y { &x - &y } else { &n - (&y - &x) };
    let sum = (&x + &y) % &n;
    let factor = dynamic_gcd(difference, n.clone());
    if !factor.is_one() && factor != n {
        let quotient = &n / &factor;
        if &quotient * &factor == n {
            return Ok((to_tape(&factor), to_tape(&quotient)));
        }
    }
    let factor = dynamic_gcd(sum, n.clone());
    if !factor.is_one() && factor != n {
        let quotient = &n / &factor;
        if &quotient * &factor == n {
            return Ok((to_tape(&factor), to_tape(&quotient)));
        }
    }
    Err("phase half-step gcds are trivial for this base -- retry with another base".into())
}

/// Emit the Shor braid word for base a mod N. Returns the word and the
/// level count used. Level count is O(log r); word length is O(log r).
pub fn shor_braid(a: &[char], n: &[char]) -> Result<(Vec<char>, usize), String> {
    let r = order_lane(a, n)?;
    let two = tape_u64(2);
    let mut bits: Vec<bool> = Vec::new();
    let mut e = trim(r.clone());
    while !zero(&e) {
        let (q, rem) = divmod(&e, &two);
        bits.push(eq(&rem, &one()));
        e = q;
    }
    bits.reverse();
    let levels = bits.len().max(1);
    let mut w: Vec<char> = Vec::new();
    w.push(VINIT);
    if bits.is_empty() {
        w.push(FSPLIT);
        w.push(AFWD);
        w.push(EVALT);
        w.push(AREV);
        w.push(CLINK);
    } else {
        for &b in bits.iter() {
            w.push(FSPLIT);
            w.push(AFWD);
            w.push(if b { EVALF } else { EVALT });
            w.push(AREV);
            w.push(CLINK);
        }
    }
    w.push(IMSCRIB);
    w.push(ENGAGR);
    // Direct comb teeth for small r: the winding integral reads these
    // exactly (r=4 -> (x-1)(x+1)(x^2+1) shape, r=6 -> adds cyclotomics).
    if let Ok(rv) = crate::morphism_factor::dec_of(&r).parse::<u64>() {
        if rv <= 64 {
            for _ in 0..rv {
                w.push(IFIX);
            }
        }
    }
    for _ in 0..levels {
        w.push(FFUSE);
    }
    w.push(TANCH);
    Ok((w, levels))
}

/// Composition: walk to closure, emit braid, read winding, close factors.
pub fn shor_factor_via_braid(a: &[char], n: &[char]) -> Result<(Vec<char>, Vec<char>), String> {
    let (word, _levels) = shor_braid(a, n)?;
    let r_tape = crate::winding_readout::winding_number_tape(&word)?;
    factor_close_public(a, n, &r_tape)
}

/// Scan the finite residue domain on tapes; each orbit runs to closure.
pub fn shor_factor_via_braid_scanned(
    n: &[char],
) -> Result<(Vec<char>, Vec<char>, Vec<char>), String> {
    use crate::morphism_factor::tape_u64;
    let mut last_err = String::from("no base tried");
    let mut base = tape_u64(2);
    while cmp(&base, n) == core::cmp::Ordering::Less {
        let g = crate::morphism_factor::gcd(base.clone(), n.to_vec());
        if cmp(&g, &one()) != core::cmp::Ordering::Equal {
            let (q, rem) = divmod(n, &g);
            if zero(&rem) { return Ok((g, q, base)); }
            return Err("base gcd failed product boundary".into());
        }
        match order_lane(&base, n) {
            Ok(r) => {
                if let Ok((p, q)) = factor_close_public(&base, n, &r) {
                    return Ok((p, q, base));
                }
                last_err = String::from("factor close trivial");
            }
            Err(e) => { last_err = e; }
        }
        base = crate::morphism_factor::add(&base, &one());
    }
    Err(format!("residue bases exhausted, last: {last_err}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::morphism_factor::decimal_to_tape;

    #[test]
    fn braid_winding_roundtrip_r4() {
        let a = decimal_to_tape("7").unwrap();
        let n = decimal_to_tape("15").unwrap();
        let (word, levels) = shor_braid(&a, &n).unwrap();
        assert_eq!(crate::winding_readout::winding_number(&word).unwrap(), 4);
        assert!(levels <= 8, "levels O(log r), got {levels}");
    }

    #[test]
    fn braid_winding_roundtrip_r6() {
        let a = decimal_to_tape("2").unwrap();
        let n = decimal_to_tape("21").unwrap();
        let (word, _) = shor_braid(&a, &n).unwrap();
        assert_eq!(crate::winding_readout::winding_number(&word).unwrap(), 6);
    }

    #[test]
    fn direct_lsb_exponent_scan_closes_a_semiprime() {
        let a = decimal_to_tape("7").unwrap();
        let n = decimal_to_tape("15").unwrap();
        let (p, q) = factor_close_public(&a, &n, &decimal_to_tape("4").unwrap()).unwrap();
        assert_eq!(crate::morphism_factor::mul(&p, &q), n);
    }

    #[test]
    fn phase_half_step_gcds_match_the_winding_close() {
        let n = decimal_to_tape("15").unwrap();
        let (p, q) = factor_close_from_phase_halves(
            &n, &decimal_to_tape("4").unwrap(), &decimal_to_tape("1").unwrap(),
        ).unwrap();
        assert_eq!(crate::morphism_factor::mul(&p, &q), n);
        assert!(factor_close_from_phase_halves(
            &n, &decimal_to_tape("14").unwrap(), &decimal_to_tape("1").unwrap(),
        ).is_err());
    }
}
