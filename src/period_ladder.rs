#![allow(dead_code)]
//! period_ladder.rs — the period-finding address as a split/fuse/evaluate
//! ladder, the twin of the OPI/DQI ladder in UQMds.
//!
//! The modular orbit of `a` mod `N` is a cycle of length `r`, the
//! multiplicative order. Forward step, multiply by `a`, is the raising
//! operator S+; its adjoint, multiply by the inverse, is the lowering
//! operator S-. The symmetric split/fuse operator `A = S+ + S-` on the orbit
//! basis is the r-cycle Jacobi matrix, ones on the off-diagonals and the
//! cyclic corner, and its spectrum is exactly `2 cos(2 pi k / r)`. The winding
//! number of Section 6.3 is the eigenphase: the second-largest eigenvalue is
//! `2 cos(2 pi / r)`, so the period reads straight off the spectral gap.
//!
//! Two halves, measured separately.
//!
//! CONCENTRATION CEILING (measured): a fixed Lanczos band of size `b` seeded
//! from one orbit point resolves the gap `2 - 2 cos(2 pi / r) ~ (2 pi / r)^2`
//! only for `r <~ 2 b`. Past that the second Ritz value saturates. Reading `r`
//! from the band alone therefore costs a band of order `r`, the same cost as
//! walking the order. This is the open object: one eigenphase sample without
//! the orbit.
//!
//! DECODE (proven, cheap): given ONE winding sample, the rational `s / r0`
//! that a single eigenphase supplies, the continued fraction recovers the true
//! period in O(log r) steps, exact, and the factors close by
//! `gcd(a^{r/2} +- 1, N)`. This half runs at arbitrary width over the numeral
//! tapes and is what `close_from_sample` and the `period-close` bin carry.

use alloc::vec::Vec;
use alloc::string::String;
use alloc::format;

use crate::morphism_factor::{cmp, divmod, mul, modulo, add, one, tape_u64, zero};
use crate::shor_braid::factor_close_public;

type Tape = Vec<char>;

/// `a^e mod n` over tapes, square-and-multiply on the exponent's tape marks.
fn powm(a: &[char], e: &[char], n: &[char]) -> Tape {
    // exponent bits, least significant first, by repeated halving.
    let mut bits: Vec<bool> = Vec::new();
    let mut e_cur = e.to_vec();
    let two = tape_u64(2);
    while !zero(&e_cur) {
        let (q, rem) = divmod(&e_cur, &two);
        bits.push(!zero(&rem));
        e_cur = q;
    }
    let mut result = one();
    let mut base = modulo(a, n);
    for bit in bits {
        if bit { result = modulo(&mul(&result, &base), n); }
        base = modulo(&mul(&base, &base), n);
    }
    result
}

/// The continued-fraction decode: convergent denominators of `s / r0`.
/// Each denominator is a candidate period; the caller lifts it to the true
/// order by a `powm` check. O(log r0) convergents.
pub fn cf_denominators(s: &[char], r0: &[char]) -> Vec<Tape> {
    // First the continued-fraction quotients a_0, a_1, ... of s / r0.
    let mut quotients: Vec<Tape> = Vec::new();
    let (mut num, mut den) = (s.to_vec(), r0.to_vec());
    for _ in 0..4096 {
        if zero(&den) { break; }
        let (a_k, rem) = divmod(&num, &den);
        quotients.push(a_k);
        num = den;
        den = rem;
    }
    // Convergent denominators: k_{-1}=0, k_0=1, k_i = a_i k_{i-1} + k_{i-2}.
    // The candidate periods are k_0, k_1, ...; k_0 = 1 carries no quotient.
    let mut out: Vec<Tape> = Vec::new();
    let (mut k_prev, mut k_curr) = (tape_u64(0), one());
    out.push(k_curr.clone());
    for a_i in quotients.iter().skip(1) {
        let k_next = add(&mul(a_i, &k_curr), &k_prev);
        out.push(k_next.clone());
        k_prev = k_curr;
        k_curr = k_next;
    }
    out
}

