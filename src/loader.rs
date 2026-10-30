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
    /// ELF process-start metadata used to construct AT_PHDR/AT_PHENT/AT_PHNUM.
    pub phdr: u64,
    pub phent: u64,
    pub phnum: u64,
    pub code: Vec<(u64, Vec<u8>)>,     // (base address, bytes) of executable segments
    pub data: Vec<(u64, Vec<u8>)>,     // (base address, bytes) the code reads
    pub symbols: BTreeMap<String, u64>,
    pub format: &'static str,
    /// The machine the code is for, so a non-x86-64 binary is refused rather
    /// than silently misdecoded: "x86-64", "x86-32", "arm64", "arm", or a name.
    pub arch: &'static str,
    /// IAT slots whose target address is outside all code segments; (slot_addr, stub_addr).
    /// `emit_standalone_to` writes halt stubs at stub_addr and patches data at slot_addr.
    pub iat_fixups: alloc::vec::Vec<(u64, u64)>,
    /// R_X86_64_IRELATIVE relocations: (GOT slot address, resolver address). A
    /// static binary leaves these for its startup to apply — run each resolver,
    /// store the pointer it returns in the slot — which is how the CPU-specific
    /// memcpy/strlen get chosen. Empty for anything without them.
    pub irelative: alloc::vec::Vec<(u64, u64)>,
    /// R_X86_64_RELATIVE relocations: (slot, value). A static-pie binary leaves
    /// these for its self-relocation to apply — store base+addend at the slot.
    /// The VM loads segments at their link addresses, base 0, so the value is
    /// the addend. This is how .init_array's function pointers and the GOT come
    /// to hold real addresses. Empty for a non-relocated binary.
    pub relative: alloc::vec::Vec<(u64, u64)>,
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
    Loaded { entry: 0x1000, phdr: 0, phent: 0, phnum: 0, code: alloc::vec![(0x1000, raw.to_vec())], data: Vec::new(),
             symbols: BTreeMap::new(), irelative: alloc::vec::Vec::new(), relative: alloc::vec::Vec::new(), iat_fixups: alloc::vec::Vec::new(), format: "raw", arch: "x86-64" }
}

/// Check the runtime-linkage boundary before exporting or running a membrane.
/// Build-time dependencies are already part of the executable. ELF interpreters
/// and DT_NEEDED entries instead name code outside that retained image.
pub fn standalone(raw: &[u8]) -> Result<(), String> {
    if raw.starts_with(b"\x7fELF") {
        if raw.len() < 52 || raw[5] != 1 || !matches!(raw[4], 1 | 2) {
            return Err("invalid or unsupported standalone ELF header".into());
        }
        let wide = raw[4] == 2;
        if wide && raw.len() < 64 { return Err("truncated ELF header".into()); }
        let (off, size, count) = if wide {
            (le(raw, 32, 8), le(raw, 54, 2), le(raw, 56, 2))
        } else { (le(raw, 28, 4), le(raw, 42, 2), le(raw, 44, 2)) };
        if count == 0 || size < if wide { 56 } else { 32 } {
            return Err("standalone ELF has no valid program headers".into());
        }
        let end = count.checked_mul(size).and_then(|n| off.checked_add(n))
            .ok_or("ELF program-header range overflow")?;
        if end > raw.len() as u64 { return Err("truncated ELF program headers".into()); }
        for index in 0..count {
            let p = (off + index * size) as usize;
            match le(raw, p, 4) {
                3 => return Err("membrane requires an external ELF interpreter; rebuild it statically".into()),
                2 => {
                    let (start, len, stride) = if wide {
                        (le(raw, p + 8, 8), le(raw, p + 32, 8), 16)
                    } else { (le(raw, p + 4, 4), le(raw, p + 16, 4), 8) };
                    let end = start.checked_add(len).ok_or("ELF dynamic range overflow")?;
                    if end > raw.len() as u64 || len % stride != 0 {
                        return Err("truncated ELF dynamic table".into());
                    }
                    let mut at = start;
                    while at < end {
                        let tag = le(raw, at as usize, (stride / 2) as usize);
                        if tag == 0 { break; }
                        if matches!(tag, 1 | 0x7fff_fffd | 0x7fff_ffff) {
                            return Err("membrane requires an external shared library; rebuild it statically".into());
                        }
                        at += stride;
                    }
                }
                _ => {}
            }
        }
        Ok(())
    } else if &raw[0..2] == b"MZ" {
        PE::standalone(raw)
    } else {
        Err("standalone membrane requires an ELF executable".into())
    }
}

