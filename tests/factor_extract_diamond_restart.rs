use vox::factor_extract::{extract, reenter_once, FactorCarrier};
use vox::morphism_factor::{cmp, mul, tape_u64};
use vox::router_marks::{run_mark, GStep, RouterG, M_B, M_FIX, M_N, M_T};
use vox::router_object::RouterObject;
use vox::tape_delete::delete_word;
use vox::trace_algebra::{admissible_relaxed_with_witness, witness_valid};
use vox::trace_word::{decode_trace, encode_trace, judge_trace};
use vox::vox::EVALF;

const N: u64 = 106_545_994_355_809;

fn production_terminal() -> (Vec<char>, Vec<char>, Vec<char>, GStep, Vec<char>) {
    let n = tape_u64(N);
    let router = RouterG::from_enum(&RouterObject::initial());
    let (found, trajectory) = run_mark(&router, N, 8);
    let (p_u64, q_u64) = found.expect("production route did not carry factors");
    let p = tape_u64(p_u64);
    let q = tape_u64(q_u64);
    let carrier = FactorCarrier::from_trace(&n, &p, &q, &trajectory).unwrap();
    let normal = extract(&carrier).unwrap().normal_form;
    let terminal_steps = decode_trace(&normal).unwrap();
    assert_eq!(terminal_steps.len(), 1);
    (n, p, q, terminal_steps[0].clone(), normal)
}

const PAYLOAD: [char; 12] = ['⊢', '⊣', '≻', '∈', '⊤', '⋈', '≺', '⊥', '⊞', '⊙', '∋', '⊡'];

fn diamond_trace(terminal: &GStep, scaffold: usize, salt: usize) -> Vec<char> {
    assert!(scaffold >= 4);
    let mut steps = Vec::with_capacity(scaffold + 1);
    for i in 0..scaffold {
        // Keep the routing representation uniform so deleting either member of
        // a critical pair cannot accidentally change the terminal projection.
        let payload_len = 3 + ((i * 7 + salt * 11) % 53);
        let payload = (0..payload_len)
            .map(|j| PAYLOAD[(i + j * 5 + salt) % PAYLOAD.len()])
            .collect();
        steps.push(GStep {
            repr: '⋈',
            judgment: if (i + salt) % 3 == 0 { M_N } else { M_B },
            recognised: M_T,
            next: if i + 1 == scaffold { terminal.repr } else { '⋈' },
            applied_word: payload,
        });
    }
    steps.push(terminal.clone());
    encode_trace(&steps)
}

fn one_step_delete(carrier: &FactorCarrier, i: usize) -> FactorCarrier {
    let candidate = delete_word(&carrier.trace, i).expect("delete_word rejected an in-range record");
    assert!(
        admissible_relaxed_with_witness(
            &carrier.trace,
            &candidate,
            &carrier.n,
            &carrier.p,
            &carrier.q,
        ),
        "chosen critical-pair arm was not admissible",
    );
    FactorCarrier::new(
        carrier.n.clone(),
        carrier.p.clone(),
        carrier.q.clone(),
        candidate,
    ).unwrap()
}

#[test]
fn admissible_critical_pairs_form_exact_local_diamonds() {
    let (n, p, q, terminal, canonical) = production_terminal();
    let cases = 40usize;
    let mut diamonds = 0usize;

    for case in 0..cases {
        let scaffold = 8 + (case * 13 % 29); // 8..36 neutral records.
        let trace = diamond_trace(&terminal, scaffold, case);
        assert_eq!(judge_trace(&trace), M_T);
        let source = FactorCarrier::new(n.clone(), p.clone(), q.clone(), trace).unwrap();

        let pairs = [
            (0usize, 1usize),
            (0usize, scaffold - 1),
            (scaffold / 3, (2 * scaffold) / 3),
            (scaffold / 4, scaffold - 2),
        ];

        for &(i0, j0) in &pairs {
            let (i, j) = if i0 < j0 { (i0, j0) } else { (j0, i0) };
            assert!(i < j && j < scaffold);

            //       C
            //      / \
            //   -i/   \-j
            //    A     B
            //     \   /
            //    -j' -i
            //       D
            //
            // Because i < j, deleting i shifts the old j to j-1 on arm A.
            let arm_a = one_step_delete(&source, i);
            let arm_b = one_step_delete(&source, j);
            assert_ne!(arm_a.trace, arm_b.trace, "case {case}: critical-pair arms collapsed prematurely");

            let join_a = one_step_delete(&arm_a, j - 1);
            let join_b = one_step_delete(&arm_b, i);
            assert_eq!(join_a.trace, join_b.trace, "case {case}: local diamond did not close exactly");
            assert_eq!(join_a.n, source.n);
            assert_eq!(join_a.p, source.p);
            assert_eq!(join_a.q, source.q);

            // Each branch must also remain globally confluent with the real
            // production normal form, not just meet at an intermediate object.
            let normal_a = extract(&arm_a).unwrap();
            let normal_b = extract(&arm_b).unwrap();
            assert_eq!(normal_a.normal_form, canonical, "case {case}: arm A missed canonical form");
            assert_eq!(normal_b.normal_form, canonical, "case {case}: arm B missed canonical form");
            assert!(witness_valid(&n, &normal_a.p, &normal_a.q));
            assert!(witness_valid(&n, &normal_b.p, &normal_b.q));
            diamonds += 1;
        }
    }

    println!("local diamonds: {diamonds} competing admissible pairs closed exactly and converged");
}

