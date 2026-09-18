#!/usr/bin/env python3
"""V⊙x's own x86-64 decoder — the standalone lane.

V⊙x read x86 through capstone. That is a C library and a pip install, which
means the tool that claims a program is a word could not read a program without
someone else's disassembler. This module removes that: it decodes x86-64 to the
same shape capstone hands back, so `vox.py` and `imasm_module.py` can run with
nothing installed.

The surface deliberately mimics capstone's, because matching an interface that
already works is cheaper and safer than changing every call site:

    md = Cs(CS_ARCH_X86, CS_MODE_64)
    md.detail = True
    for ins in md.disasm(data, vaddr):
        ins.address, ins.size, ins.mnemonic, ins.op_str, ins.operands
        ins.reg_name(op.reg)

Operands carry `.type` of X86_OP_REG / X86_OP_IMM / X86_OP_MEM, and a memory
operand carries `.mem.base`, `.mem.index`, `.mem.scale`, `.mem.disp`. That is
everything `imasm_module.encode` reads, which is the demanding consumer: it
emits an executable module, so an operand it gets wrong becomes a wrong answer
in the machine rather than a wrong-looking listing.

An opcode this decoder does not know raises no guess. `disasm` stops there. A
length decoder that guesses a width slips out of phase with the instruction
stream and keeps producing confident, wrong instructions.
"""

# ── capstone-compatible constants ──────────────────────────────

CS_ARCH_X86 = 3
CS_MODE_16 = 1 << 1
CS_MODE_32 = 1 << 2
CS_MODE_64 = 1 << 3

X86_OP_INVALID = 0
X86_OP_REG = 1
X86_OP_IMM = 2
X86_OP_MEM = 3

# ── registers ──────────────────────────────────────────────────
#
# Register ids are indices into these tables, offset by class, so `reg_name`
# is a lookup rather than a decode. Capstone's own ids are opaque integers and
# nothing outside `reg_name` depends on their values.

_R64 = ["rax", "rcx", "rdx", "rbx", "rsp", "rbp", "rsi", "rdi",
        "r8", "r9", "r10", "r11", "r12", "r13", "r14", "r15"]
_R32 = ["eax", "ecx", "edx", "ebx", "esp", "ebp", "esi", "edi",
        "r8d", "r9d", "r10d", "r11d", "r12d", "r13d", "r14d", "r15d"]
_R16 = ["ax", "cx", "dx", "bx", "sp", "bp", "si", "di",
        "r8w", "r9w", "r10w", "r11w", "r12w", "r13w", "r14w", "r15w"]
_R8REX = ["al", "cl", "dl", "bl", "spl", "bpl", "sil", "dil",
          "r8b", "r9b", "r10b", "r11b", "r12b", "r13b", "r14b", "r15b"]
_R8 = ["al", "cl", "dl", "bl", "ah", "ch", "dh", "bh"]
_XMM = [f"xmm{i}" for i in range(16)]

_REG_TABLES = {8: _R64, 4: _R32, 2: _R16, 1: _R8REX, 0: _R8, 16: _XMM}

# A register id encodes (size-class, index) so one integer round-trips a name.
_CLASS_ORDER = [8, 4, 2, 1, 0, 16]


def _reg_id(size, idx):
    return _CLASS_ORDER.index(size) * 32 + idx + 1


def reg_name(rid):
    """Name for a register id. Id 0 is 'no register', as in capstone."""
    if not rid:
        return None
    rid -= 1
    cls, idx = divmod(rid, 32)
    table = _REG_TABLES[_CLASS_ORDER[cls]]
    return table[idx] if idx < len(table) else None


# ── operand containers ─────────────────────────────────────────

class _Mem:
    __slots__ = ("base", "index", "scale", "disp", "segment")

    def __init__(self, base=0, index=0, scale=1, disp=0):
        self.base = base
        self.index = index
        self.scale = scale
        self.disp = disp
        self.segment = 0


class Operand:
    __slots__ = ("type", "reg", "imm", "mem", "size")

    def __init__(self, type_, reg=0, imm=0, mem=None, size=0):
        self.type = type_
        self.reg = reg
        self.imm = imm
        self.mem = mem if mem is not None else _Mem()
        self.size = size


