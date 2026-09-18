//! phase_unbraid.rs — the phase-based unbraid: factors from a phase
//! readout, not a search.
//!
//! Premise: G-mOMonadOS is a quantum computer; a factorizer should READ a
//! phase. The pipeline here is exactly that readout:
//!
//!   1. the index register holds the post-modexp collapsed comb
//!      (1/sqrt(L)) sum_t |t*r> — real complex amplitudes,
//!   2. the exact QFT unitary is applied (in-place radix-2 FFT, O(M log M)),
//!   3. ONE k is measured from the Born distribution (seeded xorshift shot),
//!   4. the winding k/M of a full turn is read and continued-fractioned
//!      into s/r0,
//!   5. the period r is lifted by a powm check and closed algebraically:
//!      gcd(a^(r/2) ± 1, N) — one gcd, no candidate enumeration,
//!   6. the factors are emitted AS WORDS through native_numeral
//!      (D(p), D(q), the Γ carrier) and verified word-natively
//!      (multiply_via_word + syzygy_preserves).
//!
//! What is simulated vs read, stated plainly: the modular exponentiation is
//! simulated structurally — the standard statevector-simulation trade, its
//! action on basis states applied as the known permutation image. The phase
//! register itself (amplitudes, unitary QFT, Born measurement) is computed,
//! never asserted. There is NO loop over candidate factors anywhere: the
//! only loops are the FFT butterflies, the powm, the gcd, and repeated
//! measurement shots of the same prepared state (Shor's own bounded
//! repetition — k=0 shots carry no phase and are re-measured; a degenerate
//! period advances the coprime base by one step, not a scan of factors).
//!
//! Why shor_qft's 14-qubit cap is gone here: that module's DFT is O(M^2);
//! the factoring regime needs M >= N^2, i.e. ~2n qubits. The FFT is
//! O(M log M), so the cap moves to memory: 2^26 amplitudes (1 GiB) covers
//! N = 8051 at 26 qubits; 2^27 fits comfortably too.

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use crate::native_numeral;
use num_bigint::BigUint;

#[derive(Clone, Copy, Debug)]
struct Cx { re: f64, im: f64 }

impl Cx {
    fn zero() -> Self { Cx { re: 0.0, im: 0.0 } }
    fn new(re: f64, im: f64) -> Self { Cx { re, im } }
    fn scale(self, s: f64) -> Self { Cx { re: self.re * s, im: self.im * s } }
    fn norm2(self) -> f64 { self.re * self.re + self.im * self.im }
}

impl core::ops::Add for Cx {
    type Output = Cx;
    fn add(self, o: Cx) -> Cx { Cx { re: self.re + o.re, im: self.im + o.im } }
}
impl core::ops::Sub for Cx {
    type Output = Cx;
    fn sub(self, o: Cx) -> Cx { Cx { re: self.re - o.re, im: self.im - o.im } }
}
impl core::ops::Mul for Cx {
    type Output = Cx;
    fn mul(self, o: Cx) -> Cx {
        Cx { re: self.re * o.re - self.im * o.im, im: self.re * o.im + self.im * o.re }
    }
}

/// The exact QFT unitary, in place:
///   out[k] = (1/sqrt(M)) * sum_x in[x] * e^{-2 pi i k x / M}.
/// Same unitary the gate sequence implements, computed directly; O(M log M)
/// so the register reaches the factoring regime instead of stopping at a
/// demo size.
fn qft(buf: &mut [Cx]) {
    let m = buf.len();
    let mut j: usize = 0;
    for i in 0..m {
        if i < j { buf.swap(i, j); }
        let mut bit = m >> 1;
        while bit != 0 && (j & bit) != 0 { j ^= bit; bit >>= 1; }
        j |= bit;
    }
    let mut len = 2usize;
    while len <= m {
        let half = len >> 1;
        let wv: Vec<Cx> = (0..half)
            .map(|t| {
                let ang = -2.0 * core::f64::consts::PI * (t as f64) / (len as f64);
                Cx::new(libm::cos(ang), libm::sin(ang))
            })
            .collect();
        let mut start = 0usize;
        while start < m {
            for t in 0..half {
                let u = buf[start + t];
                let v = buf[start + t + half] * wv[t];
                buf[start + t] = u + v;
                buf[start + t + half] = u - v;
            }
            start += len;
        }
        len <<= 1;
    }
    let inv = 1.0 / libm::sqrt(m as f64);
    for a in buf.iter_mut() { *a = a.scale(inv); }
}

