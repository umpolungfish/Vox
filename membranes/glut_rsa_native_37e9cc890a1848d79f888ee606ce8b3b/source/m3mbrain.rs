//! Single prepared membrane emitter: GLUT with nested quantum syndrome folds.
//! Bake N with VOX_PHASE_MODULUS_WORD or VOX_BAKED_INPUT_FILE.
//! Source, factors and certificate leave together as IMASM words.
#![deny(warnings)]
use vox::factor_extract::FactorCarrier;
use vox::fixed_point_quantum_phase::factor_oracle::{Control, FactorPhaseOracle, GateSink};
use vox::glut_system::{verify_glut_reentry_certificate, PreparedGlutMembrane};
use vox::morphism_factor::{emit_numeral, parse_numeral};
use vox::reentry_certificate::{
    certify_reentry, decode_reentry_certificate, encode_reentry_certificate,
};
use vox::vox::{EVALF, EVALT};
include!(concat!(env!("OUT_DIR"), "/baked_inputs.rs"));

fn factor_phase_closes(n: &[char], p: &[char], q: &[char]) -> Result<(), &'static str> {
    let oracle = FactorPhaseOracle::from_n(n)?;
    let layout = oracle.layout();
    let mut state = FactorPhaseBasis {
        cells: vec![EVALT; layout.cells],
        phase: EVALT,
    };
    for (range, factor) in [(layout.p.clone(), p), (layout.q.clone(), q)] {
        for (offset, &bit) in factor.iter().enumerate() {
            state.cells[range.start + offset] = bit;
        }
    }
    let input = state.cells.clone();
    oracle.lower(&mut state)?;
    if state.phase != EVALF {
        return Err("factor-pair phase oracle did not mark the closed pair");
    }
    if state.cells[layout.p.clone()] != input[layout.p.clone()]
        || state.cells[layout.q.clone()] != input[layout.q.clone()]
        || state.cells[layout.product.clone()]
            .iter()
            .any(|&bit| bit != EVALT)
        || state.cells[layout.row.clone()]
            .iter()
            .any(|&bit| bit != EVALT)
        || state.cells[layout.carry] != EVALT
        || state.cells[layout.nontrivial_p] != EVALT
        || state.cells[layout.nontrivial_q] != EVALT
    {
        return Err("factor-pair phase oracle changed a live register or retained workspace");
    }
    Ok(())
}

struct FactorPhaseBasis {
    cells: Vec<char>,
    phase: char,
}

impl GateSink for FactorPhaseBasis {
    fn toggle(&mut self, controls: &[Control], target: usize) -> Result<(), &'static str> {
        if target >= self.cells.len() || controls.iter().any(|control| control.cell == target) {
            return Err("factor-phase basis toggle is out of range");
        }
        if controls
            .iter()
            .all(|control| self.cells.get(control.cell) == Some(&control.value))
        {
            self.cells[target] = if self.cells[target] == EVALT {
                EVALF
            } else {
                EVALT
            };
        }
        Ok(())
    }

    fn phase_flip(&mut self, controls: &[Control]) -> Result<(), &'static str> {
        if controls
            .iter()
            .any(|control| self.cells.get(control.cell) != Some(&control.value))
        {
            return Ok(());
        }
        self.phase = if self.phase == EVALT { EVALF } else { EVALT };
        Ok(())
    }
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
