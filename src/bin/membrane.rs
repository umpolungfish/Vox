//! Baked membrane CLI with arbitrary-width IMASM operands.
//!
//! MEMBRANE_WORDS is "<a-word> <n-word> <qubits-word>" — three IMASM numeral
//! words, baked at compile time by `option_env!`. The run never touches them
//! again: `membrane` with no args dispatches `run_shor_big_report` against
//! the baked (a, n, qubits). Same shape as `shor-big-one` / `factor-one` /
//! `factorization-31-one` — the numeral word IS the program.
//!
//! Build fast (same shape as factor_one.sh):
//!   cargo build --release --bin vox
//!   NWORD="$(./target/release/vox numeral <N>)"
//!   AWORD="$(./target/release/vox numeral 2)"
//!   QWORD="$(./target/release/vox numeral 0)"   # 0 = auto (2*|N| bits)
//!   MEMBRANE_WORDS="$AWORD $NWORD $QWORD" cargo build --release --bin membrane
//!   ./target/release/membrane

extern crate alloc;
#[allow(dead_code)]
#[path = "../shor_qft.rs"] mod shor_qft;
#[allow(dead_code)]
#[path = "../membrane_complex.rs"] mod membrane_complex;
#[allow(dead_code)]
#[path = "../baked_membrane.rs"] mod baked_membrane;
#[path = "../fde_shor_membrane.rs"] mod fde_shor_membrane;

fn tape_to_usize(tape: &[char]) -> Result<usize, String> {
    tape.iter().enumerate().try_fold(0usize, |value, (bit, &mark)| {
        if mark == vox::vox::EVALF {
            value.checked_add(1usize.checked_shl(bit as u32)?)
        } else { Some(value) }
    }).ok_or("resident QFT depth cannot be represented by this host index type".into())
}

fn main() {
    let result = (|| -> Result<String, String> {
        let words = baked_membrane::numeral_words(option_env!("MEMBRANE_WORDS"))?;
        if words.len() != 3 { return Err("membrane needs a, N, and qubits".into()); }
        let qubits = tape_to_usize(&words[2])?;
        // Same shape as shor-big-one: a, N, qubits — phase-based period-finding
        // via the QFT register (the "fast one" Vox ships).
        shor_qft::run_shor_big_report(words[0].clone(), words[1].clone(), qubits)
    })();
    match result {
        Ok(report) => println!("{report}"),
        Err(error) => { eprintln!("{error}"); std::process::exit(2); }
    }
}
