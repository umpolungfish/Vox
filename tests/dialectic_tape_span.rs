use vox::dialectic_reentry::{Descent, DialecticObject};
use vox::morphism_factor::{mul, tape_u64};

#[test]
fn imscribed_span_is_consumed_as_tape_without_host_cell_count() {
    // Architectural guard: semantic lattice extent must not be reconstructed as
    // a host loop count. Host usize remains legitimate for wire-buffer framing.
    let source = include_str!("../src/dialectic_reentry.rs");
    assert!(!source.contains("span_to_usize"));
    assert!(!source.contains("cells: usize"));
    assert!(!source.contains("for _ in 0..cells"));
    assert!(
        source.matches("while !zero(&remaining)").count() >= 2,
        "both Fermat and Lehman walkers must consume a tape countdown",
    );

    // The measured near-root fixture still consumes 64 then 4032 directly from
    // the imscribed span tapes and closes at the same absolute tape cell 126.
    let near_n = mul(&tape_u64(1_000_003), &tape_u64(1_032_007));
    let near = DialecticObject::new(near_n).unwrap();
    let near = match near.descend() {
        Descent::B(next) => next,
        other => panic!("near-root short frontier descended as FOUR={}", other.four()),
    };
    let near_closed = match near.descend() {
        Descent::T(closed) => closed,
        other => panic!("near-root extended Fermat descended as FOUR={}", other.four()),
    };
    assert_eq!(near_closed.lattice_cell, tape_u64(126));

    // The far-gap fixture still changes lattice and re-imscribes complete Lehman
    // objects until k=10; each local 64-cell span is consumed as a tape numeral.
    let far_n = mul(&tape_u64(1_000_003), &tape_u64(10_000_019));
    let mut far = DialecticObject::new(far_n).unwrap();
    let far_closed = loop {
        match far.descend() {
            Descent::B(next) => far = next,
            Descent::T(closed) => break closed,
            Descent::N(_) => panic!("factoring tape-span execution unexpectedly exposed N"),
            Descent::F => panic!("validated tape-span execution unexpectedly exposed F"),
        }
    };
    assert_eq!(far_closed.lehman_multiplier.as_deref(), Some(tape_u64(10).as_slice()));
    assert_eq!(far_closed.lattice_cell, tape_u64(0));

    println!(
        "dialectic tape-span executor: no semantic span->usize conversion; near-root cell=126; far-gap k=10 cell=0; descent preserves FOUR"
    );
}
