use core::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

use vox::factor_extract::{extract, FactorCarrier};
use vox::factorization_31_membrane::{UnboundedResident, WORD};
use vox::morphism_factor::{cmp, divmod, miller_rabin, mul, tape_u64, zero};
use vox::producer_provenance::{
    resident_route_provenance, route_provenance, ProducerRouteProvenance,
    SUPPORT_PRODUCT_BOUNDARY,
};
use vox::provenance_envelope::{is_descending, suffix_fibre_size};
use vox::reentry_certificate::{certify_reentry, verify_reentry_certificate};
use vox::router_marks::{GStep, M_B, M_FIX, M_T};
use vox::trace_algebra::{relaxed_equivalent_with_witness, witness_valid};
use vox::trace_word::encode_trace;
use vox::vox::{EVALF, EVALT};

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

fn terminal_trace() -> Vec<char> {
    encode_trace(&[GStep {
        repr: '⋈',
        judgment: M_T,
        recognised: M_T,
        next: M_FIX,
        applied_word: vec!['⊣'],
    }])
}

fn resident_pair(n: &[char], resident: &UnboundedResident) -> (Vec<char>, Vec<char>) {
    let one = tape_u64(1);
    let factor = resident
        .factors
        .iter()
        .find(|f| cmp(f, &one) == Ordering::Greater && cmp(f, n) == Ordering::Less)
        .expect("resident produced no proper factor")
        .clone();
    let (cofactor, remainder) = divmod(n, &factor);
    assert!(zero(&remainder));
    assert!(witness_valid(n, &factor, &cofactor));
    (factor, cofactor)
}

fn route_trace(provenance: &ProducerRouteProvenance) -> Vec<char> {
    let mut steps = Vec::with_capacity(provenance.deposits.len() + 1);
    for &support in &provenance.deposits {
        let mut payload = vec!['⊢', '∈'];
        for bit in 0..6u32 {
            payload.push(if (support >> bit) & 1 == 1 { EVALF } else { EVALT });
        }
        payload.extend(['∋', '⊙', '⊣']);
        steps.push(GStep {
            repr: '⋈',
            judgment: M_B,
            recognised: M_T,
            next: '⋈',
            applied_word: payload,
        });
    }
    steps.push(GStep {
        repr: '⋈',
        judgment: M_T,
        recognised: M_T,
        next: M_FIX,
        applied_word: vec!['⊣'],
    });
    encode_trace(&steps)
}

#[test]
fn real_semiprime_routes_land_in_distinct_native_provenance_fibres() {
    let p_value = next_prime(1_000_000);
    let p = tape_u64(p_value);
    let requested_gaps = [1_000u64, 4_000, 8_000, 16_000, 32_000, 48_000, 64_000];
    let terminal = terminal_trace();

    let mut ladders_by_route: BTreeMap<String, BTreeSet<Vec<u32>>> = BTreeMap::new();

    for requested_gap in requested_gaps {
        let q_value = next_prime(p_value + requested_gap);
        let q = tape_u64(q_value);
        let n = mul(&p, &q);

        let mut resident = UnboundedResident::new(n.clone());
        resident.run();
        assert!(resident.boundary_ok);
        assert!(resident.sidearm_round_trip);

        let provenance = resident_route_provenance(&resident)
            .expect("closed resident did not expose a route provenance envelope");
        assert_eq!(provenance.route, resident.shape_route);
        assert!(is_descending(&provenance.ladder));
        assert_eq!(provenance.deposits.len(), 2);
        assert_eq!(provenance.ladder.len(), 2);
        assert_eq!(provenance.ladder[1], SUPPORT_PRODUCT_BOUNDARY);
        assert_eq!(suffix_fibre_size(&provenance.ladder), Some(2));
        ladders_by_route
            .entry(provenance.route.clone())
            .or_default()
            .insert(provenance.ladder.clone());

        let (factor, cofactor) = resident_pair(&n, &resident);
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
        let summary = verify_reentry_certificate(&certify_reentry(&carrier).unwrap()).unwrap();
        assert_eq!(summary.transforms, 30);
        assert_eq!(summary.normal_form, terminal);

        println!(
            "route fibre: requested_gap={} actual_gap={} route={} deposits={:?} ladder={:?}",
            requested_gap,
            q_value - p_value,
            resident.shape_route,
            provenance.deposits,
            provenance.ladder,
        );
    }

    let frontier = ladders_by_route.get("frontier").expect("no frontier provenance fibre");
    let near_root = ladders_by_route.get("near-root").expect("no near-root provenance fibre");
    assert_eq!(frontier.len(), 1, "frontier route split across multiple ladders");
    assert_eq!(near_root.len(), 1, "near-root route split across multiple ladders");
    assert_ne!(frontier.iter().next(), near_root.iter().next());

    println!(
        "producer provenance: frontier and near-root occupy distinct native fibres with one shared product-boundary tail"
    );
}

#[test]
fn distinct_route_fibres_are_erased_by_the_same_factor_object_quotient() {
    let p_value = next_prime(1_000_000);
    let q_value = next_prime(p_value + 32_000);
    let p = tape_u64(p_value);
    let q = tape_u64(q_value);
    let n = mul(&p, &q);

    let frontier = route_provenance("frontier").unwrap();
    let near_root = route_provenance("near-root").unwrap();
    assert_ne!(frontier.ladder, near_root.ladder);
    assert_eq!(frontier.ladder[1], near_root.ladder[1]);
    assert_eq!(frontier.ladder[1], SUPPORT_PRODUCT_BOUNDARY);

    let frontier_carrier = FactorCarrier::new(
        n.clone(),
        p.clone(),
        q.clone(),
        route_trace(&frontier),
    )
    .unwrap();
    let near_root_carrier = FactorCarrier::new(
        n.clone(),
        p.clone(),
        q.clone(),
        route_trace(&near_root),
    )
    .unwrap();

    assert_ne!(frontier_carrier.trace, near_root_carrier.trace);
    assert!(relaxed_equivalent_with_witness(
        &frontier_carrier.trace,
        (&frontier_carrier.p, &frontier_carrier.q),
        &near_root_carrier.trace,
        (&near_root_carrier.p, &near_root_carrier.q),
        &n,
    ));

    let a = extract(&frontier_carrier).unwrap();
    let b = extract(&near_root_carrier).unwrap();
    assert_eq!(a.transforms, 2);
    assert_eq!(b.transforms, 2);
    assert_eq!(a.normal_form, b.normal_form);
    assert_eq!(a.normal_form, terminal_trace());

    let ca = verify_reentry_certificate(&certify_reentry(&frontier_carrier).unwrap()).unwrap();
    let cb = verify_reentry_certificate(&certify_reentry(&near_root_carrier).unwrap()).unwrap();
    assert_eq!(ca.normal_form, cb.normal_form);
    assert_eq!(ca.transforms, 2);
    assert_eq!(cb.transforms, 2);

    println!(
        "route quotient syzygy: two distinct producer fibres -> one exact terminal trace for the same frozen factor object"
    );
}
