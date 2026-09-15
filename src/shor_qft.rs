//! Hosted port of G-mOMonadOS/src/shor_qft.rs.
//! The modular orbit prepares the measured branch. A shared phase table feeds
//! nested even/odd transform stages, then continued fractions recover a period
//! from the resulting peaks. The positive Fourier sign and unitary normalization
//! match the source's direct transform, retained below as a test control.

use crate::membrane_complex::Complex;
mod libm {
    pub fn sqrt(x: f64) -> f64 { x.sqrt() }
    #[cfg(test)] pub fn cos(x: f64) -> f64 { x.cos() }
    #[cfg(test)] pub fn sin(x: f64) -> f64 { x.sin() }
}
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

fn mod_pow_u64(mut base: u64, mut exp: u64, modulus: u64) -> u64 {
    if modulus <= 1 {
        return 0;
    }
    let mut result = 1u64;
    base %= modulus;
    while exp > 0 {
        if exp & 1 != 0 {
            result = ((result as u128 * base as u128) % modulus as u128) as u64;
        }
        exp >>= 1;
        base = ((base as u128 * base as u128) % modulus as u128) as u64;
    }
    result
}

fn gcd_u64(a: u64, b: u64) -> u64 {
    let (mut a, mut b) = (a, b);
    while b != 0 {
        let t = b;
        b = a % b;
        a = t;
    }
    a
}

/// The true period of a mod n_val, by direct classical walk -- used only
/// as the ground truth to check the simulation's extracted period
/// against, never as part of the extraction itself.
fn true_period(a: u64, n_val: u64) -> u64 {
    let mut val = 1u64;
    for r in 1..=n_val {
        val = ((val as u128 * a as u128) % n_val as u128) as u64;
        if val == 1 {
            return r;
        }
    }
    0
}

/// The QFT, applied as the exact unitary DFT matrix on the register:
/// out[k] = (1/sqrt(M)) * sum_x state[x] * exp(2*pi*i*k*x/M).
/// This is not an approximation of what the QFT gate sequence does --
/// it is the same unitary, computed directly rather than via H/CR gates,
/// exact for the register sizes this module uses.
#[cfg(test)]
fn qft_forward_reference(state: &[Complex]) -> Vec<Complex> {
    let m = state.len();
    let scale = 1.0 / libm::sqrt(m as f64);
    let mut out = alloc::vec![Complex::zero(); m];
    for k in 0..m {
        let mut acc = Complex::zero();
        for (x, &amp) in state.iter().enumerate() {
            if amp.re == 0.0 && amp.im == 0.0 {
                continue;
            }
            let angle = 2.0 * core::f64::consts::PI * (k as f64) * (x as f64) / (m as f64);
            acc = acc + amp * Complex::new(libm::cos(angle), libm::sin(angle));
        }
        out[k] = acc.scale(scale);
    }
    out
}

/// The enclosing transform holds roots of unity and the permutation. Each
/// doubling stage fuses even and odd subtransforms using those same roots.
struct QftMembrane {
    roots: Vec<Complex>,
    permutation: Vec<usize>,
}

impl QftMembrane {
    fn new(m: usize) -> Self {
        assert!(m.is_power_of_two());
        let bits = m.trailing_zeros();
        let permutation = (0..m).map(|i| if m == 1 { 0 } else {
            i.reverse_bits() >> (usize::BITS - bits)
        }).collect();
        let roots = (0..m / 2).map(|j| {
            let angle = 2.0 * core::f64::consts::PI * j as f64 / m as f64;
            Complex::new(angle.cos(), angle.sin())
        }).collect();
        Self { roots, permutation }
    }

    fn apply(&self, state: &[Complex]) -> Vec<Complex> {
        let m = self.permutation.len();
        assert_eq!(state.len(), m);
        let mut out: Vec<_> = self.permutation.iter().map(|&i| state[i]).collect();
        let mut width = 2;
        while width <= m {
            let half = width / 2;
            let stride = m / width;
            for block in out.chunks_exact_mut(width) {
                for j in 0..half {
                    let even = block[j];
                    let odd = block[j + half] * self.roots[j * stride];
                    block[j] = even + odd;
                    block[j + half] = even - odd;
                }
            }
            if width == m { break; }
            width *= 2;
        }
        let scale = 1.0 / (m as f64).sqrt();
        for value in &mut out { *value = value.scale(scale); }
        out
    }
}

fn qft_forward(state: &[Complex]) -> Vec<Complex> {
    QftMembrane::new(state.len()).apply(state)
}

fn modular_orbit(a: u64, modulus: u64, size: usize) -> Vec<u64> {
    let mut value = 1;
    (0..size).map(|_| {
        let current = value;
        value = ((value as u128 * (a % modulus) as u128) % modulus as u128) as u64;
        current
    }).collect()
}

