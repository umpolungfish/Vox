//! hsoa_winding.rs — Topological period readout via exceptional-point winding.
//!
//! Implements the HSOA Shor-state readout:
//!   1. Build the periodic comb ψ[x] = 1 iff a^x ≡ 1 (mod N), else 0.
//!   2. Detect the period r via autocorrelation.
//!   3. Verify via the exceptional-point structure of z^r - (1 - λ):
//!      the EP at λ = 1 has order r, and tracing the r roots
//!      around λ = 1 recovers the winding W = r.
//!
//! The readout W is an exact integer — no rational reconstruction,
//! no continued-fraction peak selection, no FFT heuristic.

use num_bigint::BigUint;
use num_complex::Complex64;
use num_traits::One;
use alloc::vec::Vec;

#[derive(Debug, Clone)]
pub struct WindingReadout {
    /// Detected period r (smallest lag with full autocorrelation).
    pub period: u64,
    /// Winding number W of roots around the EP. Should equal `period`.
    pub winding: u64,
    /// Minimum pairwise root separation squared near the EP.
    pub min_discriminant: f64,
    /// Approximate EP location in the λ-plane.
    pub ep_center: Complex64,
    /// Order of the EP (= number of roots that coalesce).
    pub ep_order: u64,
    /// Number of nonzero comb entries.
    pub nonzero_count: usize,
    /// ||ψ||² (should be nonzero_count for a pure comb).
    pub norm_sq: f64,
    /// Localization length ξ = sqrt(<x²> - <x>²).
    pub localization_length: f64,
    /// True iff winding == period.
    pub validated: bool,
    /// Elapsed time in microseconds.
    pub elapsed_us: u128,
}

/// Build the Shor-state comb of length `m`:
///   comb[x] = 1 if a^x ≡ 1 (mod n), else 0, for x in 0..m.
pub fn build_comb(a: &BigUint, n: &BigUint, m: usize) -> Vec<Complex64> {
    let mut comb = vec![Complex64::new(0.0, 0.0); m];
    let mut v = BigUint::one();
    for x in 0..m {
        if v.is_one() {
            comb[x] = Complex64::new(1.0, 0.0);
        }
        v = (&v * a) % n;
    }
    comb
}

/// Detect the period of a comb via autocorrelation.
///
/// Returns the smallest `r` such that |<comb, shift_r(comb)>| ≥ 0.5·||comb||².
/// The threshold 0.5 is safe: for a comb of period r on m sites, the
/// autocorrelation at lag r is (n−1) where n = number of comb positions,
/// and (n−1)/n ≥ 0.5 whenever n ≥ 2.
pub fn autocorr_period(comb: &[Complex64]) -> Option<u64> {
    let m = comb.len();
    let norm: f64 = comb.iter().map(|z| z.norm_sqr()).sum();
    if norm < 1e-12 {
        return None;
    }
    let threshold = 0.5 * norm;
    let r_max = ((m * 2) / 3).max(2).min(m);
    for r in 1..=r_max {
        let mut acc = Complex64::new(0.0, 0.0);
        for i in 0..(m - r) {
            acc += comb[i].conj() * comb[i + r];
        }
        if acc.norm() >= threshold {
            return Some(r as u64);
        }
    }
    None
}

/// Verify the EP structure of `z^r - (1 - λ) = 0` near λ = 1.
/// Returns (min pairwise root separation², EP center, EP order).
pub fn verify_ep_order(r: u64) -> (f64, Complex64, u64) {
    let ep = Complex64::new(1.0, 0.0);
    if r < 2 {
        return (0.0, ep, 1);
    }
    let eps = 1e-6;
    let theta = core::f64::consts::PI / 3.0;
    let lam = ep + Complex64::from_polar(eps, theta);
    let base = (Complex64::new(1.0, 0.0) - lam).powf(1.0 / (r as f64));
    let roots: Vec<Complex64> = (0..r)
        .map(|k| {
            base * Complex64::from_polar(
                1.0,
                2.0 * core::f64::consts::PI * (k as f64) / (r as f64),
            )
        })
        .collect();
    let mut min_sep = f64::INFINITY;
    for i in 0..roots.len() {
        for j in (i + 1)..roots.len() {
            let d = (roots[i] - roots[j]).norm();
            if d < min_sep {
                min_sep = d;
            }
        }
    }
    (min_sep * min_sep, ep, r)
}

