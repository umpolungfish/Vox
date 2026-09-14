//! Single-execution factorizer with N baked in at compile time.
//!
//! The number is part of the program, not an argument: FACTOR_N is read at
//! compile by option_env!, so the built artifact IS the factorization of that
//! one N. Build and run it in one step with factor_one.sh, or:
//!   FACTOR_N=8051 cargo build --release --bin factor_one && ./target/release/factor_one

fn main() {
    // Baked at compile time. Defaults to a marker when unset so the crate still
    // builds normally; the wrapper always sets it.
    let n_dec: &str = option_env!("FACTOR_N").unwrap_or("0");
    match ::vox::morphism_factor::decimal_to_tape(n_dec) {
        Some(n) => println!("{}", ::vox::morphism_factor::repl_smart_factor(&n)),
        None => {
            eprintln!("FACTOR_N was not a decimal: {n_dec:?}");
            std::process::exit(2);
        }
    }
}
