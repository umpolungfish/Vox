#!/usr/bin/env python3
"""V⊙x — control-flow closure auditor for real bytecode.

V⊙x lifts a program's control-flow graph to a word in the twelve-opcode
Imscribing Grammar (IMASM) and runs the SIXTEEN_3 verdict engine over it, no
hand-judging. The load-bearing bug shape is a fork that COMMITS state (or
returns) before its paths rejoin — reentrancy, an unhandled path, a leak. In
IMASM that is a δ (FSPLIT) that never fuses (FFUSE) before an IFIX/TANCH, and the
kernel reads it OPEN (verdict B) with no language-specific knowledge.

Lift (CFG skeleton, the noise filtered to the load-bearing ops):
  function entry            -> VINIT   ⊢
  conditional branch        -> FSPLIT  ∈   (fork; its target is the merge)
  the merge point (target)  -> FFUSE   ∋   (paths rejoin)
  external call / work       -> AFWD    >
  state write (STORE_*)     -> IFIX    ◻
  return                    -> TANCH   ⊣

Verdicts are Belnap FOUR (the universe is not two-valued): T = the control flow
closes; B = a fork holds open across a commit (finding, triage to definite); N =
a linear routine that never forked, clean. Four front ends, one law: CPython
(`dis`), EVM (`--evm HEX`, jump targets resolved from the preceding PUSH), WASM
(`--wasm HEX`, structured if/end control), and native x86 PE binaries
(auto-detected by the MZ magic; needs `capstone` and `pefile`). The vulnerable
case in each lifts to the same open-fork signature (VINIT FSPLIT IFIX TANCH… →
B): one bug shape, one verdict, regardless of language. `--selftest` runs the
EVM+WASM vuln/safe pairs.

It reads the REAL bytecode CFG, not the source, and that distinction is load
bearing: CPython 3.12 tail-duplicates a common continuation into both arms of an
if/else, so a source-level merge can be compiled away and both arms genuinely
commit-and-return. The scanner reports what actually executes. A true merge (a
target with ≥2 predecessors — an `if` with a single continuation, or a loop
back-edge) survives and reads as FFUSE.
"""
import argparse
import dis
import importlib.util
from pathlib import Path

# The SIXTEEN_3 trilattice engine, vendored — V⊙x is standalone.
from imasm16_3_core import IMASM16_3_Machine, Sequence16_3Trace  # noqa

# The twelve IMASM opcodes, as the glyphs and nothing else. A word is a list of
# these, and every front end below builds one directly — there is no name form
# to translate out of. ⊙ stands at slot nine, Criticality, and it is the same ⊙
# whether read as the primitive, as its own type, or as the SIXTEEN_3 opcode
# IMASM16_3_core calls IMSCRIB — a critical point is where a system turns on
# itself, and imscribing is inclosure. Self-reference is the thread that makes
# the readings one, which is also why lifting into SIXTEEN_3 needs no
# translation table: FSPLIT/FFUSE/ENGAGR and FSPLIT3/FFUSE3/EVALI are the same
# three glyphs (∈, ∋, ⊞) under two names, and a glyph does not have two names.
VINIT, TANCH, AFWD, AREV = "⊢", "⊣", ">", "<"
CLINK, EVALT, FSPLIT, FFUSE = "⋈", "⊤", "∈", "∋"
IMSCRIB, EVALF, ENGAGR, IFIX = "⊙", "⊥", "⊞", "◻"


def glyphs(word):
    """The lifted word, joined into one string. It already is glyphs — every
    front end appends them directly — so this is just the join."""
    return "".join(word)


_STORE = ("STORE_FAST", "STORE_GLOBAL", "STORE_DEREF", "STORE_NAME",
          "STORE_ATTR", "STORE_SUBSCR")


_RETURN = ("RETURN_VALUE", "RETURN_CONST")
_UNCOND_JUMP = ("JUMP_FORWARD", "JUMP_BACKWARD", "JUMP_ABSOLUTE")


