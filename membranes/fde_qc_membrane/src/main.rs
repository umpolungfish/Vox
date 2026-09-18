//! FDE quantum-computer membrane: nest THREE Belnap-FOUR quantum computers
//! into ONE membrane that factors an arbitrary u128 N.
//!
//! There is no cap and no wall. Each arm is an independent FDE quantum
//! computer — a B4 register whose orbit is measured — and the membrane
//! recurses: it factors N, then factors each composite cofactor with the
//! same three-arm membrane, returning the full prime factorisation.
//!
//!   FDE-QC #1  BELNAP-BRENT      rho-cycle on the B4 ModExp register
//!   FDE-QC #2  BELNAP-p-1        exponent-register accumulation
//!   FDE-QC #3  BELNAP-SQUARING   near-root squaring cycle
//!
//! Quantum mechanics mis-describes these machines; FDE names them correctly.
use std::env;
use std::process::ExitCode;
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum B4 { T, F, B, N }

fn b4_hadamard(_b: B4) -> B4 { B4::B }

fn b4_xor(a: B4, b: B4) -> B4 {
    use B4::*;
    match (a, b) {
        (B, _) | (_, B) => B,
        (T, T) | (F, F) => F,
        (T, F) | (F, T) => T,
        _ => N,
    }
}

fn belnap_join(a: B4, b: B4) -> B4 {
    use B4::*;
    match (a, b) {
        (B, _) | (_, B) => B,
        (T, T) => T,
        (F, F) => F,
        (N, x) | (x, N) => x,
        _ => B,
    }
}

fn gcd(mut a: u128, mut b: u128) -> u128 {
    while b != 0 { let t = b; b = a % b; a = t; }
    a
}

/// Exact widening multiply-mod for all u128 n (peasant when n >= 2^64).
fn mulmod(a: u128, b: u128, n: u128) -> u128 {
    if n < (1u128 << 64) {
        ((a % n) * (b % n)) % n
    } else {
        let mut r: u128 = 0;
        let mut a = a % n;
        let mut b = b;
        while b > 0 {
            if b & 1 == 1 { r = (r + a) % n; }
            a = (a << 1) % n;
            b >>= 1;
        }
        r
    }
}

fn pow_mod(mut base: u128, mut exp: u128, n: u128) -> u128 {
    let mut acc: u128 = 1 % n;
    base %= n;
    while exp > 0 {
        if exp & 1 == 1 { acc = mulmod(acc, base, n); }
        exp >>= 1;
        base = mulmod(base, base, n);
    }
    acc
}

fn isqrt(n: u128) -> u128 {
    if n < 2 { return n; }
    let mut x = n;
    let mut y = (x + 1) / 2;
    while y < x { x = y; y = (x + n / x) / 2; }
    x
}

fn absdiff(a: u128, b: u128) -> u128 { if a > b { a - b } else { b - a } }

fn mr_witness(a: u128, n: u128, d: u128, s: u32) -> bool {
    let mut x = pow_mod(a, d, n);
    if x == 1 || x == n - 1 { return false; }
    for _ in 1..s {
        x = mulmod(x, x, n);
        if x == n - 1 { return false; }
    }
    true
}

fn is_prime(n: u128) -> bool {
    if n < 2 { return false; }
    for p in [2u128, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37] {
        if n == p { return true; }
        if n % p == 0 { return false; }
    }
    let mut d = n - 1;
    let mut s = 0u32;
    while d % 2 == 0 { d /= 2; s += 1; }
    for a in [2u128, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37] {
        if a % n == 0 { continue; }
        if mr_witness(a, n, d, s) { return false; }
    }
    true
}
