//! A single baked numeral, measured through the production glut and reverse rail.
#![deny(warnings)]
use std::io::{self, Write};
use std::time::Instant;
use vox::glut_system::{GlutSieve, verify_glut_trace, verify_glut_reentry_certificate};
use vox::morphism_factor::{parse_numeral, dec_of};
use vox::factor_extract::FactorCarrier;
use vox::reentry_certificate::{certify_reentry, encode_reentry_certificate, decode_reentry_certificate};
fn main() {
    let word = option_env!("GLUT_STRESS_WORD")
        .unwrap_or(include_str!("../membranes/glut_dynamic_8051/input.imasm").trim());
    let n = parse_numeral(word).expect("baked numeral");
    println!("INPUT\t{}\t{}\t{}\t{}", dec_of(&n), n.len(),
        n.iter().filter(|&&c| c == '⊥').count(), word.chars().count());
    let started = Instant::now();
    let mut sieve = GlutSieve::new(&n);
    let mut frames = 0;
    let mut peak = 0;
    sieve.frame_sweep_observed(|sieve, width| {
        if sieve.fold_stats.active && width == 0 {
            let f = &sieve.fold_stats;
            println!("FOLD\t{}\t{}\t{}\t{}\t{}\t{}\t{}", f.decisions,
                f.frontier, f.peak_frontier, f.cells, f.peak_cells, f.zero_run,
                started.elapsed().as_micros());
            println!("RAIL\t{}\t{}\t{}\t{}", f.p_width, f.q_width, f.rejected, f.splits);
            println!("CORRELATION\t{}\t{}\t{}\t{}", f.correlation_decisions, f.correlation_conflicts, f.correlation_gates, f.correlation_cells);
            io::stdout().flush().unwrap();
            return;
        }
        peak = peak.max(sieve.states.len());
        if width > 0 { frames += 1; }
        let position = sieve.states.first().map_or(0, |s| s.position);
        let carry_bits = sieve.states.iter().map(|s| s.carry.len()).max().unwrap_or(0);
        println!("FRAME\t{}\t{}\t{}\t{}\t{}", position, width,
            sieve.states.len(), carry_bits, started.elapsed().as_micros());
        io::stdout().flush().unwrap();
    });
    let search_us = started.elapsed().as_micros();
    if let Some(execution) = sieve.readout_execution() {
        let verify_started = Instant::now();
        execution.verify(&n).unwrap();
        let trace = execution.trace(&n).unwrap();
        verify_glut_trace(&n, &execution.p, &execution.q, &trace).unwrap();
        let carrier = FactorCarrier::new(n.clone(), execution.p.clone(), execution.q.clone(), trace.clone()).unwrap();
        let certificate = certify_reentry(&carrier).unwrap();
        let wire = encode_reentry_certificate(&certificate);
        let decoded = decode_reentry_certificate(&wire).unwrap();
        verify_glut_reentry_certificate(&decoded).unwrap();
        let mut altered = execution.clone();
        altered.checkpoints.last_mut().unwrap().carry.push('⊥');
        assert!(altered.verify(&n).is_err(), "carry mutation control");
        println!("RESULT\tclosed\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            dec_of(&execution.p), dec_of(&execution.q), search_us,
            verify_started.elapsed().as_micros(), frames, peak, trace.len(), wire.len());
    } else {
        println!("RESULT\texhausted_exact_set\t-\t-\t{}\t0\t{}\t{}\t0\t0", search_us, frames, peak);
    }
}
