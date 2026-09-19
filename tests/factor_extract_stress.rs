use core::cmp::Ordering;

use vox::factor_extract::{extract, extract_word, reenter_once, FactorCarrier};
use vox::morphism_factor::{cmp, dec_of, mul};
use vox::router_marks::{GStep, M_B, M_FIX, M_N, M_T};
use vox::trace_algebra::witness_valid;
use vox::trace_word::{decode_trace, encode_trace, judge_trace};
use vox::vox::{EVALF, EVALT};

fn payload(i: usize) -> Vec<char> {
    match i % 6 {
        0 => "⊢∈≻⊤⊣".chars().collect(),
        1 => "∈∋⊞⊙".chars().collect(),
        2 => "⊢∈∈∋≺∋⊣".chars().collect(),
        3 => "∋∈⊤⊥∋∈".chars().collect(),
        4 => "⊢⊙⊡⊣".chars().collect(),
        _ => "∈⊞∋∈⊙∋".chars().collect(),
    }
}

fn repr(i: usize) -> char {
    match i % 3 {
        0 => '⊢',
        1 => '⊣',
        _ => '⋈',
    }
}

/// A long, recognised, closing provenance trace.  Every nonterminal record is
/// productive B routing scaffold.  Applied payloads deliberately contain raw
/// ∈/∋ marks so the test also stresses the length-delimited trace framing while
/// self-entry deletes records.
fn deep_trace(records: usize) -> Vec<char> {
    assert!(records >= 1);
    let mut steps = Vec::with_capacity(records);
    for i in 0..records {
        let last = i + 1 == records;
        steps.push(GStep {
            repr: repr(i),
            judgment: if last { M_T } else { M_B },
            recognised: M_T,
            next: if last { M_FIX } else { repr(i + 1) },
            applied_word: payload(i),
        });
    }
    encode_trace(&steps)
}

fn mersenne_tape(bits: usize) -> Vec<char> {
    assert!(bits >= 2);
    vec![EVALF; bits] // LSB-first 2^bits - 1
}

#[test]
fn deep_31_record_carrier_reenters_one_generation_at_a_time() {
    let p = mersenne_tape(521);
    let q = mersenne_tape(607);
    let n = mul(&p, &q);
    let original = FactorCarrier::new(n.clone(), p, q, deep_trace(31)).unwrap();

    assert!(dec_of(&n).len() > 211);
    assert_eq!(decode_trace(&original.trace).unwrap().len(), 31);
    assert_eq!(judge_trace(&original.trace), M_T);

    // Walk the public one-generation API explicitly.  The resulting carrier is
    // the next object; there is no hidden generation counter or machine-width
    // numeric state involved in the recursion.
    let mut current = original.clone();
    let mut generation = 0usize;
    loop {
        let steps = decode_trace(&current.trace).unwrap();
        let records = steps.len();
        let terminal = steps.last().unwrap().repr;
        let verdict = judge_trace(&current.trace);
        let witness_closed = witness_valid(&current.n, &current.p, &current.q);

        println!(
            "gen {generation:02}: verdict {verdict} records {records:02} terminal {terminal} factor-invariant {}",
            if witness_closed { "closed" } else { "BROKEN" },
        );

        assert_eq!(verdict, M_T);
        assert!(witness_closed);

        let (next, changed) = reenter_once(&current).unwrap();
        let next_records = decode_trace(&next.trace).unwrap().len();
        if changed {
            assert_eq!(next_records + 1, records);
        } else {
            assert_eq!(next_records, records);
            assert_eq!(records, 1);
            current = next;
            break;
        }
        current = next;
        generation += 1;
    }

    assert_eq!(generation, 30);
    assert_eq!(decode_trace(&current.trace).unwrap().len(), 1);
    assert!(witness_valid(&current.n, &current.p, &current.q));

    // Cross-check the explicit walk against the convenience fixed-point runner.
    let readout = extract(&original).unwrap();
    assert_eq!(readout.transforms, 30);
    assert_eq!(readout.generations.len(), 31);
    assert_eq!(readout.normal_form, current.trace);
    assert_eq!(cmp(&mul(&readout.p, &readout.q), &n), Ordering::Equal);
}

