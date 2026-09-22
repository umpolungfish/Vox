//! Heterogeneous hypernest factorizer with N baked in as its IMASM numeral.
//!
//! The number and the carrier stack are part of the program, not arguments. The
//! decimal is consumed before compilation by `vox numeral`, and the carrier
//! sequence (phase, shor, fib, ...) by HYPERSTACK_TYPES; both are baked at
//! compile by option_env!. The built artifact IS the heterogeneous membrane over
//! that one N: the numeral is loaded into the first carrier, that into the next,
//! and so on, and the run takes no input and factors instantly.
//!
//!   ./hyperstack_one.sh 8051 phase shor fib
use vox::morphism_factor::{parse_numeral, repl_smart_factor};

/// The carrier registry: a name to the ob3ect glyph word for that operator type.
fn carrier(name: &str) -> Option<&'static str> {
    Some(match name {
        "phase"      => "⊢⊙∈≻⊤⋈≺⊥⊞∋⊡⋈⊙⊣",
        "shor"       => "⊢∈≻⋈⊞∈⊤≻⊥≺∋⊙⋈⊡⊣",
        "fib" | "fibonacci" | "anyon" => "⊢⊙∈≻⋈⊤≻⊥⊞≺⋈∈⊤⊥∋⊡⋈≻⊙∋⊣",
        "arithmetic" => "⊢⊙∈≻⊤⋈≺⊥⊞∋⊡⋈⊙⊣",
        "branch"     => "⊢∈⊤⊥∋⊡⊣",
        // The factoring membrane ob3ects, by their grounded glyph words.
        "mk" | "msep" => "⊢∈≻⊤≺⊥⊞⋈∋⊙⊡⊣",            // factor-separating M_κ
        "braider"     => "⊢∈≻⊤≺⊥⊞⋈∋⊙⊡⊣",            // prime-number inverse braider
        "imprime"     => "⊢⊣≻∈⊤⋈≺⊥⊞⊙⋈∈⊤≺⊥∋⊡⊣",       // imscribing prime factorizer
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
    let n = match parse_numeral(word) {
        Ok(n) => n,
        Err(e) => { eprintln!("FACTOR_N_WORD was not an IMASM numeral: {e}"); std::process::exit(2); }
    };
    // Build the baked membrane's identity: N's numeral loaded through the stack.
    let mut cur = String::from(word);
    let mut depth = 0usize;
    for t in types.split_whitespace() {
        match carrier(t) {
            Some(c) => { cur = frame_with(c, &cur); depth += 1; }
            None => { eprintln!("unknown carrier type '{t}'"); std::process::exit(2); }
        }
    }
    let winding = cur.matches('⊡').count();
    println!("baked heterogeneous membrane: N through [{}]  (depth {}, ⊡ winding {})", types, depth, winding);
    // Factor instantly by reading the winding of the baked numeral.
    println!("{}", repl_smart_factor(&n));
}
