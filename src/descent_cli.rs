#![deny(warnings)]
use vox::godel_calculus::{decode, Structure};
use vox::morphism_factor::{dec_of, decimal_to_tape};
use vox::semiprime_descent::{run, until_closed, Limits, PROTOCOL};

pub fn execute(args: &[String]) -> Result<bool, String> {
    if args.is_empty() || matches!(args[0].as_str(), "--help" | "-h") {
        println!("semiprime_descent <decimal-source|cell-binary-word> [--word GLYPHS] [--seed DECIMAL] [--constant DECIMAL] [--attempts COUNT] [--steps COUNT] [--total-steps COUNT] [--until-closed]\nRuns ∈⊤⊥⊞∋ using native tape arithmetic. Bounded mode emits JSON and exits 1 on exhaustion. --until-closed traverses all cycle parameters, streams JSONL and has no attempt/time limit; --steps must be at least 2. Eventual closure assumes a semiprime and sufficient resources, without a practical runtime guarantee. Default word: {PROTOCOL}");
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
    let mut complete = false;
    let mut i = 1;
    while i < args.len() {
        if args[i] == "--until-closed" {
            complete = true;
            i += 1;
            continue;
        }
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
    if complete {
        if args
            .iter()
            .any(|arg| matches!(arg.as_str(), "--attempts" | "--total-steps"))
        {
            return Err("until-closed has no attempt/total budget; use only --steps for each finite attempt".to_string());
        }
        until_closed(
            source,
            seed,
            constant,
            &word,
            limits.steps_per_attempt,
            |report, attempts, steps| {
                use std::io::Write;
                let json = report.json();
                let status = if report.pair.is_some() {
                    "verified"
                } else {
                    "searching"
                };
                let mut out = std::io::stdout().lock();
                writeln!(out, "{{\"mode\":\"until-closed\",\"status\":\"{status}\",\"search_attempts\":\"{}\",\"search_steps\":\"{}\",\"attempt\":{json}}}", dec_of(attempts), dec_of(steps))
                .and_then(|_| out.flush()).map_err(|e| e.to_string())
            },
        )?;
        return Ok(true);
    }
    let report = run(source, seed, constant, &word, limits)?;
    println!("{}", report.json());
    Ok(report.pair.is_some())
}
