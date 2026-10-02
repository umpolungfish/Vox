//! Single prepared membrane emitter: GLUT with nested quantum syndrome folds.
//! Bake N with VOX_PHASE_MODULUS_WORD or VOX_BAKED_INPUT_FILE.
//! Source, factors and certificate leave together as IMASM words.
#![deny(warnings)]
use vox::factor_extract::FactorCarrier;
use vox::fixed_point_quantum_phase::factor_oracle::{FactorPhaseOracle, GateSink};
use vox::fixed_point_quantum_phase::register::FoldedRegister;
use vox::glut_system::{verify_glut_reentry_certificate, PreparedGlutMembrane};
use vox::morphism_factor::{emit_numeral, parse_numeral, trim};
use vox::reentry_certificate::{
    certify_reentry, decode_reentry_certificate, encode_reentry_certificate,
};
use vox::vox::{EVALF, EVALT};
include!(concat!(env!("OUT_DIR"), "/baked_inputs.rs"));

fn factor_phase_closes(n: &[char], p: &[char], q: &[char]) -> Result<(), &'static str> {
    let oracle = FactorPhaseOracle::from_n(n)?;
    let layout = oracle.layout();
    let mut register = FoldedRegister::zero(layout.cells);
    let mut basis = vec![EVALT; layout.cells];
    for (range, factor) in [(layout.p.clone(), p), (layout.q.clone(), q)] {
        for (offset, &bit) in factor.iter().enumerate() {
            if bit == EVALF {
                let cell = range.start + offset;
                register.toggle(&[], cell)?;
                basis[cell] = EVALF;
            }
        }
    }
    oracle.lower(&mut register)?;
    if register.basis_phase(&basis)? != EVALF {
        return Err("factor-pair phase oracle did not mark the closed pair");
    }
    let recovered = register.measure(&[EVALT], &[EVALF])?;
    if recovered != basis
        || trim(recovered[layout.p.clone()].to_vec()) != p
        || trim(recovered[layout.q.clone()].to_vec()) != q
        || recovered[layout.product.clone()]
            .iter()
            .any(|&bit| bit != EVALT)
        || recovered[layout.row.clone()]
            .iter()
            .any(|&bit| bit != EVALT)
        || recovered[layout.carry] != EVALT
        || recovered[layout.nontrivial_p] != EVALT
        || recovered[layout.nontrivial_q] != EVALT
    {
        return Err("factor-pair phase oracle changed a live register or retained workspace");
    }
    Ok(())
}

fn main() {
    let word = BAKED_MODULUS_WORD.expect("bake the resident IMASM modulus");
    let n = parse_numeral(word).expect("baked source numeral");
    let membrane =
        PreparedGlutMembrane::prepare(&n).expect("resident GLUT nested inside the carrier");
    let execution = membrane
        .execute()
        .expect("collapsed carrier returns its GLUT payload");
    let execution = execution.expect("resident GLUT membrane must close a proper pair");
    factor_phase_closes(&n, &execution.p, &execution.q)
        .expect("shared quantum phase oracle marks the resident GLUT pair");
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
