use vox::dialectic_certificate::certify_dialectic;
use vox::dialectic_reentry::{
    decode_imasm_execution, Descent, DialecticJudgment, DialecticObject, ImasmExecution,
};
use vox::morphism_factor::{mul, tape_u64};
use vox::vox::{verdict, IMSCRIB, TANCH, VINIT};

#[test]
fn all_four_states_are_operational_through_one_judgment_path() {
    // B is the real productive short-frontier exposure of the near-root fixture.
    let n = mul(&tape_u64(1_000_003), &tape_u64(1_032_007));
    let short = DialecticObject::new(n.clone()).unwrap();
    let b = short.judge_current_imscription().unwrap();
    assert!(matches!(b, DialecticJudgment::B { .. }));
    assert_eq!(b.four(), 'B');

    // T is the real extended-Fermat closure reached by the same semiprime.
    let extended = match short.clone().descend().unwrap() {
        Descent::Continue(next) => next,
        Descent::Closed(_) => panic!("near-root fixture unexpectedly closed in short frontier"),
    };
    let t = extended.judge_current_imscription().unwrap();
    let terminal = match &t {
        DialecticJudgment::T {
            imscription,
            witness,
            ..
        } => {
            assert_eq!(witness.lattice_cell, tape_u64(126));
            imscription.clone()
        }
        other => panic!("extended Fermat exposed FOUR={} instead of T", other.four()),
    };
    assert_eq!(t.four(), 'T');
    assert_eq!(verdict(terminal.word()), 'T');
    assert!(matches!(
        decode_imasm_execution(terminal.word()).unwrap(),
        ImasmExecution::T { .. }
    ));

    // N is not a fabricated enum constructor. route_around() installs an actual
    // IMASM grammar whose empty ∈∋ region makes FOUR=N on the same lattice.
    let neutral = short.route_around().unwrap();
    assert_eq!(verdict(neutral.word()), 'N');
    let neutral_exec = decode_imasm_execution(neutral.word()).unwrap();
    assert!(matches!(neutral_exec, ImasmExecution::N { .. }));
    assert_eq!(neutral_exec.four(), 'N');
    assert!(neutral_exec.is_neutral());
    let n_judgment = neutral.judge_current_imscription().unwrap();
    match &n_judgment {
        DialecticJudgment::N {
            imscription,
            support,
        } => {
            assert_eq!(imscription, &neutral.imscription);
            assert_eq!(*support, neutral.support);
        }
        other => panic!("route-around exposed FOUR={} instead of N", other.four()),
    }

    // N is a complete persisted whole object and descent is identity: no new
    // distinction means no boundary, span, support, bulk, or relation change.
    let restarted_n = DialecticObject::decode(&neutral.encode()).unwrap();
    assert_eq!(restarted_n, neutral);
    let unchanged = match neutral.clone().descend().unwrap() {
        Descent::Continue(next) => next,
        Descent::Closed(_) => panic!("FOUR=N route-around unexpectedly closed"),
    };
    assert_eq!(unchanged, neutral);

    // A factor-closing certificate has no terminal carrier for an N fixed point.
    // It must return explicitly instead of looping on the unchanged continuation.
    let neutral_certificate_error = certify_dialectic(&neutral).unwrap_err();
    assert!(neutral_certificate_error.contains("FOUR=N unchanged imscription"));

    // F is likewise reached through the same judgment machinery. The relation
    // remains a complete r/w/x object, but its executable grammar has a fuse
    // with no split. SIXTEEN_3 therefore judges F before any lattice is walked.
    let mut malformed = DialecticObject::new(n).unwrap();
    malformed.imscription.rwx.execute_word =
        vec![VINIT, IMSCRIB, '∋', '≻', '⊤', '≺', '⊥', '⊡', TANCH];
    assert_eq!(verdict(malformed.word()), 'F');
    assert!(decode_imasm_execution(malformed.word()).is_err());
    assert!(matches!(
        malformed.judge_current_imscription().unwrap(),
        DialecticJudgment::F
    ));
    assert!(malformed.validate().is_err());
    assert!(DialecticObject::decode(&malformed.encode()).is_err());
    assert!(malformed.descend().is_err());

    // Architectural guard: arithmetic walkers expose only T/B, valid executable
    // grammar is structurally B/T/N, and F stays a judgment of malformed grammar.
    let source = include_str!("../src/dialectic_reentry.rs");
    let exposure = source
        .split("enum LatticeExposure")
        .nth(1)
        .expect("LatticeExposure not found")
        .split("/// Decode one complete dialectic IMASM grammar")
        .next()
        .unwrap();
    assert!(exposure.contains("T {"));
    assert!(exposure.contains("B {"));
    assert!(!exposure.contains("N,"));
    assert!(!exposure.contains("F,"));
    assert!(!source.contains("#[allow(dead_code)]\nenum LatticeExposure"));

    let execution_decl = source
        .split("pub enum ImasmExecution")
        .nth(1)
        .expect("ImasmExecution sum not found")
        .split("/// A factor relation exposed")
        .next()
        .unwrap();
    assert!(execution_decl.contains("B { lattice:"));
    assert!(execution_decl.contains("T { lattice:"));
    assert!(execution_decl.contains("N { lattice:"));
    assert!(!execution_decl.contains("F"));
    assert!(!source.contains("pub struct ImasmExecution"));
    assert!(!source.contains("pub four: Mark"));
    assert!(!source.contains("pub closed: bool"));
    assert!(source.contains("pub fn route_around"));
    assert!(source.contains("execution.is_neutral()"));
    assert!(source.contains("return Ok(DialecticJudgment::F)"));

    println!(
        "dialectic FOUR total: B/T/N executable grammar is structurally typed; malformed grammar reaches F only through the judgment path"
    );
}