class Insn:
    """One decoded instruction, shaped like a capstone `CsInsn`."""

    __slots__ = ("address", "size", "mnemonic", "op_str", "operands", "bytes")

    def __init__(self, address, size, mnemonic, op_str, operands, raw):
        self.address = address
        self.size = size
        self.mnemonic = mnemonic
        self.op_str = op_str
        self.operands = operands
        self.bytes = raw

    def reg_name(self, rid):
        return reg_name(rid)

    def __repr__(self):
        return f"<Insn 0x{self.address:x} {self.mnemonic} {self.op_str}>"


class DecodeError(Exception):
    """Raised for an opcode the decoder does not know."""


# ── the decoder ────────────────────────────────────────────────

_CC = ["o", "no", "b", "ae", "e", "ne", "be", "a",
       "s", "ns", "p", "np", "l", "ge", "le", "g"]

_ARITH = ["add", "or", "adc", "sbb", "and", "sub", "xor", "cmp"]
_SHIFT = ["rol", "ror", "rcl", "rcr", "shl", "shr", "sal", "sar"]
_UNARY = ["test", "test", "not", "neg", "mul", "imul", "div", "idiv"]

_PREFIXES = {0x66, 0x67, 0xF0, 0xF2, 0xF3, 0x2E, 0x36, 0x3E, 0x26, 0x64, 0x65}


class _Dec:
    """One decode in progress. Holds the prefix state the opcode needs."""

    def __init__(self, data, pos, addr, mode64):
        self.d = data
        self.p = pos
        self.start = pos
        self.addr = addr
        self.mode64 = mode64
        self.rex = 0
        self.has_rex = False        # a bare 0x40 REX sets no bits but still
                                    # selects the spl/bpl/sil/dil byte file
        self.o16 = False
        self.rep = None
        self.seg = None

    # byte readers
    def u8(self):
        if self.p >= len(self.d):
            raise DecodeError("truncated")
        v = self.d[self.p]
        self.p += 1
        return v

    def i8(self):
        v = self.u8()
        return v - 256 if v > 127 else v

    def u16(self):
        return self.u8() | (self.u8() << 8)

    def i16(self):
        v = self.u16()
        return v - 0x10000 if v > 0x7FFF else v

    def u32(self):
        v = 0
        for k in range(4):
            v |= self.u8() << (8 * k)
        return v

    def i32(self):
        v = self.u32()
        return v - 0x100000000 if v > 0x7FFFFFFF else v

    def u64(self):
        v = 0
        for k in range(8):
            v |= self.u8() << (8 * k)
        return v

    # rex bits
    @property
    def w(self):
        return bool(self.rex & 8)

    @property
    def r(self):
        return 8 if self.rex & 4 else 0

    @property
    def x(self):
        return 8 if self.rex & 2 else 0

    @property
    def b(self):
        return 8 if self.rex & 1 else 0

    def opsize(self):
        if self.w:
            return 8
        if self.o16:
            return 2
        return 4

    def regcls(self, size):
        # Without REX the byte registers are the legacy ah/ch/dh/bh set. The
        # test is whether a REX byte was present at all, not whether it set any
        # bits: 0x40 sets none and still switches the file.
        if size == 1 and not self.has_rex:
            return 0
        return size

    def reg_op(self, idx, size):
        return Operand(X86_OP_REG, reg=_reg_id(self.regcls(size), idx), size=size)

    def modrm(self, size, xmm_reg=False, xmm_rm=False):
        """Decode a ModRM byte into (reg operand, rm operand)."""
        m = self.u8()
        mod = m >> 6
        reg = ((m >> 3) & 7) | self.r
        rm = m & 7

        rsize = 16 if xmm_reg else size
        rop = Operand(X86_OP_REG, reg=_reg_id(self.regcls(rsize), reg), size=rsize)

        if mod == 3:
            msize = 16 if xmm_rm else size
            rmop = Operand(X86_OP_REG,
                           reg=_reg_id(self.regcls(msize), rm | self.b), size=msize)
            return rop, rmop

        base = index = 0
        scale = 1
        disp = 0
        if rm == 4:
            sib = self.u8()
            scale = 1 << (sib >> 6)
            idx = ((sib >> 3) & 7) | self.x
            bs = (sib & 7) | self.b
            if idx != 4:                      # index 4 means "no index"
                index = _reg_id(8, idx)
            if (sib & 7) == 5 and mod == 0:
                disp = self.i32()
            else:
                base = _reg_id(8, bs)
        elif rm == 5 and mod == 0:
            # RIP-relative. The base is recorded as rip so a caller can
            # recompute the target the way the CPU does.
            disp = self.i32()
            base = _RIP_ID
        else:
            base = _reg_id(8, rm | self.b)

        if mod == 1:
            disp = self.i8()
        elif mod == 2:
            disp = self.i32()

        mem = _Mem(base=base, index=index, scale=scale, disp=disp)
        return rop, Operand(X86_OP_MEM, mem=mem, size=16 if xmm_rm else size)


