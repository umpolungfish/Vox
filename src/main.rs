//! V⊙x CLI — lift an x86-64 ELF to twelve-glyph IMASM words and verdict them.
//!
//! The native decoder and lifter carry no external crates, so this builds on
//! its own. Subcommands mirror the surface of the original vox.py.

use ::vox::vox;
use ::vox::vox_decode;
use ::vox::lanes;
use ::vox::genetic;
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
    eprintln!("  vox verdict --tsv <file>   verdict name<TAB>word lines in bulk");
    eprintln!("  vox evm <hex>             lift EVM bytecode, verdict its closure");
    eprintln!("  vox wasm <hex>            lift a WASM function body, verdict it");
    eprintln!("  vox rna <seq>             lift a coding sequence, verdict the transcript");
    eprintln!("  vox self                  lift V⊙x's own image and read it back");
    eprintln!("  vox pyc <file.pyc>        lift every code object in a .pyc, verdict each");
    eprintln!("  vox classify <mn> [ops]   the glyph an instruction lifts to");
    eprintln!("  vox --selftest            planted open/closed forks");
    eprintln!();
    eprintln!("T closes · B holds a fork open across a terminal · N never forked ·");
    eprintln!("F is ill-typed (a ∋ with no ∈ to pair).");
}


/// A mode-aware linear-sweep audit: decode every executable byte at the given
/// width, split into functions at each terminal, verdict each. Used where
/// recursive descent's length decoder does not apply (32-bit x86).
/// Read a coding sequence as a word in the twelve and verdict it.
fn rna(seq: &str) -> i32 {
    let t = genetic::lift_rna(seq);
    if t.word.is_empty() {
        eprintln!("no promoted codon in that sequence");
        return 1;
    }
    println!("{:<8}{:<6}{:<16}{}", "CODON", "AA", "AXIS", "GLYPH");
    for r in &t.reading {
        println!("{:<8}{:<6}{:<16}{}", r.codon, r.aa, r.axis, r.glyph);
    }
    println!();
    println!("frame    {} at offset {}", if t.implicit_frame { "no AUG, read from the start" } else { "AUG" }, t.start);
    println!("word     {}", vox::glyphs(&t.word));
    match t.stopped {
        Some(s) => println!("stop     {}", s),
        None => println!("stop     (none: the sequence ran out before a stop)"),
    }
    println!("verdict  {}", vox::verdict(&t.word));
    0
}

/// The organism reads itself.
///
/// V⊙x lifts every substrate it is pointed at; pointed at its own image it
/// lifts the lifter. What comes back is not decoration: the self-image is the
/// only binary whose source is here to check the reading against.
/// Lift every code object in a .pyc and verdict each.
fn pyc_file(path: &str) -> i32 {
    let raw = match std::fs::read(path) {
        Ok(r) => r,
        Err(e) => { eprintln!("cannot read {}: {}", path, e); return 2; }
    };
    let objs = match ::vox::pyc::read_pyc(&raw) {
        Ok(o) => o,
        Err(e) => { eprintln!("{}: {}", path, e); return 3; }
    };
    println!("{}  cpython {}  {} code object(s)", path, ::vox::pyc_table::PY_VERSION, objs.len());
    let mut tally = [0usize; 4];
    for o in &objs {
        let w = ::vox::pyc::lift(&o.code);
        let v = vox::verdict(&w);
        match v { 'T'=>tally[0]+=1,'B'=>tally[1]+=1,'N'=>tally[2]+=1,_=>tally[3]+=1 }
        let f = vox::open_forks(&w);
        println!("  {:<28} {}  {}   surplus {:>3} exits {}", o.name, v, vox::glyphs(&w), f.surplus, f.exits);
    }
    println!("  verdicts  T {}   B {}   N {}   F {}", tally[0], tally[1], tally[2], tally[3]);
    0
}

