//! A factoring transformation baked into a membrane, run as pure execution.
//!
//! N is baked as its IMASM numeral before compilation, so the built artifact IS
//! this one transformation and the run takes no input. The membrane splits N
//! (delta into the two lanes), the core multiply-mu closes only on p*q = N, and
//! the factor pair is read off the closure.
fn main() {
    let word: &str = option_env!("FACTOR_N_WORD").unwrap_or("⊢⊙⊡⊣");
    match ::vox::factorization_31_membrane::factor_n(word) {
        Ok(report) => println!("{report}"),
        Err(e) => { eprintln!("factor membrane: {e}"); std::process::exit(2); }
    }
}
