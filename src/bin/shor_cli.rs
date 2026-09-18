//! Unbaked arbitrary-width Shor membrane: `shor_cli <a> <N> <qubits>`.
//! shor_qft is a bin-only module (it uses crate-relative paths for its two
//! local deps), so it is included by path here rather than from the lib.
extern crate alloc;
use std::env;
#[path = "../membrane_complex.rs"] mod membrane_complex;
#[path = "../fde_shor_membrane.rs"] mod fde_shor_membrane;
#[path = "../shor_qft.rs"] mod shor_qft;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 4 { eprintln!("usage: shor_cli <a> <N> <qubits>"); std::process::exit(1); }
    let a_tape = match vox::morphism_factor::decimal_to_tape(&args[1]) { Some(t) => t, None => { eprintln!("bad a"); std::process::exit(1); } };
    let n_tape = match vox::morphism_factor::decimal_to_tape(&args[2]) { Some(t) => t, None => { eprintln!("bad N"); std::process::exit(1); } };
    let qbits: usize = match args[3].parse() { Ok(n) => n, Err(_) => { eprintln!("bad qubits"); std::process::exit(1); } };
    match shor_qft::run_shor_big_report(a_tape, n_tape, qbits) {
        Ok(r) => println!("{}", r),
        Err(e) => { eprintln!("{}", e); std::process::exit(2); }
    }
}