fn selfread(path: &str) -> i32 {
    let raw = match std::fs::read(path) {
        Ok(r) => r,
        Err(e) => { eprintln!("cannot read {}: {}", path, e); return 2; }
    };
    let l = loader::load(&raw);
    let image = vox_decode::Image { segments: l.code.clone() };
    let mut seeds: Vec<u64> = l.symbols.values().copied().collect();
    seeds.push(l.entry);
    let w = vox_decode::walk(&image, l.entry, &seeds);

    let mut tally = [0usize; 4];
    let mut by_exit: std::collections::BTreeMap<i32, (usize, i64)> = std::collections::BTreeMap::new();
    let mut worst: Vec<(i32, u64, i32, i32)> = Vec::new();   // residual, addr, surplus, exits
    for (addr, f) in &w.functions {
        let word = vox::recompile_function(f);
        match vox::verdict(&word) { 'T'=>tally[0]+=1, 'B'=>tally[1]+=1, 'N'=>tally[2]+=1, _=>tally[3]+=1 }
        let o = vox::open_forks(&word);
        let e = by_exit.entry(o.exits.min(4)).or_insert((0, 0));
        e.0 += 1; e.1 += o.surplus as i64;
        if o.residual > 0 { worst.push((o.residual, *addr, o.surplus, o.exits)); }
    }
    worst.sort_by(|a, b| b.0.cmp(&a.0));

    println!("{}  {}  {} function(s) by descent", path, l.format, w.functions.len());
    println!("  verdicts  T {}   B {}   N {}   F {}", tally[0], tally[1], tally[2], tally[3]);
    println!("  F is zero when the decoder is in phase with the image.\n");
    println!("  open forks against exits — an early return is a fork that never rejoins:");
    println!("    {:>6}  {:>7}  {:>14}", "exits", "funcs", "mean surplus");
    for (k, (n, sum)) in &by_exit {
        println!("    {:>5}{}  {:>7}  {:>14.2}", k, if *k == 4 { "+" } else { " " }, n, *sum as f64 / *n as f64);
    }
    println!("\n  {} function(s) carry surplus the exits do not explain; deepest first:", worst.len());
    for (res, addr, sur, ex) in worst.iter().take(10) {
        println!("    0x{:<8x}  residual {:>3}   surplus {:>3}   exits {}", addr, res, sur, ex);
    }
    println!("\n  Ranking by raw surplus ranks by how many ways a function can return.");
    println!("  The residual is what is left once that is paid for.");
    0
}

fn audit_linear(path: &str, l: &loader::Loaded, bits: u8) -> i32 {
    let total: usize = l.code.iter().map(|(_, b)| b.len()).sum();
    println!("{}  {} {}  entry 0x{:x}  {} byte(s) of code", path, l.format, l.arch, l.entry, total);
    let mut tally = [0usize; 4]; let mut funcs = 0usize; let mut covered = 0usize;
    let mut b_findings: Vec<(u64, String)> = Vec::new();
    for (base, bytes) in &l.code {
        let mut pos = 0usize; let mut cur: Vec<x86::Insn> = Vec::new(); let mut fstart = *base;
        let mut flush = |cur: &mut Vec<x86::Insn>, fstart: u64, tally: &mut [usize;4], funcs: &mut usize, bf: &mut Vec<(u64,String)>| {
            if cur.is_empty() { return; }
            let word = alloc_word(cur);
            let v = vox::verdict(&word);
            match v { 'T'=>tally[0]+=1,'B'=>tally[1]+=1,'N'=>tally[2]+=1,_=>tally[3]+=1 }
            if v == 'B' && bf.len() < 40 { bf.push((fstart, vox::glyphs(&word))); }
            *funcs += 1; cur.clear();
        };
        while pos < bytes.len() {
            let addr = base + pos as u64;
            match x86::decode_mode(&bytes[pos..], addr, bits) {
                Some(d) if d.len > 0 => {
                    if cur.is_empty() { fstart = addr; }
                    let mn = d.mnemonic.clone(); covered += d.len; pos += d.len;
                    let term = mn.starts_with("ret") || matches!(mn.as_str(), "int3"|"ud2"|"hlt"|"jmp");
                    cur.push(d);
                    if term { flush(&mut cur, fstart, &mut tally, &mut funcs, &mut b_findings); }
                }
                _ => { flush(&mut cur, fstart, &mut tally, &mut funcs, &mut b_findings); pos += 1; }
            }
        }
        flush(&mut cur, fstart, &mut tally, &mut funcs, &mut b_findings);
    }
    println!("  {} function(s) by linear sweep, {}% decoded ({} of {} bytes)",
        funcs, (covered*100/total.max(1)).min(100), covered.min(total), total);
    println!("  verdicts  T {}   B {}   N {}   F {}", tally[0], tally[1], tally[2], tally[3]);
    for (a, w) in b_findings.iter().take(12) { println!("    0x{:x}  {}", a, w); }
    0
}

