use core::cmp::Ordering;

use vox::dialectic_reentry::{
    Descent, DialecticObject, EXTENDED_FERMAT_SPAN, EXTENDED_FERMAT_WORD, IM_RWX,
    LEHMAN_LOCAL_SPAN, LEHMAN_WORD, SHORT_FRONTIER_SPAN, SHORT_FRONTIER_WORD,
};
use vox::factor_extract::extract;
use vox::morphism_factor::{add, cmp, dec_of, mul, tape_u64};
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

fn flip(mark: char) -> char {
    if mark == EVALF { EVALT } else { EVALF }
}

fn assert_live_relation(object: &DialecticObject) {
    assert_eq!(object.imscription.rwx.rights, IM_RWX);
    assert_eq!(object.imscription.rwx.read_bulk, object.n);
    assert_eq!(object.imscription.rwx.write_boundary, object.imscription.boundary);
    assert_eq!(object.imscription.rwx.execute_span, object.imscription.span);
    assert_eq!(object.imscription.rwx.execute_word, object.word);
    assert!(object.imscription.relation_is_live_for(&object.n, &object.word));
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
    assert_eq!(first.imscription.span, tape_u64(SHORT_FRONTIER_SPAN));
    assert_live_relation(&first);
    assert_eq!(DialecticObject::decode(&first.encode()).unwrap(), first);

    let first_support = first.support;
    let first_boundary = first.imscription.boundary.clone();
    let first_relation = first.imscription.rwx.clone();
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
    assert_eq!(second.imscription.span, tape_u64(EXTENDED_FERMAT_SPAN));
    assert_live_relation(&second);

    // All capabilities remain live, but the relation has transformed: the same
    // bulk is now coupled to a different written boundary, span and IMASM word.
    assert_eq!(second.imscription.rwx.rights, first_relation.rights);
    assert_eq!(second.imscription.rwx.read_bulk, first_relation.read_bulk);
    assert_ne!(second.imscription.rwx.write_boundary, first_relation.write_boundary);
    assert_ne!(second.imscription.rwx.execute_span, first_relation.execute_span);
    assert_ne!(second.imscription.rwx.execute_word, first_relation.execute_word);
    assert_ne!(second.imscription.rwx, first_relation);

    // The consumed frontier is material in the next object: its boundary is the
    // first unwalked lattice point, exactly 64 cells beyond the old boundary.
    let expected_second_boundary = add(&first_boundary, &tape_u64(SHORT_FRONTIER_SPAN));
    assert_eq!(second.imscription.boundary, expected_second_boundary);

    // Simulated process death: only the marks survive between levels, including
    // every endpoint of the dynamic bulk/boundary r/w/x relation.
    let persisted_boundary = second.imscription.boundary.clone();
    let persisted_span = second.imscription.span.clone();
    let persisted_relation = second.imscription.rwx.clone();
    let persisted = second.encode();
    drop(second);
    let restarted = DialecticObject::decode(&persisted).unwrap();
    assert_eq!(restarted.imscription.boundary, persisted_boundary);
    assert_eq!(restarted.imscription.span, persisted_span);
    assert_eq!(restarted.imscription.rwx, persisted_relation);
    assert_live_relation(&restarted);

    let closure = match restarted.descend().unwrap() {
        Descent::Closed(closed) => closed,
        Descent::Continue(_) => panic!("extended Fermat imscription failed to close measured near-root pair"),
    };

    assert_eq!(verdict(&closure.word), 'T');
    assert!(closure.word.contains(&IMSCRIB));
    assert_eq!(closure.lattice_cell, tape_u64(126));
    assert!(closure.lehman_multiplier.is_none());
    assert_eq!(closure.imscription.span, tape_u64(EXTENDED_FERMAT_SPAN));
    assert_eq!(closure.imscription.rwx.rights, IM_RWX);
    assert_eq!(closure.imscription.rwx.read_bulk, closure.carrier.n);
    assert_eq!(closure.imscription.rwx.write_boundary, closure.imscription.boundary);
    assert_eq!(closure.imscription.rwx.execute_span, closure.imscription.span);
    assert_eq!(closure.imscription.rwx.execute_word, closure.word);
    assert!(closure.imscription.relation_is_live_for(&closure.carrier.n, &closure.word));
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
        "dialectic descent: gap=32004 transformed the full r/w/x relation, restarted from it, and closed T at tape cell {}",
        dec_of(&closure.lattice_cell),
    );
}

