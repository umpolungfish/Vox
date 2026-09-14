//! sieve.rs — a Dixon / quadratic-sieve core over the folded numeral tapes.
//!
//! The sub-exponential arm for the HARD shape: a factor that is large, far from
//! the root, and not smooth, where trial, rho, the frontier and p±1 all fail.
//! It collects relations a^2 == q (mod N) whose q factors over a small-prime
//! base, finds a subset of relations whose exponents are all even (a linear
//! dependency over GF(2)), and from the resulting X^2 == Y^2 (mod N) takes
//! gcd(X - Y, N). Value-sized arithmetic (a^2 mod N, the gcd) runs on the shared
//! folded kernel; the base primes and the GF(2) matrix are machine words.

use crate::morphism_factor::{add, cmp, divmod, gcd, isqrt, modulo, mul, one, sub, tape_u64, trim, zero};
use crate::vox::EVALF;
use alloc::format;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

type Tape = Vec<char>;

// ---- machine-word number theory for the base and the roots ----

fn mulmod(a: u64, b: u64, m: u64) -> u64 {
    ((a as u128 * b as u128) % m as u128) as u64
}
fn powmod(mut a: u64, mut e: u64, m: u64) -> u64 {
    let mut r = 1u64 % m;
    a %= m;
    while e > 0 {
        if e & 1 == 1 {
            r = mulmod(r, a, m);
        }
        a = mulmod(a, a, m);
        e >>= 1;
    }
    r
}
fn legendre(a: u64, p: u64) -> i64 {
    let r = powmod(a % p, (p - 1) / 2, p);
    if r == 0 {
        0
    } else if r == 1 {
        1
    } else {
        -1
    }
}
/// Tonelli-Shanks: a square root of n mod p (odd prime, n a QR), or None.
fn tonelli(n: u64, p: u64) -> Option<u64> {
    if p == 2 {
        return Some(n % 2);
    }
    if legendre(n, p) != 1 {
        return None;
    }
    if p % 4 == 3 {
        return Some(powmod(n, (p + 1) / 4, p));
    }
    let mut q = p - 1;
    let mut s = 0u32;
    while q % 2 == 0 {
        q /= 2;
        s += 1;
    }
    let mut z = 2u64;
    while legendre(z, p) != -1 {
        z += 1;
    }
    let mut m = s;
    let mut c = powmod(z, q, p);
    let mut t = powmod(n, q, p);
    let mut r = powmod(n, (q + 1) / 2, p);
    while t != 1 {
        let mut i = 0u32;
        let mut t2 = t;
        while t2 != 1 {
            t2 = mulmod(t2, t2, p);
            i += 1;
            if i == m {
                return None;
            }
        }
        let b = powmod(c, 1u64 << (m - i - 1), p);
        m = i;
        c = mulmod(b, b, p);
        t = mulmod(t, c, p);
        r = mulmod(r, b, p);
    }
    Some(r)
}
/// N mod p (p small) by folding the tape modulo through the kernel.
fn n_mod_u64(n: &Tape, p: u64) -> u64 {
    let r = modulo(n, &tape_u64(p));
    let mut v = 0u64;
    for (i, &c) in trim(r).iter().enumerate() {
        if c == EVALF && i < 64 {
            v |= 1u64 << i;
        }
    }
    v
}

/// Small odd primes up to bound b (plus 2), by a byte sieve of Eratosthenes.
fn small_primes(b: usize) -> Vec<u64> {
    let mut is_c = vec![false; b + 1];
    let mut ps = Vec::new();
    let mut i = 2usize;
    while i <= b {
        if !is_c[i] {
            ps.push(i as u64);
            let mut j = i * i;
            while j <= b {
                is_c[j] = true;
                j += i;
            }
        }
        i += 1;
    }
    ps
}

/// Trial-factor the tape v over the base; return the exponent per base prime if
/// v is fully smooth (reduced to 1), else None.
fn smooth_over(v: &Tape, base: &[u64]) -> Option<Vec<u32>> {
    let mut cur = trim(v.clone());
    let mut exps = vec![0u32; base.len()];
    for (i, &p) in base.iter().enumerate() {
        let pt = tape_u64(p);
        loop {
            let (q, r) = divmod(&cur, &pt);
            if zero(&r) {
                exps[i] += 1;
                cur = q;
            } else {
                break;
            }
        }
    }
    if cur == vec![EVALF] {
        Some(exps)
    } else {
        None
    }
}