def _merge_offsets(instrs) -> set:
    """Offsets where control converges from ≥2 predecessors — the true merges.

    A jump TARGET is not a merge by itself: if the fall-through arm returned or
    jumped away first, the target has a single predecessor and the fork never
    rejoins. Only ≥2 incoming edges is a real FFUSE. This is what separates a
    guard whose paths merge from a branch that commits and returns early.
    """
    from collections import Counter
    succ = []
    for idx, ins in enumerate(instrs):
        op = ins.opname
        # fall-through edge (returns and unconditional jumps have none)
        if op not in _RETURN and op not in _UNCOND_JUMP and idx + 1 < len(instrs):
            succ.append(instrs[idx + 1].offset)
        # jump edge
        if "JUMP" in op and isinstance(ins.argval, int):
            succ.append(ins.argval)
    pred = Counter(succ)
    return {off for off, c in pred.items() if c >= 2}


def lift_function(func) -> list:
    """Lift a function's bytecode CFG skeleton to an IMASM opcode-name word."""
    instrs = list(dis.get_instructions(func))
    merges = _merge_offsets(instrs)
    tokens = [VINIT]
    for ins in instrs:
        if ins.offset in merges:
            tokens.append(FFUSE)       # paths genuinely rejoin here
        op = ins.opname
        if op.startswith("POP_JUMP_IF"):
            tokens.append(FSPLIT)
        elif op in _STORE:
            tokens.append(IFIX)
        elif op.startswith("CALL"):
            tokens.append(AFWD)
        elif op in _RETURN:
            tokens.append(TANCH)
    # A fork with no matching merge is left dangling on purpose: the engine reads
    # the openness (a commit/return that escaped the fork) rather than us hiding it.
    return tokens


# ── EVM front-end ────────────────────────────────────────────────────────────
# Same control-flow-closure law on smart-contract bytecode: a state commit
# (SSTORE) that lands inside a branch which has not rejoined is the DAO-class
# reentrancy shape. EVM jump destinations are computed (pushed then JUMP/JUMPI),
# so we resolve a jump's target from the PUSH immediately before it — the form
# every compiler emits.
_EVM = {
    0x00: "STOP", 0x54: "SLOAD", 0x55: "SSTORE", 0x56: "JUMP", 0x57: "JUMPI",
    0x5b: "JUMPDEST", 0x35: "CALLDATALOAD", 0x58: "PC", 0x5a: "GAS",
    0xf1: "CALL", 0xf2: "CALLCODE", 0xf3: "RETURN", 0xf4: "DELEGATECALL",
    0xfa: "STATICCALL", 0xfd: "REVERT",
    0x01: "ADD", 0x03: "SUB", 0x10: "LT", 0x11: "GT", 0x14: "EQ", 0x15: "ISZERO",
}
_EVM_WORK = {"SLOAD", "CALLDATALOAD", "ADD", "SUB", "LT", "GT", "EQ", "ISZERO",
             "GAS", "PC", "CALL", "CALLCODE", "DELEGATECALL", "STATICCALL"}
_EVM_HALT = {"STOP", "RETURN", "REVERT", "JUMP"}   # no fall-through edge


def parse_evm(hexstr: str) -> list:
    b = bytes.fromhex(hexstr.replace("0x", "").strip())
    instrs, i, prev = [], 0, None
    while i < len(b):
        op = b[i]
        if 0x60 <= op <= 0x7f:                     # PUSH1..PUSH32
            n = op - 0x5f
            rec = {"off": i, "name": f"PUSH{n}", "val": int.from_bytes(b[i + 1:i + 1 + n], "big")}
            i += 1 + n
        else:
            rec = {"off": i, "name": _EVM.get(op, f"OP_{op:02x}")}
            if rec["name"] in ("JUMP", "JUMPI") and prev and prev["name"].startswith("PUSH"):
                rec["target"] = prev["val"]         # target = the PUSH just before
            i += 1
        instrs.append(rec)
        prev = rec
    return instrs


def lift_evm(instrs: list) -> list:
    from collections import Counter
    succ = []
    for idx, ins in enumerate(instrs):
        if ins["name"] not in _EVM_HALT and idx + 1 < len(instrs):
            succ.append(instrs[idx + 1]["off"])
        if ins["name"] in ("JUMP", "JUMPI") and ins.get("target") is not None:
            succ.append(ins["target"])
    merges = {off for off, c in Counter(succ).items() if c >= 2}
    tokens = [VINIT]
    for ins in instrs:
        nm = ins["name"]
        if nm == "JUMPDEST" and ins["off"] in merges:
            tokens.append(FFUSE)
        if nm == "JUMPI":
            tokens.append(FSPLIT)
        elif nm == "SSTORE":
            tokens.append(IFIX)
        elif nm in _EVM_WORK:
            tokens.append(AFWD)
        elif nm in ("STOP", "RETURN", "REVERT"):
            tokens.append(TANCH)
    return tokens