#[cfg(test)]
mod standalone_tests {
    use super::standalone;
    fn image(kind: u32, tag: u64) -> alloc::vec::Vec<u8> {
        let mut bytes = alloc::vec![0; 144];
        bytes[..7].copy_from_slice(b"\x7fELF\x02\x01\x01");
        bytes[32..40].copy_from_slice(&64u64.to_le_bytes());
        bytes[54..56].copy_from_slice(&56u16.to_le_bytes());
        bytes[56..58].copy_from_slice(&1u16.to_le_bytes());
        bytes[64..68].copy_from_slice(&kind.to_le_bytes());
        bytes[72..80].copy_from_slice(&128u64.to_le_bytes());
        bytes[96..104].copy_from_slice(&16u64.to_le_bytes());
        bytes[128..136].copy_from_slice(&tag.to_le_bytes());
        bytes
    }
    #[test]
    fn static_image_is_admitted() { assert!(standalone(&image(1, 0)).is_ok()); }
    #[test]
    fn interpreter_and_external_libraries_are_rejected() {
        assert!(standalone(&image(3, 0)).is_err());
        assert!(standalone(&image(2, 1)).is_err());
        assert!(standalone(&image(2, 0)).is_ok());
    }
    #[test]
    fn truncated_headers_and_dynamic_tables_are_rejected() {
        assert!(standalone(&image(1, 0)[..100]).is_err());
        assert!(standalone(&image(2, 0)[..136]).is_err());
        assert!(standalone(b"\x7fELF").is_err());
    }
    #[test]
    fn resident_image_retains_allocated_dynamic_metadata_and_omits_nobits() {
        let mut raw = alloc::vec![0; 0x600];
        raw[..7].copy_from_slice(b"\x7fELF\x02\x01\x01");
        raw[40..48].copy_from_slice(&0x100u64.to_le_bytes());
        raw[58..60].copy_from_slice(&64u16.to_le_bytes());
        raw[60..62].copy_from_slice(&5u16.to_le_bytes());
        for (index, kind) in [3u32, 6, 11, 0x6fff_fff6, 8].into_iter().enumerate() {
            let section = 0x100 + index * 64;
            let offset = 0x300 + index * 16;
            let address = 0x400000 + offset as u64;
            raw[section + 4..section + 8].copy_from_slice(&kind.to_le_bytes());
            raw[section + 8..section + 16].copy_from_slice(&2u64.to_le_bytes());
            raw[section + 16..section + 24].copy_from_slice(&address.to_le_bytes());
            raw[section + 24..section + 32].copy_from_slice(&(offset as u64).to_le_bytes());
            raw[section + 32..section + 40].copy_from_slice(&8u64.to_le_bytes());
            raw[offset..offset + 8].fill(index as u8 + 1);
        }
        let loaded = super::load(&raw);
        for index in 0..4 {
            let address = 0x400300 + index * 16;
            let expected = alloc::vec![index as u8 + 1; 8];
            assert!(loaded.data.iter().any(|(at, data)| *at == address && *data == expected));
        }
        assert!(!loaded.data.iter().any(|(at, _)| *at == 0x400340));
    }
    #[test]
    fn zero_pie_base_keeps_headers_at_zero_with_later_segment_padding() {
        let mut raw = alloc::vec![0; 512];
        raw[..7].copy_from_slice(b"\x7fELF\x02\x01\x01");
        raw[32..40].copy_from_slice(&64u64.to_le_bytes());
        raw[54..56].copy_from_slice(&56u16.to_le_bytes());
        raw[56..58].copy_from_slice(&2u16.to_le_bytes());
        for (index, offset, address, length, flags) in [(0, 0u64, 0u64, 512u64, 4u32),
            (1, 256, 4352, 16, 5)] {
            let ph = 64 + index * 56;
            raw[ph..ph + 4].copy_from_slice(&1u32.to_le_bytes());
            raw[ph + 4..ph + 8].copy_from_slice(&flags.to_le_bytes());
            raw[ph + 8..ph + 16].copy_from_slice(&offset.to_le_bytes());
            raw[ph + 16..ph + 24].copy_from_slice(&address.to_le_bytes());
            raw[ph + 32..ph + 40].copy_from_slice(&length.to_le_bytes());
        }
        let loaded = super::load(&raw);
        assert_eq!(loaded.phdr, 64);
        assert!(loaded.data.iter().any(|(at, bytes)| *at == 0 && bytes.starts_with(b"\x7fELF")));
        assert!(!loaded.data.iter().any(|(at, _)| *at == 4096));
    }
}

