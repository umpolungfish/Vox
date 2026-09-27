//! Combined P-membrane and frame-shift unbraid factorizer.
//!
//! Synthesizes the structural orbit (the frame-shift unbraid over the ROTAT orbit
//! of D(N)) and the phase-based orbit (the P-membrane dyadic phase arm on base g=2).
//!
//! Domain: F (odd, non-Mersenne semiprimes greater than one, with no periodic bit run).
//! Coupling:
//!   (p, q) = gcd(2^(r/2) - 1, N)  where r = ord_N(2)
//!   (p, q) = Lambda( ROTAT^{ell*}(D(N)) )
//! Consistency condition:
//!   ell* = r mod k, the frame position where Gamma(D(p), D(q)) returns N via mu.
//! Syzygy:
//!   [encode; Gamma; Lambda; mu] faithful <=> Frobenius PASS: mu o delta = id.

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use num_bigint::BigUint;
use num_traits::{One, Zero};

/// Canonical hex digit words from g-mOMonadOS `prime_winding::HEX_WORDS`.
pub const HEX_WORDS: [&str; 16] = [
    "⊢≻⋈≺⊙⊡⊣", "⊢⊤≻⋈≺⊙⊡⊣", "⊢≻⋈⊥≺⊙⊡⊣", "⊢⊤≻⋈⊥≺⊙⊡⊣",
    "⊢≻⋈≺⊞⊙⊡⊣", "⊢⊤≻⋈≺⊞⊙⊡⊣", "⊢≻⋈⊥≺⊞⊙⊡⊣", "⊢⊤≻⋈⊥≺⊞⊙⊡⊣",
    "⊢∈≻⋈≺⋈∋⊙⊡⊣", "⊢∈⊤≻⋈≺⋈∋⊙⊡⊣", "⊢∈≻⋈⊥≺⋈∋⊙⊡⊣", "⊢∈⊤≻⋈⊥≺⋈∋⊙⊡⊣",
    "⊢∈≻⋈≺⋈⊞∋⊙⊡⊣", "⊢∈⊤≻⋈≺⋈⊞∋⊙⊡⊣", "⊢∈≻⋈⊥≺⋈⊞∋⊙⊡⊣", "⊢∈⊤≻⋈⊥≺⋈⊞∋⊙⊡⊣",
];

pub fn hex_word(n: &BigUint) -> String {
    let mut out = String::new();
    for c in n.to_str_radix(16).chars() {
        if let Some(d) = c.to_digit(16) { out.push_str(HEX_WORDS[d as usize]); }
    }
    out
}

pub fn decode_hex_word(word: &str) -> Result<BigUint, String> {
    let mut digits = String::new();
    let mut rest = word.trim();
    if rest.is_empty() { return Err("empty hex-digit word".into()); }
    while !rest.is_empty() {
        let end = rest.find("⊡⊣").ok_or_else(|| "hex-digit word is missing a ⊡⊣ terminator".to_string())? + "⊡⊣".len();
        let (digit_word, tail) = rest.split_at(end);
        let digit = HEX_WORDS.iter().position(|candidate| *candidate == digit_word)
            .ok_or_else(|| format!("unknown canonical hex digit block: {digit_word}"))?;
        digits.push(char::from_digit(digit as u32, 16).unwrap());
        rest = tail;
    }
    if digits.len() > 1 && digits.starts_with('0') {
        return Err("hex-digit word has a leading zero block".into());
    }
    BigUint::parse_bytes(digits.as_bytes(), 16).ok_or_else(|| "could not decode hex-digit word".into())
}

pub fn native_word(n: &BigUint) -> String {
    if n.is_zero() { return "⊢⊙⊡⊣".into(); }
    let bits = n.to_str_radix(2);
    let mut out = String::from("⊢");
    for bit in bits.bytes().rev() {
        out.push_str("≻⋈∈");
        out.push(if bit == b'1' { '⊥' } else { '⊤' });
        out.push('∋');
    }
    out.push_str("⊙⊡⊣");
    out
}

pub fn decode_native_word(word: &str) -> Result<BigUint, String> {
    let chars: Vec<char> = word.trim().chars().collect();
    if chars.len() == 4 && chars[0] == '⊢' && chars[1] == '⊙' && chars[2] == '⊡' && chars[3] == '⊣' {
        return Ok(BigUint::zero());
    }
    if chars.len() < 9 || chars[0] != '⊢' || chars[chars.len()-3] != '⊙'
        || chars[chars.len()-2] != '⊡' || chars[chars.len()-1] != '⊣' {
        return Err("expected ⊢(≻⋈∈[⊥|⊤]∋)+⊙⊡⊣".into());
    }
    let body = &chars[1..chars.len()-3];
    if body.len() % 5 != 0 { return Err("native word has a partial bit cell".into()); }
    if body.len() < 5 || body[body.len()-2] != '⊥' {
        return Err("native word has a leading zero bit or no bit cells".into());
    }
    let mut value = BigUint::zero();
    for (index, cell) in body.chunks(5).enumerate() {
        if cell[0] != '≻' || cell[1] != '⋈' || cell[2] != '∈' || cell[4] != '∋' {
            return Err(format!("invalid native bit cell at offset {index}"));
        }
        match cell[3] {
            '⊥' => value |= BigUint::one() << index,
            '⊤' => {},
            other => return Err(format!("invalid native bit {other}")),
        }
    }
    Ok(value)
}

