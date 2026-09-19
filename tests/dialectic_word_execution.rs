use vox::dialectic_reentry::{
    decode_imasm_execution, Descent, DialecticObject, ImscriptionLattice,
    EXTENDED_FERMAT_WORD, LEHMAN_WORD, SHORT_FRONTIER_WORD,
};
use vox::morphism_factor::{mul, tape_u64};

fn marks(word: &str) -> Vec<char> {
    word.chars().collect()
}

#[test]
fn imasm_marks_decode_to_the_lattice_action_the_operator_executes() {
    let open_cases = [
        (SHORT_FRONTIER_WORD, ImscriptionLattice::ShortFrontier),
        (EXTENDED_FERMAT_WORD, ImscriptionLattice::ExtendedFermat),
        (LEHMAN_WORD, ImscriptionLattice::Lehman),
    ];

    for (word, lattice) in open_cases {
        let execution = decode_imasm_execution(&marks(word)).unwrap();
        assert_eq!(execution.lattice, lattice);
        assert_eq!(execution.four, 'B');
        assert!(!execution.closed);
    }

    // A structural mutation is not another route name: it is no longer a word
    // in the executable dialectic grammar.
    let mut malformed = marks(SHORT_FRONTIER_WORD);
    malformed[5] = '⋈';
    assert!(decode_imasm_execution(&malformed).is_err());

    let mut bad_frame = marks(LEHMAN_WORD);
    bad_frame[1] = '⊙';
    bad_frame[0] = '⊣';
    assert!(decode_imasm_execution(&bad_frame).is_err());
}

#[test]
fn real_descents_and_closures_are_dispatched_by_decoded_imasm_grammar() {
    // Near-root: short B re-imscribes extended B, which closes as extended T.
    let near_n = mul(&tape_u64(1_000_003), &tape_u64(1_032_007));
    let near = DialecticObject::new(near_n).unwrap();
    assert_eq!(
        decode_imasm_execution(near.word()).unwrap().lattice,
        ImscriptionLattice::ShortFrontier,
    );
    let extended = match near.descend().unwrap() {
        Descent::Continue(next) => next,
        Descent::Closed(_) => panic!("near-root fixture unexpectedly closed in short frontier"),
    };
    let extended_execution = decode_imasm_execution(extended.word()).unwrap();
    assert_eq!(extended_execution.lattice, ImscriptionLattice::ExtendedFermat);
    assert_eq!(extended_execution.four, 'B');
    let extended_closed = match extended.descend().unwrap() {
        Descent::Closed(closed) => closed,
        Descent::Continue(_) => panic!("near-root fixture failed to close in extended Fermat"),
    };
    let closed_execution = decode_imasm_execution(extended_closed.word()).unwrap();
    assert_eq!(closed_execution.lattice, ImscriptionLattice::ExtendedFermat);
    assert_eq!(closed_execution.four, 'T');
    assert!(closed_execution.closed);

    // Far-gap: after both Fermat words the grammar selects Lehman, and the
    // terminal word still decodes as that same lattice with FOUR=T.
    let far_n = mul(&tape_u64(1_000_003), &tape_u64(10_000_019));
    let mut current = DialecticObject::new(far_n).unwrap();
    let far_closed = loop {
        let execution = decode_imasm_execution(current.word()).unwrap();
        match current.descend().unwrap() {
            Descent::Continue(next) => {
                if execution.lattice == ImscriptionLattice::ExtendedFermat {
                    assert_eq!(
                        decode_imasm_execution(next.word()).unwrap().lattice,
                        ImscriptionLattice::Lehman,
                    );
                }
                current = next;
            }
            Descent::Closed(closed) => break closed,
        }
    };
    let far_execution = decode_imasm_execution(far_closed.word()).unwrap();
    assert_eq!(far_execution.lattice, ImscriptionLattice::Lehman);
    assert_eq!(far_execution.four, 'T');
    assert!(far_execution.closed);

    let source = include_str!("../src/dialectic_reentry.rs");
    assert!(!source.contains("word == short_word"));
    assert!(!source.contains("word == extended_word"));
    assert!(!source.contains("word == lehman_word"));
    assert!(source.contains("match execution.lattice"));
    assert!(source.contains("decode_imasm_execution(self.word())"));

    println!(
        "dialectic IMASM execution: grammar selects short -> extended -> Lehman; open words are FOUR=B and closing words decode as FOUR=T"
    );
}
