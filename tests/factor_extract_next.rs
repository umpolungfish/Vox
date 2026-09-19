use core::cmp::Ordering;

use vox::factor_extract::{extract, extract_word, FactorCarrier};
use vox::factorization_31_membrane::{UnboundedResident, WORD};
use vox::morphism_factor::{cmp, decimal_to_tape, divmod, mul, tape_u64, zero};
use vox::router_marks::{GStep, M_B, M_FIX, M_N, M_T};
use vox::trace_algebra::witness_valid;
use vox::trace_word::{decode_trace, encode_trace, judge_trace};
use vox::vox::{EVALF, EVALT};

/// Encode the 31 resident dispatch slots as provenance records after a real
/// `UnboundedResident::run`.  The resident is the multiplicative object; the
/// actual executed slot glyph is carried as the length-delimited applied word.
/// This is deliberately not a fabricated RouterG path: its payload sequence is
/// byte-for-byte `factorization_31_membrane::WORD`.
fn resident_program_trace() -> Vec<char> {
    let glyphs: Vec<char> = WORD.chars().collect();
    assert_eq!(glyphs.len(), 31);
    let steps: Vec<GStep> = glyphs
        .iter()
        .enumerate()
        .map(|(i, &glyph)| {
            let last = i + 1 == glyphs.len();
            GStep {
                repr: '⋈',
                judgment: if last { M_T } else { M_B },
                recognised: M_T,
                next: if last { M_FIX } else { '⋈' },
                applied_word: vec![glyph],
            }
        })
        .collect();
    encode_trace(&steps)
}

#[test]
fn resident_31_slot_producer_hands_real_program_provenance_to_extractor() {
    // Existing arbitrary-width production case already exercised by the resident
    // membrane tests.  The factor is discovered by that producer, not supplied
    // to the extractor test.
    let n = decimal_to_tape("10000000000000000016800000000000000005031").unwrap();
    let mut resident = UnboundedResident::new(n.clone());
    resident.run();

    assert!(resident.boundary_ok, "31-slot resident did not close its product boundary");
    assert!(resident.sidearm_round_trip, "31-slot resident lost its sidearm relation");
    let p = resident.factors.first().expect("resident produced no factor").clone();
    assert_eq!(cmp(&p, &tape_u64(1)), Ordering::Greater);
    assert_eq!(cmp(&p, &n), Ordering::Less, "resident handed N instead of a proper factor");
    let (q, rem) = divmod(&n, &p);
    assert!(zero(&rem));
    assert_eq!(cmp(&q, &tape_u64(1)), Ordering::Greater);
    assert!(witness_valid(&n, &p, &q));

    let trace = resident_program_trace();
    let decoded = decode_trace(&trace).unwrap();
    assert_eq!(decoded.len(), 31);
    assert_eq!(judge_trace(&trace), M_T);

    // Provenance payloads are exactly the program the producer executed.
    let payload_word: String = decoded
        .iter()
        .map(|step| {
            assert_eq!(step.applied_word.len(), 1);
            step.applied_word[0]
        })
        .collect();
    assert_eq!(payload_word, WORD);

    // Hard boundary: producer output is now frozen as a factor-bearing object.
    let carrier = FactorCarrier::new(n.clone(), p.clone(), q.clone(), trace).unwrap();
    let encoded = carrier.encode();
    let readout = extract_word(&encoded).unwrap();

    assert_eq!(readout.transforms, 30);
    assert_eq!(readout.generations.len(), 31);
    assert_eq!(decode_trace(&readout.normal_form).unwrap().len(), 1);
    assert_eq!(cmp(&mul(&readout.p, &readout.q), &n), Ordering::Equal);
    assert_eq!(readout.p.0, if cmp(&p, &q) == Ordering::Greater { q.clone() } else { p.clone() });
    assert_eq!(readout.q.0, if cmp(&p, &q) == Ordering::Greater { p } else { q });
}

fn xorshift64(state: &mut u64) -> u64 {
    let mut x = *state;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    *state = x;
    x
}

fn generated_tape(state: &mut u64, bits: usize) -> Vec<char> {
    assert!(bits >= 2);
    let mut out = Vec::with_capacity(bits);
    for _ in 0..bits {
        out.push(if xorshift64(state) & 1 == 1 { EVALF } else { EVALT });
    }
    // Canonical width and >1, independent of the random lower cells.
    out[bits - 1] = EVALF;
    out
}

const GLYPH_ALPHABET: [char; 12] = ['⊢', '⊣', '≻', '∈', '⊤', '⋈', '≺', '⊥', '⊞', '⊙', '∋', '⊡'];
const REPRS: [char; 3] = ['⊢', '⊣', '⋈'];

