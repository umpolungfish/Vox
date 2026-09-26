//! Decode membrane numeral encodings, factor arbitrary inputs with Vox, and
//! verify the product and binary carry trace. The standalone binary adds the
//! canonical g-mOMonadOS per-value trilattice readings at runtime.

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use num_bigint::BigUint;
use num_traits::{One, Zero};

/// Canonical digit words from g-mOMonadOS `prime_winding::HEX_WORDS`.
const HEX_WORDS: [&str; 16] = [
    "⊢≻⋈≺⊙⊡⊣", "⊢⊤≻⋈≺⊙⊡⊣", "⊢≻⋈⊥≺⊙⊡⊣", "⊢⊤≻⋈⊥≺⊙⊡⊣",
    "⊢≻⋈≺⊞⊙⊡⊣", "⊢⊤≻⋈≺⊞⊙⊡⊣", "⊢≻⋈⊥≺⊞⊙⊡⊣", "⊢⊤≻⋈⊥≺⊞⊙⊡⊣",
    "⊢∈≻⋈≺⋈∋⊙⊡⊣", "⊢∈⊤≻⋈≺⋈∋⊙⊡⊣", "⊢∈≻⋈⊥≺⋈∋⊙⊡⊣", "⊢∈⊤≻⋈⊥≺⋈∋⊙⊡⊣",
    "⊢∈≻⋈≺⋈⊞∋⊙⊡⊣", "⊢∈⊤≻⋈≺⋈⊞∋⊙⊡⊣", "⊢∈≻⋈⊥≺⋈⊞∋⊙⊡⊣", "⊢∈⊤≻⋈⊥≺⋈⊞∋⊙⊡⊣",
];

fn hex_word(n: &BigUint) -> String {
    let mut out = String::new();
    for c in n.to_str_radix(16).chars() {
        if let Some(d) = c.to_digit(16) { out.push_str(HEX_WORDS[d as usize]); }
    }
    out
}

