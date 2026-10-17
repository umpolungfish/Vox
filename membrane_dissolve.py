#!/usr/bin/env python3
"""membrane_dissolve.py: decode, nest and dissolve IMASM membrane words.

Ports three things from the Vox repo so they can be run without cargo:

  src/vox.rs                     glyph constants
  src/godel_calculus.rs          decode_cell / encode_cell_binary (strict)
  src/fixed_point_hypernest.rs   collapse_to_nested (the dissolution rule)

and adds one thing the repo does not have: reading a "word of words", where
complete skins (VINIT ... TANCH) sit inside FSPLIT ... FFUSE cells of an outer
word, joined by CLINK.

Usage:
  membrane_dissolve.py enc <natural>          cell-binary word for a natural
  membrane_dissolve.py dec <word>             strict decode of a cell-binary word
  membrane_dissolve.py nest <n> [<n> ...]     word of words holding those values
  membrane_dissolve.py lanes <word>           decode every lane of a word of words
  membrane_dissolve.py dissolve <word>        apply collapse_to_nested and report
  membrane_dissolve.py shift <word> <s>       prepend s EVALT frames (n >= 1 only)
  membrane_dissolve.py unshift <word> <s>     remove s leading EVALT frames
  membrane_dissolve.py pipe add|mul|braid <a> <b> <s>
                                              encode, shift both by s, op, unshift, decode
  membrane_dissolve.py pipe unbraid <n> <k>   same for unbraid; odd k swaps the lanes
  membrane_dissolve.py selftest
"""
import sys

# src/vox.rs
VINIT, TANCH = "⊢", "⊣"
AFWD, AREV = "≻", "≺"
FSPLIT, FFUSE = "∈", "∋"
IMSCRIB, IFIX = "⊙", "⊡"
CLINK = "⋈"
EVALT, EVALF = "⊤", "⊥"
ENGAGR = "⊞"
GLYPHS = set(
    [VINIT, TANCH, AFWD, AREV, FSPLIT, FFUSE, IMSCRIB, IFIX, CLINK, EVALT, EVALF, ENGAGR]
)
SUFFIX = IMSCRIB + IFIX + TANCH


# ---------------------------------------------------------------- cell-binary
def push_cell(bit):
    """One frame: AFWD CLINK FSPLIT EVAL{F=1,T=0} FFUSE."""
    return AFWD + CLINK + FSPLIT + (EVALF if bit else EVALT) + FFUSE


def encode_cell_binary(n):
    """godel_calculus::encode_cell_binary. Leftmost cell is 2^0; EVALF is 1."""
    if n < 0:
        raise ValueError("naturals only")
    bits = [(n >> k) & 1 for k in range(n.bit_length())] or [0]
    return VINIT + "".join(push_cell(b) for b in bits) + SUFFIX


