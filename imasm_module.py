"""The module format: a word in the twelve, and the payload each glyph carries.

The glyph is the opcode. It says which of the twelve axes an instruction is,
and that alone decides how the payload is read — a ∈ carries a condition and a
target, a ⊞ carries an ALU operation and its operands, a ⊤ carries the two
things compared. Structure and data stay separate the whole way down, which is
why the word prints as glyphs with nothing else in it while the module still
runs.

Line format, one instruction per line:

    GLYPH \t field \t field ...

Operand fields are normalised so the machine never parses assembly:

    r:rax                          a register
    i:0x10                         an immediate
    m:base:index:scale:disp:size   a memory reference
"""
import vox_x86 as capstone
from vox_x86 import X86_OP_IMM, X86_OP_MEM, X86_OP_REG

ENTRY, TERM, SPLIT, FUSE = "⊢", "⊣", "∈", "∋"
CALL, XFER, INDIRECT, COMMIT = ">", "<", "⊙", "◻"
LINK, TRUTH, CONSUME, ENGAGE = "⋈", "⊤", "⊥", "⊞"

_MOVE = {"mov", "movzx", "movsx", "movsxd", "movabs", "push", "pop", "xchg",
         "leave"}          # leave is slot movement: rsp ← rbp, then rbp popped
_INERT = {"nop", "endbr64", "endbr32"}   # never a commit, whatever they address
_TRUTH = {"cmp", "test"}
_TERMINAL = {"ret", "retf", "int3", "ud2", "hlt", "iret"}


def _operand(ins, op) -> str:
    if op.type == X86_OP_REG:
        return "r:" + ins.reg_name(op.reg)
    if op.type == X86_OP_IMM:
        return f"i:{op.imm:#x}" if op.imm >= 0 else f"i:-{-op.imm:#x}"
    if op.type == X86_OP_MEM:
        m = op.mem
        base = ins.reg_name(m.base) if m.base else ""
        index = ins.reg_name(m.index) if m.index else ""
        return f"m:{base}:{index}:{m.scale}:{m.disp:#x}:{op.size}"
    return "?:"


_PREFIX = ("notrack ", "lock ", "bnd ", "rep ", "repe ", "repne ", "data16 ")


def _mnemonic(ins) -> str:
    """Prefixes ride on the mnemonic in capstone's text. They change how an
    instruction is guarded, not which of the twelve it is, so they come off
    before classification — a `notrack jmp` is a jmp."""
    mn = ins.mnemonic
    for p in _PREFIX:
        if mn.startswith(p):
            mn = mn[len(p):]
    return mn


def encode(ins, is_merge: bool) -> list:
    """One instruction → its lines. A merge emits a bare ∋ first, because the
    fuse is a property of the address, not of the instruction sitting on it."""
    import vox
    lines = [FUSE] if is_merge else []
    mn, ops = _mnemonic(ins), ins.operands
    f = [_operand(ins, o) for o in ops]
    direct = bool(ops) and ops[0].type == X86_OP_IMM

    # The glyph is vox.classify's call, not a second opinion. Only the payload
    # is decided here, so the module text and the auditor's word cannot differ
    # on which of the twelve an instruction is.
    g = vox.classify(ins)

    if g == TERM:
        lines.append(f"{TERM}\t{mn}")
    elif g == INDIRECT and mn in ("syscall", "sysenter", "int"):
        # A syscall's real target is chosen by the value in a register (rax),
        # not written anywhere in the instruction — the same structural shape
        # as an indirect call/jmp, just to the kernel instead of the program's
        # own code. ⊙ INDIRECT is what that shape is; no thirteenth glyph
        # needed for "transfer to something outside what was disassembled."
        lines.append(f"{INDIRECT}\tsyscall")
    elif mn == "call":
        lines.append("\t".join([g, mn] + f))
    elif mn == "jmp":
        lines.append("\t".join([g, mn] + f))
    elif mn.startswith("j"):
        lines.append("\t".join([SPLIT, mn[1:]] + f))
    elif mn.startswith("set"):
        lines.append("\t".join([CONSUME, mn[3:], "set"] + f))
    elif mn.startswith("cmov"):
        lines.append("\t".join([CONSUME, mn[4:], "cmov"] + f))
    elif g == TRUTH:
        lines.append("\t".join([TRUTH, mn] + f))
    elif g == COMMIT:
        lines.append("\t".join([COMMIT, mn] + f))
    elif g == LINK:
        lines.append("\t".join([LINK, mn] + f))
    else:
        lines.append("\t".join([ENGAGE, mn] + f))
    return lines


