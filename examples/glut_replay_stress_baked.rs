//! Increasing-width reverse-rail transport control with a supplied baked witness.
#![deny(warnings)]
use std::time::Instant;
use vox::glut_system::{GlutState, GlutExecution, verify_glut_trace, verify_glut_reentry_certificate};
use vox::morphism_factor::{parse_numeral, mul};
use vox::factor_extract::FactorCarrier;
use vox::reentry_certificate::{certify_reentry, encode_reentry_certificate, decode_reentry_certificate};
fn main() {
    let default = include_str!("../membranes/glut_stress_2026-09-29/control_35/input.imasm").trim();
    let p = parse_numeral(option_env!("GLUT_REPLAY_P").unwrap_or(default)).unwrap();
    let q = parse_numeral(option_env!("GLUT_REPLAY_Q").unwrap_or(default)).unwrap();
    let n = mul(&p, &q);
    println!("SUPPLIED_WITNESS\t{}\t{}\t{}", n.len(), p.len(), q.len());
    let start = Instant::now();
    let bit = |t: &[char], i: usize| u8::from(t.get(i) == Some(&'⊥'));
    let mut state = GlutState::seed(bit(&n, 0)).into_iter().find(|s|
        s.p_prefix[0] == p[0] && s.q_prefix[0] == q[0]).unwrap();
    let mut checkpoints = vec![state.clone()];
    let end = p.len().max(q.len());
    let mut next_checkpoint = 2;
    while state.position < end {
        let k = state.position;
        state = state.advance(bit(&n, k)).into_iter().find(|s|
            bit(&s.p_prefix, k) == bit(&p, k) && bit(&s.q_prefix, k) == bit(&q, k)).unwrap();
        if state.position == next_checkpoint || state.position == end {
            checkpoints.push(state.clone());
            next_checkpoint = next_checkpoint.checked_mul(2).unwrap_or(end).min(end);
        }
    }
    let construction_us = start.elapsed().as_micros();
    let execution = GlutExecution { p: p.clone(), q: q.clone(), checkpoints };
    let start = Instant::now();
    let trace = execution.trace(&n).unwrap();
    verify_glut_trace(&n, &p, &q, &trace).unwrap();
    let carrier = FactorCarrier::new(n, p, q, trace.clone()).unwrap();
    let cert = certify_reentry(&carrier).unwrap();
    let wire = encode_reentry_certificate(&cert);
    let decoded = decode_reentry_certificate(&wire).unwrap();
    verify_glut_reentry_certificate(&decoded).unwrap();
    println!("REPLAY\t{}\t{}\t{}\t{}\t{}\t{}", construction_us,
        start.elapsed().as_micros(), execution.checkpoints.len(), trace.len(),
        vox::trace_word::decode_trace(&trace).unwrap().len(), wire.len());
}
