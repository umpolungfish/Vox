use core::cmp::Ordering;

use vox::dialectic_reentry::{
    Descent, DialecticObject, EXTENDED_FERMAT_WORD, IM_RWX, LEHMAN_WORD,
    SHORT_FRONTIER_WORD,
};
use vox::factor_extract::extract;
use vox::morphism_factor::{add, cmp, mul, tape_u64};
use vox::producer_provenance::{
    route_provenance, SUPPORT_DEEP_ARM, SUPPORT_EXTENDED_FERMAT, SUPPORT_PARITY,
    SUPPORT_PRIMALITY, SUPPORT_PRODUCT_BOUNDARY, SUPPORT_SHORT_FRONTIER,
};
use vox::reentry_certificate::{certify_reentry, verify_reentry_certificate};
use vox::trace_algebra::witness_valid;
use vox::vox::{verdict, EVALF, EVALT, IMSCRIB};

fn same_pair(a: &[char], b: &[char], x: &[char], y: &[char]) -> bool {
    (cmp(a, x) == Ordering::Equal && cmp(b, y) == Ordering::Equal)
        || (cmp(a, y) == Ordering::Equal && cmp(b, x) == Ordering::Equal)
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
fn operator_consumes_frontier_then_reimscribes_from_persisted_boundary() {
    let p = tape_u64(1_000_003);
    let q = tape_u64(1_032_007); // measured actual gap 32,004: beyond short frontier
    let n = mul(&p, &q);

    let first = DialecticObject::new(n.clone()).unwrap();
    assert_eq!(first.word, SHORT_FRONTIER_WORD.chars().collect::<Vec<_>>());
    assert_eq!(verdict(&first.word), 'B');
    assert!(first.word.contains(&IMSCRIB));
    assert_eq!(first.support, SUPPORT_PARITY | SUPPORT_PRIMALITY);
    assert_eq!(first.imscription.rwx, IM_RWX);
    assert!(first.imscription.can_read());
    assert!(first.imscription.can_write());
    assert!(first.imscription.can_execute());
    assert_eq!(DialecticObject::decode(&first.encode()).unwrap(), first);

    let first_support = first.support;
    let first_boundary = first.imscription.boundary.clone();
    let second = match first.descend().unwrap() {
        Descent::Continue(next) => next,
        Descent::Closed(_) => panic!("32,004-gap pair closed inside the 64-cell frontier"),
    };

    assert_eq!(second.word, EXTENDED_FERMAT_WORD.chars().collect::<Vec<_>>());
    assert_eq!(verdict(&second.word), 'B');
    assert!(second.word.contains(&IMSCRIB));
    assert_eq!(second.n, n);
    assert_eq!(second.support, first_support | SUPPORT_SHORT_FRONTIER);
    assert_ne!(second.support, first_support);
    assert_eq!(second.support & first_support, first_support);
    assert_eq!(second.envelope().ladder, vec![second.support]);
    assert_eq!(second.imscription.rwx, IM_RWX);

    // The consumed frontier is material in the next object: its boundary is the
    // first unwalked lattice point, exactly 64 cells beyond the old boundary.
    let expected_second_boundary = add(&first_boundary, &tape_u64(64));
    assert_eq!(second.imscription.boundary, expected_second_boundary);

    // Simulated process death: only the marks survive between levels, including
    // the dynamic bulk/boundary r/w/x relation and the transformed boundary.
    let persisted_boundary = second.imscription.boundary.clone();
    let persisted = second.encode();
    drop(second);
    let restarted = DialecticObject::decode(&persisted).unwrap();
    assert_eq!(restarted.imscription.boundary, persisted_boundary);
    assert_eq!(restarted.imscription.rwx, IM_RWX);

    let closure = match restarted.descend().unwrap() {
        Descent::Closed(closed) => closed,
        Descent::Continue(_) => panic!("extended Fermat imscription failed to close measured near-root pair"),
    };

    assert_eq!(verdict(&closure.word), 'T');
    assert!(closure.word.contains(&IMSCRIB));
    assert_eq!(closure.lattice_cell, 126);
    assert!(closure.lehman_multiplier.is_none());
    assert_eq!(closure.imscription.rwx, IM_RWX);
    assert!(closure.support & SUPPORT_SHORT_FRONTIER != 0);
    assert!(closure.support & SUPPORT_EXTENDED_FERMAT != 0);
    assert!(closure.support & SUPPORT_PRODUCT_BOUNDARY != 0);
    assert!(witness_valid(&closure.carrier.n, &closure.carrier.p, &closure.carrier.q));
    assert!(same_pair(&closure.carrier.p, &closure.carrier.q, &p, &q));

    // The second walk begins from the persisted boundary. Cell 126 is 62 cells
    // into that imscribed continuation, not a replay from the original origin.
    let expected_closing_boundary = add(&persisted_boundary, &tape_u64(62));
    assert_eq!(closure.imscription.boundary, expected_closing_boundary);

    let readout = extract(&closure.carrier).unwrap();
    assert_eq!(readout.transforms, 0);
    assert_eq!(readout.normal_form, closure.carrier.trace);
    let summary = verify_reentry_certificate(&certify_reentry(&closure.carrier).unwrap()).unwrap();
    assert_eq!(summary.transforms, 0);
    assert_eq!(summary.normal_form, closure.carrier.trace);

    println!(
        "dialectic descent: gap=32004 consumed frontier B, wrote boundary cell 64 into the next imscription, restarted from that boundary, closed T at cell {}",
        closure.lattice_cell,
    );
}

#[test]
fn operator_closes_without_descent_when_first_imscribed_space_affords_the_pair() {
    let p = tape_u64(1_000_003);
    let q = tape_u64(1_001_003); // measured gap 1,000: frontier route
    let n = mul(&p, &q);

    let object = DialecticObject::new(n).unwrap();
    let initial_boundary = object.imscription.boundary.clone();
    assert_eq!(object.imscription.rwx, IM_RWX);
    let closure = match object.descend().unwrap() {
        Descent::Closed(closed) => closed,
        Descent::Continue(_) => panic!("1,000-gap pair should close in the short-frontier imscription"),
    };

    assert_eq!(verdict(&closure.word), 'T');
    assert_eq!(closure.lattice_cell, 0);
    assert!(closure.lehman_multiplier.is_none());
    assert_eq!(closure.imscription.boundary, initial_boundary);
    assert_eq!(closure.imscription.rwx, IM_RWX);
    assert_eq!(closure.support & (SUPPORT_PARITY | SUPPORT_PRIMALITY), SUPPORT_PARITY | SUPPORT_PRIMALITY);
    assert!(closure.support & SUPPORT_SHORT_FRONTIER != 0);
    assert!(closure.support & SUPPORT_PRODUCT_BOUNDARY != 0);
    assert_eq!(closure.support & SUPPORT_EXTENDED_FERMAT, 0);
    assert!(same_pair(&closure.carrier.p, &closure.carrier.q, &p, &q));

    println!(
        "dialectic descent: gap=1000 locked in first imscription at lattice cell {}",
        closure.lattice_cell,
    );
}

#[test]
fn operator_changes_lattice_and_reenters_until_lehman_locks_the_pair() {
    // This pair lies beyond the 4096-cell Fermat ring. The next whole object
    // changes lattice and closes on Lehman multiplier k=10.
    let p = tape_u64(1_000_003);
    let q = tape_u64(10_000_019);
    let n = mul(&p, &q);

    let first = DialecticObject::new(n.clone()).unwrap();
    let extended = match first.descend().unwrap() {
        Descent::Continue(next) => next,
        Descent::Closed(_) => panic!("far-gap fixture unexpectedly closed in short frontier"),
    };
    assert_eq!(extended.word, EXTENDED_FERMAT_WORD.chars().collect::<Vec<_>>());
    assert_eq!(verdict(&extended.word), 'B');

    // The second Fermat ring is consumed as one complete imscription. Its B does
    // not mean retry Fermat: the operator changes what its boundary denotes.
    let lehman = match extended.descend().unwrap() {
        Descent::Continue(next) => next,
        Descent::Closed(_) => panic!("far-gap fixture unexpectedly closed in Fermat ring"),
    };
    assert_eq!(lehman.word, LEHMAN_WORD.chars().collect::<Vec<_>>());
    assert_eq!(verdict(&lehman.word), 'B');
    assert_eq!(lehman.support, SUPPORT_PARITY | SUPPORT_PRIMALITY | SUPPORT_SHORT_FRONTIER | SUPPORT_EXTENDED_FERMAT);
    assert_eq!(lehman.imscription.boundary, tape_u64(1));
    assert_eq!(lehman.imscription.rwx, IM_RWX);

    let mut current = lehman;
    let closure = loop {
        // Every multiplier is a complete marks-only restart point. The boundary
        // is the multiplier itself, not a hosted loop counter outside the object.
        let persisted = current.encode();
        drop(current);
        let restarted = DialecticObject::decode(&persisted).unwrap();
        let k = restarted.imscription.boundary.clone();

        match restarted.descend().unwrap() {
            Descent::Continue(next) => {
                assert_eq!(next.word, LEHMAN_WORD.chars().collect::<Vec<_>>());
                assert_eq!(next.imscription.boundary, add(&k, &tape_u64(1)));
                current = next;
            }
            Descent::Closed(closed) => break closed,
        }
    };

    assert_eq!(verdict(&closure.word), 'T');
    assert_eq!(closure.lattice_cell, 0);
    assert_eq!(closure.lehman_multiplier.as_deref(), Some(tape_u64(10).as_slice()));
    assert_eq!(closure.imscription.boundary, tape_u64(10));
    assert_eq!(closure.imscription.rwx, IM_RWX);
    assert_eq!(
        closure.support,
        SUPPORT_PARITY
            | SUPPORT_PRIMALITY
            | SUPPORT_SHORT_FRONTIER
            | SUPPORT_EXTENDED_FERMAT
            | SUPPORT_DEEP_ARM
            | SUPPORT_PRODUCT_BOUNDARY,
    );

    // The new tower support lands on the same restored outer support already
    // occupied by the existing HARD producer provenance.
    let hard = route_provenance("HARD").unwrap();
    assert_eq!(hard.ladder[0], closure.support);

    assert!(witness_valid(&closure.carrier.n, &closure.carrier.p, &closure.carrier.q));
    assert!(same_pair(&closure.carrier.p, &closure.carrier.q, &p, &q));
    let readout = extract(&closure.carrier).unwrap();
    assert_eq!(readout.transforms, 0);
    let summary = verify_reentry_certificate(&certify_reentry(&closure.carrier).unwrap()).unwrap();
    assert_eq!(summary.transforms, 0);
    assert_eq!(summary.normal_form, closure.carrier.trace);

    println!(
        "dialectic tower: far-gap semiprime consumed frontier B -> Fermat B -> changed lattice -> Lehman B re-entry -> T at k=10; restored support={}",
        closure.support,
    );
}

#[test]
fn persisted_operator_space_rejects_boundary_rwx_or_support_mismatch() {
    let p = tape_u64(1_000_003);
    let q = tape_u64(1_032_007);
    let n = mul(&p, &q);
    let first = DialecticObject::new(n).unwrap();
    let second = match first.descend().unwrap() {
        Descent::Continue(next) => next,
        Descent::Closed(_) => panic!("fixture unexpectedly closed in frontier"),
    };

    let encoded = second.encode();
    // Framing: ⊢ ⊙ ∈N∋ ∈boundary∋ ∈rwx[3]∋ ∈support[6]∋ word ⊣.
    let mut i = 2usize;
    let _n = skip_field(&encoded, &mut i);
    let boundary = skip_field(&encoded, &mut i);
    let rwx = skip_field(&encoded, &mut i);
    let support = skip_field(&encoded, &mut i);

    let mut bad_boundary = encoded.clone();
    bad_boundary[boundary.0] = if bad_boundary[boundary.0] == EVALF { EVALT } else { EVALF };
    assert!(DialecticObject::decode(&bad_boundary).is_err());

    let mut bad_rwx = encoded.clone();
    let write_bit = 1usize;
    bad_rwx[rwx.0 + write_bit] = EVALT;
    assert!(DialecticObject::decode(&bad_rwx).is_err());

    let mut bad_support = encoded;
    let short_bit = 2usize;
    bad_support[support.0 + short_bit] = EVALT;
    assert!(DialecticObject::decode(&bad_support).is_err());
}
