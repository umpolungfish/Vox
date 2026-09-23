//! 2-Adic Prefix Integer Factorization for Arbitrary-Precision Integers
//!
//! This module implements 2-adic modular inversion and prefix-based factorization
//! for arbitrarily large integers using the num-bigint crate.
//!
//! Quantum Advantage Context:
//! - 2-adic arithmetic enables efficient period-finding circuits for Shor's algorithm
//! - Prefix inversion reduces search space exponentially when bit lengths are known
//! - Modular exponentiation in 2-adic rings has favorable circuit depth properties
//!
//! Usage: cargo run --bin factor_2adic_bigint <N> [lp] [lq] [max_solutions]

use num_bigint::BigUint;
use num_traits::{One, Zero};
use std::env;
use std::vec::Vec;

/// Check if a BigUint is even (least significant bit is 0)
fn is_even(n: &BigUint) -> bool {
    n.to_bytes_le().first().map_or(true, |b| (b & 1) == 0)
}

/// Compute modular inverse of odd a modulo 2^k using Newton-Raphson.
/// Quadratic convergence: doubles correct bits each iteration.
///
/// For 2-adic rings, this is the core operation enabling prefix factorization.
/// The inverse exists iff a is odd (unit in Z_2).
fn mod_inverse_odd_2adic(a: &BigUint, k: u32) -> Option<BigUint> {
    if is_even(a) {
        return None; // Must be odd (unit in 2-adic ring)
    }
    
    // Start with inverse mod 8 (since a is odd, a^2 ≡ 1 mod 8)
    // a * (a mod 8) ≡ 1 mod 8 for odd a
    let mut x = a % BigUint::from(8u32);
    let mut bits: u32 = 3;
    
    while bits < k {
        // Newton-Raphson: x_{n+1} = x_n * (2 - a * x_n) mod 2^{2*bits}
        let modulus = BigUint::one() << (bits * 2).min(64); // Cap at 64-bit steps
        let two = BigUint::from(2u32);
        let ax = a * &x;
        let two_minus_ax = (two - (ax % &modulus)) % &modulus;
        x = (&x * two_minus_ax) % &modulus;
        bits *= 2;
        if bits > k {
            bits = k;
        }
    }
    
    let final_modulus = BigUint::one() << k;
    Some(x % final_modulus)
}

/// Check if prefix mod 2^k can extend to [lower, upper] interval
///
/// This is the key pruning operation: given a k-bit prefix, determine if
/// any extension can fall within the target bit-length bounds.
///
/// An extension is of the form: prefix + j * 2^k for j >= 0
/// We need to check if any such extension falls in [lower, upper].
fn prefix_intersects_interval(
    prefix: &BigUint,
    k: u32,
    lower: &BigUint,
    upper: &BigUint,
) -> bool {
    if lower > upper {
        return false;
    }
    
    let modulus = BigUint::one() << k;
    
    // Find smallest extension: prefix + j*modulus >= lower
    // If prefix >= lower, then j=0 works and candidate = prefix
    // If prefix < lower, we need j = ceil((lower - prefix) / modulus)
    let candidate = if prefix >= lower {
        prefix.clone()
    } else {
        let diff = lower - prefix;
        let jumps = (diff + &modulus - BigUint::one()) / &modulus;
        prefix + &jumps * &modulus
    };
    
    // Check if this candidate is within [lower, upper]
    candidate <= *upper
}
/// Search for factors with given bit lengths using 2-adic prefix inversion
///
/// DFS over p prefixes, computing q = N * p^(-1) mod 2^k at each step.
/// Prune branches where either p or q prefix cannot extend to valid bit lengths.
fn search_p_prefix(
    n: &BigUint,
    lp: u32,
    lq: u32,
    solutions: &mut Vec<(BigUint, BigUint)>,
) {
    let p_lower = BigUint::one() << (lp - 1);
    let p_upper = (BigUint::one() << lp) - BigUint::one();
    let q_lower = BigUint::one() << (lq - 1);
    let q_upper = (BigUint::one() << lq) - BigUint::one();
    
    // DFS stack: (p_k, k) where p_k = p mod 2^k, k bits determined
    let mut stack: Vec<(BigUint, u32)> = Vec::new();
    stack.push((BigUint::one(), 1)); // Start with p_1 = 1 (odd, 1 bit)
    
    while let Some((p_k, k)) = stack.pop() {
        if k == lp {
            // Found candidate p with exactly lp bits — check if it divides N
            if n % &p_k == BigUint::zero() {
                let q = n / &p_k;
                // Filter out trivial factors (p=1 or q=1)
                if &p_k > &BigUint::one() && &q > &BigUint::one() 
                    && &q >= &q_lower && &q <= &q_upper {
                    if &p_k <= &q {
                        solutions.push((p_k, q));
                    } else {
                        solutions.push((q, p_k));
                    }
                }
            }
            continue;
        }
        
        // Try extending p by bit 0 or 1 at position k
        for _bit in [0u32, 1u32].iter() {
            let next_p = &p_k | (BigUint::one() << k);
            
            // Prune p prefix
            if !prefix_intersects_interval(&next_p, k + 1, &p_lower, &p_upper) {
                continue;
            }
            
            // Compute q_k = N * p_k^(-1) mod 2^(k+1)
            if let Some(p_inv) = mod_inverse_odd_2adic(&next_p, k + 1) {
                let modulus = BigUint::one() << (k + 1);
                let next_q = (n * &p_inv) % &modulus;
                
                // Prune q prefix
                if !prefix_intersects_interval(&next_q, k + 1, &q_lower, &q_upper) {
                    continue;
                }
                
                stack.push((next_p, k + 1));
            }
        }
    }
}

/// Main 2-adic factorization routine for arbitrary precision
pub fn factor_2adic_bigint(
    n: &BigUint,
    lp: Option<usize>,
    lq: Option<usize>,
    max_solutions: Option<usize>,
) -> Vec<(BigUint, BigUint)> {
    if n <= &BigUint::one() || is_even(n) {
        return Vec::new();
    }
    
    let mut solutions: Vec<(BigUint, BigUint)> = Vec::new();
    let n_bits = n.bits() as usize;
    
    if let (Some(lp), Some(lq)) = (lp, lq) {
        search_p_prefix(n, lp as u32, lq as u32, &mut solutions);
    } else {
        // Try all reasonable bit-length pairs, assuming p <= q
        // So lp <= n_bits/2 and lq >= n_bits/2
        for lp in 1..=(n_bits / 2 + 1) {
            // Try both possible lq values: n_bits - lp and n_bits - lp + 1
            for lq in [n_bits - lp, n_bits - lp + 1].iter() {
                if *lq >= lp {
                    search_p_prefix(n, lp as u32, *lq as u32, &mut solutions);
                }
            }
            
            if let Some(max) = max_solutions {
                if solutions.len() >= max {
                    break;
                }
            }
        }
    }
    
    // Remove duplicates and sort
    solutions.sort();
    solutions.dedup();
    
    solutions
}
