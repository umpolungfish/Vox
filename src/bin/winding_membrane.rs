//! winding_membrane — one baked case, three winding reads, iterated.
//!
//! The base and modulus are baked in as IMASM numerals at build time; there is
//! no runtime input. The period of `a` mod `N` is the winding, and this membrane
//! reads it three ways, because there is never a case to run only one:
//!
//!   1. the phase register    (shor_qft, the QFT membrane's extract_order)
//!   2. the dyadic partners   (phase_partners, a^(2^k) collisions)
//!   3. the split/fuse ladder (the r-cycle Jacobi, period as eigenphase)
//!
//! Each arm reads a winding and closes the factors by gcd(a^{r/2} +- 1, N).
//! The arms are separate instruments answering the same question; the membrane
//! runs all three and reports each.
#![allow(dead_code)]
extern crate alloc;

#[path = "../membrane_complex.rs"] mod membrane_complex;
#[path = "../fde_shor_membrane.rs"] mod fde_shor_membrane;
#[path = "../shor_qft.rs"] mod shor_qft;
#[path = "../phase_word.rs"] mod phase_word;
#[path = "../phase_partners.rs"] mod phase_partners;

use vox::morphism_factor::{parse_numeral, dec_of, tape_u64, one, cmp, modulo, mul};
use vox::shor_braid::factor_close_public;

include!(concat!(env!("OUT_DIR"), "/baked_inputs.rs"));

fn my_sqrt(x: f64) -> f64 {
    if x <= 0.0 { return 0.0; }
    let mut g = x;
    for _ in 0..64 { g = 0.5 * (g + x / g); }
    g
}

fn my_cos(theta: f64) -> f64 {
    let pi = core::f64::consts::PI;
    let two_pi = 2.0 * pi;
    let mut t = theta - two_pi * ((theta / two_pi) as i64 as f64);
    if t > pi { t -= two_pi; }
    if t < -pi { t += two_pi; }
    let t2 = t * t;
    1.0 - t2 / 2.0 + t2 * t2 / 24.0 - t2 * t2 * t2 / 720.0
        + t2 * t2 * t2 * t2 / 40320.0 - t2 * t2 * t2 * t2 * t2 / 3628800.0
}

fn bit_len(n: &[char]) -> usize {
    use vox::morphism_factor::{divmod, zero};
    let mut v = n.to_vec();
    let two = tape_u64(2);
    let mut bits = 0usize;
    while !zero(&v) { let (q, _r) = divmod(&v, &two); v = q; bits += 1; }
    bits.max(1)
}

// ---- arm 3: the split/fuse ladder, period as the eigenphase 2 cos(2 pi / r) ----

fn jacobi_eigs(mut a: Vec<Vec<f64>>) -> Vec<f64> {
    let n = a.len();
    if n == 0 { return Vec::new(); }
    for _ in 0..100 {
        let mut off = 0.0;
        for i in 0..n { for j in (i + 1)..n { off += a[i][j] * a[i][j]; } }
        if off < 1e-24 { break; }
        for p in 0..n {
            for q in (p + 1)..n {
                if a[p][q].abs() < 1e-18 { continue; }
                let theta = (a[q][q] - a[p][p]) / (2.0 * a[p][q]);
                let s = if theta >= 0.0 { 1.0 } else { -1.0 };
                let t = s / (theta.abs() + my_sqrt(theta * theta + 1.0));
                let c = 1.0 / my_sqrt(t * t + 1.0);
                let sn = t * c;
                for k in 0..n {
                    let akp = a[k][p]; let akq = a[k][q];
                    a[k][p] = c * akp - sn * akq; a[k][q] = sn * akp + c * akq;
                }
                for k in 0..n {
                    let apk = a[p][k]; let aqk = a[q][k];
                    a[p][k] = c * apk - sn * aqk; a[q][k] = sn * apk + c * aqk;
                }
            }
        }
    }
    let mut ev: Vec<f64> = (0..n).map(|i| a[i][i]).collect();
    ev.sort_by(|x, y| x.partial_cmp(y).unwrap_or(core::cmp::Ordering::Equal));
    ev
}

