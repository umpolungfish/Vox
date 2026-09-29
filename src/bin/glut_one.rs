//! Baked glut membrane. Source, factors and certificate leave as IMASM words.
#![deny(warnings)]
use vox::factor_extract::FactorCarrier;
use vox::glut_system::{glut_factor_execution, verify_glut_reentry_certificate};
use vox::morphism_factor::{emit_numeral, parse_numeral};
use vox::reentry_certificate::{
    certify_reentry, decode_reentry_certificate, encode_reentry_certificate,
};
fn main() {
    let word = option_env!("GLUT_SOURCE_WORD")
        .unwrap_or(include_str!("../../membranes/glut_dynamic_8051/input.imasm").trim());
    let n = parse_numeral(word).expect("baked source numeral");
    let Some(execution) = glut_factor_execution(&n) else {
        println!("⊢⊙⊣");
        return;
    };
    let trace = execution.trace(&n).expect("executed convolution rail");
    let carrier = FactorCarrier::new(n.clone(), execution.p.clone(), execution.q.clone(), trace)
        .expect("proper exact source closure");
    let certificate = certify_reentry(&carrier).expect("passive re-entry");
    let wire = encode_reentry_certificate(&certificate);
    let decoded = decode_reentry_certificate(&wire).expect("certificate transport");
    verify_glut_reentry_certificate(&decoded).expect("source and re-entry replay");
    println!("{}", emit_numeral(&n));
    println!("{}", emit_numeral(&execution.p));
    println!("{}", emit_numeral(&execution.q));
    println!("{}", wire.iter().collect::<String>());
}
