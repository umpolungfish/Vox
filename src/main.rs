//! V⊙x CLI — lift an x86-64 ELF to twelve-glyph IMASM words and verdict them.
//!
//! The native decoder and lifter carry no external crates, so this builds on
//! its own. Subcommands mirror the surface of the original vox.py.

use ::vox::vox;
use ::vox::vox_decode;

fn usage() {
    eprintln!("V⊙x — control-flow closure auditor");
    eprintln!();
    eprintln!("  vox <file.so|.elf>        lift every function, tally verdicts");
    eprintln!("  vox lift <file>           same");
    eprintln!("  vox verdict <glyph-word>  verdict one word (T/B/N/F)");
    eprintln!("  vox classify <mn> [ops]   the glyph an instruction lifts to");
    eprintln!("  vox --selftest            planted open/closed forks");
    eprintln!();
    eprintln!("T closes · B holds a fork open across a terminal · N never forked ·");
    eprintln!("F is ill-typed (a ∋ with no ∈ to pair).");
}

fn lift_file(path: &str) -> i32 {
    let raw = match std::fs::read(path) {
        Ok(r) => r,
        Err(e) => { eprintln!("cannot read {}: {}", path, e); return 2; }
    };
    let (entry, segments) = vox::parse_elf(&raw);
    if segments.is_empty() {
        eprintln!("{}: no executable sections found", path);
        return 1;
    }
    let image = vox_decode::Image { segments };
    println!("{}  entry 0x{:x}  {} byte(s) of code", path, entry, image.total_bytes());

    let seeds = vox::elf_function_symbols(&raw);
    let w = vox_decode::walk(&image, entry, &seeds);
    let decoded: usize = w.functions.iter().map(|f| f.1.len()).sum();
    println!("  {} function(s), {} instruction(s)", w.functions.len(), decoded);
    println!("  claimed {}% of the image ({} of {} bytes)",
        w.claimed_percent(), w.claimed_bytes, w.total_bytes);

    let mut tally = [0usize; 4]; // T B N F
    let mut illtyped: Vec<(u64, String)> = Vec::new();
    for (start, f) in &w.functions {
        let word = vox::recompile_function(f);
        match vox::verdict(&word) {
            'T' => tally[0] += 1,
            'B' => tally[1] += 1,
            'N' => tally[2] += 1,
            _ => { tally[3] += 1; if illtyped.len() < 8 { illtyped.push((*start, vox::glyphs(&word))); } }
        }
    }
    println!();
    println!("  verdicts  T {}   B {}   N {}   F {}", tally[0], tally[1], tally[2], tally[3]);
    for (a, g) in &illtyped {
        let head: String = g.chars().take(90).collect();
        println!("    0x{:x}  {}", a, head);
    }
    0
}

fn selftest() -> i32 {
    // Planted glyph words: the auditor's own law, independent of any decoder.
    // ⊢ open ∈ fork ◻ commit ⊣ terminal ∋ merge
    let cases: &[(&str, &str, char)] = &[
        ("linear routine, never forks",        "⊢◻⊣",   'N'),
        ("fork that merges before terminal",   "⊢∈◻∋⊣", 'T'),
        ("fork held open across the terminal", "⊢∈◻⊣",  'B'),
        ("merge with nothing to pair",         "⊢∋⊣",   'F'),
    ];
    let mut ok = true;
    for (name, w, want) in cases {
        let word: Vec<char> = w.chars().collect();
        let got = vox::verdict(&word);
        let mark = if got == *want { "ok" } else { ok = false; "FAIL" };
        println!("  {:<38} {}  {}  (expect {})  {}", name, w, got, want, mark);
    }
    if ok { println!("selftest OK: the closure law holds."); 0 } else { eprintln!("selftest FAILED"); 1 }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let code = match args.first().map(|s| s.as_str()) {
        None | Some("-h") | Some("--help") | Some("help") => { usage(); 0 }
        Some("--selftest") | Some("selftest") => selftest(),
        Some("verdict") => {
            if args.len() < 2 { eprintln!("vox verdict <glyph-word>"); 1 }
            else {
                let word: Vec<char> = args[1..].join("").chars().collect();
                println!("{}", vox::glyphs(&word));
                println!("verdict {}", vox::verdict(&word));
                0
            }
        }
        Some("classify") => {
            if args.len() < 2 { eprintln!("vox classify <mnemonic> [operands]"); 1 }
            else {
                let ins = vox::Instruction { address: 0, mnemonic: args[1].to_lowercase(), op_str: args[2..].join(" ") };
                println!("{} {}", ins.mnemonic, vox::classify_instruction(&ins));
                0
            }
        }
        Some("lift") => { if args.len() < 2 { eprintln!("vox lift <file>"); 1 } else { lift_file(&args[1]) } }
        Some(path) => lift_file(path),
    };
    std::process::exit(code);
}
