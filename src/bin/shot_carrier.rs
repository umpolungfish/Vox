//! Resident order-finding carrier for the Shor state.
//! Holds the index register as an amplitude vector of length m (O(m) memory);
//! the output register |a^x mod N> is carried implicitly (deterministic per x),
//! so the readout is not bounded by the dense O(m*N) joint vector.
use std::env;
use std::f64::consts::PI;

fn mod_pow(mut a: u64, mut e: u64, n: u64) -> u64 {
    let mut r = 1u64; a %= n;
    while e > 0 {
        if e & 1 == 1 { r = (r as u128 * a as u128 % n as u128) as u64; }
        a = (a as u128 * a as u128 % n as u128) as u64;
        e >>= 1;
    }
    r
}

fn gcd(mut a: u64, mut b: u64) -> u64 { while b != 0 { let t = a % b; a = b; b = t; } a }

fn fft(re: &mut [f64], im: &mut [f64]) {
    let n = re.len();
    let mut j = 0usize;
    for i in 1..n {
        let mut bit = n >> 1;
        while j & bit != 0 { j ^= bit; bit >>= 1; }
        j |= bit;
        if i < j { re.swap(i, j); im.swap(i, j); }
    }
    let mut len = 2;
    while len <= n {
        let ang = -2.0 * PI / (len as f64);
        let (wr, wi) = (ang.cos(), ang.sin());
        let mut i = 0;
        while i < n {
            let (mut wrk, mut wik) = (1.0f64, 0.0f64);
            for k in 0..len / 2 {
                let ur = re[i + k]; let ui = im[i + k];
                let xr = re[i + k + len / 2]; let xi = im[i + k + len / 2];
                let vr = xr * wrk - xi * wik;
                let vi = xr * wik + xi * wrk;
                re[i + k] = ur + vr; im[i + k] = ui + vi;
                re[i + k + len / 2] = ur - vr; im[i + k + len / 2] = ui - vi;
                let nwr = wrk * wr - wik * wi;
                wik = wrk * wi + wik * wr; wrk = nwr;
            }
            i += len;
        }
        len <<= 1;
    }
}

fn period_from_peak(k: usize, m: usize, a: u64, n: u64) -> Option<u64> {
    let (mut p_prev, mut p_curr) = (0i128, 1i128);
    let (mut q_prev, mut q_curr) = (1i128, 0i128);
    let (mut kk, mut mm) = (k as i128, m as i128);
    while mm != 0 {
        let aa = kk / mm;
        let p_next = aa * p_curr + p_prev;
        let q_next = aa * q_curr + q_prev;
        if q_next > 0 && (q_next as u128) < n as u128 && mod_pow(a, q_next as u64, n) == 1 {
            return Some(q_next as u64);
        }
        p_prev = p_curr; p_curr = p_next;
        q_prev = q_curr; q_curr = q_next;
        let rem = kk % mm; kk = mm; mm = rem;
    }
    None
}

fn shot_order(a: u64, n: u64, q: u32) -> Option<u64> {
    let m = 1usize << q;
    let mut re = vec![0.0f64; m];
    let mut im = vec![0.0f64; m];
    let mut v = 1u64;
    let mut cnt = 0usize;
    for x in 0..m {
        if v == 1 { re[x] = 1.0; cnt += 1; }
        v = (v as u128 * a as u128 % n as u128) as u64;
    }
    if cnt == 0 { return None; }
    let norm = (cnt as f64).sqrt();
    for x in 0..m { re[x] /= norm; }
    fft(&mut re, &mut im);
    let probs: Vec<f64> = (0..m).map(|k| re[k] * re[k] + im[k] * im[k]).collect();
    let mut idx: Vec<usize> = (1..m).collect();
    idx.sort_by(|&x, &y| probs[y].partial_cmp(&probs[x]).unwrap());
    for &k in idx.iter().take(64) {
        if let Some(r) = period_from_peak(k, m, a, n) { return Some(r); }
    }
    None
}

fn factor_close(a: u64, n: u64, r: u64) -> Option<(u64, u64)> {
    if r == 0 || r % 2 != 0 { return None; }
    let half = mod_pow(a, r / 2, n);
    if half == n - 1 { return None; }
    let g1 = gcd((half + n - 1) % n, n);
    if g1 > 1 && g1 < n { return Some((g1, n / g1)); }
    let g2 = gcd((half + 1) % n, n);
    if g2 > 1 && g2 < n { return Some((g2, n / g2)); }
    None
}

fn main() {
    let args: Vec<u64> = env::args().skip(1).filter_map(|s| s.parse().ok()).collect();
    let cases: Vec<(u64, u32)> = if args.len() >= 2 {
        vec![(args[0], args[1] as u32)]
    } else {
        vec![(15, 10), (21, 10), (35, 10), (91, 10), (143, 14), (713, 11)]
    };
    let bases: [u64; 8] = [2, 3, 5, 7, 11, 13, 17, 19];
    for (n, q) in cases {
        let m = 1usize << q;
        let t0 = std::time::Instant::now();
        match shot_factor(n, q, &bases) {
            Some((p, f, a)) => println!(
                "N={n:>8} m=2^{q:<2} carrier={:>9} B (dense {:>10} B)  a={a:<3} -> {p} x {f}  ok={}  {}us",
                irect_mem(m), m * (n as usize) * 16, p * f == n, t0.elapsed().as_micros()),
            None => println!("N={n:>8} m=2^{q:<2}  no factor any base  {}us", t0.elapsed().as_micros()),
        }
    }
}

fn shot_factor(n: u64, q: u32, bases: &[u64]) -> Option<(u64, u64, u64)> {
    for &a in bases {
        let g = gcd(a, n);
        if g > 1 && g < n { return Some((g, n / g, a)); }
        if let Some(r) = shot_order(a, n, q) {
            if let Some((p, f)) = factor_close(a, n, r) { return Some((p, f, a)); }
        }
    }
    None
}

fn irect_mem(m: usize) -> usize { 2 * m * 8 }
