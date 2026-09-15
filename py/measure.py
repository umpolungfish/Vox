#!/usr/bin/env python3
"""How much of a binary is program, and how much is encoding?

Recompiles a native PE to IMASM and measures the result three ways: the
structure alone (what the program IS), the operands (what it operates ON), and
the two together, which is the ceiling for a lossless recompile that still runs.
Run it on any PE; the numbers below in MEASUREMENTS.md are one instance.
"""
import math
import sys
import zlib
from collections import Counter

import vox


def measure(path):
    section = vox._composition(path)["code"]
    mod = vox.recompile_module(path)
    words = [vox.glyphs(w) for _, _, w in mod]
    stream = "".join(words)
    n = len(stream)

    # Structure: order-2 conditional entropy. A glyph given the two before it,
    # which is the honest cost of the word once its own regularity is counted.
    ctx = Counter(stream[i:i + 2] for i in range(n - 2))
    tri = Counter(stream[i:i + 3] for i in range(n - 2))
    h2 = -sum(v / (n - 2) * math.log2(v / ctx[k[:2]]) for k, v in tri.items())
    struct = (n - 2) * h2 / 8

    # Operands: what the glyph does not carry, and what a lossless recompile
    # would have to add. Measured, not estimated.
    # The denominator is the bytes that ACTUALLY DECODED, not the section size.
    # An executable section holds padding, jump tables, and embedded data that
    # never become instructions; charging the word against those bytes inflates
    # every ratio by however much of the section the sweep could not read.
    insns = [i for _, f in vox._native_functions(path) for i in f]
    code = sum(i.size for i in insns)
    ops = [i.op_str.strip() for i in insns]
    c = Counter(ops)
    h_ops = -sum(v / len(ops) * math.log2(v / len(ops)) for v in c.values())
    operand = len(ops) * h_ops / 8

    uniq = Counter(words)
    return {
        "code": code, "section": section, "coverage": code / section,
        "glyphs": n, "insns": len(ops),
        "struct": struct, "operand": operand, "lossless": struct + operand,
        "zlib_glyphs": len(zlib.compress(stream.encode(), 9)),
        "words": len(words), "unique": len(uniq), "h2": h2,
    }


def main():
    if len(sys.argv) != 2:
        raise SystemExit("usage: measure.py PROGRAM.exe")
    m = measure(sys.argv[1])
    pct = lambda v: f"{v / m['code'] * 100:5.1f}%"
    print(f"instructions          {m['insns']:>10,}")
    print(f"glyphs                {m['glyphs']:>10,}")
    print(f"words (unique)        {m['words']:>10,} ({m['unique']:,})")
    print(f"glyph cost at order 2 {m['h2']:>10.2f} bits\n")
    print(f"section bytes         {m['section']:>10,} B")
    print(f"decoded to instrs     {m['code']:>10,} B  {m['coverage'] * 100:5.1f}%"
          f"  <- the denominator")
    print(f"machine code          {m['code']:>10,} B   100.0%")
    print(f"structure             {m['struct']:>10,.0f} B  {pct(m['struct'])}")
    print(f"  zlib'd              {m['zlib_glyphs']:>10,} B  {pct(m['zlib_glyphs'])}")
    print(f"operands              {m['operand']:>10,.0f} B  {pct(m['operand'])}")
    print(f"lossless IMASM        {m['lossless']:>10,.0f} B  {pct(m['lossless'])}"
          f"   ({m['code'] / m['lossless']:.1f}x smaller)")


if __name__ == "__main__":
    main()
