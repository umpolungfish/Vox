//! Exact fibre counts for the native suffix-envelope quotient and the powerset tower.
//!
//! The restored-support ladder now comes from `vox::provenance_envelope`; the local
//! `model_envelope` remains only as an independent theorem oracle. Every enumerated
//! deposit sequence therefore crosses the same no_std restoration seam used by the
//! syzygy tests.
//!
//! Run:  cargo test --release --test suffix_envelope_fibre -- --nocapture
//!
//! What is asserted (all exact, no sampling):
//!   1. Native restored-support ladder equals the suffix-envelope oracle.
//!   2. Image theorem: envelopes are exactly the descending chains, (d+1)^k of them.
//!   3. Fibre theorem: the fibre over a chain G has size 2^(sum_{i>=2} |G_i|).
//!   4. Fixed union K, |K| = k: (2^d - 1)^k deposit sequences.
//!   5. Suffix-mass generating polynomial (1 + sum_r 2^(r-1) y^r)^k.
//!   6. Stirling bridge: (2^d-1)^k = sum_r N(k,r) r! S(d,r).
//!   7. Union fibres of the tower: |fibre over K| = F(|K|) = 2,2,10,218,64594,
//!      with totals 2^(2^m); eta/mu/rho retraction facts; the FOUR collapse table.

use std::collections::HashMap;
use vox::provenance_envelope::{restored_support_ladder, suffix_fibre_size};

/// Independent oracle for the suffix envelope of a deposit sequence. `q[i]` is
/// the lane bitmask deposited in frame i+1 (outermost first); `g[i]` is the union
/// of `q[i..]`.
fn model_envelope(q: &[u32]) -> Vec<u32> {
    let mut g = vec![0u32; q.len()];
    let mut acc = 0u32;
    for i in (0..q.len()).rev() {
        acc |= q[i];
        g[i] = acc;
    }
    g
}

fn provenance_envelope(q: &[u32]) -> Vec<u32> {
    let native = restored_support_ladder(q);
    assert_eq!(native, model_envelope(q), "native restoration diverged from suffix envelope");
    native
}

fn all_sequences(k: u32, d: usize) -> impl Iterator<Item = Vec<u32>> {
    let base = 1u64 << k;
    let total = 1u64 << ((k as usize) * d);
    (0..total).map(move |mut n| {
        let mut q = Vec::with_capacity(d);
        for _ in 0..d {
            q.push((n % base) as u32);
            n /= base;
        }
        q
    })
}

fn binom(n: u128, r: u128) -> u128 {
    if r > n {
        return 0;
    }
    let r = r.min(n - r);
    let mut out: u128 = 1;
    for i in 0..r {
        out = out * (n - i) / (i + 1);
    }
    out
}

/// N(k,r): r-element families of subsets of a k-set with union the whole set.
fn n_kr(k: u32, r: u128) -> i128 {
    let mut s: i128 = 0;
    for j in 0..=k {
        let term = binom(k as u128, j as u128) as i128 * binom(1u128 << (k - j), r) as i128;
        if j % 2 == 0 {
            s += term;
        } else {
            s -= term;
        }
    }
    s
}

fn stirling2(d: usize, r: usize) -> u128 {
    let mut t = vec![vec![0u128; r + 2]; d + 2];
    t[0][0] = 1;
    for n in 1..=d {
        for j in 1..=r.min(n) {
            t[n][j] = j as u128 * t[n - 1][j] + t[n - 1][j - 1];
        }
    }
    t[d][r]
}

fn factorial(n: u128) -> u128 {
    (1..=n).product()
}

