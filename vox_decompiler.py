#!/usr/bin/env python3
"""V⊙x decompiler — the missing recompose stage of the circular pipeline.

Where imasm_module.emit() turns x86 bytecode into an executable IMASM module
text (glyph + payload), this module turns the IMASM module text back into
an IMASM word — the glyph sequence that vox.py would lift.

The μ∘δ=id law for the circular pipeline is:

    lift(decompile(emit(binary))) == lift(recompile_module(binary))

i.e. lifting the decompiled-and-recompiled word must return the same word.
This is the round-trip property: structure is preserved exactly.

The IMASM module text format (from imasm_module.emit) is:

    ; ⊙ <source path>
    ; entry 0x{entry_addr}
    ={addr}\t{hex_data}          # data section (not code)
    @0x{addr}
    {glyph}\t{field}\t{field}...  # instruction lines (one per address)
    ; ...

Key format details:
- Each instruction has its own @addr label
- Function boundaries are detected by matching addresses against
  vox._native_functions() function start addresses
- ⊢ (ENTRY/VINIT) is NOT in the IMASM text — it must be prepended at
  each function entry point detected from address analysis
- ∋ (FUSE) appears as a bare line with no tab-separated fields (merge marker)
- = lines are data sections, not code
- ; lines are comments

Mapping note: The glyphs in imasm_module.encode() and vox.recompile_native()
are the SAME glyphs for the SAME x86 instructions. There are no "simple" vs
"complex" rules — both classify to the same twelve glyphs. The only difference
is that vox.recompile_native prepends ⊢ at function entries, while the IMASM
text format uses @addr labels for every instruction and ⊢ does not appear.
"""
import re
import sys

# The twelve IMASM opcodes as glyphs
# Entry/Terminal
ENTRY, TERM, SPLIT, FUSE = "⊢", "⊣", "∈", "∋"
# Calls/xfers
CALL, XFER, INDIRECT, COMMIT = ">", "<", "⊙", "◻"
# Data movement/truth/engagement
LINK, TRUTH, CONSUME, ENGAGE = "⋈", "⊤", "⊥", "⊞"


def parse_module_text(text: str) -> dict:
    """Parse IMASM module text into a structured representation.

    Returns:
        {
            "entry": int or None,
            "data": {addr: bytes},
            "instructions": [(addr, glyph, fields), ...],
            "addresses": [addr, ...] sorted,
        }

    Instructions are ordered as they appear in the text (address order).
    """
    entry = None
    data = {}
    instructions = []
    current_addr = None

    for line in text.splitlines():
        line = line.strip()
        if not line:
            continue
        if line.startswith(";"):
            m = re.match(r"; entry (0x[0-9a-f]+)", line)
            if m:
                entry = int(m.group(1), 16)
            continue
        if line.startswith("="):
            # Data section: =addr\thex_data
            parts = line[1:].split("\t", 1)
            if len(parts) == 2:
                at = int(parts[0], 16)
                raw = bytes.fromhex(parts[1])
                data[at] = raw
            continue
        if line.startswith("@"):
            current_addr = int(line[1:], 16)
            continue
        if line and current_addr is not None:
            # Glyph line: glyph optionally followed by tab-separated fields
            parts = line.split("\t")
            glyph = parts[0]
            fields = parts[1:]
            instructions.append((current_addr, glyph, fields))

    addresses = sorted(set(addr for addr, _, _ in instructions))
    return {
        "entry": entry,
        "data": data,
        "instructions": instructions,
        "addresses": addresses,
    }


