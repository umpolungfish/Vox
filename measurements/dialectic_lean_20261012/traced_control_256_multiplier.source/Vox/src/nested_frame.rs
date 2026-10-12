//! nested_frame.rs — capacity-gated nesting carrier for semiprime factoring.
//!
//! One nested frame holds BOTH descriptions of the same unresolved factor:
//!   LOW   : p ≡ r      (mod 2^k)
//!   HIGH  : L ≤ p ≤ U
//!   QLOW  : q ≡ N·r⁻¹  (mod 2^k)   (p odd ⇒ r⁻¹ exists; q's low residue is
//!                                   DETERMINED once p's residue is — no
//!                                   independent bit-by-bit wind of q)
//!   QHIGH : ceil(N/U) ≤ q ≤ floor(N/L)
//!
//! The bridge is arithmetic, not enumerative: for a residue r mod M inside the
//! interval [L,U], the first member is x0 = L + ((r−L) mod M); if x0 > U the
//! frame dies, else its exact remaining capacity is 1 + floor((U−x0)/M).
//! Counted for p and q at once, every frame carries a real capacity — a number,
//! not a Boolean "still alive".
//!
//! reenter() projects the child capacities for BOTH refinements — LOW
//! (mod 2^k → 2^(k+1)) and HIGH (interval bisection) — and enters the split
//! whose children carry the smaller total capacity. capacity = 0 dies;
//! capacity = 1 FIXes; capacity > 1 re-enters. Nesting follows information gain.

use crate::morphism_factor::tape_u64;
use crate::vox::EVALF;
use alloc::vec::Vec;

/// A numeral tape over IMASM marks (LSB-first; EVALF = 1). The four boundary
/// descriptions of a frame are tapes; the two capacities are u64 counts.
pub type Tape = Vec<char>;


/// Tape → u64 (LSB-first; EVALF = 1), for the residue-in-interval bridge.
pub fn tape_to_u64(t: &[char]) -> u64 {
    let mut v = 0u64;
    for (i, &c) in t.iter().enumerate() {
        if i < 64 && c == EVALF { v |= 1u64 << i; }
    }
    v
}

/// Integer square root (no_std Newton).
pub fn isqrt_u64(n: u64) -> u64 {
    if n < 2 { return n; }
    let bits = 64 - n.leading_zeros();
    let mut x = 1u64 << (bits / 2 + 1);
    loop {
        let y = (x + n / x) / 2;
        if y >= x { return x; }
        x = y;
    }
}

fn div_ceil(a: u64, b: u64) -> u64 { if b == 0 { 0 } else { (a + b - 1) / b } }

/// count_residue_in_interval — exact count of members of the arithmetic
/// progression r (mod M) that lie in [lo, hi]. This is the whole bridge.
pub fn count_residue_in_interval(residue: u64, modulus: u64, lo: u64, hi: u64) -> u64 {
    if modulus == 0 || lo > hi { return 0; }
    let r = residue % modulus;
    let delta = (r + modulus - (lo % modulus)) % modulus;
    let x0 = lo + delta;
    if x0 > hi { return 0; }
    1 + (hi - x0) / modulus
}

/// derive_q_residue — q ≡ N·p⁻¹ (mod 2^k). p odd ⇒ p⁻¹ exists mod 2^k.
pub fn derive_q_residue(n: u64, p_residue: u64, k: u32) -> u64 {
    if k == 0 { return 0; }
    let inv = inv_mod_pow2(p_residue, k);
    let modulus = 1u64 << k;
    (n.wrapping_mul(inv)) & (modulus - 1)
}

/// Inverse of an odd a modulo 2^k, by Newton doubling.
pub fn inv_mod_pow2(a: u64, k: u32) -> u64 {
    if k == 0 { return 0; }
    let mut x: u64 = 1; // inverse of a mod 2
    let mut bits: u32 = 1;
    while bits < k {
        let nb = (bits * 2).min(k);
        x = x.wrapping_mul(2u64.wrapping_sub(a.wrapping_mul(x)));
        if nb < 64 { x &= (1u64 << nb) - 1; }
        bits = nb;
    }
    x & ((1u64 << k) - 1)
}

/// One nested frame: both descriptions of the same unresolved factor.
#[derive(Clone)]
pub struct NestedFrame {
    pub k: usize,
    pub p_residue: Tape,
    pub q_residue: Tape,
    pub p_lo: Tape,
    pub p_hi: Tape,
    pub q_lo: Tape,
    pub q_hi: Tape,
    pub capacity_p: u64,
    pub capacity_q: u64,
}

