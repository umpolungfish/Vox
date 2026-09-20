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
    let b = short.judge_current_imscription();
    assert!(matches!(b, DialecticJudgment::B { .. }));
    assert_eq!(b.four(), 'B');

    // Descent preserves that B structurally rather than calling it generic continuation.
    let extended = match short.clone().descend() {
        Descent::B(next) => next,
        other => panic!("near-root short frontier descended as FOUR={} instead of B", other.four()),
    };

    // T is the real extended-Fermat closure reached by the same semiprime.
    let t = extended.judge_current_imscription();
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
    let closed = match extended.clone().descend() {
        Descent::T(closed) => closed,
        other => panic!("extended Fermat descended as FOUR={} instead of T", other.four()),
    };
    assert_eq!(closed.imscription, terminal);

    // N is not a fabricated enum constructor. route_around() installs an actual
    // IMASM grammar whose empty ∈∋ region makes FOUR=N on the same lattice.
    let neutral = short.route_around().unwrap();
    assert_eq!(verdict(neutral.word()), 'N');
    let neutral_exec = decode_imasm_execution(neutral.word()).unwrap();
    assert!(matches!(neutral_exec, ImasmExecution::N { .. }));
    assert_eq!(neutral_exec.four(), 'N');
    assert!(neutral_exec.is_neutral());
    let n_judgment = neutral.judge_current_imscription();
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

    // N is a complete persisted whole object and descent preserves N itself:
    // no new distinction means no boundary, span, support, bulk, or relation change.
    let restarted_n = DialecticObject::decode(&neutral.encode()).unwrap();
    assert_eq!(restarted_n, neutral);
    let unchanged = match neutral.clone().descend() {
        Descent::N(next) => next,
        other => panic!("FOUR=N route-around descended as FOUR={}", other.four()),
    };
    assert_eq!(unchanged, neutral);

    // A factor-closing certificate has no terminal carrier for an N fixed point.
    // It rejects the explicit N variant instead of inferring N from wire identity.
    let neutral_certificate_error = certify_dialectic(&neutral).unwrap_err();
    assert!(neutral_certificate_error.contains("FOUR=N unchanged imscription"));

    // F is reached through the same total judgment and descent machinery. A complete
    // r/w/x object whose executable grammar has a fuse with no split is malformed,
    // so both judgment and whole-object consumption expose FOUR=F directly.
    let mut malformed_grammar = DialecticObject::new(n.clone()).unwrap();
    malformed_grammar.imscription.rwx.execute_word =
        vec![VINIT, IMSCRIB, '∋', '≻', '⊤', '≺', '⊥', '⊡', TANCH];
    assert_eq!(verdict(malformed_grammar.word()), 'F');
    assert!(decode_imasm_execution(malformed_grammar.word()).is_err());
    assert!(matches!(
        malformed_grammar.judge_current_imscription(),
        DialecticJudgment::F
    ));
    assert!(matches!(malformed_grammar.clone().descend(), Descent::F));
    assert!(malformed_grammar.validate().is_err());
    assert!(DialecticObject::decode(&malformed_grammar.encode()).is_err());

    // Broken relation shape is likewise F, not a Rust execution error.
    let mut severed_read = DialecticObject::new(n.clone()).unwrap();
    severed_read.imscription.rwx.read_bulk = tape_u64(3);
    assert!(matches!(
        severed_read.judge_current_imscription(),
        DialecticJudgment::F
    ));
    assert!(matches!(severed_read.descend(), Descent::F));

    // A well-framed executable word bound to the wrong span/support is also F:
    // grammar decoded, but the live imscription relation is inconsistent.
    let mut bad_span = DialecticObject::new(n.clone()).unwrap();
    bad_span.imscription.rwx.execute_span = tape_u64(63);
    assert!(matches!(
        bad_span.judge_current_imscription(),
        DialecticJudgment::F
    ));
    assert!(matches!(bad_span.descend(), Descent::F));

    let mut bad_support = DialecticObject::new(n).unwrap();
    bad_support.support ^= 1 << 5;
    assert!(matches!(
        bad_support.judge_current_imscription(),
        DialecticJudgment::F
    ));
    assert!(matches!(bad_support.descend(), Descent::F));

    // Architectural guards: arithmetic walkers expose only T/B; valid executable
    // grammar is structurally B/T/N; judgment and descent both preserve FOUR.
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

    let judge = source
        .split("pub fn judge_current_imscription")
        .nth(1)
        .expect("FOUR-total judgment boundary not found")
        .split("pub fn envelope")
        .next()
        .unwrap();
    assert!(source.contains(
        "pub fn judge_current_imscription(&self) -> DialecticJudgment"
    ));
    assert!(!judge.contains("Result<DialecticJudgment"));
    assert!(!judge.contains("return Err("));
    assert!(!judge.contains("Ok(DialecticJudgment"));
    assert!(judge.contains("return DialecticJudgment::F"));

    let descent_decl = source
        .split("pub enum Descent")
        .nth(1)
        .expect("FOUR-preserving descent sum not found")
        .split("impl Descent")
        .next()
        .unwrap();
    assert!(descent_decl.contains("T(DialecticClosure)"));
    assert!(descent_decl.contains("B(DialecticObject)"));
    assert!(descent_decl.contains("N(DialecticObject)"));
    assert!(descent_decl.contains("F,"));
    assert!(!descent_decl.contains("Continue"));
    assert!(!descent_decl.contains("Closed"));
    assert!(source.contains("pub fn descend(self) -> Descent"));
    assert!(!source.contains("pub fn descend(self) -> Result<Descent"));

    let certificate_source = include_str!("../src/dialectic_certificate.rs");
    assert!(!certificate_source.contains("next.encode() == current_wire"));
    assert!(certificate_source.contains("Descent::N(_)"));
    assert!(certificate_source.contains("Descent::F"));

    println!(
        "dialectic FOUR total: B/T/N/F survive judgment and whole-object descent structurally; N is explicit identity and F is not a Rust execution error"
    );
}
