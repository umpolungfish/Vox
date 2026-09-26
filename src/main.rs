Warning: truncated output (original token count: 18200)
Total output lines: 1365

//! V⊙x CLI — lift an x86-64 ELF to twelve-glyph IMASM words and verdict them.
//!
//! The native decoder and lifter carry no external crates, so this builds on
//! its own. Subcommands mirror the surface of the original vox.py.

use ::vox::vox;
mod circuit_cli;
use ::vox::vox_decode;
use ::vox::lanes;
use ::vox::genetic;
use ::vox::protein;
use ::vox::fold;
use ::vox::fold3d;
use ::vox::x86;
use ::vox::{imasm_module, imasm_vm, loader, safetensors, divisor_membrane};

fn usage() {
    eprintln!("V⊙x — control-flow closure auditor");
    eprintln!();
    eprintln!("  vox <file.so|.elf>        lift every function, tally verdicts");
    eprintln!("  vox lift <file>           same");
    eprintln!("  vox run <sym> --args a,b <file>   recompile and RUN a function");
    eprintln!("  vox imasm <file>          emit the executable IMASM module");
    eprintln!("  vox glyphs <module.imasm> <output.glyphs>   encode complete module as glyphs");
    eprintln!("  vox unglyphs <word.glyphs> <output.imasm>  restore exact executable module");
    eprintln!("  vox circuit <module.imasm> [--stdin | hex-mask[:feedback] ...]   prepare once, switch resident QFT gates");
    eprintln!("  vox word <file>           emit the structure word per function");
    eprintln!("  vox verdict <glyph-word>  verdict one word (T/B/N/F)");
    eprintln!("  vox morphism-factor <native-numeral-word>   factor entirely over IMASM tapes");
    eprintln!("  vox coprime <base> <N>   validate that a phase base is a unit modulo N");
    eprintln!("  vox extract-factor <factor-carrier-word>    passive ≡c extraction from an already factor-bearing trace");
    eprintln!("  vox construct-carrier <operator-word>        decompose factoring morphisms and EML frame transport");
    eprintln!("  vox factor-with <operator-word> <n-word>     factor N on a carrier built from the operator word");
    eprintln!("  vox factor-operator resolve|full <N>         the CL9NK moat resolver over folded tapes");
    eprintln!("  vox scout <N>                                read the shape of N and hand the factor");
    eprintln!("  vox factor <N>                               shape-routed full factorization");
    eprintln!("  vox factor-membrane <N|membrane-word> [--factors P Q]  decode and verify membrane numerals");
    eprintln!("  vox membrane tower <levels>                  build a complete bidirectional tower");
    eprintln!("  vox membrane bridge <N> <m>   coupled divisor-ring W_t trace over IMASM tapes");
    eprintln!("  vox pairs <glyph-word>    the pairing: every region, what it holds, what is left open");
    eprintln!("  vox verdict --tsv <file>   verdict name<TAB>word lines in bulk");
    eprintln!("  vox evm <hex>             lift EVM bytecode, verdict its closure");
    eprintln!("  vox wasm <hex>            lift a WASM function body, verdict it");
    eprintln!("  vox hex <hex>             lift raw machine-code hex, verdict its closure");
    eprintln!("  vox rna <seq> [--dialect mito]   lift a coding sequence, verdict the transcript");
    eprintln!("  vox aa <seq>              lift a protein (one-letter residues), verdict the fold");
    eprintln!("  vox fasta <file>          lift a protein from FASTA");
    eprintln!("  vox pdb <file>            lift a protein from a PDB (CA per residue)");
    eprintln!("  vox glyco <seq|file>      locate the glycosylation boundary interfaces");
    eprintln!("  vox compile <seq> [--code standard|mitochondrial] [--pdb <path>]");
    eprintln!("                            the full pipeline, both ends: RNA/DNA in gives a");
    eprintln!("                            compiled protein with real fold info (Chou-Fasman");
    eprintln!("                            secondary structure, heuristic tertiary contacts, a");
    eprintln!("                            real 3D backbone via B4-Ramachandran-NeRF); protein");
    eprintln!("                            in gives RNA/DNA back out (Frobenius-preferred codon");
    eprintln!("                            per residue, full degeneracy) with the same fold on");
    eprintln!("                            the input. Direction auto-detects from the alphabet.");
    eprintln!("                            --pdb writes a real PDB file, readable by vox pdb.");
    eprintln!("  vox self                  lift V⊙x's own image and read it back");
    eprintln!("  vox pyc <file.pyc>        lift every code object in a .pyc, verdict each");
    eprintln!("  vox safetensors <file>  lift a HuggingFace safetensors file, verdict each tensor");
    eprintln!("  vox classify <mn> [ops]   the glyph an instruction lifts to");
    eprintln!("  vox tables <file> <symbol>   shift a function one nibble, verdict a random");
    eprintln!("                            baseline of the same length, and flag any resync run");
    eprintln!("                            far longer than chance -- an embedded constant table");
    eprintln!("                            (witness sets, factor bases, curve parameters), located");
    eprintln!("                            purely from the binary and shown at its real alignment");
    eprintln!("  vox --selftest            planted open/closed forks");
    eprintln!();
    eprintln!("T closes · B holds a fork open across a terminal · N never forked ·");
    eprintln!("F is ill-typed (a ∋ with no ∈ to pair).");
}


