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
    // relations: (a_tape, parity-bitmask rows as Vec<u64>, full exps)
    let words = width / 64 + 1;
    let mut rows: Vec<Vec<u64>> = Vec::new();
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
            let mut mask = vec![0u64; words];
            for (i, &e) in exps.iter().enumerate() {
                if e & 1 == 1 {
                    mask[i / 64] |= 1u64 << (i % 64);
                }
            }
            rows.push(mask);
            a_of.push(a.clone());
            exp_of.push(exps);
        }
        a = add(&a, &one());
    }
    if a_of.len() < 2 {
        return None;
    }

    // GF(2) elimination tracking which relations combine (a "history" mask over
    // relation indices) to reach an all-even exponent sum.
    let rel = a_of.len();
    let hwords = rel / 64 + 1;
    let mut mat = rows.clone();
    let mut hist: Vec<Vec<u64>> = (0..rel)
        .map(|i| {
            let mut h = vec![0u64; hwords];
            h[i / 64] |= 1u64 << (i % 64);
            h
        })
        .collect();

    let mut pivot_row = vec![usize::MAX; width];
    for r in 0..rel {
        // find a pivot column in row r
        loop {
            let col = (0..width).find(|&c| (mat[r][c / 64] >> (c % 64)) & 1 == 1);
            match col {
                None => break, // all-zero row: r is a dependency
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
        // if row r is now all zero, it is a dependency: combine those relations.
        if mat[r].iter().all(|&w| w == 0) {
            let sel: Vec<usize> = (0..rel).filter(|&i| (hist[r][i / 64] >> (i % 64)) & 1 == 1).collect();
            if sel.is_empty() {
                continue;
            }
            // X = prod a_i mod N ; Y = prod p^(sum e / 2) mod N
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
                let half = total[c] / 2;
                for _ in 0..half {
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

pub fn repl_sieve(n: &Tape) -> String {
    // base bound scales with the width of N; small default, generous candidates.
    let bits = trim(n.clone()).len();
    let bound = 200 + bits * 40;
    match dixon(n, bound, 8, 2_000_000) {
        Some(g) => {
            let q = divmod(n, &g).0;
            format!("{} = {} x {}  [sieve, base<= {}]", dec(n), dec(&g), dec(&q), bound)
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
}