// ── ELF (32- and 64-bit) ──────────────────────────────────────────────────────
fn elf(raw: &[u8]) -> Loaded {
    let is64 = raw.get(4).copied() == Some(2);
    let mut out = Loaded { entry: 0, phdr: 0, phent: 0, phnum: 0, code: Vec::new(), data: Vec::new(), symbols: BTreeMap::new(), irelative: alloc::vec::Vec::new(), relative: alloc::vec::Vec::new(), iat_fixups: alloc::vec::Vec::new(), format: "elf", arch: arch_name(le(raw,18,2), "elf") };
    // header field widths and offsets differ by class
    let w = if is64 { 8usize } else { 4 };                 // address width
    out.entry = le(raw, 24, w);
    let phoff = le(raw, if is64 {0x20} else {0x1c}, w) as usize;
    out.phent = le(raw, if is64 {0x36} else {0x2a}, 2);
    out.phnum = le(raw, if is64 {0x38} else {0x2c}, 2);
    let phent = out.phent as usize;
    let mut load_base = None;
    for k in 0..out.phnum as usize {
        let p = phoff + k * phent;
        if p + phent > raw.len() { break; }
        let ptype = le(raw, p, 4);
        let (off, addr) = if is64 { (le(raw,p+8,8),le(raw,p+16,8)) } else { (le(raw,p+4,4),le(raw,p+8,4)) };
        if ptype == 6 { out.phdr = addr; }
        if ptype == 1 && load_base.is_none() { load_base = Some(addr.wrapping_sub(off)); }
    }
    if out.phdr == 0 { out.phdr = load_base.unwrap_or(0).wrapping_add(phoff as u64); }
    // __ehdr_start and the in-memory program-header table are read by glibc
    // before main. Section loading omits these non-section ELF bytes, so carry
    // the ELF header and PHDR table as ordinary module data.
    let headers_end = phoff.saturating_add((out.phnum as usize).saturating_mul(phent));
    if let Some(base) = load_base.filter(|_| headers_end <= raw.len()) {
        out.data.push((base, raw[..headers_end].to_vec()));
    }
    let shoff = le(raw, if is64 {0x28} else {0x20}, w) as usize;
    let shentsize = le(raw, if is64 {0x3a} else {0x2e}, 2) as usize;
    let shnum = le(raw, if is64 {0x3c} else {0x30}, 2) as usize;
    // section field offsets: sh_type, sh_flags, sh_addr, sh_offset, sh_size, sh_link, sh_entsize
    let (f_flags, f_addr, f_off, f_size, f_link, f_ent) = if is64 {
        (8, 16, 24, 32, 40, 56) } else { (8, 12, 16, 20, 24, 36) };
    let sh = |k: usize, f: usize, n: usize| le(raw, shoff + k * shentsize + f, n);
    for k in 0..shnum {
        let sh_type = sh(k, 4, 4);
        let sh_flags = sh(k, f_flags, w);
        let sh_addr = sh(k, f_addr, w);
        let sh_off = sh(k, f_off, w) as usize;
        let sh_size = sh(k, f_size, w) as usize;
        // Every allocated, file-backed section belongs to the resident image.
        // This includes dynamic symbol/string/version/hash tables read by
        // dlopen, as well as relocations and constructor/destructor arrays.
        // Only NOBITS (.bss) has no file bytes and remains zero in the map.
        let loadable = sh_type != 8;
        if loadable && sh_off + sh_size <= raw.len() && sh_size > 0 && (sh_flags & 0x2) != 0 {
            let bytes = raw[sh_off..sh_off + sh_size].to_vec();
            if (sh_flags & 0x4) != 0 { out.code.push((sh_addr, bytes)); }
            else { out.data.push((sh_addr, bytes)); }
        }
    }
    if out.code.is_empty() {
        for k in 0..out.phnum as usize {
            let p = phoff + k * phent;
            if p + phent > raw.len() || le(raw, p, 4) != 1 { continue; }
            let (flags, off, addr, size) = if is64 {
                (le(raw, p + 4, 4), le(raw, p + 8, 8), le(raw, p + 16, 8), le(raw, p + 32, 8))
            } else {
                (le(raw, p + 24, 4), le(raw, p + 4, 4), le(raw, p + 8, 4), le(raw, p + 16, 4))
            };
            let (Ok(off), Ok(size)) = (usize::try_from(off), usize::try_from(size)) else { continue; };
            let Some(end) = off.checked_add(size) else { continue; };
            if size == 0 || end > raw.len() { continue; }
            let bytes = raw[off..end].to_vec();
            if flags & 1 != 0 { out.code.push((addr, bytes)); }
            else if flags & 4 != 0 { out.data.push((addr, bytes)); }
        }
    }
    // function symbols. Elf32_Sym is 16 bytes (name,value,size,info,other,shndx);
    // Elf64_Sym is 24 (name,info,other,shndx,value,size). info and value move.
    for k in 0..shnum {
        let sh_type = sh(k, 4, 4);
        if sh_type != 2 && sh_type != 11 { continue; }
        let sym_off = sh(k, f_off, w) as usize; let sym_size = sh(k, f_size, w) as usize;
        let strtab = sh(k, f_link, 4) as usize; let str_off = sh(strtab, f_off, w) as usize;
        let entsize: usize = if is64 { 24 } else { 16 };
        let ent = (sh(k, f_ent, w) as usize).max(entsize);
        let mut o = sym_off;
        while o + entsize <= sym_off + sym_size && o + entsize <= raw.len() {
            let st_name = le(raw, o, 4) as usize;
            let (st_info, st_value) = if is64 {
                (raw.get(o + 4).copied().unwrap_or(0), le(raw, o + 8, 8))
            } else {
                (raw.get(o + 12).copied().unwrap_or(0), le(raw, o + 4, 4))
            };
            if (st_info & 0xf) == 2 && st_value != 0 {
                let name = cstr(raw, str_off + st_name);
                if !name.is_empty() { out.symbols.insert(name, st_value); }
            }
            o += ent;
        }
    }
    // IRELATIVE relocations from any SHT_RELA section (.rela.dyn on a static
    // binary). Elf64_Rela is r_offset(8) r_info(8) r_addend(8); r_type is the
    // low 32 bits of r_info, 0x25 is R_X86_64_IRELATIVE, and its addend is the
    // resolver address. 32-bit ELF carries no IRELATIVE, so this is 64-bit only.
    if is64 {
        for k in 0..shnum {
            if sh(k, 4, 4) != 4 { continue; } // SHT_RELA
            let r_off = sh(k, f_off, w) as usize;
            let r_size = sh(k, f_size, w) as usize;
            let mut o = r_off;
            while o + 24 <= r_off + r_size && o + 24 <= raw.len() {
                let r_offset = le(raw, o, 8);
                let r_info = le(raw, o + 8, 8);
                let r_addend = le(raw, o + 16, 8);
                match r_info & 0xffff_ffff {
                    0x25 => out.irelative.push((r_offset, r_addend)), // IRELATIVE
                    0x08 => out.relative.push((r_offset, r_addend)),  // RELATIVE
                    _ => {}
                }
                o += 24;
            }
        }
    }
    out
}

