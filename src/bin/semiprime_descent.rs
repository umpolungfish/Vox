#![deny(warnings)]
use vox::godel_calculus::{decode, Structure};
use vox::morphism_factor::decimal_to_tape;
use vox::semiprime_descent::{run, Limits, PROTOCOL};

fn execute() -> Result<bool, String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() || matches!(args[0].as_str(), "--help" | "-h") {
        println!("semiprime_descent <decimal-source|cell-binary-word> [--word GLYPHS] [--seed DECIMAL] [--constant DECIMAL] [--attempts COUNT] [--steps COUNT] [--total-steps COUNT]\nRuns ∈⊤⊥⊞∋ using native tape arithmetic; emits JSON and exits 1 on budget exhaustion. Default word: {PROTOCOL}");
        return Ok(true);
    }
    let source = if let Some(n) = decimal_to_tape(&args[0]) {
        n
    } else {
        let reading = decode(&args[0]).map_err(|e| e.to_string())?;
        match reading.structure {
            Structure::CellBinary { bits_le } => bits_le
                .into_iter()
                .map(|b| if b { '⊥' } else { '⊤' })
                .collect(),
            _ => return Err("source must be decimal or a cell-binary numeral".to_string()),
        }
    };
    let mut seed = decimal_to_tape("2").unwrap();
    let mut constant = decimal_to_tape("1").unwrap();
    let mut word = PROTOCOL.to_string();
    let mut limits = Limits::default();
    let mut i = 1;
    while i < args.len() {
        let value = args
            .get(i + 1)
            .ok_or_else(|| format!("missing value for {}", args[i]))?;
        match args[i].as_str() {
            "--word" => word = value.clone(),
            "--seed" => seed = decimal_to_tape(value).ok_or("seed must be decimal")?,
            "--constant" => constant = decimal_to_tape(value).ok_or("constant must be decimal")?,
            "--attempts" => {
                limits.attempts = value
                    .parse()
                    .map_err(|_| "attempts must be a positive integer")?
            }
            "--steps" => {
                limits.steps_per_attempt = value
                    .parse()
                    .map_err(|_| "steps must be a positive integer")?
            }
            "--total-steps" => {
                limits.total_steps = value
                    .parse()
                    .map_err(|_| "total steps must be a positive integer")?
            }
            _ => return Err(format!("unknown argument {}", args[i])),
        }
        i += 2;
    }
    let report = run(source, seed, constant, &word, limits)?;
    println!("{}", report.json());
    Ok(report.pair.is_some())
}

fn main() {
    match execute() {
        Ok(true) => {}
        Ok(false) => std::process::exit(1),
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(2);
        }
    }
}
