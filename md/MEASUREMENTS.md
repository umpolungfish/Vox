# How much of a binary is program?

`measure.py PROGRAM` recompiles a native binary (PE or ELF) to IMASM and
measures the result three ways: the structure alone, the operands, and the two
together, which is the ceiling for a lossless recompile that still runs.

## Across compilers and languages

| compiler | binary | decoded | coverage | bits/glyph | structure | lossless | ratio |
|---|---|---|---|---|---|---|---|
| MSVC C++ | ChemDraw.exe | 214,782 | 36% | 1.37 | 5.3% | 38.1% | 2.6x |
| rustc | momonados | 501,519 | 100% | 2.02 | 5.7% | 40.4% | 2.5x |
| Go | go | 5,274,730 | 100% | 1.81 | 5.7% | 41.1% | 2.4x |
| gcc C | xterm | 460,298 | 100% | 2.03 | 6.3% | 40.6% | 2.5x |
| gcc C | pkg-config | 23,869 | 100% | 1.84 | 6.0% | 34.4% | 2.9x |
| gcc C++ | grub-render-label | 560,953 | 100% | 2.05 | 6.9% | 44.6% | 2.2x |
| gcc C | ls | 77,962 | 100% | 2.12 | 7.1% | 40.0% | 2.5x |

Seven binaries, four compilers, four languages, two containers, decoded sizes
spanning two hundredfold. **Structure lands between 5.3% and 7.1% every time,
and a lossless recompile between 34% and 45%, a 2.2x to 2.9x shrink.**
Whatever produced the code, saying what it *is* costs about a sixteenth of
saying it in x86, and rewriting it so it still runs costs about two-fifths.

## The denominator, which is where this went wrong once

The ratios are charged against **the bytes that actually decoded into
instructions**, not the size of the executable section. A section holds padding,
jump tables, and embedded data that never become instructions. ChemDraw's
executable section is 596,480 bytes but only 214,782 of them — 36% — decode; the
other 64% is not program at all. An earlier version of this file divided by the
section size and reported 1.9% structure and a 7.3x shrink, which made that
binary look like an outlier with a uniquely compressible structure. It was not.
It was a denominator that included bytes the disassembler never read. Corrected,
it sits with everything else.

The `coverage` column is kept in the output for exactly this reason: a low
number is a fact about the file (packed, data-heavy, or defeating recursive
descent), not a fact about the program, and it must never silently scale a
ratio. Coverage above 100% would be the same kind of fact in the other
direction — the same bytes counted under two functions — and `measure.py`
treats it as a bug to fix, not a number to print; every row above reads 100%
or explains, like ChemDraw's, exactly which bytes are missing and why.

## What each row is

**Structure** is the recompiled word and nothing else: which of the twelve each
instruction is, in order. Measured as order-2 conditional entropy, so the word's
own regularity is already subtracted — a glyph, given the two before it, costs
between 1.37 and 2.12 bits. Compiled code is a small number of shapes laid end
to end, and that predictability is the finding rather than an artifact.

**Operands** is what the glyph does not carry: registers, immediates, widths.
Measured, not estimated, from the entropy of the operand strings.

**Lossless IMASM** is the two together. The glyph stream compresses tighter
still under generic zlib — for ChemDraw, 4.4% of the decoded machine code,
under the entropy estimate itself — with the difference that IMASM stays
executable and zlib does not.

## The caveat that keeps the numbers true

The structure figure is a **lossy** lift and is not a compression ratio. The
claim is the lossless column. The structure column is a different quantity —
what it costs to say what a program is, as against what it operates on.