# ── WASM front-end ───────────────────────────────────────────────────────────
# WASM control flow is STRUCTURED: `if`/`else`/`end`, `block`/`loop`, `br`/`br_if`.
# So the fork and its merge are explicit in the opcodes, no predecessor analysis
# needed — an `if` forks, its matching `end` is the merge UNLESS a `return`/`br`
# escaped the then-branch first, exactly the CPython rule. Input is a function
# body's instruction bytes (hex); operand immediates are skipped to stay aligned.


def _leb_len(b, i):
    n = 0
    while i + n < len(b) and (b[i + n] & 0x80):
        n += 1
    return n + 1


def parse_wasm_body(hexstr: str) -> list:
    b = bytes.fromhex(hexstr.replace("0x", "").strip())
    names = {0x00: "unreachable", 0x01: "nop", 0x02: "block", 0x03: "loop",
             0x04: "if", 0x05: "else", 0x0b: "end", 0x0c: "br", 0x0d: "br_if",
             0x0f: "return", 0x10: "call", 0x11: "call_indirect", 0x24: "global.set"}
    out, i = [], 0
    while i < len(b):
        op = b[i]
        nm = names.get(op, f"0x{op:02x}")
        j = i + 1
        if 0x36 <= op <= 0x3e:                       # stores: memarg (2 LEB)
            nm = "store"; j = j + _leb_len(b, j); j = j + _leb_len(b, j)
        elif 0x28 <= op <= 0x35:                     # loads: memarg (2 LEB)
            nm = "load"; j = j + _leb_len(b, j); j = j + _leb_len(b, j)
        elif op in (0x02, 0x03, 0x04):               # block/loop/if: 1-byte blocktype
            j += 1
        elif op in (0x0c, 0x0d, 0x10, 0x20, 0x21, 0x22, 0x23, 0x24, 0x41, 0x42):
            j = j + _leb_len(b, j)                    # LEB immediate
        elif op == 0x11:                             # call_indirect: 2 LEB
            j = j + _leb_len(b, j); j = j + _leb_len(b, j)
        elif op == 0x43:
            j += 4                                    # f32.const
        elif op == 0x44:
            j += 8                                    # f64.const
        out.append(nm)
        i = j
    return out


def lift_wasm(names: list) -> list:
    tokens = [VINIT]
    ctrl = []  # stack of [kind, escaped] for block/loop/if
    for nm in names:
        if nm in ("block", "loop"):
            ctrl.append([nm, False])
        elif nm == "if":
            ctrl.append(["if", False])
            tokens.append(FSPLIT)
        elif nm in ("return", "br"):
            tokens.append(TANCH if nm == "return" else FSPLIT)
            for c in reversed(ctrl):                  # innermost if escaped early
                if c[0] == "if":
                    c[1] = True
                    break
        elif nm == "end":
            top = ctrl.pop() if ctrl else ["", False]
            if top[0] == "if" and not top[1]:         # merged (no early exit)
                tokens.append(FFUSE)
        elif nm in ("store", "global.set"):
            tokens.append(IFIX)
        elif nm in ("call", "call_indirect"):
            tokens.append(AFWD)
    return tokens


def verdict(word: list):
    # No translation: the twelve glyphs V⊙x lifts to are the same twelve glyphs
    # IMASM16_3_core's opcodes carry (FSPLIT/FFUSE/ENGAGR and FSPLIT3/FFUSE3/EVALI
    # are two names for one glyph each), so the word runs as-is, and the engine's
    # own verdict reasons are already written in the alphabet, not translated
    # into it after the fact.
    tr = Sequence16_3Trace(word, machine=IMASM16_3_Machine())
    tr.run()
    return tr.tri_ancestral_verdict()