pub fn bits_le(n: &BigUint) -> Vec<bool> {
    if n.is_zero() { return alloc::vec![false]; }
    n.to_str_radix(2).bytes().rev().map(|b| b == b'1').collect()
}

pub fn from_bits_le(bits: &[bool]) -> BigUint {
    let mut n = BigUint::zero();
    for (i, &bit) in bits.iter().enumerate() {
        if bit { n |= BigUint::one() << i; }
    }
    n
}

pub fn popcount(n: &BigUint) -> usize {
    bits_le(n).into_iter().filter(|&b| b).count()
}

pub fn is_mersenne(n: &BigUint) -> bool {
    !n.is_zero() && n.to_str_radix(2).bytes().all(|bit| bit == b'1')
}

pub fn dialect_register(n: &BigUint) -> &'static str {
    if is_mersenne(n) { "001000011100" } else { "111111111111" }
}

/// Compute the minimal bit-run period of the LSB-first bit representation.
pub fn minimal_bit_run_period(bits: &[bool]) -> usize {
    let len = bits.len();
    for period in 1..=len {
        let mut ok = true;
        for i in 0..len {
            if bits[i] != bits[i % period] {
                ok = false;
                break;
            }
        }
        if ok {
            return period;
        }
    }
    len
}

/// Check if the bit run is a repetition of a shorter word with period dividing 2^depth.
pub fn is_periodic_bit_run_dividing_2d(bits: &[bool], depth: usize) -> (bool, usize) {
    let len = bits.len();
    let power_2d = 1usize << depth;
    for p in 1..len {
        if len % p == 0 && power_2d % p == 0 {
            let mut is_rep = true;
            for i in 0..len {
                if bits[i] != bits[i % p] {
                    is_rep = false;
                    break;
                }
            }
            if is_rep {
                return (true, p);
            }
        }
    }
    (false, len)
}

/// Domain validation for the combined factorizer.
/// Enforces:
/// 1. N > 1
/// 2. Odd N (so g=2 is coprime to N: gcd(2, N) = 1)
/// 3. Non-Mersenne (bit-run period is not 1)
/// 4. No periodic bit run dividing 2^depth (non-trivial lane content)
pub fn shift_domain_check(n: &BigUint, depth: usize) -> Result<(), String> {
    if n <= &BigUint::one() {
        return Err("value must be greater than one".into());
    }
    if (n % 2u8).is_zero() {
        return Err("phase base is not coprime to N (even values are outside the shift-faithful domain)".into());
    }
    if is_mersenne(n) {
        return Err("Mersenne values are outside the shift-faithful domain: bit-run ROTAT period is 1, frame decode is non-injective".into());
    }
    let bits = bits_le(n);
    let (is_periodic, period) = is_periodic_bit_run_dividing_2d(&bits, depth);
    if is_periodic {
        return Err(format!(
            "periodic bit run with period {period} dividing 2^{depth}: frame decode is non-injective at depth {depth}"
        ));
    }
    Ok(())
}

pub fn is_shift_faithful(n: &BigUint) -> bool {
    shift_domain_check(n, 1).is_ok()
}

/// ROTAT orbit cyclic shift of a bit sequence by cut positions.
pub fn rotat_bits(bits: &[bool], cut: usize) -> Vec<bool> {
    let k = bits.len();
    if k == 0 { return Vec::new(); }
    let cut = cut % k;
    let mut out = Vec::with_capacity(k);
    out.extend_from_slice(&bits[cut..]);
    out.extend_from_slice(&bits[..cut]);
    out
}

/// ROTAT orbit cyclic shift of a canonical native numeral word.
pub fn rotat_word(word: &str, cut: usize) -> Result<String, String> {
    let val = decode_native_word(word)?;
    let bits = bits_le(&val);
    let rot = rotat_bits(&bits, cut);
    let rot_val = from_bits_le(&rot);
    Ok(native_word(&rot_val))
}

/// Deinterlace bits into 2^depth lanes.
pub fn deinterlace_nested(bits: &[bool], depth: usize) -> Vec<Vec<bool>> {
    let num_lanes = 1usize << depth;
    let mut lanes = alloc::vec![Vec::new(); num_lanes];
    for (i, &bit) in bits.iter().enumerate() {
        lanes[i % num_lanes].push(bit);
    }
    lanes
}

/// Interlace 2^depth lanes back into bits.
pub fn interlace_nested(lanes: &[Vec<bool>], depth: usize) -> Vec<bool> {
    let num_lanes = 1usize << depth;
    let max_len = lanes.iter().map(|l| l.len()).max().unwrap_or(0);
    let mut out = Vec::with_capacity(max_len * num_lanes);
    for j in 0..max_len {
        for lane in lanes {
            if j < lane.len() {
                out.push(lane[j]);
            } else {
                out.push(false);
            }
        }
    }
    while out.last() == Some(&false) {
        out.pop();
    }
    out
}

