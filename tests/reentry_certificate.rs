use core::cmp::Ordering;

use vox::factor_extract::{extract, FactorCarrier};
use vox::morphism_factor::{cmp, mul, tape_u64};
use vox::reentry_certificate::{
    certify_reentry, verify_reentry_certificate, ReentryCertificate, ReentryLink,
};
use vox::router_marks::{run_mark, GStep, RouterG, M_B, M_FIX, M_N, M_T};
use vox::router_object::RouterObject;
use vox::tape_delete::delete_word;
use vox::trace_algebra::{admissible_relaxed_with_witness, witness_valid};
use vox::trace_word::{decode_trace, encode_trace};

const N: u64 = 106_545_994_355_809;
const PAYLOAD: [char; 12] = ['⊢', '⊣', '≻', '∈', '⊤', '⋈', '≺', '⊥', '⊞', '⊙', '∋', '⊡'];

fn production_carrier() -> FactorCarrier {
    let n = tape_u64(N);
    let router = RouterG::from_enum(&RouterObject::initial());
    let (found, trajectory) = run_mark(&router, N, 8);
    let (p, q) = found.expect("production route did not carry factors");
    FactorCarrier::from_trace(&n, &tape_u64(p), &tape_u64(q), &trajectory).unwrap()
}

#[test]
fn canonical_certificate_matches_the_passive_extractor_generation_for_generation() {
    let carrier = production_carrier();
    let readout = extract(&carrier).unwrap();
    let certificate = certify_reentry(&carrier).unwrap();
    let summary = verify_reentry_certificate(&certificate).unwrap();

    assert_eq!(summary.generations, readout.generations.len());
    assert_eq!(summary.transforms, readout.transforms);
    assert_eq!(summary.normal_form, readout.normal_form);
    assert_eq!(certificate.links.len(), readout.generations.len());
    assert_eq!(certificate.normal_form(), Some(readout.normal_form.as_slice()));

    for (link, generation) in certificate.links.iter().zip(readout.generations.iter()) {
        assert_eq!(decode_trace(&link.before).unwrap().len(), generation.records_before);
        assert_eq!(decode_trace(&link.after).unwrap().len(), generation.records_after);
        assert_eq!(link.before != link.after, generation.changed);
    }
}

fn xorshift64(state: &mut u64) -> u64 {
    let mut x = *state;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    *state = x;
    x
}

fn expanded_trace(terminal: &GStep, records: usize) -> Vec<char> {
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
                .take(1 + (i * 23 % 71))
                .collect(),
        });
    }
    // Keep the real production terminal payload/representation rather than the
    // synthetic last record.  This makes the independent chains converge to the
    // same byte-for-byte terminal object as the production route.
    let mut decoded = steps;
    *decoded.last_mut().unwrap() = terminal.clone();
    encode_trace(&decoded)
}

#[derive(Clone, Copy)]
enum Schedule {
    Reverse,
    Permuted,
}

fn order(len: usize, schedule: Schedule, state: &mut u64) -> Vec<usize> {
    let mut indices: Vec<usize> = (0..len).collect();
    match schedule {
        Schedule::Reverse => indices.reverse(),
        Schedule::Permuted => {
            for i in (1..indices.len()).rev() {
                let j = (xorshift64(state) as usize) % (i + 1);
                indices.swap(i, j);
            }
        }
    }
    indices
}

/// Construct a certificate without `reenter_once` and without `certify_reentry`.
/// The only reduction rule used here is public `delete_word + admissible ≡c`.
fn independently_generated_certificate(
    source: &FactorCarrier,
    schedule: Schedule,
    seed: u64,
) -> ReentryCertificate {
    let mut current = source.clone();
    let mut state = seed;
    let mut links = Vec::new();

    loop {
        let before = current.trace.clone();
        let count = decode_trace(&before).unwrap().len();
        let mut next_trace = None;

        for i in order(count, schedule, &mut state) {
            let Some(candidate) = delete_word(&before, i) else { continue };
            if admissible_relaxed_with_witness(
                &before,
                &candidate,
                &current.n,
                &current.p,
                &current.q,
            ) {
                next_trace = Some(candidate);
                break;
            }
        }

        match next_trace {
            Some(after) => {
                links.push(ReentryLink {
                    before,
                    after: after.clone(),
                });
                current = FactorCarrier::new(
                    current.n.clone(),
                    current.p.clone(),
                    current.q.clone(),
                    after,
                )
                .unwrap();
            }
            None => {
                links.push(ReentryLink {
                    before: before.clone(),
                    after: before,
                });
                break;
            }
        }
    }

    ReentryCertificate {
        n: source.n.clone(),
        p: source.p.clone(),
        q: source.q.clone(),
        links,
    }
}

#[test]
fn independent_schedules_verify_while_forged_chains_fail_closed() {
    let production = production_carrier();
    let production_readout = extract(&production).unwrap();
    let terminal = decode_trace(&production_readout.normal_form).unwrap()[0].clone();

    let trace = expanded_trace(&terminal, 33);
    let source = FactorCarrier::new(
        production.n.clone(),
        production.p.clone(),
        production.q.clone(),
        trace,
    )
    .unwrap();
    assert!(witness_valid(&source.n, &source.p, &source.q));

    let reverse = independently_generated_certificate(&source, Schedule::Reverse, 0);
    let permuted = independently_generated_certificate(
        &source,
        Schedule::Permuted,
        0x4345_5254_4946_5931u64,
    );

    let reverse_summary = verify_reentry_certificate(&reverse).unwrap();
    let permuted_summary = verify_reentry_certificate(&permuted).unwrap();
    assert_eq!(reverse_summary.normal_form, production_readout.normal_form);
    assert_eq!(permuted_summary.normal_form, production_readout.normal_form);
    assert_eq!(reverse_summary.transforms, 32);
    assert_eq!(permuted_summary.transforms, 32);
    assert_eq!(cmp(&mul(&reverse.p, &reverse.q), &reverse.n), Ordering::Equal);

    // Forge 1: replace the first changed edge with a premature self-link.
    let mut premature = reverse.clone();
    premature.links[0].after = premature.links[0].before.clone();
    assert!(verify_reentry_certificate(&premature).is_err());

    // Forge 2: keep valid traces but break chain continuity.
    let mut discontinuous = reverse.clone();
    discontinuous.links[1].before = discontinuous.links[0].before.clone();
    assert!(verify_reentry_certificate(&discontinuous).is_err());

    // Forge 3: remove the required terminal unchanged self-link.
    let mut no_fixed_point = reverse.clone();
    no_fixed_point.links.pop();
    assert!(verify_reentry_certificate(&no_fixed_point).is_err());

    // Forge 4: claim a two-record jump as one certified generation.
    let mut skipped = reverse.clone();
    skipped.links[0].after = skipped.links[1].after.clone();
    skipped.links.remove(1);
    assert!(verify_reentry_certificate(&skipped).is_err());

    // Forge 5: alter the frozen witness without touching the chain.
    let mut bad_witness = reverse;
    bad_witness.p = tape_u64(3);
    assert!(verify_reentry_certificate(&bad_witness).is_err());
}