# ── native front-end (x86 PE) ────────────────────────────────────────────────
# The same closure law on real machine code. Only the control-flow skeleton is
# needed, so a disassembler (capstone) that gives branches, calls, rets, and
# memory writes is enough — no full semantics. A conditional jump forks, a jump
# target reached from two paths merges, a `mov [mem], _` commits state, a `call`
# is work, a `ret` terminates. Needs `capstone` and `pefile` (pip).


def _imm(op_str: str):
    op_str = op_str.strip()
    try:
        return int(op_str, 16) if op_str.startswith("0x") else None
    except ValueError:
        return None


def _native_func_word(insns) -> list:
    from collections import Counter
    aset = {i.address for i in insns}
    succ = []
    for idx, ins in enumerate(insns):
        mn = ins.mnemonic
        terminates = mn == "jmp" or mn.startswith("ret")
        if not terminates and idx + 1 < len(insns):
            succ.append(insns[idx + 1].address)
        if mn.startswith("j"):                          # jmp or conditional jcc
            t = _imm(ins.op_str)
            if t is not None and t in aset:
                succ.append(t)
    merges = {a for a, c in Counter(succ).items() if c >= 2}
    tokens = [VINIT]
    for ins in insns:
        if ins.address in merges:
            tokens.append(FFUSE)
        mn = ins.mnemonic
        if mn.startswith("j") and mn != "jmp":
            tokens.append(FSPLIT)
        elif mn == "call":
            tokens.append(AFWD)
        elif mn.startswith("ret"):
            tokens.append(TANCH)
        elif mn == "mov" and ins.op_str.split(",", 1)[0].strip().endswith("]"):
            tokens.append(IFIX)
    return tokens


# ── the recompiler: x86 → IMASM, total ─────────────────────────────────────
# The auditor above keeps only the control-flow skeleton, because a verdict does
# not need the arithmetic. A recompile does. Here nothing is noise: every
# decoded instruction lands on exactly one of the twelve axes, so the emitted
# word is the program and not a sketch of it.
#
#   ⊢ entry          ⊣ terminal (ret, int3, ud2, hlt)
#   ∈ conditional branch      ∋ a merge, two paths rejoining
#   > direct call             < unconditional transfer (jmp, tail call)
#   ⊙ INDIRECT call/jmp — the target is data, the structure taking itself as
#     its own object, which is exactly where a linear disassembler goes blind
#   ◻ a write to memory, irreversible
#   ⋈ data movement between named slots (mov, lea, movzx, push, pop, xchg)
#   ⊤ a truth produced (cmp, test)   ⊥ a truth consumed (setcc, cmovcc)
#   ⊞ engagement: everything that computes on values

_ENGAGE = ("add", "sub", "adc", "sbb", "imul", "mul", "idiv", "div", "and",
           "or", "xor", "not", "neg", "inc", "dec", "shl", "shr", "sar", "rol",
           "ror", "sal", "bt", "bsf", "bsr", "popcnt", "cdq", "cqo", "cwde")
_MOVE = ("mov", "movzx", "movsx", "movsxd", "lea", "push", "pop", "xchg",
         "movabs", "movaps", "movdqa", "movdqu", "movups", "movd", "movq")
_TERMINAL = ("ret", "int3", "ud2", "hlt", "iret")


def _writes_memory(ins) -> bool:
    """A destination operand that is a memory reference: the commit."""
    dst = ins.op_str.split(",", 1)[0].strip()
    return dst.endswith("]")


def recompile_native(insns, merges) -> list:
    """Every instruction, one glyph each. Total, order-preserving, nothing
    dropped — this is the program rewritten in the twelve, not a summary."""
    tokens = [VINIT]
    for ins in insns:
        if ins.address in merges:
            tokens.append(FFUSE)
        mn, ops = ins.mnemonic, ins.op_str.strip()
        indirect = _imm(ops) is None and not ops.startswith("0x")
        if mn.startswith("ret") or mn in _TERMINAL:
            tokens.append(TANCH)
        elif mn == "call":
            tokens.append(IMSCRIB if indirect else AFWD)
        elif mn == "jmp":
            tokens.append(IMSCRIB if indirect else AREV)
        elif mn.startswith("j"):
            tokens.append(FSPLIT)
        elif mn.startswith("set") or mn.startswith("cmov"):
            tokens.append(EVALF)
        elif mn in ("cmp", "test", "ucomiss", "ucomisd"):
            tokens.append(EVALT)
        elif _writes_memory(ins):
            tokens.append(IFIX)
        elif mn in _MOVE:
            tokens.append(CLINK)
        elif mn in _ENGAGE:
            tokens.append(ENGAGR)
        else:
            tokens.append(ENGAGR)                          # total: no instruction
    return tokens                                          # leaves the alphabet


