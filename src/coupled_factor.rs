//! coupled_factor.rs — the coupled factorizer.
//!
//! The P-membrane phase arm and the frame-shift unbraid, run together on one
//! odd N so that each arm certifies the other:
//!
//! membrane arm (phase-based, base g): wind the dyadic orbit x_{k+1} = x_k^2
//!   (mod N) from a coprime phase base; the first collision of the walk — or
//!   a power-of-two close x_k = 1 — reads off an exponent r with g^r = 1 mod
//!   N; the half-step braid gcd(g^{r/2} ± 1, N) splits N. Domain: gcd(g, N)=1.
//!
//! unbraid arm (structural, word-native, on N's own bits):
//!   Hensel arm — bottom-up, μ-fuse-inward: FoldMembrane DFS on the 2-adic
//!     parity equation; one binary branch choice per width; product ≤ N
//!     pruning; the live-state count per width is reported.
//!   interval arm — top-down, δ-split-outward: the exact interval
//!     [P·2^m, (P+1)·2^m − 1] × [Q·2^m, (Q+1)·2^m − 1] contains N or does not;
//!     the surviving top prefixes are the frame positions. ℓ* is the first
//!     level of the terminal run of unique survivors: the ROTAT cut that
//!     deinterlaces to (p, q).
//!
//! self-certification (the Frobenius condition μ∘δ = id):
//!   multiply_via_word:  D(p) × D(q) == D(N)
//!   Γ/Λ round-trip:     Λ(Γ(D(p), D(q))) == (D(p), D(q))
//!   encode(μ(Λ(D))) ==  D(N)
//! plus arm agreement: the membrane's {p, q} == the unbraid's {p, q}, and the
//! interval arm's pinned top prefix matches the Hensel lift.

use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use core::cmp::Ordering;

use crate::factor_2adic::FoldMembrane;
use crate::morphism_factor::{
    add, cmp, dec_of, divmod, emit_numeral, gcd, miller_rabin, mul, mul_mod, modulo, one, sub,
    tape_u64, trim, zero,
};

const BIT0: char = '⊤';
const BIT1: char = '⊥';

type Tape = Vec<char>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reject {
    Even,
    TooSmall,
}

fn bit(t: &[char], i: usize) -> u8 {
    if i < t.len() && t[i] == BIT1 { 1 } else { 0 }
}

/// Left shift by m bits (value x 2^m; LSB-first tapes: prepend m low zeros).
fn shl(t: &[char], m: usize) -> Tape {
    let mut v = vec![BIT0; m];
    v.extend_from_slice(t);
    v
}

/// Right shift by m bits (drop m low bits).
fn rshl(t: &[char], m: usize) -> Tape {
    trim(if t.len() <= m { vec![BIT0] } else { t[m..].to_vec() })
}

fn ones(m: usize) -> Tape {
    (0..m).map(|_| BIT1).collect()
}

fn is_one(t: &[char]) -> bool {
    t.len() == 1 && t[0] == BIT1
}

fn is_two_pow(t: &[char]) -> bool {
    !zero(t) && t.iter().filter(|&&c| c == BIT1).count() == 1
}

// ── Γ / Λ: the interlace and its inverse, on the tapes ──────────────────────

fn gamma(p: &[char], q: &[char]) -> Tape {
    let m = p.len().max(q.len());
    let mut d = Vec::with_capacity(2 * m);
    for i in 0..m {
        d.push(if bit(p, i) == 1 { BIT1 } else { BIT0 });
        d.push(if bit(q, i) == 1 { BIT1 } else { BIT0 });
    }
    trim(d)
}

fn lambda(d: &[char]) -> (Tape, Tape) {
    let d = trim(d.to_vec());
    let m = (d.len() + 1) / 2;
    let mut p = Vec::with_capacity(m);
    let mut q = Vec::with_capacity(m);
    for i in 0..m {
        p.push(if bit(&d, 2 * i) == 1 { BIT1 } else { BIT0 });
        q.push(if bit(&d, 2 * i + 1) == 1 { BIT1 } else { BIT0 });
    }
    (trim(p), trim(q))
}
// ── the dyadic orbit: first-collision table and close ───────────────────────

