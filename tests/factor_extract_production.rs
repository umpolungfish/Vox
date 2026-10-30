use std::process::Command;

use vox::factor_extract::FactorCarrier;
use vox::morphism_factor::tape_u64;
use vox::router_marks::{run_mark, RouterG, M_B, M_FIX, M_T};
use vox::router_object::RouterObject;
use vox::trace_word::decode_trace;

const N: u64 = 106_545_994_355_809;
const EXPECTED: (u64, u64) = (9_245_087, 11_524_607);

fn ordered((a, b): (u64, u64)) -> (u64, u64) {
    if a <= b { (a, b) } else { (b, a) }
}

/// Production acceptance boundary:
///
///     RouterObject::initial -> RouterG::run_mark
///       -> (found factor, successful GStep trajectory)
///       -> explicit conversion of producer values to IMASM numeral tapes
///       -> FactorCarrier::from_trace
///       -> marks-only FactorCarrier::encode
///       ---------------- HARD SERIALIZED BOUNDARY ----------------
///       -> `vox extract-factor <carrier-word>`
///       -> iterated self-entry under ≡c
///       -> same already-carried factor, one-record relaxed fixed point
///
/// `run_mark` is deliberately upstream and remains a machine-word producer.
/// The extractor API itself is arbitrary-width and accepts only numeral tapes.
#[test]
fn production_route_to_serialized_passive_extractor() {
    let router = RouterG::from_enum(&RouterObject::initial());
    let (found, trajectory) = run_mark(&router, N, 8);
    let upstream = found.expect("production VOX route did not carry a factor");

    assert_eq!(ordered(upstream), EXPECTED);
    assert_eq!(trajectory.len(), 3, "expected measured B -> B -> T route");
    assert_eq!(
        trajectory.iter().map(|s| s.judgment).collect::<Vec<_>>(),
        vec![M_B, M_B, M_T],
    );
    let terminal = trajectory.last().expect("successful route has no terminal step");
    assert_eq!(terminal.judgment, M_T);
    assert_eq!(terminal.next, M_FIX);

    // Conversion happens at the producer boundary.  The factor carrier itself
    // knows only arbitrary-width IMASM numeral tapes.
    let n_tape = tape_u64(N);
    let p_tape = tape_u64(upstream.0);
    let q_tape = tape_u64(upstream.1);
    let carrier = FactorCarrier::from_trace(&n_tape, &p_tape, &q_tape, &trajectory)
        .expect("successful production route did not form a valid factor carrier");
    let carrier_word: String = carrier.encode().into_iter().collect();

    // HARD BOUNDARY: after this point the consumer receives serialized marks only.
    let output = Command::new(env!("CARGO_BIN_EXE_vox"))
        .arg("extract-factor")
        .arg(&carrier_word)
        .output()
        .expect("failed to launch vox extract-factor");

    assert!(
        output.status.success(),
        "extract-factor failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );

    let stdout = String::from_utf8(output.stdout).expect("extract-factor stdout is not UTF-8");
    let mut lines = stdout.lines();

    let expected_line = format!("{} x {}", EXPECTED.0, EXPECTED.1);
    assert_eq!(lines.next(), Some(expected_line.as_str()));

    let normal_line = lines.next().expect("extract-factor omitted normal form");
    let normal_word = normal_line
        .strip_prefix("normal-form ")
        .expect("unexpected normal-form output prefix");
    let normal_marks: Vec<char> = normal_word.chars().collect();
    let normal_steps = decode_trace(&normal_marks).expect("CLI emitted malformed trace normal form");
    assert_eq!(normal_steps.len(), 1, "relaxed extractor did not remove routing scaffold");
    assert_eq!(normal_steps[0].judgment, M_T);
    assert_eq!(normal_steps[0].next, M_FIX);

    assert_eq!(lines.next(), Some("transforms 2"));
    assert_eq!(lines.next(), None, "unexpected extra extract-factor output");
}