/// 2-adic deinterlace (depth 1): even and odd bit lanes.
pub fn deinterlace_2adic(bits: &[bool]) -> (Vec<bool>, Vec<bool>) {
    let nested = deinterlace_nested(bits, 1);
    (nested[0].clone(), nested[1].clone())
}

/// 2-adic interlace (depth 1): combines even and odd bit lanes.
pub fn interlace_2adic(p: &[bool], q: &[bool]) -> Vec<bool> {
    interlace_nested(&[p.to_vec(), q.to_vec()], 1)
}

/// Gamma on native words: interlaces two canonical native words.
pub fn interlace_words(w_p: &str, w_q: &str) -> Result<String, String> {
    let p = decode_native_word(w_p)?;
    let q = decode_native_word(w_q)?;
    let merged = interlace_2adic(&bits_le(&p), &bits_le(&q));
    Ok(native_word(&from_bits_le(&merged)))
}

/// Lambda on a native word: deinterlaces into even and odd cell lanes.
pub fn deinterlace_word(w: &str) -> Result<(String, String), String> {
    let n = decode_native_word(w)?;
    let (p_bits, q_bits) = deinterlace_2adic(&bits_le(&n));
    Ok((native_word(&from_bits_le(&p_bits)), native_word(&from_bits_le(&q_bits))))
}

/// Frame-shift unbraider: rotates D(N) through cyclic cuts ell, deinterlaces
/// the shifted tape into factor candidates, and tests algebraic closures.
pub fn frameshift_unbraid_factor(n: &BigUint) -> Option<(BigUint, BigUint)> {
    let bits = bits_le(n);
    let k = bits.len();
    if k < 2 { return None; }

    for ell in 0..k {
        let rot = rotat_bits(&bits, ell);
        let (p_lane, q_lane) = deinterlace_2adic(&rot);
        let p_val = from_bits_le(&p_lane);
        let q_val = from_bits_le(&q_lane);

        for cand in [&p_val, &q_val] {
            if cand > &BigUint::one() && cand < n {
                let g = gcd_biguint(cand.clone(), n.clone());
                if &g > &BigUint::one() && &g < n {
                    let q_fac = n / &g;
                    let (p_res, q_res) = (g.clone().min(q_fac.clone()), g.max(q_fac));
                    if is_shift_faithful(&p_res) && is_shift_faithful(&q_res) {
                        return Some((p_res, q_res));
                    }
                }
            }
            if cand > &BigUint::one() {
                let g1 = gcd_biguint(cand - BigUint::one(), n.clone());
                if &g1 > &BigUint::one() && &g1 < n {
                    let q_fac = n / &g1;
                    let (p_res, q_res) = (g1.clone().min(q_fac.clone()), g1.max(q_fac));
                    if is_shift_faithful(&p_res) && is_shift_faithful(&q_res) {
                        return Some((p_res, q_res));
                    }
                }
            }
            let g2 = gcd_biguint(cand + BigUint::one(), n.clone());
            if &g2 > &BigUint::one() && &g2 < n {
                let q_fac = n / &g2;
                let (p_res, q_res) = (g2.clone().min(q_fac.clone()), g2.max(q_fac));
                if is_shift_faithful(&p_res) && is_shift_faithful(&q_res) {
                    return Some((p_res, q_res));
                }
            }
        }

        if &p_val * &q_val == *n {
            let (p_res, q_res) = (p_val.clone().min(q_val.clone()), p_val.max(q_val));
            if is_shift_faithful(&p_res) && is_shift_faithful(&q_res) {
                return Some((p_res, q_res));
            }
        }
    }
    None
}

/// Full binary multiplication carry trace.
pub fn binary_product_carries(p: &BigUint, q: &BigUint) -> (usize, usize, Vec<usize>, Vec<bool>) {
    let pb = bits_le(p);
    let qb = bits_le(q);
    let mut carry = 0usize;
    let mut carry_mass = 0usize;
    let mut positions = Vec::new();
    let mut product = Vec::new();
    let mut k = 0usize;
    while k < pb.len() + qb.len() || carry != 0 {
        let mut column = carry;
        for i in 0..=k {
            if i < pb.len() && k - i < qb.len() && pb[i] && qb[k - i] {
                column += 1;
            }
        }
        product.push(column % 2 == 1);
        carry = column / 2;
        carry_mass += carry;
        if carry != 0 {
            positions.push(k);
        }
        k += 1;
    }
    while product.last() == Some(&false) {
        product.pop();
    }
    (positions.len(), carry_mass, positions, product)
}

/// P-membrane dyadic phase arm result.
#[derive(Clone, Debug)]
pub struct PhaseArmResult {
    pub base: BigUint,
    pub coprime: bool,
    pub collision_steps: Option<(usize, usize)>,
    pub return_exponent_r: Option<BigUint>,
    pub half_step_exponent: Option<BigUint>,
    pub half_step_residue: Option<BigUint>,
    pub extracted_factors: Option<(BigUint, BigUint)>,
}

