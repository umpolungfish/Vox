use alloc::vec;

use crate::factor_extract::{extract, FactorCarrier};
use crate::hadamard_factor_bridge::HadamardDescent;
use crate::hadamard_gate::{HadamardCarrier, Tape};
use crate::morphism_factor::{add, divmod, mul, tape_u64, trim};
use crate::vox::{EVALF, EVALT};

struct SemiprimeCase {
    bits: usize,
    n: &'static str,
    period: &'static str,
    p: &'static str,
    q: &'static str,
}

fn decimal_tape(text: &str) -> Tape {
    assert!(!text.is_empty());
    let ten = tape_u64(10);
    let mut value = vec![EVALT];
    for byte in text.bytes() {
        assert!(byte.is_ascii_digit());
        value = add(
            &mul(&value, &ten),
            &tape_u64(u64::from(byte - b'0')),
        );
    }
    trim(value)
}

fn power_of_two_tape(exponent: usize) -> Tape {
    let mut tape = vec![EVALT; exponent + 1];
    tape[exponent] = EVALF;
    tape
}

/// Build one idealized s=1 Fourier read near 1/r for the regression oracle.
/// The period is used only here, in the test harness, to synthesize k/M.  The
/// production carrier receives only N, base=2, k, and M; p/q and r are not
/// resident production state.
fn phase_sample(period: &[char], n_bits: usize) -> (Tape, Tape) {
    // M > 4*N^2, hence M > 2*r^2 because a useful multiplicative order r < N.
    // Therefore floor(M/r)/M is close enough that r is a continued-fraction
    // convergent of this one sample.
    let m = power_of_two_tape(2 * n_bits + 2);
    let (k, _) = divmod(&m, period);
    (k, m)
}

fn extracted_pair(carrier: FactorCarrier) -> (Tape, Tape) {
    let readout = extract(&carrier).unwrap();
    assert_eq!(readout.transforms, 0);
    (readout.p.0, readout.q.0)
}

fn run_case(case: &SemiprimeCase) {
    let n = decimal_tape(case.n);
    assert_eq!(n.len(), case.bits, "N={} bit width", case.n);

    let period = decimal_tape(case.period);
    let (k, m) = phase_sample(&period, case.bits);

    // Production starts from N only.  The phase read carries no p/q witness.
    let seed = HadamardCarrier::new(&n).unwrap();
    assert_eq!(seed.n(), n.as_slice());
    assert_eq!(seed.source(), [EVALT]);

    let factor_carrier = match seed.descend_phase_sample(&tape_u64(2), &k, &m) {
        HadamardDescent::T(carrier) => carrier,
        other => panic!(
            "{}-bit semiprime failed to close from one phase sample: {other:?}",
            case.bits
        ),
    };

    assert_eq!(factor_carrier.n, n);
    let (got_p, got_q) = extracted_pair(factor_carrier);

    // The factor oracle is consulted only after T has produced the
    // factor-bearing limiting carrier.
    let expected_p = decimal_tape(case.p);
    let expected_q = decimal_tape(case.q);
    let direct = got_p == expected_p && got_q == expected_q;
    let swapped = got_p == expected_q && got_q == expected_p;
    assert!(direct || swapped, "{}-bit factor pair mismatch", case.bits);
}

#[test]
fn one_phase_sample_scales_from_31_through_64_bit_semiprimes() {
    let cases = [
        SemiprimeCase {
            bits: 31,
            n: "2031940837",
            period: "112880502",
            p: "54559",
            q: "37243",
        },
        SemiprimeCase {
            bits: 40,
            n: "949112279051",
            period: "31637011020",
            p: "972221",
            q: "976231",
        },
        SemiprimeCase {
            bits: 47,
            n: "114360513180839",
            period: "14295061397682",
            p: "8424287",
            q: "13575097",
        },
        SemiprimeCase {
            bits: 56,
            n: "48787819824905957",
            period: "1219695484526904",
            p: "243325237",
            q: "200504561",
        },
        SemiprimeCase {
            bits: 60,
            n: "813080171761801367",
            period: "2746892466075366",
            p: "893585183",
            q: "909907849",
        },
        SemiprimeCase {
            bits: 64,
            n: "14672336402354228699",
            period: "7336168197340556268",
            p: "4052987087",
            q: "3620129077",
        },
    ];

    for case in &cases {
        run_case(case);
    }
}

#[test]
fn one_phase_sample_crosses_the_host_word_and_reaches_128_bit_semiprimes() {
    let cases = [
        SemiprimeCase {
            bits: 80,
            n: "966797818948726127655223",
            period: "8334463956437499719120",
            p: "1085571153761",
            q: "890589083543",
        },
        SemiprimeCase {
            bits: 96,
            n: "44183798750402948400038412281",
            period: "2761487421900157966178043552",
            p: "221151070004537",
            q: "199790119710913",
        },
        SemiprimeCase {
            bits: 112,
            n: "3924631727675843096135905777198787",
            period: "654105287945973828429353931842276",
            p: "58695408508679653",
            q: "66864373677465479",
        },
        SemiprimeCase {
            bits: 128,
            n: "182342945995603068530746742010183178459",
            period: "5698217062362595890737663640694859090",
            p: "12220795601362844411",
            q: "14920709906584843169",
        },
    ];

    for case in &cases {
        run_case(case);
    }
}
