//! Baked executable for the 31-step resident factorization membrane.

fn main() {
    let word = option_env!("FACTOR_N_WORD").unwrap_or("⊢⊙⊡⊣");
    match ::vox::factorization_31_membrane::dispatch_report_word(word) {
        Ok(report) => println!("{report}"),
        Err(e) => { eprintln!("factorization membrane failed: {e}"); std::process::exit(1); }
    }
}
