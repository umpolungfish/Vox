//! Phase-sample descent battery at decimal widths 15, 25, 50, 100, 150.
//!
//! Each N is a balanced semiprime built with a known base-2 order r. One ideal
//! s=1 Fourier sample k/M near 1/r is synthesised in the harness (the step a
//! coherent device produces), and the production descent recovers the factors
//! from N, base=2, k, M alone, with p, q and r not resident.

use alloc::vec;

use crate::factor_extract::{extract, FactorCarrier};
use crate::hadamard_factor_bridge::HadamardDescent;
use crate::hadamard_gate::{HadamardCarrier, Tape};
use crate::morphism_factor::{add, divmod, mul, tape_u64, trim};
use crate::vox::{EVALF, EVALT};

struct Case {
    digits: usize,
    n: &'static str,
    period: &'static str,
    p: &'static str,
    q: &'static str,
}

const CASES: [Case; 5] = [
    Case {
        digits: 15,
        n: "535021934776789",
        period: "267510942859478",
        p: "32692607",
        q: "16365227",
    },
    Case {
        digits: 25,
        n: "6164313165635958886545991",
        period: "3082156582814445211227092",
        p: "6049482130619",
        q: "1018981961189",
    },
    Case {
        digits: 50,
        n: "48349723036530870075922973585054479735474630204621",
        period: "24174861518265435037961470424417365296595408847726",
        p: "31185845795740153371181487",
        q: "1550373953402130441327683",
    },
    Case {
        digits: 100,
        n: "2621677406169137542116755930796524919971216394555870076393784105824084536152180455164178508020025773",
        period: "1310838703084568771058377965398262459985608197277863411520953370269462827290054084173996614111393390",
        p: "121713633321377805403070634689475034409898529523891",
        q: "21539718555987479755810937382811781775381267715103",
    },
    Case {
        digits: 150,
        n: "554953800860125839687764366199008874604645458724200340390218982684892436881122631618427078936583512385340780427289285215277179386583677155054332646247",
        period: "1616743871477465199787225687680285953272636178235942888910891009820402491679280533644380445216665782234918902579134026621896939657485564353202604",
        p: "1959686305250025190056098194447159804895806920815017798984126035487036926613",
        q: "283185017608888404126017775628767636495407120389549462409001213073089086219",
    },
];

fn decimal_tape(text: &str) -> Tape {
    let ten = tape_u64(10);
    let mut value = vec![EVALT];
    for byte in text.bytes() {
        assert!(byte.is_ascii_digit());
        value = add(&mul(&value, &ten), &tape_u64(u64::from(byte - b'0')));
    }
    trim(value)
}

fn power_of_two_tape(exponent: usize) -> Tape {
    let mut tape = vec![EVALT; exponent + 1];
    tape[exponent] = EVALF;
    tape
}

fn extracted_pair(carrier: FactorCarrier) -> (Tape, Tape) {
    let readout = extract(&carrier).unwrap();
    assert_eq!(readout.transforms, 0);
    (readout.p.0, readout.q.0)
}

fn run_case(case: &Case) {
    let n = decimal_tape(case.n);
    let bits = n.len();
    let period = decimal_tape(case.period);

    // one ideal s=1 sample k/M near 1/r; M = 2^(2*bits+2).
    let m = power_of_two_tape(2 * bits + 2);
    let (k, _) = divmod(&m, &period);

    let seed = HadamardCarrier::new(&n).unwrap();
    let carrier = match seed.descend_phase_sample(&tape_u64(2), &k, &m) {
        HadamardDescent::T(carrier) => carrier,
        other => panic!("{}-digit semiprime did not close from one sample: {other:?}", case.digits),
    };
    assert_eq!(carrier.n, n);

    let (got_p, got_q) = extracted_pair(carrier);
    let (ep, eq) = (decimal_tape(case.p), decimal_tape(case.q));
    let direct = got_p == ep && got_q == eq;
    let swapped = got_p == eq && got_q == ep;
    assert!(direct || swapped, "{}-digit factor pair mismatch", case.digits);
}

#[test]
fn battery_15_digits() {
    run_case(&CASES[0]);
}

#[test]
fn battery_25_digits() {
    run_case(&CASES[1]);
}

#[test]
fn battery_50_digits() {
    run_case(&CASES[2]);
}

#[test]
fn battery_100_digits() {
    run_case(&CASES[3]);
}

#[test]
fn battery_150_digits() {
    run_case(&CASES[4]);
}