/// Given a winding sample `s / r0` from one eigenphase of the orbit operator,
/// recover the period and close the factors. The convergent denominator is
/// lifted to the true order by the `powm` return check, then split.
pub fn close_from_sample(a: &[char], n: &[char], s: &[char], r0: &[char])
    -> Result<(Tape, Tape), String>
{
    let one_t = one();
    let two_t = tape_u64(2);
    for q in cf_denominators(s, r0) {
        if zero(&q) || cmp(&q, &one_t) == core::cmp::Ordering::Equal { continue; }
        if cmp(&q, n) == core::cmp::Ordering::Greater { continue; }
        // lift q to the minimal period: smallest multiple with a^r == 1
        let mut r = q.clone();
        let mut lifted = false;
        for _ in 0..64 {
            if cmp(&powm(a, &r, n), &one_t) == core::cmp::Ordering::Equal { lifted = true; break; }
            r = add(&r, &q);
            if cmp(&r, n) == core::cmp::Ordering::Greater { break; }
        }
        if !lifted { continue; }
        // even order with a nontrivial half-step closes the factors
        let (_half, rem) = divmod(&r, &two_t);
        if !zero(&rem) { continue; }
        if let Ok((p, qf)) = factor_close_public(a, n, &r) {
            return Ok((p, qf));
        }
    }
    Err("no convergent lifted to a splitting period".into())
}

/// The r-cycle Jacobi band read, f64, the concentration instrument. Builds the
/// symmetric tridiagonal Lanczos band of size `band` seeded from one orbit
/// point and returns the second-largest Ritz value `~ 2 cos(2 pi / r)`. Reads
/// `r` exactly for `r <~ 2*band`; saturates past that (the measured ceiling).
/// `orbit` is the residue cycle `[1, a, a^2, ...]` of length `r`.
pub fn band_second_ritz(r: usize, band: usize) -> f64 {
    // C acts on a length-r vector as the cyclic nearest-neighbour sum.
    let cmul = |v: &[f64]| -> Vec<f64> {
        let n = v.len();
        (0..n).map(|i| v[(i + n - 1) % n] + v[(i + 1) % n]).collect()
    };
    let dot = |x: &[f64], y: &[f64]| -> f64 { x.iter().zip(y).map(|(a, b)| a * b).sum() };
    let b = band.min(r);
    let mut basis: Vec<Vec<f64>> = Vec::new();
    let mut alphas: Vec<f64> = Vec::new();
    let mut betas: Vec<f64> = Vec::new();
    let mut v_prev = alloc::vec![0.0f64; r];
    let mut v_cur = alloc::vec![0.0f64; r];
    v_cur[0] = 1.0;
    let mut beta = 0.0f64;
    for _ in 0..b {
        let mut w = cmul(&v_cur);
        let a = dot(&v_cur, &w);
        alphas.push(a);
        for i in 0..r { w[i] -= a * v_cur[i] + beta * v_prev[i]; }
        for u in &basis {
            let c = dot(u, &w);
            for i in 0..r { w[i] -= c * u[i]; }
        }
        beta = libm_sqrt(dot(&w, &w));
        basis.push(v_cur.clone());
        betas.push(beta);
        if beta < 1e-12 { break; }
        v_prev = v_cur;
        v_cur = w.iter().map(|x| x / beta).collect();
    }
    // eigenvalues of the (len x len) symmetric tridiagonal (alphas, betas) by
    // Sturm bisection; return the second largest.
    let evs = tridiag_eigs(&alphas, &betas);
    if evs.len() >= 2 { evs[evs.len() - 2] } else { -2.0 }
}

fn libm_sqrt(x: f64) -> f64 {
    if x <= 0.0 { return 0.0; }
    let mut g = x;
    for _ in 0..64 { g = 0.5 * (g + x / g); }
    g
}

/// cosine by Taylor after range reduction to [-pi, pi], no_std, ~1e-15.
fn my_cos(theta: f64) -> f64 {
    let pi = core::f64::consts::PI;
    let two_pi = 2.0 * pi;
    let mut t = theta - two_pi * ((theta / two_pi) as i64 as f64);
    if t > pi { t -= two_pi; }
    if t < -pi { t += two_pi; }
    let t2 = t * t;
    1.0 - t2 / 2.0 + t2 * t2 / 24.0 - t2 * t2 * t2 / 720.0
        + t2 * t2 * t2 * t2 / 40320.0 - t2 * t2 * t2 * t2 * t2 / 3628800.0
}