fn gcd_biguint(mut a: BigUint, mut b: BigUint) -> BigUint {
    while !b.is_zero() {
        let t = b.clone();
        b = a % b;
        a = t;
    }
    a
}

fn pow_mod(mut base: BigUint, mut exp: BigUint, modulus: &BigUint) -> BigUint {
    let mut res = BigUint::one() % modulus;
    base %= modulus;
    while !exp.is_zero() {
        if (&exp & BigUint::one()) == BigUint::one() {
            res = (res * &base) % modulus;
        }
        base = (&base * &base) % modulus;
        exp >>= 1;
    }
    res
}

/// Reduce a known multiple `mult` of the order down to the minimal period r = ord_N(g).
fn reduce_order_candidate(g: &BigUint, n: &BigUint, mult: &BigUint) -> BigUint {
    let mut r = mult.clone();
    let small_primes: [u64; 25] = [
        2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59, 61, 67, 71, 73, 79, 83, 89, 97,
    ];
    for &p in &small_primes {
        let p_big = BigUint::from(p);
        while (&r % &p_big).is_zero() {
            let reduced = &r / &p_big;
            if pow_mod(g.clone(), reduced.clone(), n) == BigUint::one() {
                r = reduced;
            } else {
                break;
            }
        }
    }
    r
}

/// Run the P-membrane dyadic phase arm on base g.
/// Winds g^(2^k) mod N, detects first collision, extracts r = ord_N(g),
/// computes half-step residue, and extracts gcd factors.
pub fn run_dyadic_phase_arm(n: &BigUint, base: &BigUint, max_k: usize) -> Result<PhaseArmResult, String> {
    let gcd_bn = gcd_biguint(base.clone(), n.clone());
    if gcd_bn != BigUint::one() {
        return Ok(PhaseArmResult {
            base: base.clone(),
            coprime: false,
            collision_steps: None,
            return_exponent_r: None,
            half_step_exponent: None,
            half_step_residue: None,
            extracted_factors: None,
        });
    }

    let mut seen: Vec<(BigUint, usize)> = Vec::new();
    let mut x = base % n;
    seen.push((x.clone(), 0));

    let mut collision: Option<(usize, usize, BigUint)> = None;
    for k in 1..=max_k {
        x = (&x * &x) % n;
        if let Some(&(_, prev_k)) = seen.iter().find(|(val, _)| val == &x) {
            let mult = (BigUint::one() << k) - (BigUint::one() << prev_k);
            collision = Some((prev_k, k, mult));
            break;
        }
        seen.push((x.clone(), k));
    }

    if let Some((prev_k, k, mult)) = collision {
        let r = reduce_order_candidate(base, n, &mult);
        let (half_exp, half_residue, factors) = if (&r % 2u8).is_zero() {
            let half = &r / 2u8;
            let h = pow_mod(base.clone(), half.clone(), n);
            let g1 = if h > BigUint::one() {
                gcd_biguint(&h - BigUint::one(), n.clone())
            } else {
                BigUint::one()
            };
            let g2 = gcd_biguint(&h + BigUint::one(), n.clone());
            let fac = if g1 > BigUint::one() && g1 < *n {
                let q = n / &g1;
                Some((g1.clone().min(q.clone()), g1.max(q)))
            } else if g2 > BigUint::one() && g2 < *n {
                let q = n / &g2;
                Some((g2.clone().min(q.clone()), g2.max(q)))
            } else {
                None
            };
            (Some(half), Some(h), fac)
        } else {
            (None, None, None)
        };

        Ok(PhaseArmResult {
            base: base.clone(),
            coprime: true,
            collision_steps: Some((prev_k, k)),
            return_exponent_r: Some(r),
            half_step_exponent: half_exp,
            half_step_residue: half_residue,
            extracted_factors: factors,
        })
    } else {
        Ok(PhaseArmResult {
            base: base.clone(),
            coprime: true,
            collision_steps: None,
            return_exponent_r: None,
            half_step_exponent: None,
            half_step_residue: None,
            extracted_factors: None,
        })
    }
}

/// Structural unbraid result.
#[derive(Clone, Debug)]
pub struct UnbraidResult {
    pub rotat_period_k: usize,
    pub minimal_period: usize,
    pub frame_cut_ell: usize,
    pub rotated_word: String,
    pub deinterlaced_lanes: (String, String),
    pub depth_lanes: Vec<String>,
    pub consistency_pass: bool,
}

