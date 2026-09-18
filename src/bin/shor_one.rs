//! Baked Shor statevector membrane: membrane_one.sh shor a N qubits.
extern crate alloc;
#[allow(dead_code)]
#[path = "../shor_qft.rs"] mod shor_qft;
#[allow(dead_code)]
#[path = "../membrane_complex.rs"] mod membrane_complex;
#[allow(dead_code)]
#[path = "../baked_membrane.rs"] mod baked_membrane;
#[path = "../fde_shor_membrane.rs"] mod fde_shor_membrane;
fn run() -> Result<(), String> {
    // The payload rides as IMASM numerals, not u64 words: the wide path is
    // tape-native end to end, so N above 2^64 - 1 enters without a register
    // wall. Only the small statevector branch narrows to u64.
    let words = baked_membrane::numeral_words(option_env!("MEMBRANE_WORDS"))?;
    if words.len() != 3 { return Err("Shor needs a, N, qubits".into()); }
    let bits = vox::morphism_factor::dec_of(&words[2]).parse::<u64>()
        .map_err(|_| "Qubit count exceeds usize".to_string())
        .and_then(|b| usize::try_from(b).map_err(|_| "Qubit count exceeds usize".to_string()))?;
    if bits <= 14 {
        let a_u = vox::morphism_factor::dec_of(&words[0]).parse::<u64>().map_err(|_| "statevector base exceeds u64".to_string())?;
        let n_u = vox::morphism_factor::dec_of(&words[1]).parse::<u64>().map_err(|_| "statevector modulus exceeds u64".to_string())?;
        let result = shor_qft::run_shor_register(a_u, n_u, bits)?;
        println!("{}", shor_qft::report(&result));
        return Ok(());
    }
    // Wider than the exact statevector register: route to the tape
    // carrier. run_shor_big_report owns the wide case -- env-register
    // override, dynamic-scaling-wide arm, and the sidearm-guarded
    // period-miss path -- and refuses honestly where it must. It still
    // refuses a base not coprime to N: with gcd(a, N) = g > 1 the
    // factor g is already in hand and period-finding is moot, so the
    // refusal names the real state of the case.
    println!("{}", shor_qft::run_shor_big_report(words[0].clone(), words[1].clone(), bits)?);
    Ok(())
}
fn main() {
    if let Err(error) = run() { eprintln!("{error}"); std::process::exit(2); }
}
