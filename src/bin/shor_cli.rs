//! Unbaked arbitrary-width Shor membrane: `shor_cli <a> <N> <qubits>`.
//! shor_qft is a bin-only module (it uses crate-relative paths for its two
//! local deps), so it is included by path here rather than from the lib.
#![deny(warnings)]
extern crate alloc;
use std::env;
#[path = "../membrane_complex.rs"] mod membrane_complex;
#[path = "../fde_shor_membrane.rs"] mod fde_shor_membrane;
#[path = "../shor_qft.rs"] mod shor_qft;

fn main() {
    #[cfg(feature = "hosted")]
    ::vox::ordering::cli::intercept();
    let args: Vec<String> = env::args().collect();
    let mut base = None;
    let mut number = None;
    let mut qubits = None;
    let mut index = 1;
    while index < args.len() {
        match args[index].as_str() {
            "--base" => { index += 1; base = args.get(index); }
            "--number" => { index += 1; number = args.get(index); }
            "--qubits" => { index += 1; qubits = args.get(index); }
            "--help" | "-h" => {
                println!("Usage: shor_cli --base DECIMAL_A --number DECIMAL_N --qubits COUNT");
                return;
            }
            flag => { eprintln!("unknown flag: {flag}"); std::process::exit(1); }
        }
        index += 1;
    }
    let (Some(base), Some(number), Some(qubits)) = (base, number, qubits) else {
        eprintln!("Usage: shor_cli --base DECIMAL_A --number DECIMAL_N --qubits COUNT");
        std::process::exit(1);
    };
    let a_tape = match vox::morphism_factor::decimal_to_tape(base) { Some(t) => t, None => { eprintln!("bad base"); std::process::exit(1); } };
    let n_tape = match vox::morphism_factor::decimal_to_tape(number) { Some(t) => t, None => { eprintln!("bad N"); std::process::exit(1); } };
    let qbits: usize = match qubits.parse() { Ok(n) => n, Err(_) => { eprintln!("bad qubits"); std::process::exit(1); } };
    match shor_qft::run_shor_big_report(a_tape, n_tape, qbits) {
        Ok(r) => println!("{}", r),
        Err(e) => { eprintln!("{}", e); std::process::exit(2); }
    }
}
