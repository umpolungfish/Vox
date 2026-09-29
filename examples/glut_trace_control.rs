//! Isolate the carried trace at the native/lifted boundary.
#![deny(warnings)]
use vox::glut_system::{glut_factor_execution, verify_glut_trace};
use vox::morphism_factor::tape_u64;
fn main() {
    let n = tape_u64(35);
    let execution = glut_factor_execution(&n).unwrap();
    let trace = execution.trace(&n).unwrap();
    println!("trace length {}", trace.len());
    for (i, c) in trace.iter().enumerate() { println!("mark {i} {}", u32::from(*c)); }
    println!("decoded {:?}", vox::trace_word::decode_trace(&trace).map(|s| s.len()));
    println!("verified {:?}", verify_glut_trace(&n, &execution.p, &execution.q, &trace));
}