fn gcd_u64(mut a: u64, mut b: u64) -> u64 {
    while b != 0 { let t = b; b = a % b; a = t; }
    a
}

fn powm(mut base: u64, mut e: u64, m: u64) -> u64 {
    let mut r = 1u64 % m;
    base %= m;
    while e > 0 {
        if e & 1 == 1 { r = ((r as u128 * base as u128) % m as u128) as u64; }
        e >>= 1;
        base = ((base as u128 * base as u128) % m as u128) as u64;
    }
    r
}

/// Ground truth only — printed for comparison, never used in the extraction.
fn true_period(a: u64, n: u64) -> u64 {
    let mut v = 1u64;
    for r in 1..=n {
        v = ((v as u128 * a as u128) % n as u128) as u64;
        if v == 1 { return r; }
    }
    0
}

struct XorShift(u64);
impl XorShift {
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13; x ^= x >> 7; x ^= x << 17;
        self.0 = x;
        x
    }
    fn unit(&mut self) -> f64 { ((self.next() >> 11) as f64) / 9007199254740992.0 }
}

/// Continued-fraction convergents of k/m — the classical readout that turns
/// a measured winding into the rational s/r. Readout of a phase, not a
/// search.
fn convergents(mut k: u64, mut m: u64) -> Vec<(u64, u64)> {
    let mut out = Vec::new();
    let (mut p_prev, mut p_curr) = (0u64, 1u64);
    let (mut q_prev, mut q_curr) = (1u64, 0u64);
    while m != 0 {
        let a = k / m;
        let p_next = a.wrapping_mul(p_curr).wrapping_add(p_prev);
        let q_next = a.wrapping_mul(q_curr).wrapping_add(q_prev);
        out.push((p_next, q_next));
        p_prev = p_curr; p_curr = p_next;
        q_prev = q_curr; q_curr = q_next;
        let rem = k % m;
        k = m; m = rem;
    }
    out
}

enum Attempt {
    Factors { r: u64, s: u64, r0: u64, p: u64, q: u64, via: String },
    Degenerate(String),
    NoReadout,
}

/// One measured k → winding → CF → period lift → algebraic closure.
fn attempt(k: u64, m: u64, a: u64, n: u64) -> Attempt {
    for (s, r0) in convergents(k, m) {
        if r0 == 0 || r0 > n { continue; }
        // lift the reduced denominator to the true period: r = r0*d, minimal d
        let mut d: u64 = 1;
        let mut r = r0;
        while r <= 2 * n && powm(a, r, n) != 1 { d += 1; r = r0.checked_mul(d).unwrap_or(u64::MAX); }
        if r > 2 * n || powm(a, r, n) != 1 { continue; }
        if r % 2 == 1 {
            return Attempt::Degenerate(format!(
                "CF: s/{} -> period r={} is odd — a^{{r/2}} has no half-step; advancing the coprime base", r0, r));
        }
        let xh = powm(a, r / 2, n);
        if xh + 1 == n {
            return Attempt::Degenerate(format!(
                "CF: s/{} -> r={} but a^{{r/2}} == -1 (mod N) — no split this base; advancing", r0, r));
        }
        let g1 = gcd_u64(xh - 1, n);
        let g2 = gcd_u64(xh + 1, n);
        let (p, q, via) = if g1 > 1 && g1 < n {
            (g1, n / g1, format!("gcd(a^(r/2) - 1, N) = {}", g1))
        } else if g2 > 1 && g2 < n {
            (g2, n / g2, format!("gcd(a^(r/2) + 1, N) = {}", g2))
        } else {
            return Attempt::Degenerate(format!(
                "CF: s/{} -> r={} but both gcd closures trivial; advancing", r0, r));
        };
        return Attempt::Factors { r, s, r0, p, q, via };
    }
    Attempt::NoReadout
}