// ── PE ───────────────────────────────────────────────────────────────────────
struct PE {}

impl PE {
    fn standalone(_raw: &[u8]) -> Result<(), String> {
        Ok(())
    }
}

#[allow(dead_code)]
fn pe_rva_to_off(raw: &[u8], rva: u64) -> Option<usize> {
    let lfanew = le(raw, 0x3c, 4) as usize;
    if lfanew + 24 > raw.len() { return None; }
    let coff = lfanew + 4;
    let nsec = le(raw, coff + 2, 2) as usize;
    let opt_size = le(raw, coff + 16, 2) as usize;
    let opt = coff + 20;
    let sects = opt + opt_size;
    for k in 0..nsec {
        let s = sects + k * 40;
        if s + 40 > raw.len() { break; }
        let vaddr = le(raw, s + 12, 4);
        let raw_size = le(raw, s + 16, 4) as usize;
        let raw_ptr = le(raw, s + 20, 4) as usize;
        if rva >= vaddr as u64 && rva < (vaddr as u64 + raw_size as u64) {
            return Some(raw_ptr + (rva - vaddr as u64) as usize);
        }
    }
    None
}

#[allow(dead_code)]
fn pe_count_iat(raw: &[u8], iat_rva: u64, is64: bool) -> usize {
    let mut count = 0;
    let mut off = match pe_rva_to_off(raw, iat_rva) { Some(o) => o, None => return 0 };
    let stride = if is64 { 8 } else { 4 };
    while off + stride <= raw.len() {
        let val = le(raw, off, stride);
        if val == 0 { break; }
        if is64 {
            if (val >> 63) & 1 == 1 { count += 1; off += stride; continue; }
            count += 1;
        } else {
            if (val >> 31) & 1 == 1 { count += 1; off += stride; continue; }
            count += 1;
        }
        off += stride;
    }
    count
}

