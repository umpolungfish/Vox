use core::cmp::Ordering;

use vox::dialectic_reentry::DialecticObject;
use vox::factor_extract::FactorCarrier;
use vox::imscription_cycle::{
    certify_imscription_cycle, verify_imscription_cycle,
};
use vox::morphism_factor::{cmp, mul, tape_u64};
use vox::trace_algebra::witness_valid;
use vox::trace_word::decode_trace;
use vox::vox::{EVALF, EVALT};

fn same_pair(a: &[char], b: &[char], x: &[char], y: &[char]) -> bool {
    (cmp(a, x) == Ordering::Equal && cmp(b, y) == Ordering::Equal)
        || (cmp(a, y) == Ordering::Equal && cmp(b, x) == Ordering::Equal)
}

#[test]
fn complete_cycle_erases_one_history_scaffold_per_imscription_descent() {
    let cases = [
        (1_000_003u64, 1_001_003u64, 1usize, 39u32),
        (1_000_003u64, 1_032_007u64, 2usize, 47u32),
        (1_000_003u64, 10_000_019u64, 12usize, 63u32),
    ];

    for (p_u64, q_u64, expected_descents, expected_support) in cases {
        let p = tape_u64(p_u64);
        let q = tape_u64(q_u64);
        let n = mul(&p, &q);
        let start = DialecticObject::new(n.clone()).unwrap();

        let certificate = certify_imscription_cycle(&start).unwrap();
        let summary = verify_imscription_cycle(&certificate).unwrap();

        assert_eq!(summary.descents, expected_descents);
        assert_eq!(summary.terminal_support, expected_support);
        assert_eq!(summary.quotient_transforms, expected_descents);
        assert_eq!(summary.quotient_generations, expected_descents + 1);
        assert!(witness_valid(
            &summary.fixed_carrier.n,
            &summary.fixed_carrier.p,
            &summary.fixed_carrier.q,
        ));
        assert!(same_pair(
            &summary.fixed_carrier.p,
            &summary.fixed_carrier.q,
            &p,
            &q,
        ));

        // The complete radial history is present at the bridge, then the passive
        // quotient returns to the exact point-like terminal factor object.
        assert_ne!(certificate.lifted_carrier, certificate.dialectic.terminal_carrier);
        assert_eq!(summary.fixed_carrier.encode(), certificate.dialectic.terminal_carrier);

        println!(
            "imscription cycle: pair={}x{} descents={} support={} quotient_transforms={} fixed=true",
            p_u64,
            q_u64,
            summary.descents,
            summary.terminal_support,
            summary.quotient_transforms,
        );
    }
}

#[test]
fn moving_lehman_write_relation_is_embodied_in_quotient_history() {
    let p = tape_u64(1_000_003);
    let q = tape_u64(10_000_019);
    let n = mul(&p, &q);
    let start = DialecticObject::new(n).unwrap();
    let certificate = certify_imscription_cycle(&start).unwrap();
    let summary = verify_imscription_cycle(&certificate).unwrap();
    assert_eq!(summary.descents, 12);

    // objects[2] and objects[3] are consecutive Lehman k=1 and k=2 restart
    // points. Rights, read bulk, span and word are unchanged; only the write
    // endpoint moves. The quotient-facing B records must therefore differ.
    let k1 = DialecticObject::decode(&certificate.dialectic.objects[2]).unwrap();
    let k2 = DialecticObject::decode(&certificate.dialectic.objects[3]).unwrap();
    assert_eq!(k1.imscription.rwx.rights, k2.imscription.rwx.rights);
    assert_eq!(k1.imscription.rwx.read_bulk, k2.imscription.rwx.read_bulk);
    assert_eq!(k1.imscription.rwx.execute_span, k2.imscription.rwx.execute_span);
    assert_eq!(k1.imscription.rwx.execute_word, k2.imscription.rwx.execute_word);
    assert_ne!(k1.imscription.rwx.write_boundary, k2.imscription.rwx.write_boundary);
    assert_eq!(k1.imscription.rwx.write_boundary, tape_u64(1));
    assert_eq!(k2.imscription.rwx.write_boundary, tape_u64(2));

    let lifted = FactorCarrier::decode(&certificate.lifted_carrier).unwrap();
    let steps = decode_trace(&lifted.trace).unwrap();
    assert_eq!(steps.len(), summary.descents + 1);
    assert_ne!(steps[2].applied_word, steps[3].applied_word);

    println!(
        "imscription quotient relation: Lehman k=1 and k=2 keep identical rights/read/exec but distinct write-boundary scaffolds",
    );
}

#[test]
fn complete_cycle_rejects_forged_bridge_and_quotient_start() {
    let p = tape_u64(1_000_003);
    let q = tape_u64(10_000_019);
    let n = mul(&p, &q);
    let start = DialecticObject::new(n).unwrap();
    let certificate = certify_imscription_cycle(&start).unwrap();
    let summary = verify_imscription_cycle(&certificate).unwrap();
    assert_eq!(summary.descents, 12);
    assert_eq!(summary.quotient_transforms, 12);

    // Change one numeral mark in the lifted carrier. Whether or not the forged
    // carrier remains locally parseable, it is no longer the exact certified
    // projection of the imscription chain.
    let mut bad_lift = certificate.clone();
    let flip_at = bad_lift.lifted_carrier.iter().position(|&m| m == EVALF || m == EVALT).unwrap();
    bad_lift.lifted_carrier[flip_at] = if bad_lift.lifted_carrier[flip_at] == EVALF { EVALT } else { EVALF };
    assert!(verify_imscription_cycle(&bad_lift).is_err());

    // Keep the lifted carrier intact but sever the exact bridge into passive
    // self-entry by replacing the quotient's first before-trace.
    let mut bad_start = certificate.clone();
    bad_start.quotient.links[0].before = bad_start.quotient.links[0].after.clone();
    assert!(verify_imscription_cycle(&bad_start).is_err());

    // Keep both proof layers individually shaped but lie about the terminal
    // dialectic carrier; the dialectic verifier or exact fixed-object return
    // must reject the circuit.
    let mut bad_terminal = certificate.clone();
    let flip_at = bad_terminal.dialectic.terminal_carrier.iter()
        .position(|&m| m == EVALF || m == EVALT)
        .unwrap();
    bad_terminal.dialectic.terminal_carrier[flip_at] =
        if bad_terminal.dialectic.terminal_carrier[flip_at] == EVALF { EVALT } else { EVALF };
    assert!(verify_imscription_cycle(&bad_terminal).is_err());

    println!(
        "imscription cycle forgery: lifted bridge / quotient start / terminal point all rejected",
    );
}