# rip is not a general register; it gets an id past the general file.
_RIP_ID = _reg_id(8, 0) + 200
_REG_NAME_EXTRA = {_RIP_ID: "rip"}

_orig_reg_name = reg_name


def reg_name(rid):  # noqa: F811 - deliberate shadow, extends the table
    if rid in _REG_NAME_EXTRA:
        return _REG_NAME_EXTRA[rid]
    return _orig_reg_name(rid)


def _fmt_mem(op):
    m = op.mem
    parts = []
    if m.base:
        parts.append(reg_name(m.base))
    if m.index:
        parts.append(f"{reg_name(m.index)}*{m.scale}")
    if m.disp or not parts:
        parts.append(f"0x{m.disp:x}" if m.disp >= 0 else f"-0x{-m.disp:x}")
    return "[" + " + ".join(parts) + "]"


def _fmt(op):
    if op.type == X86_OP_REG:
        return reg_name(op.reg) or "?"
    if op.type == X86_OP_IMM:
        return f"0x{op.imm:x}" if op.imm >= 0 else f"-0x{-op.imm:x}"
    return _fmt_mem(op)


def _mk(dec, mnemonic, ops):
    size = dec.p - dec.start
    op_str = ", ".join(_fmt(o) for o in ops)
    raw = bytes(dec.d[dec.start:dec.p])
    return Insn(dec.addr, size, mnemonic, op_str, ops, raw)


def decode(data, pos, addr, mode64=True):
    """Decode one instruction at `data[pos:]`, sitting at virtual `addr`.

    Raises DecodeError on anything unrecognised or truncated.
    """
    dec = _Dec(data, pos, addr, mode64)

    while True:
        if dec.p >= len(dec.d):
            raise DecodeError("truncated in prefixes")
        p = dec.d[dec.p]
        if p == 0x66:
            dec.o16 = True
            dec.p += 1
        elif p in (0xF2, 0xF3):
            dec.rep = p
            dec.p += 1
        elif p in _PREFIXES:
            if p in (0x2E, 0x36, 0x3E, 0x26, 0x64, 0x65):
                dec.seg = p
            dec.p += 1
        elif mode64 and 0x40 <= p <= 0x4F:
            dec.has_rex = True
            dec.rex = p & 0x0F
            dec.p += 1
            break
        else:
            break

    op = dec.u8()
    if op == 0x0F:
        return _decode_0f(dec)
    return _decode_1(dec, op)


