//! One-value arbitrary-width Shor membrane.
//!
//! `BAKED_INDEX` selects exactly one line from the compile-time embedded
//! bvalsd payload.  The resulting executable has no runtime operand input.

extern crate alloc;
#[allow(dead_code)]
#[path = "../shor_qft.rs"] mod shor_qft;
#[allow(dead_code)]
#[path = "../membrane_complex.rs"] mod membrane_complex;
#[allow(dead_code)]
#[path = "../baked_membrane.rs"] mod baked_membrane;
#[path = "../fde_shor_membrane.rs"] mod fde_shor_membrane;

const BAKED_VALUES: &str = include_str!("/home/mrnob0dy666/imsgct/bvalsd.txt");

fn main() {
    let result = (|| -> Result<String, String> {
        let index: usize = option_env!("BAKED_INDEX")
            .ok_or("BAKED_INDEX was not supplied at compile time")?
            .parse().map_err(|_| "invalid BAKED_INDEX")?;
        let value = BAKED_VALUES.lines().filter(|line| !line.trim().is_empty())
            .nth(index).ok_or("BAKED_INDEX is outside bvalsd.txt")?;
        let n = vox::morphism_factor::decimal_to_tape(value.trim())
            .ok_or("invalid baked decimal")?;
        let reg_bits: usize = option_env!("REGISTER_BITS")
            .and_then(|s| s.parse().ok())
            .unwrap_or_else(|| (n.len() * 2).max(12));
        let report = shor_qft::run_shor_big_report(
            vox::morphism_factor::tape_u64(2), n, reg_bits
        )?;
        Ok(report)
    })();
    match result {
        Ok(report) => println!("{report}"),
        Err(error) => { eprintln!("{error}"); std::process::exit(2); }
    }
}
