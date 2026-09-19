use core::cmp::Ordering;

use vox::dialectic_certificate::{
    certify_dialectic, decode_dialectic_certificate, encode_dialectic_certificate,
    verify_dialectic_certificate,
};
use vox::dialectic_reentry::{
    DialecticObject, EXTENDED_FERMAT_SPAN, IM_RWX, LEHMAN_LOCAL_SPAN,
    SHORT_FRONTIER_SPAN,
};
use vox::factorization_31_membrane::UnboundedResident;
use vox::morphism_factor::{cmp, mul, tape_u64};
use vox::producer_provenance::{resident_route_provenance, route_provenance};
use vox::reentry_certificate::{certify_reentry, verify_reentry_certificate};
use vox::trace_algebra::witness_valid;
use vox::vox::{EVALF, EVALT};

fn same_pair(a: &[char], b: &[char], x: &[char], y: &[char]) -> bool {
    (cmp(a, x) == Ordering::Equal && cmp(b, y) == Ordering::Equal)
        || (cmp(a, y) == Ordering::Equal && cmp(b, x) == Ordering::Equal)
}

fn flip(mark: char) -> char {
    if mark == EVALF { EVALT } else { EVALF }
}

fn skip_field(encoded: &[char], i: &mut usize) -> (usize, usize) {
    assert_eq!(encoded[*i], '∈');
    *i += 1;
    let start = *i;
    while encoded[*i] != '∋' {
        *i += 1;
    }
    let end = *i;
    *i += 1;
    (start, end)
}

#[test]
fn three_dialectic_closure_depths_are_the_existing_route_provenance_fibres() {
    let cases = [
        (1_000_003u64, 1_001_003u64, "frontier", 1usize),
        (1_000_003u64, 1_032_007u64, "near-root", 2usize),
        (1_000_003u64, 10_000_019u64, "HARD", 12usize),
    ];

    for (p_u64, q_u64, expected_route, expected_descents) in cases {
        let p = tape_u64(p_u64);
        let q = tape_u64(q_u64);
        let n = mul(&p, &q);

        // Dialectic side: every whole operator-space object is persisted in the
        // certificate before the next imscription is allowed to execute.
        let start = DialecticObject::new(n.clone()).unwrap();
        let certificate = certify_dialectic(&start).unwrap();
        let summary = verify_dialectic_certificate(&certificate).unwrap();
        assert_eq!(summary.descents, expected_descents);
        assert!(witness_valid(
            &summary.terminal_carrier.n,
            &summary.terminal_carrier.p,
            &summary.terminal_carrier.q,
        ));
        assert!(same_pair(
            &summary.terminal_carrier.p,
            &summary.terminal_carrier.q,
            &p,
            &q,
        ));

        // Provenance side: the named producer fibre already defines the restored
        // outer support. The dialectic terminal must land on exactly that support.
        let named = route_provenance(expected_route).unwrap();
        assert_eq!(summary.terminal_support, named.ladder[0]);

        // Existing resident side: factor the same N independently of the new
        // dialectic tower and require its actual route provenance to be the same
        // native restored-support fibre.
        let mut resident = UnboundedResident::new(n.clone());
        resident.run();
        let resident_provenance = resident_route_provenance(&resident)
            .expect("resident did not close the common product boundary");
        assert_eq!(resident_provenance.route, expected_route);
        assert_eq!(resident_provenance.ladder[0], summary.terminal_support);

        // The ordinary passive factor-carrier certificate remains the downstream
        // quotient after the imscription has locked around the pair.
        let terminal = verify_reentry_certificate(
            &certify_reentry(&summary.terminal_carrier).unwrap(),
        )
        .unwrap();
        assert_eq!(terminal.transforms, 0);
        assert_eq!(terminal.normal_form, summary.terminal_carrier.trace);

        println!(
            "imscription syzygy: route={} descents={} terminal_support={} pair={}x{}",
            expected_route,
            summary.descents,
            summary.terminal_support,
            p_u64,
            q_u64,
        );
    }
}