#[test]
fn width_depth_matrix_survives_serialization_and_self_entry() {
    // (p bits, q bits, provenance records).  The final row is ~2300 bits / ~690
    // decimal digits and forces 30 explicit self-entry transformations.
    let cases = [
        (127usize, 131usize, 3usize),
        (521, 607, 11),
        (1021, 1279, 31),
    ];

    for &(p_bits, q_bits, records) in &cases {
        let p = mersenne_tape(p_bits);
        let q = mersenne_tape(q_bits);
        let n = mul(&p, &q);
        let carrier = FactorCarrier::new(n.clone(), p, q, deep_trace(records)).unwrap();
        let encoded = carrier.encode();
        let decoded = FactorCarrier::decode(&encoded).unwrap();
        assert_eq!(decoded, carrier);

        // Exercise the exact serialized consumer surface, not only extract(&carrier).
        let readout = extract_word(&encoded).unwrap();
        let normal = decode_trace(&readout.normal_form).unwrap();

        println!(
            "matrix p={p_bits}b q={q_bits}b N={} digits records={records} transforms={} generations={}",
            dec_of(&n).len(),
            readout.transforms,
            readout.generations.len(),
        );

        assert_eq!(readout.transforms, records - 1);
        assert_eq!(readout.generations.len(), records);
        assert_eq!(normal.len(), 1);
        assert_eq!(normal[0].judgment, M_T);
        assert_eq!(normal[0].next, M_FIX);
        assert_eq!(cmp(&mul(&readout.p, &readout.q), &n), Ordering::Equal);
    }
}

#[test]
fn hostile_carrier_mutations_fail_closed() {
    let p = mersenne_tape(127);
    let q = mersenne_tape(131);
    let n = mul(&p, &q);
    let carrier = FactorCarrier::new(n.clone(), p, q, deep_trace(7)).unwrap();

    // 1. Change one carried factor bit: product boundary must reject it.
    let mut bad_factor = carrier.clone();
    bad_factor.p[0] = EVALT;
    assert!(bad_factor.validate().is_err());

    // 2. Inject a non-numeral mark into the serialized N field.
    let mut non_numeral = carrier.encode();
    assert_eq!(non_numeral[0], '⊢');
    assert_eq!(non_numeral[1], '∈');
    non_numeral[2] = '⋈';
    assert!(FactorCarrier::decode(&non_numeral).is_err());
    assert!(extract_word(&non_numeral).is_err());

    // 3. Truncate the outer carrier anchor.
    let mut truncated = carrier.encode();
    truncated.pop();
    assert!(FactorCarrier::decode(&truncated).is_err());

    // 4. An unrecognised provenance record changes the trace judgment away from T.
    let mut unrecognised_steps = decode_trace(&carrier.trace).unwrap();
    unrecognised_steps[0].recognised = M_N;
    assert!(FactorCarrier::new(
        carrier.n.clone(),
        carrier.p.clone(),
        carrier.q.clone(),
        encode_trace(&unrecognised_steps),
    ).is_err());

    // 5. Remove terminal FIX without changing the carried factor.
    let mut unterminated_steps = decode_trace(&carrier.trace).unwrap();
    unterminated_steps.last_mut().unwrap().next = '⊣';
    assert!(FactorCarrier::new(
        carrier.n.clone(),
        carrier.p.clone(),
        carrier.q.clone(),
        encode_trace(&unterminated_steps),
    ).is_err());

    // 6. Keep recognition but replace terminal T with B: the object is open, not closed.
    let mut open_steps = decode_trace(&carrier.trace).unwrap();
    open_steps.last_mut().unwrap().judgment = M_B;
    assert!(FactorCarrier::new(
        carrier.n.clone(),
        carrier.p.clone(),
        carrier.q.clone(),
        encode_trace(&open_steps),
    ).is_err());

    // Control: witness order is irrelevant; the same carried pair swapped still closes.
    let swapped = FactorCarrier::new(
        carrier.n.clone(),
        carrier.q.clone(),
        carrier.p.clone(),
        carrier.trace.clone(),
    ).unwrap();
    let r = extract(&swapped).unwrap();
    assert_eq!(cmp(&mul(&r.p, &r.q), &n), Ordering::Equal);
}
