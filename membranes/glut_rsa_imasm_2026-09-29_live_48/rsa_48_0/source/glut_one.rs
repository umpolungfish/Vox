//! Baked glut membrane. Source, factors and certificate leave as IMASM words.
#![deny(warnings)]
use vox::factor_extract::FactorCarrier;
use vox::glut_system::{glut_factor_execution, verify_glut_reentry_certificate, GlutSieve};
use vox::morphism_factor::{emit_numeral, parse_numeral};
use vox::reentry_certificate::{
    certify_reentry, decode_reentry_certificate, encode_reentry_certificate,
};
// Observer positions are transported as dynamic numeral tapes too.
fn position_word(mut value: usize) -> String {
    let mut tape = Vec::new();
    while value != 0 {
        tape.push(if value & 1 == 1 { '⊥' } else { '⊤' });
        value >>= 1;
    }
    if tape.is_empty() {
        tape.push('⊤');
    }
    emit_numeral(&tape)
}

fn main() {
    let word = option_env!("GLUT_SOURCE_WORD")
        .unwrap_or(include_str!("../../membranes/glut_dynamic_8051/input.imasm").trim());
    let n = parse_numeral(word).expect("baked source numeral");
    let execution = if option_env!("GLUT_OBSERVE_WORD") == Some("⊥") {
        use std::io::{self, Write};
        println!("{}", emit_numeral(&n));
        let mut sieve = GlutSieve::new(&n);
        sieve.frame_sweep_observed(|sieve, width| {
            if sieve.fold_stats.active && width == 0 {
                let f = &sieve.fold_stats;
                // Eleven canonical numeral words per progress record.
                for value in [
                    f.decisions,
                    f.p_width,
                    f.q_width,
                    f.frontier,
                    f.cells,
                    f.correlation_decisions,
                    f.correlation_conflicts,
                    f.correlation_gates,
                    f.correlation_cells,
                    f.square_advances,
                    f.square_cells,
                ] {
                    println!("{}", position_word(value));
                }
                io::stdout().flush().expect("IMASM progress transport");
            }
        });
        sieve.readout_execution()
    } else {
        glut_factor_execution(&n)
    };
    let Some(execution) = execution else {
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
