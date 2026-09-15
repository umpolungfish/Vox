# Complete executable glyph sequences

[Five-run timings, ranges and verification records](timings.md).

Every `.glyphs` file is one uninterrupted sequence from the twelve-glyph
alphabet, without ASCII annotations or a trailing newline. It contains the
complete compiled membrane and baked payload, not just the input numerals or
a per-function structural summary.

| Membrane | Glyph sequence | Glyph count | Baked payload |
|---|---|---:|---|
| Binomial-row GCD | [binomial.glyphs](binomial.glyphs) | 24,381,701 | 0,1,2,8,9,12,25,27,30,81,120,81 |
| Cumulative LCM | [lcm.glyphs](lcm.glyphs) | 24,460,697 | 0,1,10,20,40,80,100,10 |
| Divisor ring | [divisor.glyphs](divisor.glyphs) | 27,266,139 | 1,2,97,360,65536 |
| Landau | [landau.glyphs](landau.glyphs) | 24,513,507 | 0,1,10,15,30 |
| Schütte | [schutte.glyphs](schutte.glyphs) | 24,333,921 | 7 vertices; k=1,2,3 |
| Shor/QFT | [shor.glyphs](shor.glyphs) | 26,931,805 | a=2, N=21, 12 qubits |
| Rep-tiling | [reptiling.glyphs](reptiling.glyphs) | 24,312,521 | 1,2,3,4,5,6,7,8,9,10,18,20,25,26,100,1000,6 |

These are version-1 **serialized executable modules**. `vox run` decodes the
glyph representation and executes the recovered instructions in its existing
machine. This is not a claim that the structure-only verdict interpreter now
executes arbitrary glyph programs, or that serializing a module optimizes it.
Instruction heads remain visible; their operand/data payloads are nested, not
dropped. The size includes the complete static executable and runtime.

From the repository root:

```sh
./target/release/vox run membranes/glyphs/lcm.glyphs
# Exact recovery; output must not already exist:
./target/release/vox unglyphs membranes/glyphs/lcm.glyphs recovered-lcm.imasm
# Encode another complete module:
./target/release/vox glyphs input.imasm output.glyphs
```

All seven words recovered byte-identical source modules and matched native stdout
and stderr exactly with zero VM exit. The `.recovered.imasm`, `.stdout`,
`.stderr` and `.time` files are retained beside them. [Run log](../../glyph_membranes.log).

The [8-port resident QFT word](qft8.glyphs) is also verified through `vox circuit`,
including feedback, clearing and repeated activation in one prepared machine.
Its [control log](../../glyph_circuit_test.log) records the checks.

See the [rep-tiling membrane notes](../../membrane_reptiling_notes.md).

See [format specification](../../GLYPH_MODULE_FORMAT.md). `bash glyph_membranes.sh`
generates and checks the six membrane words without replacing existing words.