def _merges_of(insns) -> set:
    from collections import Counter
    aset = {i.address for i in insns}
    succ = []
    for idx, ins in enumerate(insns):
        if not (ins.mnemonic == "jmp" or ins.mnemonic.startswith("ret")) \
                and idx + 1 < len(insns):
            succ.append(insns[idx + 1].address)
        if ins.mnemonic.startswith("j"):
            t = _imm(ins.op_str)
            if t is not None and t in aset:
                succ.append(t)
    return {a for a, c in Counter(succ).items() if c >= 2}


def recompile_module(path: str):
    """The whole PE recompiled: one IMASM word per function, in address order,
    with the call graph kept as labels so the module is a program and not a
    pile of words. Returns [(label, address, word)]."""
    out = []
    for start, insns in _native_functions(path):
        word = recompile_native(insns, _merges_of(insns))
        out.append((f"f_{start:x}", start, word))
    return out


def emit_imasm(path: str) -> str:
    """The binary as an executable IMASM module: the word in the twelve, plus
    the payload each glyph carries and the initialised data the code reads.
    Runs in imasm_vm.Machine with no reference back to the original."""
    import imasm_module
    return imasm_module.emit(path)


def emit_word(path: str) -> str:
    """The structure alone: one word per function, glyphs and nothing else.
    This is what the measurements are taken over; it does not execute."""
    mod = recompile_module(path)
    total = sum(len(w) for _, _, w in mod)
    lines = [f"; ⊙ {path}", f"; {len(mod)} words   {total} glyphs"]
    for _, addr, word in mod:
        lines.append(f"0x{addr:x}")
        lines.append(glyphs(word))
    return "\n".join(lines) + "\n"


def _elf_composition(path: str) -> dict:
    import os
    _, secs, _ = _elf_sections(path)
    code = sum(len(d) for d, _ in secs)
    return {"size": os.path.getsize(path), "code": code, "overlay": 0, "sig": ""}


def _composition(path: str) -> dict:
    with open(path, "rb") as fh:
        magic = fh.read(4)
    return _elf_composition(path) if magic == b"\x7fELF" else _pe_composition(path)


# ── the genetics lane ────────────────────────────────────────────────────
# Not an analogy. The Imscriber's Guide states the identity plainly: the twelve
# operations and the twelve axes are ONE alphabet, "read as an operation or as
# an axis according to where it stands." The chain from a nucleotide to a glyph
# is proved in Lean and parsed into genetic_table.py by its generator — G is B
# because guanine wobble-pairs with both C and U, C is T because it pairs only
# with G, A is F, U is N; codons carry to amino acids by the genetic code; and
# exactly twelve amino acids are promoted, bijecting the twelve axes.
#
# So a gene is already a word. This lane reads it, and the same SIXTEEN_3 engine
# that verdicts x86 verdicts the transcript.


def lift_rna(seq: str):
    """An RNA or DNA sequence → (word, reading). Reads from the first AUG in
    frame, stops at a stop codon, and emits a glyph only where the codon names
    a promoted amino acid; the ground layer activates no axis and is silent,
    which is a fact of the code and not a gap in the lift."""
    import genetic_table as gt
    seq = "".join(c for c in seq.upper() if c in "ACGTU").replace("T", "U")
    start = seq.find("AUG")
    if start < 0:
        start = 0
    word, reading, stopped = [], [], None
    for k in range(start, len(seq) - 2, 3):
        codon = seq[k:k + 3]
        kind, val = gt.CODON.get(codon, (None, None))
        if kind == "stop":
            stopped = val
            break
        if val in gt.AA_GLYPH:
            glyph, family, slot = gt.AA_GLYPH[val]
            word.append(glyph)
            reading.append((codon, val, glyph, family))
    return word, reading, stopped


