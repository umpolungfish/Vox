use vox::dialectic_reentry::{
    decode_imasm_execution, Descent, DialecticObject, ImasmExecution, ImscriptionLattice,
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
        assert_eq!(execution, ImasmExecution::B { lattice });
        assert_eq!(execution.four(), 'B');
        assert!(!execution.is_terminal());
        assert!(!execution.is_neutral());
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
        decode_imasm_execution(near.word()).unwrap(),
        ImasmExecution::B {
            lattice: ImscriptionLattice::ShortFrontier,
        },
    );
    let extended = match near.descend() {
        Descent::B(next) => next,
        other => panic!("near-root short frontier descended as FOUR={}", other.four()),
    };
    let extended_execution = decode_imasm_execution(extended.word()).unwrap();
    assert_eq!(
        extended_execution,
        ImasmExecution::B {
            lattice: ImscriptionLattice::ExtendedFermat,
        }
    );
    let extended_closed = match extended.descend() {
        Descent::T(closed) => closed,
        other => panic!("near-root extended Fermat descended as FOUR={}", other.four()),
    };
    let closed_execution = decode_imasm_execution(extended_closed.word()).unwrap();
    assert_eq!(
        closed_execution,
        ImasmExecution::T {
            lattice: ImscriptionLattice::ExtendedFermat,
        }
    );
    assert!(closed_execution.is_terminal());

    // Far-gap: after both Fermat words the grammar selects Lehman, and the
    // terminal word still decodes as that same lattice with FOUR=T.
    let far_n = mul(&tape_u64(1_000_003), &tape_u64(10_000_019));
    let mut current = DialecticObject::new(far_n).unwrap();
    let far_closed = loop {
        let execution = decode_imasm_execution(current.word()).unwrap();
        match current.descend() {
            Descent::B(next) => {
                if execution.lattice() == ImscriptionLattice::ExtendedFermat {
                    assert_eq!(
                        decode_imasm_execution(next.word()).unwrap(),
                        ImasmExecution::B {
                            lattice: ImscriptionLattice::Lehman,
                        },
                    );
                }
                current = next;
            }
            Descent::T(closed) => break closed,
            Descent::N(_) => panic!("factoring execution unexpectedly routed around as FOUR=N"),
            Descent::F => panic!("validated factoring execution unexpectedly exposed FOUR=F"),
        }
    };
    let far_execution = decode_imasm_execution(far_closed.word()).unwrap();
    assert_eq!(
        far_execution,
        ImasmExecution::T {
            lattice: ImscriptionLattice::Lehman,
        }
    );
    assert!(far_execution.is_terminal());

    let source = include_str!("../src/dialectic_reentry.rs");
    assert!(!source.contains("word == short_word"));
    assert!(!source.contains("word == extended_word"));
    assert!(!source.contains("word == lehman_word"));
    assert!(!source.contains("pub struct ImasmExecution"));
    let execution_decl = source
        .split("pub enum ImasmExecution")
        .nth(1)
        .expect("structural IMASM execution enum not found")
        .split("/// A factor relation exposed")
        .next()
        .unwrap();
    assert!(execution_decl.contains("B { lattice:"));
    assert!(execution_decl.contains("T { lattice:"));
    assert!(execution_decl.contains("N { lattice:"));
    assert!(!execution_decl.contains("pub four:"));
    assert!(!execution_decl.contains("pub closed:"));
    assert!(source.contains("decode_imasm_execution(self.word())"));
    assert!(source.contains("pub fn descend(self) -> Descent"));
    assert!(!source.contains("Descent::Continue"));
    assert!(!source.contains("Descent::Closed"));

    println!(
        "dialectic IMASM execution sum: B/T/N grammar states are structurally typed; whole-object execution preserves FOUR while short -> extended -> Lehman remains grammar-selected"
    );
}