fn decode_hex_word(word: &str) -> Result<BigUint, String> {
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

fn native_word(n: &BigUint) -> String {
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

fn decode_native_word(word: &str) -> Result<BigUint, String> {
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

fn bit_string(n: &BigUint) -> String { n.to_str_radix(2) }

fn popcount(n: &BigUint) -> usize { bit_string(n).bytes().filter(|b| *b == b'1').count() }

fn is_mersenne(n: &BigUint) -> bool {
    !n.is_zero() && n.to_str_radix(2).bytes().all(|bit| bit == b'1')
}

fn dialect_register(n: &BigUint) -> &'static str {
    if is_mersenne(n) { "001000011100" } else { "111111111111" }
}

/// Search non-Mersenne divisor candidates whose registers match N. Mersenne
/// values are excluded from this candidate space.
fn register_filtered_trial_factor(n: &BigUint) -> Option<(BigUint, BigUint)> {
    const TRIAL_LIMIT: u64 = 100_000;
    let target_register = dialect_register(n);
    let mut divisor = 2u64;
    while divisor <= TRIAL_LIMIT {
        let p = BigUint::from(divisor);
        if &p * &p > n.clone() { break; }
        let remainder = n % &p;
        if remainder.is_zero() {
            let q = n / &p;
            if !is_mersenne(&p)
                && !is_mersenne(&q)
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

/// Return (nonzero carry-out columns, sum of carry-out values, positions,
/// output bits LSB-first). Carry values can exceed one in multiplication.
fn binary_product_carries(p: &BigUint, q: &BigUint) -> (usize, usize, Vec<usize>, Vec<bool>) {
    let pb = bit_string(p).bytes().rev().map(|b| b == b'1').collect::<Vec<_>>();
    let qb = bit_string(q).bytes().rev().map(|b| b == b'1').collect::<Vec<_>>();
    let mut carry = 0usize;
    let mut carry_mass = 0usize;
    let mut positions = Vec::new();
    let mut product = Vec::new();
    let mut k = 0usize;
    while k < pb.len() + qb.len() || carry != 0 {
        let mut column = carry;
        for i in 0..=k {
            if i < pb.len() && k - i < qb.len() && pb[i] && qb[k-i] { column += 1; }
        }
        product.push(column % 2 == 1);
        carry = column / 2;
        carry_mass += carry;
        if carry != 0 { positions.push(k); }
        k += 1;
    }
    (positions.len(), carry_mass, positions, product)
}

fn from_lsb_bits(bits: &[bool]) -> BigUint {
    bits.iter().enumerate().fold(BigUint::zero(), |mut n, (i, bit)| {
        if *bit { n |= BigUint::one() << i; }
        n
    })
}

fn decode_integer(text: &str) -> Result<BigUint, String> {
    if let Some(hex) = text.strip_prefix("0x").or_else(|| text.strip_prefix("0X")) {
        return BigUint::parse_bytes(hex.as_bytes(), 16).ok_or_else(|| "invalid hexadecimal integer".into());
    }
    BigUint::parse_bytes(text.as_bytes(), 10).ok_or_else(|| "expected a non-negative decimal integer or 0x-prefixed hex".into())
}

fn run(args: &[String]) -> Result<String, String> {
    if args.is_empty() || args.iter().any(|a| a == "--help" || a == "-h") {
        return Ok("vox factor-membrane <decimal|0xHEX|--hex-word WORD|--native-word WORD> [--factors P Q]\n\
             Decodes and factors non-Mersenne semiprimes whose factors are non-Mersenne.\n\
             Register filter: 111111111111 for non-Mersenne values; Mersenne values are excluded.\n\
             The standalone binary reads N and its factors with g-momonados trilattice_factor read.\n\
             Set VOX_TRILATTICE_FACTOR when g-momonados is not on PATH.\n".into());
    }
    let mut source: Option<BigUint> = None;
    let mut factor_pair: Option<(BigUint, BigUint)> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--hex-word" | "--native-word" => {
                let flag = args[i].as_str();
                let value = args.get(i+1).ok_or_else(|| format!("{flag} requires a word"))?;
                if source.is_some() { return Err("provide exactly one integer source".into()); }
                source = Some(if flag == "--hex-word" { decode_hex_word(value)? } else { decode_native_word(value)? });
                i += 2;
            }
            "--factors" => {
                if factor_pair.is_some() { return Err("--factors was provided more than once".into()); }
                let p = args.get(i+1).ok_or_else(|| "--factors requires P and Q".to_string())?;
                let q = args.get(i+2).ok_or_else(|| "--factors requires P and Q".to_string())?;
                factor_pair = Some((decode_integer(p)?, decode_integer(q)?));
                i += 3;
            }
            value if value.starts_with('-') => return Err(format!("unknown option {value}")),
            value => {
                if source.is_some() { return Err("provide exactly one integer source".into()); }
                source = Some(decode_integer(value)?);
                i += 1;
            }
        }
    }
    let n = source.ok_or_else(|| "missing integer source".to_string())?;
    if n <= BigUint::one() { return Err("a semiprime modulus must be greater than one".into()); }
    let n_is_mersenne = is_mersenne(&n);
    if n_is_mersenne {
        return Err("Mersenne values are excluded by the dialect-register candidate filter".into());
    }
    let n_register = dialect_register(&n);

    let (p, q, source_label) = if let Some((p, q)) = factor_pair {
        if p <= BigUint::one() || q <= BigUint::one() || &p * &q != n {
            return Err(format!("factor witness does not multiply to N={n}"));
        }
        (p, q, "supplied factor witness")
    } else if let Some((p, q)) = register_filtered_trial_factor(&n) {
        (p, q, "dialect-register filtered candidate search")
    } else {
        let tape = crate::morphism_factor::decimal_to_tape(&n.to_string()).ok_or_else(|| "could not encode N for Vox factorizer".to_string())?;
        let (factors, _log) = crate::morphism_factor::smart_factor(&tape);
        if factors.len() != 2 { return Err(format!("Vox returned {} prime factors; expected a semiprime pair", factors.len())); }
        let a = decode_integer(&crate::morphism_factor::dec_of(&factors[0]))?;
        let b = decode_integer(&crate::morphism_factor::dec_of(&factors[1]))?;
        (a, b, "Vox smart factorizer")
    };
    if is_mersenne(&p) || is_mersenne(&q) {
        return Err("Mersenne factors are excluded by the dialect-register candidate filter".into());
    }

    let (carry_columns, carry_mass, carry_positions, product_bits) = binary_product_carries(&p, &q);
    let product = from_lsb_bits(&product_bits);
    if product != n { return Err("internal carry trace failed to reconstruct N".into()); }
    let mut out = String::new();
    out.push_str(&format!("N = {n}\nhex = {}\nbitlength = {}\npopcount = {}\n", n.to_str_radix(16), n.bits(), popcount(&n)));
    out.push_str(&format!("dialect register = {n_register} ({})\n", if n_is_mersenne { "Mersenne" } else { "non-Mersenne" }));
    out.push_str(&format!("hex-digit word = {}\n", hex_word(&n)));
    out.push_str(&format!("native word = {}\n", native_word(&n)));
    out.push_str(&format!("roundtrip hex-word = {}\n", decode_hex_word(&hex_word(&n))? == n));
    out.push_str(&format!("roundtrip native-word = {}\n", decode_native_word(&native_word(&n))? == n));
    out.push_str(&format!("factor source = {source_label}\np = {p}\np_bits = {}\np_popcount = {}\nq = {q}\nq_bits = {}\nq_popcount = {}\n",
        p.bits(), popcount(&p), q.bits(), popcount(&q)));
    out.push_str(&format!("p dialect register = {} ({})\nq dialect register = {} ({})\n",
        dialect_register(&p), if is_mersenne(&p) { "Mersenne" } else { "non-Mersenne" },
        dialect_register(&q), if is_mersenne(&q) { "Mersenne" } else { "non-Mersenne" }));
    out.push_str(&format!("product check = {}\npopcount sum = {}\npopcount delta = {}\n",
        &p * &q == n, popcount(&p) + popcount(&q), popcount(&n) as isize - popcount(&p) as isize - popcount(&q) as isize));
    out.push_str(&format!("binary multiplication nonzero carry-out columns = {carry_columns}\ncarry-out positions (0-based) = {carry_positions:?}\n"));
    out.push_str(&format!("sum of carry-out values = {carry_mass}\n"));
    out.push_str(&format!("carry identity: popcount(p) * popcount(q) - popcount(N) = {}\n",
        popcount(&p) * popcount(&q) - popcount(&n)));
    out.push_str("This identity equals the sum of carry-out values; it does not equal the number of nonzero carry columns.\n");
    Ok(out)
}

/// CLI entrypoint shared by `vox factor-membrane` and its standalone binary.
pub fn command(args: &[String]) -> Result<String, String> { run(args) }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_words_roundtrip_and_popcount_delta_is_not_carry_count() {
        let n = decode_integer("1522605027922533360535618378132637429718068114961380688657908494580122963258952897654000350692006139").unwrap();
        let p = decode_integer("37975227936943673922808872755445627854565536638199").unwrap();
        let q = decode_integer("40094690950920881030683735292761468389214899724061").unwrap();
        assert_eq!(decode_hex_word(&hex_word(&n)).unwrap(), n);
        assert_eq!(decode_native_word(&native_word(&n)).unwrap(), n);
        assert_eq!((popcount(&n), popcount(&p), popcount(&q)), (186, 87, 82));
        let (columns, carry_mass, _, _) = binary_product_carries(&p, &q);
        assert_eq!(columns, 327);
        assert_eq!(carry_mass, 6948);
        assert_eq!(popcount(&p) * popcount(&q) - popcount(&n), carry_mass);
        assert_eq!(popcount(&n) - popcount(&p) - popcount(&q), 17);
    }

    #[test]
    fn carry_mass_identity_holds_for_small_products() {
        for p in 2u32..40 {
            for q in 2u32..40 {
                let p = BigUint::from(p);
                let q = BigUint::from(q);
                let n = &p * &q;
                let (columns, carry_mass, _, bits) = binary_product_carries(&p, &q);
                assert_eq!(from_lsb_bits(&bits), n);
                assert!(carry_mass >= columns);
                assert_eq!(carry_mass, popcount(&p) * popcount(&q) - popcount(&n));
            }
        }
    }

    #[test]
    fn register_filter_excludes_mersenne_values_and_factors() {
        let n = BigUint::from(21u32);
        assert!(register_filtered_trial_factor(&n).is_none());
        assert_eq!(dialect_register(&n), "111111111111");
        assert_eq!(dialect_register(&BigUint::from(3u32)), "001000011100");
        assert_eq!(dialect_register(&BigUint::from(7u32)), "001000011100");

        let n = BigUint::from(143u32);
        let (p, q) = register_filtered_trial_factor(&n).unwrap();
        assert_eq!(&p * &q, n);
        assert_eq!(dialect_register(&p), dialect_register(&n));
        assert_eq!(dialect_register(&q), dialect_register(&n));
        assert!(!is_mersenne(&p));
        assert!(!is_mersenne(&q));
    }
}