def _detect_function_entries(instructions: list, addresses: list = None, binary_path: str = None) -> set:
    """Detect function entry addresses with high fidelity.

    Strategy (ordered by precision):
      1. If binary_path is given, query vox._native_functions for the authoritative
         function start addresses — this is the same recursive-descent that
         emitted the module text, so every start address is exact.
      2. Fallback: use the gap heuristic (address diff > 0x100) for cases where
         the binary is not available (e.g. testing with hand-crafted module text).
    """
    if not instructions:
        return set()

    addrs = sorted(set(addr for addr, _, _ in instructions))
    if not addrs:
        return set()

    # Strategy 1: authoritative function starts from vox._native_functions
    if binary_path:
        try:
            import vox
            func_addrs = {start for start, _ in vox._native_functions(binary_path)}
            present = func_addrs & set(addrs)
            if present:
                return present
        except Exception:
            pass

    # Strategy 2: gap heuristic fallback
    entries = {addrs[0]}
    for i in range(1, len(addrs)):
        if addrs[i] - addrs[i - 1] > 0x100:
            entries.add(addrs[i])
    return entries


def reconstruct_word(parsed: dict, binary_path: str = None) -> list:
    """Extract the IMASM word (glyph sequence) from parsed module text.

    This produces the same word that vox.recompile_module would lift.

    Each function starts with ⊢ (ENTRY/VINIT). Function boundaries are
    detected from address analysis, using authoritative function start
    addresses from vox._native_functions when binary_path is available.

    Key: ⊢ does NOT appear in the IMASM text. It is added at each function
    entry, matching vox.recompile_native's [VINIT] prepend.
    """
    instrs = parsed["instructions"]
    if not instrs:
        return []

    func_entries = _detect_function_entries(instrs, parsed["addresses"], binary_path)

    # A PLT stub is emitted by imasm_module.emit as ONE external-call line,
    # because the jump it really holds goes through a GOT slot nothing ever
    # loaded and the machine has no binary to read it from. The auditor's word
    # has the stub's real instructions instead. So the line has to be expanded
    # back, or μ∘δ=id fails at every external call for a reason that is about
    # the module text's convenience rather than about the structure.
    stub_words = _stub_expansions(binary_path) if binary_path else {}

    word = []
    emitted_addrs = set()

    for addr, glyph, fields in instrs:
        # Prepend ⊢ at function entry points (only once per address)
        if addr in func_entries and addr not in emitted_addrs:
            word.append(ENTRY)
            emitted_addrs.add(addr)
        if fields and fields[0] == "external" and addr in stub_words:
            word.extend(stub_words[addr])
            continue
        word.append(glyph)

    return word


def _stub_expansions(binary_path: str) -> dict:
    """{stub address: the glyphs its real instructions carry}.

    Read from the same recursive descent that produced the module text, and
    classified by vox.classify, so the expansion is what the auditor would have
    said rather than a second guess at it.
    """
    try:
        import vox
    except Exception:
        return {}
    try:
        stubs = set()
        with open(binary_path, "rb") as fh:
            magic = fh.read(4)
        if magic == b"\x7fELF":
            mode, secs, _ = vox._elf_sections(binary_path)
            stubs = set(vox._plt_stub_map(binary_path, secs, mode))
        if not stubs:
            return {}
        out = {}
        for start, func in vox._native_functions(binary_path):
            if start in stubs:
                out[start] = [vox.classify(i) for i in func]
        return out
    except Exception:
        return {}


def _word_to_imasm_text(word: list) -> str:
    """A word back to minimal module text.

    Not the executable module — that needs payload this direction does not
    have. This is the structure alone, written so `parse_module_text` reads
    back exactly the glyphs it was given. ⊢ opens a function rather than
    sitting on an address, so it becomes the next line's `@` label, and ∋ is a
    bare line because a fuse belongs to the address and not to an instruction.
    """
    lines = ["; ⊙ reconstituted", "; entry 0x0"]
    addr = 0
    pending_entry = False
    for g in word:
        if g == ENTRY:
            pending_entry = True
            continue
        addr += 1
        lines.append(f"@0x{addr:x}")
        lines.append(g if g == FUSE else f"{g}\tglyph")
        pending_entry = False
    return "\n".join(lines) + "\n"