fn pe(raw: &[u8]) -> Loaded {
    let mut out = Loaded { entry: 0, phdr: 0, phent: 0, phnum: 0, code: Vec::new(), data: Vec::new(), symbols: BTreeMap::new(), irelative: alloc::vec::Vec::new(), relative: alloc::vec::Vec::new(), iat_fixups: alloc::vec::Vec::new(), format: "pe", arch: "other" };
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

    // ── Import directory: collect IAT slots pointing outside code ──
    // Data directory entry 1 = imports. PE32+ puts entry 0 at opt+112, entry 1 at opt+120.
    // PE32 puts entry 0 at opt+96, entry 1 at opt+104.
    let (is64, dir_off) = if magic == 0x20b {
        (true, opt + 120)         // PE32+: entry 1 (imports)
    } else {
        (false, opt + 104)         // PE32: entry 1 (imports)
    };
    let dir_rva = le(raw, dir_off, 4);
    let dir_size = le(raw, dir_off + 4, 4) as usize;
    if dir_rva != 0 && dir_size != 0 {
        let mut at_rva = dir_rva;
        let mut at_off = pe_rva_to_off(raw, at_rva);
        while let Some(off) = at_off {
            if off + 20 > raw.len() { break; }
            let orig_rva = le(raw, off, 4);
            let iat_rva   = le(raw, off + 16, 4);
            if orig_rva == 0 && iat_rva == 0 { break; }
            if iat_rva != 0 {
                let mut slot_rva = iat_rva;
                loop {
                    let slot_off = match pe_rva_to_off(raw, slot_rva) { Some(o) => o, None => break };
                    let target = le(raw, slot_off, if is64 { 8 } else { 4 });
                    if target == 0 { break; }
                    // Only stub slots whose target is outside all code segments
                    let mut in_code = false;
                    for (cb, cbytes) in &out.code {
                        let cstart = *cb;
                        let cend = cstart + cbytes.len() as u64;
                        if target >= cstart && target < cend { in_code = true; break; }
                    }
                    if !in_code {
                        out.iat_fixups.push((image_base + slot_rva, target));
                    }
                    slot_rva += if is64 { 8 } else { 4 };
                }
            }
            at_rva += 20;
            at_off = pe_rva_to_off(raw, at_rva);
        }
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
    Loaded { entry: 0, phdr: 0, phent: 0, phnum: 0, code: Vec::new(), data: Vec::new(), symbols: BTreeMap::new(), irelative: alloc::vec::Vec::new(), relative: alloc::vec::Vec::new(), iat_fixups: alloc::vec::Vec::new(), format: "macho-fat", arch: "other" }
}

fn macho(raw: &[u8], base_off: usize) -> Loaded {
    let mut out = Loaded { entry: 0, phdr: 0, phent: 0, phnum: 0, code: Vec::new(), data: Vec::new(), symbols: BTreeMap::new(), irelative: alloc::vec::Vec::new(), relative: alloc::vec::Vec::new(), iat_fixups: alloc::vec::Vec::new(), format: "macho", arch: arch_name(le(raw, base_off+4, 4), "macho") };
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


/// Read a defined ELF64 little-endian object symbol without treating it as code.
pub fn elf_object_symbol(raw: &[u8], requested: &str) -> Option<(u64, u64)> {
    if raw.get(..6)? != b"\x7fELF\x02\x01" { return None; }
    let shoff = usize::try_from(le(raw, 40, 8)).ok()?;
    let shsize = le(raw, 58, 2) as usize;
    let shcount = le(raw, 60, 2) as usize;
    if shsize < 64 { return None; }
    let section = |index: usize| -> Option<&[u8]> {
        if index >= shcount { return None; }
        let start = shoff.checked_add(index.checked_mul(shsize)?)?;
        raw.get(start..start.checked_add(64)?)
    };
    let contents = |header: &[u8]| -> Option<&[u8]> {
        let start = usize::try_from(le(header, 24, 8)).ok()?;
        let size = usize::try_from(le(header, 32, 8)).ok()?;
        raw.get(start..start.checked_add(size)?)
    };
    for index in 0..shcount {
        let header = section(index)?;
        if !matches!(le(header, 4, 4), 2 | 11) { continue; }
        let strings = contents(section(le(header, 40, 4) as usize)?)?;
        let symbols = contents(header)?;
        let entry_size = usize::try_from(le(header, 56, 8)).ok()?;
        if entry_size < 24 { continue; }
        for entry in symbols.chunks_exact(entry_size) {
            if entry[4] & 15 != 1 || le(entry, 6, 2) == 0 { continue; }
            let start = le(entry, 0, 4) as usize;
            let Some(tail) = strings.get(start..) else { continue; };
            let Some(end) = tail.iter().position(|&byte| byte == 0) else { continue; };
            if &tail[..end] == requested.as_bytes() {
                return Some((le(entry, 8, 8), le(entry, 16, 8)));
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn put_le(raw: &mut [u8], offset: usize, value: u64, size: usize) {
        for byte in 0..size {
            raw[offset + byte] = (value >> (8 * byte)) as u8;
        }
    }

    #[test]
    fn sectionless_elf_loads_program_header_segments() {
        let mut raw = alloc::vec![0u8; 0x104];
        raw[..6].copy_from_slice(b"\x7fELF\x02\x01");
        put_le(&mut raw, 16, 2, 2);
        put_le(&mut raw, 18, 0x3e, 2);
        put_le(&mut raw, 24, 0x400080, 8);
        put_le(&mut raw, 32, 64, 8);
        put_le(&mut raw, 54, 56, 2);
        put_le(&mut raw, 56, 2, 2);
        put_le(&mut raw, 64, 1, 4);
        put_le(&mut raw, 68, 5, 4);
        put_le(&mut raw, 72, 0, 8);
        put_le(&mut raw, 80, 0x400000, 8);
        put_le(&mut raw, 96, 0x100, 8);
        put_le(&mut raw, 120, 1, 4);
        put_le(&mut raw, 124, 6, 4);
        put_le(&mut raw, 128, 0x100, 8);
        put_le(&mut raw, 136, 0x600000, 8);
        put_le(&mut raw, 152, 4, 8);
        raw[0x100..].copy_from_slice(&[1, 2, 3, 4]);

        let loaded = load(&raw);

        assert_eq!(loaded.entry, 0x400080);
        assert_eq!(loaded.code, alloc::vec![(0x400000, raw[..0x100].to_vec())]);
        assert!(loaded.data.iter().any(|(address, bytes)| {
            *address == 0x600000 && bytes == &[1, 2, 3, 4]
        }));
    }
}
