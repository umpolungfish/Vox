//! Register-bound numeral morphisms and source-baked native tape extraction.
#![deny(warnings)]
use std::cmp::Ordering;
use vox::morphism_factor as tape;

fn trace(word: &str, source: &str, extract: bool, submitted: Option<&[String]>) -> Result<Vec<char>, String> {
    let mut value = Vec::new();
    let mut frames = Vec::new();
    let mut fixed = false;
    let mut factors = Vec::new();
    let mut cursor = 0usize;
    let mut linked = Vec::new();
    let expected = tape::decimal_to_tape(source).ok_or("source must be a decimal natural")?;
    let chars: Vec<char> = word.chars().collect();
    if chars.first() != Some(&'⊢') || chars.last() != Some(&'⊣') {
        return Err("numeral morphisms require source and terminal boundaries".into());
    }
    for (i, symbol) in chars.iter().copied().enumerate() {
        let before = tape::emit_numeral(&tape::trim(value.clone()));
        let before_state = format!("{}:{}:{}:{}:{}:{}", value.len(), frames.len(), cursor, linked.len(), fixed, factors.len());
        println!("started\t{i}\t{symbol}");
        if fixed && symbol != '⊣' { return Err("fixed numeral admits only release".into()); }
        match symbol {
            '⊢' if i == 0 => {},
            '∈' => frames.push(value.len()),
            '⊤' | '⊥' if !frames.is_empty() => value.push(symbol),
            '≺' if !frames.is_empty() => {
                // Deposited cells belong to the retained frame. Clearing the
                // transient cursor leaves those banked cells intact.
                println!("return\t{i}\tbanked_cells={}\ttransient_cells=0", value.len());
                cursor = 0;
            },
            '∋' => { frames.pop().ok_or("numeral fuse has no retained frame")?; },
            '≻' => cursor += 1,
            '⋈' => linked.push(tape::emit_numeral(&tape::trim(value.clone()))),
            '⊙' => {},
            '⊡' if frames.is_empty() && !value.is_empty() => {
                if tape::cmp(&tape::trim(value.clone()), &expected) != Ordering::Equal {
                    return Err(format!("numeral source mismatch: word reconstructs {}, requested {source}", tape::dec_of(&value)));
                }
                if extract {
                    let (found, route) = tape::smart_factor(&tape::trim(value.clone()));
                    factors = found.iter().map(|f| tape::emit_numeral(f)).collect();
                    eprintln!("{route}");
                } else if let Some(words) = submitted {
                    factors = words.to_vec();
                }
                fixed = true;
            },
            '⊣' if fixed && i == chars.len() - 1 => {
                if extract || submitted.is_some() {
                    verify(&expected, &factors)?;
                    for word in &factors {
                        println!("factor\t{}\t{word}", tape::dec_of(&tape::parse_numeral(word)?));
                    }
                    if extract { println!("producer\tvox_native_tape_smart_factor"); }
                }
            },
            _ => return Err(format!("unbound numeral operator {symbol} at {i}")),
        }
        let after = tape::emit_numeral(&tape::trim(value.clone()));
        let after_state = format!("{}:{}:{}:{}:{}:{}", value.len(), frames.len(), cursor, linked.len(), fixed, factors.len());
        println!("event\t{i}\t{symbol}\t{before}\t{after}\t{before_state}\t{after_state}");
    }
    Ok(tape::trim(value))
}

fn bind(word: &str, source: &str) -> Result<Vec<char>, String> {
    let value = trace(word, source, false, None)?;
    let expected = tape::decimal_to_tape(source).ok_or("source must be a decimal natural")?;
    if tape::cmp(&value, &expected) != Ordering::Equal {
        return Err(format!("numeral source mismatch: word reconstructs {}, requested {source}", tape::dec_of(&value)));
    }
    println!("source\t{}\t{}", tape::dec_of(&value), tape::emit_numeral(&value));
    Ok(value)
}

fn verify(n: &[char], words: &[String]) -> Result<(), String> {
    if words.len() < 2 { return Err("no nontrivial extracted factor multiset".into()); }
    let mut product = tape::one();
    for word in words {
        let factor = tape::parse_numeral(word)?;
        if tape::cmp(&factor, &tape::one()) != Ordering::Greater || tape::cmp(&factor, n) != Ordering::Less {
            return Err("factor is not strictly interior to source".into());
        }
        if !tape::miller_rabin(&factor) { return Err("factor failed native primality screening".into()); }
        product = tape::mul(&product, &factor);
    }
    if tape::cmp(&tape::trim(product), n) != Ordering::Equal {
        return Err("native tape factor product differs from sealed source".into());
    }
    println!("verified\tnative_tape_product\tnative_miller_rabin_screening");
    Ok(())
}

fn execute() -> Result<(), String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().map(String::as_str) == Some("--inspect") {
        if args.len() != 3 { return Err("--inspect WORD DECIMAL_SOURCE is preparation-only".into()); }
        bind(&args[1], &args[2])?;
        return Ok(());
    }
    let word = option_env!("EXCRIBE_NUMERAL_WORD").ok_or("build with a bound numeral word")?;
    let source = option_env!("EXCRIBE_NUMERAL_SOURCE").ok_or("build with an independently bound source")?;
    if args.first().map(String::as_str) == Some("--verify") {
        if args.len() < 3 { return Err("--verify requires canonical factor words".into()); }
        trace(word, source, false, Some(&args[1..]))?;
        return Ok(());
    }
    if !args.is_empty() { return Err("source-baked extraction takes no runtime inputs".into()); }
    trace(word, source, true, None)?;
    Ok(())
}

fn main() {
    if let Err(error) = execute() {
        eprintln!("NUMERAL ERROR: {error}");
        std::process::exit(1);
    }
}
