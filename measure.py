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
    comp = vox._composition(path)
    code = comp["code"]
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
    ops = [i.op_str.strip() for _, f in vox._native_functions(path) for i in f]
    c = Counter(ops)
    h_ops = -sum(v / len(ops) * math.log2(v / len(ops)) for v in c.values())
    operand = len(ops) * h_ops / 8

    uniq = Counter(words)
    return {
        "code": code, "glyphs": n, "insns": len(ops),
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
    print(f"machine code          {m['code']:>10,} B   100.0%")
    print(f"structure             {m['struct']:>10,.0f} B  {pct(m['struct'])}")
    print(f"  zlib'd              {m['zlib_glyphs']:>10,} B  {pct(m['zlib_glyphs'])}")
    print(f"operands              {m['operand']:>10,.0f} B  {pct(m['operand'])}")
    print(f"lossless IMASM        {m['lossless']:>10,.0f} B  {pct(m['lossless'])}"
          f"   ({m['code'] / m['lossless']:.1f}x smaller)")


if __name__ == "__main__":
    main()
