use core::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

use vox::factor_extract::{extract, FactorCarrier};
use vox::factorization_31_membrane::{UnboundedResident, WORD};
use vox::morphism_factor::{cmp, divmod, miller_rabin, mul, tape_u64, zero};
use vox::provenance_envelope::{restored_support_ladder, suffix_fibre_size};
use vox::reentry_certificate::{certify_reentry, verify_reentry_certificate};
use vox::router_marks::{GStep, M_B, M_FIX, M_N, M_T};
use vox::trace_algebra::witness_valid;
use vox::trace_word::encode_trace;
use vox::vox::{EVALF, EVALT};

fn all_sequences(k: u32, d: usize) -> Vec<Vec<u32>> {
    let base = 1u64 << k;
    let total = 1u64 << (k as usize * d);
    (0..total)
        .map(|mut n| {
            let mut q = Vec::with_capacity(d);
            for _ in 0..d {
                q.push((n % base) as u32);
                n /= base;
            }
            q
        })
        .collect()
}

fn terminal_step() -> GStep {
    GStep {
        repr: '⋈',
        judgment: M_T,
        recognised: M_T,
        next: M_FIX,
        applied_word: "⊢⊙⊡⊣".chars().collect(),
    }
}

fn trace_for_deposits(q: &[u32], lane_bits: u32) -> Vec<char> {
    let mut steps = Vec::with_capacity(q.len() + 1);
    for (i, &mask) in q.iter().enumerate() {
        let mut payload = vec!['⊢', '∈'];
        for bit in 0..lane_bits {
            payload.push(if (mask >> bit) & 1 == 1 { EVALF } else { EVALT });
        }
        payload.extend(['∋', '⊙', if i % 2 == 0 { '≻' } else { '≺' }, '⊣']);
        steps.push(GStep {
            repr: '⋈',
            judgment: if mask == 0 { M_N } else { M_B },
            recognised: M_T,
            next: '⋈',
            applied_word: payload,
        });
    }
    steps.push(terminal_step());
    encode_trace(&steps)
}

#[test]
fn scaled_native_fibres_are_exact_relaxed_quotient_classes() {
    let cases = [(1u32, 6usize), (2, 4), (3, 3), (4, 2)];
    let n = tape_u64(8051);
    let p = tape_u64(83);
    let q_factor = tape_u64(97);
    let terminal = encode_trace(&[terminal_step()]);

    let mut total_representatives = 0usize;
    let mut total_fibres = 0usize;

    for &(k, d) in &cases {
        let mut fibres: BTreeMap<Vec<u32>, Vec<Vec<u32>>> = BTreeMap::new();
        for deposits in all_sequences(k, d) {
            fibres
                .entry(restored_support_ladder(&deposits))
                .or_default()
                .push(deposits);
        }

        let expected_fibres = (d + 1).pow(k);
        let expected_representatives = 1usize << (k as usize * d);
        assert_eq!(fibres.len(), expected_fibres, "image size changed for k={k} d={d}");

        let mut represented = 0usize;
        for (ladder, deposit_schedules) in fibres {
            let expected = suffix_fibre_size(&ladder)
                .expect("native restored-support ladder was not descending") as usize;
            assert_eq!(
                deposit_schedules.len(),
                expected,
                "fibre multiplicity changed for k={k} d={d} ladder={ladder:?}"
            );

            let mut encoded = BTreeSet::new();
            for deposits in deposit_schedules {
                let carrier = FactorCarrier::new(
                    n.clone(),
                    p.clone(),
                    q_factor.clone(),
                    trace_for_deposits(&deposits, k),
                )
                .unwrap();
                assert!(encoded.insert(carrier.encode()));

                let readout = extract(&carrier).unwrap();
                assert_eq!(readout.transforms, d);
                assert_eq!(readout.generations.len(), d + 1);
                assert_eq!(readout.normal_form, terminal);

                let cert = certify_reentry(&carrier).unwrap();
                let summary = verify_reentry_certificate(&cert).unwrap();
                assert_eq!(summary.transforms, d);
                assert_eq!(summary.generations, d + 1);
                assert_eq!(summary.normal_form, terminal);
                represented += 1;
            }
            assert_eq!(encoded.len(), expected);
        }

        assert_eq!(represented, expected_representatives);
        total_representatives += represented;
        total_fibres += expected_fibres;
        println!(
            "scaled syzygy: k={} d={} representatives={} fibres={} -> one certified fixed object",
            k, d, represented, expected_fibres,
        );
    }

    assert_eq!(total_representatives, 1_088);
    assert_eq!(total_fibres, 177);
    println!(
        "scaled syzygy total: {} exact representatives across {} native fibres, one factor-bearing normal form",
        total_representatives, total_fibres,
    );
}

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