fn mersenne_tape(bits: usize) -> Vec<char> {
    assert!(bits >= 2);
    vec![EVALF; bits]
}

fn restart_trace(records: usize) -> Vec<char> {
    assert!(records >= 1);
    let mut steps = Vec::with_capacity(records);
    for i in 0..records {
        let last = i + 1 == records;
        steps.push(GStep {
            repr: '⋈',
            judgment: if last { M_T } else if i % 4 == 0 { M_N } else { M_B },
            recognised: M_T,
            next: if last { M_FIX } else { '⋈' },
            applied_word: PAYLOAD
                .iter()
                .copied()
                .cycle()
                .skip(i % PAYLOAD.len())
                .take(1 + (i * 17 % 61))
                .collect(),
        });
    }
    encode_trace(&steps)
}

/// Continue from persisted bytes only. Between generations no FactorCarrier,
/// cursor, generation counter, or reduction schedule is retained.
fn resume_from_wire(mut wire: Vec<char>) -> FactorCarrier {
    loop {
        let current = FactorCarrier::decode(&wire).expect("restart decode failed");
        let (next, changed) = reenter_once(&current).expect("restart re-entry failed");
        drop(current);
        wire = next.encode();
        drop(next);
        if !changed {
            return FactorCarrier::decode(&wire).expect("fixed carrier did not survive final restart");
        }
    }
}

#[test]
fn every_generation_is_a_complete_restart_point_from_serialized_object_alone() {
    // 2300-bit product, deep enough to make restart a repeated mechanism rather
    // than a single codec check.
    let p = mersenne_tape(1021);
    let q = mersenne_tape(1279);
    let n = mul(&p, &q);
    let records = 47usize;
    let initial = FactorCarrier::new(n.clone(), p, q, restart_trace(records)).unwrap();
    let baseline = extract(&initial).unwrap();
    assert_eq!(baseline.transforms, records - 1);
    assert_eq!(baseline.generations.len(), records);

    // Interrupt after every possible number of generations, including zero and
    // the final unchanged fixed-point generation. Each experiment reconstructs
    // its prefix from the initial wire and persists only the resulting bytes.
    for cut in 0..=records {
        let mut wire = initial.encode();
        let mut observed_fixed = false;

        for generation in 0..cut {
            let current = FactorCarrier::decode(&wire).unwrap();
            let before = decode_trace(&current.trace).unwrap().len();
            let (next, changed) = reenter_once(&current).unwrap();
            let after = decode_trace(&next.trace).unwrap().len();
            wire = next.encode();
            drop(current);
            drop(next);

            if changed {
                assert_eq!(after + 1, before, "cut {cut}, generation {generation}: no strict descent");
            } else {
                assert_eq!(before, 1, "fixed point appeared before one-record carrier");
                assert_eq!(after, 1);
                observed_fixed = true;
                assert_eq!(generation + 1, records, "fixed generation occurred at wrong depth");
            }
        }

        if cut == records {
            assert!(observed_fixed, "final cut did not include fixed-point recognition");
        }

        // Simulated process death: the only surviving object is the serialized
        // mark vector. resume_from_wire has no access to the prefix's host state.
        let persisted = wire;
        let resumed = resume_from_wire(persisted);

        assert_eq!(resumed.trace, baseline.normal_form, "cut {cut}: restart changed normal form");
        assert_eq!(decode_trace(&resumed.trace).unwrap().len(), 1);
        assert!(witness_valid(&resumed.n, &resumed.p, &resumed.q));
        assert_eq!(cmp(&mul(&resumed.p, &resumed.q), &n), core::cmp::Ordering::Equal);
    }

    println!(
        "restart sweep: {} interruption points over {}-generation self-entry all resumed from marks alone",
        records + 1,
        records,
    );
}