/// Continued-fraction expansion of k/m, returning every convergent
/// (numerator, denominator) -- the standard classical post-processing
/// step of Shor's algorithm, run on the frequency actually measured from
/// the simulated distribution below, not on a hypothetical.
fn convergents(mut k: u64, mut m: u64) -> Vec<(u64, u64)> {
    let mut out = Vec::new();
    // p_{-2}=0, p_{-1}=1, q_{-2}=1, q_{-1}=0 -- the standard convergent
    // recurrence's seed values. Checked directly against k=64, m=256
    // (period 4 case): with this seed the first two convergents come out
    // (0,1) then (1,4), matching k/m=1/4 exactly. The previous version
    // had p and q each seeded with their own two values swapped, which
    // silently produced every convergent's reciprocal instead.
    let (mut p_prev, mut p_curr) = (0u64, 1u64);
    let (mut q_prev, mut q_curr) = (1u64, 0u64);
    while m != 0 {
        let a = k / m;
        let p_next = a.wrapping_mul(p_curr).wrapping_add(p_prev);
        let q_next = a.wrapping_mul(q_curr).wrapping_add(q_prev);
        out.push((p_next, q_next));
        p_prev = p_curr;
        p_curr = p_next;
        q_prev = q_curr;
        q_curr = q_next;
        let rem = k % m;
        k = m;
        m = rem;
    }
    out
}

pub struct ShorSimResult {
    pub a: u64,
    pub n_val: u64,
    pub n_qubits: usize,
    pub register_size: usize,
    pub true_period: u64,
    pub top_peaks: Vec<(usize, f64)>,
    pub extracted_period: Option<u64>,
    pub factors: Option<(u64, u64)>,
}

/// The full pipeline: build the real post-measurement index-register
/// state (a periodic amplitude comb of period r, r = true_period(a,
/// n_val), collapsed onto the branch where the output register reads
/// f(0)=1 -- one genuine, equally-likely branch among r), run it through
/// the exact QFT, read the resulting probability distribution, extract
/// the strongest peaks, run continued fractions on the best one, and
/// attempt to factor n_val via gcd on the recovered period. `n_qubits`
/// supports 1..=14 as in the source entry. The nested transform uses
/// O(M log M) arithmetic and O(M) storage at register size M=2^n_qubits.
pub fn simulate_shor(a: u64, n_val: u64, n_qubits: usize) -> Result<ShorSimResult, String> {
    if n_qubits == 0 || n_qubits > 14 {
        return Err(format!(
            "n_qubits={} outside supported range 1..=14",
            n_qubits
        ));
    }
    if n_val < 2 {
        return Err(format!("n_val={} is not a valid modulus (need ≥ 2)", n_val));
    }
    if gcd_u64(a, n_val) != 1 {
        return Err(format!(
            "gcd(a={}, N={}) = {} ≠ 1 -- a must be coprime to N for period-finding to apply",
            a, n_val, gcd_u64(a, n_val)
        ));
    }
    let m = 1usize << n_qubits;
    let r_true = true_period(a, n_val);

    // Step 1-2 (uniform superposition) + step 3 (ModExp as a permutation)
    // + step 4 (measure the output register, branch f(x)=1): computed
    // directly rather than gate-by-gate, since a full H-layer into a
    // permutation into a projective measurement onto one output value
    // has one exact closed form -- equal amplitude on every x with
    // a^x mod n_val landing on the observed value, zero elsewhere. This
    // is the real post-measurement state, not an approximation of it.
    let f_vals = modular_orbit(a, n_val, m);
    let observed = f_vals[0]; // x=0 always maps to 1 -- a real, always-available branch
    let matching: Vec<usize> = (0..m).filter(|&x| f_vals[x] == observed).collect();
    let amp = 1.0 / libm::sqrt(matching.len() as f64);
    let mut state = alloc::vec![Complex::zero(); m];
    for &x in &matching {
        state[x] = Complex::new(amp, 0.0);
    }

    // Step 5: the QFT, exact.
    let spectrum = qft_forward(&state);
    let probs: Vec<f64> = spectrum.iter().map(|c| c.norm_sq()).collect();

    // Step 6: read off the strongest peaks -- what an actual measurement
    // would sample from, weighted by these same probabilities.
    let mut indexed: Vec<(usize, f64)> = probs.iter().cloned().enumerate().collect();
    indexed.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    let top_peaks: Vec<(usize, f64)> = indexed.into_iter().take(8).collect();

    // Step 7: continued fractions, tried against each top peak in turn,
    // not just the single strongest one. k=0 is always a peak (every
    // periodic comb has a zero-frequency component) and is always
    // uninformative -- its convergent is 0/1, certifying nothing -- so
    // whenever probabilities tie exactly (a real, common outcome when M
    // is a multiple of r, seen directly in this simulation's own
    // output), taking only the first-sorted peak can hand the extractor
    // the one peak with no period information in it. A real measurement
    // would simply land on a different sample if k=0 came up; trying
    // every peak a real measurement could have landed on matches that,
    // rather than stopping silently at the first.
    let mut extracted_period = None;
    for &(k, _) in &top_peaks {
        if k == 0 {
            continue;
        }
        let mut found = None;
        for (_, q) in convergents(k as u64, m as u64) {
            if q > 0 && q < n_val && mod_pow_u64(a, q, n_val) == 1 {
                found = Some(q);
                break;
            }
        }
        if found.is_some() {
            extracted_period = found;
            break;
        }
    }

    // Step 8: factor n_val via gcd, the standard closing step, only if
    // the extracted period is even and not a trivial square root of 1.
    let mut factors = None;
    if let Some(r) = extracted_period {
        if r % 2 == 0 {
            let half = mod_pow_u64(a, r / 2, n_val);
            if half != n_val - 1 {
                let f1 = gcd_u64(if half >= 1 { half - 1 } else { n_val - 1 }, n_val);
                let f2 = gcd_u64(half + 1, n_val);
                if f1 > 1 && f1 < n_val {
                    factors = Some((f1, n_val / f1));
                } else if f2 > 1 && f2 < n_val {
                    factors = Some((f2, n_val / f2));
                }
            }
        }
    }

    Ok(ShorSimResult {
        a,
        n_val,
        n_qubits,
        register_size: m,
        true_period: r_true,
        top_peaks,
        extracted_period,
        factors,
    })
}

