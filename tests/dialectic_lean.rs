use vox::dialectic_lean::ResidentDialectic;
use vox::dialectic_reentry::{Descent, DialecticObject};
use vox::morphism_factor::{cmp, mul, one, tape_u64};
use core::cmp::Ordering;

#[test]
fn lean_y_reenters_the_same_source_and_preserves_recovered_closures() {
    for (p, q) in [(7, 13), (83, 97), (1_000_003, 1_032_007), (101, 211)] {
        let n = mul(&tape_u64(p), &tape_u64(q));
        let mut original = DialecticObject::new(n.clone()).unwrap();
        let baseline = loop {
            match original.descend() {
                Descent::T(result) => break result,
                Descent::B(next) => original = next,
                other => panic!("baseline FOUR={}", other.four()),
            }
        };
        let mut resident = ResidentDialectic::new(n.clone()).unwrap();
        let mut last_support = resident.object().unwrap().support;
        for _ in 0..100 {
            if resident.advance().unwrap() { break; }
            let object = resident.object().unwrap();
            assert_eq!(object.n, n);
            assert!(object.imscription.relation_is_live_for(&n));
            assert_eq!(object.support & last_support, last_support);
            last_support = object.support;
        }
        let closure = resident.closure().expect("resident source closure");
        let frame = resident.scan.last_frame.as_ref().unwrap();
        assert_eq!(mul(&frame.a, &frame.a),
            vox::morphism_factor::add(&frame.source, &frame.residual));
        assert_eq!(mul(&closure.carrier.p, &closure.carrier.q), n);
        assert_eq!(mul(&baseline.carrier.p, &baseline.carrier.q), n);
        assert_eq!(cmp(&closure.carrier.p, &one()), Ordering::Greater);
        assert_eq!(cmp(&closure.carrier.p, &n), Ordering::Less);
        assert!(closure.imscription.relation_is_live_for(&n));
        assert_eq!(resident.operations.temporal_ticks, resident.operations.descents);
        assert_eq!(resident.operations.information_joins, resident.operations.descents);
        assert!(resident.operations.contractions > resident.operations.descents);
        assert_eq!(resident.scan.cells, resident.scan.square_tests + resident.scan.rejected_cells);
        if resident.operations.descents > 1 {
            assert!(resident.operations.reversed_descents > 0);
            assert!(resident.scan.residual_advances > 0);
            assert!(resident.scan.rejected_cells > 0);
        }
    }
}

#[test]
fn both_temporal_phases_preserve_every_square_residue() {
    // Across this source battery, the clock selects both traversal directions.
    // The emitted pair is checked against N after production, without seeds.
    for n in [91, 143, 221, 437, 8051, 10403] {
      for phase in [2, 4] {
        let source = tape_u64(n);
        let mut resident = ResidentDialectic::new(source.clone()).unwrap();
        resident.phase = phase;
        while !resident.advance().unwrap() {}
        let closure = resident.closure().unwrap();
        assert_eq!(mul(&closure.carrier.p, &closure.carrier.q), source);
      }
    }
}

#[test]
fn compact_y_tail_keeps_contraction_work_linear_and_retains_full_residual() {
    let n = vox::morphism_factor::decimal_to_tape(include_str!(
        "../measurements/dialectic_lean_20261012/inputs/fresh_balanced_224.txt").trim()).unwrap();
    let mut resident = ResidentDialectic::new(n).unwrap();
    let mut prefix = None;
    for _ in 0..32 {
        let done = resident.advance().unwrap();
        let retained_prefix = resident.operations.contractions - resident.operations.descents;
        assert_eq!(*prefix.get_or_insert(retained_prefix), retained_prefix);
        let frame = resident.scan.last_frame.as_ref().unwrap();
        assert_eq!(mul(&frame.a, &frame.a),
            vox::morphism_factor::add(&frame.source, &frame.residual));
        if done { break; }
    }
}
