//! Single prepared membrane emitter: GLUT with nested quantum syndrome folds.
//! Bake N with VOX_PHASE_MODULUS_WORD or VOX_BAKED_INPUT_FILE.
//! Source, factors and certificate leave together as IMASM words.
#![deny(warnings)]
use vox::factor_extract::FactorCarrier;
use vox::glut_system::{verify_glut_reentry_certificate, PreparedGlutMembrane};
use vox::morphism_factor::{emit_numeral, parse_numeral};
use vox::reentry_certificate::{
    certify_reentry, decode_reentry_certificate, encode_reentry_certificate,
};
include!(concat!(env!("OUT_DIR"), "/baked_inputs.rs"));

fn main() {
    let word = BAKED_MODULUS_WORD.expect("bake the resident IMASM modulus");
    let n = parse_numeral(word).expect("baked source numeral");
    let membrane =
        PreparedGlutMembrane::prepare(&n).expect("resident GLUT nested inside the carrier");
    let execution = membrane
        .execute()
        .expect("collapsed carrier returns its GLUT payload");
    let execution = execution.expect("resident GLUT membrane must close a proper pair");
    let trace = execution.trace(&n).expect("executed convolution rail");
    let carrier = FactorCarrier::new(n.clone(), execution.p.clone(), execution.q.clone(), trace)
        .expect("proper exact source closure");
    let certificate = certify_reentry(&carrier).expect("passive re-entry");
    let wire = encode_reentry_certificate(&certificate);
    let decoded = decode_reentry_certificate(&wire).expect("certificate transport");
    verify_glut_reentry_certificate(&decoded).expect("source and re-entry replay");
    // Publish only after the whole source, pair and certificate have closed.
    use std::io::{self, Write};
    let result = format!(
        "{}\n{}\n{}\n{}\n",
        emit_numeral(&n),
        emit_numeral(&execution.p),
        emit_numeral(&execution.q),
        wire.iter().collect::<String>()
    );
    io::stdout()
        .lock()
        .write_all(result.as_bytes())
        .expect("closed IMASM result transport");
}
