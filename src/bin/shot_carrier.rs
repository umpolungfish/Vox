//! shot_carrier.rs — BigUint N, arbitrary q, bitfolded Shor, no silent fallback.
#![deny(warnings)]
use std::env;
use std::process;
use num_bigint::BigUint;
use num_traits::{One, Zero};

fn mod_pow(mut a: BigUint, mut e: BigUint, n: &BigUint) -> BigUint {
    a %= n;
    let mut r = BigUint::one();
    while !e.is_zero() {
        if e.bit(0) { r = (&r * &a) % n; }
        a = (&a * &a) % n;
        e >>= 1u64;
    }
    r
}

fn gcd(mut a: BigUint, mut b: BigUint) -> BigUint {
    while !b.is_zero() { let t = &a % &b; a = b; b = t; }
    a
}

// ===== Bitfold #1: precomputed a^(2^k) =====
struct FoldedPow { powers: Vec<BigUint> }
impl FoldedPow {
    fn new(a: &BigUint, n: &BigUint, q: usize) -> Self {
        let mut powers = Vec::with_capacity(q);
        let mut cur = a % n;
        for _ in 0..q { powers.push(cur.clone()); cur = (&cur * &cur) % n; }
        Self { powers }
    }
    fn eval(&self, mut x: u64, n: &BigUint) -> BigUint {
        let mut r = BigUint::one();
        let mut k = 0usize;
        while x > 0 {
            if x & 1 == 1 { r = (&r * &self.powers[k]) % n; }
            x >>= 1; k += 1;
        }
        r
    }
}

// ===== Bitfold #2: sparse register =====
struct SparseState { positions: Vec<u64>, m: u64 }
impl SparseState {
    fn build(a: &BigUint, n: &BigUint, q: usize) -> Self {
        let m = 1u64 << q;
        let fp = FoldedPow::new(a, n, q);
        let mut positions = Vec::new();
        for x in 0..m {
            if fp.eval(x, n).is_one() { positions.push(x); }
        }
        Self { positions, m }
    }
    fn cnt(&self) -> usize { self.positions.len() }
    fn fft_prob(&self, k: u64) -> f64 {
        let cnt = self.cnt() as f64;
        let (mut re, mut im) = (0.0f64, 0.0f64);
        for &p in &self.positions {
            let ang = -2.0 * std::f64::consts::PI * (k as f64) * (p as f64) / (self.m as f64);
            re += ang.cos(); im += ang.sin();
        }
        (re * re + im * im) / (cnt * cnt)
    }
}

// ===== Direct order: smallest nonzero position =====
fn order_direct(s: &SparseState, a: &BigUint, n: &BigUint) -> Option<u64> {
    if s.cnt() >= 2 {
        let r = s.positions[1];
        if mod_pow(a.clone(), BigUint::from(r), n).is_one() { return Some(r); }
    }
    None
}

// ===== FFT peak → continued fraction =====
fn period_from_peak(k: u64, m: u64, a: &BigUint, n: &BigUint) -> Option<u64> {
    let (mut pp, mut pc) = (0i128, 1i128);
    let (mut qp, mut qc) = (1i128, 0i128);
    let (mut kk, mut mm) = (k as i128, m as i128);
    while mm != 0 {
        let aa = kk / mm;
        let pn = aa * pc + pp;
        let qn = aa * qc + qp;
        if qn > 0 {
            let qb = BigUint::from(qn as u64);
            if &qb < n && mod_pow(a.clone(), qb.clone(), n).is_one() {
                return Some(qn as u64);
            }
        }
        pp = pc; pc = pn;
        qp = qc; qc = qn;
        let rem = kk % mm; kk = mm; mm = rem;
    }
    None
}

fn factor_from_order(a: &BigUint, n: &BigUint, r: u64) -> Option<(BigUint, BigUint)> {
    if r == 0 || r % 2 != 0 { return None; }
    let half = mod_pow(a.clone(), BigUint::from(r / 2), n);
    let one = BigUint::one();
    let n_minus_1 = n - &one;
    if half == n_minus_1 { return None; }
    let g1 = gcd((&half + &n_minus_1) % n, n.clone());
    if g1 > one && g1 < *n { return Some((g1.clone(), n.clone() / &g1)); }
    let g2 = gcd((&half + &one) % n, n.clone());
    if g2 > one && g2 < *n { return Some((g2.clone(), n.clone() / &g2)); }
    None
}

fn small_factor(n: &BigUint, bound: u64) -> Option<BigUint> {
    if !n.bit(0) { return Some(BigUint::from(2u64)); }
    let mut p = 3u64;
    while p <= bound {
        let pb = BigUint::from(p);
        if &pb * &pb > *n { break; }
        if (n % &pb).is_zero() { return Some(pb); }
        p += 2;
    }
    None
}

