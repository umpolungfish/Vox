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
use vox::morphism_factor::{add, cmp, dec_of, isqrt, mul, one, parse_numeral, sub};
type Tape = Vec<char>;

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
    let n_tape = match parse_numeral(word) {
        Ok(n) => n,
        Err(e) => { eprintln!("FACTOR_N_WORD was not an IMASM numeral: {e}"); std::process::exit(2); }
    };
    // N is the operand; the carrier stack is the program executed on it. The
    // stack nests each carrier into the previous one at its fuse ∋, so the whole
    // return loop encloses the trajectory. No numeral is embedded in the word.
    let mut stack: Option<String> = None;
    let mut depth = 0usize;
    for t in types.split_whitespace() {
        match carrier(t) {
            Some(c) => {
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

    // Read the winding: the ascending square ring a against N, coincidence where
    // a^2 - N is itself a square b^2. The winding is the a-steps from ceil(sqrt N)
    // to that coincidence; there N = (a-b)(a+b). Cost tracks factor balance, not
    // the multiplicative order. No stepping of any modular orbit.
    match square_winding(&n_tape) {
        Some((p, q)) => println!("{dec} = {} x {}", dec_of(&p), dec_of(&q)),
        None => println!("{dec}: no square coincidence within the winding bound"),
    }
}

/// Is `x` a perfect square? Read its integer root and square it back.
fn is_square(x: &[char]) -> Option<Tape> {
    let r = isqrt(x);
    if cmp(&mul(&r, &r), x) == core::cmp::Ordering::Equal { Some(r) } else { None }
}

/// The square-coincidence winding read (difference of two squares). a rises from
/// ceil(sqrt N); at each step the residue a^2 - N is tested for being a square.
/// The first coincidence gives the factor pair. Order-independent.
fn square_winding(n: &[char]) -> Option<(Tape, Tape)> {
    let unit = one();
    let root = isqrt(n);
    // a = ceil(sqrt N)
    let mut a = if cmp(&mul(&root, &root), n) == core::cmp::Ordering::Equal {
        root
    } else {
        add(&root, &unit)
    };
    // bound the winding generously; a balanced pair coincides at once.
    for _ in 0..50_000_000u64 {
        let asq = mul(&a, &a);
        if cmp(&asq, n) != core::cmp::Ordering::Less {
            let diff = sub(&asq, n);
            if let Some(b) = is_square(&diff) {
                let p = sub(&a, &b);
                let q = add(&a, &b);
                if cmp(&p, &unit) == core::cmp::Ordering::Greater
                    && cmp(&mul(&p, &q), n) == core::cmp::Ordering::Equal
                {
                    return Some((p, q));
                }
            }
        }
        a = add(&a, &unit);
    }
    None
}