/// A mode-aware linear-sweep audit: decode every executable byte at the given
/// width, split into functions at each terminal, verdict each. Used where
/// recursive descent's length decoder does not apply (32-bit x86).
/// Read a coding sequence as a word in the twelve and verdict it. The dialect
/// names the code table: the standard one, or a mitochondrial gene, where UGA
/// delivers tryptophan rather than terminating.
fn rna(seq: &str, dialect: &str) -> i32 {
    let t = if dialect.is_empty() {
        genetic::lift_rna(seq)
    } else {
        genetic::lift_rna_dialect(seq, dialect)
    };
    if t.word.is_empty() {
        eprintln!("no promoted codon in that sequence");
        return 1;
    }
    println!("{:<8}{:<6}{:<16}{}", "CODON", "AA", "AXIS", "GLYPH");
    for r in &t.reading {
        println!("{:<8}{:<6}{:<16}{}", r.codon, r.aa, r.axis, r.glyph);
    }
    println!();
    println!("dialect  {}", if dialect.is_empty() { "standard" } else { dialect });
    println!("codons   {} promoted of {} read", t.word.len(), t.reading.len());
    println!("frame    {} at offset {}", if t.implicit_frame { "no AUG, read from the start" } else { "AUG" }, t.start);
    println!("word     {}", vox::glyphs(&t.word));
    match t.stopped {
        Some(s) => println!("stop     {}", s),
        None => println!("stop     (none: the sequence ran out before a stop)"),
    }
    println!("verdict  {}", vox::verdict(&t.word));
    0
}

fn protein(seq: &str, source: &str) -> i32 {
    let t = genetic::lift_protein(seq);
    if t.word.is_empty() {
        eprintln!("no promoted residue in that sequence (the eight ground-layer amino acids are silent)");
        return 1;
    }
    println!("{:<8}{:<6}{:<16}{}", "POS", "AA", "AXIS", "GLYPH");
    for r in &t.reading {
        println!("{:<8}{:<6}{:<16}{}", r.codon, r.aa, r.axis, r.glyph);
    }
    println!();
    println!("source   {}", source);
    println!("residues {} promoted of the sequence", t.reading.len());
    println!("word     {}", vox::glyphs(&t.word));
    println!("verdict  {}", vox::verdict(&t.word));
    0
}

/// `vox compile <seq> [--code standard|mitochondrial] [--pdb <path>]`
///
/// One entry point, both directions of the same pipeline. RNA/DNA in:
/// `protein::translate_full` (every residue, not just the twelve promoted
/// ones `genetic::lift_rna` keeps), then fold (`fold::fold_sequence` for
/// secondary/tertiary structure, `fold3d` for a real 3D backbone). Protein
/// in: reverse-translate with the Frobenius-preferred codon per residue
/// (`protein::preferred_codon_for_aa`) and run the SAME fold on the input,
/// so the "vice versa" direction carries fold info too. Direction
/// auto-detects from the input alphabet: pure A/C/G/T/U reads as nucleic
/// acid, anything else as protein codes.
fn compile(args: &[String]) -> i32 {
    let mut dialect = String::new();
    let mut pdb_path: Option<String> = None;
    let mut seq_parts: Vec<String> = Vec::new();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--code" if i + 1 < args.len() => {
                dialect = if args[i + 1] == "mitochondrial" || args[i + 1] == "mito" {
                    String::from("mitochondrial")
                } else { String::new() };
                i += 2;
            }
            "--pdb" if i + 1 < args.len() => { pdb_path = Some(args[i + 1].clone()); i += 2; }
            other => { seq_parts.push(other.to_string()); i += 1; }
        }
    }
    let seq = seq_parts.join(" ");
    let dialect_name = if dialect.is_empty() { "standard" } else { "mitochondrial" };

    let compact: String = seq.chars().filter(|c| !c.is_whitespace() && *c != '-' && *c != ',').collect();
    let is_nucleic = !compact.is_empty()
        && compact.chars().all(|c| matches!(c.to_ascii_uppercase(), 'A' | 'C' | 'G' | 'T' | 'U'));

    fn report_fold(chain: &[&'static str], b4_path: &[char], pdb_path: Option<&str>) {
        let f = fold::fold_sequence(chain);
        let n_h = f.residues.iter().filter(|r| r.secondary == fold::SecondaryLabel::Helix).count();
        let n_s = f.residues.iter().filter(|r| r.secondary == fold::SecondaryLabel::Sheet).count();
        let n_c = f.residues.len() - n_h - n_s;
        println!();
        println!("fold     helix {}  sheet {}  coil {}   ({} contacts, SerpentRod invariant {})",
            n_h, n_s, n_c, f.contacts.len(), if f.frobenius_ok { "PASS" } else { "FAIL" });
        let steps = fold3d::rama_steps(b4_path);
        let backbone = fold3d::build_backbone(&steps);
        println!("backbone {} residues placed (B4-Ramachandran-NeRF)", backbone.len());
        if let Some(path) = pdb_path {
            let elements = fold3d::group_ss_elements(&steps);
            let winding = f.residues.iter().map(|r| r.winding_number).max().unwrap_or(0);
            let pdb = fold3d::write_pdb(chain, &backbone, &elements, f.frobenius_ok, winding, "COMPILED THROUGH VOX", 'A');
            match std::fs::write(path, pdb.as_bytes()) {
                Ok(()) => println!("pdb      written to {}", path),
                Err(e) => println!("pdb      could not write to {}: {}", path, e),
            }
        }
    }

    if is_nucleic {
        let t = protein::translate_full(&compact, &dialect);
        if t.protein.is_empty() {
            eprintln!("no protein translated from '{}'; needs an ATG/AUG start codon", seq);
            return 1;
        }
        let bytes: Vec<char> = t.mrna.chars().collect();
        let b4_path: Vec<char> = (0..t.protein.len()).map(|k| {
            let p = t.start + k * 3;
            bytes.get(p).and_then(|&c| genetic::nuc_b4(c)).unwrap_or('N')
        }).collect();

        println!("== vox compile: RNA/DNA -> protein ({}) ==", dialect_name);
        println!("input    {}", seq);
        println!("mrna     {}", t.mrna);
        println!("protein  {}", t.protein.join("-"));
        match t.stopped {
            Some(s) => println!("stop     {}", s),
            None => println!("stop     (none: the sequence ran out before a stop)"),
        }
        report_fold(&t.protein, &b4_path, pdb_path.as_deref());
        0
    } else {
        let chain = match protein::parse_chain(&seq) {
            Some(c) if !c.is_empty() => c,
            _ => {
                eprintln!("could not parse '{}' as protein or nucleic acid", seq);
                eprintln!("use 3-letter (Met-Ala) or 1-letter (MA) amino acid codes, or A/C/G/T/U");
                return 1;
            }
        };
        let reverse = match protein::reverse_translate_full(&chain, &dialect) {
            Some(r) => r,
            None => { eprintln!("no codon exists for some residue under the {} table", dialect_name); return 1; }
        };
        let dna = protein::reverse_transcribe(&reverse.mrna);
        let bytes: Vec<char> = reverse.mrna.chars().collect();
        let b4_path: Vec<char> = (0..chain.len()).map(|k| {
            genetic::nuc_b4(bytes[k * 3]).unwrap_or('N')
        }).collect();

        println!("== vox compile: protein -> RNA/DNA ({}) ==", dialect_name);
        println!("input    {}", chain.join("-"));
        println!("mrna     {}  (Frobenius-preferred codon per residue)", reverse.mrna);
        println!("dna      {}", dna);
        println!("degeneracy  {} total possible mRNA sequences (product of per-residue codon counts)",
            reverse.total_combinations);
        report_fold(&chain, &b4_path, pdb_path.as_deref());
        0
    }
}