def _pe_composition(path: str) -> dict:
    """Where the bytes are: how much is code the lane reads vs an appended
    overlay (installer payload, resources) that is data, not program."""
    import os
    import pefile
    pe = pefile.PE(path, fast_load=True)
    size = os.path.getsize(path)
    secs = sum(s.SizeOfRawData for s in pe.sections)
    code = sum(s.SizeOfRawData for s in pe.sections
               if s.Characteristics & 0x20000000)
    head = open(path, "rb").read(2_000_000)
    sig = next((n for m, n in ((b"Nullsoft", "NSIS"), (b"Inno Setup", "Inno"),
                               (b"WiX", "WiX")) if m in head), "")
    return {"size": size, "code": code, "overlay": max(0, size - secs), "sig": sig}


def _pe_sections(path: str):
    """(capstone mode, executable [(bytes, vaddr)], entry vaddr) for a PE."""
    import capstone
    import pefile
    pe = pefile.PE(path, fast_load=True)
    mode = capstone.CS_MODE_64 if pe.FILE_HEADER.Machine == 0x8664 else capstone.CS_MODE_32
    base = pe.OPTIONAL_HEADER.ImageBase
    secs = [(s.get_data(), base + s.VirtualAddress) for s in pe.sections
            if s.Characteristics & 0x20000000]          # IMAGE_SCN_MEM_EXECUTE
    return mode, secs, base + pe.OPTIONAL_HEADER.AddressOfEntryPoint


def _elf_sections(path: str):
    """The same three things for an ELF. Only the header walk differs from PE —
    the lift downstream cannot tell which container it came from."""
    import capstone
    import struct
    raw = open(path, "rb").read()
    is64 = raw[4] == 2
    end = "<" if raw[5] == 1 else ">"
    if is64:
        e_entry, _phoff, e_shoff = struct.unpack_from(end + "QQQ", raw, 24)
        e_shentsize, e_shnum = struct.unpack_from(end + "HH", raw, 58)
    else:
        e_entry, _phoff, e_shoff = struct.unpack_from(end + "III", raw, 24)
        e_shentsize, e_shnum = struct.unpack_from(end + "HH", raw, 46)
    secs = []
    for k in range(e_shnum):
        off = e_shoff + k * e_shentsize
        if is64:
            _, sh_type, sh_flags, sh_addr, sh_off, sh_size = \
                struct.unpack_from(end + "IIQQQQ", raw, off)
        else:
            _, sh_type, sh_flags, sh_addr, sh_off, sh_size = \
                struct.unpack_from(end + "IIIIII", raw, off)
        if sh_type == 1 and sh_flags & 0x4 and sh_size:  # PROGBITS + EXECINSTR
            secs.append((raw[sh_off:sh_off + sh_size], sh_addr))
    mode = capstone.CS_MODE_64 if is64 else capstone.CS_MODE_32
    return mode, secs, e_entry


def _native_functions(path: str):
    """Disassemble a binary's executable sections and split the stream into
    functions at the entry point and at every direct call target. PE and ELF
    both land here; only the container parse above differs. Linear sweep with a
    call-target split, not recursive descent, so padding between functions can
    produce a little noise. Yields (start_address, [insns])."""
    import capstone
    with open(path, "rb") as fh:
        magic = fh.read(4)
    parse = _elf_sections if magic == b"\x7fELF" else _pe_sections
    mode, secs, entry = parse(path)
    md = capstone.Cs(capstone.CS_ARCH_X86, mode)
    insns = []
    for data, vaddr in secs:
        insns.extend(md.disasm(data, vaddr))
    if not insns:
        return
    aset = {i.address for i in insns}
    idx_of = {i.address: k for k, i in enumerate(insns)}
    starts = {insns[0].address}
    if entry in aset:
        starts.add(entry)
    for ins in insns:
        if ins.mnemonic == "call":
            t = _imm(ins.op_str)
            if t is not None and t in aset:
                starts.add(t)
    starts = sorted(starts)
    for si, start in enumerate(starts):
        end_a = starts[si + 1] if si + 1 < len(starts) else None
        k, func = idx_of[start], []
        while k < len(insns) and (end_a is None or insns[k].address < end_a):
            func.append(insns[k]); k += 1
        yield start, func


