use core::cmp::Ordering;

use vox::factor_extract::{extract, reenter_once, FactorCarrier};
use vox::morphism_factor::{cmp, mul, tape_u64};
use vox::router_marks::{run_mark, GStep, RouterG, M_B, M_FIX, M_N, M_T};
use vox::router_object::RouterObject;
use vox::tape_delete::delete_word;
use vox::trace_algebra::{admissible_relaxed_with_witness, witness_valid};
use vox::trace_word::{decode_trace, encode_trace, judge_trace};
use vox::vox::{EVALF, EVALT};

const MEASURED_N: u64 = 106_545_994_355_809;
const REPRS: [char; 3] = ['⊢', '⊣', '⋈'];
const PAYLOAD: [char; 12] = ['⊢', '⊣', '≻', '∈', '⊤', '⋈', '≺', '⊥', '⊞', '⊙', '∋', '⊡'];

fn xorshift64(state: &mut u64) -> u64 {
    let mut x = *state;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    *state = x;
    x
}

fn mersenne_tape(bits: usize) -> Vec<char> {
    assert!(bits >= 2);
    vec![EVALF; bits]
}

fn deep_trace(records: usize) -> Vec<char> {
    assert!(records >= 1);
    let mut steps = Vec::with_capacity(records);
    for i in 0..records {
        let last = i + 1 == records;
        steps.push(GStep {
            repr: '⋈',
            judgment: if last { M_T } else if i % 5 == 0 { M_N } else { M_B },
            recognised: M_T,
            next: if last { M_FIX } else { '⋈' },
            applied_word: PAYLOAD
                .iter()
                .copied()
                .cycle()
                .skip(i % PAYLOAD.len())
                .take(1 + (i * 19 % 67))
                .collect(),
        });
    }
    encode_trace(&steps)
}

fn resume_from_wire(mut wire: Vec<char>) -> Result<FactorCarrier, String> {
    loop {
        let current = FactorCarrier::decode(&wire)?;
        let (next, changed) = reenter_once(&current)?;
        wire = next.encode();
        if !changed {
            return FactorCarrier::decode(&wire);
        }
    }
}

#[test]
fn every_persisted_depth_rejects_corrupt_siblings_and_resumes_the_intact_carrier() {
    let p = mersenne_tape(1021);
    let q = mersenne_tape(1279);
    let n = mul(&p, &q);
    let records = 37usize;
    let initial = FactorCarrier::new(n.clone(), p, q, deep_trace(records)).unwrap();
    let baseline = extract(&initial).unwrap().normal_form;

    let mut current = initial;
    let mut snapshots = Vec::with_capacity(records + 1);
    snapshots.push(current.encode());

    loop {
        let (next, changed) = reenter_once(&current).unwrap();
        current = next;
        snapshots.push(current.encode());
        if !changed { break; }
    }

    assert_eq!(snapshots.len(), records + 1);

    for (depth, wire) in snapshots.into_iter().enumerate() {
        let resumed = resume_from_wire(wire.clone()).unwrap();
        assert_eq!(resumed.trace, baseline, "depth {depth}: intact snapshot changed normal form");
        assert!(witness_valid(&resumed.n, &resumed.p, &resumed.q));
        assert_eq!(cmp(&mul(&resumed.p, &resumed.q), &n), Ordering::Equal);

        let mut flipped = wire.clone();
        assert_eq!(flipped[0], '⊢');
        assert_eq!(flipped[1], '∈');
        flipped[2] = if flipped[2] == EVALF { EVALT } else { EVALF };
        assert!(FactorCarrier::decode(&flipped).is_err(), "depth {depth}: flipped N bit was accepted");
        assert!(resume_from_wire(flipped).is_err(), "depth {depth}: flipped sibling resumed");

        let mut injected = wire.clone();
        injected.insert(2, '⋈');
        assert!(FactorCarrier::decode(&injected).is_err(), "depth {depth}: structural insertion was accepted");
        assert!(resume_from_wire(injected).is_err(), "depth {depth}: injected sibling resumed");

        let mut truncated = wire;
        assert_eq!(truncated.pop(), Some('⊣'));
        assert!(FactorCarrier::decode(&truncated).is_err(), "depth {depth}: truncated frame was accepted");
        assert!(resume_from_wire(truncated).is_err(), "depth {depth}: truncated sibling resumed");
    }

    println!(
        "persistence corruption: {} depths each resumed intact and rejected 3 corrupted siblings",
        records + 1,
    );
}

fn production_object() -> (Vec<char>, Vec<char>, Vec<char>, GStep, Vec<char>) {
    let n = tape_u64(MEASURED_N);
    let router = RouterG::from_enum(&RouterObject::initial());
    let (found, trajectory) = run_mark(&router, MEASURED_N, 8);
    let (p_u64, q_u64) = found.expect("production route did not carry factors");
    let p = tape_u64(p_u64);
    let q = tape_u64(q_u64);
    assert!(witness_valid(&n, &p, &q));

    let carrier = FactorCarrier::from_trace(&n, &p, &q, &trajectory).unwrap();
    let normal = extract(&carrier).unwrap().normal_form;
    let steps = decode_trace(&normal).unwrap();
    assert_eq!(steps.len(), 1);
    (n, p, q, steps[0].clone(), normal)
}