def disassembler(path: str):
    """The decoder with operand detail on — the payload comes from structure,
    not from re-parsing assembly text. Recursive descent, function-grouped, the
    same walk the auditor uses — not a second, separate linear sweep."""
    import vox
    with open(path, "rb") as fh:
        magic = fh.read(4)
    parse = vox._elf_sections if magic == b"\x7fELF" else vox._pe_sections
    _mode, _secs, entry = parse(path)
    funcs = list(vox._native_functions(path))
    insns = [ins for _, func in funcs for ins in func]
    return insns, entry, funcs


def _plt_map(path: str):
    """(stub address -> external symbol name), ELF x86-64 only — see
    vox._plt_stub_map. PE import-table resolution isn't attempted; a PE's
    external calls stay unresolved, same as before."""
    import vox
    with open(path, "rb") as fh:
        magic = fh.read(4)
    if magic != b"\x7fELF":
        return {}
    mode, secs, _entry = vox._elf_sections(path)
    return vox._plt_stub_map(path, secs, mode)


def data_sections(path: str):
    """Initialised bytes the code will read: constants, tables, strings. A
    module with no data is not executable — a rip-relative load of a vector
    constant reads zeros and the program quietly computes the wrong answer."""
    import struct
    with open(path, "rb") as fh:
        raw = fh.read()
    if raw[:4] == b"\x7fELF":
        is64 = raw[4] == 2
        end = "<" if raw[5] == 1 else ">"
        off_shoff, fmt = (40, "QQQ") if is64 else (32, "III")
        e_shoff = struct.unpack_from(end + fmt, raw, 24)[2]
        e_shentsize, e_shnum = struct.unpack_from(
            end + "HH", raw, 58 if is64 else 46)
        out = []
        for k in range(e_shnum):
            o = e_shoff + k * e_shentsize
            if is64:
                _, sh_type, sh_flags, sh_addr, sh_off, sh_size = \
                    struct.unpack_from(end + "IIQQQQ", raw, o)
            else:
                _, sh_type, sh_flags, sh_addr, sh_off, sh_size = \
                    struct.unpack_from(end + "IIIIII", raw, o)
            # ALLOC, PROGBITS, not executable: the data the code reads
            if sh_type == 1 and sh_flags & 0x2 and not sh_flags & 0x4 and sh_size:
                out.append((sh_addr, raw[sh_off:sh_off + sh_size]))
        return out
    import pefile
    pe = pefile.PE(path, fast_load=True)
    base = pe.OPTIONAL_HEADER.ImageBase
    return [(base + s.VirtualAddress, s.get_data()) for s in pe.sections
            if not s.Characteristics & 0x20000000 and s.SizeOfRawData]



def _merges_of_function(insns) -> set:
    """Compute merge points (addresses with >=2 predecessors) for a single function.
    Uses the same prefix-stripping logic as _mnemonic to stay consistent with vox._merges_of."""
    from collections import Counter
    aset = {i.address for i in insns}
    succ = []
    for idx, ins in enumerate(insns):
        mn = _mnemonic(ins)
        if not (mn == "jmp" or mn.startswith("ret")) \
                and idx + 1 < len(insns):
            succ.append(insns[idx + 1].address)
        if mn.startswith("j") and ins.operands \
                and ins.operands[0].type == X86_OP_IMM:
            if ins.operands[0].imm in aset:
                succ.append(ins.operands[0].imm)
    return {a for a, c in Counter(succ).items() if c >= 2}


def emit(path: str) -> str:
    """The whole binary as an executable IMASM module. Recursive descent,
    function by function; a function that is a known PLT stub (see
    vox._plt_stub_map) is emitted as a single external-call line instead of
    the endbr64/indirect-jump-through-an-unloaded-GOT-slot it really holds —
    the machine never sees a binary, so that jump has nothing real to read.
    """
    from collections import Counter
    insns, entry, funcs = disassembler(path)
    stubs = _plt_map(path)

    out = [f"; ⊙ {path}", f"; entry 0x{entry:x}"]
    for addr, blob in data_sections(path):
        out.append(f"={addr:#x}\t{blob.hex()}")
    for start, func in funcs:
        if start in stubs:
            out.append(f"@0x{start:x}")
            out.append(f"{INDIRECT}\texternal\t{stubs[start]}")
            continue
        # Compute merges PER FUNCTION, matching vox._merges_of
        func_merges = _merges_of_function(func)
        for ins in func:
            out.append(f"@0x{ins.address:x}")
            out.extend(encode(ins, ins.address in func_merges))
    return "\n".join(out) + "\n"