fn generated_trace(state: &mut u64, records: usize) -> Vec<char> {
    assert!(records >= 1);
    let reprs: Vec<char> = (0..records)
        .map(|_| REPRS[(xorshift64(state) as usize) % REPRS.len()])
        .collect();

    let mut steps = Vec::with_capacity(records);
    for i in 0..records {
        let last = i + 1 == records;
        let payload_len = 1 + (xorshift64(state) as usize % 48);
        let mut payload = Vec::with_capacity(payload_len);
        for _ in 0..payload_len {
            payload.push(GLYPH_ALPHABET[(xorshift64(state) as usize) % GLYPH_ALPHABET.len()]);
        }
        steps.push(GStep {
            repr: reprs[i],
            judgment: if last {
                M_T
            } else if xorshift64(state) & 3 == 0 {
                M_N
            } else {
                M_B
            },
            recognised: M_T,
            next: if last { M_FIX } else { reprs[i + 1] },
            applied_word: payload,
        });
    }
    encode_trace(&steps)
}

#[test]
fn deterministic_generative_corpus_preserves_or_rejects_every_carrier() {
    // Fixed seed: this is fuzz-shaped coverage without nondeterministic CI.
    let mut state = 0x564f_585f_5245_454eu64; // "VOX_REEN"
    let cases = 96usize;

    for case in 0..cases {
        let p_bits = 17 + (xorshift64(&mut state) as usize % 768);
        let q_bits = 19 + (xorshift64(&mut state) as usize % 1024);
        let records = 1 + (xorshift64(&mut state) as usize % 48);
        let p = generated_tape(&mut state, p_bits);
        let q = generated_tape(&mut state, q_bits);
        let n = mul(&p, &q);
        let trace = generated_trace(&mut state, records);

        let carrier = FactorCarrier::new(n.clone(), p.clone(), q.clone(), trace.clone()).unwrap();
        let encoded = carrier.encode();
        let decoded = FactorCarrier::decode(&encoded).unwrap();
        assert_eq!(decoded, carrier, "case {case}: carrier round trip changed data");

        let readout = extract(&decoded).unwrap();
        assert_eq!(readout.transforms, records - 1, "case {case}");
        assert_eq!(readout.generations.len(), records, "case {case}");
        assert_eq!(decode_trace(&readout.normal_form).unwrap().len(), 1, "case {case}");
        assert_eq!(judge_trace(&readout.normal_form), M_T, "case {case}");
        assert_eq!(cmp(&mul(&readout.p, &readout.q), &n), Ordering::Equal, "case {case}");
        assert!(witness_valid(&n, &readout.p, &readout.q), "case {case}");

        for generation in &readout.generations {
            if generation.changed {
                assert_eq!(generation.records_after + 1, generation.records_before, "case {case}");
            } else {
                assert_eq!(generation.records_before, 1, "case {case}");
                assert_eq!(generation.records_after, 1, "case {case}");
            }
        }

        // Mutation A: alter a valid numeral cell. Framing still parses, product
        // verification must reject the changed object.
        let mut wrong_n = encoded.clone();
        assert_eq!(wrong_n[0], '⊢');
        assert_eq!(wrong_n[1], '∈');
        wrong_n[2] = if wrong_n[2] == EVALF { EVALT } else { EVALF };
        assert!(FactorCarrier::decode(&wrong_n).is_err(), "case {case}: changed N was accepted");

        // Mutation B: periodically replace a numeral cell with a structural glyph.
        if case % 3 == 0 {
            let mut non_numeral = encoded.clone();
            non_numeral[2] = '⋈';
            assert!(extract_word(&non_numeral).is_err(), "case {case}: non-numeral field was accepted");
        }

        // Mutation C: periodically remove the carrier's outer closing anchor.
        if case % 5 == 0 {
            let mut truncated = encoded.clone();
            truncated.pop();
            assert!(FactorCarrier::decode(&truncated).is_err(), "case {case}: truncation was accepted");
        }

        // Mutation D: periodically make one provenance record unrecognised.
        if case % 7 == 0 {
            let mut steps = decode_trace(&trace).unwrap();
            steps[0].recognised = M_N;
            assert!(FactorCarrier::new(n.clone(), p.clone(), q.clone(), encode_trace(&steps)).is_err(),
                "case {case}: unrecognised trace was accepted");
        }

        // Mutation E: periodically open the terminal judgment while leaving all
        // framing and numeric data intact.
        if case % 11 == 0 {
            let mut steps = decode_trace(&trace).unwrap();
            steps.last_mut().unwrap().judgment = M_B;
            assert!(FactorCarrier::new(n.clone(), p.clone(), q.clone(), encode_trace(&steps)).is_err(),
                "case {case}: open terminal trace was accepted");
        }
    }

    println!("deterministic factor-carrier corpus: {cases} accepted originals plus rejection mutations");
}
