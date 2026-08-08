//! The module format: a word in the twelve, and the executable payload each
//! glyph carries. Ported from imasm_module.py. The glyph is the opcode (which of
//! the twelve axes the instruction is); the payload is normalised operands the
//! machine reads without ever parsing assembly.
//!
//!     GLYPH \t field \t field ...      with  r:reg  i:imm  m:base:index:scale:disp:size

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::format;

use crate::x86::{self, Op};
use crate::vox_decode;

const ENTRY: char = '⊢'; const TERM: char = '⊣'; const SPLIT: char = '∈'; const FUSE: char = '∋';
const CALL: char = '>'; const XFER: char = '<'; const INDIRECT: char = '⊙'; const COMMIT: char = '◻';
const LINK: char = '⋈'; const TRUTH: char = '⊤'; const CONSUME: char = '⊥'; const ENGAGE: char = '⊞';

fn is_move(mn: &str) -> bool {
    matches!(mn, "mov"|"movzx"|"movsx"|"movsxd"|"movabs"|"push"|"pop"|"xchg"|"leave")
}
fn is_truth(mn: &str) -> bool { matches!(mn, "cmp"|"test") }
fn is_terminal(mn: &str) -> bool { mn.starts_with("ret") || matches!(mn, "int3"|"ud2"|"hlt"|"iret"|"retf") }

/// The glyph — the same decision vox's classifier makes.
fn classify(i: &x86::Insn) -> char {
    let mn = i.mnemonic.as_str();
    if is_terminal(mn) { return TERM; }
    if mn == "call" { return if i.target.is_some() { CALL } else { INDIRECT }; }
    if mn == "jmp"  { return if i.target.is_some() { XFER } else { INDIRECT }; }
    if mn == "syscall" || mn == "sysenter" || mn == "int" { return INDIRECT; }
    if mn.starts_with('j') && mn != "jmp" { return SPLIT; }
    if mn.starts_with("set") { return CONSUME; }
    if mn.starts_with("cmov") { return CONSUME; }
    if is_truth(mn) { return TRUTH; }
    if i.writes_mem && !is_move(mn) { return COMMIT; }
    if is_move(mn) { return LINK; }
    ENGAGE
}

fn fields(ops: &[Op]) -> Vec<String> { ops.iter().map(|o| o.field()).collect() }

/// One instruction → its module lines. A merge emits a bare ∋ first.
fn encode(i: &x86::Insn, is_merge: bool) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    if is_merge { lines.push(FUSE.to_string()); }
    let mn = i.mnemonic.as_str();
    let g = classify(i);
    let f = fields(&i.ops);
    let join = |parts: &[String]| parts.join("\t");

    if g == TERM {
        lines.push(format!("{}\t{}", TERM, mn));
    } else if g == INDIRECT && matches!(mn, "syscall"|"sysenter"|"int") {
        lines.push(format!("{}\tsyscall", INDIRECT));
    } else if mn == "call" || mn == "jmp" {
        let mut parts = vec![g.to_string(), mn.to_string()]; parts.extend(f);
        lines.push(join(&parts));
    } else if mn.starts_with('j') {
        let mut parts = vec![SPLIT.to_string(), mn[1..].to_string()]; parts.extend(f);
        lines.push(join(&parts));
    } else if mn.starts_with("set") {
        let mut parts = vec![CONSUME.to_string(), mn[3..].to_string(), "set".to_string()]; parts.extend(f);
        lines.push(join(&parts));
    } else if mn.starts_with("cmov") {
        let mut parts = vec![CONSUME.to_string(), mn[4..].to_string(), "cmov".to_string()]; parts.extend(f);
        lines.push(join(&parts));
    } else {
        let glyph = if g == TRUTH { TRUTH } else if g == COMMIT { COMMIT } else if g == LINK { LINK } else { ENGAGE };
        let mut parts = vec![glyph.to_string(), mn.to_string()]; parts.extend(f);
        lines.push(join(&parts));
    }
    lines
}

/// Merge points of one function: addresses with ≥2 predecessors.
fn merges_of(insns: &[x86::Insn]) -> BTreeSet<u64> {
    let aset: BTreeSet<u64> = insns.iter().map(|i| i.addr).collect();
    let mut succ: BTreeMap<u64, u32> = BTreeMap::new();
    for (idx, i) in insns.iter().enumerate() {
        let mn = i.mnemonic.as_str();
        let falls = !(mn == "jmp" || mn.starts_with("ret"));
        if falls && idx + 1 < insns.len() { *succ.entry(insns[idx+1].addr).or_insert(0) += 1; }
        if mn.starts_with('j') {
            if let Some(t) = i.target { if aset.contains(&t) { *succ.entry(t).or_insert(0) += 1; } }
        }
    }
    succ.into_iter().filter(|&(_, c)| c >= 2).map(|(a, _)| a).collect()
}