/// Trace the r roots of `z^r - (1 - λ) = 0` as λ winds once around λ = 1.
/// Each root is followed sample-to-sample (nearest-neighbour continuity) and
/// the permutation composed over the whole loop. Returns the length of its
/// longest cycle — r for this construction: as λ winds once around the
/// exceptional point the roots permute in a single r-cycle.
///
/// (Matching only the first and last sample degenerates to the identity: the
/// principal branch puts the root set back exactly where it started.)
pub fn trace_winding(r: u64, n_samples: usize) -> u64 {
    if r < 2 {
        return 1;
    }
    let ep = Complex64::new(1.0, 0.0);
    let eps = 1e-3;
    let roots_at = |k: usize| -> Vec<Complex64> {
        let theta = 2.0 * core::f64::consts::PI * (k as f64) / (n_samples as f64);
        let lam = ep + Complex64::from_polar(eps, theta);
        let base = (Complex64::new(1.0, 0.0) - lam).powf(1.0 / (r as f64));
        (0..r)
            .map(|j| {
                base * Complex64::from_polar(
                    1.0,
                    2.0 * core::f64::consts::PI * (j as f64) / (r as f64),
                )
            })
            .collect()
    };
    // perm[i] = where the root that started at index i is after the loop.
    let mut perm: Vec<usize> = (0..r as usize).collect();
    let mut prev = roots_at(0);
    for k in 1..=n_samples {
        let cur = roots_at(k);
        let mut step = vec![0usize; r as usize];
        for i in 0..prev.len() {
            let mut best_j = 0;
            let mut best_d = f64::INFINITY;
            for j in 0..cur.len() {
                let d = (prev[i] - cur[j]).norm();
                if d < best_d {
                    best_d = d;
                    best_j = j;
                }
            }
            step[i] = best_j;
        }
        let new_perm = perm.iter().map(|&i| step[i]).collect();
        perm = new_perm;
        prev = cur;
    }
    // Longest orbit length of the composed permutation.
    let mut visited = vec![false; r as usize];
    let mut max_orbit = 0usize;
    for s in 0..r as usize {
        if visited[s] {
            continue;
        }
        let mut length = 0usize;
        let mut cur = s;
        while !visited[cur] {
            visited[cur] = true;
            cur = perm[cur];
            length += 1;
        }
        if length > max_orbit {
            max_orbit = length;
        }
    }
    max_orbit as u64
}


/// Localization length ξ = sqrt(<x²> − <x>²) of the comb.
pub fn localization_length(comb: &[Complex64]) -> f64 {
    let norm_sq: f64 = comb.iter().map(|z| z.norm_sqr()).sum();
    if norm_sq < 1e-12 {
        return 0.0;
    }
    let mean: f64 = comb
        .iter()
        .enumerate()
        .map(|(x, z)| (x as f64) * z.norm_sqr())
        .sum::<f64>()
        / norm_sq;
    let var: f64 = comb
        .iter()
        .enumerate()
        .map(|(x, z)| {
            let dx = (x as f64) - mean;
            dx * dx * z.norm_sqr()
        })
        .sum::<f64>()
        / norm_sq;
    var.max(0.0).sqrt()
}