/// Dixon over the folded tapes. Returns a nontrivial factor of n, or None if no
/// dependency inside the relation/candidate budget yielded one.
pub fn dixon(n: &Tape, base_bound: usize, extra: usize, max_candidates: u64) -> Option<Tape> {
    let base = small_primes(base_bound);
    let width = base.len();
    if width == 0 {
        return None;
    }
    let mut a_of: Vec<Tape> = Vec::new();
    let mut exp_of: Vec<Vec<u32>> = Vec::new();
    let mut a = isqrt(n);
    if cmp(&mul(&a, &a), n) != core::cmp::Ordering::Greater {
        a = add(&a, &one());
    }
    let mut tried = 0u64;
    let need = width + extra;
    while a_of.len() < need && tried < max_candidates {
        tried += 1;
        let q = modulo(&mul(&a, &a), n); // a^2 mod N
        if let Some(exps) = smooth_over(&q, &base) {
            a_of.push(a.clone());
            exp_of.push(exps);
        }
        a = add(&a, &one());
    }
    combine(n, &a_of, &exp_of, &base)
}

/// GF(2) solve over the relation exponent-parity rows plus reconstruct: find
/// dependencies (relation subsets with all-even exponent sum), and for each,
/// X = prod a_i mod N, Y = prod p^(e/2) mod N, then gcd(X - Y, N). Returns the
/// first nontrivial factor. Shared by Dixon and the quadratic sieve.
fn combine(n: &Tape, a_of: &[Tape], exp_of: &[Vec<u32>], base: &[u64]) -> Option<Tape> {
    let width = base.len();
    let rel = a_of.len();
    if rel < 2 || width == 0 {
        return None;
    }
    let words = width / 64 + 1;
    let hwords = rel / 64 + 1;
    let mut mat: Vec<Vec<u64>> = exp_of
        .iter()
        .map(|exps| {
            let mut m = vec![0u64; words];
            for (i, &e) in exps.iter().enumerate() {
                if e & 1 == 1 {
                    m[i / 64] |= 1u64 << (i % 64);
                }
            }
            m
        })
        .collect();
    let mut hist: Vec<Vec<u64>> = (0..rel)
        .map(|i| {
            let mut h = vec![0u64; hwords];
            h[i / 64] |= 1u64 << (i % 64);
            h
        })
        .collect();
    let mut pivot_row = vec![usize::MAX; width];
    for r in 0..rel {
        loop {
            let col = (0..width).find(|&c| (mat[r][c / 64] >> (c % 64)) & 1 == 1);
            match col {
                None => break,
                Some(c) => {
                    if pivot_row[c] == usize::MAX {
                        pivot_row[c] = r;
                        break;
                    } else {
                        let pr = pivot_row[c];
                        for w in 0..words {
                            mat[r][w] ^= mat[pr][w];
                        }
                        for w in 0..hwords {
                            hist[r][w] ^= hist[pr][w];
                        }
                    }
                }
            }
        }
        if mat[r].iter().all(|&w| w == 0) {
            let sel: Vec<usize> = (0..rel).filter(|&i| (hist[r][i / 64] >> (i % 64)) & 1 == 1).collect();
            if sel.is_empty() {
                continue;
            }
            let mut x = one();
            for &i in &sel {
                x = modulo(&mul(&x, &a_of[i]), n);
            }
            let mut total = vec![0u32; width];
            for &i in &sel {
                for c in 0..width {
                    total[c] += exp_of[i][c];
                }
            }
            let mut y = one();
            for c in 0..width {
                for _ in 0..total[c] / 2 {
                    y = modulo(&mul(&y, &tape_u64(base[c])), n);
                }
            }
            let diff = if cmp(&x, &y) != core::cmp::Ordering::Less {
                trim(sub(&x, &y))
            } else {
                trim(sub(&y, &x))
            };
            if zero(&diff) {
                continue;
            }
            let g = gcd(diff, n.clone());
            if cmp(&g, &one()) == core::cmp::Ordering::Greater && cmp(&g, n) == core::cmp::Ordering::Less {
                return Some(trim(g));
            }
        }
    }
    None
}

