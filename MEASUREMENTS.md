# How much of a binary is program?

`measure.py PROGRAM.exe` recompiles a native PE to IMASM and measures the result
three ways. The numbers here are one instance, a 596 KB x86-64 build; rerun it
on anything.

```
instructions              63,508
glyphs                    66,585
words (unique)               408 (243)
glyph cost at order 2       1.37 bits

machine code             596,480 B   100.0%
structure                 11,421 B    1.9%
  zlib'd                   9,513 B    1.6%
operands                  70,532 B   11.8%
lossless IMASM            81,953 B   13.7%   (7.3x smaller)
```

## What each row is

**Structure** is the recompiled word and nothing else: which of the twelve each
instruction is, in order. It is measured as order-2 conditional entropy, so the
word's own regularity is already subtracted — a glyph, given the two before it,
costs 1.37 bits. Compiled code is a small number of shapes laid end to end, and
that predictability is the finding rather than an artifact of the coding.

**Operands** is what the glyph does not carry: registers, immediates, widths.
Measured, not estimated — 9,306 distinct operand strings across the module, 8.88
bits of entropy each.

**Lossless IMASM** is the two together, the ceiling for a recompile that still
runs. Against zlib on the machine code (188,424 B, 31.6%) it is 2.3x smaller,
and unlike zlib it is executable.

## The caveat that keeps the number true

The 1.9% is a **lossy** lift. It is not a like-for-like compression figure and
must not be quoted as one. The honest claim is 13.7%: what it costs to rewrite
the program so it still does what it did. The 1.9% is a different quantity —
what it costs to say what the program *is*, as against what it operates on.

## Repetition

165 of 408 functions lift to a word another function already had; 243 are
distinct. Two words that differ by a couple of glyphs appear 36 and 27 times
apiece — one template stamped out per type. In the original those are 63
separate runs of machine code. In IMASM they are one word and a count.
