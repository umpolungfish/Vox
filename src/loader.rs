//! Universal executable loader: ELF, PE, Mach-O (including fat), and a raw
//! fallback. The x86 decoder and the machine are format-agnostic; they take
//! code segments and data segments. This is the one place a file's container is
//! read, so every format arrives at the same pipeline: (entry, code, data).
//!
//! The container is universal; the instruction set is x86-64. A code segment
//! that is not x86 simply fails to decode, instruction by instruction, and the
//! walk reports how far it got, rather than being refused up front.

use alloc::string::String;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;

pub struct Loaded {
    pub entry: u64,
    pub code: Vec<(u64, Vec<u8>)>,     // (base address, bytes) of executable segments
    pub data: Vec<(u64, Vec<u8>)>,     // (base address, bytes) the code reads
    pub symbols: BTreeMap<String, u64>,
    pub format: &'static str,
    /// The machine the code is for, so a non-x86-64 binary is refused rather
    /// than silently misdecoded: "x86-64", "x86-32", "arm64", "arm", or a name.
    pub arch: &'static str,
}

fn arch_name(machine: u64, kind: &str) -> &'static str {
    match kind {
        "elf" => match machine { 0x3E => "x86-64", 0x03 => "x86-32", 0xB7 => "arm64", 0x28 => "arm", 0xF3 => "riscv", _ => "other" },
        "pe"  => match machine { 0x8664 => "x86-64", 0x14c => "x86-32", 0xAA64 => "arm64", 0x1c0|0x1c4 => "arm", _ => "other" },
        "macho" => match machine { 0x0100_0007 => "x86-64", 0x0000_0007 => "x86-32", 0x0100_000C => "arm64", 0x0000_000C => "arm", _ => "other" },
        _ => "unknown",
    }
}

fn le(raw: &[u8], o: usize, n: usize) -> u64 {
    let mut v = 0u64;
    for k in 0..n { if o + k < raw.len() { v |= (raw[o + k] as u64) << (8 * k); } }
    v
}
fn be(raw: &[u8], o: usize, n: usize) -> u64 {
    let mut v = 0u64;
    for k in 0..n { if o + k < raw.len() { v = (v << 8) | raw[o + k] as u64; } }
    v
}

pub fn load(raw: &[u8]) -> Loaded {
    if raw.len() >= 4 {
        if &raw[0..4] == b"\x7fELF" { return elf(raw); }
        if &raw[0..2] == b"MZ"      { return pe(raw); }
        let m = le(raw, 0, 4);
        if m == 0xFEED_FACF || m == 0xFEED_FACE { return macho(raw, 0); }
        if be(raw, 0, 4) == 0xCAFE_BABE         { return macho_fat(raw); }
    }
    // raw / flat binary: the whole file as code, a conventional load base.
    Loaded { entry: 0x1000, code: alloc::vec![(0x1000, raw.to_vec())], data: Vec::new(),
             symbols: BTreeMap::new(), format: "raw", arch: "x86-64" }
}

// ── ELF ──────────────────────────────────────────────────────────────────────
fn elf(raw: &[u8]) -> Loaded {
    let is64 = raw.get(4).copied() == Some(2);
    let mut out = Loaded { entry: 0, code: Vec::new(), data: Vec::new(), symbols: BTreeMap::new(), format: "elf", arch: arch_name(le(raw,18,2), "elf") };
    if !is64 { return out; }
    out.entry = le(raw, 24, 8);
    let shoff = le(raw, 0x28, 8) as usize;
    let shentsize = le(raw, 0x3a, 2) as usize;
    let shnum = le(raw, 0x3c, 2) as usize;
    let sh = |k: usize, f: usize, n: usize| le(raw, shoff + k * shentsize + f, n);
    for k in 0..shnum {
        let sh_type = sh(k, 4, 4);
        let sh_flags = sh(k, 8, 8);
        let sh_addr = sh(k, 16, 8);
        let sh_off = sh(k, 24, 8) as usize;
        let sh_size = sh(k, 32, 8) as usize;
        if sh_type == 1 && sh_off + sh_size <= raw.len() && sh_size > 0 && (sh_flags & 0x2) != 0 {
            let bytes = raw[sh_off..sh_off + sh_size].to_vec();
            if (sh_flags & 0x4) != 0 { out.code.push((sh_addr, bytes)); }   // SHF_EXECINSTR
            else { out.data.push((sh_addr, bytes)); }
        }
    }
    // function symbols
    for k in 0..shnum {
        let sh_type = sh(k, 4, 4);
        if sh_type != 2 && sh_type != 11 { continue; }
        let sym_off = sh(k, 24, 8) as usize; let sym_size = sh(k, 32, 8) as usize;
        let strtab = sh(k, 40, 4) as usize; let str_off = sh(strtab, 24, 8) as usize;
        let ent = sh(k, 56, 8).max(24) as usize;
        let mut o = sym_off;
        while o + 24 <= sym_off + sym_size && o + 24 <= raw.len() {
            let st_name = le(raw, o, 4) as usize;
            let st_info = raw.get(o + 4).copied().unwrap_or(0);
            let st_value = le(raw, o + 8, 8);
            if (st_info & 0xf) == 2 && st_value != 0 {
                let name = cstr(raw, str_off + st_name);
                if !name.is_empty() { out.symbols.insert(name, st_value); }
            }
            o += ent;
        }
    }
    out
}