/// ELF data sections the code reads: ALLOC, PROGBITS, not executable.
fn data_sections(raw: &[u8]) -> Vec<(u64, Vec<u8>)> {
    let mut out = Vec::new();
    if raw.len() < 64 || &raw[0..4] != b"\x7fELF" { return out; }
    let is64 = raw[4] == 2;
    if !is64 { return out; }
    let rd = |o: usize, n: usize| -> u64 { let mut v=0u64; for k in 0..n { v |= (raw[o+k] as u64) << (8*k); } v };
    let e_shoff = rd(0x28, 8) as usize;
    let e_shentsize = rd(0x3a, 2) as usize;
    let e_shnum = rd(0x3c, 2) as usize;
    for k in 0..e_shnum {
        let o = e_shoff + k * e_shentsize;
        if o + 64 > raw.len() { break; }
        let sh_type = rd(o + 4, 4);
        let sh_flags = rd(o + 8, 8);
        let sh_addr = rd(o + 16, 8);
        let sh_off = rd(o + 24, 8) as usize;
        let sh_size = rd(o + 32, 8) as usize;
        // SHT_PROGBITS(1), SHF_ALLOC(0x2), not SHF_EXECINSTR(0x4)
        if sh_type == 1 && (sh_flags & 0x2) != 0 && (sh_flags & 0x4) == 0 && sh_size > 0 {
            if sh_off + sh_size <= raw.len() {
                out.push((sh_addr, raw[sh_off..sh_off+sh_size].to_vec()));
            }
        }
    }
    out
}

/// The whole binary as an executable IMASM module. Every executable byte is
/// decoded linearly, so an address reached only through an indirect jump — a
/// switch's jump-table arm, a call through a function pointer — is in the module
/// too, not just what recursive descent reached from direct edges.
pub fn emit(raw: &[u8]) -> String {
    let (entry, segments) = crate::vox::parse_elf(raw);

    let mut out: Vec<String> = Vec::new();
    out.push(format!("; {} module", INDIRECT));
    out.push(format!("; entry 0x{:x}", entry));
    for (at, blob) in data_sections(raw) {
        let hex: String = blob.iter().map(|b| format!("{:02x}", b)).collect();
        out.push(format!("={:#x}\t{}", at, hex));
    }
    for (base, bytes) in &segments {
        let mut pos = 0usize;
        while pos < bytes.len() {
            let addr = base + pos as u64;
            match x86::decode(&bytes[pos..], addr) {
                Some(d) if d.len > 0 => {
                    out.push(format!("@0x{:x}", addr));
                    for line in encode(&d, false) { out.push(line); }
                    pos += d.len;
                }
                _ => pos += 1, // alignment padding or data between functions
            }
        }
    }
    let mut s = out.join("\n"); s.push('\n'); s
}

/// Just the structure word (glyphs), per function.
pub fn words(raw: &[u8]) -> String {
    let (entry, segments) = crate::vox::parse_elf(raw);
    let image = vox_decode::Image { segments };
    let seeds = crate::vox::elf_function_symbols(raw);
    let w = vox_decode::walk(&image, entry, &seeds);
    let mut out: Vec<String> = Vec::new();
    for (start, f) in &w.functions {
        let mut insns: Vec<x86::Insn> = Vec::new();
        for ins in f {
            if let Some(bytes) = image.bytes_at(ins.address) {
                if let Some(d) = x86::decode(bytes, ins.address) { insns.push(d); }
            }
        }
        if insns.is_empty() { continue; }
        let merges = merges_of(&insns);
        let mut word = String::from("⊢");
        for i in &insns {
            if merges.contains(&i.addr) { word.push('∋'); }
            word.push(classify(i));
        }
        out.push(format!("0x{:x}\t{}", start, word));
    }
    out.join("\n")
}

/// Function symbol names → address, from .symtab or .dynsym.
pub fn symbols(raw: &[u8]) -> BTreeMap<String, u64> {
    let mut out = BTreeMap::new();
    if raw.len() < 64 || &raw[0..4] != b"\x7fELF" || raw[4] != 2 { return out; }
    let rd = |o: usize, n: usize| -> u64 { let mut v=0u64; for k in 0..n { if o+k<raw.len(){ v |= (raw[o+k] as u64) << (8*k);} } v };
    let e_shoff = rd(0x28,8) as usize; let e_shentsize = rd(0x3a,2) as usize; let e_shnum = rd(0x3c,2) as usize;
    let sh = |k: usize, field: usize, n: usize| rd(e_shoff + k*e_shentsize + field, n);
    for k in 0..e_shnum {
        let sh_type = sh(k, 4, 4);
        if sh_type != 2 && sh_type != 11 { continue; } // SYMTAB or DYNSYM
        let sym_off = sh(k, 24, 8) as usize; let sym_size = sh(k, 32, 8) as usize;
        let strtab_idx = sh(k, 40, 4) as usize; // sh_link
        let str_off = sh(strtab_idx, 24, 8) as usize;
        let ent = sh(k, 56, 8).max(24) as usize;
        let mut o = sym_off;
        while o + 24 <= sym_off + sym_size && o + 24 <= raw.len() {
            let st_name = rd(o, 4) as usize;
            let st_info = raw.get(o+4).copied().unwrap_or(0);
            let st_value = rd(o+8, 8);
            if (st_info & 0xf) == 2 && st_value != 0 { // STT_FUNC
                let mut name = String::new();
                let mut p = str_off + st_name;
                while let Some(&b) = raw.get(p) { if b == 0 { break; } name.push(b as char); p += 1; }
                if !name.is_empty() { out.insert(name, st_value); }
            }
            o += ent;
        }
    }
    out
}