impl NestedFrame {
    /// The number of admissible factor completions is bounded by both counts.
    pub fn live_capacity(&self) -> u64 { self.capacity_p.min(self.capacity_q) }
}

/// Recompute both capacities from the boundary descriptions (the bridge).
fn recompute(f: &mut NestedFrame) {
    let m = 1u64 << f.k;
    let pr = tape_to_u64(&f.p_residue);
    let qr = tape_to_u64(&f.q_residue);
    f.capacity_p = count_residue_in_interval(pr, m, tape_to_u64(&f.p_lo), tape_to_u64(&f.p_hi));
    f.capacity_q = count_residue_in_interval(qr, m, tape_to_u64(&f.q_lo), tape_to_u64(&f.q_hi));
}

/// The root frame: p odd, p in [3, isqrt(N)]; q = N/p mirrored. k = 1, so the
/// low residue of p is 1 and q's is derived — the coupling is exact from the start.
pub fn root_frame(n: u64) -> Option<NestedFrame> {
    if n < 9 || n & 1 == 0 { return None; }
    let p_lo = 3u64;
    let p_hi = isqrt_u64(n);
    if p_hi < p_lo { return None; }
    let q_lo = div_ceil(n, p_hi);
    let q_hi = n / p_lo;
    let k = 1usize;
    let p_res = 1u64; // p ≡ 1 (mod 2)
    let q_res = derive_q_residue(n, p_res, k as u32);
    let mut f = NestedFrame {
        k,
        p_residue: tape_u64(p_res),
        q_residue: tape_u64(q_res),
        p_lo: tape_u64(p_lo),
        p_hi: tape_u64(p_hi),
        q_lo: tape_u64(q_lo),
        q_hi: tape_u64(q_hi),
        capacity_p: 0,
        capacity_q: 0,
    };
    recompute(&mut f);
    Some(f)
}

/// refine_low — extend the low residue: mod 2^k → mod 2^(k+1). Two children
/// (bit k = 0 or 1); q's residue is re-derived for each, so one wind of p winds
/// q at the same time. Intervals are untouched.
pub fn refine_low(f: &NestedFrame, n: u64) -> Vec<NestedFrame> {
    let k2 = f.k as u32 + 1;
    let pro = tape_to_u64(&f.p_residue);
    let mut out = Vec::with_capacity(2);
    for bit in 0..2u64 {
        let pr = pro | (bit << f.k);
        let qr = derive_q_residue(n, pr, k2);
        let mut c = NestedFrame {
            k: k2 as usize,
            p_residue: tape_u64(pr),
            q_residue: tape_u64(qr),
            p_lo: f.p_lo.clone(),
            p_hi: f.p_hi.clone(),
            q_lo: f.q_lo.clone(),
            q_hi: f.q_hi.clone(),
            capacity_p: 0,
            capacity_q: 0,
        };
        recompute(&mut c);
        out.push(c);
    }
    out
}

/// refine_high — split the p-interval at its midpoint; the q-interval is the
/// image of each half under q = N/p. Residues are untouched.
pub fn refine_high(f: &NestedFrame, n: u64) -> Vec<NestedFrame> {
    let plo = tape_to_u64(&f.p_lo);
    let phi = tape_to_u64(&f.p_hi);
    if plo >= phi { return Vec::new(); }
    let mid = plo + (phi - plo) / 2;
    let mut out = Vec::with_capacity(2);
    for (lo, hi) in [(plo, mid), (mid + 1, phi)] {
        if lo > hi { continue; }
        let qlo = div_ceil(n, hi);
        let qhi = n / lo;
        let mut c = NestedFrame {
            k: f.k,
            p_residue: f.p_residue.clone(),
            q_residue: f.q_residue.clone(),
            p_lo: tape_u64(lo),
            p_hi: tape_u64(hi),
            q_lo: tape_u64(qlo),
            q_hi: tape_u64(qhi),
            capacity_p: 0,
            capacity_q: 0,
        };
        recompute(&mut c);
        out.push(c);
    }
    out
}

/// Metrics for the nesting experiment.
#[derive(Default, Clone)]
pub struct NestStats {
    pub frames_entered: u64,
    pub killed_p: u64,
    pub killed_q: u64,
    pub fixed: u64,
    pub max_live_capacity: u64,
    pub max_depth: usize,
    pub capped: bool,
    pub false_fix: u64,
    pub interior: u64,
    pub parent_cap_sum: u64,
    pub child_sum: u64,
    pub low_sum: u64,
    pub high_sum: u64,
    pub shape_closed: bool,
    pub shape_step: u64,
}

