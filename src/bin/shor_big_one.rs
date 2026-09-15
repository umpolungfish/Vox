//! Baked Shor membrane with arbitrary-width IMASM operands.

extern crate alloc;
#[allow(dead_code)]
#[path = "../shor_qft.rs"] mod shor_qft;
#[allow(dead_code)]
#[path = "../membrane_complex.rs"] mod membrane_complex;
#[allow(dead_code)]
#[path = "../baked_membrane.rs"] mod baked_membrane;

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
        if words.len() != 3 { return Err("Shor needs a, N, and qubits".into()); }
        let qubits = tape_to_usize(&words[2])?;
        shor_qft::run_shor_big_report(words[0].clone(), words[1].clone(), qubits)
    })();
    match result {
        Ok(report) => println!("{report}"),
        Err(error) => { eprintln!("{error}"); std::process::exit(2); }
    }
}
