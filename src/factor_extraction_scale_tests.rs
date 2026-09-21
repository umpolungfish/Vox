use alloc::vec;

use crate::morphism_factor::{add, cmp, miller_rabin, mul, one, smart_factor, tape_u64, trim};
use crate::vox::{EVALF, EVALT};

#[derive(Clone, Copy)]
struct ExtractionCase {
    bits: usize,
    n: &'static str,
}

// These fixtures contain N only.  No factor, period, phase sample, or other
// factor-derived witness is resident in the test input.
const CASES: [ExtractionCase; 11] = [
    ExtractionCase { bits: 31, n: "2031940837" },
    ExtractionCase { bits: 40, n: "949112279051" },
    ExtractionCase { bits: 47, n: "114360513180839" },
    ExtractionCase { bits: 56, n: "48787819824905957" },
    ExtractionCase { bits: 60, n: "813080171761801367" },
    ExtractionCase { bits: 64, n: "14672336402354228699" },
    ExtractionCase { bits: 80, n: "966797818948726127655223" },
    ExtractionCase { bits: 96, n: "44183798750402948400038412281" },
    ExtractionCase { bits: 112, n: "3924631727675843096135905777198787" },
    ExtractionCase { bits: 128, n: "182342945995603068530746742010183178459" },
    ExtractionCase { bits: 144, n: "22300745198530620650629069553396669406425677" },
];

fn decimal_tape(text: &str) -> Vec<char> {
    assert!(!text.is_empty());
    let ten = tape_u64(10);
    let mut value = vec![EVALT];
    for byte in text.bytes() {
        assert!(byte.is_ascii_digit());
        value = add(&mul(&value, &ten), &tape_u64(u64::from(byte - b'0')));
    }
    trim(value)
}

fn assert_extracts(case: ExtractionCase) {
    let n = decimal_tape(case.n);
    assert_eq!(n.len(), case.bits, "N bit width");

    // This is the contract under test: factorization receives N and nothing
    // else.  The returned factors must establish their own correctness.
    let (factors, route) = smart_factor(&n);
    assert_eq!(
        factors.len(),
        2,
        "{}-bit N did not extract exactly two factors; route:\n{}",
        case.bits,
        route,
    );

    let p = &factors[0];
    let q = &factors[1];
    assert!(cmp(p, &one()).is_gt() && cmp(p, &n).is_lt());
    assert!(cmp(q, &one()).is_gt() && cmp(q, &n).is_lt());
    assert!(miller_rabin(p), "{}-bit left output is not prime", case.bits);
    assert!(miller_rabin(q), "{}-bit right output is not prime", case.bits);
    assert_eq!(mul(p, q), n, "{}-bit outputs do not reconstruct N", case.bits);
}

#[test]
fn n_only_extracts_31_through_64_bit_semiprimes() {
    for &case in &CASES[..6] {
        assert_extracts(case);
    }
}

#[test]
fn n_only_extracts_80_bit_semiprime() {
    assert_extracts(CASES[6]);
}

#[test]
fn n_only_extracts_96_bit_semiprime() {
    assert_extracts(CASES[7]);
}

#[test]
fn n_only_extracts_112_bit_semiprime() {
    assert_extracts(CASES[8]);
}

#[test]
fn n_only_extracts_128_bit_semiprime() {
    assert_extracts(CASES[9]);
}

#[test]
fn n_only_extracts_144_bit_semiprime() {
    assert_extracts(CASES[10]);
}