#[test]
fn operator_closes_without_descent_when_first_imscribed_space_affords_the_pair() {
    let p = tape_u64(1_000_003);
    let q = tape_u64(1_001_003); // measured gap 1,000: frontier route
    let n = mul(&p, &q);

    let object = DialecticObject::new(n).unwrap();
    let initial_boundary = object.imscription.boundary.clone();
    assert_eq!(object.imscription.span, tape_u64(SHORT_FRONTIER_SPAN));
    assert_live_relation(&object);
    let closure = match object.descend().unwrap() {
        Descent::Closed(closed) => closed,
        Descent::Continue(_) => panic!("1,000-gap pair should close in the short-frontier imscription"),
    };

    assert_eq!(verdict(&closure.word), 'T');
    assert_eq!(closure.lattice_cell, tape_u64(0));
    assert!(closure.lehman_multiplier.is_none());
    assert_eq!(closure.imscription.boundary, initial_boundary);
    assert_eq!(closure.imscription.span, tape_u64(SHORT_FRONTIER_SPAN));
    assert_eq!(closure.imscription.rwx.rights, IM_RWX);
    assert_eq!(closure.imscription.rwx.read_bulk, closure.carrier.n);
    assert_eq!(closure.imscription.rwx.write_boundary, closure.imscription.boundary);
    assert_eq!(closure.imscription.rwx.execute_span, closure.imscription.span);
    assert_eq!(closure.imscription.rwx.execute_word, closure.word);
    assert!(closure.imscription.relation_is_live_for(&closure.carrier.n, &closure.word));
    assert_eq!(closure.support & (SUPPORT_PARITY | SUPPORT_PRIMALITY), SUPPORT_PARITY | SUPPORT_PRIMALITY);
    assert!(closure.support & SUPPORT_SHORT_FRONTIER != 0);
    assert!(closure.support & SUPPORT_PRODUCT_BOUNDARY != 0);
    assert_eq!(closure.support & SUPPORT_EXTENDED_FERMAT, 0);
    assert!(same_pair(&closure.carrier.p, &closure.carrier.q, &p, &q));

    println!(
        "dialectic descent: gap=1000 locked in first imscription at tape lattice cell {}",
        dec_of(&closure.lattice_cell),
    );
}

#[test]
fn operator_changes_lattice_and_reenters_until_lehman_locks_the_pair() {
    // This pair lies beyond the 4096-cell Fermat region. The next whole object
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
    assert_eq!(extended.imscription.span, tape_u64(EXTENDED_FERMAT_SPAN));
    assert_eq!(verdict(&extended.word), 'B');
    assert_live_relation(&extended);

    // The second Fermat region is consumed as one complete imscription. Its B
    // changes what the boundary denotes and rewrites the relation around k=1.
    let lehman = match extended.descend().unwrap() {
        Descent::Continue(next) => next,
        Descent::Closed(_) => panic!("far-gap fixture unexpectedly closed in Fermat ring"),
    };
    assert_eq!(lehman.word, LEHMAN_WORD.chars().collect::<Vec<_>>());
    assert_eq!(verdict(&lehman.word), 'B');
    assert_eq!(lehman.support, SUPPORT_PARITY | SUPPORT_PRIMALITY | SUPPORT_SHORT_FRONTIER | SUPPORT_EXTENDED_FERMAT);
    assert_eq!(lehman.imscription.boundary, tape_u64(1));
    assert_eq!(lehman.imscription.span, tape_u64(LEHMAN_LOCAL_SPAN));
    assert_live_relation(&lehman);

    let mut current = lehman;
    let closure = loop {
        // Every multiplier is a complete marks-only restart point. The write leg
        // itself moves with k even though rights, span and word remain unchanged.
        let persisted = current.encode();
        drop(current);
        let restarted = DialecticObject::decode(&persisted).unwrap();
        assert_live_relation(&restarted);
        let k = restarted.imscription.boundary.clone();
        let relation = restarted.imscription.rwx.clone();

        match restarted.descend().unwrap() {
            Descent::Continue(next) => {
                assert_eq!(next.word, LEHMAN_WORD.chars().collect::<Vec<_>>());
                assert_eq!(next.imscription.boundary, add(&k, &tape_u64(1)));
                assert_eq!(next.imscription.span, relation.execute_span);
                assert_eq!(next.imscription.rwx.rights, relation.rights);
                assert_eq!(next.imscription.rwx.read_bulk, relation.read_bulk);
                assert_eq!(next.imscription.rwx.execute_span, relation.execute_span);
                assert_eq!(next.imscription.rwx.execute_word, relation.execute_word);
                assert_ne!(next.imscription.rwx.write_boundary, relation.write_boundary);
                assert_eq!(next.imscription.rwx.write_boundary, next.imscription.boundary);
                assert_ne!(next.imscription.rwx, relation);
                current = next;
            }
            Descent::Closed(closed) => break closed,
        }
    };

    assert_eq!(verdict(&closure.word), 'T');
    assert_eq!(closure.lattice_cell, tape_u64(0));
    assert_eq!(closure.lehman_multiplier.as_deref(), Some(tape_u64(10).as_slice()));
    assert_eq!(closure.imscription.boundary, tape_u64(10));
    assert_eq!(closure.imscription.span, tape_u64(LEHMAN_LOCAL_SPAN));
    assert_eq!(closure.imscription.rwx.rights, IM_RWX);
    assert_eq!(closure.imscription.rwx.read_bulk, closure.carrier.n);
    assert_eq!(closure.imscription.rwx.write_boundary, closure.imscription.boundary);
    assert_eq!(closure.imscription.rwx.execute_span, closure.imscription.span);
    assert_eq!(closure.imscription.rwx.execute_word, closure.word);
    assert!(closure.imscription.relation_is_live_for(&closure.carrier.n, &closure.word));
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
        "dialectic tower: far-gap semiprime rewrote the r/w/x relation on every re-imscription -> T at k=10, tape cell={}; restored support={}",
        dec_of(&closure.lattice_cell),
        closure.support,
    );
}

