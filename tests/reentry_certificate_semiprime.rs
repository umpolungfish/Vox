use core::cmp::Ordering;

use vox::factor_extract::{extract, FactorCarrier};
use vox::factorization_31_membrane::{UnboundedResident, WORD};
use vox::morphism_factor::{cmp, dec_of, decimal_to_tape, divmod, mul, tape_u64, zero};
use vox::reentry_certificate::{
    certify_reentry, compose_reentry_fragments, decode_reentry_certificate,
    encode_reentry_certificate, verify_reentry_certificate, ReentryCertificate,
    ReentryLink,
};
use vox::router_marks::{run_mark, GStep, RouterG, M_B, M_FIX, M_N, M_T};
use vox::router_object::RouterObject;
use vox::tape_delete::delete_word;
use vox::trace_algebra::{admissible_relaxed_with_witness, relaxed_equivalent_with_witness, witness_valid};
use vox::trace_word::{decode_trace, encode_trace};

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

fn same_pair(a: &[char], b: &[char], x: &[char], y: &[char]) -> bool {
    (cmp(a, x) == Ordering::Equal && cmp(b, y) == Ordering::Equal)
        || (cmp(a, y) == Ordering::Equal && cmp(b, x) == Ordering::Equal)
}

fn resident_factor_pair(n: &[char]) -> (Vec<char>, Vec<char>, String) {
    let mut resident = UnboundedResident::new(n.to_vec());
    resident.run();
    assert!(resident.boundary_ok, "31-slot arbitrary-width resident boundary did not close");
    assert!(resident.sidearm_round_trip, "31-slot sidearm relation did not round-trip");

    let one = tape_u64(1);
    let factor = resident
        .factors
        .iter()
        .find(|f| cmp(f, &one) == Ordering::Greater && cmp(f, n) == Ordering::Less)
        .expect("resident produced no proper factor")
        .clone();
    let (cofactor, remainder) = divmod(n, &factor);
    assert!(zero(&remainder), "resident factor did not divide N exactly");
    assert!(witness_valid(n, &factor, &cofactor));
    (factor, cofactor, resident.shape_route)
}

#[test]
fn semiprime_ladder_runs_real_resident_factors_through_serialized_certificates() {
    // These are factoring experiments upstream of the passive extractor.  Each
    // case starts from N alone, the UnboundedResident produces a proper factor,
    // and only then is the frozen factor object handed across the carrier boundary.
    let cases = [
        ("8051", "83", "97"),
        ("1000000016000000063", "1000000007", "1000000009"),
        (
            "10000000000000000016800000000000000005031",
            "100000000000000000039",
            "100000000000000000129",
        ),
    ];

    for &(n_dec, p_dec, q_dec) in &cases {
        let n = decimal_to_tape(n_dec).expect("decimal N did not parse to arbitrary-width tape");
        let expected_p = decimal_to_tape(p_dec).unwrap();
        let expected_q = decimal_to_tape(q_dec).unwrap();

        let (p, q, route) = resident_factor_pair(&n);
        assert!(same_pair(&p, &q, &expected_p, &expected_q), "resident returned an unexpected semiprime pair for {n_dec}");
        assert_eq!(cmp(&mul(&p, &q), &n), Ordering::Equal);

        let carrier = FactorCarrier::new(
            n.clone(),
            p.clone(),
            q.clone(),
            resident_program_trace(),
        )
        .unwrap();
        let readout = extract(&carrier).unwrap();
        assert_eq!(cmp(&mul(&readout.p, &readout.q), &n), Ordering::Equal);

        let cert = certify_reentry(&carrier).unwrap();
        let wire = encode_reentry_certificate(&cert);
        let decoded = decode_reentry_certificate(&wire).expect("marks-only certificate did not decode");
        assert_eq!(decoded, cert, "certificate codec changed the proof object");
        let summary = verify_reentry_certificate(&decoded).expect("decoded certificate did not independently verify");
        assert_eq!(summary.transforms, 30);
        assert_eq!(summary.generations, 31);
        assert_eq!(summary.normal_form, readout.normal_form);

        // Framing corruption stays separate from proof corruption and fails at decode.
        let mut truncated = wire.clone();
        assert_eq!(truncated.pop(), Some('⊣'));
        assert!(decode_reentry_certificate(&truncated).is_err());

        println!(
            "semiprime experiment: digits={} p={} q={} route={} certificate_marks={}",
            n_dec.len(),
            dec_of(&p),
            dec_of(&q),
            route,
            wire.len(),
        );
    }
}

fn production_object() -> FactorCarrier {
    const N: u64 = 106_545_994_355_809;
    let n = tape_u64(N);
    let router = RouterG::from_enum(&RouterObject::initial());
    let (found, trajectory) = run_mark(&router, N, 8);
    let (p, q) = found.expect("production route did not carry factors");
    FactorCarrier::from_trace(&n, &tape_u64(p), &tape_u64(q), &trajectory).unwrap()
}

