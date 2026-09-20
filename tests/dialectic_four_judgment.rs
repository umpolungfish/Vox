use vox::dialectic_reentry::{
    decode_imasm_execution, Descent, DialecticObject, ImscriptionLattice,
    EXTENDED_FERMAT_SPAN, LEHMAN_LOCAL_SPAN,
};
use vox::morphism_factor::{mul, tape_u64};

#[test]
fn four_judgment_drives_reimscription_and_closure() {
    // Near-root: short B materializes the entire succeeding extended-Fermat
    // relation before descend() consumes it.
    let near_n = mul(&tape_u64(1_000_003), &tape_u64(1_032_007));
    let short = DialecticObject::new(near_n).unwrap();
    let short_judgment = short.judge_current_imscription().unwrap();
    assert_eq!(short_judgment.four, 'B');
    assert!(short_judgment.witness.is_none());
    let short_result = short_judgment
        .resulting_imscription
        .clone()
        .expect("FOUR=B did not materialize the succeeding relation");
    let short_execution = decode_imasm_execution(short_result.word()).unwrap();
    assert_eq!(short_execution.lattice, ImscriptionLattice::ExtendedFermat);
    assert_eq!(short_execution.four, 'B');
    assert_eq!(short_result.span(), &tape_u64(EXTENDED_FERMAT_SPAN));
    let short_support = short_judgment
        .resulting_support
        .expect("FOUR=B did not materialize succeeding support");

    let extended = match short.descend().unwrap() {
        Descent::Continue(next) => next,
        Descent::Closed(_) => panic!("near-root fixture unexpectedly closed in short frontier"),
    };
    assert_eq!(extended.imscription, short_result);
    assert_eq!(extended.support, short_support);

    // Extended T materializes the entire terminal relation. descend() must use
    // that exact relation rather than reconstructing a second terminal object.
    let extended_judgment = extended.judge_current_imscription().unwrap();
    assert_eq!(extended_judgment.four, 'T');
    let extended_witness = extended_judgment
        .witness
        .clone()
        .expect("FOUR=T did not carry the near-root factor witness");
    assert_eq!(extended_witness.lattice_cell, tape_u64(126));
    let extended_terminal = extended_judgment
        .resulting_imscription
        .clone()
        .expect("FOUR=T did not materialize the terminal relation");
    let extended_terminal_execution = decode_imasm_execution(extended_terminal.word()).unwrap();
    assert_eq!(
        extended_terminal_execution.lattice,
        ImscriptionLattice::ExtendedFermat
    );
    assert_eq!(extended_terminal_execution.four, 'T');
    assert!(extended_terminal_execution.closed);
    let extended_support = extended_judgment
        .resulting_support
        .expect("FOUR=T did not materialize terminal support");

    let extended_closed = match extended.descend().unwrap() {
        Descent::Closed(closed) => closed,
        Descent::Continue(_) => panic!("near-root fixture failed to close after FOUR=T"),
    };
    assert_eq!(extended_closed.imscription, extended_terminal);
    assert_eq!(extended_closed.support, extended_support);
    assert_eq!(extended_closed.lattice_cell, extended_witness.lattice_cell);

    // Far-gap: extended B materializes the topology change itself: Lehman k=1,
    // span=64 and the Lehman IMASM word are already inside the judgment.
    let far_n = mul(&tape_u64(1_000_003), &tape_u64(10_000_019));
    let short = DialecticObject::new(far_n).unwrap();
    let extended = match short.descend().unwrap() {
        Descent::Continue(next) => next,
        Descent::Closed(_) => panic!("far-gap fixture unexpectedly closed in short frontier"),
    };
    let change = extended.judge_current_imscription().unwrap();
    assert_eq!(change.four, 'B');
    assert!(change.witness.is_none());
    let lehman1_relation = change
        .resulting_imscription
        .clone()
        .expect("extended B did not materialize the Lehman relation");
    assert_eq!(lehman1_relation.boundary(), &tape_u64(1));
    assert_eq!(lehman1_relation.span(), &tape_u64(LEHMAN_LOCAL_SPAN));
    assert_eq!(
        decode_imasm_execution(lehman1_relation.word())
            .unwrap()
            .lattice,
        ImscriptionLattice::Lehman
    );

    let lehman1 = match extended.descend().unwrap() {
        Descent::Continue(next) => next,
        Descent::Closed(_) => panic!("far-gap fixture unexpectedly closed before Lehman"),
    };
    assert_eq!(lehman1.imscription, lehman1_relation);

    // Lehman B likewise materializes the complete k+1 relation, not merely a
    // boundary that descend() later interprets.
    let k1 = lehman1.judge_current_imscription().unwrap();
    assert_eq!(k1.four, 'B');
    let k2_relation = k1
        .resulting_imscription
        .clone()
        .expect("Lehman B did not materialize k+1");
    assert_eq!(k2_relation.boundary(), &tape_u64(2));
    assert_eq!(k2_relation.span(), &tape_u64(LEHMAN_LOCAL_SPAN));
    assert_eq!(
        decode_imasm_execution(k2_relation.word()).unwrap().lattice,
        ImscriptionLattice::Lehman
    );

    let mut current = match lehman1.descend().unwrap() {
        Descent::Continue(next) => next,
        Descent::Closed(_) => panic!("far-gap fixture unexpectedly closed at k=1"),
    };
    assert_eq!(current.imscription, k2_relation);

    let far_closed = loop {
        let judgment = current.judge_current_imscription().unwrap();
        match judgment.four {
            'B' => {
                assert!(judgment.witness.is_none());
                let expected_relation = judgment
                    .resulting_imscription
                    .clone()
                    .expect("FOUR=B did not carry the next Lehman relation");
                let expected_support = judgment
                    .resulting_support
                    .expect("FOUR=B did not carry succeeding support");
                current = match current.descend().unwrap() {
                    Descent::Continue(next) => next,
                    Descent::Closed(_) => panic!("FOUR=B unexpectedly closed the far-gap object"),
                };
                assert_eq!(current.imscription, expected_relation);
                assert_eq!(current.support, expected_support);
            }
            'T' => {
                let witness = judgment
                    .witness
                    .clone()
                    .expect("FOUR=T did not carry the far-gap factor witness");
                let terminal_relation = judgment
                    .resulting_imscription
                    .clone()
                    .expect("FOUR=T did not carry the terminal Lehman relation");
                let terminal_support = judgment
                    .resulting_support
                    .expect("FOUR=T did not carry terminal support");
                assert_eq!(current.boundary(), &tape_u64(10));
                assert_eq!(terminal_relation.boundary(), &tape_u64(10));
                assert_eq!(witness.lattice_cell, tape_u64(0));
                let closed = match current.descend().unwrap() {
                    Descent::Closed(closed) => closed,
                    Descent::Continue(_) => panic!("FOUR=T failed to close the far-gap object"),
                };
                assert_eq!(closed.imscription, terminal_relation);
                assert_eq!(closed.support, terminal_support);
                break closed;
            }
            other => panic!("validated factoring imscription exposed unexpected FOUR={other}"),
        }
    };
    assert_eq!(
        far_closed.lehman_multiplier.as_deref(),
        Some(tape_u64(10).as_slice())
    );
    assert_eq!(far_closed.lattice_cell, tape_u64(0));

    // Architectural guard: descend consumes a relation that the judgment has
    // already materialized. It must not select or construct the next lattice.
    let source = include_str!("../src/dialectic_reentry.rs");
    let descend = source
        .split("pub fn descend(self)")
        .nth(1)
        .expect("descend source not found")
        .split("fn close(")
        .next()
        .unwrap();
    assert!(!descend.contains("next_after_b"));
    assert!(!descend.contains("open_word"));
    assert!(!descend.contains("closed_word"));
    assert!(!descend.contains("Imscription::active"));
    assert!(descend.contains("judgment.resulting_imscription"));
    assert!(descend.contains("judgment.resulting_support"));
    assert!(descend.contains("match judgment.four"));
    assert!(descend.contains("'N' => Err"));
    assert!(descend.contains("'F' => Err"));

    println!(
        "dialectic FOUR relation: B owns the complete succeeding IMSCRIB relation; T owns the complete terminal relation; descend only installs the judgment result"
    );
}