#[test]
fn persisted_operator_space_rejects_boundary_span_rwx_relation_or_support_mismatch() {
    let p = tape_u64(1_000_003);
    let q = tape_u64(1_032_007);
    let n = mul(&p, &q);
    let first = DialecticObject::new(n).unwrap();
    let second = match first.descend().unwrap() {
        Descent::Continue(next) => next,
        Descent::Closed(_) => panic!("fixture unexpectedly closed in frontier"),
    };
    assert_live_relation(&second);

    let mut bad_boundary = second.clone();
    bad_boundary.imscription.boundary[0] = flip(bad_boundary.imscription.boundary[0]);
    assert!(DialecticObject::decode(&bad_boundary.encode()).is_err());

    let mut bad_span = second.clone();
    bad_span.imscription.span[0] = flip(bad_span.imscription.span[0]);
    assert!(DialecticObject::decode(&bad_span.encode()).is_err());

    let mut bad_rights = second.clone();
    bad_rights.imscription.rwx.rights &= !vox::dialectic_reentry::IM_WRITE;
    assert!(DialecticObject::decode(&bad_rights.encode()).is_err());

    let mut bad_read = second.clone();
    bad_read.imscription.rwx.read_bulk[0] = flip(bad_read.imscription.rwx.read_bulk[0]);
    assert!(DialecticObject::decode(&bad_read.encode()).is_err());

    let mut bad_write = second.clone();
    bad_write.imscription.rwx.write_boundary[0] = flip(bad_write.imscription.rwx.write_boundary[0]);
    assert!(DialecticObject::decode(&bad_write.encode()).is_err());

    let mut bad_execute_span = second.clone();
    bad_execute_span.imscription.rwx.execute_span[0] =
        flip(bad_execute_span.imscription.rwx.execute_span[0]);
    assert!(DialecticObject::decode(&bad_execute_span.encode()).is_err());

    let mut bad_execute_word = second.clone();
    bad_execute_word.imscription.rwx.execute_word = SHORT_FRONTIER_WORD.chars().collect();
    assert!(DialecticObject::decode(&bad_execute_word.encode()).is_err());

    let mut bad_support = second;
    bad_support.support ^= SUPPORT_SHORT_FRONTIER;
    assert!(DialecticObject::decode(&bad_support.encode()).is_err());

    println!(
        "dialectic r/w/x relation: boundary/span/rights/read/write/execute/support forgeries all rejected",
    );
}