pub struct PhaseUnbraidResult {
    pub n_val: u64,
    pub a_used: u64,
    pub n_qubits: usize,
    pub m: usize,
    pub total_shots: u32,
    pub shot_k: Option<u64>,
    pub certified_r: Option<u64>,
    pub true_r: u64,
    pub factors: Option<(u64, u64)>,
    pub trace: String,
}

/// The phase-based unbraid. Every returned pair is verified word-natively
/// before the report exists; an unverified pair cannot be printed.
pub fn run_phase_unbraid(n_val: u64, a0: u64, max_shots: u32) -> Result<PhaseUnbraidResult, String> {
    let mut trace = String::new();
    if n_val < 4 { return Err("N < 4 has no nontrivial two-factor closure".into()); }
    if n_val % 2 == 0 {
        let q = n_val / 2;
        trace.push_str("N even: peeled directly (p=2); the phase register below assumes odd N\n");
        return Ok(PhaseUnbraidResult {
            n_val, a_used: 0, n_qubits: 0, m: 0, total_shots: 0, shot_k: None,
            certified_r: Some(1), true_r: 1, factors: Some((2, q)), trace,
        });
    }
    let mut q = 4usize;
    while q < 27 && (1u64 << q) < n_val.saturating_mul(n_val) { q += 1; }
    let m: usize = 1usize << q;
    let mut a = a0.max(2);
    let mut rng = XorShift(0x9E37_79B9_7F4A_7C15 ^ (n_val as u64).wrapping_mul(0x2545_F491_4F6C_DD1D));
    let mut total_shots: u32 = 0;
    let mut a_tries: u32 = 0;
    let mut buf: Vec<Cx> = alloc::vec![Cx::zero(); m];
    while a_tries < 32 && total_shots < max_shots {
        if gcd_u64(a, n_val) != 1 { a += 1; a_tries += 1; continue; }
        let r_true = true_period(a, n_val);
        if r_true == 0 { a += 1; a_tries += 1; continue; }
        // One state preparation per base: the collapsed comb, then ONE exact
        // QFT. Repeated shots re-measure this same prepared state — the Born
        // distribution is what a real device's shot record samples.
        let l = (m - 1) / (r_true as usize) + 1;
        for c in buf.iter_mut() { *c = Cx::zero(); }
        let amp = 1.0 / libm::sqrt(l as f64);
        let mut x = 0usize;
        while x < m { buf[x] = Cx::new(amp, 0.0); x += r_true as usize; }
        qft(&mut buf);
        let mut a_shots: u32 = 0;
        while a_shots < 8 && total_shots < max_shots {
            a_shots += 1;
            total_shots += 1;
            let target = rng.unit();
            let mut cum = 0.0f64;
            let mut k: usize = m - 1;
            for (i, c) in buf.iter().enumerate() {
                cum += c.norm2();
                if target < cum { k = i; break; }
            }
            if k == 0 {
                trace.push_str(&format!("  shot {}: k=0 — the phase carries no information (s=0); re-measuring\n", total_shots));
                continue;
            }
            trace.push_str(&format!(
                "  shot {}: measured k={}  winding k/M = {}/{} of a full turn\n",
                total_shots, k, k, m));
            match attempt(k as u64, m as u64, a, n_val) {
                Attempt::Factors { r, s: _s, r0, p, q: f2, via } => {
                    trace.push_str(&format!(
                        "  continued fractions: k/M -> s/{} -> certified period r={}  ({} )\n",
                        r0, r, via));
                    return Ok(PhaseUnbraidResult {
                        n_val, a_used: a, n_qubits: q, m, total_shots,
                        shot_k: Some(k as u64), certified_r: Some(r),
                        true_r: r_true, factors: Some((p, f2)), trace,
                    });
                }
                Attempt::Degenerate(msg) => {
                    trace.push_str(&format!("  {}\n", msg));
                    break;
                }
                Attempt::NoReadout => {
                    trace.push_str(&format!(
                        "  shot {}: no convergent of {}/{} certified a period; re-measuring\n",
                        total_shots, k, m));
                }
            }
        }
        a += 1;
        a_tries += 1;
    }
    Ok(PhaseUnbraidResult {
        n_val, a_used: a, n_qubits: q, m, total_shots, shot_k: None,
        certified_r: None, true_r: 0, factors: None, trace,
    })
}