/// Quadratic sieve. The factor base is only the primes where N is a quadratic
/// residue (the only ones that can divide a^2 - N), each with its two roots by
/// Tonelli-Shanks. A log-sieve over a window above sqrt(N) marks where each
/// prime divides, so only positions whose log-sum approaches log2(a^2 - N) are
/// trial-factored exactly. Then the shared GF(2) combine closes it.
pub fn qs(n: &Tape, b_bound: usize, m_interval: usize, extra: usize) -> Option<Tape> {
    let primes = small_primes(b_bound);
    let mut base: Vec<u64> = Vec::new();
    let mut roots: Vec<(u64, u64)> = Vec::new();
    for &p in &primes {
        let np = n_mod_u64(n, p);
        if np == 0 {
            return Some(tape_u64(p)); // p actually divides N
        }
        if p == 2 {
            base.push(2);
            roots.push((1, 1));
        } else if legendre(np, p) == 1 {
            if let Some(r) = tonelli(np, p) {
                base.push(p);
                roots.push((r, p - r));
            }
        }
    }
    let width = base.len();
    if width == 0 {
        return None;
    }
    let mut root = isqrt(n);
    if cmp(&mul(&root, &root), n) == core::cmp::Ordering::Less {
        root = add(&root, &one());
    }
    // Integer log-sieve (bit-length weights) over the window [root, root + M),
    // no_std having no float log. Each prime contributes floor(log2 p).
    let flog2 = |x: u64| -> u32 { if x < 2 { 0 } else { 63 - x.leading_zeros() } };
    let mut logs = vec![0u32; m_interval];
    for (k, &p) in base.iter().enumerate() {
        let lp = flog2(p);
        let rootmod = n_mod_u64(&root, p) % p;
        let (r1, r2) = roots[k];
        for &r in &[r1, r2] {
            let start = ((r + p - rootmod % p) % p) as usize;
            let mut i = start;
            while i < m_interval {
                logs[i] += lp;
                i += p as usize;
            }
            if p == 2 {
                break;
            }
        }
    }
    let bits_n = trim(n.clone()).len() as u32;
    let slack = 2 * (flog2(b_bound as u64) + 1) + 4;
    let mut a_of: Vec<Tape> = Vec::new();
    let mut exp_of: Vec<Vec<u32>> = Vec::new();
    let need = width + extra;
    for i in 0..m_interval {
        if a_of.len() >= need {
            break;
        }
        // log2(a^2 - N) ~ bits_n/2 + log2(i+1) + 1; accept within slack of it.
        let target = bits_n / 2 + flog2(i as u64 + 1) + 1;
        if logs[i] + slack < target {
            continue;
        }
        let a = add(&root, &tape_u64(i as u64));
        let v = trim(sub(&mul(&a, &a), n)); // a^2 - N >= 0 for a >= ceil(sqrt N)
        if let Some(exps) = smooth_over(&v, &base) {
            a_of.push(a);
            exp_of.push(exps);
        }
    }
    combine(n, &a_of, &exp_of, &base)
}

/// Base bound and window sized from the width of N: B grows about like the
/// square of the digit count, the window a few hundred thousand.
pub fn sieve_params(n: &Tape) -> (usize, usize) {
    let bits = trim(n.clone()).len();
    let bound = ((bits * bits) / 6 + 200).min(20_000);
    let m = 600_000usize;
    (bound, m)
}

pub fn sieve_factor(n: &Tape) -> Option<Tape> {
    let (bound, m) = sieve_params(n);
    qs(n, bound, m, 16).or_else(|| dixon(n, bound, 8, 2_000_000))
}

pub fn repl_sieve(n: &Tape) -> String {
    let (bound, _m) = sieve_params(n);
    match sieve_factor(n) {
        Some(g) => {
            let q = divmod(n, &g).0;
            format!("{} = {} x {}  [quadratic sieve, base<= {}]", dec(n), dec(&g), dec(&q), bound)
        }
        None => format!("{}  [sieve found no dependency within budget]", dec(n)),
    }
}

fn dec(t: &[char]) -> String {
    let mut b = trim(t.to_vec());
    if zero(&b) {
        return "0".into();
    }
    let ten = tape_u64(10);
    let mut ds = Vec::new();
    while !zero(&b) {
        let (q, r) = divmod(&b, &ten);
        let mut dv = 0u8;
        for (i, &c) in trim(r).iter().enumerate() {
            if c == EVALF {
                dv |= 1 << i;
            }
        }
        ds.push(b'0' + dv);
        b = q;
    }
    ds.reverse();
    String::from_utf8(ds).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vox::EVALT;
    fn tape(mut n: u64) -> Tape {
        if n == 0 {
            return vec![EVALT];
        }
        let mut t = Vec::new();
        while n != 0 {
            t.push(if n & 1 == 1 { EVALF } else { EVALT });
            n >>= 1;
        }
        t
    }
    fn val(t: &Tape) -> u64 {
        let mut a = 0u64;
        for &c in trim(t.clone()).iter().rev() {
            a = (a << 1) | if c == EVALF { 1 } else { 0 };
        }
        a
    }
    #[test]
    fn dixon_factors() {
        for &n in &[8051u64, 100160063, 16843009, 2027651281] {
            let g = dixon(&tape(n), 500, 8, 2_000_000).expect("no factor");
            let gv = val(&g);
            assert!(gv > 1 && gv < n && n % gv == 0, "dixon({n}) = {gv}");
        }
    }

    #[test]
    fn qs_factors_through_the_full_pipeline() {
        // QR base + Tonelli roots + log-sieve + GF(2) solve, end to end.
        for &n in &[8051u64, 100160063, 2027651281, 191873633311] {
            let g = qs(&tape(n), 500, 200_000, 12).expect("qs no factor");
            let gv = val(&g);
            assert!(gv > 1 && gv < n && n % gv == 0, "qs({n}) = {gv}");
        }
    }
}
