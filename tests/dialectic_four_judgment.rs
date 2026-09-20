use vox::dialectic_reentry::{
    decode_imasm_execution, Descent, DialecticJudgment, DialecticObject, ImasmExecution,
    ImscriptionLattice, EXTENDED_FERMAT_SPAN, LEHMAN_LOCAL_SPAN,
};
use vox::morphism_factor::{mul, tape_u64};

#[test]
fn four_judgment_variants_drive_reimscription_and_closure() {
    // Near-root: short B owns the complete succeeding extended-Fermat relation.
    let near_n = mul(&tape_u64(1_000_003), &tape_u64(1_032_007));
    let short = DialecticObject::new(near_n).unwrap();
    let short_judgment = short.judge_current_imscription().unwrap();
    assert_eq!(short_judgment.four(), 'B');
    assert!(short_judgment.witness().is_none());
    let (short_result, short_support) = match &short_judgment {
        DialecticJudgment::B {
            imscription,
            support,
        } => (imscription.clone(), *support),
        other => panic!("short frontier exposed unexpected FOUR={}", other.four()),
    };
    let short_execution = decode_imasm_execution(short_result.word()).unwrap();
    assert_eq!(
        short_execution,
        ImasmExecution::B {
            lattice: ImscriptionLattice::ExtendedFermat,
        }
    );
    assert_eq!(short_result.span(), &tape_u64(EXTENDED_FERMAT_SPAN));

    let extended = match short.descend().unwrap() {
        Descent::Continue(next) => next,
        Descent::Closed(_) => panic!("near-root fixture unexpectedly closed in short frontier"),
    };
    assert_eq!(extended.imscription, short_result);
    assert_eq!(extended.support, short_support);

    // Extended T cannot exist without all three terminal payloads because they
    // are fields of the T variant itself.
    let extended_judgment = extended.judge_current_imscription().unwrap();
    assert_eq!(extended_judgment.four(), 'T');
    let (extended_terminal, extended_support, extended_witness) = match &extended_judgment {
        DialecticJudgment::T {
            imscription,
            support,
            witness,
        } => (imscription.clone(), *support, witness.clone()),
        other => panic!("extended Fermat exposed unexpected FOUR={}", other.four()),
    };
    assert_eq!(extended_witness.lattice_cell, tape_u64(126));
    let extended_terminal_execution = decode_imasm_execution(extended_terminal.word()).unwrap();
    assert_eq!(
        extended_terminal_execution,
        ImasmExecution::T {
            lattice: ImscriptionLattice::ExtendedFermat,
        }
    );

    let extended_closed = match extended.descend().unwrap() {
        Descent::Closed(closed) => closed,
        Descent::Continue(_) => panic!("near-root fixture failed to close after FOUR=T"),
    };
    assert_eq!(extended_closed.imscription, extended_terminal);
    assert_eq!(extended_closed.support, extended_support);
    assert_eq!(extended_closed.lattice_cell, extended_witness.lattice_cell);

    // Far-gap: extended B owns the topology change itself: Lehman k=1,
    // span=64 and the Lehman word are all already in the B variant.
    let far_n = mul(&tape_u64(1_000_003), &tape_u64(10_000_019));
    let short = DialecticObject::new(far_n).unwrap();
    let extended = match short.descend().unwrap() {
        Descent::Continue(next) => next,
        Descent::Closed(_) => panic!("far-gap fixture unexpectedly closed in short frontier"),
    };
    let change = extended.judge_current_imscription().unwrap();
    let lehman1_relation = match &change {
        DialecticJudgment::B { imscription, .. } => imscription.clone(),
        other => panic!("lattice change exposed unexpected FOUR={}", other.four()),
    };
    assert_eq!(lehman1_relation.boundary(), &tape_u64(1));
    assert_eq!(lehman1_relation.span(), &tape_u64(LEHMAN_LOCAL_SPAN));
    assert_eq!(
        decode_imasm_execution(lehman1_relation.word()).unwrap(),
        ImasmExecution::B {
            lattice: ImscriptionLattice::Lehman,
        }
    );

    let lehman1 = match extended.descend().unwrap() {
        Descent::Continue(next) => next,
        Descent::Closed(_) => panic!("far-gap fixture unexpectedly closed before Lehman"),
    };
    assert_eq!(lehman1.imscription, lehman1_relation);

    // Lehman B structurally cannot carry a witness; it owns only relation+support.
    let k1 = lehman1.judge_current_imscription().unwrap();
    let k2_relation = match &k1 {
        DialecticJudgment::B { imscription, .. } => imscription.clone(),
        other => panic!("Lehman k=1 exposed unexpected FOUR={}", other.four()),
    };
    assert_eq!(k2_relation.boundary(), &tape_u64(2));
    assert_eq!(k2_relation.span(), &tape_u64(LEHMAN_LOCAL_SPAN));

    let mut current = match lehman1.descend().unwrap() {
        Descent::Continue(next) => next,
        Descent::Closed(_) => panic!("far-gap fixture unexpectedly closed at k=1"),
    };
    assert_eq!(current.imscription, k2_relation);

    let far_closed = loop {
        let judgment = current.judge_current_imscription().unwrap();
        match &judgment {
            DialecticJudgment::B {
                imscription,
                support,
            } => {
                let expected_relation = imscription.clone();
                let expected_support = *support;
                current = match current.descend().unwrap() {
                    Descent::Continue(next) => next,
                    Descent::Closed(_) => panic!("FOUR=B unexpectedly closed the far-gap object"),
                };
                assert_eq!(current.imscription, expected_relation);
                assert_eq!(current.support, expected_support);
            }
            DialecticJudgment::T {
                imscription,
                support,
                witness,
            } => {
                let terminal_relation = imscription.clone();
                let terminal_support = *support;
                let witness = witness.clone();
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
            DialecticJudgment::N { .. } => panic!("factoring descent unexpectedly judged N"),
            DialecticJudgment::F => panic!("validated factoring descent unexpectedly judged F"),
        }
    };
    assert_eq!(
        far_closed.lehman_multiplier.as_deref(),
        Some(tape_u64(10).as_slice())
    );
    assert_eq!(far_closed.lattice_cell, tape_u64(0));

    // Architectural guard: FOUR is a sum type, not a mark plus nullable payloads.
    let source = include_str!("../src/dialectic_reentry.rs");
    let judgment_decl = source
        .split("pub enum DialecticJudgment")
        .nth(1)
        .expect("FOUR judgment enum not found")
        .split("/// The arithmetic lattice itself")
        .next()
        .unwrap();
    assert!(!source.contains("pub struct DialecticJudgment"));
    assert!(judgment_decl.contains("T {"));
    assert!(judgment_decl.contains("B {"));
    assert!(judgment_decl.contains("N {"));
    assert!(judgment_decl.contains("F,"));
    assert!(!judgment_decl.contains("Option<Imscription>"));
    assert!(!judgment_decl.contains("Option<DialecticWitness>"));

    let execution_decl = source
        .split("pub enum ImasmExecution")
        .nth(1)
        .expect("IMASM execution sum not found")
        .split("/// A factor relation exposed")
        .next()
        .unwrap();
    assert!(execution_decl.contains("B { lattice:"));
    assert!(execution_decl.contains("T { lattice:"));
    assert!(execution_decl.contains("N { lattice:"));
    assert!(!source.contains("pub struct ImasmExecution"));
    assert!(!execution_decl.contains("closed: bool"));
    assert!(!execution_decl.contains("four: Mark"));

    let descend = source
        .split("pub fn descend(self)")
        .nth(1)
        .expect("descend source not found")
        .split("fn close(")
        .next()
        .unwrap();
    assert!(descend.contains("DialecticJudgment::T"));
    assert!(descend.contains("DialecticJudgment::B"));
    assert!(descend.contains("DialecticJudgment::N"));
    assert!(descend.contains("DialecticJudgment::F"));
    assert!(!descend.contains("judgment.witness"));
    assert!(!descend.contains("judgment.resulting_imscription"));
    assert!(!descend.contains("judgment.resulting_support"));

    println!(
        "dialectic FOUR sums: judgment and IMASM execution states are structurally distinct; no mark+boolean or nullable-payload encoding remains"
    );
}
