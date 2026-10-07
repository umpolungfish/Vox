//! Heterogeneous hypernest factorizer with N baked in as its IMASM numeral.
//!
//! The number and the carrier stack are part of the program, not arguments. The
//! decimal is consumed before compilation by `vox numeral`, and the carrier
//! sequence (phase, shor, fib, ...) by HYPERSTACK_TYPES; both are baked at
//! compile by option_env!. The built artifact IS the heterogeneous membrane over
//! that one N: the numeral is loaded into the first carrier, that into the next,
//! and so on, and the run takes no input.
//!
//!   ./hyperstack_one.sh 8051 phase shor fib
#![deny(warnings)]
use vox::morphism_factor::{cmp, construct_carrier, dec_of, divmod, mul, morphism_name, one, parse_numeral, run_carrier_rounds};

/// The carrier registry: a name to the ob3ect glyph word for that operator type.
fn carrier(name: &str) -> Option<&'static str> {
    Some(match name {
        "phase"      => "⊢⊙∈≻⊤⋈≺⊥⊞∋⊡⋈⊙⊣",
        "shor"       => "⊢∈≻⋈⊞∈⊤≻⊥≺∋⊙⋈∋⊡⊣",
        "fib" | "fibonacci" | "anyon" => "⊢⊙∈≻⋈⊤≻⊥⊞≺⋈∈⊤⊥∋⊡⋈≻⊙∋⊣",
        "arithmetic" => "⊢⊙∈≻⊤⋈≺⊥⊞∋⊡⋈⊙⊣",
        "branch"     => "⊢∈⊤⊥∋⊙⊡⊣",
        // The factoring membrane ob3ects, by their grounded glyph words.
        "mk" | "msep" => "⊢∈≻⊤≺⊥⊞⋈∋⊙⊡⊣",            // factor-separating M_κ
        "braider"     => "⊢∈≻⊤≺⊥⊞⋈∋⊙⊡⊣",            // prime-number inverse braider
        "imprime"     => "⊢≻∈⊤⋈≺⊥⊞⊙⋈∈⊤≺⊥∋∋⊡⊣",       // imscribing prime factorizer (fork closed)
        "fixation"    => "⊢⊙≻∈⊤⋈⊥≺⊞∋⊡⊣",             // post-membrane numerical fixation
        "divisor"     => "⊢⊣≻∈⊤⋈≺⊥⊞⊙∋⋈∈⊤≺⊥∋⊡⊣",       // closed prime factorizer, divisor search
        "semiprime" | "period" => "⊢∈≻⋈∈⊤≺⊥∋∈⊤⊥⊞∋⊙≺⋈∋⊡⊣≻≺⋈⊞⊥⊤∋∈⊡⊙⊣⊣", // semiprime factorizer, period-combining
        _ => return None,
    })
}

/// Load a payload into a carrier: place it immediately adjacent to the carrier's
/// fuse ∋, so the payload rides into the FFUSE and the return loop closes over
/// it. The insertion is at the fuse of the level being inserted into (the first
/// ∋), never after the ∈ fork.
fn frame_with(carrier: &str, payload: &str) -> String {
    match carrier.find('∋') {
        Some(i) => {
            let mut s = String::with_capacity(carrier.len() + payload.len());
            s.push_str(&carrier[..i]);
            s.push_str(payload);
            s.push_str(&carrier[i..]);
            s
        }
        None => { let mut s = String::from(carrier); s.push_str(payload); s }
    }
}

fn main() {
    // Baked at compile time: the IMASM numeral for N, and the carrier stack.
    let word: &str = option_env!("FACTOR_N_WORD").unwrap_or("⊢⊙⊡⊣");
    let types: &str = option_env!("HYPERSTACK_TYPES").unwrap_or("phase shor fib");
    let n_tape = match parse_numeral(word) {
        Ok(n) => n,
        Err(e) => { eprintln!("FACTOR_N_WORD was not an IMASM numeral: {e}"); std::process::exit(2); }
    };
    // N is the operand; the carrier stack is the program executed on it. The
    // stack nests each carrier into the previous one at its fuse ∋, so the whole
    // return loop encloses the trajectory. No numeral is embedded in the word.
    let mut stack: Option<String> = None;
    let mut depth = 0usize;
    let mut tower = Vec::new();
    for t in types.split_whitespace() {
        match carrier(t) {
            Some(c) => {
                let operators = match construct_carrier(c) {
                    Ok(operators) if !operators.is_empty() => operators,
                    Ok(_) => { eprintln!("carrier '{t}' has no executable morphisms"); std::process::exit(2); }
                    Err(error) => { eprintln!("carrier '{t}' cannot execute: {error}"); std::process::exit(2); }
                };
                tower.splice(0..0, operators);
                stack = Some(match stack {
                    Some(inner) => frame_with(c, &inner),
                    None => String::from(c),
                });
                depth += 1;
            }
            None => { eprintln!("unknown carrier type '{t}'"); std::process::exit(2); }
        }
    }
    let program = stack.unwrap_or_default();
    let winding = program.matches('⊡').count();
    let dec = dec_of(&n_tape);
    println!("baked heterogeneous membrane: N={dec} through [{types}]  (depth {depth}, ⊡ winding {winding})");

    println!("executing morphisms: {}", tower.iter().map(|op| morphism_name(op)).collect::<Vec<_>>().join(" "));
    let rounds = option_env!("HYPERSTACK_ROUNDS").unwrap_or("50000000")
        .parse::<u64>().ok().filter(|rounds| *rounds > 0)
        .unwrap_or_else(|| { eprintln!("HYPERSTACK_ROUNDS must be a positive integer"); std::process::exit(2); });
    match run_carrier_rounds(&tower, &n_tape, rounds) {
        Some(p) => {
            let (q, remainder) = divmod(&n_tape, &p);
            if cmp(&p, &one()) != core::cmp::Ordering::Greater
                || cmp(&q, &one()) != core::cmp::Ordering::Greater
                || dec_of(&remainder) != "0"
                || cmp(&mul(&p, &q), &n_tape) != core::cmp::Ordering::Equal
            {
                eprintln!("carrier returned no nontrivial verified factor pair");
                std::process::exit(1);
            }
            println!("{dec} = {} x {}", dec_of(&p), dec_of(&q));
        }
        None => { eprintln!("{dec}: carrier returned no factor within the round bound"); std::process::exit(1); }
    }
}
