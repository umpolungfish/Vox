//! Single-execution factorizer with N baked in as its IMASM numeral.
//!
//! The number is part of the program, not an argument. The decimal is consumed
//! before compilation by `vox numeral`, which emits the IMASM numeral word; that
//! word is baked in here at compile by option_env!, so the built artifact IS the
//! factorization of that one N and the run takes no input. Build and run in one
//! step with factor_one.sh, which does the encode-then-bake.
//!
//!   FACTOR_N_WORD="$(vox numeral 8051)" cargo build --release --bin factor_one
//!   ./target/release/factor_one
#![deny(warnings)]

fn main() {
    // Baked at compile time as the IMASM numeral word. The default is the numeral
    // for 0 so the crate still builds normally when unset; the wrapper sets it.
    let word: &str = option_env!("FACTOR_N_WORD").unwrap_or("⊢⊙⊡⊣");
    match ::vox::morphism_factor::parse_numeral(word) {
        Ok(n) => {
            if let Some(carrier) = option_env!("FACTOR_CARRIER_WORD").filter(|word| !word.is_empty()) {
                use vox::morphism_factor::{dec_of, divmod, emit_numeral, factor_with, parse_numeral, zero};
                if option_env!("FACTOR_DIAG_STAGES") == Some("1") {
                    let tower = vox::morphism_factor::construct_carrier(carrier)
                        .expect("baked carrier must construct");
                    let rounds = option_env!("FACTOR_DIAG_ROUNDS")
                        .unwrap_or("1").parse::<u64>().expect("baked diagnostic rounds");
                    let first_depth = option_env!("FACTOR_DIAG_DEPTH")
                        .unwrap_or("1").parse::<usize>().expect("baked diagnostic depth");
                    assert!((1..=tower.len()).contains(&first_depth));
                    for depth in first_depth..=tower.len() {
                        let name = vox::morphism_factor::morphism_name(tower[depth - 1]);
                        eprintln!("prefix_enter depth={depth} last={name}");
                        let start = std::time::Instant::now();
                        let selected = vox::morphism_factor::run_carrier_rounds(&tower[..depth], &n, rounds);
                        eprintln!("prefix_return depth={depth} last={name} elapsed_ms={} selected={}",
                            start.elapsed().as_millis(), selected.is_some());
                    }
                    return;
                }
                let pair = (|| {
                    let p = parse_numeral(&factor_with(carrier, word)?)?;
                    let (q, remainder) = divmod(&n, &p);
                    if !zero(&remainder) {
                        return Err("carrier factor leaves a source remainder".to_string());
                    }
                    vox::semiprime_descent::verify_pair(&n, &p, &q)?;
                    Ok::<_, String>((p, q))
                })();
                match pair {
                    Ok((p, q)) => {
                        println!("{} = {} x {}", dec_of(&n), dec_of(&p), dec_of(&q));
                        println!("factor_word {}\ncofactor_word {}\nproduct_verified true",
                            emit_numeral(&p), emit_numeral(&q));
                    }
                    Err(error) => {
                        eprintln!("{error}");
                        std::process::exit(2);
                    }
                }
            } else {
                println!("{}", ::vox::morphism_factor::repl_smart_factor(&n));
            }
        }
        Err(e) => {
            eprintln!("FACTOR_N_WORD was not an IMASM numeral: {e}");
            std::process::exit(2);
        }
    }
}