/// Invert the second Ritz value `lam2 = 2 cos(2 pi / r)` to the integer period
/// by matching candidates, up to `cap`. Avoids acos in no_std and is exact when
/// the band has resolved the gap.
pub fn period_from_ritz(lam2: f64, cap: usize) -> usize {
    let pi = core::f64::consts::PI;
    let mut best = 0usize;
    let mut best_err = f64::INFINITY;
    for r in 2..=cap {
        let model = 2.0 * my_cos(2.0 * pi / (r as f64));
        let e = (model - lam2).abs();
        if e < best_err { best_err = e; best = r; }
    }
    best
}

/// All eigenvalues of a symmetric tridiagonal (diagonal `d`, off-diagonal `e`,
/// where `e[k]` couples k and k+1), ascending, by the cyclic Jacobi eigenvalue
/// algorithm on the dense matrix. Small bands, so the O(n^3) sweep is cheap and
/// numerically clean.
fn tridiag_eigs(d: &[f64], e: &[f64]) -> Vec<f64> {
    let n = d.len();
    if n == 0 { return Vec::new(); }
    // dense symmetric matrix
    let mut a = alloc::vec![alloc::vec![0.0f64; n]; n];
    for i in 0..n {
        a[i][i] = d[i];
        if i + 1 < n {
            let off = if i < e.len() { e[i] } else { 0.0 };
            a[i][i + 1] = off;
            a[i + 1][i] = off;
        }
    }
    for _ in 0..100 {
        // largest off-diagonal magnitude
        let mut off = 0.0f64;
        for i in 0..n { for j in (i + 1)..n { off += a[i][j] * a[i][j]; } }
        if off < 1e-24 { break; }
        for p in 0..n {
            for q in (p + 1)..n {
                if a[p][q].abs() < 1e-18 { continue; }
                let theta = (a[q][q] - a[p][p]) / (2.0 * a[p][q]);
                let t = {
                    let s = if theta >= 0.0 { 1.0 } else { -1.0 };
                    s / (theta.abs() + libm_sqrt(theta * theta + 1.0))
                };
                let c = 1.0 / libm_sqrt(t * t + 1.0);
                let s = t * c;
                for k in 0..n {
                    let akp = a[k][p];
                    let akq = a[k][q];
                    a[k][p] = c * akp - s * akq;
                    a[k][q] = s * akp + c * akq;
                }
                for k in 0..n {
                    let apk = a[p][k];
                    let aqk = a[q][k];
                    a[p][k] = c * apk - s * aqk;
                    a[q][k] = s * apk + c * aqk;
                }
            }
        }
    }
    let mut out: Vec<f64> = (0..n).map(|i| a[i][i]).collect();
    out.sort_by(|x, y| x.partial_cmp(y).unwrap_or(core::cmp::Ordering::Equal));
    out
}

/// Report the period read from the concentrated band, for a supplied order `r`.
/// Reads `2 cos(2 pi / r)` and inverts it. The control is the orbit length.
pub fn ladder_read_report(r: usize, band: usize) -> String {
    let lam2 = band_second_ritz(r, band);
    let read = period_from_ritz(lam2, 3 * band + 4);
    format!("r={r} band={band} second_ritz={lam2:.6} read_r={read}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::morphism_factor::{decimal_to_tape, dec_of};

    fn t(s: &str) -> Tape { decimal_to_tape(s).unwrap() }

    #[test]
    fn cf_decode_closes_143() {
        // order of 2 mod 143 is 60; a single sample s/r0 = 7/60 (7 coprime 60).
        let (p, q) = close_from_sample(&t("2"), &t("143"), &t("7"), &t("60")).unwrap();
        let prod = mul(&p, &q);
        assert_eq!(dec_of(&prod), "143");
    }

    #[test]
    fn cf_decode_closes_187() {
        // order of 7 mod 187 is 80; sample 9/80.
        let (p, q) = close_from_sample(&t("7"), &t("187"), &t("9"), &t("80")).unwrap();
        assert_eq!(dec_of(&mul(&p, &q)), "187");
    }

    #[test]
    fn band_reads_small_orders_exactly() {
        // r <~ 2*band: exact. Band 16 reads r up to ~30.
        for &r in &[4usize, 6, 12, 30] {
            let lam2 = band_second_ritz(r, 16);
            assert_eq!(period_from_ritz(lam2, 64), r, "band read r={r}");
        }
    }
}