impl NestStats {
}

/// The unique member of r (mod M) in [lo, hi], if any.
fn first_member(residue: u64, modulus: u64, lo: u64, hi: u64) -> Option<u64> {
    if modulus == 0 || lo > hi { return None; }
    let r = residue % modulus;
    let x0 = lo + ((r + modulus - (lo % modulus)) % modulus);
    if x0 > hi { None } else { Some(x0) }
}

/// reenter — the adaptive recursion. Project the children for BOTH refinements,
/// enter the split whose children carry the smaller total live capacity, then
/// descend each child by ascending capacity. capacity 0 → die; capacity 1 → FIX
/// (verified); otherwise re-enter. Results land in `fixes`.
pub fn reenter(f: &NestedFrame, n: u64, depth: usize, budget: u64, st: &mut NestStats, fixes: &mut Vec<(u64, u64)>) {
    if st.frames_entered >= budget { st.capped = true; return; }
    // --- kill / live bookkeeping ---
    st.frames_entered += 1;
    if depth > st.max_depth { st.max_depth = depth; }
    if f.capacity_p == 0 { st.killed_p += 1; return; }
    if f.capacity_q == 0 { st.killed_q += 1; return; }
    let live = f.live_capacity();
    if live > st.max_live_capacity { st.max_live_capacity = live; }

    // --- FIX: p is pinned to a single admissible completion (q = N/p follows) ---
    if f.capacity_p == 1 {
        if let Some(p) = first_member(tape_to_u64(&f.p_residue), 1u64 << f.k, tape_to_u64(&f.p_lo), tape_to_u64(&f.p_hi)) {
            if p > 1 && n % p == 0 {
                st.fixed += 1;
                fixes.push((p, n / p));
            } else {
                st.false_fix += 1; // pinned but not a divisor: this residue branch dies
            }
        }
        return;
    }

    // --- project both refinements ---
    let low = refine_low(f, n);
    let high = refine_high(f, n);
    let low_total: u64 = low.iter().map(|c| c.live_capacity()).sum();
    let high_total: u64 = high.iter().map(|c| c.live_capacity()).sum();
    let chosen: &Vec<NestedFrame> =
        if low.is_empty() { &high }
        else if high.is_empty() { &low }
        else if low_total <= high_total { &low }
        else { &high };

    // --- information gain: sum(child capacities) / parent capacity ----
    st.interior += 1;
    st.parent_cap_sum += live;
    st.child_sum += chosen.iter().map(|c| c.live_capacity()).sum::<u64>();
    st.low_sum += low_total;
    st.high_sum += high_total;

    // --- descend by ascending capacity ---
    let mut order: Vec<usize> = (0..chosen.len()).collect();
    order.sort_by_key(|&i| chosen[i].live_capacity());
    for i in order {
        if st.frames_entered >= budget { st.capped = true; return; }
        reenter(&chosen[i], n, depth + 1, budget, st, fixes);
    }
}

/// Fermat frontier — the SHAPE probe the carrier reads first (scout_factor's gate):
/// a = ceil(sqrt(N)); if a^2 - N is a square within 64 steps then N = (a-b)(a+b).
/// A near-root semiprime closes here at step (p+q)/2 - ceil(sqrt(N)) ~ 0..20.
pub fn fermat_frontier(n: u64) -> Option<(u64, u64, u64)> {
    let mut a = isqrt_u64(n);
    if a * a < n { a += 1; }
    for i in 0..64u64 {
        let a2 = a.checked_mul(a)?;
        if a2 >= n {
            let d = a2 - n;
            let b = isqrt_u64(d);
            if b * b == d && a > b {
                let p = a - b;
                if p > 1 && n % p == 0 { return Some((p, n / p, i)); }
            }
        }
        a += 1;
    }
    None
}

/// Top-level: factor N with the shape probe first (the nest under it), then the
/// capacity-gated recursion. Returns the first distinct factor pair plus metrics.
pub fn factor_nested(n: u64, budget: u64) -> (Option<(u64, u64)>, NestStats) {
    let mut st = NestStats::default();
    if let Some((p, q, i)) = fermat_frontier(n) {
        st.shape_closed = true;
        st.shape_step = i;
        st.fixed = 1;
        return (Some((p, q)), st);
    }
    let mut fixes = Vec::new();
    match root_frame(n) {
        Some(f) => reenter(&f, n, 0, budget, &mut st, &mut fixes),
        None => st.capped = false,
    }
    fixes.sort();
    fixes.dedup();
    (fixes.first().copied(), st)
}
