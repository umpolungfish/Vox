//! V⊙x CLI — lift an x86-64 ELF to twelve-glyph IMASM words and verdict them.
//!
//! The native decoder and lifter carry no external crates, so this builds on
//! its own. Subcommands mirror the surface of the original vox.py.

use ::vox::vox;
use ::vox::vox_decode;
use ::vox::lanes;
use ::vox::x86;
use ::vox::{imasm_module, imasm_vm, loader};

fn usage() {
    eprintln!("V⊙x — control-flow closure auditor");
    eprintln!();
    eprintln!("  vox <file.so|.elf>        lift every function, tally verdicts");
    eprintln!("  vox lift <file>           same");
    eprintln!("  vox run <sym> --args a,b <file>   recompile and RUN a function");
    eprintln!("  vox imasm <file>          emit the executable IMASM module");
    eprintln!("  vox word <file>           emit the structure word per function");
    eprintln!("  vox verdict <glyph-word>  verdict one word (T/B/N/F)");
    eprintln!("  vox evm <hex>             lift EVM bytecode, verdict its closure");
    eprintln!("  vox wasm <hex>            lift a WASM function body, verdict it");
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
    let l = loader::load(&raw);
    if l.code.is_empty() {
        eprintln!("{}: no executable sections found", path);
        return 1;
    }
    let image = vox_decode::Image { segments: l.code };
    println!("{}  {}  entry 0x{:x}  {} byte(s) of code", path, l.format, l.entry, image.total_bytes());
    let mut seeds: Vec<u64> = l.symbols.values().copied().collect(); seeds.push(l.entry);
    let w = vox_decode::walk(&image, l.entry, &seeds);
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

fn lane(isa: &str, word: &[char]) -> i32 {
    let v = vox::verdict(word);
    let mark = if v == 'B' { "   <-- FINDING (fork open across commit)" } else { "" };
    println!("{:<6} {}  {}{}", isa, v, vox::glyphs(word), mark);
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
    // EVM and WASM bytecode: a state commit inside an unmerged branch (B) vs
    // paths that rejoin before the commit (T). Same law, real bytes.
    let bc: &[(&str, char, Vec<char>)] = &[
        ("EVM reentrant (commit in unmerged branch)", 'B', lanes::evm_word("600160075755005b00")),
        ("EVM guarded  (paths merge before commit)",  'T', lanes::evm_word("6001600657545b5500")),
        ("WASM reentrant (commit + return in branch)", 'B', lanes::wasm_word("20000440410141003602000f0b0b")),
        ("WASM guarded  (if merges before commit)",    'T', lanes::wasm_word("2000044010000b410041003602000b")),
    ];
    for (name, want, word) in bc {
        let got = vox::verdict(word);
        let mark = if got == *want { "ok" } else { ok = false; "FAIL" };
        println!("  {:<42} {}  (expect {})  {}", name, got, want, mark);
    }
    if ok { println!("selftest OK: the closure law holds on x86, EVM and WASM."); 0 }
    else { eprintln!("selftest FAILED"); 1 }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let code = match args.first().map(|s| s.as_str()) {
        None | Some("-h") | Some("--help") | Some("help") => { usage(); 0 }
        Some("--selftest") | Some("--self-test") | Some("selftest") | Some("self-test") => selftest(),
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
        Some("evm") | Some("--evm") => { if args.len() < 2 { eprintln!("vox evm <hex>"); 1 } else { lane("EVM", &lanes::evm_word(&args[1])) } }
        Some("wasm") | Some("--wasm") => { if args.len() < 2 { eprintln!("vox wasm <hex>"); 1 } else { lane("WASM", &lanes::wasm_word(&args[1])) } }
        Some("run") => {
            // vox run SYMBOL --args a,b FILE   (order-tolerant)
            let mut sym=String::new(); let mut argv:Vec<i64>=Vec::new(); let mut file=String::new(); let mut i=1;
            while i < args.len() {
                match args[i].as_str() {
                    "--args" => { i+=1; if i<args.len() { for a in args[i].split(',') { let a=a.trim(); if !a.is_empty() {
                        let v = if let Some(h)=a.strip_prefix("0x") { i64::from_str_radix(h,16).unwrap_or(0) } else { a.parse().unwrap_or(0) }; argv.push(v);} } } }
                    other => { if sym.is_empty() { sym=other.to_string(); } else { file=other.to_string(); } }
                }
                i+=1;
            }
            if sym.is_empty() || file.is_empty() { eprintln!("vox run <symbol> --args a,b <file>"); return; }
            let raw = std::fs::read(&file).expect("read");
            let syms = imasm_module::symbols(&raw);
            let addr = match syms.get(&sym) { Some(a)=>*a, None=>{ eprintln!("no symbol '{}' in {}", sym, file); std::process::exit(1);} };
            let module = imasm_module::emit(&raw);
            let mut m = imasm_vm::Machine::new(&module);
            match m.call(addr, &argv, 50_000_000) {
                Ok(r) => println!("{}({}) = {}   [{} steps in the twelve]", sym, argv.iter().map(|a|a.to_string()).collect::<Vec<_>>().join(", "), r, m.steps),
                Err(imasm_vm::Stop::SysExit(c)) => println!("{}(...) called exit({})   [{} steps in the twelve]", sym, c, m.steps),
                Err(imasm_vm::Stop::Halt(e)) => println!("{}(...) halted: {}   [{} steps]", sym, e, m.steps),
            }
            std::process::exit(0);
        }
        Some("imasm") => { if args.len()<2 { eprintln!("vox imasm <file>"); return; }
            let raw=std::fs::read(&args[1]).expect("read"); print!("{}", imasm_module::emit(&raw)); std::process::exit(0); }
        Some("word") | Some("words") => { if args.len()<2 { eprintln!("vox word <file>"); return; }
            let raw=std::fs::read(&args[1]).expect("read"); println!("{}", imasm_module::words(&raw)); std::process::exit(0); }
        Some("disasm") => {
            if args.len() < 2 { eprintln!("vox disasm <file> [symbol]"); return; }
            let raw = std::fs::read(&args[1]).expect("read");
            let (entry, segments) = vox::parse_elf(&raw);
            let image = vox_decode::Image { segments };
            let seeds = vox::elf_function_symbols(&raw);
            let w = vox_decode::walk(&image, entry, &seeds);
            for (start, f) in &w.functions {
                if args.len() > 2 { /* filter by address later */ }
                let _ = start;
                for ins in f {
                    if let Some(bytes) = image.bytes_at(ins.address) {
                        if let Some(d) = x86::decode(bytes, ins.address) {
                            let ops: Vec<String> = d.ops.iter().map(|o| o.field()).collect();
                            println!("{:x}	{}	{}", d.addr, d.mnemonic, ops.join(" "));
                        } else {
                            println!("{:x}	??? (undecoded)", ins.address);
                        }
                    }
                }
            }
            std::process::exit(0);
        }
        Some("lift") => { if args.len() < 2 { eprintln!("vox lift <file>"); 1 } else { lift_file(&args[1]) } }
        Some(flag) if flag.starts_with('-') => { eprintln!("vox: unknown option {}\n", flag); usage(); 2 }
        Some(path) => lift_file(path),
    };
    std::process::exit(code);
}