def _decode_1(dec, op):
    sz = dec.opsize()

    # 00..3D: the eight arithmetic operations, in a regular grid.
    if op < 0x40 and (op & 7) <= 5 and (op & 0xC7) != 0x0F:
        name = _ARITH[op >> 3]
        lo = op & 7
        if lo in (0, 1, 2, 3):
            bsize = 1 if lo in (0, 2) else sz
            r, rm = dec.modrm(bsize)
            ops = [rm, r] if lo in (0, 1) else [r, rm]
            return _mk(dec, name, ops)
        if lo == 4:
            return _mk(dec, name, [dec.reg_op(0, 1), Operand(X86_OP_IMM, imm=dec.i8())])
        imm = dec.u16() if dec.o16 else dec.u32()
        return _mk(dec, name, [dec.reg_op(0, sz), Operand(X86_OP_IMM, imm=imm)])

    if 0x50 <= op <= 0x57:
        return _mk(dec, "push", [dec.reg_op((op & 7) | dec.b, 8)])
    if 0x58 <= op <= 0x5F:
        return _mk(dec, "pop", [dec.reg_op((op & 7) | dec.b, 8)])
    if op == 0x63:
        r, rm = dec.modrm(4)
        r.reg = _reg_id(8, (r.reg - 1) % 32)
        return _mk(dec, "movsxd", [r, rm])
    if op == 0x68:
        return _mk(dec, "push", [Operand(X86_OP_IMM, imm=dec.i32())])
    if op == 0x6A:
        return _mk(dec, "push", [Operand(X86_OP_IMM, imm=dec.i8())])
    if op in (0x69, 0x6B):
        r, rm = dec.modrm(sz)
        imm = dec.i8() if op == 0x6B else (dec.i16() if dec.o16 else dec.i32())
        return _mk(dec, "imul", [r, rm, Operand(X86_OP_IMM, imm=imm)])
    if 0x70 <= op <= 0x7F:
        d = dec.i8()
        tgt = dec.addr + (dec.p - dec.start) + d
        return _mk(dec, "j" + _CC[op & 15], [Operand(X86_OP_IMM, imm=tgt)])
    if op in (0x80, 0x81, 0x83):
        bsize = 1 if op == 0x80 else sz
        m = dec.d[dec.p]
        name = _ARITH[(m >> 3) & 7]
        _, rm = dec.modrm(bsize)
        if op == 0x81:
            imm = dec.u16() if dec.o16 else dec.u32()
        else:
            imm = dec.i8()
            # A bitwise operation reads its immediate as a mask, so the value
            # is the sign-extension widened to the operand, not the small
            # negative. Arithmetic keeps the signed reading.
            if name in ("and", "or", "xor") and imm < 0:
                imm &= (1 << (8 * bsize)) - 1
        return _mk(dec, name, [rm, Operand(X86_OP_IMM, imm=imm)])
    if op in (0x84, 0x85):
        r, rm = dec.modrm(1 if op == 0x84 else sz)
        return _mk(dec, "test", [rm, r])
    if op in (0x86, 0x87):
        r, rm = dec.modrm(1 if op == 0x86 else sz)
        return _mk(dec, "xchg", [rm, r])
    if 0x88 <= op <= 0x8B:
        bsize = 1 if op in (0x88, 0x8A) else sz
        r, rm = dec.modrm(bsize)
        ops = [rm, r] if op in (0x88, 0x89) else [r, rm]
        return _mk(dec, "mov", ops)
    if op == 0x8D:
        r, rm = dec.modrm(sz)
        return _mk(dec, "lea", [r, rm])
    if op == 0x8F:
        _, rm = dec.modrm(8)
        return _mk(dec, "pop", [rm])
    if op == 0x90 and not dec.rex:
        return _mk(dec, "nop", [])
    if 0x90 <= op <= 0x97:
        return _mk(dec, "xchg", [dec.reg_op(0, sz), dec.reg_op((op & 7) | dec.b, sz)])
    if op == 0x98:
        return _mk(dec, "cdqe" if dec.w else "cwde", [])
    if op == 0x99:
        return _mk(dec, "cqo" if dec.w else "cdq", [])
    if op == 0x9C:
        return _mk(dec, "pushfq", [])
    if op == 0x9D:
        return _mk(dec, "popfq", [])
    if op in (0xA4, 0xA5):
        return _mk(dec, "movsb" if op == 0xA4 else "movsq", [])
    if op in (0xA6, 0xA7):
        return _mk(dec, "cmpsb" if op == 0xA6 else "cmpsq", [])
    if op in (0xAA, 0xAB):
        return _mk(dec, "stosb" if op == 0xAA else "stosq", [])
    if op in (0xAC, 0xAD):
        return _mk(dec, "lodsb" if op == 0xAC else "lodsq", [])
    if op in (0xAE, 0xAF):
        return _mk(dec, "scasb" if op == 0xAE else "scasq", [])
    if op == 0xA8:
        return _mk(dec, "test", [dec.reg_op(0, 1), Operand(X86_OP_IMM, imm=dec.i8())])
    if op == 0xA9:
        imm = dec.i16() if dec.o16 else dec.i32()
        return _mk(dec, "test", [dec.reg_op(0, sz), Operand(X86_OP_IMM, imm=imm)])
    if 0xB0 <= op <= 0xB7:
        return _mk(dec, "mov", [dec.reg_op((op & 7) | dec.b, 1),
                                Operand(X86_OP_IMM, imm=dec.u8())])
    if 0xB8 <= op <= 0xBF:
        if dec.w:
            imm = dec.u64()
        elif dec.o16:
            imm = dec.u16()
        else:
            imm = dec.u32()
        return _mk(dec, "movabs" if dec.w else "mov",
                   [dec.reg_op((op & 7) | dec.b, dec.opsize()),
                    Operand(X86_OP_IMM, imm=imm)])
    if op in (0xC0, 0xC1):
        m = dec.d[dec.p]
        name = _SHIFT[(m >> 3) & 7]
        _, rm = dec.modrm(1 if op == 0xC0 else sz)
        return _mk(dec, name, [rm, Operand(X86_OP_IMM, imm=dec.u8())])
    if op == 0xC2:
        return _mk(dec, "ret", [Operand(X86_OP_IMM, imm=dec.u16())])
    if op == 0xC3:
        return _mk(dec, "ret", [])
    if op in (0xC6, 0xC7):
        bsize = 1 if op == 0xC6 else sz
        _, rm = dec.modrm(bsize)
        imm = dec.i8() if op == 0xC6 else (dec.i16() if dec.o16 else dec.i32())
        return _mk(dec, "mov", [rm, Operand(X86_OP_IMM, imm=imm)])
    if op == 0xC9:
        return _mk(dec, "leave", [])
    if op == 0xCC:
        return _mk(dec, "int3", [])
    if op == 0xCD:
        return _mk(dec, "int", [Operand(X86_OP_IMM, imm=dec.u8())])
    if 0xD0 <= op <= 0xD3:
        m = dec.d[dec.p]
        name = _SHIFT[(m >> 3) & 7]
        _, rm = dec.modrm(1 if op in (0xD0, 0xD2) else sz)
        second = Operand(X86_OP_IMM, imm=1) if op in (0xD0, 0xD1) \
            else dec.reg_op(1, 1)
        return _mk(dec, name, [rm, second])
    if 0xD8 <= op <= 0xDF:
        _, rm = dec.modrm(8)
        return _mk(dec, "fld", [rm])
    if op == 0xE8:
        d = dec.i32()
        tgt = dec.addr + (dec.p - dec.start) + d
        return _mk(dec, "call", [Operand(X86_OP_IMM, imm=tgt)])
    if op == 0xE9:
        d = dec.i32()
        tgt = dec.addr + (dec.p - dec.start) + d
        return _mk(dec, "jmp", [Operand(X86_OP_IMM, imm=tgt)])
    if op == 0xEB:
        d = dec.i8()
        tgt = dec.addr + (dec.p - dec.start) + d
        return _mk(dec, "jmp", [Operand(X86_OP_IMM, imm=tgt)])
    if 0xE0 <= op <= 0xE3:
        d = dec.i8()
        tgt = dec.addr + (dec.p - dec.start) + d
        return _mk(dec, "loop", [Operand(X86_OP_IMM, imm=tgt)])
    if op == 0xF4:
        return _mk(dec, "hlt", [])
    if op in (0xF6, 0xF7):
        m = dec.d[dec.p]
        which = (m >> 3) & 7
        bsize = 1 if op == 0xF6 else sz
        _, rm = dec.modrm(bsize)
        if which <= 1:
            imm = dec.i8() if op == 0xF6 else (dec.i16() if dec.o16 else dec.i32())
            return _mk(dec, "test", [rm, Operand(X86_OP_IMM, imm=imm)])
        return _mk(dec, _UNARY[which], [rm])
    if op == 0xFE:
        m = dec.d[dec.p]
        which = (m >> 3) & 7
        _, rm = dec.modrm(1)
        return _mk(dec, "inc" if which == 0 else "dec", [rm])
    if op == 0xFF:
        m = dec.d[dec.p]
        which = (m >> 3) & 7
        _, rm = dec.modrm(8 if which in (2, 3, 4, 5, 6) else sz)
        name = {0: "inc", 1: "dec", 2: "call", 3: "call",
                4: "jmp", 5: "jmp", 6: "push"}.get(which)
        if name is None:
            raise DecodeError(f"FF /{which}")
        return _mk(dec, name, [rm])
    if op in (0x9B, 0x9E, 0x9F, 0xD7, 0xF5, 0xF8, 0xF9, 0xFA, 0xFB, 0xFC, 0xFD):
        return _mk(dec, "lahf", [])
    if op in (0x8C, 0x8E):
        r, rm = dec.modrm(2)
        return _mk(dec, "mov", [rm, r] if op == 0x8C else [r, rm])

    raise DecodeError(f"opcode 0x{op:02x}")


