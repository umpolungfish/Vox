use core::cmp::Ordering;

use vox::dialectic_reentry::{
    DialecticObject, EXTENDED_FERMAT_SPAN, IM_RWX, LEHMAN_LOCAL_SPAN,
    SHORT_FRONTIER_SPAN,
};
use vox::imscription_cycle::{
    certify_imscription_cycle, decode_imscription_cycle, encode_imscription_cycle,
    verify_imscription_cycle,
};
use vox::morphism_factor::{
    add, cmp, dec_of, decimal_to_tape, miller_rabin, mul, tape_u64,
};
use vox::trace_algebra::witness_valid;

fn same_pair(a: &[char], b: &[char], x: &[char], y: &[char]) -> bool {
    (cmp(a, x) == Ordering::Equal && cmp(b, y) == Ordering::Equal)
        || (cmp(a, y) == Ordering::Equal && cmp(b, x) == Ordering::Equal)
}

fn next_prime_tape(mut value: Vec<char>) -> Vec<char> {
    let two = tape_u64(2);
    while !miller_rabin(&value) {
        value = add(&value, &two);
    }
    value
}

fn next_prime_u64(mut value: u64) -> u64 {
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

fn assert_live_relation(object: &DialecticObject, n: &[char]) {
    assert_eq!(object.n, n);
    assert_eq!(object.imscription.rwx.rights, IM_RWX);
    assert_eq!(object.imscription.rwx.read_bulk, n);
    assert_eq!(object.imscription.rwx.write_boundary, object.imscription.boundary);
    assert_eq!(object.imscription.rwx.execute_span, object.imscription.span);
    assert_eq!(object.imscription.rwx.execute_word, object.word);
    assert!(object.imscription.relation_is_live_for(&object.n, &object.word));
}

#[test]
fn baked_forty_one_digit_semiprime_closes_as_a_native_tape_cycle() {
    let p = decimal_to_tape("100000000000000000039").unwrap();
    let q = decimal_to_tape("100000000000000000129").unwrap();
    let n = decimal_to_tape("10000000000000000016800000000000000005031").unwrap();
    assert_eq!(mul(&p, &q), n);
    assert!(n.len() > 64, "fixture unexpectedly narrowed below machine width");

    let start = DialecticObject::new(n.clone()).unwrap();
    assert_eq!(start.imscription.span, tape_u64(SHORT_FRONTIER_SPAN));
    assert_live_relation(&start, &n);
    let certificate = certify_imscription_cycle(&start).unwrap();
    let summary = verify_imscription_cycle(&certificate).unwrap();

    assert_eq!(summary.descents, 1);
    assert_eq!(summary.terminal_support, 39);
    assert_eq!(summary.quotient_transforms, 1);
    assert_eq!(summary.quotient_generations, 2);
    assert_eq!(certificate.dialectic.terminal_span, tape_u64(SHORT_FRONTIER_SPAN));
    assert_eq!(certificate.dialectic.terminal_rwx.rights, IM_RWX);
    assert_eq!(certificate.dialectic.terminal_rwx.read_bulk, n);
    assert_eq!(
        certificate.dialectic.terminal_rwx.write_boundary,
        certificate.dialectic.terminal_boundary,
    );
    assert_eq!(
        certificate.dialectic.terminal_rwx.execute_span,
        certificate.dialectic.terminal_span,
    );
    assert_eq!(
        certificate.dialectic.terminal_rwx.execute_word,
        certificate.dialectic.terminal_word,
    );
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

    let wire = encode_imscription_cycle(&certificate);
    let restored = decode_imscription_cycle(&wire).unwrap();
    let replay = verify_imscription_cycle(&restored).unwrap();
    assert_eq!(restored.dialectic.terminal_span, tape_u64(SHORT_FRONTIER_SPAN));
    assert_eq!(restored.dialectic.terminal_rwx, certificate.dialectic.terminal_rwx);
    assert_eq!(replay.fixed_carrier.encode(), summary.fixed_carrier.encode());

    println!(
        "imscription arbitrary width: baked N digits={} tape_marks={} dynamic_rwx=true span={} descents=1 support=39 fixed=true",
        dec_of(&n).len(),
        n.len(),
        SHORT_FRONTIER_SPAN,
    );
}

#[test]
fn arbitrary_width_bulk_changes_lattice_without_narrowing_the_boundary() {
    let p = decimal_to_tape("100000000000000000039").unwrap();
    assert!(miller_rabin(&p));

    // q is generated entirely as a tape near 2p. For q=2p+delta, the k=2
    // Lehman lattice is near-square and should close at its first local cell.
    let two_p = add(&p, &p);
    let seed = add(&two_p, &tape_u64(1));
    let q = next_prime_tape(seed);
    let n = mul(&p, &q);
    assert!(n.len() > 64);

    let start = DialecticObject::new(n.clone()).unwrap();
    let certificate = certify_imscription_cycle(&start).unwrap();
    let summary = verify_imscription_cycle(&certificate).unwrap();

    assert_eq!(summary.terminal_support, 63);
    assert_eq!(summary.descents, 4, "wide q≈2p fixture should consume short + extended + k1 + k2");
    assert_eq!(summary.quotient_transforms, 4);
    assert_eq!(certificate.dialectic.lehman_multiplier.as_deref(), Some(tape_u64(2).as_slice()));
    assert_eq!(certificate.dialectic.lattice_cell, tape_u64(0));
    assert_eq!(certificate.dialectic.terminal_span, tape_u64(LEHMAN_LOCAL_SPAN));

    let objects: Vec<DialecticObject> = certificate
        .dialectic
        .objects
        .iter()
        .map(|wire| DialecticObject::decode(wire).unwrap())
        .collect();
    for object in &objects {
        assert_live_relation(object, &n);
        assert!(object.imscription.rwx.read_bulk.len() > 64);
    }
    let spans: Vec<Vec<char>> = objects
        .iter()
        .map(|object| object.imscription.span.clone())
        .collect();
    assert_eq!(
        spans,
        vec![
            tape_u64(SHORT_FRONTIER_SPAN),
            tape_u64(EXTENDED_FERMAT_SPAN),
            tape_u64(LEHMAN_LOCAL_SPAN),
            tape_u64(LEHMAN_LOCAL_SPAN),
        ],
    );
    assert_ne!(objects[0].imscription.rwx, objects[1].imscription.rwx);
    assert_ne!(objects[1].imscription.rwx, objects[2].imscription.rwx);
    assert_eq!(objects[2].imscription.rwx.write_boundary, tape_u64(1));
    assert_eq!(objects[3].imscription.rwx.write_boundary, tape_u64(2));
    assert_eq!(objects[2].imscription.rwx.execute_span, objects[3].imscription.rwx.execute_span);
    assert_eq!(objects[2].imscription.rwx.execute_word, objects[3].imscription.rwx.execute_word);
    assert_ne!(objects[2].imscription.rwx, objects[3].imscription.rwx);

    assert_eq!(certificate.dialectic.terminal_rwx.read_bulk, n);
    assert_eq!(certificate.dialectic.terminal_rwx.write_boundary, tape_u64(2));
    assert_eq!(
        certificate.dialectic.terminal_rwx.execute_span,
        tape_u64(LEHMAN_LOCAL_SPAN),
    );
    assert_eq!(
        certificate.dialectic.terminal_rwx.execute_word,
        certificate.dialectic.terminal_word,
    );

    assert!(same_pair(
        &summary.fixed_carrier.p,
        &summary.fixed_carrier.q,
        &p,
        &q,
    ));

    let wire = encode_imscription_cycle(&certificate);
    let restored = decode_imscription_cycle(&wire).unwrap();
    let replay = verify_imscription_cycle(&restored).unwrap();
    assert_eq!(restored.dialectic.terminal_span, tape_u64(LEHMAN_LOCAL_SPAN));
    assert_eq!(restored.dialectic.terminal_rwx, certificate.dialectic.terminal_rwx);
    assert_eq!(replay.fixed_carrier.encode(), summary.fixed_carrier.encode());

    println!(
        "imscription arbitrary-width lattice change: p={} q={} N_digits={} rwx_read_marks={} spans=64->4032->64->64 writes=Fermat->Fermat->1->2 descents={} k=2 support=63 fixed=true",
        dec_of(&p),
        dec_of(&q),
        dec_of(&n).len(),
        n.len(),
        summary.descents,
    );
}

#[test]
fn lehman_boundary_reimscribes_past_the_old_sixty_four_multiplier_wall() {
    let p_value = next_prime_u64(1_000_000);
    let k = 96u64;
    let q_value = next_prime_u64(
        p_value
            .checked_mul(k)
            .expect("multiplier fixture overflow"),
    );
    let p = tape_u64(p_value);
    let q = tape_u64(q_value);
    let n = mul(&p, &q);

    let start = DialecticObject::new(n.clone()).unwrap();
    let certificate = certify_imscription_cycle(&start).unwrap();
    let summary = verify_imscription_cycle(&certificate).unwrap();

    assert_eq!(summary.terminal_support, 63);
    assert_eq!(summary.descents, k as usize + 2);
    assert_eq!(summary.quotient_transforms, k as usize + 2);
    assert_eq!(summary.quotient_generations, k as usize + 3);
    assert_eq!(
        certificate.dialectic.lehman_multiplier.as_deref(),
        Some(tape_u64(k).as_slice()),
    );
    assert_eq!(certificate.dialectic.terminal_span, tape_u64(LEHMAN_LOCAL_SPAN));
    assert_eq!(certificate.dialectic.terminal_rwx.rights, IM_RWX);
    assert_eq!(certificate.dialectic.terminal_rwx.read_bulk, n);
    assert_eq!(certificate.dialectic.terminal_rwx.write_boundary, tape_u64(k));
    assert_eq!(certificate.dialectic.terminal_rwx.execute_span, tape_u64(LEHMAN_LOCAL_SPAN));
    assert_eq!(certificate.dialectic.terminal_rwx.execute_word, certificate.dialectic.terminal_word);
    assert!(same_pair(
        &summary.fixed_carrier.p,
        &summary.fixed_carrier.q,
        &p,
        &q,
    ));

    let wire = encode_imscription_cycle(&certificate);
    let restored = decode_imscription_cycle(&wire).unwrap();
    let replay = verify_imscription_cycle(&restored).unwrap();
    assert_eq!(replay.descents, 98);
    assert_eq!(replay.quotient_transforms, 98);
    assert_eq!(restored.dialectic.terminal_span, tape_u64(LEHMAN_LOCAL_SPAN));
    assert_eq!(restored.dialectic.terminal_rwx, certificate.dialectic.terminal_rwx);
    assert_eq!(replay.fixed_carrier.encode(), summary.fixed_carrier.encode());

    println!(
        "imscription unbounded multiplier: k=96 dynamic_write=96 span={} descents=98 quotient_transforms=98 old_k64_wall_crossed=true fixed=true marks={}",
        LEHMAN_LOCAL_SPAN,
        wire.len(),
    );
}
