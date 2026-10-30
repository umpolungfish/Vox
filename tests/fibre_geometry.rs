//! §7.7 acceptance tests for the FibreGeometry Core-Numeral axis.
//!
//! Reproduces the four acceptance quantities of `daplan.md` §7.7 exactly:
//!   1. the `N(k,r)` table for `k = 0..4` (covering families of exact size r),
//!   2. fibre sizes `2, 2, 10, 218, 64594`,
//!   3. minimal-cover counts `1, 1, 2, 8, 49` (A046165),
//!   4. Hasse-edge counts `1, 1, 15, 805, 513135`,
//! plus the Stage-58/200 Stirling bridge `(2^d - 1)^k = sum_r N(k,r) r! S(d, r)`
//! for `k <= 4`, `d <= 6`.

use vox::fibre_geometry::{
    fibre_size, hasse_edge_count, minimal_cover_count, n_kr, stirling_bridge_holds,
};

/// `N(k, r)` for `k = 0..4`, `r = 0..2^k` (covering families of exact size r).
const TABLE: [[i128; 17]; 5] = [
    [1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 1, 4, 4, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 1, 13, 44, 67, 56, 28, 8, 1, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 1, 40, 360, 1546, 4144, 7896, 11408, 12866, 11440, 8008, 4368, 1820, 560, 120, 16, 1],
];

#[test]
fn n_kr_table_is_exact_for_k_le_4() {
    for k in 0..5u32 {
        for r in 0..17u128 {
            assert_eq!(n_kr(k, r), TABLE[k as usize][r as usize], "N({k}, {r})");
        }
    }
    println!("n_kr table k=0..4 exact");
}

#[test]
fn fibre_sizes_match_seven_seven() {
    const F: [u128; 5] = [2, 2, 10, 218, 64594];
    for k in 0..5u32 {
        assert_eq!(fibre_size(k), F[k as usize], "F({k})");
    }
    // cross-axis: the tower test enumerates fibres exactly; F(k) must agree.
    for k in 0..5u32 {
        let total: i128 = (0..17u128).map(|r| n_kr(k, r)).sum();
        assert_eq!(total as u128, F[k as usize], "sum_r N({k},r)");
    }
    println!("fibre sizes 2, 2, 10, 218, 64594 exact");
}

#[test]
fn minimal_cover_counts_are_a046165() {
    const M: [u128; 5] = [1, 1, 2, 8, 49];
    for k in 0..5u32 {
        assert_eq!(minimal_cover_count(k), M[k as usize], "M({k})");
    }
    println!("minimal-cover counts 1, 1, 2, 8, 49 exact");
}

#[test]
fn hasse_edge_counts_match_seven_seven() {
    const H: [u128; 5] = [1, 1, 15, 805, 513135];
    for k in 0..5u32 {
        assert_eq!(hasse_edge_count(k), H[k as usize], "Hasse({k})");
    }
    println!("Hasse-edge counts 1, 1, 15, 805, 513135 exact");
}

#[test]
fn stirling_bridge_holds_for_k_le_4_d_le_6() {
    for k in 0..5u32 {
        for d in 1..=7usize {
            assert!(stirling_bridge_holds(k, d), "bridge k={k} d={d}");
        }
    }
    println!("Stirling bridge (2^d-1)^k = sum_r N(k,r) r! S(d,r) exact for k<=4, d<=6");
}
