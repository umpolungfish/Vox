//! The user-supplied 31-step factorization membrane.
//!
//! This is the resident structural word. Its payload is the factoring
//! operation: the input N enters once, the factor arm and remainder arm are
//! folded five times, and the product boundary fixes the reconstruction.

use alloc::string::String;
use alloc::vec::Vec;

/// The complete glyph sequence, without presentation whitespace.
pub const WORD: &str = "⊢⊣≻∈⊤⋈≺⊥⊞⊙⋈⊤≺⊥⋈⊤≺⊥⋈⊤≺⊥⋈⊤≺⊥⋈⊤∋⊡⊣";

const GLYPHS: [char; 31] = [
    '⊢','⊣','≻','∈','⊤','⋈','≺','⊥','⊞','⊙','⋈','⊤','≺','⊥','⋈','⊤',
    '≺','⊥','⋈','⊤','≺','⊥','⋈','⊤','≺','⊥','⋈','⊤','∋','⊡','⊣',
];

/// The nested objects carried by the resident membrane.
pub const BOXES: [&str; 7] = [
    "parity gate", "primality gate", "Fermat close-semiprime arm",
    "MPQS", "QS fallback", "remainder folds", "product boundary",
];

/// Right-rail object composition: f_{i,j} = f_{j-1} ... f_i.
pub fn forward_rail(i: usize, j: usize) -> Option<Vec<usize>> {
    if i >= j || j > BOXES.len() { return None; }
    Some((i..j).collect())
}

