#!/usr/bin/env python3
"""V⊙x's own PE reader — the last dependency gone.

`pefile` is a pip install, and V⊙x needed it only to answer three questions
about a PE: which machine, which sections are executable and where they land in
memory, and where execution starts. Those are fixed-offset fields in a header
that has not changed shape since 1993, so reading them here costs less than the
dependency did.

Enough of the format to answer those questions, and no more. Nothing here parses
imports, relocations or resources; a PE's import table is still unresolved, the
same as it was with pefile.
"""

IMAGE_SCN_MEM_EXECUTE = 0x20000000
IMAGE_FILE_MACHINE_AMD64 = 0x8664


class PEError(Exception):
    pass


def _u16(b, o):
    return int.from_bytes(b[o:o + 2], "little")


def _u32(b, o):
    return int.from_bytes(b[o:o + 4], "little")


def _u64(b, o):
    return int.from_bytes(b[o:o + 8], "little")


class Section:
    __slots__ = ("name", "vaddr", "vsize", "raw_off", "raw_size", "chars")

    def __init__(self, name, vaddr, vsize, raw_off, raw_size, chars):
        self.name = name
        self.vaddr = vaddr
        self.vsize = vsize
        self.raw_off = raw_off
        self.raw_size = raw_size
        self.chars = chars

    @property
    def executable(self):
        return bool(self.chars & IMAGE_SCN_MEM_EXECUTE)

    def data(self, raw):
        return raw[self.raw_off:self.raw_off + self.raw_size]


class PE:
    """A PE, read far enough to disassemble it."""

    def __init__(self, path):
        with open(path, "rb") as fh:
            self.raw = fh.read()
        raw = self.raw
        if len(raw) < 0x40 or raw[:2] != b"MZ":
            raise PEError("not a PE: no MZ")
        pe_off = _u32(raw, 0x3C)
        if pe_off + 24 > len(raw) or raw[pe_off:pe_off + 4] != b"PE\0\0":
            raise PEError("not a PE: no PE signature")

        coff = pe_off + 4
        self.machine = _u16(raw, coff)
        self.n_sections = _u16(raw, coff + 2)
        opt_size = _u16(raw, coff + 16)

        opt = coff + 20
        magic = _u16(raw, opt)
        self.pe32_plus = magic == 0x20B
        self.entry_rva = _u32(raw, opt + 16)
        # ImageBase sits at a different offset in the two optional-header
        # shapes, and is eight bytes wide only in PE32+.
        self.image_base = _u64(raw, opt + 24) if self.pe32_plus else _u32(raw, opt + 28)

        sec_off = opt + opt_size
        self.sections = []
        for k in range(self.n_sections):
            o = sec_off + k * 40
            if o + 40 > len(raw):
                break
            name = raw[o:o + 8].rstrip(b"\0").decode("latin-1")
            self.sections.append(Section(
                name=name,
                vaddr=_u32(raw, o + 12),
                vsize=_u32(raw, o + 8),
                raw_off=_u32(raw, o + 20),
                raw_size=_u32(raw, o + 16),
                chars=_u32(raw, o + 36),
            ))

    @property
    def is64(self):
        return self.machine == IMAGE_FILE_MACHINE_AMD64

    @property
    def entry(self):
        return self.image_base + self.entry_rva

    def executable_sections(self):
        """[(bytes, virtual address)] for every section marked executable."""
        out = []
        for s in self.sections:
            if s.executable and s.raw_size:
                out.append((s.data(self.raw), self.image_base + s.vaddr))
        return out

    def total_raw(self):
        return sum(s.raw_size for s in self.sections)

    def total_code(self):
        return sum(s.raw_size for s in self.sections if s.executable)