fn glyco(seq: &str, source: &str) -> i32 {
    let t = genetic::lift_protein(seq);
    let sites = genetic::glyco_sites(seq);
    let n_linked: Vec<_> = sites.iter().filter(|s| s.kind == "N-linked").collect();
    let o_linked: Vec<_> = sites.iter().filter(|s| s.kind == "O-linked").collect();

    println!("source   {}", source);
    println!("peptide  {}   ({} promoted residues, the bulk)", vox::glyphs(&t.word), t.reading.len());
    println!("verdict  {}   (of the peptide backbone)", vox::verdict(&t.word));
    println!();
    println!("BOUNDARY INTERFACES — where a glycan meets the peptide:");
    println!("  N-linked sequons (Asn-X-[Ser/Thr], X≠Pro) — determinate:");
    if n_linked.is_empty() {
        println!("    none");
    } else {
        for s in &n_linked {
            println!("    pos {:<5} {}   anchor ∈ (recognition gate) opens the sequon", s.pos, s.motif);
        }
    }
    println!("  O-linked candidates (Ser/Thr) — admitted, not determinate, and");
    println!("  GROUND-LAYER so they carry no mark in the peptide word: {}", o_linked.len());
    println!();
    println!("  the glycan tree at each site is a branched word of its own; lifting it");
    println!("  needs the monosaccharides grounded through the chem pipeline, not here.");
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

/// Lift a raw hex value stream: decode the hex to bytes, load them as a code
/// artifact, walk the control flow, recompile each function to its glyph word,
/// and read the closure verdict of each. The delta half of the vox pair pointed
/// at inline hex, the same lift `vox self` runs on a file.
fn hexlift(hexstr: &str) -> i32 {
    let raw = lanes::from_hex(hexstr);
    if raw.is_empty() { eprintln!("vox hex: no hex bytes in input"); return 2; }
    let l = loader::load(&raw);
    let image = vox_decode::Image { segments: l.code.clone() };
    let mut seeds: Vec<u64> = l.symbols.values().copied().collect();
    seeds.push(l.entry);
    let mut w = vox_decode::walk(&image, l.entry, &seeds);
    vox_decode::mark_noreturn(&mut w.functions, &l.symbols);

    println!("HEX    {} bytes  {}  {}  {} function(s) by descent",
             raw.len(), l.format, l.arch, w.functions.len());
    if w.functions.is_empty() {
        eprintln!("  no function reachable from entry 0x{:x}", l.entry);
        return 1;
    }
    let mut tally = [0usize; 4];
    for (addr, f) in &w.functions {
        let word = vox::recompile_function(f);
        let v = vox::verdict(&word);
        match v { 'T'=>tally[0]+=1, 'B'=>tally[1]+=1, 'N'=>tally[2]+=1, _=>tally[3]+=1 }
        let mark = if v == 'B' { "   <-- FINDING (fork open across commit)" } else { "" };
        println!("  0x{:08x}  {}  {}{}", addr, v, vox::glyphs(&word), mark);
    }
    println!("  verdicts  T {}   B {}   N {}   F {}", tally[0], tally[1], tally[2], tally[3]);
    0
}

/// Linear sweep of `bytes` as x86-64, splitting into fragments at each
/// terminal instruction (ret/int3/ud2/hlt/jmp) or at the first undecodable
/// byte. Returns each fragment's (start_offset, length_in_bytes) within
/// `bytes` -- the same terminal predicate the fallback sweep in `lift_file`
/// uses, applied here to a standalone buffer rather than a whole image.
fn linear_fragments(bytes: &[u8]) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let mut pos = 0usize;
    let mut frag_start = 0usize;
    let mut frag_len = 0usize;
    while pos < bytes.len() {
        match x86::decode(&bytes[pos..], pos as u64) {
            Some(d) if d.len > 0 => {
                let terminal = d.mnemonic.starts_with("ret")
                    || matches!(d.mnemonic.as_str(), "int3" | "ud2" | "hlt" | "jmp");
                frag_len += d.len;
                pos += d.len;
                if terminal { out.push((frag_start, frag_len)); frag_start = pos; frag_len = 0; }
            }
            _ => {
                if frag_len > 0 { out.push((frag_start, frag_len)); }
                pos += 1; frag_start = pos; frag_len = 0;
            }
        }
    }
    if frag_len > 0 { out.push((frag_start, frag_len)); }
    out
}