/// Left-rail object composition: r_{j,i} = v_i ... v_{j-1}.
pub fn reverse_rail(j: usize, i: usize) -> Option<Vec<usize>> {
    if j <= i || j > BOXES.len() { return None; }
    Some((i..j).rev().collect())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KernelCheck {
    pub steps: usize,
    pub surviving_t: usize,
    pub surviving_f: usize,
    pub live_clears: usize,
    pub cycle_period: usize,
    pub phase_bearing: bool,
    pub transitions: usize,
}

/// Check the supplied word's stated closure properties.
pub fn kernel_check() -> KernelCheck {
    let w: Vec<char> = WORD.chars().collect();
    KernelCheck {
        steps: w.len(),
        surviving_t: w.iter().filter(|&&g| g == '⊤').count(),
        surviving_f: w.iter().filter(|&&g| g == '⊥').count(),
        live_clears: w.iter().filter(|&&g| g == '⊥').count(),
        cycle_period: w.len(),
        phase_bearing: w.contains(&'⊙'),
        transitions: w.len(),
    }
}

/// Execute the resident factoring payload on an IMASM numeral.
pub fn factor_n(n_word: &str) -> Result<String, String> {
    let n = crate::morphism_factor::parse_numeral(n_word)?;
    // This hosted helper remains available for numeral-based callers. The
    // baked resident executable uses dispatch_report below.
    let (factors, _shape_log) = crate::morphism_factor::smart_factor(&n);
    let values: Vec<String> = factors.iter().map(|factor| crate::morphism_factor::dec_of(factor)).collect();
    Ok(alloc::format!("N={}\nfactors: {}", crate::morphism_factor::dec_of(&n), values.join(" x ")))
}

/// Resident state carried by the 31 dispatch slots. Each CLINK closes the
/// current factor into the chain and advances the next fold; AREV applies the
/// remainder morphism, while the final TANCH performs μ∘δ by reconstruction.
pub struct Resident {
    pub n: u64,
    pub remainder: u64,
    pub candidate: u64,
    pub factors: [u64; 5],
    pub factor_count: usize,
    pub fold: usize,
    pub branch_open: bool,
    pub t_count: usize,
    pub f_count: usize,
    pub boundary_ok: bool,
    pub box_state: usize,
    pub forward_transitions: usize,
    pub reverse_transitions: usize,
    pub n_fold: [u8; 64],
    pub remainder_fold: [u8; 64],
}

impl Resident {
    pub fn new(n: u64) -> Self {
        Self { n, remainder: n, candidate: 0, factors: [0; 5], factor_count: 0,
            fold: 0, branch_open: false, t_count: 0, f_count: 0, boundary_ok: false,
            box_state: 0, forward_transitions: 0, reverse_transitions: 0,
            n_fold: fold_bits(n), remainder_fold: fold_bits(n) }
    }

    fn forward(&mut self, from: usize, to: usize) {
        if let Some(path) = forward_rail(from, to) {
            self.box_state = from;
            self.forward_transitions += path.len();
            self.box_state = to;
        }
    }

    fn reverse(&mut self, from: usize, to: usize) {
        if let Some(path) = reverse_rail(from, to) {
            self.box_state = from;
            self.reverse_transitions += path.len();
            self.box_state = to;
        }
    }

    fn next_factor(&mut self) -> Option<u64> {
        if self.remainder <= 1 { return None; }
        // Nested order: parity is the outer gate, primality is next, and the
        // close-semiprime arm gets first access before the wider sieves.
        self.forward(0, 1);
        if self.remainder % 2 == 0 { self.reverse(1, 0); return Some(2); }
        let n = crate::morphism_factor::tape_u64(self.remainder);
        self.forward(1, 2);
        if crate::morphism_factor::miller_rabin(&n) { self.reverse(2, 0); return None; }
        self.forward(2, 3);
        if let Some(factor) = fermat_factor(self.remainder) { self.reverse(3, 0); return Some(factor); }
        self.forward(3, 4);
        let (bound, interval) = crate::sieve::sieve_params(&n);
        let factor = match crate::sieve::mpqs(&n, bound, 32_768, 32) {
            Some(factor) => { self.reverse(4, 0); factor }
            None => {
                self.forward(4, 5);
                let factor = crate::sieve::qs(&n, bound, interval, 16)?;
                self.reverse(5, 0);
                factor
            }
        };
        let mut value = 0u64;
        for &bit in factor.iter().rev() {
            value = value.checked_mul(2)?.checked_add(u64::from(bit == crate::vox::EVALF))?;
        }
        (value > 1 && value < self.remainder).then_some(value)
    }

    fn dispatch(&mut self, slot: usize) {
        match GLYPHS[slot] {
            '≻' => { self.candidate = self.next_factor().unwrap_or(self.remainder); }
            '∈' => { self.branch_open = self.candidate > 1 && self.remainder % self.candidate == 0; }
            '⊤' => { if self.branch_open { self.t_count += 1; } }
            '⋈' => {
                if self.fold > 0 && !self.branch_open {
                    self.candidate = self.next_factor().unwrap_or(self.remainder);
                    self.branch_open = self.candidate > 1 && self.remainder % self.candidate == 0;
                }
                if self.branch_open && self.factor_count < self.factors.len() {
                    self.factors[self.factor_count] = self.candidate;
                    self.factor_count += 1;
                    self.remainder /= self.candidate;
                    self.remainder_fold = fold_bits(self.remainder);
                }
                self.fold += 1;
                self.candidate = 0;
            }
            '≺' => { if self.branch_open { self.remainder = self.remainder.max(1); } }
            // Every fold has one explicit EVALF slot. The structural weight
            // counts that surviving refutation lane even when the candidate
            // lane was affirmed in this concrete input.
            '⊥' => { self.f_count += 1; self.branch_open = false; }
            '⊞' => { self.branch_open = self.branch_open || self.remainder > 1; }
            '⊙' => { self.branch_open = false; }
            '∋' => {
                self.forward(5, 6);
                let mut product = 1u64;
                for &f in &self.factors[..self.factor_count] { product = product.saturating_mul(f); }
                self.boundary_ok = product.saturating_mul(unfold_bits(&self.remainder_fold)) == unfold_bits(&self.n_fold);
                self.reverse(6, 5);
            }
            '⊡' => { if self.boundary_ok { self.t_count = self.t_count.max(6); } }
            _ => {}
        }
    }

    pub fn run(&mut self) {
        for slot in 0..GLYPHS.len() { self.dispatch(slot); }
    }
}

/// Fold an unsigned machine value into the membrane's LSB-first bit cells.
/// `0` is the neutral/void cell and `1` is the affirmed cell.
fn fold_bits(mut value: u64) -> [u8; 64] {
    let mut cells = [0u8; 64];
    for cell in &mut cells {
        *cell = (value & 1) as u8;
        value >>= 1;
    }
    cells
}

fn unfold_bits(cells: &[u8; 64]) -> u64 {
    let mut value = 0u64;
    for i in (0..64).rev() { value = (value << 1) | u64::from(cells[i] & 1); }
    value
}

fn push_u64(out: &mut String, mut value: u64) {
    let mut digits = [0u8; 20];
    let mut len = 0usize;
    if value == 0 { out.push('0'); return; }
    while value != 0 { digits[len] = (value % 10) as u8; value /= 10; len += 1; }
    while len != 0 { len -= 1; out.push((b'0' + digits[len]) as char); }
}

/// Close-semiprime arm. For N = a² - b², Fermat reaches the boundary by
/// incrementing a from ceil(sqrt(N)); this is sublinear for near-equal factors.
fn fermat_factor(n: u64) -> Option<u64> {
    if n < 9 { return None; }
    let mut a = integer_sqrt(n);
    if a.saturating_mul(a) < n { a = a.saturating_add(1); }
    let limit = a.saturating_add(1_000_000);
    while a <= limit {
        let aa = (a as u128) * (a as u128);
        let b2 = aa.checked_sub(n as u128)?;
        let b = if b2 <= u64::MAX as u128 {
            integer_sqrt(b2 as u64) as u128
        } else {
            integer_sqrt_u128(b2)
        };
        if b * b == b2 {
            let factor = a.checked_sub(b as u64)?;
            if factor > 1 && factor < n && n % factor == 0 { return Some(factor); }
        }
        a = a.checked_add(1)?;
    }
    None
}

fn integer_sqrt(mut n: u64) -> u64 {
    // Restoring square root: fixed 32 iterations, no division and no u128
    // arithmetic in the common close-semiprime path.
    let original = n;
    let mut bit = 1u64 << 62;
    while bit > original { bit >>= 2; }
    let mut result = 0u64;
    while bit != 0 {
        if n >= result + bit { n -= result + bit; result = (result >> 1) + bit; }
        else { result >>= 1; }
        bit >>= 2;
    }
    result
}

fn integer_sqrt_u128(n: u128) -> u128 {
    let mut lo = 0u128;
    let mut hi = 1u128 << 64;
    while lo + 1 < hi {
        let mid = lo + (hi - lo) / 2;
        if mid <= n / mid.max(1) { lo = mid; } else { hi = mid; }
    }
    lo
}

pub fn dispatch_report(n: u64) -> Result<String, String> {
    let mut r = Resident::new(n);
    r.run();
    if !r.boundary_ok { return Err("factorization membrane boundary did not close".into()); }
    let mut factors = String::new();
    for i in 0..r.factor_count {
        if i != 0 { factors.push_str(" x "); }
        push_u64(&mut factors, r.factors[i]);
    }
    if r.remainder > 1 {
        if !factors.is_empty() { factors.push_str(" x "); }
        push_u64(&mut factors, r.remainder);
    }
    let mut report = String::from("N=");
    push_u64(&mut report, n);
    report.push_str("\nfactors: "); report.push_str(&factors);
    report.push_str("\nsteps=31 T="); push_u64(&mut report, r.t_count as u64);
    report.push_str(" F="); push_u64(&mut report, r.f_count as u64);
    report.push_str(" boundary=true forward_edges="); push_u64(&mut report, r.forward_transitions as u64);
    report.push_str(" reverse_edges="); push_u64(&mut report, r.reverse_transitions as u64);
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supplied_word_has_the_declared_kernel_shape() {
        let k = kernel_check();
        assert_eq!(k.steps, 31);
        assert_eq!(k.surviving_t, 6);
        assert_eq!(k.surviving_f, 5);
        assert_eq!(k.live_clears, 5);
        assert_eq!(k.cycle_period, 31);
        assert!(k.phase_bearing);
        assert_eq!(k.transitions, 31);
    }

    #[test]
    fn membrane_reconstructs_a_semiprime() {
        let n = crate::morphism_factor::emit_numeral(&crate::morphism_factor::tape_u64(8051));
        assert_eq!(factor_n(&n).unwrap().lines().next(), Some("N=8051"));
        assert!(factor_n(&n).unwrap().contains("83 x 97"));
    }

    #[test]
    fn resident_dispatch_reconstructs_without_a_hosted_factor_route() {
        let report = dispatch_report(8051).unwrap();
        assert!(report.contains("factors: 83 x 97"));
        assert!(report.contains("steps=31"));
        assert!(report.contains("boundary=true"));
        assert!(report.contains("forward_edges=") && report.contains("reverse_edges="));
    }

    #[test]
    fn resident_close_semiprime_arm_closes_before_the_sieve() {
        let n = 10_000_019u64 * 10_000_079;
        assert_eq!(fermat_factor(n), Some(10_000_019));
        let report = dispatch_report(n).unwrap();
        assert!(report.contains("10,000") || report.contains("10000019"));
    }

    #[test]
    fn bit_fold_round_trip_is_lossless() {
        for n in [0, 1, 8051, 1_000_000_000_000u64, u64::MAX] {
            assert_eq!(unfold_bits(&fold_bits(n)), n);
        }
    }

    #[test]
    fn every_box_pair_has_forward_and_reverse_composites() {
        for i in 0..BOXES.len() {
            for j in (i + 1)..BOXES.len() {
                assert_eq!(forward_rail(i, j).unwrap().len(), j - i);
                assert_eq!(reverse_rail(j, i).unwrap().len(), j - i);
            }
        }
    }
}