fn expanded_trace(terminal: &GStep, records: usize) -> Vec<char> {
    assert!(records >= 2);
    let mut steps = Vec::with_capacity(records);
    for i in 0..records - 1 {
        steps.push(GStep {
            repr: '⋈',
            judgment: if i % 3 == 0 { M_N } else { M_B },
            recognised: M_T,
            next: if i + 1 == records - 1 { terminal.repr } else { '⋈' },
            applied_word: "⊢∈⊞⊙∋≻⊤≺⊥⋈⊡⊣"
                .chars()
                .cycle()
                .skip(i % 12)
                .take(1 + (i * 17 % 47))
                .collect(),
        });
    }
    steps.push(terminal.clone());
    encode_trace(&steps)
}

fn next_admissible(carrier: &FactorCarrier, reverse: bool) -> Option<FactorCarrier> {
    let count = decode_trace(&carrier.trace)?.len();
    let mut indices: Vec<usize> = (0..count).collect();
    if reverse {
        indices.reverse();
    }
    for i in indices {
        let candidate = delete_word(&carrier.trace, i)?;
        if admissible_relaxed_with_witness(
            &carrier.trace,
            &candidate,
            &carrier.n,
            &carrier.p,
            &carrier.q,
        ) {
            return FactorCarrier::new(
                carrier.n.clone(),
                carrier.p.clone(),
                carrier.q.clone(),
                candidate,
            )
            .ok();
        }
    }
    None
}

fn prefix_fragment(source: &FactorCarrier, transforms: usize) -> (ReentryCertificate, FactorCarrier) {
    let mut current = source.clone();
    let mut links = Vec::new();
    for _ in 0..transforms {
        let next = next_admissible(&current, false).expect("front schedule ran out before prefix cut");
        links.push(ReentryLink {
            before: current.trace.clone(),
            after: next.trace.clone(),
        });
        current = next;
    }
    (
        ReentryCertificate {
            n: source.n.clone(),
            p: source.p.clone(),
            q: source.q.clone(),
            links,
        },
        current,
    )
}

fn suffix_fragment(source: &FactorCarrier) -> ReentryCertificate {
    let mut current = source.clone();
    let mut links = Vec::new();
    loop {
        match next_admissible(&current, true) {
            Some(next) => {
                links.push(ReentryLink {
                    before: current.trace.clone(),
                    after: next.trace.clone(),
                });
                current = next;
            }
            None => {
                links.push(ReentryLink {
                    before: current.trace.clone(),
                    after: current.trace.clone(),
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
fn certificate_fragments_compose_only_at_identical_carriers_then_survive_the_wire() {
    let production = production_object();
    let production_readout = extract(&production).unwrap();
    let terminal = decode_trace(&production_readout.normal_form).unwrap()[0].clone();

    let source = FactorCarrier::new(
        production.n.clone(),
        production.p.clone(),
        production.q.clone(),
        expanded_trace(&terminal, 29),
    )
    .unwrap();

    // Front-first prefix, then reverse-first suffix from the exact represented
    // midpoint. The two fragments were generated under different schedules.
    let (prefix, midpoint) = prefix_fragment(&source, 11);
    let suffix = suffix_fragment(&midpoint);
    let composed = compose_reentry_fragments(&prefix, &suffix).expect("exact midpoint did not compose");
    let summary = verify_reentry_certificate(&composed).expect("composed proof did not verify");
    assert_eq!(summary.transforms, 28);
    assert_eq!(summary.generations, 29);
    assert_eq!(summary.normal_form, production_readout.normal_form);

    // Transport the composed proof as marks only and verify it after reconstruction.
    let wire = encode_reentry_certificate(&composed);
    let reconstructed = decode_reentry_certificate(&wire).unwrap();
    assert_eq!(reconstructed, composed);
    assert_eq!(verify_reentry_certificate(&reconstructed).unwrap(), summary);

    // A merely ≡c-equivalent but non-identical join is deliberately not silently
    // normalized into a chain equality proof.
    assert_ne!(midpoint.trace, source.trace);
    assert!(relaxed_equivalent_with_witness(
        &midpoint.trace,
        (&midpoint.p, &midpoint.q),
        &source.trace,
        (&source.p, &source.q),
        &source.n,
    ));
    let mut wrong_join = suffix.clone();
    wrong_join.links[0].before = source.trace.clone();
    assert!(compose_reentry_fragments(&prefix, &wrong_join).is_err());

    println!(
        "certificate composition: 11 front transforms + {} reverse transforms, marks-only proof size {}",
        summary.transforms - 11,
        wire.len(),
    );
}
