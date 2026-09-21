use alloc::vec;

use crate::factor_extract::{extract, FactorCarrier};
use crate::hadamard_factor_bridge::HadamardDescent;
use crate::hadamard_gate::{HadamardCarrier, Tape};
use crate::morphism_factor::{add, divmod, mul, tape_u64, trim};
use crate::vox::{EVALF, EVALT};

#[derive(Clone, Copy)]
struct SemiprimeCase {
    bits: usize,
    n: &'static str,
    period: &'static str,
    p: &'static str,
    q: &'static str,
}

const CASES: [SemiprimeCase; 20] = [
    SemiprimeCase { bits: 31, n: "2031940837", period: "112880502", p: "54559", q: "37243" },
    SemiprimeCase { bits: 40, n: "949112279051", period: "31637011020", p: "972221", q: "976231" },
    SemiprimeCase { bits: 47, n: "114360513180839", period: "14295061397682", p: "8424287", q: "13575097" },
    SemiprimeCase { bits: 56, n: "48787819824905957", period: "1219695484526904", p: "243325237", q: "200504561" },
    SemiprimeCase { bits: 60, n: "813080171761801367", period: "2746892466075366", p: "893585183", q: "909907849" },
    SemiprimeCase { bits: 64, n: "14672336402354228699", period: "7336168197340556268", p: "4052987087", q: "3620129077" },
    SemiprimeCase { bits: 80, n: "966797818948726127655223", period: "8334463956437499719120", p: "1085571153761", q: "890589083543" },
    SemiprimeCase { bits: 96, n: "44183798750402948400038412281", period: "2761487421900157966178043552", p: "221151070004537", q: "199790119710913" },
    SemiprimeCase { bits: 112, n: "3924631727675843096135905777198787", period: "654105287945973828429353931842276", p: "58695408508679653", q: "66864373677465479" },
    SemiprimeCase { bits: 128, n: "182342945995603068530746742010183178459", period: "5698217062362595890737663640694859090", p: "12220795601362844411", q: "14920709906584843169" },
    SemiprimeCase { bits: 144, n: "22300745198530620650629069553396669406425677", period: "11150372599265310325309812410215465058262878", p: "4722366482869644978023", q: "4722366482869644921899" },
    SemiprimeCase { bits: 160, n: "1461501637330902917555683668441367177275902271661", period: "730750818665451458777840625294863974008776697662", p: "1208925819614629174468199", q: "1208925819614629174408139" },
    SemiprimeCase { bits: 191, n: "3138550867693340381917842054546708083858672444497848798349", period: "1569275433846670190958921027213932920043637969053766970718", p: "39614081257132168796771714507", q: "79228162514264337593543142407" },
    SemiprimeCase { bits: 223, n: "13479973333575319897333507540174556430480872021152793249913675077821", period: "6739986666787659948666753770087274321017792109455675227084591142222", p: "2596148429267413814265248164362119", q: "5192296858534827628530496328431259" },
    SemiprimeCase { bits: 255, n: "57896044618658097711785492504343728987229868832960740349236731826687410798541", period: "28948022309329048855892746252171864493359722641289666327020834957769879769166", p: "170141183460469231731687303715883840627", q: "340282366920938463463374607431767419583" },
    SemiprimeCase { bits: 287, n: "132057622348656887672497286151776692678589416745347226462918768927144655719574760686733", period: "66028811174328443836248643075888346339294695268443669191255802839602815095879191192654", p: "6806355693340763852467671011242997535381407", q: "19402104194739643310780268014284818842920019" },
    SemiprimeCase { bits: 319, n: "581453302228393347177371717563159270601439445684297874643765742908935832091946980982785579511557", period: "290726651114196673588685858781579635300719722841295573323793062475626043497624044216005736598118", p: "470249379460566177518714778065268307040951509103", q: "1236478616719051780165030318633624243733154806219" },
    SemiprimeCase { bits: 383, n: "12117078883062091223581626981175349304509781230496212419856353005113799515361552940616399710584257513929717664541181", period: "6058539441531045611790813490587674652254890615248106209924403927148993411693701451160003425344635284104607906217326", p: "2318186764772179599966291686563703002392294720995094703127", q: "5226964051040512374183746609829156892594650999506757403403" },
    SemiprimeCase { bits: 447, n: "305100884042193417263568279997656609308598225057460346983143158950631384201219238105125249767457334793902808057789165433520719660182333", period: "152550442021096708631784139998828304654299112528730173491571579475297616371348371445195004011628941970158008442753003443714613846595134", p: "13424804161471267566133790630585501859097498936714704581803781808767", q: "22726654361023947169107953568865351727693673346443841509688185183299" },
    SemiprimeCase { bits: 511, n: "4521299763651038267854831935796405862942924301726072228529938030280396807370390301103275666015126099891434326723308081458446609998451024357484034764592373", period: "2260649881825519133927415967898202931471462150863036114264969015140198403685124959013398155200320834791663844083299760311002465681251098714126516004827174", p: "50053209689345890006518114057229021198083818215016307644690502271601694058727", q: "90329866790009724477912194049409535510477018226662328304136426959401060879299" },
];

fn decimal_tape(text: &str) -> Tape {
    assert!(!text.is_empty());
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

/// Build one idealized s=1 Fourier read near 1/r for the regression oracle.
/// The period is used only here, in the test harness, to synthesize k/M.  The
/// production carrier receives only N, base=2, k, and M; p/q and r are not
/// resident production state.
fn phase_sample(period: &[char], n_bits: usize) -> (Tape, Tape) {
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
    let expected_p = decimal_tape(case.p);
    let expected_q = decimal_tape(case.q);
    let direct = got_p == expected_p && got_q == expected_q;
    let swapped = got_p == expected_q && got_q == expected_p;
    assert!(direct || swapped, "{}-bit factor pair mismatch", case.bits);
}

fn run_cases(cases: &[SemiprimeCase]) {
    for case in cases {
        run_case(case);
    }
}

#[test]
fn one_phase_sample_scales_from_31_through_64_bit_semiprimes() {
    run_cases(&CASES[0..6]);
}

#[test]
fn one_phase_sample_crosses_the_host_word_and_reaches_128_bit_semiprimes() {
    run_cases(&CASES[6..10]);
}

#[test]
fn one_phase_sample_scales_from_144_through_255_bit_semiprimes() {
    run_cases(&CASES[10..15]);
}

#[test]
fn one_phase_sample_scales_from_287_through_511_bit_semiprimes() {
    run_cases(&CASES[15..20]);
}