def decompile(text: str, binary_path: str = None) -> list:
    """The main decompiler entry point.

    Takes IMASM module text (as produced by imasm_module.emit) and returns
    the IMASM word — the glyph sequence that vox.py would lift.

    Args:
        text: IMASM module text (e.g. from imasm_module.emit())
        binary_path: optional path to the original binary, used to get
                     authoritative function start addresses

    Returns:
        list of glyph characters (the IMASM word)
    """
    parsed = parse_module_text(text)
    return reconstruct_word(parsed, binary_path)


def decompile_bytecode(text: str) -> bytes:
    """Reconstruct a byte stream from IMASM module text.

    Maps each glyph back to a representative x86 opcode.
    """
    GLYPH_OPCODE = {
        ENTRY: b"\xf3\x0f\x1e\xfa",      # endbr64
        TERM: b"\xc3",                    # ret
        SPLIT: b"\x0f\x84",               # jne
        FUSE: b"",                        # merge point, no instruction
        CALL: b"\xe8",                    # call rel32
        XFER: b"\xe9",                    # jmp rel32
        INDIRECT: b"\xff\x25",            # jmp [rip+disp32]
        COMMIT: b"\x48\x89",              # mov [mem], reg
        LINK: b"\x48\x8b",                # mov reg, [mem]
        TRUTH: b"\x48\x31",               # cmp reg, reg
        CONSUME: b"\x0f\x94\xc0",         # sete al
        ENGAGE: b"\x48\x01",             # add reg, reg
    }

    parsed = parse_module_text(text)
    out = bytearray()
    for addr, glyph, fields in parsed["instructions"]:
        op = GLYPH_OPCODE.get(glyph, b"")
        out.extend(op)
        if glyph in (SPLIT, CALL, XFER, INDIRECT):
            out.extend(b"\x00\x00\x00\x00")
    return bytes(out)


def round_trip_verify(binary_path: str) -> tuple:
    """Full μ∘δ=id round-trip test on a binary.

    Returns (passed, decompiled_word, reference_word, details)
    """
    import imasm_module
    import vox

    # Step 1: Lift binary → IMASM module text
    text = imasm_module.emit(binary_path)

    # Step 2: Decompile IMASM text → IMASM word
    decompiled_word = decompile(text, binary_path)

    # Step 3: Get reference word from vox.recompile_module
    mod = vox.recompile_module(binary_path)
    reference_word = []
    for _, _, w in mod:
        reference_word.extend(w)

    passed = decompiled_word == reference_word
    return passed, decompiled_word, reference_word, {
        "decompiled_length": len(decompiled_word),
        "reference_length": len(reference_word),
        "match": passed,
    }


if __name__ == "__main__":
    import imasm_module
    import vox

    binary = "corpus_O0.so"
    if len(sys.argv) > 1:
        binary = sys.argv[1]

    # Step 1: emit IMASM module text
    text = imasm_module.emit(binary)
    print(f"emit() produced {len(text)} chars")

    # Step 2: decompile to word
    word = decompile(text, binary)
    print(f"decompile() produced {len(word)} glyphs")
    print(f"  word: {vox.glyphs(word)}")

    # Step 3: get reference word from vox.recompile_module
    mod = vox.recompile_module(binary)
    ref = []
    for _, _, w in mod:
        ref.extend(w)
    print(f"vox.recompile_module() produced {len(ref)} glyphs")
    print(f"  word: {vox.glyphs(ref)}")

    # Step 4: μ∘δ=id check
    if word == ref:
        print("\n✓ μ∘δ=id LOOP CLOSED")
        sys.exit(0)
    else:
        print(f"\n✗ μ∘δ=id LOOP BROKEN")
        # Find first divergence
        for i, (a, b) in enumerate(zip(word, ref)):
            if a != b:
                print(f"  first divergence at position {i}: decompiled={a}, reference={b}")
                break
        if len(word) != len(ref):
            print(f"  length mismatch: decompiled={len(word)}, reference={len(ref)}")
        sys.exit(1)