/// Lift a decoded run to a word. This goes through `vox::recompile_function`
/// so the merge pass runs: without it no ∋ is ever emitted, every fork reads as
/// dangling, and the lane cannot return T at all.
fn alloc_word(insns: &[x86::Insn]) -> Vec<char> {
    let lifted: Vec<vox::Instruction> = insns.iter().map(|i| vox::Instruction {
        address: i.addr,
        mnemonic: i.mnemonic.clone(),
        op_str: if let (true, Some(t)) = (i.ops.len() == 1, i.target) { format!("{:#x}", t) }
                else { i.ops.iter().map(|o| o.intel()).collect::<Vec<_>>().join(", ") },
    }).collect();
    vox::recompile_function(&lifted)
}
fn alloc_prefix() -> Vec<char> { vec!['⊢'] }

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
    if l.arch != "x86-64" && l.arch != "x86-32" {
        eprintln!("{}: {} {} code. V⊙x decodes x86; it will not misread another", path, l.format, l.arch);
        eprintln!("architecture and hand back a confident, wrong word.");
        return 3;
    }
    if l.arch == "x86-32" {
        return audit_linear(path, &l, 32);
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
    // recursive-descent functions: the trustworthy set
    let mut claimed: std::collections::BTreeSet<u64> = std::collections::BTreeSet::new();
    for (start, f) in &w.functions {
        for ins in f { claimed.insert(ins.address); }
        let word = vox::recompile_function(f);
        match vox::verdict(&word) {
            'T' => tally[0] += 1, 'B' => tally[1] += 1, 'N' => tally[2] += 1,
            _ => { tally[3] += 1; if illtyped.len() < 8 { illtyped.push((*start, vox::glyphs(&word))); } }
        }
    }
    let descended = w.functions.len();
    // fallback sweep: linear-decode what descent never reached, group into
    // functions of their own at each terminal, and verdict those too.
    let mut swept = 0usize; let mut swept_bytes = 0usize;
    for (base, bytes) in &image.segments {
        let mut pos = 0usize;
        let mut cur: Vec<x86::Insn> = Vec::new();
        let flush = |cur: &mut Vec<x86::Insn>, tally: &mut [usize;4], swept: &mut usize| {
            if cur.is_empty() { return; }
            let mut word = alloc_word(cur);
            match vox::verdict(&word) { 'T'=>tally[0]+=1,'B'=>tally[1]+=1,'N'=>tally[2]+=1,_=>tally[3]+=1 }
            *swept += 1; word.clear(); cur.clear();
        };
        while pos < bytes.len() {
            let addr = base + pos as u64;
            if claimed.contains(&addr) { flush(&mut cur, &mut tally, &mut swept); 
                // skip the claimed instruction
                if let Some(d) = x86::decode(&bytes[pos..], addr) { pos += d.len.max(1); } else { pos += 1; }
                continue;
            }
            match x86::decode(&bytes[pos..], addr) {
                Some(d) if d.len > 0 => {
                    let mn = d.mnemonic.clone(); swept_bytes += d.len; pos += d.len;
                    let terminal = mn.starts_with("ret") || matches!(mn.as_str(), "int3"|"ud2"|"hlt"|"jmp");
                    cur.push(d);
                    if terminal { flush(&mut cur, &mut tally, &mut swept); }
                }
                _ => { flush(&mut cur, &mut tally, &mut swept); pos += 1; }
            }
        }
        flush(&mut cur, &mut tally, &mut swept);
    }
    let total_cov = w.claimed_bytes + swept_bytes;
    println!();
    if swept > 0 {
        println!("  {} function(s) by descent, {} more by fallback sweep", descended, swept);
        println!("  covered {}% of the image ({} of {} bytes)",
            (total_cov*100/w.total_bytes.max(1)).min(100), total_cov.min(w.total_bytes), w.total_bytes);
    }
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


/// Read a file, or say which one and why, and leave without a backtrace.
///
/// `.expect("read")` printed `read: Os { code: 2 }` and a panic notice, naming
/// neither the path nor the verb. Fed a list of a few hundred binaries — which is
/// how this is actually used — that is a wall of identical panics with nothing in
/// them to act on, and the one path that was wrong stays hidden.
fn read_or_exit(path: &str) -> Vec<u8> {
    match std::fs::read(path) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("vox: cannot read {}: {}", path, e);
            std::process::exit(2);
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let code = match args.first().map(|s| s.as_str()) {
        None | Some("-h") | Some("--help") | Some("help") => { usage(); 0 }
        Some("--selftest") | Some("--self-test") | Some("selftest") | Some("self-test") => selftest(),
        Some("verdict") => {
            // A word given as an ARGUMENT is capped by the OS's argv limit, and a
            // lifted proof term runs to tens of thousands of glyphs — the words this
            // is most wanted for are exactly the ones that will not fit. `-` reads
            // the word from stdin instead, and prints only the verdict, so a sweep
            // can pipe thousands through without the shell in the way.
            if args.len() < 2 { eprintln!("vox verdict <glyph-word> | vox verdict - (word on stdin) | vox verdict --tsv <file>"); 1 }
            else if args[1] == "--tsv" {
                // A sweep hands over thousands of lifted words at once. Spawning a
                // process per word makes the shell the bottleneck and, worse, makes
                // it tempting to reimplement the verdict on the caller's side. One
                // process, one authority: `name<TAB>word` in, `name<TAB>verdict<TAB>len`
                // out, in input order. A blank word is not verdicted — it is
                // reported as `-`, since a word nobody lifted is not a word that
                // failed to close.
                if args.len() < 3 { eprintln!("vox verdict --tsv <file>"); 1 }
                else {
                    match std::fs::read_to_string(&args[2]) {
                        Err(e) => { eprintln!("vox verdict --tsv: {}: {}", args[2], e); 1 }
                        Ok(txt) => {
                            let mut counts = std::collections::BTreeMap::new();
                            for line in txt.lines() {
                                if line.trim().is_empty() { continue; }
                                let mut it = line.splitn(2, '\t');
                                let name = it.next().unwrap_or("");
                                let w = it.next().unwrap_or("").trim();
                                if w.is_empty() {
                                    println!("{}\t-\t0", name);
                                    *counts.entry('-').or_insert(0usize) += 1;
                                    continue;
                                }
                                let word: Vec<char> = w.chars().collect();
                                let v = vox::verdict(&word);
                                *counts.entry(v).or_insert(0usize) += 1;
                                println!("{}\t{}\t{}", name, v, word.len());
                            }
                            let total: usize = counts.values().sum();
                            let tally: Vec<String> = counts.iter().map(|(k, n)| format!("{} {}", k, n)).collect();
                            eprintln!("verdicted {} words: {}", total, tally.join("  "));
                            0
                        }
                    }
                }
            }
            else if args[1] == "-" {
                use std::io::Read;
                let mut buf = String::new();
                match std::io::stdin().read_to_string(&mut buf) {
                    Err(_) => { eprintln!("vox verdict -: could not read stdin"); 1 }
                    Ok(_) => {
                        let word: Vec<char> = buf.trim().chars().collect();
                        println!("verdict {}", vox::verdict(&word));
                        0
                    }
                }
            }
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
        Some("rna") | Some("--rna") => {
            if args.len() < 2 { eprintln!("vox rna <sequence>"); 1 } else { rna(&args[1..].join("")) }
        }
        Some("self") => {
            let me = std::env::current_exe().map(|p| p.display().to_string())
                .unwrap_or_else(|_| "vox".into());
            selfread(&me)
        }
        Some("pyc") | Some("py") => {
            if args.len() < 2 { eprintln!("vox pyc <file.pyc>"); 1 } else { pyc_file(&args[1]) }
        }
        Some("findings") => {
            if args.len()<2 { eprintln!("vox findings <file>"); return; }
            let raw = read_or_exit(&args[1]);
            let l = loader::load(&raw);
            let image = vox_decode::Image { segments: l.code };
            let mut seeds: Vec<u64> = l.symbols.values().copied().collect(); seeds.push(l.entry);
            let w = vox_decode::walk(&image, l.entry, &seeds);
            let mut claimed: std::collections::BTreeSet<u64> = std::collections::BTreeSet::new();
            let mut b: Vec<(u64,String)> = Vec::new();
            for (start,f) in &w.functions {
                for ins in f { claimed.insert(ins.address); }
                let word = vox::recompile_function(f);
                if vox::verdict(&word)=='B' { b.push((*start, vox::glyphs(&word))); }
            }
            // sweep
            for (base,bytes) in &image.segments {
                let mut pos=0usize; let mut cur:Vec<x86::Insn>=Vec::new(); let mut fstart=*base;
                let mut flush=|cur:&mut Vec<x86::Insn>, fstart:u64, b:&mut Vec<(u64,String)>| {
                    if cur.is_empty(){return;} let word=alloc_word(cur);
                    if vox::verdict(&word)=='B' { b.push((fstart, vox::glyphs(&word))); } cur.clear();
                };
                while pos<bytes.len() {
                    let addr=base+pos as u64;
                    if claimed.contains(&addr){ flush(&mut cur,fstart,&mut b); if let Some(d)=x86::decode(&bytes[pos..],addr){pos+=d.len.max(1);}else{pos+=1;} continue; }
                    match x86::decode(&bytes[pos..],addr){
                        Some(d) if d.len>0 => { if cur.is_empty(){fstart=addr;} let mn=d.mnemonic.clone(); pos+=d.len; let term=mn.starts_with("ret")||matches!(mn.as_str(),"int3"|"ud2"|"hlt"|"jmp"); cur.push(d); if term{flush(&mut cur,fstart,&mut b);} }
                        _ => { flush(&mut cur,fstart,&mut b); pos+=1; }
                    }
                }
                flush(&mut cur,fstart,&mut b);
            }
            println!("{} B-finding(s): a fork held open across a commit or return.", b.len());
            println!("Inspect each; B is a candidate shape, not a proof.");
            for (a,word) in &b { println!("  0x{:x}  {}", a, word); }
            std::process::exit(0);
        }
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
            let raw = read_or_exit(&file);
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
            let raw=read_or_exit(&args[1]); print!("{}", imasm_module::emit(&raw)); std::process::exit(0); }
        Some("word") | Some("words") => { if args.len()<2 { eprintln!("vox word <file>"); return; }
            let raw=read_or_exit(&args[1]); println!("{}", imasm_module::words(&raw)); std::process::exit(0); }
        Some("disasm") => {
            if args.len() < 2 { eprintln!("vox disasm <file> [symbol]"); return; }
            let raw = read_or_exit(&args[1]);
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