def decode_cell_binary(word):
    """godel_calculus::decode_cell, strict. Returns (value, bits_le)."""
    if not word:
        raise ValueError("empty glyph word")
    for c in word:
        if c not in GLYPHS:
            raise ValueError(f"not an IMASM glyph: {c}")
    n = len(word)
    if n < 9 or word[0] != VINIT:
        raise ValueError("not a cell-binary word: bad opening or too short")
    if word[n - 3 :] != SUFFIX:
        raise ValueError("not a cell-binary word: bad suffix")
    body = n - 4
    if body == 0 or body % 5 != 0:
        raise ValueError("not a cell-binary word: body is not whole 5-glyph frames")
    bits = []
    for cell in range(body // 5):
        i = 1 + cell * 5
        if word[i : i + 3] != AFWD + CLINK + FSPLIT or word[i + 4] != FFUSE:
            raise ValueError(f"frame {cell} is malformed")
        if word[i + 3] == EVALF:
            bits.append(1)
        elif word[i + 3] == EVALT:
            bits.append(0)
        else:
            raise ValueError(f"frame {cell} has no EVALT/EVALF")
    return sum(b << k for k, b in enumerate(bits)), bits


# ------------------------------------------------------------ word of words
def nest(values):
    """Outer word: VINIT FSPLIT CLINK (FSPLIT <word> FFUSE CLINK)* FFUSE IMSCRIB IFIX TANCH.

    This is the shape of the three-copy word in the conversation. The repo has no
    generator for it that I could find, so this is reconstructed from that example.
    """
    inner = CLINK + "".join(FSPLIT + encode_cell_binary(v) + FFUSE + CLINK for v in values)
    return VINIT + FSPLIT + inner + FFUSE + SUFFIX


def split_lanes(word):
    """Return the inner words of a word of words, one per depth-1 FSPLIT..FFUSE group."""
    if not (word.startswith(VINIT + FSPLIT) and word.endswith(FFUSE + SUFFIX)):
        raise ValueError("not a word of words: expected VINIT FSPLIT ... FFUSE IMSCRIB IFIX TANCH")
    body = word[2:-4]
    lanes, depth, start = [], 0, 0
    for i, c in enumerate(body):
        if c == FSPLIT:
            if depth == 0:
                start = i + 1
            depth += 1
        elif c == FFUSE:
            depth -= 1
            if depth < 0:
                raise ValueError("unbalanced FFUSE between lanes")
            if depth == 0:
                lanes.append(body[start:i])
        elif depth == 0 and c != CLINK:
            raise ValueError(f"unexpected {c!r} between lanes (only CLINK allowed)")
    if depth != 0:
        raise ValueError("unbalanced FSPLIT")
    return lanes


# ---------------------------------------------------------------- dissolution
def collapse_to_nested(word):
    """Port of fixed_point_hypernest::collapse_to_nested.

    Keep the first VINIT, the first FSPLIT, the last FFUSE and the last TANCH.
    Every other occurrence of those four marks is dissolved. All other marks,
    including every IFIX winding record, are untouched.
    """
    last_ffuse = word.rfind(FFUSE)
    last_tanch = word.rfind(TANCH)
    seen_vinit = seen_fsplit = False
    out = []
    for i, mark in enumerate(word):
        if mark == VINIT:
            keep = not seen_vinit
            seen_vinit = True
        elif mark == FSPLIT:
            keep = not seen_fsplit
            seen_fsplit = True
        elif mark == FFUSE:
            keep = i == last_ffuse
        elif mark == TANCH:
            keep = i == last_tanch
        else:
            keep = True
        if keep:
            out.append(mark)
    return "".join(out)


def census(word):
    names = [
        (VINIT, "VINIT"), (FSPLIT, "FSPLIT"), (FFUSE, "FFUSE"), (TANCH, "TANCH"),
        (IFIX, "IFIX (winding)"), (IMSCRIB, "IMSCRIB"), (CLINK, "CLINK"),
        (AFWD, "AFWD"), (EVALT, "EVALT"), (EVALF, "EVALF"),
    ]
    return {name: word.count(g) for g, name in names}


def report_dissolve(word):
    before, after = census(word), census(collapse_to_nested(word))
    lines = [f"{'mark':16} {'before':>7} {'after':>7}"]
    for name in before:
        lines.append(f"{name:16} {before[name]:>7} {after[name]:>7}")
    return "\n".join(lines)


# --------------------------------------------------------------- frame shift
ZERO_FRAME = AFWD + CLINK + FSPLIT + EVALT + FFUSE


def shift_word(word, s):
    """Prepend s EVALT frames, i.e. multiply by 2^s. Zero has no shifted word."""
    if s < 0:
        raise ValueError("shift must be >= 0")
    value, _ = decode_cell_binary(word)
    if value == 0:
        raise ValueError("zero has no shifted word: encode(0) is one frame and 0 << s == 0")
    return VINIT + ZERO_FRAME * s + word[1:]


def unshift_word(word, s):
    """Remove s leading EVALT frames. Zero is shift-invariant and is returned unchanged."""
    if s < 0:
        raise ValueError("shift must be >= 0")
    value, bits = decode_cell_binary(word)
    if value == 0:
        return word
    if s > len(bits) or any(bits[:s]):
        raise ValueError(f"word does not start with {s} EVALT frames")
    return VINIT + word[1 + 5 * s :]


# ------------------------------------------- numeral operations (value level)
def _bits(n):
    return [(n >> k) & 1 for k in range(n.bit_length())]


def _from_bits(b):
    return sum(x << k for k, x in enumerate(b))


def braid_values(g, l):
    """godel_calculus::braid_values: interleave, gamma on even positions."""
    bg, bl = _bits(g), _bits(l)
    out = []
    for i in range(max(len(bg), len(bl))):
        out.append(bg[i] if i < len(bg) else 0)
        out.append(bl[i] if i < len(bl) else 0)
    return _from_bits(out)


def unbraid_value(n):
    """godel_calculus::unbraid_value: (even positions, odd positions)."""
    b = _bits(n)
    return _from_bits(b[0::2]), _from_bits(b[1::2])


# How an input shift of s on both operands appears on the result: result << (DEGREE*s).
DEGREE = {"add": 1, "mul": 2, "braid": 2}
BINARY = {
    "add": lambda a, b: a + b,
    "mul": lambda a, b: a * b,
    "braid": braid_values,
}


def pipe(op, a, b, s):
    """encode -> shift both by s -> op -> unshift by DEGREE*s -> decode.

    The operation itself decodes its inputs, computes on naturals and re-encodes,
    as `godel check` does; only the shift and unshift act on words.
    Returns (got, expected, stage_lines).
    """
    if op not in BINARY:
        raise ValueError(f"op must be one of {sorted(BINARY)}")
    fr = lambda w: w.count(FSPLIT)
    wa, wb = encode_cell_binary(a), encode_cell_binary(b)
    sa, sb = shift_word(wa, s), shift_word(wb, s)
    r = BINARY[op](decode_cell_binary(sa)[0], decode_cell_binary(sb)[0])
    wr = encode_cell_binary(r)
    back = unshift_word(wr, DEGREE[op] * s)
    got = decode_cell_binary(back)[0]
    expected = BINARY[op](a, b)
    stages = [
        f"encode     a {fr(wa)} frames, b {fr(wb)} frames",
        f"shift      by {s}: a {fr(sa)} frames, b {fr(sb)} frames",
        f"{op:<10} result {fr(wr)} frames  (value {r})",
        f"unshift    by {DEGREE[op]}*{s} = {DEGREE[op] * s}: {fr(back)} frames",
        f"decode     {got}",
        f"expected   {expected}  {'PASS' if got == expected else 'FAIL'}",
    ]
    return got, expected, stages


def pipe_unbraid(n, k):
    """encode -> shift by k -> unbraid -> unshift each lane -> decode.

    Even k = 2s: both lanes come out shifted by s.
    Odd  k = 2s+1: the lanes swap, first lane shifted by s+1 and second by s.
    A zero lane stays zero and is not unshifted.
    Returns ((gamma, lambda), expected, stage_lines).
    """
    fr = lambda w: w.count(FSPLIT)
    wn = encode_cell_binary(n)
    sn = shift_word(wn, k)
    first, second = unbraid_value(decode_cell_binary(sn)[0])
    w1, w2 = encode_cell_binary(first), encode_cell_binary(second)
    s, odd = divmod(k, 2)
    if odd:
        gamma_w, lambda_w = unshift_word(w2, s), unshift_word(w1, s + 1)
        how = f"odd shift: lanes swapped; unshift first lane by {s + 1}, second by {s}"
    else:
        gamma_w, lambda_w = unshift_word(w1, s), unshift_word(w2, s)
        how = f"even shift: unshift both lanes by {s}"
    got = (decode_cell_binary(gamma_w)[0], decode_cell_binary(lambda_w)[0])
    expected = unbraid_value(n)
    stages = [
        f"encode     n {fr(wn)} frames",
        f"shift      by {k}: {fr(sn)} frames",
        f"unbraid    lanes {fr(w1)} and {fr(w2)} frames",
        f"unshift    {how}",
        f"decode     gamma {got[0]}, lambda {got[1]}",
        f"expected   gamma {expected[0]}, lambda {expected[1]}  {'PASS' if got == expected else 'FAIL'}",
    ]
    return got, expected, stages


# ------------------------------------------------------------------ selftest
def _check(cond, label):
    print(("PASS  " if cond else "FAIL  ") + label)
    if not cond:
        raise SystemExit(1)


def selftest():
    # 1. The repo's own unit-test vectors for collapse_to_nested
    #    (src/fixed_point_hypernest.rs, collapse_dissolves_interior_boundaries_and_keeps_the_winding).
    expanded = (
        "⊢⊙∈≻⊤⋈≺⊥⊞⊢⊙∈≻⊤⋈≺⊥⊞⊢⊙∈≻⊤⋈≺⊥⊞⊢⊙∈≻⊤⋈≺⊥⊞⊢⊙∈≻⊤⋈≺⊥⊞⊢⊙∈≻⊤⋈≺⊥⊞"
        "∋⊡⋈⊙⊣∋⊡⋈⊙⊣∋⊡⋈⊙⊣∋⊡⋈⊙⊣∋⊡⋈⊙⊣∋⊡⋈⊙⊣"
    )
    nested = (
        "⊢⊙∈≻⊤⋈≺⊥⊞⊙≻⊤⋈≺⊥⊞⊙≻⊤⋈≺⊥⊞⊙≻⊤⋈≺⊥⊞⊙≻⊤⋈≺⊥⊞⊙≻⊤⋈≺⊥⊞"
        "⊡⋈⊙⊡⋈⊙⊡⋈⊙⊡⋈⊙⊡⋈⊙∋⊡⋈⊙⊣"
    )
    _check(collapse_to_nested(expanded) == nested, "collapse_to_nested reproduces the repo's hand-nested vector")
    got = collapse_to_nested(expanded)
    _check(all(got.count(m) == 1 for m in (VINIT, FSPLIT, FFUSE, TANCH)), "one of each boundary mark survives")
    _check(got.count(IFIX) == expanded.count(IFIX) == 6, "winding (IFIX count 6) preserved")

    # 2. Codec: round trip, and the values printed by `godel encode 1234567890`.
    for n in list(range(0, 400)) + [2**64 - 1, 2**200 + 12345]:
        _check_quiet = decode_cell_binary(encode_cell_binary(n))[0] == n
        if not _check_quiet:
            _check(False, f"round trip {n}")
    _check(True, "encode/decode round trip for 0..399 and two large values")
    n = 1234567890
    word = encode_cell_binary(n)
    value, bits = decode_cell_binary(word)
    bits_le = "".join(map(str, bits))
    support = [k for k, b in enumerate(bits) if b]
    _check(bits_le == "0100101101000000011010011001001", "bits-le matches godel encode output")
    _check(bin(value)[2:] == "1001001100101100000001011010010", "binary matches godel encode output")
    _check(support == [1, 4, 6, 7, 9, 17, 18, 20, 23, 24, 27, 30], "support matches godel encode output")
    _check(word.count(FSPLIT) == len(bits) == 31, "braid.crossings (FSPLIT count) equals frame count, 31")

    # 3. Strict decoder rejects malformed words.
    for bad in ("", "abc", VINIT + SUFFIX, word[:-1], word.replace(CLINK, AFWD, 1)):
        try:
            decode_cell_binary(bad)
            _check(False, f"rejected {bad[:12]!r}")
        except ValueError:
            pass
    _check(True, "strict decoder rejects empty, non-glyph, short, truncated and corrupted words")

    # 4. Word of words and its dissolution.
    outer = nest([n, n, n])
    lanes = split_lanes(outer)
    _check([decode_cell_binary(w)[0] for w in lanes] == [n, n, n], "three lanes decode to [1234567890] * 3")
    flat = collapse_to_nested(outer)
    c = census(flat)
    _check(all(c[k] == 1 for k in ("VINIT", "FSPLIT", "FFUSE", "TANCH")), "dissolved word has one skin")
    _check(census(outer)["IFIX (winding)"] == c["IFIX (winding)"] == 4, "winding preserved: 4 IFIX before and after")
    _check(c["EVALT"] + c["EVALF"] == 93, "all 93 bit marks survive")
    _check(collapse_to_nested(flat) == flat, "dissolution is idempotent")
    try:
        decode_cell_binary(flat)
        _check(False, "dissolved word is not a cell-binary word")
    except ValueError:
        _check(True, "dissolved word is NOT a cell-binary numeral (cell boundaries were dissolved too)")
    # 5. Frame shift and the encode -> shift -> op -> unshift -> decode pipelines.
    import random

    rng = random.Random(7)
    ok = all(
        shift_word(encode_cell_binary(m), s) == encode_cell_binary(m << s)
        and unshift_word(shift_word(encode_cell_binary(m), s), s) == encode_cell_binary(m)
        for m in range(1, 200)
        for s in range(0, 7)
    )
    _check(ok, "shift_word(encode(n), s) == encode(n << s) and unshift inverts it, n in 1..199")
    for bad_call, label in (
        (lambda: shift_word(encode_cell_binary(0), 3), "shifting zero is refused"),
        (lambda: unshift_word(encode_cell_binary(5), 1), "unshift refuses a word without a leading EVALT frame"),
    ):
        try:
            bad_call()
            _check(False, label)
        except ValueError:
            _check(True, label)
    _check(unshift_word(encode_cell_binary(0), 4) == encode_cell_binary(0), "zero is shift-invariant under unshift")

    def rnd():
        return rng.getrandbits(rng.randint(1, 80)) | 1 if rng.random() < 0.5 else rng.getrandbits(rng.randint(1, 80)) + 1

    for op in ("add", "mul", "braid"):
        ok = True
        for _ in range(400):
            a, b, s = rnd(), rnd(), rng.randint(0, 9)
            got, expected, _ = pipe(op, a, b, s)
            ok = ok and got == expected
        _check(ok, f"pipe {op}: 400 random round trips, shifts 0..9")
    ok = True
    for _ in range(600):
        m, k = rnd(), rng.randint(0, 11)
        got, expected, _ = pipe_unbraid(m, k)
        ok = ok and got == expected
    for m in (1, 2, 3):
        for k in range(0, 8):
            got, expected, _ = pipe_unbraid(m, k)
            ok = ok and got == expected
    _check(ok, "pipe unbraid: 600 random plus n in 1..3 (zero lanes), shifts 0..11, odd shifts swap lanes")
    print("all checks passed")


def main(argv):
    if len(argv) < 2:
        print(__doc__)
        return 1
    cmd, args = argv[1], argv[2:]
    if cmd == "enc" and len(args) == 1:
        print(encode_cell_binary(int(args[0])))
    elif cmd == "dec" and len(args) == 1:
        value, bits = decode_cell_binary(args[0])
        print(f"value   {value}\nframes  {len(bits)}\nbits-le {''.join(map(str, bits))}")
    elif cmd == "nest" and args:
        print(nest([int(a) for a in args]))
    elif cmd == "lanes" and len(args) == 1:
        for i, lane in enumerate(split_lanes(args[0])):
            value, bits = decode_cell_binary(lane)
            print(f"lane {i}  value {value}  frames {len(bits)}")
    elif cmd == "dissolve" and len(args) == 1:
        print(collapse_to_nested(args[0]))
        print()
        print(report_dissolve(args[0]))
    elif cmd == "shift" and len(args) == 2:
        print(shift_word(args[0], int(args[1])))
    elif cmd == "unshift" and len(args) == 2:
        print(unshift_word(args[0], int(args[1])))
    elif cmd == "pipe" and len(args) == 4 and args[0] in BINARY:
        _, _, stages = pipe(args[0], int(args[1]), int(args[2]), int(args[3]))
        print("\n".join(stages))
    elif cmd == "pipe" and len(args) == 3 and args[0] == "unbraid":
        _, _, stages = pipe_unbraid(int(args[1]), int(args[2]))
        print("\n".join(stages))
    elif cmd == "selftest":
        selftest()
    else:
        print(__doc__)
        return 1
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main(sys.argv))
    except ValueError as error:
        print(f"error: {error}", file=sys.stderr)
        sys.exit(1)
