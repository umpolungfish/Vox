//! The module format: a word in the twelve, and the executable payload each
//! glyph carries. Ported from imasm_module.py. The glyph is the opcode (which of
//! the twelve axes the instruction is); the payload is normalised operands the
//! machine reads without ever parsing assembly.
//!
//! ```text
//! GLYPH \t field \t field ...      with  r:reg  i:imm  m:base:index:scale:disp:size
//! ```

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use alloc::format;

use crate::x86::{self, Op};
use crate::vox_decode;
use crate::loader;

fn bits_of(arch: &str) -> u8 { if arch == "x86-32" { 32 } else { 64 } }

const TERM: char = '⊣'; const SPLIT: char = '∈'; const FUSE: char = '∋';
const CALL: char = '≻'; const XFER: char = '≺'; const INDIRECT: char = '⊙'; const COMMIT: char = '⊡';
const LINK: char = '⋈'; const TRUTH: char = '⊤'; const CONSUME: char = '⊥'; const ENGAGE: char = '⊞';

fn is_move(mn: &str) -> bool {
    matches!(mn, "mov"|"movzx"|"movsx"|"movsxd"|"movabs"|"push"|"pop"|"xchg"|"leave")
}
fn is_truth(mn: &str) -> bool { matches!(mn, "cmp"|"test") }
fn is_terminal(mn: &str) -> bool { mn.starts_with("ret") || matches!(mn, "int3"|"ud2"|"hlt"|"iret"|"retf") }