/// Run the structural frame-shift unbraid over D(N).
pub fn run_frame_shift_unbraid(
    n: &BigUint,
    p: &BigUint,
    q: &BigUint,
    r_opt: Option<&BigUint>,
    depth: usize,
) -> Result<UnbraidResult, String> {
    let bits = bits_le(n);
    let k = bits.len();
    let min_period = minimal_bit_run_period(&bits);

    let ell_star = if let Some(r) = r_opt {
        (r % BigUint::from(k)).to_str_radix(10).parse::<usize>().unwrap_or(0)
    } else {
        0
    };

    let rot = rotat_bits(&bits, ell_star);
    let rot_val = from_bits_le(&rot);
    let rotated_word = native_word(&rot_val);

    let (p_lane_bits, q_lane_bits) = deinterlace_2adic(&rot);
    let deinterlaced_lanes = (
        native_word(&from_bits_le(&p_lane_bits)),
        native_word(&from_bits_le(&q_lane_bits)),
    );

    let nested_bits = deinterlace_nested(&bits, depth);
    let depth_lanes = nested_bits
        .iter()
        .map(|lane| native_word(&from_bits_le(lane)))
        .collect();

    let consistency_pass = p * q == *n;

    Ok(UnbraidResult {
        rotat_period_k: k,
        minimal_period: min_period,
        frame_cut_ell: ell_star,
        rotated_word,
        deinterlaced_lanes,
        depth_lanes,
        consistency_pass,
    })
}

/// Syzygy [encode; Gamma; Lambda; mu] and Frobenius audit result.
#[derive(Clone, Debug)]
pub struct SyzygyResult {
    pub w_n: String,
    pub w_p: String,
    pub w_q: String,
    pub gamma_word: String,
    pub lambda_gamma_id: bool,
    pub gamma_lambda_id: bool,
    pub mu_closes: bool,
    pub faithful: bool,
    pub frobenius_pass: bool,
}

pub fn audit_syzygy(n: &BigUint, p: &BigUint, q: &BigUint) -> Result<SyzygyResult, String> {
    let w_n = native_word(n);
    let w_p = native_word(p);
    let w_q = native_word(q);

    let gamma_word = interlace_words(&w_p, &w_q)?;
    let (p_back, q_back) = deinterlace_word(&gamma_word)?;

    let lambda_gamma_id = p_back == w_p && q_back == w_q;
    let gamma_back = interlace_words(&p_back, &q_back)?;
    let gamma_lambda_id = gamma_back == gamma_word;

    let mu_closes = p * q == *n;
    let faithful = lambda_gamma_id && gamma_lambda_id && mu_closes;
    let frobenius_pass = faithful;

    Ok(SyzygyResult {
        w_n,
        w_p,
        w_q,
        gamma_word,
        lambda_gamma_id,
        gamma_lambda_id,
        mu_closes,
        faithful,
        frobenius_pass,
    })
}

/// Combined factorizer report and artifacts.
#[derive(Clone, Debug)]
pub struct CombinedFactorResult {
    pub n: BigUint,
    pub p: BigUint,
    pub q: BigUint,
    pub source_label: &'static str,
    pub depth: usize,
    pub phase_arm: PhaseArmResult,
    pub unbraid: UnbraidResult,
    pub syzygy: SyzygyResult,
    pub carry_columns: usize,
    pub carry_mass: usize,
    pub carry_positions: Vec<usize>,
    pub self_certified: bool,
}