def _decode_0f(dec):
    sz = dec.opsize()
    op2 = dec.u8()

    if op2 == 0x01:
        # Group 7. The mod=3 forms (xgetbv, xsetbv, vmcall, swapgs, rdtscp)
        # are still ModRM-encoded, so consuming the ModRM gives the length
        # whichever form it is.
        m = dec.d[dec.p] if dec.p < len(dec.d) else 0
        _, rm = dec.modrm(8)
        name = {0xD0: "xgetbv", 0xD1: "xsetbv", 0xF9: "rdtscp",
                0xF8: "swapgs", 0xC1: "vmcall"}.get(m, "sgdt")
        return _mk(dec, name, [] if m >= 0xC0 else [rm])
    if op2 == 0x05:
        return _mk(dec, "syscall", [])
    if op2 == 0x0B:
        return _mk(dec, "ud2", [])
    if op2 in (0x1E, 0x1F):
        _, rm = dec.modrm(sz)
        # F3 0F 1E FA is endbr64; vox's descent keys on the mnemonic.
        if dec.rep == 0xF3 and op2 == 0x1E:
            return _mk(dec, "endbr64", [])
        return _mk(dec, "nop", [rm])
    if 0x40 <= op2 <= 0x4F:
        r, rm = dec.modrm(sz)
        return _mk(dec, "cmov" + _CC[op2 & 15], [r, rm])
    if 0x80 <= op2 <= 0x8F:
        d = dec.i32()
        tgt = dec.addr + (dec.p - dec.start) + d
        return _mk(dec, "j" + _CC[op2 & 15], [Operand(X86_OP_IMM, imm=tgt)])
    if 0x90 <= op2 <= 0x9F:
        _, rm = dec.modrm(1)
        return _mk(dec, "set" + _CC[op2 & 15], [rm])
    if op2 == 0xAF:
        r, rm = dec.modrm(sz)
        return _mk(dec, "imul", [r, rm])
    if op2 in (0xB6, 0xB7):
        r, rm = dec.modrm(1 if op2 == 0xB6 else 2)
        r.reg = _reg_id(dec.regcls(sz), (r.reg - 1) % 32)
        return _mk(dec, "movzx", [r, rm])
    if op2 in (0xBE, 0xBF):
        r, rm = dec.modrm(1 if op2 == 0xBE else 2)
        r.reg = _reg_id(dec.regcls(sz), (r.reg - 1) % 32)
        return _mk(dec, "movsx", [r, rm])
    if op2 in (0xB0, 0xB1):
        r, rm = dec.modrm(1 if op2 == 0xB0 else sz)
        return _mk(dec, "cmpxchg", [rm, r])
    if op2 in (0xC0, 0xC1):
        r, rm = dec.modrm(1 if op2 == 0xC0 else sz)
        return _mk(dec, "xadd", [rm, r])
    if op2 == 0xA2:
        return _mk(dec, "cpuid", [])
    if op2 in (0x31,):
        return _mk(dec, "rdtsc", [])
    if 0xC8 <= op2 <= 0xCF:
        return _mk(dec, "bswap", [dec.reg_op((op2 & 7) | dec.b, sz)])
    if op2 == 0xBA:
        _, rm = dec.modrm(sz)
        return _mk(dec, "bt", [rm, Operand(X86_OP_IMM, imm=dec.u8())])
    if op2 in (0xA3, 0xAB, 0xB3, 0xBB):
        r, rm = dec.modrm(sz)
        return _mk(dec, "bt", [rm, r])
    if op2 in (0xBC, 0xBD):
        r, rm = dec.modrm(sz)
        return _mk(dec, "bsf" if op2 == 0xBC else "bsr", [r, rm])
    if op2 in (0xA4, 0xAC):
        r, rm = dec.modrm(sz)
        return _mk(dec, "shld", [rm, r, Operand(X86_OP_IMM, imm=dec.u8())])
    if op2 in (0xA5, 0xAD):
        r, rm = dec.modrm(sz)
        return _mk(dec, "shld", [rm, r])
    if op2 in (0x6E, 0x7E) and dec.rep != 0xF3:
        r, rm = dec.modrm(8 if dec.w else 4, xmm_reg=True, xmm_rm=False)
        name = "movq" if dec.w else "movd"
        return _mk(dec, name, [rm, r] if op2 == 0x7E else [r, rm])
    if op2 == 0x38:
        dec.u8()
        r, rm = dec.modrm(16, xmm_reg=True, xmm_rm=True)
        return _mk(dec, "pshufb", [r, rm])
    if op2 == 0x3A:
        dec.u8()
        r, rm = dec.modrm(16, xmm_reg=True, xmm_rm=True)
        return _mk(dec, "palignr", [r, rm, Operand(X86_OP_IMM, imm=dec.u8())])
    if op2 == 0x70:
        r, rm = dec.modrm(16, xmm_reg=True, xmm_rm=True)
        name = "pshufd" if dec.o16 else ("pshufhw" if dec.rep == 0xF3 else "pshufw")
        return _mk(dec, name, [r, rm, Operand(X86_OP_IMM, imm=dec.u8())])
    if op2 in (0x71, 0x72, 0x73):
        # The reg field selects the shift; the r/m register is the destination.
        m = dec.d[dec.p]
        which = (m >> 3) & 7
        _, rm = dec.modrm(16, xmm_reg=True, xmm_rm=True)
        name = _SHIFT_GRP.get((op2, which))
        if name is None:
            raise DecodeError(f"0f {op2:02x} /{which}")
        return _mk(dec, name, [rm, Operand(X86_OP_IMM, imm=dec.u8())])
    if op2 in (0xC2, 0xC4, 0xC5, 0xC6):
        r, rm = dec.modrm(16, xmm_reg=True, xmm_rm=True)
        return _mk(dec, "shufps", [r, rm, Operand(X86_OP_IMM, imm=dec.u8())])
    if op2 == 0xAE:
        _, rm = dec.modrm(sz)
        return _mk(dec, "fence", [rm])
    if op2 == 0xC7:
        _, rm = dec.modrm(8)
        return _mk(dec, "cmpxchg16b", [rm])

    # Everything else in the escape space is ModRM-shaped with no immediate.
    # The forms that carry an immediate, and the ones with no operand at all,
    # are enumerated above; defaulting the remainder to ModRM covers SSE, the
    # prefetch hints, the control/debug moves and the rest without a table.
    if True:
        r, rm = dec.modrm(16, xmm_reg=True, xmm_rm=True)
        name = _sse_name(op2, dec)
        # The odd member of each load/store pair writes memory, so the
        # operands run the other way. Getting this backwards turns a store
        # into a load, which the machine then executes.
        if op2 in _SSE_STORE:
            return _mk(dec, name, [rm, r])
        return _mk(dec, name, [r, rm])

    raise DecodeError(f"opcode 0f {op2:02x}")


