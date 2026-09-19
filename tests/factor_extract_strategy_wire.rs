use vox::factor_extract::{extract, reenter_once, FactorCarrier};
use vox::morphism_factor::{cmp, mul, tape_u64};
use vox::router_marks::{run_mark, GStep, RouterG, M_B, M_FIX, M_N, M_T};
use vox::router_object::RouterObject;
use vox::tape_delete::delete_word;
use vox::trace_algebra::{admissible_relaxed_with_witness, witness_valid};
use vox::trace_word::{decode_trace, encode_trace, judge_trace};
use vox::vox::EVALF;

const N: u64 = 106_545_994_355_809;

fn xorshift64(state: &mut u64) -> u64 {
    let mut x = *state;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    *state = x;
    x
}

const REPRS: [char; 3] = ['⊢', '⊣', '⋈'];
const PAYLOAD: [char; 12] = ['⊢', '⊣', '≻', '∈', '⊤', '⋈', '≺', '⊥', '⊞', '⊙', '∋', '⊡'];

fn expanded_trace(state: &mut u64, terminal: &GStep, scaffold: usize) -> Vec<char> {
    let mut reprs = Vec::with_capacity(scaffold);
    for _ in 0..scaffold {
        reprs.push(REPRS[(xorshift64(state) as usize) % REPRS.len()]);
    }

    let mut steps = Vec::with_capacity(scaffold + 1);
    for i in 0..scaffold {
        let next = if i + 1 < scaffold { reprs[i + 1] } else { terminal.repr };
        let payload_len = 1 + (xorshift64(state) as usize % 64);
        let mut payload = Vec::with_capacity(payload_len);
        for _ in 0..payload_len {
            payload.push(PAYLOAD[(xorshift64(state) as usize) % PAYLOAD.len()]);
        }
        steps.push(GStep {
            repr: reprs[i],
            judgment: if xorshift64(state) & 1 == 0 { M_B } else { M_N },
            recognised: M_T,
            next,
            applied_word: payload,
        });
    }
    steps.push(terminal.clone());
    encode_trace(&steps)
}

#[derive(Clone, Copy, Debug)]
enum Strategy {
    Front,
    Reverse,
    Permuted,
}

fn candidate_order(len: usize, strategy: Strategy, state: &mut u64) -> Vec<usize> {
    let mut order: Vec<usize> = (0..len).collect();
    match strategy {
        Strategy::Front => {}
        Strategy::Reverse => order.reverse(),
        Strategy::Permuted => {
            // Deterministic Fisher-Yates: strategy changes every generation,
            // but a failure is exactly reproducible from the fixed seed.
            for i in (1..order.len()).rev() {
                let j = (xorshift64(state) as usize) % (i + 1);
                order.swap(i, j);
            }
        }
    }
    order
}

fn reduce_with_strategy(
    carrier: &FactorCarrier,
    strategy: Strategy,
    seed: u64,
) -> FactorCarrier {
    carrier.validate().unwrap();
    let mut current = carrier.clone();
    let mut state = seed;

    loop {
        let records = decode_trace(&current.trace).unwrap().len();
        let mut committed = false;
        for i in candidate_order(records, strategy, &mut state) {
            let Some(candidate) = delete_word(&current.trace, i) else { continue };
            if admissible_relaxed_with_witness(
                &current.trace,
                &candidate,
                &current.n,
                &current.p,
                &current.q,
            ) {
                current.trace = candidate;
                current.validate().unwrap();
                committed = true;
                break;
            }
        }
        if !committed { break; }
    }

    current
}