fn resident_terminal_trace() -> Vec<char> {
    encode_trace(&[GStep {
        repr: '⋈',
        judgment: M_T,
        recognised: M_T,
        next: M_FIX,
        applied_word: vec!['⊣'],
    }])
}

fn next_prime(mut value: u64) -> u64 {
    if value <= 2 {
        return 2;
    }
    if value & 1 == 0 {
        value += 1;
    }
    loop {
        if miller_rabin(&tape_u64(value)) {
            return value;
        }
        value = value.checked_add(2).expect("prime scan overflow");
    }
}

fn same_pair(a: &[char], b: &[char], x: &[char], y: &[char]) -> bool {
    (cmp(a, x) == Ordering::Equal && cmp(b, y) == Ordering::Equal)
        || (cmp(a, y) == Ordering::Equal && cmp(b, x) == Ordering::Equal)
}

#[test]
fn semiprime_gap_sweep_exposes_route_transition_without_changing_certificate_semantics() {
    let p_value = next_prime(1_000_000);
    let p_expected = tape_u64(p_value);
    let requested_gaps = [
        1_000u64, 2_000, 4_000, 8_000, 16_000, 32_000,
        48_000, 64_000, 80_000, 96_000, 112_000,
    ];
    let terminal = resident_terminal_trace();

    let mut observed = Vec::new();
    let mut route_set = BTreeSet::new();

    for requested_gap in requested_gaps {
        let q_value = next_prime(p_value + requested_gap);
        let q_expected = tape_u64(q_value);
        let n = mul(&p_expected, &q_expected);

        let mut resident = UnboundedResident::new(n.clone());
        resident.run();
        assert!(resident.boundary_ok, "resident boundary failed for requested gap {requested_gap}");
        assert!(resident.sidearm_round_trip, "sidearm relation failed for requested gap {requested_gap}");

        let one = tape_u64(1);
        let factor = resident
            .factors
            .iter()
            .find(|f| cmp(f, &one) == Ordering::Greater && cmp(f, &n) == Ordering::Less)
            .expect("resident produced no proper factor")
            .clone();
        let (cofactor, remainder) = divmod(&n, &factor);
        assert!(zero(&remainder));
        assert!(witness_valid(&n, &factor, &cofactor));
        assert!(same_pair(&factor, &cofactor, &p_expected, &q_expected));

        let carrier = FactorCarrier::new(
            n.clone(),
            factor,
            cofactor,
            resident_program_trace(),
        )
        .unwrap();
        let readout = extract(&carrier).unwrap();
        assert_eq!(readout.transforms, 30);
        assert_eq!(readout.generations.len(), 31);
        assert_eq!(readout.normal_form, terminal);

        let cert = certify_reentry(&carrier).unwrap();
        let summary = verify_reentry_certificate(&cert).unwrap();
        assert_eq!(summary.transforms, 30);
        assert_eq!(summary.generations, 31);
        assert_eq!(summary.normal_form, terminal);

        let actual_gap = q_value - p_value;
        let route = resident.shape_route.clone();
        route_set.insert(route.clone());
        observed.push((requested_gap, actual_gap, route));
    }

    assert!(route_set.contains("frontier"), "sweep never entered the frontier route: {route_set:?}");
    assert!(route_set.contains("near-root"), "sweep never entered the near-root route: {route_set:?}");

    let transitions: Vec<_> = observed
        .windows(2)
        .filter(|w| w[0].2 != w[1].2)
        .map(|w| (w[0].1, w[0].2.clone(), w[1].1, w[1].2.clone()))
        .collect();
    assert!(!transitions.is_empty(), "gap sweep did not expose a route transition");

    for (left_gap, left_route, right_gap, right_route) in &transitions {
        println!(
            "semiprime route transition bracket: actual_gap {} route={} -> actual_gap {} route={}",
            left_gap, left_route, right_gap, right_route,
        );
    }
    println!("semiprime gap sweep: {:?}", observed);
}