pub fn report(result: &ShorSimResult) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "shor_qft: a={}, N={}, {} index qubits (register size {})\n",
        result.a, result.n_val, result.n_qubits, result.register_size
    ));
    out.push_str(&format!("  true period (classical ground truth): {}\n", result.true_period));
    out.push_str("  strongest peaks in the measured distribution (index, probability):\n");
    for &(k, p) in &result.top_peaks {
        let ratio = k as f64 * result.true_period as f64 / result.register_size as f64;
        out.push_str(&format!(
            "    k={:<6} p={:.5}  k/M*r={:.3} (should land near an integer if peaks are where theory predicts)\n",
            k, p, ratio
        ));
    }
    match result.extracted_period {
        Some(r) => {
            out.push_str(&format!(
                "  period extracted via continued fractions from a measured peak: {}  (matches true period: {})\n",
                r, r == result.true_period
            ));
        }
        None => out.push_str("  continued fractions did not certify a period from any of the top peaks\n"),
    }
    match result.factors {
        Some((f1, f2)) => {
            out.push_str(&format!(
                "  factors recovered via gcd: {} * {} = {}  (verified: {})\n",
                f1, f2, f1 * f2, f1 * f2 == result.n_val
            ));
        }
        None => out.push_str("  no factors recovered this run (even/odd or gcd degeneracy -- try a different a)\n"),
    }
    out
}


#[cfg(test)]
mod membrane_tests {
    use super::*;
    fn check_spectrum(state: &[Complex]) {
        let reference = qft_forward_reference(state);
        let actual = qft_forward(state);
        for (i, (a, b)) in actual.iter().zip(&reference).enumerate() {
            assert!((*a - *b).norm_sq().sqrt() < 1e-9, "frequency={i} {a:?} != {b:?}");
        }
        let before: f64 = state.iter().map(Complex::norm_sq).sum();
        let after: f64 = actual.iter().map(Complex::norm_sq).sum();
        assert!((before - after).abs() < 1e-9 * before.max(1.0));
    }

    #[test]
    fn membrane_amplitudes_match_direct_transform() {
        for m in [1, 2, 4, 8, 32, 128] {
            let mut state: Vec<_> = (0..m).map(|i|
                Complex::new((i as f64 * 0.37).sin(), (i as f64 * 0.19).cos())).collect();
            check_spectrum(&state);
            state.reverse();
            check_spectrum(&state);
            for (i, value) in state.iter_mut().enumerate() {
                *value = if i % 6 == 0 { Complex::one() } else { Complex::zero() };
            }
            check_spectrum(&state);
        }
    }

    #[test]
    fn membrane_orbit_and_period_controls() {
        for (a, n) in [(7, 15), (2, 21), (2, 35), (8, 21)] {
            let orbit = modular_orbit(a, n, 256);
            for (x, value) in orbit.into_iter().enumerate() {
                assert_eq!(value, mod_pow_u64(a, x as u64, n));
            }
            let result = simulate_shor(a, n, 10).unwrap();
            assert_eq!(result.extracted_period, Some(result.true_period));
            if let Some((p, q)) = result.factors {
                assert!(p > 1 && q > 1);
                assert_eq!(p as u128 * q as u128, n as u128);
            }
        }
        assert_eq!(simulate_shor(7, 15, 8).unwrap().factors, Some((3, 5)));
    }

    #[test]
    fn membrane_timing_control() {
        let m = 1024;
        let state: Vec<_> = (0..m).map(|i|
            Complex::new((i as f64 * 0.37).sin(), (i as f64 * 0.19).cos())).collect();
        let start = std::time::Instant::now();
        let reference = qft_forward_reference(std::hint::black_box(&state));
        let old = start.elapsed();
        let start = std::time::Instant::now();
        let nested = qft_forward(std::hint::black_box(&state));
        let elapsed = start.elapsed();
        for (a, b) in nested.iter().zip(reference) {
            assert!((*a - b).norm_sq().sqrt() < 1e-8);
        }
        println!("qft size={m} direct={old:?} nested_with_setup={elapsed:?}");
    }
}