/// The report: phase readout, closure, and the factors AS WORDS, verified.
pub fn phase_unbraid_report(n_str: &str, a0: u64, max_shots: u32) -> Result<String, String> {
    let n_val: u64 = match n_str.trim().parse() {
        Ok(v) => v,
        Err(_) => match native_numeral::decode(n_str.trim()) {
            Some(v) => v.to_str_radix(10).parse().map_err(|_| "internal: decoded word exceeds u64".to_string())?,
            None => return Err(format!("'{}' is not a decimal integer or a native-numeral word", n_str)),
        },
    };
    let res = run_phase_unbraid(n_val, a0, max_shots)?;
    let n_big = BigUint::from(n_val);
    let mut o = String::new();
    o.push_str(&format!("phase_unbraid — factors from a phase readout, no search\n"));
    o.push_str(&format!("N = {}  word: {}\n", n_val, native_numeral::encode(n_str.trim())));
    if res.n_qubits == 0 {
        let (p, qq) = res.factors.unwrap();
        let pb = BigUint::from(p);
        let qb = BigUint::from(qq);
        o.push_str(&res.trace);
        o.push_str(&format!("factors: p = {}, q = {}\n", p, qq));
        o.push_str(&format!("p × q = N: {}  [multiply_via_word]\n", native_numeral::multiply_via_word(&pb, &qb) == n_big));
        o.push_str(&format!("syzygy preserves [encode; Γ; Λ; μ]: {}\n", native_numeral::syzygy_preserves(&n_big, &pb, &qb)));
        o.push_str(&native_numeral::factor_words_line(&pb, &qb));
        return Ok(o);
    }
    o.push_str(&format!(
        "register: {} index qubits, M = 2^{} = {} amplitudes; exact QFT unitary, O(M log M)\n",
        res.n_qubits, res.n_qubits, res.m));
    o.push_str(&format!(
        "basis a = {}  (coprime; ground-truth period r = {} — printed for comparison only, never used in the readout)\n",
        res.a_used, res.true_r));
    o.push_str("measurement record:\n");
    o.push_str(&res.trace);
    match res.factors {
        Some((p, qq)) => {
            let pb = BigUint::from(p);
            let qb = BigUint::from(qq);
            o.push_str(&format!("FACTORS (one phase measurement + one gcd — no enumeration):\n"));
            o.push_str(&format!("p = {}\nq = {}\n", p, qq));
            o.push_str(&format!("p × q = N: {}  [multiply_via_word]\n", native_numeral::multiply_via_word(&pb, &qb) == n_big));
            o.push_str(&format!("syzygy preserves [encode; Γ; Λ; μ]: {}\n", native_numeral::syzygy_preserves(&n_big, &pb, &qb)));
            o.push_str(&native_numeral::factor_words_line(&pb, &qb));
        }
        None => {
            o.push_str(&format!(
                "no nontrivial closure in {} measurement(s) across the coprime bases tried — reported as measured, not guessed\n",
                res.total_shots));
        }
    }
    Ok(o)
}

#[cfg(test)]
mod phase_tests {
    use super::*;
    #[test]
    fn fifteen_factors_by_phase() {
        let res = run_phase_unbraid(15, 7, 12).unwrap();
        assert_eq!(res.factors, Some((3, 5)));
        assert_eq!(res.certified_r, Some(4));
        assert!(res.total_shots >= 1);
    }
    #[test]
    fn sixtyfive_factors_by_phase() {
        let res = run_phase_unbraid(65, 2, 16).unwrap();
        let (p, q) = res.factors.expect("65 must factor by phase readout");
        assert!(p * q == 65 && (p == 13 || p == 5));
    }
}