_SSE_STORE = {0x11, 0x13, 0x17, 0x29, 0x2B, 0x7F, 0xD6, 0xE7}

_SHIFT_GRP = {
    (0x71, 2): "psrlw", (0x71, 4): "psraw", (0x71, 6): "psllw",
    (0x72, 2): "psrld", (0x72, 4): "psrad", (0x72, 6): "pslld",
    (0x73, 2): "psrlq", (0x73, 3): "psrldq", (0x73, 6): "psllq", (0x73, 7): "pslldq",
}

# Integer SSE, where the operation is the same whatever the prefix.
_SSE_FIXED = {
    0x60: "punpcklbw", 0x61: "punpcklwd", 0x62: "punpckldq", 0x63: "packsswb",
    0x64: "pcmpgtb", 0x65: "pcmpgtw", 0x66: "pcmpgtd", 0x67: "packuswb",
    0x68: "punpckhbw", 0x69: "punpckhwd", 0x6A: "punpckhdq", 0x6B: "packssdw",
    0x6C: "punpcklqdq", 0x6D: "punpckhqdq",
    0x74: "pcmpeqb", 0x75: "pcmpeqw", 0x76: "pcmpeqd",
    0xD4: "paddq", 0xD5: "pmullw", 0xD8: "psubusb", 0xD9: "psubusw",
    0xDA: "pminub", 0xDB: "pand", 0xDC: "paddusb", 0xDD: "paddusw",
    0xDE: "pmaxub", 0xDF: "pandn",
    0xE0: "pavgb", 0xE1: "psraw", 0xE2: "psrad", 0xE3: "pavgw",
    0xE4: "pmulhuw", 0xE5: "pmulhw", 0xE8: "psubsb", 0xE9: "psubsw",
    0xEA: "pminsw", 0xEB: "por", 0xEC: "paddsb", 0xED: "paddsw",
    0xEE: "pmaxsw", 0xEF: "pxor",
    0xF1: "psllw", 0xF2: "pslld", 0xF3: "psllq", 0xF4: "pmuludq",
    0xF5: "pmaddwd", 0xF6: "psadbw", 0xF8: "psubb", 0xF9: "psubw",
    0xFA: "psubd", 0xFB: "psubq", 0xFC: "paddb", 0xFD: "paddw", 0xFE: "paddd",
    0xD1: "psrlw", 0xD2: "psrld", 0xD3: "psrlq",
}

