use core::cmp::Ordering;

use vox::dialectic_reentry::{
    Descent, DialecticObject, EXTENDED_FERMAT_WORD, SHORT_FRONTIER_WORD,
};
use vox::factor_extract::extract;
use vox::morphism_factor::{cmp, mul, tape_u64};
use vox::producer_provenance::{
    SUPPORT_EXTENDED_FERMAT, SUPPORT_PARITY, SUPPORT_PRIMALITY,
    SUPPORT_PRODUCT_BOUNDARY, SUPPORT_SHORT_FRONTIER,
};
use vox::reentry_certificate::{certify_reentry, verify_reentry_certificate};
use vox::trace_algebra::witness_valid;
use vox::vox::{verdict, IMSCRIB};

fn same_pair(a: &[char], b: &[char], x: &[char], y: &[char]) -> bool {
    (cmp(a, x) == Ordering::Equal && cmp(b, y) == Ordering::Equal)
        || (cmp(a, y) == Ordering::Equal && cmp(b, x) == Ordering::Equal)
}

#[test]
fn operator_consumes_frontier_then_reimscribes_from_lattice_cell_64() {
    let p = tape_u64(1_000_003);
    let q = tape_u64(1_032_007); // measured actual gap 32,004: beyond short frontier
    let n = mul(&p, &q);

    let first = DialecticObject::new(n.clone()).unwrap();
    assert_eq!(first.word, SHORT_FRONTIER_WORD.chars().collect::<Vec<_>>());
    assert_eq!(verdict(&first.word), 'B');
    assert!(first.word.contains(&IMSCRIB));
    assert_eq!(first.support, SUPPORT_PARITY | SUPPORT_PRIMALITY);
    assert_eq!(DialecticObject::decode(&first.encode()).unwrap(), first);

    let first_support = first.support;
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

    // Simulated process death: only the marks survive between levels.
    let persisted = second.encode();
    drop(second);
    let restarted = DialecticObject::decode(&persisted).unwrap();

    let closure = match restarted.descend().unwrap() {
        Descent::Closed(closed) => closed,
        Descent::Continue(_) => panic!("extended Fermat inscription failed to close measured near-root pair"),
    };

    assert_eq!(verdict(&closure.word), 'T');
    assert!(closure.word.contains(&IMSCRIB));
    assert!(closure.lattice_cell >= 64, "continuation replayed the short frontier");
    assert!(closure.support & SUPPORT_SHORT_FRONTIER != 0);
    assert!(closure.support & SUPPORT_EXTENDED_FERMAT != 0);
    assert!(closure.support & SUPPORT_PRODUCT_BOUNDARY != 0);
    assert!(witness_valid(&closure.carrier.n, &closure.carrier.p, &closure.carrier.q));
    assert!(same_pair(&closure.carrier.p, &closure.carrier.q, &p, &q));

    let readout = extract(&closure.carrier).unwrap();
    assert_eq!(readout.transforms, 0);
    assert_eq!(readout.normal_form, closure.carrier.trace);
    let summary = verify_reentry_certificate(&certify_reentry(&closure.carrier).unwrap()).unwrap();
    assert_eq!(summary.transforms, 0);
    assert_eq!(summary.normal_form, closure.carrier.trace);

    println!(
        "dialectic descent: gap=32004 consumed frontier B, re-inscribed at cell 64, closed T at lattice cell {}",
        closure.lattice_cell,
    );
}

#[test]
fn operator_closes_without_descent_when_first_inscribed_space_affords_the_pair() {
    let p = tape_u64(1_000_003);
    let q = tape_u64(1_001_003); // measured gap 1,000: frontier route
    let n = mul(&p, &q);

    let object = DialecticObject::new(n).unwrap();
    let closure = match object.descend().unwrap() {
        Descent::Closed(closed) => closed,
        Descent::Continue(_) => panic!("1,000-gap pair should close in the short frontier inscription"),
    };

    assert_eq!(verdict(&closure.word), 'T');
    assert!(closure.lattice_cell < 64);
    assert_eq!(closure.support & (SUPPORT_PARITY | SUPPORT_PRIMALITY), SUPPORT_PARITY | SUPPORT_PRIMALITY);
    assert!(closure.support & SUPPORT_SHORT_FRONTIER != 0);
    assert!(closure.support & SUPPORT_PRODUCT_BOUNDARY != 0);
    assert_eq!(closure.support & SUPPORT_EXTENDED_FERMAT, 0);
    assert!(same_pair(&closure.carrier.p, &closure.carrier.q, &p, &q));

    println!(
        "dialectic descent: gap=1000 locked in first inscription at lattice cell {}",
        closure.lattice_cell,
    );
}

#[test]
fn persisted_operator_space_rejects_a_support_word_mismatch() {
    let p = tape_u64(1_000_003);
    let q = tape_u64(1_032_007);
    let n = mul(&p, &q);
    let first = DialecticObject::new(n).unwrap();
    let second = match first.descend().unwrap() {
        Descent::Continue(next) => next,
        Descent::Closed(_) => panic!("fixture unexpectedly closed in frontier"),
    };

    let mut corrupted = second.encode();
    // Framing is: ⊢ ⊙ ∈N∋ ∈six support marks∋ word ⊣.
    // Find the support field after the N field and clear the short-frontier bit.
    let mut i = 2usize;
    assert_eq!(corrupted[i], '∈');
    i += 1;
    while corrupted[i] != '∋' { i += 1; }
    i += 1;
    assert_eq!(corrupted[i], '∈');
    i += 1;
    let short_bit = 2usize;
    corrupted[i + short_bit] = vox::vox::EVALT;

    assert!(DialecticObject::decode(&corrupted).is_err());
}