fn quotient_trace(state: &mut u64, terminal: &GStep, scaffold: usize) -> Vec<char> {
    let mut reprs = Vec::with_capacity(scaffold);
    for _ in 0..scaffold {
        reprs.push(REPRS[(xorshift64(state) as usize) % REPRS.len()]);
    }

    let mut steps = Vec::with_capacity(scaffold + 1);
    for i in 0..scaffold {
        let payload_len = 1 + (xorshift64(state) as usize % 72);
        let payload = (0..payload_len)
            .map(|_| PAYLOAD[(xorshift64(state) as usize) % PAYLOAD.len()])
            .collect();
        steps.push(GStep {
            repr: reprs[i],
            judgment: if xorshift64(state) & 1 == 0 { M_B } else { M_N },
            recognised: M_T,
            next: if i + 1 == scaffold { terminal.repr } else { reprs[i + 1] },
            applied_word: payload,
        });
    }
    steps.push(terminal.clone());
    encode_trace(&steps)
}

#[derive(Clone, Copy)]
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
            for i in (1..order.len()).rev() {
                let j = (xorshift64(state) as usize) % (i + 1);
                order.swap(i, j);
            }
        }
    }
    order
}

fn one_strategy_step(
    carrier: &FactorCarrier,
    strategy: Strategy,
    state: &mut u64,
) -> Option<FactorCarrier> {
    let records = decode_trace(&carrier.trace).unwrap().len();
    for i in candidate_order(records, strategy, state) {
        let Some(candidate) = delete_word(&carrier.trace, i) else { continue };
        if admissible_relaxed_with_witness(
            &carrier.trace,
            &candidate,
            &carrier.n,
            &carrier.p,
            &carrier.q,
        ) {
            return Some(FactorCarrier::new(
                carrier.n.clone(),
                carrier.p.clone(),
                carrier.q.clone(),
                candidate,
            ).unwrap());
        }
    }
    None
}

fn reduce_with_restart(
    carrier: &FactorCarrier,
    first: Strategy,
    second: Strategy,
    restart_after: usize,
    seed: u64,
) -> FactorCarrier {
    let mut current = carrier.clone();
    let mut state = seed;
    let mut generation = 0usize;
    let mut restarted = false;

    if restart_after == 0 {
        let wire = current.encode();
        drop(current);
        current = FactorCarrier::decode(&wire).unwrap();
        state = seed ^ 0xa5a5_5a5a_d3c1_b2e7u64;
        restarted = true;
    }

    loop {
        let strategy = if restarted { second } else { first };
        let Some(next) = one_strategy_step(&current, strategy, &mut state) else { break };
        current = next;
        generation += 1;

        if !restarted && generation >= restart_after {
            let wire = current.encode();
            drop(current);
            current = FactorCarrier::decode(&wire).unwrap();
            state = seed ^ 0xa5a5_5a5a_d3c1_b2e7u64;
            restarted = true;
        }
    }

    current
}

fn fnv1a_chars(word: &[char]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    for &ch in word {
        for byte in (ch as u32).to_le_bytes() {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3u64);
        }
    }
    hash
}

#[test]
fn quotient_class_scaling_collapses_all_schedules_and_restarts_to_one_hash() {
    let (n, p, q, terminal, canonical_trace) = production_object();
    let canonical_carrier = FactorCarrier::new(
        n.clone(),
        p.clone(),
        q.clone(),
        canonical_trace.clone(),
    ).unwrap();
    let canonical_wire = canonical_carrier.encode();
    let canonical_hash = fnv1a_chars(&canonical_wire);

    let mut generator = 0x5155_4f54_4945_4e54u64;
    let representatives = 128usize;
    let mut reductions = 0usize;

    for case in 0..representatives {
        let scaffold = xorshift64(&mut generator) as usize % 96;
        let trace = quotient_trace(&mut generator, &terminal, scaffold);
        assert_eq!(judge_trace(&trace), M_T, "case {case}: representative is not closed");
        let source = FactorCarrier::new(n.clone(), p.clone(), q.clone(), trace).unwrap();

        let patterns = [
            (Strategy::Front, Strategy::Reverse),
            (Strategy::Reverse, Strategy::Permuted),
            (Strategy::Permuted, Strategy::Front),
        ];

        for (pattern, &(first, second)) in patterns.iter().enumerate() {
            let restart_after = if scaffold == 0 {
                0
            } else {
                (xorshift64(&mut generator) as usize) % (scaffold + 2)
            };
            let final_carrier = reduce_with_restart(
                &source,
                first,
                second,
                restart_after,
                0x9e37_79b9_7f4a_7c15u64 ^ ((case as u64) << 8) ^ pattern as u64,
            );

            assert_eq!(final_carrier.trace, canonical_trace, "case {case}, pattern {pattern}: trace diverged");
            assert_eq!(final_carrier.encode(), canonical_wire, "case {case}, pattern {pattern}: serialized fixed point diverged");
            assert_eq!(fnv1a_chars(&final_carrier.encode()), canonical_hash, "case {case}, pattern {pattern}: hash diverged");
            assert!(witness_valid(&final_carrier.n, &final_carrier.p, &final_carrier.q));
            assert_eq!(cmp(&mul(&final_carrier.p, &final_carrier.q), &n), Ordering::Equal);
            reductions += 1;
        }
    }

    println!(
        "quotient scaling: {representatives} representatives, {reductions} schedule/restart reductions -> hash {canonical_hash:016x}",
    );
}