# Float SSE, where the prefix picks packed/scalar and single/double.
_SSE_ARITH = {
    0x51: "sqrt", 0x54: "and", 0x55: "andn", 0x56: "or", 0x57: "xor",
    0x58: "add", 0x59: "mul", 0x5C: "sub", 0x5D: "min", 0x5E: "div", 0x5F: "max",
}


def _fsuffix(dec):
    if dec.rep == 0xF3:
        return "ss"
    if dec.rep == 0xF2:
        return "sd"
    return "pd" if dec.o16 else "ps"


def _sse_name(op2, dec):
    """The mnemonic for an SSE opcode.

    The glyph does not depend on this — every one of these is engagement — but
    the machine executes off the mnemonic, so a wrong name is a wrong answer
    rather than a cosmetic blemish.
    """
    if op2 in _SSE_FIXED:
        return _SSE_FIXED[op2]
    if op2 in _SSE_ARITH:
        base = _SSE_ARITH[op2]
        if op2 in (0x54, 0x55, 0x56, 0x57):
            return base + ("pd" if dec.o16 else "ps")
        return base + _fsuffix(dec)
    if op2 in (0x10, 0x11):
        if dec.rep == 0xF3:
            return "movss"
        if dec.rep == 0xF2:
            return "movsd"
        return "movupd" if dec.o16 else "movups"
    if op2 in (0x28, 0x29):
        return "movapd" if dec.o16 else "movaps"
    if op2 in (0x12, 0x13):
        return "movlpd" if dec.o16 else "movlps"
    if op2 in (0x16, 0x17):
        return "movhpd" if dec.o16 else "movhps"
    if op2 == 0x14:
        return "unpcklpd" if dec.o16 else "unpcklps"
    if op2 == 0x15:
        return "unpckhpd" if dec.o16 else "unpckhps"
    if op2 in (0x2A,):
        return "cvtsi2" + _fsuffix(dec)
    if op2 in (0x2C, 0x2D):
        return "cvttsi" if op2 == 0x2C else "cvtsi"
    if op2 in (0x2E, 0x2F):
        base = "ucomis" if op2 == 0x2E else "comis"
        return base + ("d" if dec.o16 else "s")
    if op2 == 0x5A:
        return "cvt" + _fsuffix(dec)
    if op2 == 0x5B:
        return "cvtdq2ps"
    if op2 == 0x6E:
        return "movq" if dec.w else "movd"
    if op2 == 0x6F:
        return "movdqu" if dec.rep == 0xF3 else ("movdqa" if dec.o16 else "movq")
    if op2 == 0x7E:
        if dec.rep == 0xF3:
            return "movq"
        return "movq" if dec.w else "movd"
    if op2 == 0x7F:
        return "movdqu" if dec.rep == 0xF3 else ("movdqa" if dec.o16 else "movq")
    if op2 == 0xD6:
        return "movq"
    if op2 == 0xE7:
        return "movntdq"
    if op2 == 0xD7:
        return "pmovmskb"
    if op2 == 0x50:
        return "movmskpd" if dec.o16 else "movmskps"
    return "paddb"