fn perfect_power(n: &BigUint) -> Option<(BigUint, u32)> {
    for k in 2u32..=64 {
        let bits = n.bits();
        let rb = (bits + k as u64 - 1) / k as u64;
        if rb == 0 { continue; }
        let mut m = BigUint::one() << (rb as usize);
        for _ in 0..200 {
            let mk1 = m.pow(k - 1);
            if mk1.is_zero() { break; }
            let num = &m * BigUint::from(k as u64 - 1) + (n / &mk1);
            let nm = num / BigUint::from(k as u64);
            if nm == m || nm.is_zero() { break; }
            m = nm;
        }
        for c in [m.clone(), m + BigUint::one()] {
            if c.pow(k) == *n { return Some((c, k)); }
        }
    }
    None
}

// ===== Bitfold #3: try a^k for small k =====
fn try_base_folded(a: &BigUint, n: &BigUint, q: usize, folds: &[u64])
    -> Option<(BigUint, u64, u64)>
{
    let g = gcd(a.clone(), n.clone());
    if g > BigUint::one() && g < *n {
        return Some((g.clone(), 1, 0));
    }
    for &k in folds {
        let a_fold = mod_pow(a.clone(), BigUint::from(k), n);
        if a_fold.is_one() { continue; }
        let state = SparseState::build(&a_fold, n, q);
        if let Some(r) = order_direct(&state, &a_fold, n) {
            if let Some((p, _f)) = factor_from_order(&a_fold, n, r) {
                return Some((p, k, r));
            }
        }
        let sample = state.m.min(1024);
        let mut peaks: Vec<(u64, f64)> = (1..sample).map(|kk| (kk, state.fft_prob(kk))).collect();
        peaks.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        for &(kk, _) in peaks.iter().take(32) {
            if let Some(r) = period_from_peak(kk, state.m, &a_fold, n) {
                if let Some((p, _f)) = factor_from_order(&a_fold, n, r) {
                    return Some((p, k, r));
                }
            }
        }
    }
    None
}

fn main() {
    #[cfg(feature = "hosted")]
    ::vox::ordering::cli::intercept();
    let raw: Vec<String> = env::args().skip(1).collect();
    if raw.len() < 2 {
        eprintln!("usage: shot_carrier <N> <q>   (q < 63)");
        process::exit(2);
    }
    let n = match BigUint::parse_bytes(raw[0].as_bytes(), 10) {
        Some(v) if !v.is_zero() && !v.is_one() => v,
        _ => { eprintln!("bad N: {:?}", raw[0]); process::exit(2); }
    };
    let q: usize = match raw[1].parse() {
        Ok(v) if v < 63 => v,
        _ => { eprintln!("bad q (0 < q < 63): {:?}", raw[1]); process::exit(2); }
    };
    let t0 = std::time::Instant::now();
    let m = 1u64 << q;

    println!("N = {}", n);
    println!("q = {}, m = {}, carrier = {} B", q, m, (m as u128) * 16);

    print!("\n[1] trial division < 10^7 ... ");
    if let Some(p) = small_factor(&n, 10_000_000) {
        let qq = &n / &p;
        println!("FOUND  N = {} × {}", p, qq);
        println!("Time: {:?}", t0.elapsed());
        return;
    }
    println!("none");

    print!("[2] perfect power ... ");
    if let Some((b, e)) = perfect_power(&n) {
        println!("FOUND  N = {} ^ {}", b, e);
        println!("Time: {:?}", t0.elapsed());
        return;
    }
    println!("none");

    println!("[3] bitfolded Shor  (q={}, m={})", q, m);
    let bases: [u64; 8] = [2, 3, 5, 7, 11, 13, 17, 19];
    let folds: [u64; 25] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 14, 15, 16, 18, 20, 21, 22, 24, 27, 28, 30, 32, 36];
    for &a in &bases {
        let ab = BigUint::from(a);
        print!("  a={:>2}: ", a);
        std::io::Write::flush(&mut std::io::stdout()).ok();
        match try_base_folded(&ab, &n, q, &folds) {
            Some((p, k, r)) => {
                let qq = &n / &p;
                println!("FOLD k={} r={} => p={}", k, r, p);
                println!("\nFOUND: N = {} × {}", p, qq);
                println!("Time: {:?}", t0.elapsed());
                return;
            }
            None => println!("—"),
        }
    }

    println!("\n[FAIL] no factor with q={}. N may be a large semiprime; try larger q.", q);
    println!("Time: {:?}", t0.elapsed());
}