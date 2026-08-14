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
(auto-detected by the MZ magic; no third-party package needed). The vulnerable
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
from vox_x86 import X86_OP_IMM, X86_OP_MEM, X86_OP_REG
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
VINIT, TANCH, AFWD, AREV = "⊢", "⊣", "≻", "≺"
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
# needed, so a decoder that gives branches, calls, rets, and memory writes is
# enough — no full semantics. A conditional jump forks, a jump target reached
# from two paths merges, a `mov [mem], _` commits state, a `call` is work, a
# `ret` terminates. The decoder is V⊙x's own (`vox_decode`) and the PE
# reader is its own too (`vox_pe`), so the native lane needs nothing installed.
# capstone can be swapped back in with VOX_CAPSTONE=1 to cross-check the two
# decoders against each other.


def _imm(op_str: str):
    op_str = op_str.strip()
    try:
        return int(op_str, 16) if op_str.startswith("0x") else None
    except ValueError:
        return None


# Mnemonic prefixes that capstone attaches but don't change classification
_MNEMONIC_PREFIXES = ("notrack ", "lock ", "bnd ", "rep ", "repe ", "repne ", "data16 ")


def _mnemonic(ins) -> str:
    """Strip capstone mnemonic prefixes so classification matches imasm_module.
    A `notrack jmp` is a jmp; a `lock add` is an add — the prefix changes
    guards, not which of the twelve it is."""
    mn = ins.mnemonic
    for p in _MNEMONIC_PREFIXES:
        if mn.startswith(p):
            mn = mn[len(p):]
    return mn


def _native_func_word(insns) -> list:
    from collections import Counter
    aset = {i.address for i in insns}
    succ = []
    for idx, ins in enumerate(insns):
        mn = _mnemonic(ins)
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
        mn = _mnemonic(ins)
        if mn.startswith("j") and mn != "jmp":
            tokens.append(FSPLIT)
        elif mn == "jmp":
            # An unconditional jump is a transfer, and the skeleton has to say
            # so. Dropping it while still emitting the ∋ at the address it
            # lands on spends a fuse that nothing opened, and the word comes
            # out ill-typed for a reason that belongs to this lifter rather
            # than to the program it is reading.
            tokens.append(AREV)
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
_MOVE = ("mov", "movzx", "movsx", "movsxd", "leave", "push", "pop", "xchg",
         "movabs", "movaps", "movdqa", "movdqu", "movups", "movd", "movq")
_TERMINAL = ("ret", "int3", "ud2", "hlt", "iret")


def _writes_memory(ins) -> bool:
    """A destination operand that is a memory reference: the commit."""
    dst = ins.op_str.split(",", 1)[0].strip()
    return dst.endswith("]")


_INERT = ("nop", "endbr64", "endbr32")     # never a commit, whatever they address
_TRUTH = ("cmp", "test", "ucomiss", "ucomisd", "comiss", "comisd")


def classify(ins) -> str:
    """Which of the twelve one instruction is.

    This is the only place that decision is made. `recompile_native` reads the
    word off it and `imasm_module.encode` builds its executable line off it, so
    the auditor's word and the module's word cannot drift apart — they are the
    same call. They did drift, before: the module read a syscall as ⊙ and the
    auditor read it as ⊞, and a round trip through the module text could never
    return the word the auditor produced.

    The order is the reading order. A terminal is a terminal whatever else it
    touches; a comparison is truth-making even when its destination is memory;
    an inert instruction never commits however it addresses memory.
    """
    mn, ops = _mnemonic(ins), ins.operands
    direct = bool(ops) and ops[0].type == X86_OP_IMM

    if mn.startswith("ret") or mn in _TERMINAL:
        return TANCH
    # A syscall's target is chosen by a register, not written in the
    # instruction: the same shape as an indirect transfer, to the kernel
    # instead of to the program's own code.
    if mn == "syscall" or mn == "sysenter" or (mn == "int" and ops
                                               and ops[0].type == X86_OP_IMM
                                               and ops[0].imm == 0x80):
        return IMSCRIB
    if mn == "call":
        return AFWD if direct else IMSCRIB
    if mn == "jmp":
        return AREV if direct else IMSCRIB
    if mn.startswith("j") or mn == "loop":
        return FSPLIT
    if mn.startswith("set") or mn.startswith("cmov"):
        return EVALF
    if mn in _TRUTH:
        return EVALT
    if mn in _INERT:
        return ENGAGR
    # The commit is a memory destination. `lea` computes an address without
    # touching it and `push` reads its operand, so neither commits.
    if ops and ops[0].type == X86_OP_MEM and mn not in ("lea", "push"):
        return IFIX
    if mn in _MOVE:
        return CLINK
    return ENGAGR                                          # total: no instruction
                                                           # leaves the alphabet


