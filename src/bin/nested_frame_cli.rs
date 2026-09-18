// nested_frame_cli — compare the capacity-gated nesting carrier (nested_frame)
// against the DFS semiprime_shot on the SAME semiprimes. Metrics: frames
// entered, killed by p, killed by q, max live capacity, max nesting depth,
// exact candidates reaching FIX — and frames-entered / factor-bits.
extern crate alloc;
use ::vox::morphism_factor::{dec_of, tape_u64};

// deterministic u64 Miller-Rabin
fn is_prime(n: u64) -> bool {
    if n < 2 { return false; }
    for p in [2u64, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37] {
        if n % p == 0 { return n == p; }
    }
    let mut d = n - 1; let mut r = 0u32;
    while d & 1 == 0 { d >>= 1; r += 1; }
    'witness: for &a in &[2u64, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37] {
        let mut x = modpow(a % n, d, n);
        if x == 1 || x == n - 1 { continue; }
        for _ in 0..r - 1 {
            x = (x as u128 * x as u128 % n as u128) as u64;
            if x == n - 1 { continue 'witness; }
        }
        return false;
    }
    true
}
fn modpow(mut b: u64, mut e: u64, m: u64) -> u64 {
    let mut r = 1u64; b %= m;
    while e > 0 { if e & 1 == 1 { r = (r as u128 * b as u128 % m as u128) as u64; } b = (b as u128 * b as u128 % m as u128) as u64; e >>= 1; }
    r
}
struct Lcg(u64);
impl Lcg { fn next(&mut self) -> u64 { self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407); self.0 } }

/// Two random odd primes with exactly `bits` bits each (top bit set).
fn gen_semiprime(bits: u32, rng: &mut Lcg) -> u64 {
    let lo = 1u64 << (bits - 1);
    let p: u64;
    loop {
        let c = (rng.next() % lo) | lo | 1;
        let cand = (c | (1u64 << (bits - 1))) | 1;
        if is_prime(cand) { p = cand; break; }
    }
    let mut q = p;
    while q == p {
        let c = (rng.next() % (lo)) | lo | 1;
        let cand = (c | (1u64 << (bits - 1))) | 1;
        if is_prime(cand) { q = cand; }
    }
    p * q
}
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let shot_budget: u64 = 20_000_000;
    let nest_budget: u64 = 50_000_000;
    let run = |n: u64| {
        // nested carrier
        let t0 = std::time::Instant::now();
        let (res, st) = ::vox::nested_frame::factor_nested(n, nest_budget);
        let dt = t0.elapsed().as_millis();
        let bits = 64 - n.leading_zeros();
        // DFS comparison
        let t1 = std::time::Instant::now();
        let (sres, sst) = ::vox::factor_operator::semiprime_shot(&tape_u64(n), shot_budget);
        let dt2 = t1.elapsed().as_millis();
        let sfound = match sres { Some((p, q)) => format!("{} x {}", dec_of(&p), dec_of(&q)), None => String::from("None") };
        let nfound = match res { Some((p, q)) => format!("{p} x {q} ok={}", p * q == n), None => String::from("None") };
        let shape = if st.shape_closed { format!("step{}", st.shape_step) } else { String::from("-") };
        println!("N={n} bits={bits}");
        println!("  NESTED -> {nfound}  frames={} kill_p={} kill_q={} fix={} maxlive={} depth={} capped={} falsefix={} shape={}  f/bits={:.2} gain={:.4} lo/hi={:.3}  {}ms",
            st.frames_entered, st.killed_p, st.killed_q, st.fixed, st.max_live_capacity, st.max_depth, st.capped, st.false_fix, shape,
            st.frames_entered as f64 / bits as f64,
            st.child_sum as f64 / st.parent_cap_sum.max(1) as f64,
            st.low_sum as f64 / st.high_sum.max(1) as f64, dt);
        println!("  SHOT   -> {sfound}  nodes={} depth={} capped={}  n/bits={:.2}  {}ms",
            sst.nodes, sst.depth, sst.capped, sst.nodes as f64 / bits as f64, dt2);
    };
    if args.is_empty() {
        // the same random semiprimes recorded last session, plus structured ones
        for &n in &[61304504203u64, 723862509691, 8097694716157, 106545994355809, 1000036000099, 4000064000087] { run(n); }
    } else if args[0] == "gen" {
        let bits: u32 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(18);
        let count: u32 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(3);
        let seed: u64 = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(0xBADC0FFEE);
        let mut rng = Lcg(seed);
        for b in bits..(bits + count) {
            let n = gen_semiprime(b, &mut rng);
            run(n);
        }
    } else {
        for s in args.iter().filter_map(|s| s.parse::<u64>().ok()) { run(s); }
    }
}