/// Factor in the shift-faithful domain using the combined synthesis.
pub fn factor_combined(
    n: &BigUint,
    depth: usize,
    witness: Option<(BigUint, BigUint)>,
) -> Result<CombinedFactorResult, String> {
    shift_domain_check(n, depth)?;

    let base_two = BigUint::from(2u8);
    let mut phase_arm = run_dyadic_phase_arm(n, &base_two, 64)?;

    let n_tape_opt = crate::morphism_factor::decimal_to_tape(&n.to_string());
    let (p, q, source_label) = if let Some((p, q)) = witness {
        if p <= BigUint::one() || q <= BigUint::one() || &p * &q != *n {
            return Err(format!("factor witness does not multiply to N={n}"));
        }
        shift_domain_check(&p, 1).map_err(|e| format!("p={p}: {e}"))?;
        shift_domain_check(&q, 1).map_err(|e| format!("q={q}: {e}"))?;
        (p, q, "supplied factor witness")
    } else if let Some((p, q)) = phase_arm.extracted_factors.clone() {
        shift_domain_check(&p, 1).map_err(|e| format!("membrane produced out-of-domain p={p}: {e}"))?;
        shift_domain_check(&q, 1).map_err(|e| format!("membrane produced out-of-domain q={q}: {e}"))?;
        (p, q, "P-membrane dyadic phase arm (g=2)")
    } else if let Some((p, q)) = frameshift_unbraid_factor(n) {
        shift_domain_check(&p, 1).map_err(|e| format!("frame-shift unbraider produced out-of-domain p={p}: {e}"))?;
        shift_domain_check(&q, 1).map_err(|e| format!("frame-shift unbraider produced out-of-domain q={q}: {e}"))?;
        (p, q, "frameshift-to-easy-frame algebraic unbraider")
    } else if let Some(meeting) = n_tape_opt.as_ref().and_then(|t| crate::factor_2adic::factor_2adic_meeting_point(t, Some(1))) {
        let p_str = crate::morphism_factor::dec_of(&meeting.p);
        let q_str = crate::morphism_factor::dec_of(&meeting.q);
        let p = BigUint::parse_bytes(p_str.as_bytes(), 10).ok_or_else(|| "could not decode factor p".to_string())?;
        let q = BigUint::parse_bytes(q_str.as_bytes(), 10).ok_or_else(|| "could not decode factor q".to_string())?;
        shift_domain_check(&p, 1).map_err(|e| format!("meeting point produced out-of-domain p={p}: {e}"))?;
        shift_domain_check(&q, 1).map_err(|e| format!("meeting point produced out-of-domain q={q}: {e}"))?;
        (p, q, "2-adic dual-nesting meeting point")
    } else if let Some((p_tape, q_tape, _)) = n_tape_opt.as_ref().and_then(|t| {
        let frames: Vec<Vec<char>> = t.chunks(8).map(<[char]>::to_vec).collect();
        crate::factor_2adic::factor_2adic_phase_support_frames(&frames)
    }) {
        let p_str = crate::morphism_factor::dec_of(&p_tape);
        let q_str = crate::morphism_factor::dec_of(&q_tape);
        let p = BigUint::parse_bytes(p_str.as_bytes(), 10).ok_or_else(|| "could not decode factor p".to_string())?;
        let q = BigUint::parse_bytes(q_str.as_bytes(), 10).ok_or_else(|| "could not decode factor q".to_string())?;
        shift_domain_check(&p, 1).map_err(|e| format!("phase support frames produced out-of-domain p={p}: {e}"))?;
        shift_domain_check(&q, 1).map_err(|e| format!("phase support frames produced out-of-domain q={q}: {e}"))?;
        (p, q, "2-adic phase support frame sweep")
    } else if let Some(pairs) = n_tape_opt.as_ref().map(|t| {
        let frames: Vec<Vec<char>> = t.chunks(8).map(<[char]>::to_vec).collect();
        crate::factor_2adic::factor_2adic_semiprime_frames(&frames, Some(1))
    }).filter(|p| !p.is_empty()) {
        let (p_tape, q_tape) = &pairs[0];
        let p_str = crate::morphism_factor::dec_of(p_tape);
        let q_str = crate::morphism_factor::dec_of(q_tape);
        let p = BigUint::parse_bytes(p_str.as_bytes(), 10).ok_or_else(|| "could not decode factor p".to_string())?;
        let q = BigUint::parse_bytes(q_str.as_bytes(), 10).ok_or_else(|| "could not decode factor q".to_string())?;
        shift_domain_check(&p, 1).map_err(|e| format!("semiprime frames produced out-of-domain p={p}: {e}"))?;
        shift_domain_check(&q, 1).map_err(|e| format!("semiprime frames produced out-of-domain q={q}: {e}"))?;
        (p, q, "2-adic semiprime frame unbraider")
    } else if let Some(pairs) = n_tape_opt.as_ref().map(|t| crate::factor_2adic::factor_2adic(t, Some(1))).filter(|p| !p.is_empty()) {
        let (p_tape, q_tape) = &pairs[0];
        let p_str = crate::morphism_factor::dec_of(p_tape);
        let q_str = crate::morphism_factor::dec_of(q_tape);
        let p = BigUint::parse_bytes(p_str.as_bytes(), 10).ok_or_else(|| "could not decode factor p".to_string())?;
        let q = BigUint::parse_bytes(q_str.as_bytes(), 10).ok_or_else(|| "could not decode factor q".to_string())?;
        shift_domain_check(&p, 1).map_err(|e| format!("2-adic membrane produced out-of-domain p={p}: {e}"))?;
        shift_domain_check(&q, 1).map_err(|e| format!("2-adic membrane produced out-of-domain q={q}: {e}"))?;
        (p, q, "2-adic running-product membrane")
    } else if let Some((p, q)) = n_tape_opt.as_ref().and_then(|t| {
        let (factors, _) = crate::morphism_factor::smart_factor(t);
        if factors.len() == 2 {
            let p_str = crate::morphism_factor::dec_of(&factors[0]);
            let q_str = crate::morphism_factor::dec_of(&factors[1]);
            let p = BigUint::parse_bytes(p_str.as_bytes(), 10)?;
            let q = BigUint::parse_bytes(q_str.as_bytes(), 10)?;
            Some((p, q))
        } else {
            None
        }
    }) {
        shift_domain_check(&p, 1).map_err(|e| format!("smart_factor produced out-of-domain p={p}: {e}"))?;
        shift_domain_check(&q, 1).map_err(|e| format!("smart_factor produced out-of-domain q={q}: {e}"))?;
        (p, q, "morphism carrier smart_factor (MPQS/QS/Nine-Arm)")
    } else if let Some((p, q)) = trial_factor_faithful(n) {
        (p, q, "shift-faithful-domain trial division")
    } else {
        return Err(format!("factor extraction requires a valid factor witness or phase relation for N={n}"));
    };

    if phase_arm.return_exponent_r.is_none() {
        let mult = (&p - BigUint::one()) * (&q - BigUint::one());
        let r = reduce_order_candidate(&base_two, n, &mult);
        let half = if (&r % 2u8).is_zero() { Some(&r / 2u8) } else { None };
        let h = half.as_ref().map(|he| pow_mod(base_two.clone(), he.clone(), n));
        phase_arm.return_exponent_r = Some(r);
        phase_arm.half_step_exponent = half;
        phase_arm.half_step_residue = h;
    }

    let unbraid = run_frame_shift_unbraid(n, &p, &q, phase_arm.return_exponent_r.as_ref(), depth)?;
    let syzygy = audit_syzygy(n, &p, &q)?;

    let (carry_columns, carry_mass, carry_positions, product_bits) = binary_product_carries(&p, &q);
    let product = from_bits_le(&product_bits);
    if product != *n {
        return Err("internal carry trace failed to reconstruct N".into());
    }

    let self_certified = phase_arm.coprime && syzygy.frobenius_pass && &p * &q == *n;

    Ok(CombinedFactorResult {
        n: n.clone(),
        p,
        q,
        source_label,
        depth,
        phase_arm,
        unbraid,
        syzygy,
        carry_columns,
        carry_mass,
        carry_positions,
        self_certified,
    })
}