def scan_native(path: str):
    """The auditor lane: skeleton lift + verdict per function.
    Returns (addr, verdict, why, word, n_insns)."""
    results = []
    for start, func in _native_functions(path):
        word = _native_func_word(func)
        v, why = verdict(word)
        results.append((f"0x{start:x}", v, why, word, len(func)))
    return results


def scan_module(path: str):
    """Lift + verdict every top-level function in a .py file."""
    spec = importlib.util.spec_from_file_location("_scan_target", path)
    if spec is None or spec.loader is None:
        raise SystemExit(f"vox: '{path}' is not an importable Python module. "
                         "For a native binary use no flag (auto-detected), for "
                         "EVM/WASM use --evm/--wasm.")
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    results = []
    for name in dir(mod):
        obj = getattr(mod, name)
        if callable(obj) and getattr(obj, "__code__", None) is not None \
                and obj.__module__ == "_scan_target":
            word = lift_function(obj)
            v, why = verdict(word)
            results.append((name, v, why, word))
    return results


def _selftest():
    # EVM: state commit inside an unmerged branch (vuln) vs a guard whose paths
    # rejoin before the commit (safe). The kernel decides.
    #   vuln: PUSH1 1, PUSH1 7, JUMPI, SSTORE, STOP, JUMPDEST(7), STOP
    #   safe: PUSH1 1, PUSH1 6, JUMPI, SLOAD, JUMPDEST(6), SSTORE, STOP
    vuln = verdict(lift_evm(parse_evm("600160075755005b00")))[0]
    safe = verdict(lift_evm(parse_evm("6001600657545b5500")))[0]
    print(f"EVM reentrant (commit in unmerged branch): {vuln}  (expect B)")
    print(f"EVM guarded  (paths merge before commit):  {safe}  (expect T)")
    assert vuln == "B" and safe == "T", f"EVM selftest failed: vuln={vuln} safe={safe}"
    # WASM: if{ store; return } (early return before the if's merge) vs
    #       if{ call } end; store (merges before the commit).
    wvuln = verdict(lift_wasm(parse_wasm_body("20000440410141003602000f0b0b")))[0]
    wsafe = verdict(lift_wasm(parse_wasm_body("2000044010000b410041003602000b")))[0]
    print(f"WASM reentrant (commit + return in if-branch): {wvuln}  (expect B)")
    print(f"WASM guarded  (if merges before the commit):   {wsafe}  (expect T)")
    assert wvuln == "B" and wsafe == "T", f"WASM selftest failed: vuln={wvuln} safe={wsafe}"
    print("selftest OK: the closure law holds on EVM AND WASM bytecode.")