/// Walk the modular orbit to closure, build the r-cycle Jacobi operator, and
/// read the period as the eigenphase: the second-largest eigenvalue is
/// 2 cos(2 pi / r). Returns the read period, capped.
fn ladder_period(a: &[char], n: &[char], cap: usize) -> Option<usize> {
    let one_t = one();
    let mut v = modulo(a, n);
    let mut r = 1usize;
    while cmp(&v, &one_t) != core::cmp::Ordering::Equal {
        v = modulo(&mul(&v, a), n);
        r += 1;
        if r > cap { return None; }
    }
    // r-cycle adjacency, eigenvalues 2 cos(2 pi k / r)
    let mut m = alloc::vec![alloc::vec![0.0f64; r]; r];
    for i in 0..r { m[i][(i + 1) % r] = 1.0; m[(i + 1) % r][i] = 1.0; }
    let ev = jacobi_eigs(m);
    let lam2 = if ev.len() >= 2 { ev[ev.len() - 2] } else { return Some(r); };
    // invert 2 cos(2 pi / rr) = lam2 by matching candidates
    let pi = core::f64::consts::PI;
    let mut best = 0usize; let mut be = f64::INFINITY;
    for rr in 2..=(r + 2) {
        let model = 2.0 * my_cos(2.0 * pi / (rr as f64));
        let e = (model - lam2).abs();
        if e < be { be = e; best = rr; }
    }
    Some(best)
}

fn close(a: &[char], n: &[char], r: &[char]) -> String {
    match factor_close_public(a, n, r) {
        Ok((p, q)) => format!("factors {} x {}", dec_of(&p), dec_of(&q)),
        Err(e) => format!("no split ({e})"),
    }
}

fn main() -> Result<(), String> {
    let a = parse_numeral(BAKED_BASE_WORD.ok_or("bake a base word")?)?;
    let n = parse_numeral(BAKED_MODULUS_WORD.ok_or("bake a modulus word")?)?;
    let qubits = match BAKED_WIDTH_WORD {
        Some(w) => dec_of(&parse_numeral(w)?).parse::<usize>().map_err(|e| e.to_string())?,
        None => 2 * bit_len(&n) + 2,
    };
    println!("base {}  modulus {}  qubits {qubits}", dec_of(&a), dec_of(&n));

    // arm 1: the phase register, register grown to the smallest width that
    // exposes the winding (near the order, not the N^2 blowup of a fixed width).
    match shor_qft::observe_order(a.clone(), n.clone()) {
        Ok(r) => println!("phase-register  winding {}  {}", dec_of(&r), close(&a, &n, &r)),
        Err(e) => println!("phase-register  refused: {e}"),
    }

    // arm 2: the dyadic partners
    match phase_partners::Partners::new(a.clone(), n.clone()) {
        Ok(mut partners) => {
            let mut got = false;
            for _ in 0..(4 * qubits + 8) {
                if let Some(rel) = partners.observe()? {
                    let r = rel.return_exponent.clone();
                    println!("dyadic-partners winding {}  {}  [squarings {}]",
                        dec_of(&r), close(&a, &n, &r), partners.squarings);
                    got = true; break;
                }
            }
            if !got { println!("dyadic-partners no return within the membrane budget"); }
        }
        Err(e) => println!("dyadic-partners refused: {e}"),
    }

    // arm 3: the split/fuse ladder
    match ladder_period(&a, &n, 200_000) {
        Some(r) => {
            let rt = tape_u64(r as u64);
            println!("split-fuse-ladder winding {}  {}", r, close(&a, &n, &rt));
        }
        None => println!("split-fuse-ladder orbit exceeds the membrane cap"),
    }
    Ok(())
}