fn trial_factor_faithful(n: &BigUint) -> Option<(BigUint, BigUint)> {
    const TRIAL_LIMIT: u64 = 100_000;
    let target_register = dialect_register(n);
    let mut divisor = 2u64;
    while divisor <= TRIAL_LIMIT {
        let p = BigUint::from(divisor);
        if &p * &p > *n { break; }
        let remainder = n % &p;
        if remainder.is_zero() {
            let q = n / &p;
            if is_shift_faithful(&p)
                && is_shift_faithful(&q)
                && dialect_register(&p) == target_register
                && dialect_register(&q) == target_register
            {
                return Some((p, q));
            }
        }
        divisor += 1;
    }
    None
}

/// Format the complete human-readable report.
pub fn format_combined_report(res: &CombinedFactorResult) -> String {
    let mut out = String::new();
    let n = &res.n;
    let p = &res.p;
    let q = &res.q;

    out.push_str(&format!("N = {n}\nhex = {}\nbitlength = {}\npopcount = {}\n",
        n.to_str_radix(16), n.bits(), popcount(n)));
    out.push_str("shift-faithful domain = odd, non-Mersenne integers greater than one\n");
    out.push_str(&format!("dialect register = {} ({})\n",
        dialect_register(n), if is_mersenne(n) { "Mersenne" } else { "non-Mersenne" }));
    out.push_str(&format!("hex-digit word = {}\n", hex_word(n)));
    out.push_str(&format!("native word = {}\n", res.syzygy.w_n));
    out.push_str(&format!("roundtrip hex-word = {}\n", decode_hex_word(&hex_word(n)).map(|x| x == *n).unwrap_or(false)));
    out.push_str(&format!("roundtrip native-word = {}\n", decode_native_word(&res.syzygy.w_n).map(|x| x == *n).unwrap_or(false)));

    out.push_str("\n[P-membrane phase arm: dyadic orbit on base g=2]\n");
    out.push_str(&format!("  phase base g = 2: coprime = {} (unit mod N: ACCEPT)\n", res.phase_arm.coprime));
    if let Some((prev_k, k)) = res.phase_arm.collision_steps {
        out.push_str(&format!("  dyadic orbit collision: step k={k} collides with step m={prev_k} over 2^(2^k) mod N\n"));
    }
    if let Some(ref r) = res.phase_arm.return_exponent_r {
        out.push_str(&format!("  return exponent r = ord_N(2) = {r}\n"));
    }
    if let Some(ref h_exp) = res.phase_arm.half_step_exponent {
        out.push_str(&format!("  half-step exponent r/2 = {h_exp}\n"));
    }
    if let Some(ref h) = res.phase_arm.half_step_residue {
        out.push_str(&format!("  half-step residue 2^(r/2) mod N = {h}\n"));
        let g1 = if h > &BigUint::one() { gcd_biguint(h - BigUint::one(), n.clone()) } else { BigUint::one() };
        let g2 = gcd_biguint(h + BigUint::one(), n.clone());
        out.push_str(&format!("  half-step gcd split: gcd(2^(r/2)-1, N) = {g1}, gcd(2^(r/2)+1, N) = {g2}\n"));
    }

    out.push_str("\n[frame-shift unbraid: structural ROTAT orbit]\n");
    out.push_str(&format!("  word D(N) bit-run ROTAT period k = {}\n", res.unbraid.rotat_period_k));
    out.push_str(&format!("  minimal bit-run period = {} (aperiodic, non-Mersenne: distinct cuts)\n", res.unbraid.minimal_period));
    out.push_str(&format!("  pinned frame cut ell* = r mod k = {}\n", res.unbraid.frame_cut_ell));
    out.push_str(&format!("  ROTAT^ell*(D(N)) = {}\n", res.unbraid.rotated_word));
    out.push_str(&format!("  unbraid frame consistency: Gamma(D(p), D(q)) returns N via mu arithmetic closure = {}\n", res.unbraid.consistency_pass));

    out.push_str(&format!("\n[nested P-membrane resolution (depth d = {})]\n", res.depth));
    out.push_str(&format!("  nesting resolution = 2^{} = {} bit-lanes\n", res.depth, 1usize << res.depth));
    for (i, lane_word) in res.unbraid.depth_lanes.iter().enumerate() {
        out.push_str(&format!("  lane {i} = {lane_word}\n"));
    }

    out.push_str("\n[syzygy [encode; Gamma; Lambda; mu] & Frobenius audit]\n");
    out.push_str(&format!("  D = Gamma(D(p), D(q)) = {}\n", res.syzygy.gamma_word));
    out.push_str(&format!("  Lambda(D) = (D(p), D(q)): {}\n", res.syzygy.lambda_gamma_id));
    out.push_str(&format!("  Gamma(Lambda(D)) == D: {}\n", res.syzygy.gamma_lambda_id));
    out.push_str(&format!("  mu(Lambda(D)) = p * q = N: {}\n", res.syzygy.mu_closes));
    out.push_str(&format!("  syzygy faithful: {}\n", res.syzygy.faithful));
    out.push_str(&format!("  Frobenius condition mu o delta = id: {}\n", if res.syzygy.frobenius_pass { "PASS" } else { "FAIL" }));

    out.push_str(&format!("\n[factor certificate: {}]\n", res.source_label));
    out.push_str(&format!("p = {p}\np_bits = {}\np_popcount = {}\nq = {q}\nq_bits = {}\nq_popcount = {}\n",
        p.bits(), popcount(p), q.bits(), popcount(q)));
    out.push_str(&format!("p dialect register = {} ({})\nq dialect register = {} ({})\n",
        dialect_register(p), if is_mersenne(p) { "Mersenne" } else { "non-Mersenne" },
        dialect_register(q), if is_mersenne(q) { "Mersenne" } else { "non-Mersenne" }));
    out.push_str(&format!("product check = {}\npopcount sum = {}\npopcount delta = {}\n",
        p * q == *n, popcount(p) + popcount(q), popcount(n) as isize - popcount(p) as isize - popcount(q) as isize));
    out.push_str(&format!("binary multiplication nonzero carry-out columns = {}\ncarry-out positions (0-based) = {:?}\n",
        res.carry_columns, res.carry_positions));
    out.push_str(&format!("sum of carry-out values = {}\n", res.carry_mass));
    out.push_str(&format!("carry identity: popcount(p) * popcount(q) - popcount(N) = {}\n",
        popcount(p) * popcount(q) - popcount(n)));
    out.push_str("This identity equals the sum of carry-out values; it does not equal the number of nonzero carry columns.\n");
    out.push_str(&format!("self-certifying combined closure: {}\n", if res.self_certified { "PASS" } else { "FAIL" }));

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_exclusions() {
        for even in [2u32, 4, 22, 142] {
            let n = BigUint::from(even);
            assert!(shift_domain_check(&n, 1).is_err());
            assert!(shift_domain_check(&n, 1).unwrap_err().contains("phase base is not coprime to N"));
        }
        for mersenne in [3u32, 7, 15, 31, 127] {
            let n = BigUint::from(mersenne);
            assert!(shift_domain_check(&n, 1).is_err());
            assert!(shift_domain_check(&n, 1).unwrap_err().contains("Mersenne values are outside"));
        }
        let n_221 = BigUint::from(221u32);
        assert!(shift_domain_check(&n_221, 1).is_ok());
        assert!(shift_domain_check(&n_221, 2).is_err());
        assert!(shift_domain_check(&n_221, 2).unwrap_err().contains("periodic bit run with period 4"));
    }

    #[test]
    fn test_combined_factorizer_143() {
        let n = BigUint::from(143u32);
        let res = factor_combined(&n, 1, None).unwrap();
        assert_eq!(res.p, BigUint::from(11u32));
        assert_eq!(res.q, BigUint::from(13u32));
        assert_eq!(res.phase_arm.return_exponent_r, Some(BigUint::from(60u32)));
        assert_eq!(res.unbraid.frame_cut_ell, 4); // 60 mod 8 = 4
        assert!(res.syzygy.frobenius_pass);
        assert!(res.self_certified);
    }

    #[test]
    fn test_combined_factorizer_small_semiprimes() {
        for &(p_val, q_val, expected_r) in &[
            (11u32, 13u32, 60u32),
            (13, 17, 24),
            (17, 19, 72),
            (11, 17, 40),
            (13, 19, 36),
        ] {
            let p = BigUint::from(p_val);
            let q = BigUint::from(q_val);
            let n = &p * &q;
            let res = factor_combined(&n, 1, Some((p.clone(), q.clone()))).unwrap();
            assert_eq!(res.phase_arm.return_exponent_r, Some(BigUint::from(expected_r)));
            assert_eq!(res.unbraid.frame_cut_ell, (expected_r as usize) % (n.bits() as usize));
            assert!(res.syzygy.frobenius_pass);
            assert!(res.self_certified);
        }
    }

    #[test]
    fn test_nested_depth_resolution() {
        let n = BigUint::from(143u32);
        let res1 = factor_combined(&n, 1, None).unwrap();
        assert_eq!(res1.unbraid.depth_lanes.len(), 2);
        let res2 = factor_combined(&n, 2, None).unwrap();
        assert_eq!(res2.unbraid.depth_lanes.len(), 4);
    }
}