def recompile_native(insns, merges) -> list:
    """Every instruction, one glyph each. Total, order-preserving, nothing
    dropped — this is the program rewritten in the twelve, not a summary."""
    tokens = [VINIT]
    for ins in insns:
        if ins.address in merges:
            tokens.append(FFUSE)
        tokens.append(classify(ins))
    return tokens


def _merges_of(insns) -> set:
    from collections import Counter
    aset = {i.address for i in insns}
    succ = []
    for idx, ins in enumerate(insns):
        mn = _mnemonic(ins)
        if not (mn == "jmp" or mn.startswith("ret")) \
                and idx + 1 < len(insns):
            succ.append(insns[idx + 1].address)
        if mn.startswith("j"):
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


# A packed/installer binary is data on the outside, program on the inside —
# V⊙x reads what's on disk, so an appended installer payload reads as one
# giant non-code overlay. Rather than only naming the fix, attempt it: NSIS,
# Inno, and WiX are all 7z-readable containers, so if `7z` is on the host,
# actually extract and report what real executables came out, instead of
# advice to go run a command by hand.
def _try_extract_overlay(path: str):
    """Extract path with 7z into a sibling directory and return the list of
    PE/ELF executables found inside, or None if extraction wasn't possible
    (no 7z, or 7z found nothing it recognised)."""
    import shutil
    import subprocess
    from pathlib import Path

    sevenzip = shutil.which("7z") or shutil.which("7za") or shutil.which("7zr")
    if not sevenzip:
        return None
    out_dir = Path(path).with_suffix("")
    out_dir = out_dir.parent / (out_dir.name + "_extracted")
    out_dir.mkdir(exist_ok=True)
    try:
        subprocess.run([sevenzip, "x", "-y", f"-o{out_dir}", path],
                       capture_output=True, timeout=120)
    except (OSError, subprocess.TimeoutExpired):
        return None
    found = []
    for p in out_dir.rglob("*"):
        if not p.is_file():
            continue
        try:
            head = p.open("rb").read(4)
        except OSError:
            continue
        if head[:2] == b"MZ" or head == b"\x7fELF":
            found.append(str(p))
    return found if found else None


def _pe_composition(path: str) -> dict:
    """Where the bytes are: how much is code the lane reads vs an appended
    overlay (installer payload, resources) that is data, not program."""
    import os
    import vox_pe
    pe = vox_pe.PE(path)
    size = os.path.getsize(path)
    secs = pe.total_raw()
    code = pe.total_code()
    head = open(path, "rb").read(2_000_000)
    sig = next((n for m, n in ((b"Nullsoft", "NSIS"), (b"Inno Setup", "Inno"),
                               (b"WiX", "WiX")) if m in head), "")
    return {"size": size, "code": code, "overlay": max(0, size - secs), "sig": sig}


def _pe_sections(path: str):
    """(decoder mode, executable [(bytes, vaddr)], entry vaddr) for a PE."""
    import vox_x86 as capstone
    import vox_pe
    pe = vox_pe.PE(path)
    mode = capstone.CS_MODE_64 if pe.is64 else capstone.CS_MODE_32
    return mode, pe.executable_sections(), pe.entry


def _elf_sections(path: str):
    """The same three things for an ELF. Only the header walk differs from PE —
    the lift downstream cannot tell which container it came from."""
    import vox_x86 as capstone
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


