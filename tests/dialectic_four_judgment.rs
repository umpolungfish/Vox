use vox::dialectic_reentry::{
    decode_imasm_execution, Descent, DialecticObject, ImscriptionLattice,
};
use vox::morphism_factor::{mul, tape_u64};

#[test]
fn four_judgment_drives_reimscription_and_closure() {
    // Near-root: the short lattice does not close, but it does transform the
    // boundary. FOUR=B therefore carries exactly the boundary written into the
    // succeeding extended-Fermat whole object.
    let near_n = mul(&tape_u64(1_000_003), &tape_u64(1_032_007));
    let short = DialecticObject::new(near_n).unwrap();
    let short_judgment = short.judge_current_imscription().unwrap();
    assert_eq!(short_judgment.lattice, ImscriptionLattice::ShortFrontier);
    assert_eq!(short_judgment.four, 'B');
    assert!(short_judgment.witness.is_none());
    let short_write = short_judgment.write_boundary.clone();

    let extended = match short.descend().unwrap() {
        Descent::Continue(next) => next,
        Descent::Closed(_) => panic!("near-root fixture unexpectedly closed in short frontier"),
    };
    assert_eq!(extended.boundary(), &short_write);

    // The next exposed relation closes. FOUR=T carries the factor witness and
    // the exact closing tape cell consumed by descend().
    let extended_judgment = extended.judge_current_imscription().unwrap();
    assert_eq!(extended_judgment.lattice, ImscriptionLattice::ExtendedFermat);
    assert_eq!(extended_judgment.four, 'T');
    let extended_witness = extended_judgment
        .witness
        .clone()
        .expect("FOUR=T did not carry the near-root factor witness");
    assert_eq!(extended_witness.lattice_cell, tape_u64(126));
    assert_eq!(extended_judgment.write_boundary, *extended.boundary());

    let extended_closed = match extended.descend().unwrap() {
        Descent::Closed(closed) => closed,
        Descent::Continue(_) => panic!("near-root fixture failed to close after FOUR=T"),
    };
    assert_eq!(extended_closed.lattice_cell, extended_witness.lattice_cell);
    assert_eq!(
        decode_imasm_execution(extended_closed.word()).unwrap().four,
        'T',
    );

    // Far-gap: exhausting extended Fermat is still productive B, but its write
    // endpoint changes lattice and becomes Lehman k=1. The next Lehman B writes
    // k+1 directly into its judgment.
    let far_n = mul(&tape_u64(1_000_003), &tape_u64(10_000_019));
    let short = DialecticObject::new(far_n).unwrap();
    let extended = match short.descend().unwrap() {
        Descent::Continue(next) => next,
        Descent::Closed(_) => panic!("far-gap fixture unexpectedly closed in short frontier"),
    };
    let change = extended.judge_current_imscription().unwrap();
    assert_eq!(change.lattice, ImscriptionLattice::ExtendedFermat);
    assert_eq!(change.four, 'B');
    assert!(change.witness.is_none());
    assert_eq!(change.write_boundary, tape_u64(1));

    let lehman1 = match extended.descend().unwrap() {
        Descent::Continue(next) => next,
        Descent::Closed(_) => panic!("far-gap fixture unexpectedly closed before Lehman"),
    };
    assert_eq!(lehman1.boundary(), &tape_u64(1));
    let k1 = lehman1.judge_current_imscription().unwrap();
    assert_eq!(k1.lattice, ImscriptionLattice::Lehman);
    assert_eq!(k1.four, 'B');
    assert!(k1.witness.is_none());
    assert_eq!(k1.write_boundary, tape_u64(2));

    let mut current = match lehman1.descend().unwrap() {
        Descent::Continue(next) => next,
        Descent::Closed(_) => panic!("far-gap fixture unexpectedly closed at k=1"),
    };
    assert_eq!(current.boundary(), &tape_u64(2));

    let far_closed = loop {
        let judgment = current.judge_current_imscription().unwrap();
        match judgment.four {
            'B' => {
                assert!(judgment.witness.is_none());
                let expected_write = judgment.write_boundary.clone();
                current = match current.descend().unwrap() {
                    Descent::Continue(next) => next,
                    Descent::Closed(_) => panic!("FOUR=B unexpectedly closed the far-gap object"),
                };
                assert_eq!(current.boundary(), &expected_write);
            }
            'T' => {
                let witness = judgment
                    .witness
                    .expect("FOUR=T did not carry the far-gap factor witness");
                assert_eq!(current.boundary(), &tape_u64(10));
                assert_eq!(judgment.write_boundary, tape_u64(10));
                assert_eq!(witness.lattice_cell, tape_u64(0));
                break match current.descend().unwrap() {
                    Descent::Closed(closed) => closed,
                    Descent::Continue(_) => panic!("FOUR=T failed to close the far-gap object"),
                };
            }
            other => panic!("validated factoring imscription exposed unexpected FOUR={other}"),
        }
    };
    assert_eq!(far_closed.lehman_multiplier.as_deref(), Some(tape_u64(10).as_slice()));
    assert_eq!(far_closed.lattice_cell, tape_u64(0));

    // Architectural guard: the arithmetic walkers no longer hide a second
    // Open/Closed logic beneath FOUR. N and F remain explicit branches rather
    // than being collapsed into B or an ordinary false value.
    let source = include_str!("../src/dialectic_reentry.rs");
    assert!(!source.contains("enum LatticeResult"));
    assert!(!source.contains("enum LehmanResult"));
    assert!(!source.contains("LatticeResult::Open"));
    assert!(!source.contains("LehmanResult::Open"));
    assert!(source.contains("match judgment.four"));
    assert!(source.contains("'N' => Err"));
    assert!(source.contains("'F' => Err"));

    println!(
        "dialectic FOUR judgment: short B writes next Fermat boundary; extended B writes Lehman k=1; Lehman B writes k+1; T carries the exact factor witness"
    );
}