/// The glyph — the same decision vox's classifier makes.
pub fn classify(i: &x86::Insn) -> char {
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
pub(crate) fn encode(i: &x86::Insn, is_merge: bool) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    if is_merge { lines.push(FUSE.to_string()); }
    let mn = i.mnemonic.as_str();
    let g = classify(i);
    let f = fields(&i.ops);
    let join = |parts: &[String]| parts.join("\t");

    if g == TERM {
        // keep ret's imm16 (stdcall stack cleanup): ⊣  ret  i:0x4
        if mn == "ret" && !f.is_empty() { let mut parts=vec![TERM.to_string(), mn.to_string()]; parts.extend(f); lines.push(join(&parts)); }
        else { lines.push(format!("{}\t{}", TERM, mn)); }
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

/// The whole binary as an executable IMASM module. Every executable byte is
/// decoded linearly, so an address reached only through an indirect jump — a
/// switch's jump-table arm, a call through a function pointer — is in the module
/// too, not just what recursive descent reached from direct edges.
pub fn emit(raw: &[u8]) -> String {
    let l = loader::load(raw);

    let bits = bits_of(l.arch);
    let mut out: Vec<String> = Vec::new();
    out.push(format!("; {} module ({} {})", INDIRECT, l.format, l.arch));
    out.push(format!("; entry 0x{:x}", l.entry));
    out.push(format!("; bits {}", bits));
    if l.phdr != 0 && l.phent != 0 && l.phnum != 0 {
        out.push(format!("; phdr 0x{:x} {} {}", l.phdr, l.phent, l.phnum));
    }
    // The symbol table travels with the module, so a saved `.imasm` file is
    // self-contained: `vox run <symbol> <file>.imasm` resolves the name from
    // the file itself, with no second read of the original binary.
    for (name, addr) in &l.symbols {
        out.push(format!("; sym {} 0x{:x}", name, addr));
    }
    // IRELATIVE relocations ride with the module so a saved `.imasm` boots the
    // same as the binary: run each resolver, store its pointer in the slot.
    for (slot, resolver) in &l.irelative {
        out.push(format!("; irel 0x{:x} 0x{:x}", slot, resolver));
    }
    // RELATIVE relocations: the machine stores the value at the slot before the
    // process runs, the self-relocation a static-pie binary does at entry.
    for (slot, value) in &l.relative {
        out.push(format!("; rela 0x{:x} 0x{:x}", slot, value));
    }
    for (at, blob) in &l.data {
        let hex: String = blob.iter().map(|b| format!("{:02x}", b)).collect();
        out.push(format!("={:#x}\t{}", at, hex));
    }
    for (base, bytes) in &l.code {
        let mut pos = 0usize;
        while pos < bytes.len() {
            let addr = base + pos as u64;
            match x86::decode_mode(&bytes[pos..], addr, bits) {
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

/// The default membrane export admits only self-contained native images.
pub fn emit_standalone(raw: &[u8]) -> Result<String, String> {
    loader::standalone(raw)?;
    Ok(format!("; standalone static-v1\n{}", emit(raw)))
}

/// Export data in address-preserving blocks directly to a caller's writer.
/// The payload remains inside the module; no sidecar or source-file read is
/// needed by the executing machine.
pub fn emit_standalone_to(raw: &[u8], mut write: impl FnMut(&[u8]) -> Result<(), String>) -> Result<(), String> {
    loader::standalone(raw)?;
    let l = loader::load(raw);
    let bits = bits_of(l.arch);
    write(b"; standalone static-v1\n")?;
    write(format!("; {} module ({} {})\n; entry 0x{:x}\n; bits {}\n", INDIRECT,l.format,l.arch,l.entry,bits).as_bytes())?;
    if l.phdr != 0 && l.phent != 0 && l.phnum != 0 {
        write(format!("; phdr 0x{:x} {} {}\n",l.phdr,l.phent,l.phnum).as_bytes())?;
    }
    for (name,addr) in &l.symbols {write(format!("; sym {} 0x{:x}\n",name,addr).as_bytes())?;}
    for (slot,resolver) in &l.irelative {write(format!("; irel 0x{:x} 0x{:x}\n",slot,resolver).as_bytes())?;}
    for (slot,value) in &l.relative {write(format!("; rela 0x{:x} 0x{:x}\n",slot,value).as_bytes())?;}
    const HEX: &[u8;16] = b"0123456789abcdef";
    let mut encoded = [0u8;8192];
    for (at,blob) in &l.data {
        for (index,chunk) in blob.chunks(4096).enumerate() {
            write(format!("={:#x}\t",at+index as u64*4096).as_bytes())?;
            for (i,byte) in chunk.iter().enumerate() {
                encoded[2*i]=HEX[(byte>>4) as usize];encoded[2*i+1]=HEX[(byte&15) as usize];
            }
            write(&encoded[..chunk.len()*2])?;write(b"\n")?;
        }
    }
    for (base,bytes) in &l.code {
        let mut pos=0;
        while pos<bytes.len() {
            let addr=base+pos as u64;
            match x86::decode_mode(&bytes[pos..],addr,bits) {
                Some(d) if d.len>0 => {
                    write(format!("@0x{:x}\n",addr).as_bytes())?;
                    for line in encode(&d,false) {write(line.as_bytes())?;write(b"\n")?;}
                    pos+=d.len;
                }
                _ => pos+=1,
            }
        }
    }
    Ok(())
}

pub fn validate_standalone(module: &str) -> Result<(), String> {
    if !module.lines().any(|line| line == "; standalone static-v1") {
        return Err("module has no standalone linkage boundary; re-emit its static executable with vox imasm".into());
    }
    Ok(())
}

/// Just the structure word (glyphs), per function.
pub fn words(raw: &[u8]) -> String {
    let l = loader::load(raw);
    let image = vox_decode::Image { segments: l.code };
    let mut seeds: Vec<u64> = l.symbols.values().copied().collect(); seeds.push(l.entry);
    let mut w = vox_decode::walk(&image, l.entry, &seeds);
    vox_decode::mark_noreturn(&mut w.functions, &l.symbols);
    let mut out: Vec<String> = Vec::new();
    for (start, f) in &w.functions {
        if f.is_empty() { continue; }
        // Audit and word export use the same CFG framing, including loop
        // regions and non-returning calls. The executable module is unchanged.
        let word = crate::vox::glyphs(&crate::vox::recompile_function(f));
        out.push(format!("0x{:x}\t{}", start, word));
    }
    out.join("\n")
}

/// Function symbol names → address, from the file's own tables.
pub fn symbols(raw: &[u8]) -> BTreeMap<String, u64> { loader::load(raw).symbols }

#[cfg(test)]
mod streaming_export_tests {
    use super::{emit_standalone,emit_standalone_to};
    use alloc::{vec,string::String};
    #[test]
    fn streamed_export_preserves_full_elf_data_and_instruction_payloads() {
        let mut raw=vec![0;10240];raw[..7].copy_from_slice(b"\x7fELF\x02\x01\x01");
        raw[18..20].copy_from_slice(&0x3eu16.to_le_bytes());raw[24..32].copy_from_slice(&0x1000u64.to_le_bytes());
        raw[32..40].copy_from_slice(&64u64.to_le_bytes());raw[54..56].copy_from_slice(&56u16.to_le_bytes());raw[56..58].copy_from_slice(&2u16.to_le_bytes());
        for (index,offset,address,length,flags) in [(0,256u64,0x1000u64,6u64,5u32),(1,512,0x2000,8193,4)] {
            let ph=64+index*56;raw[ph..ph+4].copy_from_slice(&1u32.to_le_bytes());raw[ph+4..ph+8].copy_from_slice(&flags.to_le_bytes());
            for (field,value) in [(8,offset),(16,address),(32,length),(40,length)]{raw[ph+field..ph+field+8].copy_from_slice(&value.to_le_bytes());}
        }
        raw[256..262].copy_from_slice(&[0xb8,42,0,0,0,0xc3]);
        for (index,byte) in raw[512..8705].iter_mut().enumerate(){*byte=(index%251) as u8;}
        let original=emit_standalone(&raw).unwrap();let mut streamed=vec![];
        emit_standalone_to(&raw,|bytes|{streamed.extend_from_slice(bytes);Ok(())}).unwrap();
        let streamed=String::from_utf8(streamed).unwrap();
        let expected=crate::imasm_vm::Machine::new(&original);let actual=crate::imasm_vm::Machine::new(&streamed);
        assert_eq!(actual.entry,expected.entry);assert_eq!(actual.symbols,expected.symbols);
        assert_eq!(actual.peek(0x2000,8193),expected.peek(0x2000,8193));
        let code=|text:&str|text.lines().filter(|line|!line.starts_with('=')).collect::<alloc::vec::Vec<_>>().join("\n");
        assert_eq!(code(&streamed),code(&original));
        assert!(emit_standalone_to(&raw,|_|Err("write failure".into())).is_err());
    }
}