def main():
    ap = argparse.ArgumentParser(description="Control-flow closure auditor: "
                                 "lift bytecode to IMASM, verdict μ∘δ over SIXTEEN_3.")
    ap.add_argument("target", nargs="?", help="path to a .py file to scan")
    ap.add_argument("--evm", metavar="HEX", help="scan an EVM bytecode hex string")
    ap.add_argument("--wasm", metavar="HEX", help="scan a WASM function-body hex string")
    ap.add_argument("--rna", metavar="SEQ",
                    help="lift a coding sequence (RNA or DNA) to the twelve and "
                         "verdict the transcript")
    ap.add_argument("--imasm", metavar="OUT", nargs="?", const="-",
                    help="recompile a native binary into an EXECUTABLE IMASM "
                         "module and write it to OUT, or to stdout")
    ap.add_argument("--word", metavar="OUT", nargs="?", const="-",
                    help="emit the structure alone: one glyph word per "
                         "function, which does not execute")
    ap.add_argument("--run", metavar="SYMBOL",
                    help="recompile, then RUN a function in the IMASM machine; "
                         "integer arguments follow as --args")
    ap.add_argument("--args", metavar="N,N", default="",
                    help="comma-separated integer arguments for --run")
    ap.add_argument("--selftest", action="store_true", help="run the EVM+WASM vuln/safe self-test")
    args = ap.parse_args()
    if args.selftest:
        _selftest(); return
    if args.rna:
        word, reading, stopped = lift_rna(args.rna)
        if not word:
            ap.error("no promoted codon in that sequence")
        v, why = verdict(word)
        print(f"{'CODON':<8}{'AA':<6}{'AXIS':<16}GLYPH")
        for codon, aa, glyph, family in reading:
            print(f"{codon:<8}{aa:<6}{family:<16}{glyph}")
        print(f"\nword     {glyphs(word)}")
        print(f"stop     {stopped or '(none: sequence ran out before a stop)'}")
        mark = "   <-- " + why if v == "B" else ""
        print(f"verdict  {v}{mark}")
        return

    for isa, hexs, lift, parse in (("EVM", args.evm, lift_evm, parse_evm),
                                   ("WASM", args.wasm, lift_wasm, parse_wasm_body)):
        if hexs:
            word = lift(parse(hexs))
            v, why = verdict(word)
            mark = "   <-- FINDING (fork open across commit): " + why if v == "B" else ""
            print(f"{isa:<24}{v:<4}{glyphs(word)}{mark}")
            return
    if not args.target:
        ap.error("give a target (.py, or a PE/ELF binary), or --evm/--wasm HEX, or --selftest")

    with open(args.target, "rb") as fh:
        magic = fh.read(4)

    native = magic[:2] == b"MZ" or magic == b"\x7fELF"

    if args.run:
        if not native:
            ap.error("--run needs a native binary (PE or ELF)")
        import subprocess

        import imasm_vm
        text = emit_imasm(args.target)
        out = subprocess.run(["nm", "-D", "--defined-only", args.target],
                             capture_output=True, text=True).stdout
        syms = {p[2]: int(p[0], 16) for p in
                (l.split() for l in out.splitlines())
                if len(p) == 3 and p[1] in "Tt"}
        if args.run not in syms:
            ap.error(f"no symbol '{args.run}' in {args.target}")
        m = imasm_vm.Machine(text)
        argv = [int(a, 0) for a in args.args.split(",") if a.strip()]
        result = m.call(syms[args.run], *argv)
        print(f"{args.run}({', '.join(map(str, argv))}) = {result}"
              f"   [{m.steps} steps in the twelve]")
        return

    for flag, fn, what in ((args.imasm, emit_imasm, "executable module"),
                           (args.word, emit_word, "words")):
        if not flag:
            continue
        if not native:
            ap.error("recompiling needs a native binary (PE or ELF)")
        text = fn(args.target)
        if flag == "-":
            print(text, end="")
        else:
            with open(flag, "w") as fh:
                fh.write(text)
            n = sum(1 for ln in text.splitlines()
                    if ln.startswith("@" if fn is emit_imasm else "0x"))
            print(f"recompiled → {flag}   {n} {what}")
        return

    if magic[:2] == b"MZ" or magic == b"\x7fELF":         # native binary lane
        from collections import Counter
        comp = _composition(args.target)
        print(f"file {comp['size']:,} B  |  code {comp['code']:,} B (read)  |  "
              f"overlay {comp['overlay']:,} B (not code)"
              + (f"  |  {comp['sig']} installer" if comp["sig"] else ""))
        if comp["overlay"] > 4 * comp["code"] and comp["code"]:
            print("  note: this file is mostly an appended payload, not program. V⊙x read"
                  " the stub; extract it (e.g. 7z x) to scan the real code inside.")
        rows = scan_native(args.target)
        dist = Counter(v for _, v, _, _, _ in rows)
        print(f"native PE: {len(rows)} functions   verdicts {dict(dist)}")
        findings = [(a, w) for a, v, _, w, n in rows if v == "B" and n >= 3]
        print(f"{len(findings)} B-finding(s): fork(s) holding open across a commit/return.")
        for a, w in findings:
            print(f"  {a:<12} {glyphs(w)}")
        return
    print(f"{'FUNCTION':<24}{'B4':<4}{'WORD'}")            # Python lane
    findings = 0
    for name, v, why, word in scan_module(args.target):
        # B is the finding: a fork held OPEN across a commit/return (reentrancy,
        # unhandled path, leak). N is a linear routine that never forked — clean,
        # nothing to weigh. T is a closed control flow.
        mark = ""
        if v == "B":
            mark = "   <-- FINDING (fork open across commit/return): " + why
            findings += 1
        print(f"{name:<24}{v:<4}{glyphs(word)}{mark}")
    print(f"\n{findings} finding(s): fork(s) holding open across a commit/return.")


if __name__ == "__main__":
    main()
