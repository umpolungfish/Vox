use core::cmp::Ordering;

use vox::dialectic_certificate::{
    certify_dialectic, decode_dialectic_certificate, encode_dialectic_certificate,
    verify_dialectic_certificate,
};
use vox::dialectic_reentry::{
    Descent, DialecticObject, EXTENDED_FERMAT_SPAN, IM_RWX, LEHMAN_LOCAL_SPAN,
    SHORT_FRONTIER_SPAN,
};
use vox::morphism_factor::{add, cmp, mul, tape_u64};
use vox::producer_provenance::{closed_support_provenance, route_provenance};
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

#[test]
fn three_dialectic_closure_depths_are_structurally_congruent_to_route_provenance() {
    let cases = [
        (1_000_003u64, 1_001_003u64, "frontier", 1usize),
        (1_000_003u64, 1_032_007u64, "near-root", 2usize),
        (1_000_003u64, 10_000_019u64, "HARD", 12usize),
    ];

    for (p_u64, q_u64, expected_route, expected_descents) in cases {
        let p = tape_u64(p_u64);
        let q = tape_u64(q_u64);
        let n = mul(&p, &q);

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

        // The terminal dialectic support is itself the complete restored outer
        // support. Project its two nested deposits directly, then compare the
        // whole envelope to the existing producer fibre. This preserves the
        // factor-bearing structural seam without running a second factor
        // producer inside a certificate/provenance congruence test.
        let expected_provenance = route_provenance(expected_route).unwrap();
        let dialectic_provenance = closed_support_provenance(summary.terminal_support)
            .expect("closed dialectic support did not encode a route provenance envelope");
        assert_eq!(dialectic_provenance, expected_provenance);

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
    assert_eq!(summary.supports[0], 3);
    assert_eq!(summary.supports[1], 7);
    assert!(summary.supports[2..].iter().all(|&s| s == 15));
    assert_eq!(summary.terminal_support, 63);
    assert_eq!(certificate.terminal_imscription.rwx.rights, IM_RWX);
    assert_eq!(certificate.terminal_imscription.rwx.read_bulk, n);
    assert_eq!(
        certificate.terminal_imscription.rwx.execute_span,
        tape_u64(LEHMAN_LOCAL_SPAN),
    );
    assert!(certificate.terminal_imscription.relation_is_live_for(&n));
    assert_eq!(certificate.lattice_cell, tape_u64(0));

    let mut runtime = start.clone();
    let runtime_closure = loop {
        match runtime.descend() {
            Descent::B(next) => runtime = next,
            Descent::T(closed) => break closed,
            Descent::N(_) => panic!("factoring certificate runtime unexpectedly exposed N"),
            Descent::F => panic!("validated certificate runtime unexpectedly exposed F"),
        }
    };
    assert_eq!(runtime_closure.lattice_cell, certificate.lattice_cell);
    assert_eq!(runtime_closure.imscription, certificate.terminal_imscription);
    assert_eq!(runtime_closure.carrier.encode(), certificate.terminal_carrier);

    for (index, wire) in certificate.objects.iter().enumerate() {
        let object = DialecticObject::decode(wire).unwrap();
        assert_eq!(object.n, n);
        assert_eq!(object.imscription.rwx.rights, IM_RWX);
        assert_eq!(object.imscription.rwx.read_bulk, object.n);
        assert!(object.imscription.relation_is_live_for(&object.n));
        let expected_span = match index {
            0 => SHORT_FRONTIER_SPAN,
            1 => EXTENDED_FERMAT_SPAN,
            _ => LEHMAN_LOCAL_SPAN,
        };
        assert_eq!(object.span(), &tape_u64(expected_span));
    }

    // Forge one intermediate Lehman object as a locally valid different
    // imscription: move the authoritative write endpoint to k+1. Exact descent
    // continuity, not local validation, must reject it.
    let mut bad_boundary = certificate.clone();
    let mut forged_object = DialecticObject::decode(&bad_boundary.objects[4]).unwrap();
    forged_object.imscription.rwx.write_boundary =
        add(forged_object.boundary(), &tape_u64(1));
    forged_object.validate().unwrap();
    bad_boundary.objects[4] = forged_object.encode();
    assert!(verify_dialectic_certificate(&bad_boundary).is_err());

    let mut bad_order = certificate.clone();
    bad_order.objects.swap(4, 5);
    assert!(verify_dialectic_certificate(&bad_order).is_err());

    let mut bad_terminal = certificate.clone();
    bad_terminal.terminal_support ^= 1 << 5;
    assert!(verify_dialectic_certificate(&bad_terminal).is_err());

    // Each terminal r/w/x endpoint is independently load-bearing. There is no
    // second terminal boundary/span/word field whose value can mask a mutation.
    let mut bad_rights = certificate.clone();
    bad_rights.terminal_imscription.rwx.rights &= !vox::dialectic_reentry::IM_WRITE;
    assert!(verify_dialectic_certificate(&bad_rights).is_err());

    let mut bad_read = certificate.clone();
    bad_read.terminal_imscription.rwx.read_bulk[0] =
        flip(bad_read.terminal_imscription.rwx.read_bulk[0]);
    assert!(verify_dialectic_certificate(&bad_read).is_err());

    let mut bad_write = certificate.clone();
    bad_write.terminal_imscription.rwx.write_boundary[0] =
        flip(bad_write.terminal_imscription.rwx.write_boundary[0]);
    let bad_write_wire = encode_dialectic_certificate(&bad_write);
    let decoded_bad_write = decode_dialectic_certificate(&bad_write_wire).unwrap();
    assert_eq!(
        decoded_bad_write.terminal_imscription.rwx.write_boundary,
        bad_write.terminal_imscription.rwx.write_boundary,
    );
    assert!(verify_dialectic_certificate(&decoded_bad_write).is_err());

    let mut bad_execute_span = certificate.clone();
    bad_execute_span.terminal_imscription.rwx.execute_span[0] =
        flip(bad_execute_span.terminal_imscription.rwx.execute_span[0]);
    assert!(verify_dialectic_certificate(&bad_execute_span).is_err());

    let mut bad_execute_word = certificate.clone();
    bad_execute_word.terminal_imscription.rwx.execute_word[0] = '⋈';
    assert!(verify_dialectic_certificate(&bad_execute_word).is_err());

    let mut bad_wide_cell = certificate.clone();
    let mut wide_cell = vec![EVALT; usize::BITS as usize + 2];
    *wide_cell.last_mut().unwrap() = EVALF;
    bad_wide_cell.lattice_cell = wide_cell.clone();
    let wide_wire = encode_dialectic_certificate(&bad_wide_cell);
    let decoded_wide = decode_dialectic_certificate(&wide_wire).unwrap();
    assert_eq!(decoded_wide.lattice_cell, wide_cell);
    assert!(verify_dialectic_certificate(&decoded_wide).is_err());

    // The terminal Lehman multiplier is a distinct structural coordinate. A
    // valid carrier and terminal r/w/x relation cannot be relabelled as a
    // closure on another multiplier lattice.
    assert_eq!(certificate.lehman_multiplier.as_deref(), Some(tape_u64(10).as_slice()));
    let mut bad_multiplier = certificate.clone();
    bad_multiplier.lehman_multiplier = Some(tape_u64(9));
    let bad_multiplier_wire = encode_dialectic_certificate(&bad_multiplier);
    let decoded_bad_multiplier = decode_dialectic_certificate(&bad_multiplier_wire).unwrap();
    assert_eq!(decoded_bad_multiplier.lehman_multiplier, bad_multiplier.lehman_multiplier);
    assert!(verify_dialectic_certificate(&decoded_bad_multiplier).is_err());

    let mut missing_multiplier = certificate.clone();
    missing_multiplier.lehman_multiplier = None;
    assert!(verify_dialectic_certificate(&missing_multiplier).is_err());

    println!(
        "dialectic certificate: {} persisted whole objects replay exact FOUR-preserving descents; runtime/certificate imscription identical; forged continuity/support/rwx-legs, lattice cell, and Lehman multiplier rejected semantically",
        summary.descents,
    );
}
