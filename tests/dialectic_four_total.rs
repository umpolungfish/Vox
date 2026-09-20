use vox::dialectic_reentry::{
    decode_imasm_execution, Descent, DialecticJudgment, DialecticObject,
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

    // N is not a fabricated enum constructor. route_around() installs an actual
    // IMASM grammar whose empty ∈∋ region makes FOUR=N on the same lattice.
    let neutral = short.route_around().unwrap();
    assert_eq!(verdict(neutral.word()), 'N');
    let neutral_exec = decode_imasm_execution(neutral.word()).unwrap();
    assert_eq!(neutral_exec.four, 'N');
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

    // F is likewise reached through the same judgment machinery. The relation
    // remains a complete r/w/x object, but its executable grammar has a fuse
    // with no split. SIXTEEN_3 therefore judges F before any lattice is walked.
    let mut malformed = DialecticObject::new(n).unwrap();
    malformed.imscription.rwx.execute_word =
        vec![VINIT, IMSCRIB, '∋', '≻', '⊤', '≺', '⊥', '⊡', TANCH];
    assert_eq!(verdict(malformed.word()), 'F');
    assert!(matches!(
        malformed.judge_current_imscription().unwrap(),
        DialecticJudgment::F
    ));
    assert!(malformed.validate().is_err());
    assert!(DialecticObject::decode(&malformed.encode()).is_err());
    assert!(malformed.descend().is_err());

    // Architectural guard: the arithmetic walkers expose only what they really
    // produce; N/F must be handled by the imscription grammar judgment itself.
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
    assert!(source.contains("pub fn route_around"));
    assert!(source.contains("execution.four == 'N'"));
    assert!(source.contains("return Ok(DialecticJudgment::F)"));

    println!(
        "dialectic FOUR total: real semiprime B/T, persisted identity N, and malformed-grammar F all pass through one judgment path"
    );
}