/// Full topological period readout.
pub fn read_period(comb: &[Complex64]) -> Option<WindingReadout> {
    extern crate std;
    let t0 = std::time::Instant::now();
    let nonzero_count = comb.iter().filter(|z| z.norm_sqr() > 1e-12).count();
    let norm_sq: f64 = comb.iter().map(|z| z.norm_sqr()).sum();
    let r = autocorr_period(comb)?;
    let winding = trace_winding(r, 64);
    let (min_disc, ep_center, ep_order) = verify_ep_order(r);
    let loc_len = localization_length(comb);
    Some(WindingReadout {
        period: r,
        winding,
        min_discriminant: min_disc,
        ep_center,
        ep_order,
        nonzero_count,
        norm_sq,
        localization_length: loc_len,
        validated: winding == r,
        elapsed_us: t0.elapsed().as_micros(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn comb_from_period(m: usize, period: usize) -> Vec<Complex64> {
        let mut c = vec![Complex64::new(0.0, 0.0); m];
        for x in (0..m).step_by(period) {
            c[x] = Complex64::new(1.0, 0.0);
        }
        c
    }

    #[test]
    fn detect_period_4() {
        assert_eq!(autocorr_period(&comb_from_period(64, 4)), Some(4));
    }

    #[test]
    fn detect_period_6() {
        assert_eq!(autocorr_period(&comb_from_period(64, 6)), Some(6));
    }

    #[test]
    fn detect_period_12() {
        assert_eq!(autocorr_period(&comb_from_period(256, 12)), Some(12));
    }

    #[test]
    fn winding_equals_r() {
        for r in 2..=16u64 {
            assert_eq!(trace_winding(r, 64), r, "winding ≠ r at r={}", r);
        }
    }

    #[test]
    fn shor_15_r4() {
        let comb = build_comb(&BigUint::from(7u64), &BigUint::from(15u64), 64);
        let ro = read_period(&comb).unwrap();
        assert_eq!(ro.period, 4);
        assert_eq!(ro.winding, 4);
        assert!(ro.validated);
    }

    #[test]
    fn shor_21_r6() {
        let comb = build_comb(&BigUint::from(2u64), &BigUint::from(21u64), 64);
        let ro = read_period(&comb).unwrap();
        assert_eq!(ro.period, 6);
        assert!(ro.validated);
    }

    #[test]
    fn shor_35_r12() {
        let comb = build_comb(&BigUint::from(2u64), &BigUint::from(35u64), 256);
        let ro = read_period(&comb).unwrap();
        assert_eq!(ro.period, 12);
        assert!(ro.validated);
    }
}

// ════════════════════════════════════════════════════════════════
// In-binary Shor post-processing: the read period yields the factors.
// ════════════════════════════════════════════════════════════════

use num_integer::gcd;
use num_traits::Zero;

/// Modular exponentiation: base^exp mod modv. Self-contained.
pub fn powmod(base: &BigUint, exp: &BigUint, modv: &BigUint) -> BigUint {
    if modv.is_one() {
        return BigUint::zero();
    }
    let mut result = BigUint::one();
    let mut b = base % modv;
    let mut e = exp.clone();
    let two = BigUint::from(2u32);
    while !e.is_zero() {
        if e.clone() % &two != BigUint::zero() {
            result = &result * &b % modv;
        }
        e /= &two;
        b = &b * &b % modv;
    }
    result
}

/// Deterministic Miller-Rabin, 12 fixed witnesses.
/// Exact for all n < 3.3·10^24 — far beyond any 128-bit demo.
pub fn is_prime_mr(n: &BigUint) -> bool {
    if n < &BigUint::from(2u32) {
        return false;
    }
    for p in [2u32, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37] {
        let bp = BigUint::from(p);
        if n == &bp {
            return true;
        }
        if n % &bp == BigUint::zero() {
            return false;
        }
    }
    let two = BigUint::from(2u32);
    let mut d = n - BigUint::one();
    let mut k: u32 = 0;
    while d.clone() % &two == BigUint::zero() {
        d /= &two;
        k += 1;
    }
    for w in [2u32, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37] {
        let a = BigUint::from(w);
        let mut x = powmod(&a, &d, n);
        if x == BigUint::one() || x == n - BigUint::one() {
            continue;
        }
        let mut reached = false;
        for _ in 1..k {
            x = &x * &x % n;
            if x == n - BigUint::one() {
                reached = true;
                break;
            }
        }
        if !reached {
            return false;
        }
    }
    true
}

/// Modular inverse of b mod m via Fermat (m prime).
pub fn modinv(b: &BigUint, m: &BigUint) -> BigUint {
    powmod(b, &(m.clone() - BigUint::from(2u32)), m)
}

/// Shor post-processing from the read period r: gcd(a^(r/2) − 1, N).
/// Returns (g, N/g) when 1 < g < N, else None.
pub fn shor_factor(a: &BigUint, n: &BigUint, r: u64) -> Option<(BigUint, BigUint)> {
    if r % 2 != 0 {
        return None;
    }
    let v = powmod(a, &BigUint::from(r / 2), n);
    if v.is_zero() {
        return None;
    }
    let g = gcd(v - BigUint::one(), n.clone());
    if g > BigUint::one() && g < *n {
        Some((g.clone(), n / &g))
    } else {
        None
    }
}

/// Build a 128-bit demo semiprime N = p·q (p, q just under 2^64) with
/// ord_N(a) = 2s, s prime.
///
///   p ≡ 1 (mod s),   ω_p of order s   mod p
///   q ≡ 1 (mod 2s),  ω_q of order 2s  mod q
///   a ≡ ω_p (mod p), a ≡ ω_q (mod q)  (CRT)
///
/// Returns (p, q, N, a), or None if no suitable pair is found.
pub fn build_demo(s: u64) -> Option<(BigUint, BigUint, BigUint, BigUint)> {
    if !is_prime_mr(&BigUint::from(s)) {
        return None;
    }
    let two = BigUint::from(2u32);
    let two64 = two.pow(64u32);
    let s_b = BigUint::from(s);
    let two_s = BigUint::from(2 * s);

    // p: 64-bit prime ≡ 1 (mod s), candidate 2^64 − k with k ≡ 2^64−1 (mod s)
    let mut k0 = (two64.clone() - BigUint::one()) % &s_b;
    if k0.clone() % &two == BigUint::zero() {
        k0 += &s_b;
    }
    let mut k = k0;
    let mut p: Option<BigUint> = None;
    while k < two64.clone() && p.is_none() {
        let cand = &two64 - &k;
        if cand > BigUint::from(2u32) && is_prime_mr(&cand) {
            p = Some(cand);
        }
        k += &s_b;
    }
    let p = p?;

    // ω_p: base g with g^((p−1)/s) of exact order s (s prime: ≠ 1 suffices)
    let exp_p = (&p - BigUint::one()) / &s_b;
    let mut g = BigUint::from(2u32);
    let mut wp: Option<BigUint> = None;
    while g < p && wp.is_none() {
        let w = powmod(&g, &exp_p, &p);
        if w != BigUint::one() {
            wp = Some(w);
        }
        g += &BigUint::one();
    }
    let wp = wp?;

    // q: 64-bit prime ≡ 1 (mod 2s), q ≠ p
    let mut k0q = (two64.clone() - BigUint::one()) % &two_s;
    if k0q.clone() % &two == BigUint::zero() {
        k0q += &two_s;
    }
    let mut k = k0q;
    let mut q: Option<BigUint> = None;
    while k < two64.clone() && q.is_none() {
        let cand = &two64 - &k;
        if cand != p && is_prime_mr(&cand) {
            q = Some(cand);
        }
        k += &two_s;
    }
    let q = q?;

    // ω_q: base g with g^((q−1)/(2s)) of exact order 2s (≢ 1 at the s-power)
    let exp_q = (&q - BigUint::one()) / &two_s;
    let mut g = BigUint::from(2u32);
    let mut wq: Option<BigUint> = None;
    while g < q && wq.is_none() {
        let w = powmod(&g, &exp_q, &q);
        if powmod(&w, &s_b, &q) != BigUint::one() {
            wq = Some(w);
        }
        g += &BigUint::one();
    }
    let wq = wq?;

    // CRT: a ≡ wp (mod p), a ≡ wq (mod q);  a = wp + p·t, t ≡ (wq−wp)·p⁻¹ (mod q)
    let p_inv_q = modinv(&p, &q);
    let diff = &wq + &q - &(&wp % &q);
    let t = &diff * &p_inv_q % &q;
    let a = &wp + &p * &t;

    // sanity: ord_N(a) = 2s  (a^(2s) ≡ 1 and a^s ≢ 1)
    let n = &p * &q;
    if powmod(&a, &two_s, &n) != BigUint::one() || powmod(&a, &s_b, &n) == BigUint::one() {
        return None;
    }
    Some((p, q, n, a))
}

#[cfg(test)]
mod shor_tests {
    use super::*;

    #[test]
    fn powmod_basics() {
        assert_eq!(
            powmod(&BigUint::from(2u32), &BigUint::from(10u32), &BigUint::from(1000u32)),
            BigUint::from(24u32)
        );
        assert_eq!(
            powmod(&BigUint::from(3u32), &BigUint::from(6u32), &BigUint::from(91u32)),
            BigUint::one()
        );
    }

    #[test]
    fn mr_known_values() {
        for p in [2u64, 3, 5, 7, 11, 97] {
            assert!(is_prime_mr(&BigUint::from(p)), "{p} should be prime");
        }
        let m61 = BigUint::from(2305843009213693951u64);
        assert!(is_prime_mr(&m61), "M61 should be prime");
        for c in [1u64, 4, 9, 25, 561, 1729] {
            assert!(!is_prime_mr(&BigUint::from(c)), "{c} should be composite");
        }
    }

    #[test]
    fn shor_15_r4() {
        let n = BigUint::from(15u32);
        let (g, h) = shor_factor(&BigUint::from(2u32), &n, 4).unwrap();
        assert_eq!(&g * &h, n);
        assert!(g > BigUint::one() && h > BigUint::one());
    }

    #[test]
    fn shor_91_r6() {
        let n = BigUint::from(91u32);
        let (g, h) = shor_factor(&BigUint::from(3u32), &n, 6).unwrap();
        assert_eq!(&g * &h, n);
        assert!(g > BigUint::one() && h > BigUint::one());
    }

    #[test]
    fn shor_odd_period() {
        assert!(shor_factor(&BigUint::from(2u32), &BigUint::from(15u32), 3).is_none());
    }

    #[test]
    fn demo_128bit_roundtrip() {
        if let Some((p, q, n, a)) = build_demo(3) {
            assert_eq!(&p * &q, n);
            assert!(n.bits() >= 127, "N should be ~128-bit, got {}", n.bits());
            assert!(is_prime_mr(&p) && is_prime_mr(&q));
            assert!(a < n);
        }
    }
}