_R_X86_64_JUMP_SLOT = 7   # ELF x86-64 relocation type: a PLT/GOT entry


def _elf_plt_targets(path: str) -> dict:
    """(GOT slot address -> imported symbol name) for every lazily-bound PLT
    entry, read straight from the section headers, the dynamic symbol table,
    and the relocations — the same three structures the real dynamic linker
    reads, not a guess at layout. 32-bit ELF only has 32-bit relocations
    (Elf32_Rel, no addend, different entsize) and isn't handled; this is
    x86-64 only, matching the one Capstone mode Vox already assumes."""
    import struct
    raw = open(path, "rb").read()
    if raw[4] != 2:                     # ELFCLASS64 only
        return {}
    end = "<" if raw[5] == 1 else ">"
    e_shoff, = struct.unpack_from(end + "Q", raw, 40)
    e_shentsize, e_shnum = struct.unpack_from(end + "HH", raw, 58)
    secs = []
    for k in range(e_shnum):
        off = e_shoff + k * e_shentsize
        sh_type, sh_flags, sh_addr, sh_off, sh_size, sh_link, sh_info = \
            struct.unpack_from(end + "IQQQQII", raw, off + 4)
        secs.append({"type": sh_type, "off": sh_off, "size": sh_size, "link": sh_link})

    dynsym = next((s for s in secs if s["type"] == 11), None)   # SHT_DYNSYM
    if dynsym is None:
        return {}
    dynstr = secs[dynsym["link"]]

    def sym_name(idx):
        st_name, = struct.unpack_from(end + "I", raw, dynsym["off"] + idx * 24)
        end_i = raw.index(b"\x00", dynstr["off"] + st_name)
        return raw[dynstr["off"] + st_name:end_i].decode("ascii", "replace")

    got_to_name = {}
    for s in secs:
        if s["type"] != 4:              # SHT_RELA; lazy PLT binding is always RELA on x86-64
            continue
        for k in range(s["size"] // 24):
            r_offset, r_info = struct.unpack_from(end + "QQ", raw, s["off"] + k * 24)
            if (r_info & 0xffffffff) == _R_X86_64_JUMP_SLOT:
                got_to_name[r_offset] = sym_name(r_info >> 32)
    return got_to_name


_STT_FUNC = 2


def _elf_defined_func_addrs(path: str) -> list:
    """Addresses of every defined (st_shndx != 0), function-typed dynamic
    symbol — a shared object's exported functions. A library's own entry
    point calls essentially none of these; they're called from outside, by
    whoever loads it. Descent that starts only at the entry point never
    reaches them, so _native_functions seeds its worklist with this list too.
    64-bit ELF only, matching _elf_plt_targets."""
    import struct
    raw = open(path, "rb").read()
    if raw[4] != 2:
        return []
    end = "<" if raw[5] == 1 else ">"
    e_shoff, = struct.unpack_from(end + "Q", raw, 40)
    e_shentsize, e_shnum = struct.unpack_from(end + "HH", raw, 58)
    dynsym = None
    for k in range(e_shnum):
        off = e_shoff + k * e_shentsize
        sh_type, _flags, _addr, sh_off, sh_size, _link, _info = \
            struct.unpack_from(end + "IQQQQII", raw, off + 4)
        if sh_type == 11:                # SHT_DYNSYM
            dynsym = (sh_off, sh_size)
            break
    if dynsym is None:
        return []
    off, size = dynsym
    addrs = []
    for k in range(size // 24):
        st_info, = struct.unpack_from(end + "B", raw, off + k * 24 + 4)
        st_shndx, = struct.unpack_from(end + "H", raw, off + k * 24 + 6)
        st_value, = struct.unpack_from(end + "Q", raw, off + k * 24 + 8)
        if (st_info & 0xf) == _STT_FUNC and st_shndx != 0 and st_value:
            addrs.append(st_value)
    return addrs


def _plt_stub_map(path: str, secs, mode) -> dict:
    """(PLT stub entry address -> imported symbol name). A stub is `[endbr64]
    jmp *disp(%rip)` (or the CET `.plt.sec` twin, same shape); the jump's
    real target is a GOT slot, computed the same way the CPU would — this
    instruction's own end address plus its displacement — and matched against
    _elf_plt_targets. Nothing here executes the stub; it only reads what the
    stub would jump through, to name it."""
    import vox_x86 as capstone
    from vox_x86 import X86_OP_MEM

    got_to_name = _elf_plt_targets(path)
    if not got_to_name:
        return {}
    md = capstone.Cs(capstone.CS_ARCH_X86, mode)
    md.detail = True
    stubs = {}
    for data, vaddr in secs:
        insns = list(md.disasm(data, vaddr))
        for k, ins in enumerate(insns):
            # a CET stub's jump carries a `bnd` prefix capstone leaves in the
            # mnemonic text (the same prefixes imasm_module._mnemonic strips)
            mn = _mnemonic(ins)
            for p in ("bnd ", "notrack "):
                if mn.startswith(p):
                    mn = mn[len(p):]
            if mn != "jmp" or not ins.operands:
                continue
            op = ins.operands[0]
            if op.type != X86_OP_MEM or ins.reg_name(op.mem.base) != "rip":
                continue
            got_addr = ins.address + ins.size + op.mem.disp
            name = got_to_name.get(got_addr)
            if name is None:
                continue
            # the stub's entry is this jmp, or the endbr64 immediately before it
            entry = insns[k - 1].address if k > 0 and insns[k - 1].mnemonic == "endbr64" \
                and insns[k - 1].address + insns[k - 1].size == ins.address else ins.address
            stubs[entry] = name
    return stubs


def _native_functions(path: str):
    """Disassemble a binary by recursive descent: walk forward from the entry
    point and every direct call target, decoding one instruction at a time and
    following every direct jump and call as a control-flow edge. A conditional
    jump's fall-through is a second edge; an unconditional jump has none. Only
    bytes actually reached this way are decoded, so padding, jump tables, and
    other data sitting between functions are never walked into and misread as
    code — the honest-edges gap the linear-sweep-plus-call-split version had.

    An indirect call or jump (⊙: the target is data, structurally where a
    disassembler goes blind) can't be followed statically, so anything reached
    ONLY through one — a switch's jump-table arms, say — would be invisible to
    descent alone. The fallback below covers exactly that: whatever descent
    never reaches, a linear sweep of the remaining bytes still finds, grouped
    into synthetic functions of their own, so total coverage matches what the
    old sweep read while attribution for everything reachable by real control
    flow is the descent's, not an after-the-fact split of one long stream.

    PE and ELF both land here; only the container parse above differs. Yields
    (start_address, [insns]), descent-discovered functions first in discovery
    order, then any fallback-swept leftovers.
    """
    import vox_x86 as capstone
    with open(path, "rb") as fh:
        magic = fh.read(4)
    parse = _elf_sections if magic == b"\x7fELF" else _pe_sections
    mode, secs, entry = parse(path)
    md = capstone.Cs(capstone.CS_ARCH_X86, mode)
    md.detail = True   # emit_imasm's recompiler needs real operands, not just text
    if not secs:
        return

    def bytes_at(addr):
        for data, vaddr in secs:
            if vaddr <= addr < vaddr + len(data):
                return data[addr - vaddr:]
        return None

    def decode_one(addr):
        b = bytes_at(addr)
        if not b:
            return None
        for ins in md.disasm(b, addr, count=1):
            return ins
        return None

    _TERM = ("hlt", "ud2", "int3", "iret")
    entry_start = entry if bytes_at(entry) is not None else secs[0][1]
    # A shared object's own entry point calls almost none of its exported
    # functions — those are called from outside, by whoever loads it — so
    # descent from the entry point alone misses them. Seed every defined
    # function symbol too, not just _start/_init.
    seeds = [entry_start] + (_elf_defined_func_addrs(path) if magic == b"\x7fELF" else [])
    seen_funcs = set()
    func_queue = list(dict.fromkeys(a for a in seeds if bytes_at(a) is not None))
    covered = {}     # address -> Instruction, across every function, for the fallback pass

    while func_queue:
        fstart = func_queue.pop(0)
        if fstart in seen_funcs or fstart in covered or bytes_at(fstart) is None:
            continue
        seen_funcs.add(fstart)

        visited, addr_queue, by_addr = set(), [fstart], {}
        while addr_queue:
            addr = addr_queue.pop(0)
            if addr in visited or addr in covered:
                continue
            ins = decode_one(addr)
            if ins is None:
                continue
            visited.add(addr)
            by_addr[addr] = ins
            mn = _mnemonic(ins)
            terminates = mn.startswith("ret") or mn in _TERM
            if mn == "call":
                t = _imm(ins.op_str)
                if t is not None and t not in seen_funcs:
                    func_queue.append(t)
                if not terminates:
                    addr_queue.append(addr + ins.size)   # the call returns here
            elif mn.startswith("j"):
                t = _imm(ins.op_str)
                if t is not None:
                    addr_queue.append(t)
                if mn != "jmp":                          # conditional: both edges
                    addr_queue.append(addr + ins.size)
            elif not terminates:
                addr_queue.append(addr + ins.size)

        if by_addr:
            covered.update(by_addr)
            yield fstart, [by_addr[a] for a in sorted(by_addr)]

    # Fallback: sweep every section for anything descent never reached — code
    # reachable only through an indirect call/jump — and group contiguous runs
    # of it into synthetic functions, exactly as before for that leftover.
    leftover = []
    for data, vaddr in secs:
        for ins in md.disasm(data, vaddr):
            if ins.address not in covered:
                leftover.append(ins)
    if leftover:
        leftover.sort(key=lambda i: i.address)
        # A contiguous run of unreached bytes is not one function. It is every
        # function descent could not reach, laid end to end, and glueing them
        # together produces words that are not words: a ∋ belonging to the next
        # function fusing into the ⊣ of the previous one, with no ∈ anywhere to
        # pair it. The verdict engine reports that correctly as ill-typed, but
        # the ill-typedness is the sweep's, not the program's.
        #
        # A function boundary is a terminal, then padding, then a body. All
        # three are needed: padding alone is not a boundary, because both
        # compilers also pad for alignment inside a body, and cutting there
        # shatters one function into dozens.
        _PAD = ("int3", "nop")

        runs = []
        run = [leftover[0]]
        saw_terminal = False
        in_pad = False
        for ins in leftover[1:]:
            prev = run[-1]
            pm, im = _mnemonic(prev), _mnemonic(ins)
            gap = ins.address != prev.address + prev.size
            if pm in _PAD:
                in_pad = True
            elif pm.startswith("ret") or pm in _TERM or pm == "jmp":
                saw_terminal = True
            else:
                saw_terminal = False
                in_pad = False
            if gap or (in_pad and saw_terminal and im not in _PAD):
                runs.append(run)
                run = [ins]
                saw_terminal = in_pad = False
            else:
                run.append(ins)
        runs.append(run)

        for r in runs:
            # A run that is nothing but padding is not a function either.
            if all(_mnemonic(i) in _PAD for i in r):
                continue
            yield r[0].address, r


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
        try:
            result = m.call(syms[args.run], *argv)
            print(f"{args.run}({', '.join(map(str, argv))}) = {result}"
                  f"   [{m.steps} steps in the twelve]")
        except imasm_vm.SysExit as e:
            print(f"{args.run}({', '.join(map(str, argv))}) called exit({e.code})"
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
                  " the stub; attempting extraction to reach the real code inside.")
            found = _try_extract_overlay(args.target)
            if found is None:
                print("  extraction found nothing usable (no 7z on this host, or the"
                      " payload isn't a 7z-readable container) — extract it by hand"
                      " (e.g. 7z x) to scan the real code inside.")
            else:
                print(f"  extracted {len(found)} executable(s):")
                for f in found[:10]:
                    print(f"    {f}")
                if len(found) > 10:
                    print(f"    ... and {len(found) - 10} more")
                print("  point V⊙x at one of those to scan the real code.")
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