#[test]
fn whole_object_certificate_replays_every_restart_and_rejects_forged_links() {
    let p = tape_u64(1_000_003);
    let q = tape_u64(10_000_019);
    let n = mul(&p, &q);
    let start = DialecticObject::new(n.clone()).unwrap();
    let certificate = certify_dialectic(&start).unwrap();
    let summary = verify_dialectic_certificate(&certificate).unwrap();

    assert_eq!(summary.descents, 12);
    assert_eq!(summary.supports.len(), 12);
    assert_eq!(summary.supports[0], 3);  // parity | primality
    assert_eq!(summary.supports[1], 7);  // + short frontier
    assert!(summary.supports[2..].iter().all(|&s| s == 15)); // + extended Fermat
    assert_eq!(summary.terminal_support, 63);
    assert_eq!(certificate.terminal_rwx, IM_RWX);
    assert_eq!(certificate.terminal_span, tape_u64(LEHMAN_LOCAL_SPAN));
    assert_eq!(certificate.lattice_cell, tape_u64(0));

    // Every certified object is independently decodable from its marks alone.
    for (index, wire) in certificate.objects.iter().enumerate() {
        let object = DialecticObject::decode(wire).unwrap();
        assert_eq!(object.n, n);
        assert_eq!(object.imscription.rwx, IM_RWX);
        let expected_span = match index {
            0 => SHORT_FRONTIER_SPAN,
            1 => EXTENDED_FERMAT_SPAN,
            _ => LEHMAN_LOCAL_SPAN,
        };
        assert_eq!(object.imscription.span, tape_u64(expected_span));
    }

    // Forge an intermediate persisted Lehman boundary. It may still be a valid
    // local object, but it cannot be the exact image of the preceding descent.
    let mut bad_boundary = certificate.clone();
    let wire = &mut bad_boundary.objects[4];
    let mut i = 2usize;
    let _n = skip_field(wire, &mut i);
    let boundary = skip_field(wire, &mut i);
    wire[boundary.0] = flip(wire[boundary.0]);
    assert!(verify_dialectic_certificate(&bad_boundary).is_err());

    // Reorder two complete restart points: continuity must fail even though each
    // individual wire object is well-formed.
    let mut bad_order = certificate.clone();
    bad_order.objects.swap(4, 5);
    assert!(verify_dialectic_certificate(&bad_order).is_err());

    // Forge the terminal restored support while leaving the factor pair intact.
    let mut bad_terminal = certificate.clone();
    bad_terminal.terminal_support ^= 1 << 5;
    assert!(verify_dialectic_certificate(&bad_terminal).is_err());

    // Forge only the terminal imscribed span. Boundary, word, r/w/x and carrier
    // remain untouched; the proof must still reject the changed space.
    let mut bad_span = certificate.clone();
    bad_span.terminal_span[0] = flip(bad_span.terminal_span[0]);
    assert!(verify_dialectic_certificate(&bad_span).is_err());

    // The closing cell is proof data, not host address state. A structurally
    // valid numeral wider than usize must survive codec reconstruction intact;
    // semantic replay then rejects it because it is not the actual closing cell.
    let mut bad_wide_cell = certificate.clone();
    let mut wide_cell = vec![EVALT; usize::BITS as usize + 2];
    *wide_cell.last_mut().unwrap() = EVALF;
    bad_wide_cell.lattice_cell = wide_cell.clone();
    let wide_wire = encode_dialectic_certificate(&bad_wide_cell);
    let decoded_wide = decode_dialectic_certificate(&wide_wire).unwrap();
    assert_eq!(decoded_wide.lattice_cell, wide_cell);
    assert!(verify_dialectic_certificate(&decoded_wide).is_err());

    println!(
        "dialectic certificate: {} persisted whole objects replay exactly; forged boundary/order/support/span and host-wider lattice cell rejected semantically",
        summary.descents,
    );
}