#[test]
fn deletion_strategy_is_irrelevant_to_the_exact_relaxed_normal_form() {
    let n = tape_u64(N);
    let router = RouterG::from_enum(&RouterObject::initial());
    let (found, trajectory) = run_mark(&router, N, 8);
    let (p_u64, q_u64) = found.expect("production route did not carry factors");
    let p = tape_u64(p_u64);
    let q = tape_u64(q_u64);
    assert!(witness_valid(&n, &p, &q));

    let production = FactorCarrier::from_trace(&n, &p, &q, &trajectory).unwrap();
    let production_normal = extract(&production).unwrap().normal_form;
    let terminal = decode_trace(&production_normal).unwrap()[0].clone();

    let mut generator = 0x5354_5241_5445_4759u64; // "STRATEGY"
    let variants = 48usize;

    for case in 0..variants {
        let scaffold = 1 + (xorshift64(&mut generator) as usize % 64);
        let trace = expanded_trace(&mut generator, &terminal, scaffold);
        assert_eq!(judge_trace(&trace), M_T);
        let carrier = FactorCarrier::new(n.clone(), p.clone(), q.clone(), trace).unwrap();

        let front = reduce_with_strategy(&carrier, Strategy::Front, 0);
        let reverse = reduce_with_strategy(&carrier, Strategy::Reverse, 0);
        let permuted = reduce_with_strategy(
            &carrier,
            Strategy::Permuted,
            0x9e37_79b9_7f4a_7c15u64 ^ case as u64,
        );

        assert_eq!(front.trace, production_normal, "case {case}: front-first diverged");
        assert_eq!(reverse.trace, production_normal, "case {case}: reverse-first diverged");
        assert_eq!(permuted.trace, production_normal, "case {case}: permuted strategy diverged");
        assert_eq!(front.trace, reverse.trace, "case {case}: front/reverse disagreement");
        assert_eq!(front.trace, permuted.trace, "case {case}: front/permuted disagreement");
    }

    println!(
        "strategy confluence: {variants} expanded carriers x 3 deletion orders -> one exact normal form"
    );
}

fn mersenne_tape(bits: usize) -> Vec<char> {
    assert!(bits >= 2);
    vec![EVALF; bits] // little-endian tape for 2^bits - 1
}

fn deep_uniform_trace(records: usize) -> Vec<char> {
    assert!(records >= 1);
    let mut steps = Vec::with_capacity(records);
    for i in 0..records {
        let last = i + 1 == records;
        steps.push(GStep {
            repr: '⋈',
            judgment: if last { M_T } else if i % 3 == 0 { M_N } else { M_B },
            recognised: M_T,
            next: if last { M_FIX } else { '⋈' },
            // Deliberately bracket-heavy so every wire crossing also rechecks
            // that trace payload framing is length-delimited rather than nested.
            applied_word: "⊢∈⊞⊙∋≻⊤≺⊥⋈⊡⊣".chars().cycle().take(1 + (i % 41)).collect(),
        });
    }
    encode_trace(&steps)
}

#[test]
fn every_self_entry_generation_survives_a_full_wire_round_trip() {
    // ~2300-bit N, well beyond the earlier perfect-membrane sample scale.
    let p = mersenne_tape(1021);
    let q = mersenne_tape(1279);
    let n = mul(&p, &q);
    let records = 47usize;
    let initial = FactorCarrier::new(n.clone(), p, q, deep_uniform_trace(records)).unwrap();
    let baseline = extract(&initial).unwrap();

    let mut current = initial;
    let mut changed_generations = 0usize;
    let mut total_generations = 0usize;

    loop {
        // The object must survive a complete marks-only boundary before the
        // resident is allowed to perform the next self-entry generation.
        let outbound = current.encode();
        let decoded = FactorCarrier::decode(&outbound).expect("pre-generation wire decode failed");
        assert_eq!(decoded, current, "wire changed carrier before generation {total_generations}");

        let before = decode_trace(&decoded.trace).unwrap().len();
        let (next, changed) = reenter_once(&decoded).unwrap();
        let after = decode_trace(&next.trace).unwrap().len();

        // And the newly produced object must cross the same boundary before it
        // can become the object for the following generation.
        let next_wire = next.encode();
        let reparsed = FactorCarrier::decode(&next_wire).expect("post-generation wire decode failed");
        assert_eq!(reparsed, next, "wire changed carrier after generation {total_generations}");
        assert!(witness_valid(&reparsed.n, &reparsed.p, &reparsed.q));
        assert_eq!(cmp(&mul(&reparsed.p, &reparsed.q), &n), core::cmp::Ordering::Equal);

        total_generations += 1;
        if changed {
            changed_generations += 1;
            assert_eq!(after + 1, before, "generation did not strictly descend");
        } else {
            assert_eq!(after, before);
            current = reparsed;
            break;
        }
        current = reparsed;
    }

    assert_eq!(changed_generations, records - 1);
    assert_eq!(total_generations, records);
    assert_eq!(current.trace, baseline.normal_form);
    assert_eq!(decode_trace(&current.trace).unwrap().len(), 1);

    println!(
        "wire re-entry: {total_generations} generations, {} serialized boundaries, exact fixed point preserved",
        total_generations * 2,
    );
}
