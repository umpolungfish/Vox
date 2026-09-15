# Complete membrane words

For uninterrupted glyph-only sequences, use the [glyph word index](../glyphs/README.md).
The files below are the annotated executable module representation.

These are complete executable IMASM modules: lifted operations, baked payload,
symbols and data. They are not numeral-only inputs or per-function structural
summaries. Each was regenerated from its saved static ELF and executed through
Vox, requiring zero VM exit and exact native stdout/stderr agreement.

| Membrane | Complete word | Baked payload |
|---|---|---|
| Binomial-row GCD | [binomial.elf.imasm](binomial.elf.imasm) | 0,1,2,8,9,12,25,27,30,81,120,81 |
| Cumulative LCM | [lcm.elf.imasm](lcm.elf.imasm) | 0,1,10,20,40,80,100,10 |
| Divisor ring | [divisor.elf.imasm](divisor.elf.imasm) | 1,2,97,360,65536 |
| Landau | [landau.elf.imasm](landau.elf.imasm) | 0,1,10,15,30 |
| Schütte | [schutte.elf.imasm](schutte.elf.imasm) | 7 vertices; k=1,2,3 |
| Shor/QFT | [shor.elf.imasm](shor.elf.imasm) | a=2, N=21, 12 qubits |

From the Vox repository root:

```sh
./target/release/vox run membranes/words/lcm.elf.imasm
```

Corresponding `.elf`, `.stdout` and `.stderr` files are retained beside each
word. Original native controls and repeated measurements remain in the
payload directories linked from [the test tables](../../membrane_test_tables.md).