/// Shift a byte buffer left by one nibble (4 bits): output byte i is built
/// from the low nibble of input byte i and the high nibble of input byte
/// i+1, so the same underlying bits decode from a half-byte-off starting
/// point -- real content, read out of alignment, not a different function.
fn nibble_shift(bytes: &[u8]) -> Vec<u8> {
    let mut out = vec![0u8; bytes.len()];
    for i in 0..bytes.len() {
        let hi = bytes[i] << 4;
        let lo = if i + 1 < bytes.len() { bytes[i + 1] >> 4 } else { 0 };
        out[i] = hi | lo;
    }
    out
}

/// xorshift64* -- a small deterministic PRNG so the random baseline is
/// reproducible from a fixed seed, with no external crate.
fn xorshift_bytes(state: &mut u64, n: usize) -> Vec<u8> {
    let mut out = Vec::with_capacity(n);
    while out.len() < n {
        *state ^= *state << 13;
        *state ^= *state >> 7;
        *state ^= *state << 17;
        for b in state.to_le_bytes() { if out.len() < n { out.push(b); } }
    }
    out
}

/// Locate embedded constant/data tables inside one named function by nibble-
/// phase resync anomaly: shift the function's real bytes by one nibble,
/// linear-sweep the result into fragments at each terminal instruction, and
/// compare the longest fragment against a random-byte baseline of the same
/// length. Ordinary code and pure noise both desync within a bounded range;
/// a run of near-identical instruction templates (a witness set, a factor
/// base, curve parameters) survives misalignment far longer than chance,
/// because its repetition is what the wrong phase accidentally resyncs
/// against. Flags the anomaly and prints the REAL, correctly aligned
/// instructions there, since the wrong-phase reading is a locator, not the
/// content.
fn find_tables(path: &str, sym: &str) -> i32 {
    let raw = match std::fs::read(path) {
        Ok(r) => r,
        Err(e) => { eprintln!("cannot read {}: {}", path, e); return 2; }
    };
    let l = loader::load(&raw);
    let mut addrs: Vec<(u64, String)> = l.symbols.iter().map(|(k, v)| (*v, k.clone())).collect();
    addrs.sort();
    // Symbol table names are Rust's raw mangled form (e.g. "_RNvNtCs...21is_prime_miller_rabin"),
    // which still carries the real identifier as a literal substring -- so match on
    // that instead of demangling, and prefer the shortest hit (the function itself,
    // not one of its closures or monomorphizations, which mangle longer).
    let candidates: Vec<usize> = addrs.iter().enumerate()
        .filter(|(_, (_, name))| name.contains(sym))
        .map(|(i, _)| i).collect();
    if candidates.is_empty() { eprintln!("vox tables: symbol '{}' not found", sym); return 1; }
    let idx = *candidates.iter().min_by_key(|&&i| addrs[i].1.len()).unwrap();
    if candidates.len() > 1 {
        eprintln!("  ({} symbols contain '{}', using the shortest: {})", candidates.len(), sym, addrs[idx].1);
    }
    let start = addrs[idx].0;
    let end = if idx + 1 < addrs.len() { addrs[idx + 1].0 } else { start + 0x10000 };
    let seg = match l.code.iter().find(|(base, bytes)| start >= *base && start < base + bytes.len() as u64) {
        Some(s) => s,
        None => { eprintln!("vox tables: 0x{:x} is not in any executable segment", start); return 1; }
    };
    let off = (start - seg.0) as usize;
    let len = ((end - start) as usize).min(seg.1.len().saturating_sub(off));
    if len == 0 { eprintln!("vox tables: empty range for '{}'", sym); return 1; }
    let bytes = &seg.1[off..off + len];

    println!("{}  0x{:x}..0x{:x}  {} bytes", sym, start, end, len);

    let shifted = nibble_shift(bytes);
    let real_frags = linear_fragments(&shifted);
    let (real_off, real_len) = real_frags.iter().copied().max_by_key(|&(_, flen)| flen).unwrap_or((0, 0));

    let trials = 20usize;
    let mut state: u64 = 0x9E3779B97F4A7C15 ^ (len as u64);
    let mut baseline: Vec<f64> = Vec::with_capacity(trials);
    for _ in 0..trials {
        let rnd = xorshift_bytes(&mut state, len);
        let m = linear_fragments(&rnd).iter().map(|&(_, flen)| flen).max().unwrap_or(0);
        baseline.push(m as f64);
    }
    let mean = baseline.iter().sum::<f64>() / trials as f64;
    let var = baseline.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / trials as f64;
    let sd = var.sqrt();
    let z = if sd > 0.0 { (real_len as f64 - mean) / sd } else if (real_len as f64) > mean { f64::INFINITY } else { 0.0 };

    println!("  phase-1 longest resync run: {} bytes at offset 0x{:x}  (random baseline, {} trials: mean {:.1}, sd {:.1})",
        real_len, real_off, trials, mean, sd);

    if z > 2.0 {
        println!("  FLAG: {:.1} standard deviations above the random baseline -- probable embedded constant/table", z);
        let show_end = (real_off + real_len + 16).min(bytes.len());
        // Walk forward from the function's real start so every printed line
        // lands on a genuine instruction boundary, rather than decoding raw
        // from `real_off` itself, which is a byte offset found in the SHIFTED
        // stream and is not guaranteed to be a boundary in the real one.
        println!("  real (correctly aligned) bytes near 0x{:x}:", start + real_off as u64);
        let mut pos = 0usize;
        let mut printing = false;
        while pos < show_end {
            match x86::decode(&bytes[pos..], start + pos as u64) {
                Some(d) if d.len > 0 => {
                    if !printing && pos + d.len > real_off.saturating_sub(8) { printing = true; }
                    if printing {
                        let op_str = if let (true, Some(t)) = (d.ops.len() == 1, d.target) { format!("0x{:x}", t) }
                            else { d.ops.iter().map(|o| o.intel()).collect::<Vec<_>>().join(", ") };
                        println!("    0x{:x}  {} {}", d.addr, d.mnemonic, op_str);
               …6200 tokens truncated…    match loaded {
                    Ok((seq, src)) => glyco(&seq, &src),
                    Err(e) => { eprintln!("vox glyco: {a}: {e}"); 1 }
                }
            }
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
            let mut w = vox_decode::walk(&image, l.entry, &seeds);
            vox_decode::mark_noreturn(&mut w.functions, &l.symbols);
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
                let flush=|cur:&mut Vec<x86::Insn>, fstart:u64, b:&mut Vec<(u64,String)>| {
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
        Some("circuit") => {
            if let Err(error) = circuit_cli::run(&args[1..]) {
                eprintln!("{error}");
                1
            } else { 0 }
        }
        Some("run") => {
            // vox run <file> [--argv a,b]            — run it: real process, real
            //                                           argv/envp/auxv stack, real
            //                                           syscalls, from its own entry.
            // vox run <symbol> <file> [--args 1,2]   — call one function directly:
            //                                           scalar int args in, one
            //                                           int back, no process at all.
            // A single bare token is the FILE, not the symbol — with no name given
            // there is no function to call, so the whole file runs as a process.
            // This is also the only thing a PE binary offers, since the loader
            // never populates a symbol table for PE at all (only ELF has one).
            let mut bare: Vec<String> = Vec::new();
            let mut argv_ints: Vec<i64> = Vec::new();
            let mut argv_strs: Vec<String> = Vec::new();
            let mut i=1;
            while i < args.len() {
                match args[i].as_str() {
                    "--args" => { i+=1; if i<args.len() { for a in args[i].split(',') { let a=a.trim(); if !a.is_empty() {
                        let v = if let Some(h)=a.strip_prefix("0x") { i64::from_str_radix(h,16).unwrap_or(0) } else { a.parse().unwrap_or(0) }; argv_ints.push(v);} } } }
                    "--argv" => { i+=1; if i<args.len() { for a in args[i].split(',') { argv_strs.push(a.to_string()); } } }
                    other => bare.push(other.to_string()),
                }
                i+=1;
            }
            let (sym, file): (String, String) = match bare.len() {
                0 => { eprintln!("vox run <file> [--argv a,b]   or   vox run <symbol> <file> [--args 1,2]"); return; }
                1 => (String::new(), bare[0].clone()),
                _ => (bare[0].clone(), bare[1..].join(" ")),
            };
            if file.is_empty() { eprintln!("vox run <file> [--argv a,b]   or   vox run <symbol> <file> [--args 1,2]"); return; }
            // A `.imasm` file is a saved module: text, already carrying its own
            // symbol table (`; sym NAME 0xADDR`), so it runs directly with no
            // second read of the original binary. Anything else is read as raw
            // bytes and lifted fresh, same as before.
            let raw = read_or_exit(&file);
            let is_glyph = file.ends_with(".glyphs") || raw.starts_with(::vox::glyph_module::PREFIX.as_bytes());
            let is_module = is_glyph || file.ends_with(".imasm")
                || raw.starts_with(b"; ");
            let mut m = if is_module {
                let text = match String::from_utf8(raw) {
                    Ok(t) => t,
                    Err(e) => { eprintln!("cannot read {}: {}", file, e); std::process::exit(1); }
                };
                let text = if is_glyph {
                    match ::vox::glyph_module::decode(&text) {
                        Ok(module) => module,
                        Err(error) => { eprintln!("invalid glyph module: {error}"); std::process::exit(2); }
                    }
                } else { text };
                imasm_vm::Machine::new(&text)
            } else {
                imasm_vm::Machine::new(&imasm_module::emit(&raw))
            };
            m.set_host(Box::new(StdHost::new()));
            let trace = std::env::var("VOX_TRACE").is_ok();
            if trace { m.trace_allocs(); }
            if let Ok(w) = std::env::var("VOX_WMEM") {
                m.wmem = u64::from_str_radix(w.trim().trim_start_matches("0x"), 16).unwrap_or(0);
            }
            if let Ok(w) = std::env::var("VOX_WMEM_RANGE") {
                let p: Vec<&str> = w.split(',').collect();
                if p.len() == 2 {
                    m.wmem_lo = u64::from_str_radix(p[0].trim().trim_start_matches("0x"), 16).unwrap_or(0);
                    m.wmem_hi = u64::from_str_radix(p[1].trim().trim_start_matches("0x"), 16).unwrap_or(0);
                }
            }
            if let Ok(r) = std::env::var("VOX_RANGE") {
                let p: Vec<&str> = r.split(',').collect();
                if p.len() == 2 {
                    m.trace_lo = u64::from_str_radix(p[0].trim_start_matches("0x"), 16).unwrap_or(0);
                    m.trace_hi = u64::from_str_radix(p[1].trim_start_matches("0x"), 16).unwrap_or(0);
                }
            }
            if sym.is_empty() {
                let mut argv = vec![file.clone()];
                argv.extend(argv_strs);
                // Step budget: 5e9 default, VOX_STEPS overrides for long
                // membrane runs (the 21-digit lift needs >5e9 at the
                // measured 954k steps/s VM rate).
                let steps: u64 = std::env::var("VOX_STEPS").ok()
                    .and_then(|s| s.trim().parse().ok())
                    .unwrap_or(5_000_000_000);
                let r = m.run_process(&argv, &[], steps);
                if let Err(imasm_vm::Stop::Halt(_)) = &r {
                    eprint!("regs at halt:");
                    for rn in ["rax","rbx","rcx","rdx","rsi","rdi","rbp","rsp","r12","r13","r14","r15"] {
                        eprint!(" {}={:x}", rn, m.reg(rn) as u64);
                    }
                    eprintln!();
                    // Walk saved rbp frames: each frame is [saved rbp][return addr].
                    let mut fp = m.reg("rbp") as u64;
                    eprint!("call chain:");
                    for _ in 0..12 {
                        if fp == 0 || fp < 0x1000 { break; }
                        let ret = { let b = m.peek(fp + 8, 8); (0..8).fold(0u64, |a,i| a | ((b[i] as u64) << (8*i))) };
                        let nfp = { let b = m.peek(fp, 8); (0..8).fold(0u64, |a,i| a | ((b[i] as u64) << (8*i))) };
                        eprint!(" {:x}", ret);
                        if nfp <= fp { break; }
                        fp = nfp;
                    }
                    eprintln!();
                }
                if trace || m.wmem != 0 { for line in &m.syslog { eprintln!("{}", line); } }
                if trace {
                    for rn in ["rax","rbx","rcx","rdx","rsi","rdi","rbp"] {
                        let a = m.reg(rn) as u64;
                        eprintln!("{} = {:x}  mem[{:x}..] = {:02x?}", rn, a, a, m.peek(a.wrapping_sub(4), 16));
                    }
                }
                match r {
                    Ok(()) => println!("entry(...) ran off the end with no exit call   [{} steps]", m.steps),
                    Err(imasm_vm::Stop::SysExit(c)) => println!("entry(...) exited({})   [{} steps in the twelve]", c, m.steps),
                    Err(imasm_vm::Stop::Halt(e)) => println!("entry(...) halted: {}   [{} steps]", e, m.steps),
                }
            } else {
                let addr = match m.resolve(&sym) {
                    Some(a) => a,
                    None => { eprintln!("no symbol '{}' in {}", sym, file); std::process::exit(1); }
                };
                if let Err(e) = m.initialize_relocations() {
                    match e {
                        imasm_vm::Stop::SysExit(c) => println!("{}(...) called exit({}) during relocation setup   [{} steps]", sym, c, m.steps),
                        imasm_vm::Stop::Halt(e) => println!("{}(...) halted during relocation setup: {}   [{} steps]", sym, e, m.steps),
                    }
                    std::process::exit(0);
                }
                match m.call(addr, &argv_ints, 50_000_000) {
                    Ok(r) => println!("{}({}) = {}   [{} steps in the twelve]", sym, argv_ints.iter().map(|a|a.to_string()).collect::<Vec<_>>().join(", "), r, m.steps),
                    Err(imasm_vm::Stop::SysExit(c)) => println!("{}(...) called exit({})   [{} steps in the twelve]", sym, c, m.steps),
                    Err(imasm_vm::Stop::Halt(e)) => println!("{}(...) halted: {}   [{} steps]", sym, e, m.steps),
                }
            }
            std::process::exit(0);
        }
        Some("glyphs") | Some("unglyphs") => {
            use std::io::Write;
            if args.len()!=3 { eprintln!("vox glyphs|unglyphs <input> <new-output>"); std::process::exit(2); }
            let source = std::fs::read_to_string(&args[1]).unwrap_or_else(|e| {
                eprintln!("cannot read {}: {e}",args[1]); std::process::exit(1);
            });
            let result = if args[0]=="glyphs" { ::vox::glyph_module::encode(&source) }
                         else { ::vox::glyph_module::decode(&source) };
            let result = result.unwrap_or_else(|e| { eprintln!("{e}"); std::process::exit(2); });
            let mut output = std::fs::OpenOptions::new().write(true).create_new(true).open(&args[2])
                .unwrap_or_else(|e| { eprintln!("cannot create {}: {e}",args[2]); std::process::exit(1); });
            output.write_all(result.as_bytes()).unwrap_or_else(|e| {
                eprintln!("cannot write {}: {e}",args[2]); std::process::exit(1);
            });
            println!("saved {} ({} glyphs/characters)",args[2],result.chars().count());
            std::process::exit(0);
        }
        Some("imasm") => { if args.len()<2 { eprintln!("vox imasm <file>"); return; }
            let raw=read_or_exit(&args[1]);
            let module = imasm_module::emit(&raw);
            print!("{}", module);
            let out_path = format!("{}.imasm", args[1]);
            match std::fs::write(&out_path, &module) {
                Ok(()) => eprintln!("saved {}", out_path),
                Err(e) => eprintln!("could not save {}: {}", out_path, e),
            }
            std::process::exit(0); }
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
        Some("morphism-factor") => {
            if args.len() != 2 { eprintln!("vox morphism-factor <native-numeral-word>"); 1 }
            else { match ::vox::morphism_factor::factor(&args[1]) { Ok(w) => { println!("{}", w); 0 }, Err(e) => { eprintln!("{}", e); 2 } } }
        }
        Some("extract-factor") => {
            if args.len() != 2 {
                eprintln!("vox extract-factor <factor-carrier-word>");
                1
            } else {
                let word: Vec<char> = args[1].chars().collect();
                match ::vox::factor_extract::extract_word(&word) {
                    Ok(readout) => {
                        println!("{} x {}", readout.p, readout.q);
                        println!("normal-form {}", readout.normal_form.iter().collect::<String>());
                        println!("transforms {}", readout.transforms);
                        0
                    }
                    Err(e) => { eprintln!("{}", e); 2 }
                }
            }
        }
        Some("verify") => {
            if args.len() != 4 { eprintln!("vox verify <p-word> <q-word> <n-word>"); 1 }
            else { match ::vox::morphism_factor::verify(&args[1], &args[2], &args[3]) { Ok(w) => { println!("{}", w); 0 }, Err(e) => { eprintln!("{}", e); 2 } } }
        }
        Some("sieve") => {
            let parsed: Option<Vec<char>> = if args.len() != 2 {
                None
            } else if args[1].starts_with('⊢') {
                ::vox::morphism_factor::parse_numeral(&args[1]).ok()
            } else {
                ::vox::morphism_factor::decimal_to_tape(&args[1])
            };
            match parsed {
                Some(n) => { println!("{}", ::vox::sieve::repl_sieve(&n)); 0 }
                None => { eprintln!("vox sieve <N-word|decimal>   Dixon/QS for the HARD shape"); 1 }
            }
        }
        Some("factor") => {
            let parsed: Option<Vec<char>> = if args.len() != 2 {
                None
            } else if args[1].starts_with('⊢') {
                ::vox::morphism_factor::parse_numeral(&args[1]).ok()
            } else {
                ::vox::morphism_factor::decimal_to_tape(&args[1])
            };
            match parsed {
                Some(n) => { println!("{}", ::vox::morphism_factor::repl_smart_factor(&n)); 0 }
                None => { eprintln!("vox factor <N-word|decimal>   shape-routed full factorization"); 1 }
            }
        }
        Some("factor-membrane") => match ::vox::membrane_factor::command(&args[1..]) {
            Ok(report) => { print!("{report}"); 0 }
            Err(error) => { eprintln!("factor-membrane: {error}"); 2 }
        },
        Some("perfect") => {
            // Build the depth-n perfect membrane and read its closure with the
            // auditor: T means mu∘delta = id holds by the matched circuitry.
            let n = args.get(1).and_then(|a| a.parse::<usize>().ok()).unwrap_or(2);
            let (v, surplus, w) = ::vox::perfect_membrane::report(n);
            println!("depth {n}: verdict {v}  fork/fuse surplus {surplus}");
            println!("{}", ::vox::vox::glyphs(&w));
            0
        }
        Some("operculum") => {
            // Walk the vessel lifecycle: open, deposit, seal, run, extract.
            let n = args.get(2).and_then(|a| a.parse::<usize>().ok()).unwrap_or(3);
            match args.get(1).and_then(|a| ::vox::morphism_factor::decimal_to_tape(a)) {
                Some(v) => { print!("{}", ::vox::perfect_membrane::operculum_demo(&v, n)); 0 }
                None => { eprintln!("vox operculum <decimal> [depth]   load a value through the membrane's one lid"); 1 }
            }
        }
        Some("coprime") => {
            let parse = |value: &str| {
                if value.starts_with('⊢') {
                    ::vox::morphism_factor::parse_numeral(value).ok()
                } else {
                    ::vox::morphism_factor::decimal_to_tape(value)
                }
            };
            match (args.get(1).and_then(|value| parse(value)), args.get(2).and_then(|value| parse(value))) {
                (Some(base), Some(modulus)) if ::vox::morphism_factor::gcd(base.clone(), modulus.clone()) == ::vox::morphism_factor::tape_u64(1) => {
                    println!("coprime"); 0
                }
                (Some(_), Some(_)) => { eprintln!("phase base is not coprime to N"); 2 }
                _ => { eprintln!("vox coprime <base> <N>   validate a phase base before baking"); 2 }
            }
        }
        Some("numeral") => {
            // Encode a decimal to its IMASM numeral word. This is the
            // pre-compilation step: the decimal is consumed here, and the word it
            // prints is baked into a program so the run itself takes no input.
            match args.get(1).and_then(|a| ::vox::morphism_factor::decimal_to_tape(a)) {
                Some(n) => { println!("{}", ::vox::morphism_factor::emit_numeral(&n)); 0 }
                None => { eprintln!("vox numeral <decimal>   emit the IMASM numeral word for N"); 1 }
            }
        }
        Some("mpqs") => {
            let parsed: Option<Vec<char>> = if args.len() < 2 {
                None
            } else {
                ::vox::morphism_factor::decimal_to_tape(&args[1])
            };
            match parsed {
                Some(n) => {
                    let (bound0, _m) = ::vox::sieve::sieve_params(&n);
                    let mh = args.get(2).and_then(|a| a.parse::<usize>().ok()).unwrap_or(32_768);
                    let bound = args.get(3).and_then(|a| a.parse::<usize>().ok()).unwrap_or(bound0);
                    match ::vox::sieve::mpqs(&n, bound, mh, 32) {
                        Some(f) => { println!("{} factor {}", ::vox::morphism_factor::dec_of(&n), ::vox::morphism_factor::dec_of(&f)); 0 }
                        None => { println!("mpqs: no factor (relations short of a dependency)"); 0 }
                    }
                }
                None => { eprintln!("vox mpqs <decimal N>   multiple-polynomial quadratic sieve arm, in isolation"); 1 }
            }
        }
        Some("scout") => {
            let parsed: Option<Vec<char>> = if args.len() != 2 {
                None
            } else if args[1].starts_with('⊢') {
                ::vox::morphism_factor::parse_numeral(&args[1]).ok()
            } else {
                ::vox::morphism_factor::decimal_to_tape(&args[1])
            };
            match parsed {
                Some(n) => { println!("{}", ::vox::morphism_factor::repl_scout(&n)); 0 }
                None => { eprintln!("vox scout <N-word|decimal>   read the shape of N and hand the factor"); 1 }
            }
        }
        Some("factor-operator") => {
            let rest: Vec<&str> = args[1..].iter().map(|x| x.as_str()).collect();
            println!("{}", ::vox::factor_operator::repl_factor_operator(&rest)); 0
        }
        Some("construct-carrier") => {
            if args.len() != 2 { eprintln!("vox construct-carrier <operator-word>   decompose factoring morphisms and EML frame transport"); 1 }
            else { match ::vox::morphism_factor::construct_carrier(&args[1]) {
                Ok(tower) => {
                    let names: Vec<&str> = tower.iter().map(|t| ::vox::morphism_factor::morphism_name(t)).collect();
                    println!("tower ({} morphisms): {}", names.len(), names.join(" -> "));
                    0
                }
                Err(e) => { eprintln!("{}", e); 2 }
            } }
        }
        Some("factor-with") => {
            if args.len() != 3 { eprintln!("vox factor-with <operator-word> <n-word>   factor N on a carrier built from the operator word"); 1 }
            else { match ::vox::morphism_factor::factor_with(&args[1], &args[2]) { Ok(w) => { println!("{}", w); 0 }, Err(e) => { eprintln!("{}", e); 2 } } }
        }
        Some("membrane") | Some("divisor-membrane") | Some("divisor_membrane") => {
            if args.get(1).map(String::as_str) == Some("tower") {
                let levels = args.get(2).and_then(|s| s.parse::<usize>().ok());
                match levels.and_then(|n| ::vox::complete_membrane::CompleteMembrane::new(n).ok()) {
                    Some(m) => {
                        println!("tower levels={} forward_sidearms={} reverse_sidearms={} relation=μ∘δ=id",
                            m.levels(), m.forward_pairs().len(), m.reverse_pairs().len());
                        0
                    }
                    None => { eprintln!("vox membrane tower <levels>   levels must be greater than two"); 1 }
                }
            } else {
            let rest: Vec<&str> = args[1..].iter().map(|x| x.as_str()).collect();
            println!("{}", divisor_membrane::repl_divisor_membrane(&rest)); 0
            }
        }
        Some("safetensors") | Some("safetensor") => {
            if args.len() < 2 { eprintln!("vox safetensors <file.safetensors>"); 1 }
            else {
                let raw = read_or_exit(&args[1]);
                match safetensors::lift_file(&raw) {
                    Ok(words) => {
                        for (name, w) in &words {
                            let v = vox::verdict(w);
                            println!("{:<20} {}  {}", name, v, vox::glyphs(w));
                        }
                        0
                    }
                    Err(e) => { eprintln!("safetensors: {}", e); 1 }
                }
            }
        }
        Some(flag) if flag.starts_with('-') => { eprintln!("vox: unknown option {}\n", flag); usage(); 2 }
        Some(path) => lift_file(path),
    };
    std::process::exit(code);
}