struct Seen {
    slots: Vec<Option<(u64, usize, usize)>>,
    vals: Vec<(Tape, usize)>,
}

impl Seen {
    fn new(bits: usize) -> Self {
        let cap = 1usize << bits;
        Seen {
            slots: vec![None; cap],
            vals: Vec::new(),
        }
    }

    fn fp(t: &[char]) -> (u64, usize) {
        let mut v = 0u64;
        for i in 0..t.len().min(64) {
            if t[i] == BIT1 {
                v |= 1u64 << i;
            }
        }
        (v, t.len())
    }

    fn idx(&self, f: (u64, usize)) -> usize {
        let h = f.0 ^ (f.1 as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
        h as usize & (self.slots.len() - 1)
    }

    fn find(&self, t: &[char]) -> Option<usize> {
        let f = Self::fp(t);
        let mut i = self.idx(f);
        loop {
            match &self.slots[i] {
                None => return None,
                Some((fv, fl, vi)) if *fv == f.0 && *fl == f.1 => {
                    if self.vals[*vi].0 == t {
                        return Some(self.vals[*vi].1);
                    }
                }
                _ => {}
            }
            i = (i + 1) & (self.slots.len() - 1);
        }
    }

    fn insert(&mut self, t: &[char], step: usize) {
        let f = Self::fp(t);
        let mut i = self.idx(f);
        loop {
            if self.slots[i].is_none() {
                let vi = self.vals.len();
                self.vals.push((t.to_vec(), step));
                self.slots[i] = Some((f.0, f.1, vi));
                return;
            }
            i = (i + 1) & (self.slots.len() - 1);
        }
    }
}

/// Wind x_{k+1} = x_k^2 (mod N) from x_0 = g. Returns (r, collision, k):
/// r = 2^k for a power-of-two close x_k = 1, else r = 2^i (2^{k−i} − 1) for
/// the first repeat x_k = x_i. In both cases g^r ≡ 1 (mod N).
fn dyadic_close(g: &[char], n: &[char], budget: usize) -> Option<(Tape, Option<(usize, usize)>, usize)> {
    let mut seen = Seen::new(20);
    let mut x = g.to_vec();
    let mut k = 0usize;
    loop {
        if is_one(&x) {
            let mut r = one();
            for _ in 0..k {
                r = add(&r, &r);
            }
            return Some((r, None, k));
        }
        if let Some(i) = seen.find(&x) {
            let m = k - i;
            let mut r = ones(m); // 2^m − 1
            for _ in 0..i {
                r = add(&r, &r);
            }
            return Some((r, Some((i, k)), k));
        }
        seen.insert(&x, k);
        k += 1;
        if k > budget {
            return None;
        }
        x = mul_mod(&x, &x, n);
    }
}

/// Square-and-multiply on the tapes: base^exp mod n, exp read LSB-first.
fn pow_tape(base: &[char], exp: &[char], n: &[char]) -> Tape {
    let mut r = one();
    let mut b = base.to_vec();
    for i in 0..exp.len() {
        if bit(exp, i) == 1 {
            r = mul_mod(&r, &b, n);
        }
        if i + 1 < exp.len() {
            b = mul_mod(&b, &b, n);
        }
    }
    r
}

/// The half-step braid. r is a multiple of ord_N(g) with r even; halve r while
/// g^{r/2} ≡ 1; then a = g^{r/2} ∉ {1, −1} and gcd(a ± 1, N) splits N.
fn half_step_braid(g: &[char], r_in: &[char], n: &[char]) -> Option<(Tape, Tape, Tape)> {
    if bit(r_in, 0) == 1 {
        return None; // odd r: no ±1 structure, base is degenerate
    }
    let mut r = r_in.to_vec();
    let mut a = pow_tape(g, &rshl(&r, 1), n);
    while is_one(&a) {
        let r2 = rshl(&r, 1);
        if is_one(&r2) {
            break;
        }
        r = r2;
        a = pow_tape(g, &rshl(&r, 1), n);
    }
    if is_one(&a) || a == sub(n, &one()) {
        return None; // a ∈ {1, −1} all the way down: base fails
    }
    let am1 = sub(&a, &one());
    let p1 = gcd(am1, n.to_vec());
    if !is_one(&p1) && cmp(&p1, n) != Ordering::Equal {
        let q1 = divmod(n, &p1).0;
        if !zero(&q1) && !is_one(&q1) {
            return Some(if cmp(&p1, &q1) == Ordering::Less { (p1, q1, a) } else { (q1, p1, a) });
        }
    }
    let ap1 = add(&a, &one());
    let p2 = gcd(ap1, n.to_vec());
    if !is_one(&p2) && cmp(&p2, n) != Ordering::Equal {
        let q2 = divmod(n, &p2).0;
        if !zero(&q2) && !is_one(&q2) {
            return Some(if cmp(&p2, &q2) == Ordering::Less { (p2, q2, a) } else { (q2, p2, a) });
        }
    }
    None
}
// ── membrane arm: the phase-based close ─────────────────────────────────────

pub struct MembraneOutcome {
    pub base: u64,
    pub bases_tried: u32,
    pub close: &'static str,
    pub r: Tape,
    pub collision: Option<(usize, usize)>,
    pub walk_len: usize,
    pub half: Tape,
    pub p: Tape,
    pub q: Tape,
}

const BASES: [u64; 16] = [2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53];

pub fn membrane_arm(n: &[char], max_bases: usize, walk_budget: usize) -> Option<MembraneOutcome> {
    let n = trim(n.to_vec());
    let mut tried = 0u32;
    for base in BASES.iter().copied().take(max_bases) {
        tried += 1;
        let g0 = tape_u64(base);
        let d = gcd(g0, n.clone());
        if !is_one(&d) && cmp(&d, &n) != Ordering::Equal {
            let q = divmod(&n, &d).0;
            if !zero(&q) && !is_one(&q) {
                return Some(MembraneOutcome {
                    base,
                    bases_tried: tried,
                    close: "gcd-hit",
                    r: one(),
                    collision: None,
                    walk_len: 0,
                    half: one(),
                    p: d,
                    q,
                });
            }
        }
        let g = modulo(&tape_u64(base), &n);
        if zero(&g) || is_one(&g) {
            continue; // not a unit mod N: base skipped
        }
        if let Some((r, collision, walk_len)) = dyadic_close(&g, &n, walk_budget) {
            if let Some((p, q, half)) = half_step_braid(&g, &r, &n) {
                return Some(MembraneOutcome {
                    base,
                    bases_tried: tried,
                    close: "orbit",
                    r,
                    collision,
                    walk_len,
                    half,
                    p,
                    q,
                });
            }
        }
    }
    None
}

// ── unbraid arm, Hensel side: bottom-up, μ-fuse-inward ──────────────────────

pub struct HenselOutcome {
    pub found: Option<(Tape, Tape)>,
    pub found_width: Option<usize>,
    pub states_expanded: usize,
    pub live_per_width: Vec<(usize, usize)>,
    pub budget_exhausted: bool,
}

pub fn hensel_arm(n_in: &[char], budget: usize) -> HenselOutcome {
    let n = trim(n_in.to_vec());
    let mut out = HenselOutcome {
        found: None,
        found_width: None,
        states_expanded: 0,
        live_per_width: Vec::new(),
        budget_exhausted: false,
    };
    let seed = match FoldMembrane::new(n.clone(), 2) {
        Ok(s) => s,
        Err(_) => return out,
    };
    let mut live: Vec<usize> = Vec::new();
    let mut stack = vec![seed];
    while let Some(st) = stack.pop() {
        out.states_expanded += 1;
        if out.states_expanded > budget {
            out.budget_exhausted = true;
            break;
        }
        live.push(st.width);
        if st.is_fixed_point() {
            let a = st.factors[0].clone();
            let b = st.factors[1].clone();
            if a.len() > 1 && b.len() > 1 {
                let (lo, hi) = if cmp(&a, &b) == Ordering::Less { (a, b) } else { (b, a) };
                out.found = Some((lo, hi));
                out.found_width = Some(st.width);
                break;
            }
            continue; // trivial (1, N) fixed point: no children at full width
        }
        if st.width >= n.len() {
            continue;
        }
        for pair in st.admissible_next_pairs() {
            if let Ok(next) = st.extend(&[pair.0, pair.1]) {
                stack.push(next);
            }
        }
    }
    let mut lpw: Vec<(usize, usize)> = Vec::new();
    for w in 1..=n.len() {
        let c = live.iter().filter(|&&x| x == w).count();
        if c > 0 {
            lpw.push((w, c));
        }
    }
    out.live_per_width = lpw;
    out
}
// ── unbraid arm, interval side: top-down, δ-split-outward ───────────────────

pub struct IntervalOutcome {
    pub pin_level: Option<usize>,
    pub pinned: Option<(Tape, Tape)>,
    pub exact: Option<(Tape, Tape)>,
    pub survivors_per_level: Vec<(usize, usize)>,
    pub budget_exhausted: bool,
}

pub fn interval_arm(n_in: &[char], budget: usize) -> IntervalOutcome {
    let n = trim(n_in.to_vec());
    let w = n.len();
    let mut out = IntervalOutcome {
        pin_level: None,
        pinned: None,
        exact: None,
        survivors_per_level: Vec::new(),
        budget_exhausted: false,
    };
    let mut cur: Vec<(Tape, Tape)> = vec![(Vec::new(), Vec::new())];
    let mut history: Vec<(usize, Option<(Tape, Tape)>)> = vec![(0, Some((Vec::new(), Vec::new())))];
    out.survivors_per_level.push((0, 1));
    for k in 1..=w {
        let m = w - k;
        let mask = if m > 0 { ones(m) } else { Vec::new() };
        let mut next: Vec<(Tape, Tape)> = Vec::new();
        'outer: for (pfx, qfx) in &cur {
            for pb in [0u8, 1u8] {
                for qb in [0u8, 1u8] {
                    let np: Tape = if pb == 1 { add(&shl(pfx, 1), &one()) } else { shl(pfx, 1) };
                    let nq: Tape = if qb == 1 { add(&shl(qfx, 1), &one()) } else { shl(qfx, 1) };
                    let plo = shl(&np, m);
                    let phi = if m > 0 { add(&plo, &mask) } else { plo.clone() };
                    let qlo = shl(&nq, m);
                    let qhi = if m > 0 { add(&qlo, &mask) } else { qlo.clone() };
                    let lo = mul(&plo, &qlo);
                    if cmp(&lo, &n) == Ordering::Greater {
                        continue;
                    }
                    let hi = mul(&phi, &qhi);
                    if cmp(&n, &hi) == Ordering::Greater {
                        continue;
                    }
                    next.push((trim(np), trim(nq)));
                    if next.len() > budget {
                        out.budget_exhausted = true;
                        break 'outer;
                    }
                }
            }
        }
        if out.budget_exhausted {
            break;
        }
        let count = next.len();
        out.survivors_per_level.push((k, count));
        let pair = if count == 1 {
            Some((next[0].0.clone(), next[0].1.clone()))
        } else {
            None
        };
        history.push((k, pair));
        cur = next;
        if cur.is_empty() {
            break;
        }
    }
    if !out.budget_exhausted {
        let last = history.len() - 1;
        if last == w {
            let mut k = w;
            while k >= 1 && history[k].1.is_some() {
                k -= 1;
            }
            let pin = k + 1;
            if let Some((_, Some(p))) = history.iter().find(|(l, _)| *l == pin) {
                out.pin_level = Some(pin);
                out.pinned = Some(p.clone());
            }
            if let Some((_, Some(p))) = history.iter().find(|(l, _)| *l == w) {
                out.exact = Some(p.clone());
            }
        }
    }
    out
}