// ── PE ───────────────────────────────────────────────────────────────────────
fn pe(raw: &[u8]) -> Loaded {
    let mut out = Loaded { entry: 0, code: Vec::new(), data: Vec::new(), symbols: BTreeMap::new(), format: "pe", arch: "other" };
    let lfanew = le(raw, 0x3c, 4) as usize;
    if lfanew + 24 > raw.len() || &raw[lfanew..lfanew + 4] != b"PE\0\0" { return out; }
    let coff = lfanew + 4;
    out.arch = arch_name(le(raw, coff, 2), "pe");
    let nsec = le(raw, coff + 2, 2) as usize;
    let opt_size = le(raw, coff + 16, 2) as usize;
    let opt = coff + 20;
    let magic = le(raw, opt, 2);
    let (image_base, entry_rva) = if magic == 0x20b {
        (le(raw, opt + 24, 8), le(raw, opt + 16, 4))         // PE32+
    } else {
        (le(raw, opt + 28, 4), le(raw, opt + 16, 4))         // PE32
    };
    out.entry = image_base + entry_rva;
    let sects = opt + opt_size;
    for k in 0..nsec {
        let s = sects + k * 40;
        if s + 40 > raw.len() { break; }
        let vaddr = le(raw, s + 12, 4);
        let raw_size = le(raw, s + 16, 4) as usize;
        let raw_ptr = le(raw, s + 20, 4) as usize;
        let chars = le(raw, s + 36, 4);
        if raw_size == 0 || raw_ptr + raw_size > raw.len() { continue; }
        let bytes = raw[raw_ptr..raw_ptr + raw_size].to_vec();
        let base = image_base + vaddr;
        // IMAGE_SCN_MEM_EXECUTE 0x20000000, IMAGE_SCN_CNT_CODE 0x20
        if (chars & 0x2000_0000) != 0 || (chars & 0x20) != 0 { out.code.push((base, bytes)); }
        else if (chars & 0x4000_0000) != 0 { out.data.push((base, bytes)); }   // MEM_READ
    }
    out
}

// ── Mach-O ───────────────────────────────────────────────────────────────────
fn macho_fat(raw: &[u8]) -> Loaded {
    let nfat = be(raw, 4, 4) as usize;
    for k in 0..nfat {
        let o = 8 + k * 20;
        let cputype = be(raw, o, 4);
        let offset = be(raw, o + 8, 4) as usize;
        if cputype == 0x0100_0007 && offset < raw.len() { return macho(raw, offset); }   // x86_64
    }
    if nfat > 0 { return macho(raw, be(raw, 16, 4) as usize); }
    Loaded { entry: 0, code: Vec::new(), data: Vec::new(), symbols: BTreeMap::new(), format: "macho-fat", arch: "other" }
}

fn macho(raw: &[u8], base_off: usize) -> Loaded {
    let mut out = Loaded { entry: 0, code: Vec::new(), data: Vec::new(), symbols: BTreeMap::new(), format: "macho", arch: arch_name(le(raw, base_off+4, 4), "macho") };
    let ncmds = le(raw, base_off + 16, 4) as usize;
    let mut cmd = base_off + 32;   // 64-bit header
    let mut text_base = 0u64;
    for _ in 0..ncmds {
        if cmd + 8 > raw.len() { break; }
        let ctype = le(raw, cmd, 4);
        let csize = le(raw, cmd + 4, 4) as usize;
        if csize == 0 { break; }
        if ctype == 0x19 {   // LC_SEGMENT_64
            let vmaddr = le(raw, cmd + 24, 8);
            let nsects = le(raw, cmd + 64, 4) as usize;
            let segname = cstr(raw, cmd + 8);
            if segname == "__TEXT" && text_base == 0 { text_base = vmaddr; }
            let mut so = cmd + 72;
            for _ in 0..nsects {
                if so + 80 > raw.len() { break; }
                let addr = le(raw, so + 32, 8);
                let size = le(raw, so + 40, 8) as usize;
                let offset = le(raw, so + 48, 4) as usize;
                let flags = le(raw, so + 64, 4);
                if size > 0 && offset + size <= raw.len() {
                    let bytes = raw[offset..offset + size].to_vec();
                    // S_ATTR_PURE_INSTRUCTIONS 0x80000000, S_ATTR_SOME_INSTRUCTIONS 0x00000400
                    if (flags & 0x8000_0000) != 0 || (flags & 0x400) != 0 { out.code.push((addr, bytes)); }
                    else { out.data.push((addr, bytes)); }
                }
                so += 80;
            }
        } else if ctype == 0x8000_0028 {   // LC_MAIN
            let entryoff = le(raw, cmd + 8, 8);
            out.entry = text_base.wrapping_add(entryoff);
        }
        cmd += csize;
    }
    if out.entry == 0 { if let Some((b, _)) = out.code.first() { out.entry = *b; } }
    out
}

fn cstr(raw: &[u8], mut p: usize) -> String {
    let mut s = String::new();
    while let Some(&b) = raw.get(p) { if b == 0 { break; } s.push(b as char); p += 1; }
    s
}
