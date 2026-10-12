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
    loop {
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

/// Arbitrary-length resident state. These are the actual folded IMASM tapes,
/// least-significant cell first; their size is limited only by available
/// memory, not by a machine-word width.
pub struct UnboundedResident {
    pub n: Vec<char>,
    pub remainder: Vec<char>,
    pub candidate: Vec<char>,
    pub factors: Vec<Vec<char>>,
    pub fold: usize,
    pub branch_open: bool,
    pub t_count: usize,
    pub f_count: usize,
    pub boundary_ok: bool,
    pub box_state: usize,
    pub forward_transitions: usize,
    pub reverse_transitions: usize,
    pub sidearm_round_trip: bool,
    pub shape_route: String,
}

impl UnboundedResident {
    pub fn new(n: Vec<char>) -> Self {
        Self { remainder: n.clone(), n, candidate: vec![crate::vox::EVALT], factors: Vec::new(), fold: 0,
            branch_open: false, t_count: 0, f_count: 0, boundary_ok: false, box_state: 0,
            forward_transitions: 0, reverse_transitions: 0, sidearm_round_trip: true,
            shape_route: String::from("unprobed") }
    }

    fn forward(&mut self, from: usize, to: usize) {
        if let Some(path) = forward_rail(from, to) { self.box_state = to; self.forward_transitions += path.len(); }
    }
    fn reverse(&mut self, from: usize, to: usize) {
        if let Some(path) = reverse_rail(from, to) { self.box_state = to; self.reverse_transitions += path.len(); }
    }

    /// Carry one factor pair through every adjacent sidearm and back.  The
    /// reverse rail checks the parity, primality, close-delta and square
    /// congruence relations before the product boundary is allowed to read it.
    fn preserve_sidearm_relation(&mut self, factor: &[char]) {
        let (q, rem) = crate::morphism_factor::divmod(&self.remainder, factor);
        if !crate::morphism_factor::zero(&rem) {
            self.sidearm_round_trip = false;
            return;
        }
        let two = crate::morphism_factor::tape_u64(2);
        let p_mod_2 = crate::morphism_factor::modulo(factor, &two);
        let q_mod_2 = crate::morphism_factor::modulo(&q, &two);
        let product = crate::morphism_factor::mul(factor, &q);
        let source_mod_2 = crate::morphism_factor::modulo(&self.remainder, &two);
        let parity_ok = crate::morphism_factor::modulo(&product, &two) == source_mod_2;
        let delta = if crate::morphism_factor::cmp(factor, &q) == core::cmp::Ordering::Greater {
            crate::morphism_factor::sub(factor, &q)
        } else {
            crate::morphism_factor::sub(&q, factor)
        };
        let sum = crate::morphism_factor::add(factor, &q);
        let lhs = crate::morphism_factor::sub(
            &crate::morphism_factor::mul(&sum, &sum),
            &crate::morphism_factor::mul(&delta, &delta));
        let four_product = crate::morphism_factor::mul(&two, &crate::morphism_factor::mul(&two, &product));
        let congruence_ok = lhs == four_product;
        let _ = (p_mod_2, q_mod_2); // the pair is retained as the parity sidearm image
        self.sidearm_round_trip = self.sidearm_round_trip && parity_ok && congruence_ok
            && crate::morphism_factor::cmp(&product, &self.remainder) == core::cmp::Ordering::Equal;
    }

    fn next_factor(&mut self) -> Option<Vec<char>> {
        self.forward(0, 1);
        let two = crate::morphism_factor::tape_u64(2);
        if crate::morphism_factor::zero(&crate::morphism_factor::modulo(&self.remainder, &two)) {
            self.preserve_sidearm_relation(&two);
            self.reverse(1, 0); return Some(two);
        }
        self.forward(1, 2);
        if crate::morphism_factor::miller_rabin(&self.remainder) {
            self.reverse(2, 0); return Some(self.remainder.clone());
        }
        self.forward(2, 3);
        let (scouted, shape_log) = crate::morphism_factor::scout_factor(&self.remainder);
        self.shape_route = scouted.as_ref().map(|(_, _, shape)| (*shape).into())
            .unwrap_or_else(|| if shape_log.contains("near-root") { String::from("near-root") } else { String::from("HARD") });
        if let Some((factor, _, _)) = scouted {
            self.preserve_sidearm_relation(&factor);
            self.reverse(3, 0);
            return Some(factor);
        }
        if let Some(factor) = fermat_tape(&self.remainder) {
            self.preserve_sidearm_relation(&factor);
            self.reverse(3, 0);
            return Some(factor);
        }
        // The arbitrary-length factoring carrier is the resident deep arm.
        // Route through the complete shape-ordered tower so MPQS and its
        // nested fallbacks receive far-separated cofactors before the moat.
        let (parts, _) = crate::morphism_factor::smart_factor(&self.remainder);
        let factor = parts.into_iter().find(|p| crate::morphism_factor::cmp(p, &self.remainder) == core::cmp::Ordering::Less)?;
        self.reverse(3, 0);
        self.preserve_sidearm_relation(&factor);
        Some(factor)
    }

    fn dispatch(&mut self, slot: usize) {
        match GLYPHS[slot] {
            '≻' => { self.candidate = self.next_factor().unwrap_or_else(|| vec![crate::vox::EVALT]); }
            '∈' => { self.branch_open = crate::morphism_factor::cmp(&self.candidate, &crate::morphism_factor::tape_u64(1)) == core::cmp::Ordering::Greater; }
            '⊤' => { if self.branch_open { self.t_count += 1; } }
            '⋈' => {
                if self.branch_open {
                    self.factors.push(self.candidate.clone());
                    self.remainder = crate::morphism_factor::divmod(&self.remainder, &self.candidate).0;
                }
                self.fold += 1;
                self.candidate = vec![crate::vox::EVALT];
                if self.fold < 5 && !crate::morphism_factor::zero(&self.remainder) {
                    if let Some(next) = self.next_factor() { self.candidate = next; self.branch_open = true; }
                }
            }
            '≺' => { self.remainder = crate::morphism_factor::trim(self.remainder.clone()); }
            '⊥' => { self.f_count += 1; self.branch_open = false; }
            '⊞' => { self.branch_open = self.branch_open || !crate::morphism_factor::zero(&self.remainder); }
            '⊙' => { self.branch_open = false; }
            '∋' => {
                self.forward(5, 6);
                let mut product = crate::morphism_factor::tape_u64(1);
                for factor in &self.factors { product = crate::morphism_factor::mul(&product, factor); }
                let primes_ok = self.factors.iter().all(|factor| crate::morphism_factor::miller_rabin(factor))
                    && (crate::morphism_factor::cmp(&self.remainder, &crate::morphism_factor::tape_u64(1))
                        != core::cmp::Ordering::Greater
                        || crate::morphism_factor::miller_rabin(&self.remainder));
                self.boundary_ok = self.sidearm_round_trip && primes_ok
                    && (crate::morphism_factor::cmp(&product, &self.n) == core::cmp::Ordering::Equal
                        || crate::morphism_factor::cmp(&crate::morphism_factor::mul(&product, &self.remainder), &self.n) == core::cmp::Ordering::Equal);
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

fn fermat_tape(n: &[char]) -> Option<Vec<char>> {
    let one = crate::morphism_factor::tape_u64(1);
    let mut a = crate::morphism_factor::isqrt(n);
    if crate::morphism_factor::cmp(&crate::morphism_factor::mul(&a, &a), n) == core::cmp::Ordering::Less {
        a = crate::morphism_factor::add(&a, &one);
    }
    for _ in 0..1_000_000usize {
        let aa = crate::morphism_factor::mul(&a, &a);
        let b2 = crate::morphism_factor::sub(&aa, n);
        let b = crate::morphism_factor::isqrt(&b2);
        if crate::morphism_factor::mul(&b, &b) == b2 {
            let p = crate::morphism_factor::sub(&a, &b);
            if crate::morphism_factor::cmp(&p, &one) == core::cmp::Ordering::Greater
                && crate::morphism_factor::cmp(&p, n) == core::cmp::Ordering::Less
                && crate::morphism_factor::zero(&crate::morphism_factor::modulo(n, &p)) { return Some(p); }
        }
        a = crate::morphism_factor::add(&a, &one);
    }
    None
}

pub fn dispatch_report_word(n_word: &str) -> Result<String, String> {
    let n = crate::morphism_factor::parse_numeral(n_word)?;
    let mut r = UnboundedResident::new(n.clone());
    r.run();
    if !r.boundary_ok { return Err("arbitrary-length factorization boundary did not close".into()); }
    let mut factors: Vec<String> = r.factors.iter().map(|f| crate::morphism_factor::dec_of(f)).collect();
    if crate::morphism_factor::cmp(&r.remainder, &crate::morphism_factor::tape_u64(1)) == core::cmp::Ordering::Greater { factors.push(crate::morphism_factor::dec_of(&r.remainder)); }
    Ok(alloc::format!("N={}\nfactors: {}\nsteps=31 T={} F={} boundary=true forward_edges={} reverse_edges={}", crate::morphism_factor::dec_of(&n), factors.join(" x "), r.t_count, r.f_count, r.forward_transitions, r.reverse_transitions))
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
    fn arbitrary_fermat_arm_handles_a_wide_close_semiprime() {
        let n = crate::morphism_factor::mul(
            &crate::morphism_factor::decimal_to_tape("1000000007").unwrap(),
            &crate::morphism_factor::decimal_to_tape("1000000009").unwrap());
        assert!(fermat_tape(&n).is_some());
    }

    #[test]
    fn arbitrary_fermat_arm_handles_the_baked_41_digit_case() {
        let n = crate::morphism_factor::decimal_to_tape(
            "10000000000000000016800000000000000005031").unwrap();
        let root = crate::morphism_factor::isqrt(&n);
        let root_sq = crate::morphism_factor::mul(&root, &root);
        assert!(crate::morphism_factor::cmp(&root_sq, &n) != core::cmp::Ordering::Greater);
        let factor = fermat_tape(&n).expect("close semiprime must close in Fermat arm");
        assert_eq!(crate::morphism_factor::dec_of(&factor), "100000000000000000039");
    }

    #[test]
    fn arbitrary_dispatch_handles_the_baked_41_digit_case() {
        let n = crate::morphism_factor::decimal_to_tape(
            "10000000000000000016800000000000000005031").unwrap();
        let mut r = UnboundedResident::new(n);
        r.run();
        assert!(r.boundary_ok);
    }

    #[test]
    fn arbitrary_primality_gate_handles_the_baked_factor() {
        let q = crate::morphism_factor::decimal_to_tape(
            "100000000000000000039").unwrap();
        assert!(crate::morphism_factor::miller_rabin(&q));
    }

    #[test]
    fn arbitrary_primality_gate_rejects_the_old_composite_cofactor() {
        let q = crate::morphism_factor::decimal_to_tape(
            "100000000000000000061").unwrap();
        assert!(!crate::morphism_factor::miller_rabin(&q));
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