/// Do the top k bits of `factor` equal the prefix value `prefix`?
fn top_match(factor: &[char], prefix: &[char], k: usize) -> bool {
    let l = factor.len();
    let v: Tape = if l >= k { factor[l - k..].to_vec() } else { factor.to_vec() };
    cmp(&trim(v), &trim(prefix.to_vec())) == Ordering::Equal
}
// ── the coupled factorizer ──────────────────────────────────────────────────

fn join_counts(counts: &[(usize, usize)]) -> String {
    let mut s = String::new();
    for (i, (w, c)) in counts.iter().enumerate() {
        if i > 0 {
            s.push_str(" ");
        }
        s.push_str(&format!("{}:{}", w, c));
    }
    s
}

pub fn coupled(n_in: &[char]) -> Result<String, Reject> {
    let n = trim(n_in.to_vec());
    if n.len() <= 1 || zero(&n) {
        return Err(Reject::TooSmall);
    }
    let w = n.len();
    if bit(&n, 0) == 0 {
        return Err(Reject::Even);
    }

    let mersenne = is_two_pow(&add(&n, &one()));
    let prime = miller_rabin(&n);

    let mut out = String::new();
    out.push_str(&format!("N = {}  (tape width {})\n", dec_of(&n), w));
    out.push_str(&format!("D(N) = {}\n", emit_numeral(&n)));
    out.push_str(&format!(
        "domain: odd{}{}\n",
        if mersenne {
            ", Mersenne (bit runs constant — leadrun-style frame decode is non-injective here; the interval arm stays exact)"
        } else {
            ""
        },
        if prime { ", prime" } else { "" },
    ));
    if prime {
        out.push_str("verdict: N is prime — the factorizer has nothing to close\n");
        return Ok(out);
    }

    let mem = membrane_arm(&n, 16, 262_144);
    let hensel = hensel_arm(&n, 200_000);
    let interval = interval_arm(&n, 200_000);

    // ── membrane arm report ──
    match &mem {
        Some(m) => {
            out.push_str(&format!("\nmembrane arm (phase base g = {}):\n", m.base));
            if m.close == "gcd-hit" {
                out.push_str("  gcd(g, N) is already a factor — lucky hit, no orbit needed\n");
            } else {
                out.push_str(&format!(
                    "  orbit x_{{k+1}} = x_k² mod N: walk {} steps, {}\n",
                    m.walk_len,
                    match m.collision {
                        Some((i, j)) => format!("first collision (i = {}, j = {})", i, j),
                        None => format!("power-of-two close (x_{} = 1)", m.walk_len),
                    }
                ));
                out.push_str(&format!("  r = {}  (g^r ≡ 1 mod N, r a multiple of ord_N(g))\n", dec_of(&m.r)));
                out.push_str(&format!("  half-step braid: a = g^{{r/2}} mod N = {}\n", dec_of(&m.half)));
                out.push_str(&format!("  → p = {}, q = {}\n", dec_of(&m.p), dec_of(&m.q)));
            }
        }
        None => out.push_str(
            "\nmembrane arm: no base in the scan closed the half-step braid (orbit budget or base scan exhausted)\n",
        ),
    }

    // ── unbraid arm report ──
    out.push_str("\nunbraid arm (word-native, on N's own bits):\n");
    out.push_str(&format!(
        "  Hensel (μ-fuse-inward): {} states expanded{}\n",
        hensel.states_expanded,
        if hensel.budget_exhausted { " (budget exhausted)" } else { "" }
    ));
    match &hensel.found {
        Some((p, q)) => out.push_str(&format!(
            "    accepted at width {} → {{p, q}} = {{{}, {}}}\n",
            hensel.found_width.unwrap_or(0),
            dec_of(p),
            dec_of(q)
        )),
        None => out.push_str("    no fixed point within budget\n"),
    }
    out.push_str(&format!("    live/width: {}\n", join_counts(&hensel.live_per_width)));
    match (interval.pin_level, &interval.pinned) {
        (Some(l), Some((pt, qt))) => out.push_str(&format!(
            "  interval (δ-split-outward): ℓ* = {} — pinned top prefix (p_top, q_top) = ({}, {})\n",
            l,
            dec_of(pt),
            dec_of(qt)
        )),
        _ => out.push_str(&format!(
            "  interval (δ-split-outward): no pin within budget{}\n",
            if interval.budget_exhausted { " (budget exhausted)" } else { "" }
        )),
    }
    out.push_str(&format!(
        "    survivors/level: {}\n",
        join_counts(&interval.survivors_per_level)
    ));

    // ── the factor pair: membrane's if it closed, else the unbraid's ──
    let factors: Option<(Tape, Tape)> = match &mem {
        Some(m) => Some((m.p.clone(), m.q.clone())),
        None => match &hensel.found {
            Some((p, q)) => Some((p.clone(), q.clone())),
            None => interval.exact.clone(),
        },
    };

    // ── cross-checks ──
    let arms_agree: Option<bool> = match (&mem, &hensel.found) {
        (Some(m), Some((hp, hq))) => {
            Some((m.p == *hp && m.q == *hq) || (m.p == *hq && m.q == *hp))
        }
        _ => None,
    };
    if arms_agree.is_some() {
        out.push_str("\ncross-check:\n");
        if let (Some(m), Some((hp, hq))) = (&mem, &hensel.found) {
            out.push_str(&format!(
                "  membrane {{{}, {}}} == unbraid {{{}, {}}}: {}\n",
                dec_of(&m.p),
                dec_of(&m.q),
                dec_of(hp),
                dec_of(hq),
                if arms_agree.unwrap() { "PASS" } else { "FAIL" }
            ));
        }
    }
    let seed_ok: Option<bool> =
        match (interval.pin_level, &interval.pinned, &hensel.found) {
            (Some(l), Some((pt, qt)), Some((hp, hq))) => {
                Some(top_match(hp, pt, l) && top_match(hq, qt, l))
            }
            _ => None,
        };
    if seed_ok.is_some() {
        if !arms_agree.is_some() {
            out.push_str("\ncross-check:\n");
        }
        out.push_str(&format!(
            "  interval seed ⊆ Hensel lift (top {} bits): {}\n",
            interval.pin_level.unwrap(),
            if seed_ok.unwrap() { "PASS" } else { "FAIL" }
        ));
    }

    // ── Frobenius certification ──
    match &factors {
        Some((p, q)) => {
            let prod = mul(p, q);
            let pv = cmp(&prod, &n) == Ordering::Equal;
            let d = gamma(p, q);
            let (lp, lq) = lambda(&d);
            let rt = cmp(&lp, p) == Ordering::Equal && cmp(&lq, q) == Ordering::Equal;
            let mu = mul(&lp, &lq);
            let syz = cmp(&mu, &n) == Ordering::Equal;
            let semiprime = miller_rabin(p) && miller_rabin(q);
            let verdict_ok = pv && rt && syz
                && arms_agree.map_or(true, |b| b)
                && seed_ok.map_or(true, |b| b);
            out.push_str("\nFrobenius (μ∘δ = id):\n");
            out.push_str(&format!("  multiply_via_word   D(p) × D(q) == D(N): {}\n", if pv { "true" } else { "false" }));
            out.push_str(&format!("  Γ/Λ round-trip      Λ(Γ(D(p), D(q))) == (D(p), D(q)): {}\n", if rt { "true" } else { "false" }));
            out.push_str(&format!("  encode(μ(Λ(D))) == D(N): {}\n", if syz { "true" } else { "false" }));
            out.push_str(&format!(
                "\nverdict: {}   p = {}  q = {}{}\n",
                if verdict_ok { "PASS" } else { "FAIL" },
                dec_of(p),
                dec_of(q),
                if semiprime {
                    "  (both factors prime — semiprime confirmed)"
                } else {
                    "  (composite factors)"
                }
            ));
        }
        None => out.push_str("\nverdict: FAIL — no arm produced a factor pair\n"),
    }
    Ok(out)
}
