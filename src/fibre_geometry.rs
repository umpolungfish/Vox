//! Fibre geometry axis of Core Numeral.
//!
//! Counting theorems for the native suffix-envelope quotient and the powerset
//! tower. This module is no_std-compatible (pure counting, no allocations) and
//! exposes exact closed forms / cheap inclusion-exclusion for the four §7.7
//! acceptance quantities:
//!
//!   * `n_kr(k, r)`              — covering families of exact size r,
//!   * `fibre_size(k)`           — F(k) = sum_r n_kr = total covering families,
//!   * `minimal_cover_count(k)`  — A046165, minimal covering families,
//!   * `hasse_edge_count(k)`     — covering relations of the cover-family poset,
//!   * `stirling_bridge_holds`   — the Stage-58/200 bridge identity.
//!
//! Every value here is a theorem, not a measurement: the counts are computed by
//! inclusion-exclusion and cross-checked in `tests/fibre_geometry.rs` against the
//! exact fibre enumeration in `tests/suffix_envelope_fibre.rs`.

use alloc::vec::Vec;

/// Number of subsets of a `k`-set, `2^k`.
#[inline]
fn nsub(k: u32) -> u128 {
    1u128 << k
}

/// Binomial coefficient `C(n, r)` for small `n` (exact, no overflow for k <= 20).
pub fn binom(n: u32, r: u32) -> u128 {
    if r > n {
        return 0;
    }
    let r = r.min(n - r);
    let mut out: u128 = 1;
    for i in 0..r {
        out = out * (n as u128 - i as u128) / (i as u128 + 1);
    }
    out
}

/// `n_kr(k, r)` — the number of `r`-element families `F ⊆ P([k])` whose union
/// is the whole `k`-set. Inclusion-exclusion over the `k` missing atoms:
///
///   N(k,r) = sum_{j=0}^{k} (-1)^j C(k,j) C(2^{k-j}, r).
///
/// The `r = 0` term is the empty family, which covers `[k]` only when `k = 0`
/// (then `N(0,0) = 1`).
pub fn n_kr(k: u32, r: u128) -> i128 {
    let mut s: i128 = 0;
    for j in 0..=k {
        let term: i128 = binom(k, j) as i128 * binom(nsub(k - j) as u32, r as u32) as i128;
        if j % 2 == 0 {
            s += term;
        } else {
            s -= term;
        }
    }
    s
}

/// Fibre size `F(k)` — the number of (any-size) covering families of a `k`-set:
///
///   F(k) = sum_{r} N(k,r) = sum_{j=0}^{k} (-1)^j C(k,j) 2^{2^{k-j}}.
///
/// Returns `2, 2, 10, 218, 64594` for `k = 0..4`.
pub fn fibre_size(k: u32) -> u128 {
    let mut s: i128 = 0;
    for j in 0..=k {
        let inner: u128 = 1u128 << (nsub(k - j) as u32); // 2^{2^{k-j}}
        let term: i128 = binom(k, j) as i128 * inner as i128;
        if j % 2 == 0 {
            s += term;
        } else {
            s -= term;
        }
    }
    s as u128
}

/// `M(k)` — the number of *minimal* covering families of a `k`-set (OEIS
/// A046165): a cover from which no member can be deleted, equivalently a cover
/// in which every member has a private element.
///
///   M(k) = sum_{r=1}^{k} (1/r!) sum_{j=0}^{r} (-1)^j C(r,j) (2^r - 1 - j)^k,
///
/// with the convention `M(0) = 1`. The exclusion term is `2^r - 1 - j` (the
/// number of nonempty sub-families of an `r`-set that avoid a fixed `j`-set of
/// members). Returns `1, 1, 2, 8, 49` for `k = 0..4`.
pub fn minimal_cover_count(k: u32) -> u128 {
    if k == 0 {
        return 1;
    }
    let mut total: u128 = 0;
    for r in 1..=k {
        // T(k,r) = sum_j (-1)^j C(r,j) (2^r - 1 - j)^k  (ordered r-tuples)
        let mut t: i128 = 0;
        for j in 0..=r {
            let base: i128 = (1i128 << r) - 1 - j as i128; // 2^r - 1 - j
            let pk = ipow(base, k as u32);
            let term: i128 = binom(r, j) as i128 * pk;
            if j % 2 == 0 {
                t += term;
            } else {
                t -= term;
            }
        }
        let mkr = t / factorial(r) as i128; // unordered: divide by r!
        total += mkr as u128;
    }
    total
}

/// `hasse_edge_count(k)` — the number of covering relations in the poset
///
///   C_k = { F ⊆ P([k]) : union F = [k] }
///
/// ordered by inclusion. Family cardinality is a rank function and any superset
/// of a covering family is still covering, so `F < G` is a cover iff
/// `G = F ∪ {S}` for a single new set `S`. Hence
///
///   #Hasse(k) = sum_{F in C_k} (2^k - |F|) = sum_{r} N(k,r) (2^k - r),
///
/// where the `r = 0` term (the empty family) is present only for `k = 0`,
/// supplying the single edge that turns the naive count `0` into `1`.
/// Returns `1, 1, 15, 805, 513135` for `k = 0..4`.
pub fn hasse_edge_count(k: u32) -> u128 {
    let cap = nsub(k); // 2^k
    let mut total: i128 = 0;
    for r in 0..=cap {
        let nkr = n_kr(k, r);
        total += nkr * (cap - r) as i128;
    }
    total as u128
}

/// The Stage-58/Stage-200 bridge:
///
///   (2^d - 1)^k = sum_{r} N(k,r) r! S(d, r),
///
/// where `S(d, r)` is a Stirling number of the second kind. Returns `true` when
/// the identity holds exactly for the given `(k, d)`.
pub fn stirling_bridge_holds(k: u32, d: usize) -> bool {
    let lhs: u128 = ((1u128 << d as u32) - 1).pow(k as u32);
    let mut rhs: u128 = 0;
    for r in 0..=nsub(k) {
        let nkr = n_kr(k, r);
        if nkr == 0 {
            continue;
        }
        let term = nkr as u128 * factorial(r as u32) * stirling2(d, r as usize);
        rhs += term;
    }
    lhs == rhs
}

/// Stirling number of the second kind `S(n, k)`.
pub fn stirling2(n: usize, k: usize) -> u128 {
    if k == 0 {
        return if n == 0 { 1 } else { 0 };
    }
    if k > n {
        return 0;
    }
    let mut row: Vec<u128> = vec![0; k + 1];
    row[0] = 1;
    for i in 1..=n {
        for j in (1..=i.min(k)).rev() {
            row[j] = j as u128 * row[j] + row[j - 1];
        }
        row[0] = 0;
    }
    row[k]
}

/// `n!` for small `n` (exact for n <= 20).
pub fn factorial(n: u32) -> u128 {
    let mut p: u128 = 1;
    for i in 2..=n {
        p *= i as u128;
    }
    p
}

/// Exact integer power for a possibly-negative base and small exponent.
fn ipow(base: i128, mut e: u32) -> i128 {
    let mut out: i128 = 1;
    let mut b = base;
    while e > 0 {
        if e & 1 == 1 {
            out *= b;
        }
        b *= b;
        e >>= 1;
    }
    out
}
