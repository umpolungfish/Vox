#![deny(warnings)]
use std::path::PathBuf;
use std::process::Command;
use vox::godel_calculus::{Nat, Operator};
use vox::{godel_analyzer, godel_calculus, godel_product, morphism_factor};

fn tape_from_word(word: &str) -> Result<Vec<char>, String> {
    let reading = godel_calculus::decode(word).map_err(|error| error.to_string())?;
    match reading.structure {
        godel_calculus::Structure::CellBinary { bits_le } => Ok(bits_le
            .into_iter()
            .map(|bit| if bit { vox::vox::EVALF } else { vox::vox::EVALT })
            .collect()),
        _ => Err("factor lane is not a cell-binary word".to_string()),
    }
}

fn folded_semiprime_pair(source_word: &str) -> Option<(Nat, Nat)> {
    let source_tape = tape_from_word(source_word).ok()?;
    let (factors, _) = morphism_factor::smart_factor(&source_tape);
    if factors.len() != 2 {
        return None;
    }
    let p = Nat::from_decimal(&morphism_factor::dec_of(&factors[0]))?;
    let q = Nat::from_decimal(&morphism_factor::dec_of(&factors[1]))?;
    let p_word = godel_calculus::encode_cell_binary(&p);
    let q_word = godel_calculus::encode_cell_binary(&q);
    (morphism_factor::miller_rabin(&tape_from_word(&p_word).ok()?)
        && morphism_factor::miller_rabin(&tape_from_word(&q_word).ok()?))
    .then_some((p, q))
}

fn dispatch(args: &[&str]) -> Result<String, String> {
    match args.first().copied().unwrap_or("help") {
        "analyze" | "lte2" => godel_analyzer::command(args),
        "product" | "separate" | "factor" | "shiab" | "frame" => godel_product::command(args),
        "selftest" | "verify" => {
            let mut out = godel_calculus::selftest_report()?;
            out.push_str(&godel_analyzer::selftest_report()?);
            out.push_str(&godel_product::selftest_report()?);
            Ok(out)
        }
        "help" | "-h" | "--help" => Ok(format!(
            "{}{}{}",
            godel_calculus::help(),
            godel_analyzer::help_addendum(),
            godel_product::help_addendum()
        )),
        _ => godel_calculus::command(args),
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    match dispatch(&refs) {
        Ok(mut report) => {
            if refs.first().copied() == Some("analyze")
                && report.contains("\nhandoff                   prime_winding factor ")
            {
                let source_decimal = report
                    .lines()
                    .find_map(|line| line.strip_prefix("value                      "))
                    .unwrap_or_else(|| {
                        eprintln!("bounded sieve hand-off has no decoded source value");
                        std::process::exit(1);
                    });
                let source = Nat::from_decimal(source_decimal)
                    .expect("decoded handoff source is a decimal natural");
                let source_word = godel_calculus::encode_cell_binary(&source);
                let (factor_pair, engine, engine_report) =
                    if let Some(pair) = folded_semiprime_pair(&source_word) {
                        let detail = format!("{source} = {} × {}\n", pair.0, pair.1);
                        (
                            pair,
                            "Vox folded tape factor",
                            detail,
                        )
                    } else {
                        let script = std::env::var_os("G_MOMONADOS_RUN_CMDS")
                            .map(PathBuf::from)
                            .unwrap_or_else(|| {
                                PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                                    .parent()
                                    .unwrap_or_else(|| std::path::Path::new("."))
                                    .join("G-mOMonadOS/run_cmds.sh")
                            });
                        let output = Command::new("bash")
                            .arg(&script)
                            .arg(format!("prime_winding factor {source_decimal}"))
                            .output()
                            .unwrap_or_else(|error| {
                                eprintln!("could not run {}: {error}", script.display());
                                std::process::exit(1);
                            });
                        if !output.status.success() {
                            eprint!("{}", String::from_utf8_lossy(&output.stderr));
                            eprint!("{}", String::from_utf8_lossy(&output.stdout));
                            std::process::exit(output.status.code().unwrap_or(1));
                        }
                        let pair = String::from_utf8_lossy(&output.stdout)
                            .lines()
                            .find_map(|line| {
                                let (source_text, factors) = line.split_once(" = ")?;
                                source_text
                                    .trim_end()
                                    .ends_with(source_decimal)
                                    .then_some(factors.trim())
                            })
                            .and_then(|factors| factors.split_once('×'))
                            .and_then(|(p, q)| {
                                Some((Nat::from_decimal(p.trim())?, Nat::from_decimal(q.trim())?))
                            })
                            .unwrap_or_else(|| {
                                eprintln!(
                                    "prime_winding returned no parseable factor pair for {source_decimal}"
                                );
                                std::process::exit(1);
                            });
                        (
                            pair,
                            "prime_winding factor",
                            String::from_utf8_lossy(&output.stdout).into_owned(),
                        )
                    };
                let p_word = godel_calculus::encode_cell_binary(&factor_pair.0);
                let q_word = godel_calculus::encode_cell_binary(&factor_pair.1);
                let closure = godel_calculus::check(&p_word, Operator::Mul, &q_word, &source_word)
                    .unwrap_or_else(|error| {
                        eprintln!("factor product word did not decode: {error}");
                        std::process::exit(1);
                    });
                if !closure.valid {
                    eprintln!("factor pair failed godel check mul");
                    std::process::exit(1);
                }
                let p_prime = tape_from_word(&p_word)
                    .is_ok_and(|tape| vox::morphism_factor::miller_rabin(&tape));
                let q_prime = tape_from_word(&q_word)
                    .is_ok_and(|tape| vox::morphism_factor::miller_rabin(&tape));
                if !p_prime || !q_prime {
                    eprintln!("factor lanes did not both close as prime on their word tapes");
                    std::process::exit(1);
                }
                report.push_str(&format!("stage4.engine               {engine}\n"));
                report.push_str(&engine_report);
                report.push_str(&format!(
                    "stage4.factor-pair          {} × {}\nstage4.product              {}\nstage4.godel.check.mul      PASS\nstage4.word-primality       PASS\nstage4.semiprime-closure    T\n",
                    factor_pair.0,
                    factor_pair.1,
                    factor_pair.0.mul(&factor_pair.1),
                ));
            }
            print!("{report}");
        }
        Err(error) => {
            eprint!("{error}");
            if !error.ends_with('\n') {
                eprintln!();
            }
            std::process::exit(1);
        }
    }
}