#[test]
fn suffix_envelope_fibres_are_exact_products() {
    let cases: Vec<(u32, usize)> = (1..=6)
        .map(|d| (1, d))
        .chain((1..=5).map(|d| (2, d)))
        .chain((1..=4).map(|d| (3, d)))
        .chain((1..=4).map(|d| (4, d)))
        .collect();
    let mut sequences_checked: u64 = 0;

    for &(k, d) in &cases {
        let mut fibre: HashMap<Vec<u32>, u128> = HashMap::new();
        let mut mass: HashMap<u32, u128> = HashMap::new();
        let mut full_union: u128 = 0;
        let full_mask = (1u32 << k) - 1;

        for q in all_sequences(k, d) {
            let g = provenance_envelope(&q);
            *fibre.entry(g.clone()).or_insert(0) += 1;
            let m: u32 = g.iter().map(|x| x.count_ones()).sum();
            *mass.entry(m).or_insert(0) += 1;
            if g[0] == full_mask {
                full_union += 1;
            }
            sequences_checked += 1;
        }

        assert_eq!(fibre.len() as u128, ((d + 1) as u128).pow(k), "image size k={k} d={d}");
        for (g, &count) in &fibre {
            for i in 0..d.saturating_sub(1) {
                assert_eq!(g[i] & g[i + 1], g[i + 1], "not descending k={k} d={d}");
            }
            let exp: u32 = g[1..].iter().map(|x| x.count_ones()).sum();
            assert_eq!(count, 1u128 << exp, "fibre size over {g:?} k={k} d={d}");
            assert_eq!(suffix_fibre_size(g), Some(count), "library fibre size over {g:?} k={k} d={d}");
        }
        assert_eq!(fibre.values().sum::<u128>(), 1u128 << (k as usize * d));

        assert_eq!(full_union, ((1u128 << d) - 1).pow(k), "fixed-union count k={k} d={d}");

        let mut p = vec![1u128];
        p.extend((1..=d).map(|r| 1u128 << (r - 1)));
        let mut poly = vec![1u128];
        for _ in 0..k {
            let mut next = vec![0u128; poly.len() + d];
            for (i, a) in poly.iter().enumerate() {
                for (j, b) in p.iter().enumerate() {
                    next[i + j] += a * b;
                }
            }
            poly = next;
        }
        for (m, &c) in poly.iter().enumerate() {
            assert_eq!(mass.get(&(m as u32)).copied().unwrap_or(0), c, "mass poly k={k} d={d} m={m}");
        }
    }
    println!(
        "native suffix envelope: {} (k,d) cases, {} deposit sequences; restoration, image, fibre product, fixed-union and mass polynomial all exact",
        cases.len(),
        sequences_checked
    );
}

#[test]
fn stirling_bridge_connects_union_fibres_to_ordered_provenance() {
    for k in 0..=4u32 {
        for d in 1..=6usize {
            let lhs = ((1u128 << d) - 1).pow(k);
            let mut rhs: i128 = 0;
            for r in 0..=(1usize << k) {
                rhs += n_kr(k, r as u128)
                    * factorial(r as u128) as i128
                    * stirling2(d, r) as i128;
            }
            assert_eq!(lhs as i128, rhs, "bridge k={k} d={d}");
        }
    }
    println!("stirling bridge: (2^d-1)^k = sum_r N(k,r) r! S(d,r) exact for k<=4, d<=6");
}

/// Union fibres of the tower. A point of V_{n+1} over a universe X of m = |V_{n-1}|
/// atoms is a family of subsets of X, coded as a bitmask over the 2^m subsets.
/// mu is union; eta(K) = {K}; rho = eta . mu.
#[test]
fn tower_union_fibres_and_frame_collapse_retraction() {
    const F: [u64; 5] = [2, 2, 10, 218, 64_594];

    for m in 0..=4u32 {
        let n_subsets = 1usize << m;
        let n_families: u64 = 1u64 << n_subsets;
        let mu = |f: u64| -> usize {
            let mut u = 0usize;
            for s in 0..n_subsets {
                if (f >> s) & 1 == 1 {
                    u |= s;
                }
            }
            u
        };
        let eta = |k: usize| -> u64 { 1u64 << k };
        let rho = |f: u64| -> u64 { eta(mu(f)) };

        let mut fibre = vec![0u64; n_subsets];
        let mut fixed = 0u64;
        for f in 0..n_families {
            fibre[mu(f)] += 1;
            assert_eq!(rho(rho(f)), rho(f), "rho not idempotent m={m}");
            if rho(f) == f {
                fixed += 1;
            }
        }
        for (kmask, &c) in fibre.iter().enumerate() {
            assert_eq!(c, F[(kmask as u32).count_ones() as usize], "fibre over {kmask:b} m={m}");
        }
        assert_eq!(fibre.iter().sum::<u64>(), n_families);
        for k in 0..n_subsets {
            assert_eq!(mu(eta(k)), k, "mu.eta != id m={m}");
        }
        assert_eq!(fixed, n_subsets as u64, "fixed points of rho m={m}");
        assert!(fixed < n_families, "rho must not be the identity m={m}");
    }

    let mu1 = |f: u64| -> usize { ((f >> 1) & 1) as usize };
    let rho1: Vec<u64> = (0..4u64).map(|f| 1u64 << mu1(f)).collect();
    assert_eq!(rho1, vec![0b01, 0b01, 0b10, 0b10], "N->F, F->F, T->T, B->T");

    println!(
        "tower: mu_0, mu_1, mu_2 fibres = 2,2 | 2,2,2,10 | 2,2,10,218,64594 by |K|; totals 4, 16, 65536; rho idempotent, |Fix rho| = |V_n|; FOUR collapse N->F, B->T"
    );
}