# ── capstone-shaped driver ─────────────────────────────────────

class Cs:
    """Stands in for `capstone.Cs`, for the calls V⊙x actually makes."""

    def __init__(self, arch=CS_ARCH_X86, mode=CS_MODE_64):
        self.arch = arch
        self.mode = mode
        self.detail = False
        self.mode64 = mode == CS_MODE_64

    def disasm(self, data, addr, count=0):
        """Yield instructions from `data`, stopping at the first byte that does
        not decode. Stopping is the point: guessing a length puts every later
        instruction in the wrong place."""
        pos = 0
        emitted = 0
        while pos < len(data):
            try:
                ins = decode(data, pos, addr + pos, self.mode64)
            except DecodeError:
                return
            if ins.size <= 0:
                return
            yield ins
            pos += ins.size
            emitted += 1
            if count and emitted >= count:
                return

    def disasm_lite(self, data, addr, count=0):
        for ins in self.disasm(data, addr, count):
            yield (ins.address, ins.size, ins.mnemonic, ins.op_str)


def rip_target(ins):
    """The absolute address a rip-relative memory operand names, or None.

    This is what PLT-stub resolution needs: the CPU computes the target as this
    instruction's own end plus the displacement.
    """
    for op in ins.operands:
        if op.type == X86_OP_MEM and op.mem.base == _RIP_ID:
            return ins.address + ins.size + op.mem.disp
    return None
