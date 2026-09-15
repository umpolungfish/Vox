# Glyph-only executable module format, version 1

This is a new lossless serialization layer for Vox's existing executable IMASM
module format. It does not change the twelve opcodes, arithmetic, instruction
operands, or compiled membrane algorithms. It is not the existing `vox word`
structural projection and must not be given to `vox verdict` as an execution
test. Executability is checked by `vox run` and native-output comparisons.

## Records and payloads

The file is one continuous glyph sequence with no whitespace. Its outer frame
is `⊢∈` followed by records and `∋⊡⊣`.

Each record retains its head glyph and attaches a payload:

```text
HEAD ∈ ⊢∈ BYTE_BITS ⊥ ≺∋⊡⊣ ∋
```

Spaces and capitalized names above are explanatory notation only. `BYTE_BITS`
contains each byte's eight bits, least-significant first: `⊤` for zero and `⊥`
for one. A final high one-bit sentinel preserves the exact byte length,
including zero-valued bytes. The payload uses the existing single-frame numeral
shape, with `≺` inside its matching fuse. No work glyph is replaced by a numeral.

The first record has head `⊙` and encodes the UTF-8 header
`VOXGLYPH1:<module-byte-length>:<16-digit-hex-FNV1a64>`.
The hash is an accidental-corruption check, not cryptographic authentication.

Subsequent records encode exact UTF-8 module lines, including their newline
when present. Instruction lines retain their original leading glyph as the
record head. Address, symbol, memory and comment lines use `⋈`. The decoder
checks that each head agrees with its recovered line, validates framing,
sentinels, byte alignment, UTF-8, version, length and checksum, and rejects
trailing content. It does not reserve memory from the declared length.

Full lines are retained, including instruction heads inside the payload, to
make the representation byte-for-byte reversible. This redundancy is deliberate;
version 1 prioritizes verifiability rather than compression. The roughly 24×
UTF-8 expansion comes from eight three-byte glyphs per original byte, plus
record framing.

## Execution and commands

`vox glyphs <module.imasm> <new-output.glyphs>` writes the sequence.
`vox unglyphs <word.glyphs> <new-output.imasm>` recovers the exact module.
Both refuse to overwrite existing files.

`vox run <word.glyphs>` decodes the sequence into the module representation,
then uses the existing machine and host interface. All code, symbols, addresses,
relocations and memory data are in the word; neither its source ELF nor a
sidecar module is needed. File content detection also recognizes the format's
prefix when the extension is not `.glyphs`. `vox circuit` supports the same
format and decodes once before preparing its resident machine.

This format does not sandbox executable code or make untrusted programs safe.
Malformed serialization is rejected before loading a machine.

## Verification

Four new library tests cover exact round trips (with and without a final source
newline), all 256 byte values, malformed/truncated/corrupted records, symbol
retention and execution. The full release all-target suite passes 91 tests.
The create-new behavior is checked against an existing word without changing it.
Six complete membrane words pass byte-identical recovery and native-output
controls. An 8-port resident circuit passes its independent transform controls.

Records: `glyph_module_tests.log`, `glyph_module_full_tests.log`,
`glyph_membranes.log`, `glyph_overwrite_test.log`, `glyph_circuit_test.log`.
