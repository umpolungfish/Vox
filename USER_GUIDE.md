# Vox User Guide

**V⊙x — the Pancosmic Disassembling Re-Compiling Organism.** Every word names
something you can run. Single Rust crate, zero external crates.

This guide covers every command, binary, script, Python entry point, data file
and directory in this repo, with worked examples. All example outputs below were
captured by running the commands in this tree.

## Contents

1. [What Vox is](#what-vox-is)
2. [Install and build](#install-and-build)
3. [The twelve glyphs and the verdict](#the-twelve-glyphs-and-the-verdict)
4. [The main `vox` CLI — every subcommand](#the-main-vox-cli--every-subcommand)
5. [Baked membranes (encode-before-compile)](#baked-membranes)
6. [The Rust binaries (`src/bin/`)](#the-rust-binaries)
7. [Shell scripts](#shell-scripts)
8. [Python entry points](#python-entry-points)
9. [Source layout (`src/`)](#source-layout)
10. [Top-level files](#top-level-files)
11. [Directories](#directories)
12. [Key documents](#key-documents)
13. [Verification and tests](#verification-and-tests)
14. [Edges and refusals](#edges-and-refusals)

## What Vox is

- **Pancosmic.** One lift, every substrate: native x86-64 (ELF/PE/Mach-O, both
  widths), EVM bytecode, WASM function bodies, CPython `.pyc`, and coding
  sequences (12 promoted amino acids biject the 12 axes — a gene is already a
  word). A merge is a merge whether `JUMPDEST`, `end`, or two-predecessor jump
  target.
- **Disassembling.** Own loader, decoder, machine — no capstone/pefile/runtime.
  Refuses unknown architectures rather than misreading them; unknown opcodes
  stop the walk and report how far it got, never guess.
- **Re-Compiling.** The lift IS the program: native code recompiled to an
  executable IMASM module, run in a machine that never looks at the original
  bytes.
- **Organism.** `vox self` lifts Vox's own image: every function, in phase,
  F zero.

## Install and build

```bash
cd /home/mrnob0dy666/imsgct/Vox
cargo build --release          # builds the `vox` binary + all src/bin binaries
cargo install --path .         # puts `vox` on PATH
```

- `Cargo.toml`: package `vox` v0.1.0, edition 2021, Unlicense, `autobins = false`
  (every binary is declared explicitly). Dependencies: `num-bigint`,
  `num-integer`, `num-traits`. Features: `resident`, `mpqs_debug`.
  Release profile: `opt-level = 3`, `lto = true`.
- `build.rs` bakes compile-time IMASM words into the binaries (the baked
  membrane pattern — see [§5](#baked-membranes)).
- `Makefile`:
  ```
  make test                 # cargo test --lib
  make test-fixed-point     # cargo test --lib fixed_point_protocol
  make check                # cargo check --lib
  ```
- CI (`.github/workflows`): runs `make test` on push/PR, 20-minute timeout.
- `rust-toolchain.toml` pins the toolchain; `Cargo.lock` is committed.

The prebuilt `vox` binary also exists at `./vox` (repo root) if `cargo` is
unavailable, and `target/release/vox` after a build.
## The twelve glyphs and the verdict

Every lifted object becomes a word over exactly twelve glyphs. Nothing outside
the twelve parses.

| Glyph | Op | Primitive axis | WORK? |
|-------|-----|----------------|-------|
| `⊢` | VINIT | Dimensionality | open |
| `⊣` | TANCH | Topology | anchor/close |
| `≻` | AFWD | Relational | work |
| `≺` | AREV | Polarity | work (clearing) |
| `⋈` | CLINK | Fidelity | work |
| `⊤` | EVALT | Kinetics | work |
| `∈` | FSPLIT | Scope | δ, opens a frame |
| `∋` | FFUSE | Composition | μ, closes a frame |
| `⊙` | IMSCRIB | Criticality | self-reference |
| `⊥` | EVALF | Chirality | work |
| `⊞` | ENGAGR | Stoichiometry | work |
| `⊡` | IFIX | Winding | work |

**The verdict** (run by `vox verdict`, and applied to every lifted function):

- **T** — closes: opens a fork and fuses it before the anchor.
- **B** — holds a fork open across a terminal (early return, reentrancy).
- **N** — never forked; clean and linear.
- **F** — ill-typed: a `∋` with no `∈` to pair with.

`⊢` opens the word; `∋` marks an address with two or more predecessors. On x86
both are recovered by analysing the instruction stream, not read off any single
instruction (WASM has real opcodes for both). Only `∈`, `∋`, `⊣` move the
verdict; the rest are carried.

**B-cost.** T needs work inside the paired region — bare split+fuse is
μ∘δ=id and verifies nothing. An early return is a fork that never rejoins, and
`∈`-surplus rises with exits (measured on the self-lift: 0.65 / 1.74 / 5.91 /
7.62 / 5.56 mean surplus at 0/1/2/3/4+ exits).

## The main `vox` CLI — every subcommand

`vox --help` is the canonical listing; every entry below was run here.

### Lift and audit

**`vox <file.so|.elf>`** — positional: lift every function, tally verdicts.

```
$ vox target/release/vox
... elf  16070 function(s) by descent
  verdicts  T 270   B 6128   N 9672   F 0
  F is zero when the decoder is in phase with the image.
```

**`vox lift <file>`** — same as the positional form.

**`vox self`** — lift Vox's own image and read it back (the Organism claim):

```
$ vox self
/home/mrnob0dy666/imsgct/Vox/target/release/vox  elf  16070 function(s) by descent
  verdicts  T 270   B 6128   N 9672   F 0
  F is zero when the decoder is in phase with the image.
  open forks against exits — an early return is a fork that never rejoins:
     exits    funcs    mean surplus
        0     14763            0.65
        1      1175            1.74
        2        90            5.91
        3       24            7.62
        4+       18            5.56
```

**`vox word <file>`** — emit the structure word per function:

```
$ vox word GodelOntological.so
0x1534	⊢⊞⋈⊤∈⊙∋⊞⊣
0x15a0	⊢⋈⋈⊤∈⋈⋈⊤∈⋈≻∋⋈⊤∈⋈≻≺∋∋⋈⊣
0x154c	⊢⊞⊞⊣
0x1610	⊢⊙
...
```

**`vox imasm <file>`** — emit the full executable IMASM module (header with
entry, bit width, program header, symbols, relocations, then hex lines):

```
$ vox imasm GodelOntological.so | head
; ⊙ module (elf x86-64)
; entry 0x0
; bits 64
; sym __do_fini 0x15a0
; sym _init 0x1534
; rela 0x2630 0x1560
=0x1000	7f454c4602010100...
```

Pipe it back: `vox imasm <f> > f.imasm` (this is how `*.imasm` files in the
repo were produced), and `vox run f.imasm` executes the module.
### Execute

**`vox run <sym> --args a,b <file>`** — recompile a function to IMASM and RUN
it in the machine that never looks at the original bytes:

```
$ vox run gcd --args 1071,462 "data & docs/corpus_O0.so"
gcd(1071, 462) = 21   [41 steps in the twelve]
```

**`vox run <word.glyphs>`** — decode a glyph-only module and execute it (see
[§12](#key-documents) for the format).

### Glyph modules (lossless serialization)

**`vox glyphs <module.imasm> <output.glyphs>`** — encode a complete module as
a continuous glyph sequence (format `VOXGLYPH1`, documented in
`data & docs/GLYPH_MODULE_FORMAT.md`). Refuses to overwrite existing files.

**`vox unglyphs <word.glyphs> <output.imasm>`** — recover the exact module,
byte-for-byte. Refuses to overwrite existing files.

Example pair:

```
$ vox imasm GodelOntological.so > /tmp/g.imasm
$ vox glyphs /tmp/g.imasm /tmp/g.glyphs
$ vox unglyphs /tmp/g.glyphs /tmp/g2.imasm && diff /tmp/g.imasm /tmp/g2.imasm
```

### Resident circuits

**`vox circuit <module.imasm> [--stdin | hex-mask[:feedback] ...]`** — prepare
once, then switch resident QFT gates. `--stdin` reads gate masks line by line;
each `hex-mask[:feedback]` switches the resident circuit in place.

### Verdicts

**`vox verdict <glyph-word>`** — verdict one word:

```
$ vox verdict ⊢∈⊡⊣
⊢∈⊡⊣
verdict B
```

**`vox verdict --tsv <file>`** — bulk: `name<TAB>word` lines, one verdict per
line.

**`vox --selftest`** — planted open/closed forks across x86, EVM and WASM:

```
$ vox --selftest
  linear routine, never forks            ⊢⊡⊣  N  (expect N)  ok
  fork that merges before terminal       ⊢∈⊡∋⊣  T  (expect T)  ok
  fork held open across the terminal     ⊢∈⊡⊣  B  (expect B)  ok
  merge with nothing to pair             ⊢∋⊣  F  (expect F)  ok
  EVM reentrant (commit in unmerged branch)  B  (expect B)  ok
  EVM guarded  (paths merge before commit)   T  (expect T)  ok
  WASM reentrant (commit + return in branch) B  (expect B)  ok
  WASM guarded  (if merges before commit)    T  (expect T)  ok
selftest OK: the closure law holds on x86, EVM and WASM.
```

### Encoding and pairing

**`vox numeral <decimal>`** — encode a decimal as its IMASM numeral word (the
encode-before-compile step every baked membrane uses):

```
$ vox numeral 42
⊢≻⋈∈⊤∋≻⋈∈⊥∋≻⋈∈⊤∋≻⋈∈⊥∋≻⋈∈⊤∋≻⋈∈⊥∋⊙⊡⊣
```

**`vox pairs <glyph-word>`** — the pairing: every region, what it holds, what
is left open:

```
$ vox pairs ⊢∈⊡⊣
OPEN   CLOSE  SPAN   WORK  INTERIOR
letters      4
regions      0 paired, 0 substantial
unopened   1 division(s) at 1
unopened   0 rejoining(s) at -
verdict      B
```

### Factorization over IMASM tapes

**`vox factor <N>`** — shape-routed full factorization:

```
$ vox factor 15
15 = 3 x 5
```

**`vox scout <N>`** — read the shape of N and hand the factor:

```
$ vox scout 15
N=15
shape: near-root, closed at frontier step 0
  15 = 3 x 5  [frontier]
```

**`vox factor-operator resolve|full <N>`** — the CL9NK moat resolver over
folded tapes.

**`vox morphism-factor <native-numeral-word>`** — factor entirely over IMASM
tapes (the `vox_factor.sh` script drives it with a baked word).

**`vox construct-carrier <operator-word>`** — decompose factoring morphisms
and EML frame transport.

**`vox factor-with <operator-word> <n-word>`** — factor N on a carrier built
from the operator word.

**`vox extract-factor <factor-carrier-word>`** — passive ≡c extraction from an
already factor-bearing trace.

**`vox coprime <base> <N>`** — validate that a phase base is a unit modulo N
(prints `coprime` when it is):

```
$ vox coprime 7 15
coprime
```

### Membranes

**`vox membrane tower <levels>`** — build a complete bidirectional tower.

**`vox membrane bridge <N> <m>`** — coupled divisor-ring W_t trace over IMASM
tapes.
### Substrate lanes

Each lane lifts its native code into glyphs and verdicts the closure.

**`vox evm <hex>`** — lift EVM bytecode, verdict its closure:

```
$ vox evm 60806040
EVM    N  ⊢
```

**`vox wasm <hex>`** — lift a WASM function body (hex of the module or body):

```
$ vox wasm 0061736d01000000
WASM   N  ⊢
```

**`vox hex <hex>`** — lift raw machine-code hex, verdict its closure:

```
$ vox hex 554889e5
HEX    4 bytes  raw  x86-64  1 function(s) by descent
  0x00001000  N  ⊢⋈⋈
  verdicts  T 0   B 0   N 1   F 0
```

**`vox rna <seq> [--dialect mito]`** — lift a coding sequence, verdict the
transcript. Codons read; the 12 promoted amino acids biject the 12 axes:

```
$ vox rna AUGUUUGCC
CODON   AA    AXIS            GLYPH
AUG     Met   Dimensionality  ⊢
UUU     Phe   Fidelity        ⋈
dialect  standard
codons   2 promoted of 2 read
frame    AUG at offset 0
word     ⊢⋈
stop     (none: the sequence ran out before a stop)
verdict  N
```

`--dialect mito` switches to the mitochondrial code table (the table itself is
generated from the Lean that proves it — `gen_genetic_table.py`, no drift).

**`vox aa <seq>`** — lift a protein (one-letter residues), verdict the fold:

```
$ vox aa MKTVR
POS     AA    AXIS            GLYPH
1       Met   Dimensionality  ⊢
2       Lys   Stoichiometry   ⊞
source   sequence
residues 2 promoted of the sequence
word     ⊢⊞
verdict  N
```

**`vox fasta <file>`** — lift a protein from FASTA.

**`vox pdb <file>`** — lift a protein from a PDB (CA per residue).

**`vox glyco <seq|file>`** — locate the glycosylation boundary interfaces
(N-linked Asn-X-S/T sequons are determinate; O-linked Ser/Thr candidates are
admitted but carry no mark in the peptide word):

```
$ vox glyco NPE
source   sequence
peptide  ∈⊡   (2 promoted residues, the bulk)
verdict  B   (of the peptide backbone)
BOUNDARY INTERFACES — where a glycan meets the peptide:
  N-linked sequons (Asn-X-[Ser/Thr], X≠Pro) — determinate: none
  O-linked candidates (Ser/Thr) — admitted, not determinate, ...: 0
```

**`vox compile <seq> [--code standard|mitochondrial] [--pdb <path>]`** — the
full pipeline, both ends. RNA/DNA in gives a compiled protein with real fold
info (Chou-Fasman secondary structure, heuristic tertiary contacts, a real 3D
backbone via B4-Ramachandran-NeRF); protein in gives RNA/DNA back out
(Frobenius-preferred codon per residue, full degeneracy) with the same fold on
the input. Direction auto-detects from the alphabet. `--pdb` writes a real
PDB file, readable by `vox pdb`.

**`vox pyc <file.pyc>`** — lift every code object in a `.pyc`, verdict each.
The magic must match the running interpreter — opcode numbers differ between
CPython versions, so Vox refuses a foreign magic instead of lifting wrong
bytes:

```
$ vox pyc __pycache__/vox.cpython-310.pyc
__pycache__/vox.cpython-310.pyc: magic 6f0d0d0a is not this interpreter's
(3.12.12); opcode numbers differ between versions, so V⊙x will not lift it
```

**`vox safetensors <file>`** — lift a HuggingFace safetensors file, verdict
each tensor.

### Classification

**`vox classify <mn> [ops]`** — the glyph an instruction lifts to:

```
$ vox classify jmp
jmp ⊙
```

**`vox tables <file> <symbol>`** — shift a function one nibble, verdict a
random baseline of the same length, and flag any resync run far longer than
chance — an embedded constant table (witness sets, factor bases, curve
parameters), located purely from the binary and shown at its real alignment.
The symbol must exist in the file's symbol table:

```
$ vox tables GodelOntological.so main
vox tables: symbol 'main' not found
```

### Organism

**`vox self`** — see [Lift and audit](#lift-and-audit). The self-lift is the
organism claim: every function of Vox, in phase, F zero.
## Baked membranes

The house pattern for factorization binaries: the decimal is consumed at BUILD
time by `vox numeral` (encode-before-compile), the resulting IMASM word is
baked into the binary by `build.rs` (via `RUSTFLAGS`/env at compile), and the
run is pure execution of a word that already contains N — no runtime input, no
decimals. Every script in [§7](#shell-scripts) is a variant of this pattern.

```bash
$ ./factor_one.sh 1000036000099     # bakes N, rebuilds, runs factor_one
$ ./membrane_one.sh 1000036000099   # same shape for `membrane`
$ ./perfect_one.sh 105 3            # depth-n perfect membrane, default depth 2
$ ./eml_factor_one.sh 105 7         # EML-first carrier, optional phase base
$ ./hyperstack_one.sh 105 phase shor fib   # emits membranes/factor_105_phase-shor-fib
$ ./frame_factor_build.sh 105 4     # N + eval-frame width imscribed, no input
$ ./factor_2adic_membrane.sh 1000036000099 membranes/case 7 10
```

`hyperstack_one.sh` and `frame_factor_build.sh` EMIT the baked binary rather
than run it; the emitted path is printed. `factor_2adic_membrane.sh` prints
the path of a self-contained membrane binary (phase base and lift radix both
default to 2).

## The Rust binaries

All live in `src/bin/`, declared in `Cargo.toml` (autobins off), built by
`cargo build --release`, output to `target/release/`. Grouped by function:

**Mathematical one-shot factorizers / membranes** (each takes a baked word at
compile time; run them or build them via their script):
`abc_one`, `binomial_one`, `divisor_one`, `lcm_one`, `landau_one`,
`schutte_one`, `reptiling_one`, `tripsum_one`, `perfect_one`, `factor_one`,
`factor_eml_one`, `factor_2adic`, `factor_2adic_bigint`, `frame_factor_one`,
`hyperstack_one`, `orbit_uncapped`, `semiprime_shot_cli`,
`baked_target`, `baked_target_bounded`, `phaseB_fac`.

**Shor / QFT family:** `shor_one`, `shor_big_one`, `shor_big_batch`,
`shor_bvalsd_one`, `shor_cli`, `qft_circuit`, `gpu_shor_one`, `gpu_bvalsd`,
`braid_gens`.

**Carrier / router / judge / trace / tape / frame families** (the CL9NK moat,
trace algebra, and tape operations; each is a small CLI over baked or given
words):
- Carriers: `carrier_cli`, `carrier_identity_cli`, `shot_carrier`
- Routers: `meta_router_cli`, `meta_router_g_cli`, `router_g_cli`,
  `router_marks_cli`, `router_rr_cli`, `router_self_cli`, `router_word_cli`
- Judges: `judge_g_cli`, `judge_judge_cli`, `judge_router_g_cli`
- Traces: `trace_algebra_cli`, `trace_measure_cli`, `trace_reentry_cli`,
  `trace_replay_cli`, `trace_word_cli`
- Tapes: `tape_algebra_words_cli`, `tape_delete_cli`, `tape_factor_cli`,
  `tape_fuse_cli`
- Frames/reentry: `nested_frame_cli`, `frame_work_cli`, `full_loop_cli`,
  `terminal_fixpoint_cli`, `transform_verdict_cli`, `self_repair_word_cli`,
  `rewrite_word_cli`
- Reduction/confluence: `confluence_cli`, `reduce_resident_cli`,
  `history_reduce_cli`, `cluster_factor_cli`
- Towers/words: `tower_words_cli`
- Membrane shapes: `base_mod_membrane`, `cylinder_membrane`,
  `double_arev_membrane`, `reduce_delete_membrane`, `ring_membrane`,
  `single_frame_membrane`, `winding_membrane`, `multi_quantum_membrane`

**Encoding / phase:** `phase_baked`, `phase_case_encode`, `phase_unbraider`,
`phase_vessel_resident`.

**Factorization-31 / verification:** `factorization_31_batch`,
`factorization_31_one`, `factorization_31_resident`, `verify_31_tower`,
`factor_bench`, `factor_2adic_bigint`.

**Probes / fixed point / Gödel:** `semiprime_probe`, `fixed_point_baked`,
`godel`, `membrane`.

**The main auditor:** `vox` (the CLI documented above).

Every binary is standalone and takes no runtime input unless it is a `_cli`
that reads a word from argv — the `_one` names mean "baked single run".
`src/main.rs` is the `vox` binary; `src/lib.rs` is the library every other
binary links against.
## Shell scripts

All `bash`/`sh`, all in the repo root. The first column is what each does;
every one was read in full.

**Baked-membrane builders** (see [§5](#baked-membranes)):

| Script | Usage | Notes |
|--------|-------|-------|
| `bench.sh` | `./bench.sh <file of "label decimal" lines>` | Bakes a batch of IMASM numerals into one sealed `factor_bench` binary; prints the table only when the whole batch is done — sealed, no peeking |
| `factor_one.sh` | `./factor_one.sh <decimal N>` | Bakes N, rebuilds `factor_one`, runs it once |
| `membrane_one.sh` | `./membrane_one.sh <decimal N>` | Same shape for `membrane` |
| `perfect_one.sh` | `./perfect_one.sh <decimal value> [depth]` | Depth-n perfect membrane (default depth 2) |
| `eml_factor_one.sh` | `./eml_factor_one.sh <decimal N> [phase-base]` | EML-first full carrier; emits `membranes/baked_eml/factor_<id>/` with input words + binary |
| `hyperstack_one.sh` | `./hyperstack_one.sh <decimal N> [carrier types...]` | Default types `phase shor fib`; EMITS `membranes/factor_<N>_<types>` binary, not run here |
| `frame_factor_build.sh` | `./frame_factor_build.sh <decimal-N> [frame-width]` | N + eval-frame width imscribed into the executable; width ≥ 2 |
| `factor_2adic_membrane.sh` | `./factor_2adic_membrane.sh <decimal-N> [output-binary] [phase-base] [lift-radix]` | Prints the path of the self-contained membrane binary; phase base and lift radix default to 2 |
| `multi_membrane_one.sh` | bakes `a`, `N` and register width | No runtime input |
| `membrane_run_15.sh` | pre-encoded numerals | No runtime encoding |
| `membrane_run_big.sh` | pre-encoded numerals, big cases | No runtime input |

**Pipeline / corpus:**

| Script | Usage | Notes |
|--------|-------|-------|
| `build_corpus.sh` | `./build_corpus.sh` | Builds `corpus_O0.so`…`corpus_Os.so` (and the 32-bit set) from `corpus.c` at every gcc opt level; hand the results to `python3 verify.py corpus_O*.so` |
| `sweep_binaries.sh` | `./sweep_binaries.sh OUT.bin ROOT [ROOT...]` | Lifts a whole tree of binaries into one IMASM module stream; written because a hand-pasted path list did not survive line-wrapping — a list you generate cannot be wrapped |
| `qft_circuit.sh` | `./qft_circuit.sh [--help] ...` | Resident QFT circuit driver over `vox circuit` |
| `vox_factor.sh` | no args | Reads the three pre-encoded words from `membrane_words.imasm` and runs `vox morphism-factor` |
| `vox_membrane.sh` | no args | Reads `membrane_word.imasm`, compiles it via G-mOMonadOS `compile_word`, runs the one-shot binary |
| `glyph_membranes.sh` | no args | Generates standalone glyph sequences for binomial/lcm/divisor/landau/…, recovers the exact modules, executes the controls; results in `membranes/glyphs/` |

**Fix / maintenance:**

| Script | Purpose |
|--------|---------|
| `apply_morphism_projection_carry_fix.sh` / `_v2.sh` | Apply the morphism-projection carry fix to the source |
| `fix_divisor_bridge_exact.sh` | Fix the divisor bridge exactness (takes a dir arg) |
| `repair_factor_extract.sh` | Repair the factor_extract trace framing |
| `fix_rustdoc_pseudocode.sh` | Drive `fix_rustdoc_pseudocode.py` |

## Python entry points

The Python surface is the CPython lane plus the tooling around it. Top-level
and `py/` hold parallel copies of the shared modules (`vox_decode.py`,
`vox_pe.py`, `vox_x86.py`, `genetic_table.py`, `imasm_module.py`,
`imasm_vm.py`, `imasm16_3_core.py`, `mem_wordio.py`, `measure.py`,
`tower_factor.py`, `test_decompiler.py`, `gen_genetic_table.py`, `gen_hard.py`,
`gen_pyc_table.py`).

| File | What it is |
|------|-----------|
| `vox.py` | The CPython lane: control-flow closure auditor for real bytecode (the `.pyc` lifter's Python side) |
| `vox_pe.py` | Vox's own PE reader — the last dependency gone |
| `vox_decode.py` | Vox's own x86-64 decoder, the standalone lane |
| `vox_decompiler.py` | The decompiler — the missing recompose stage of the circular pipeline |
| `vox_x86.py` | Which x86 backend Vox reads through |
| `vox.imasm.txt` / `vox.txt` / `vox.word` | The self-lift: Vox's own image as an IMASM module / text / word |
| `imasm_module.py` | The module format: a word in the twelve, and the payload each glyph carries |
| `imasm_vm.py` | A machine that runs an IMASM module |
| `imasm16_3_core.py` | The SIXTEEN_3 core (the verdict engine) |
| `mem_wordio.py` | IMASM numeral encode/decode: `python3 mem_wordio.py enc 42` / `dec <word>` |
| `genetic_table.py` | The 12 promoted amino acids → 12 axes; generated from the Lean that proves it |
| `gen_genetic_table.py` | Regenerates `genetic_table.py` from the Lean (no drift) |
| `gen_pyc_table.py` | Generates `src/pyc_table.rs` from the running interpreter |
| `gen_hard.py` | Generate hard-shape semiprimes: large factors, wide gap, neither p−1 nor p+1 smooth |
| `measure.py` | How much of a binary is program, and how much is encoding |
| `membrane_factor.py` | Factor-membrane driver |
| `membrane_word.io` | — |
| `ladder_word.py` | Generate the period-finding ladder as closing IMASM words, two forms |
| `tower_factor.py` | IMASM tower driver — IMASM running on IMASM operating on IMASM |
| `render_carrier_diagram.py` | Render an IMASM sequence as a symbolic wiring (circuit) diagram → SVG |
| `probe_open.py` | Check whether perfect_one's decomposition on OPEN values reveals a factor |
| `verify.py` | The recompile is executable, or it is not: runs corpus functions natively via ctypes AND as IMASM over identical inputs, compares |
| `test_decompiler.py` | Test the μ∘δ=id loop closure for the circular pipeline |
| `finstant.py` | Finstant driver |
| `crystalgebra.py` | Crystalgebra |
| `umnifold.py` / `numnifold.py` | Umnifold / numnifold (unified manifold numerics) |
| `hsoa_shor_shot.py` | Shor shot: (a,N) → joint state → QFT → continued fractions |
| `hsoa_shor_state.py` | HSOA Shor state |
| `hsoa_membrane_check.py` | HSOA readout through compiled-binary execution and serialized membrane via Vox VM |
| `lane_close_probe.py` | Exploit the small N//(L0·L1) and forced residues |
| `factor_membrane_check.py` | Audit, lift, and execute baked factor cases through complete Vox membranes |
| `phase_scaling_check.py` | Measure sparse phase construction on the fixed 175-bit balanced control |
| `baked_case.py` | Build-only inputs, canonical IMASM encoding, retained per-case executable |
| `baked_phase_check.py` | Compile one executable per IMASM case; execute without numeric inputs |
| `semiprime_campaign.py` | The semiprime campaign (see `semiprime_campaign.md`) |
| `tv_battery.py` | Run the baked dialectic factor producer over TV.txt values |
| `go.py` | Relate perfect_one delta lanes to known factors; every relation tested exactly |
| `runz.py` | The loop written as recursive DFS over the s = p_k + q_k branching |
| `patch.py` | Patch driver |
| `fix_rustdoc_pseudocode.py` | Fix rustdoc pseudocode in the source |
## Source layout

`src/` is one library (`lib.rs`) plus `main.rs` (the `vox` binary) plus
`src/bin/` (documented in [§6](#the-rust-binaries)). The library modules, by
role:

**Core lift + machine:** `vox.rs` (the classifier and the verdict),
`vox_decode.rs` (the x86-64 decoder), `loader.rs` (ELF/PE/Mach-O, both
widths), `x86.rs` (the x86 backend), `imasm_module.rs` (the module format),
`imasm_vm.rs` (the machine that runs a module), `glyph_module.rs` (the
glyph-only serialization), `lanes.rs` (lane dispatch).

**Lanes:** `pyc.rs` + `pyc_table.rs` (the CPython lane; the opcode table is
generated by `gen_pyc_table.py` from the running interpreter), `safetensors.rs`
(+`.bak`), `genetic.rs` + `genetic_table.rs` (RNA/AA/fasta/pdb/glyco/compile),
`protein.rs`, `fold.rs`, `fold3d.rs` (Chou-Fasman + B4-Ramachandran-NeRF
backbone).

**Factorization / membranes:** `morphism_factor.rs`, `factor_operator.rs`,
`factor_2adic.rs`, `factor_extract.rs` (+`_complete`, `lib_factor_extract.rs`,
`main_factor_extract.rs`), `carrier.rs`, `phase_partners.rs`, `phase_word.rs`,
`phase_unbraid.rs`, `divisor_membrane.rs`, `divisor_ring.rs`,
`baked_membrane.rs`, `complete_membrane.rs`, `membrane_complex.rs`,
`membrane_state.rs`, `landau_membrane.rs`, `perfect_membrane.rs`,
`prime_power_membrane.rs`, `reptiling_membrane.rs`, `schutte_membrane.rs`,
`tripsum_membrane.rs`, `factorization_31_membrane.rs`,
`fde_shor_membrane.rs`, `shor_braid.rs`, `shor_qft.rs`, `sieve.rs`,
`hadamard_gate.rs`, `hadamard_factor_bridge.rs`, `abc_iutt.rs`, `fold.rs`.

**Fixed point / dialectic / trace:** `fixed_point_protocol.rs`,
`fixed_point_imasm.rs`, `fixed_point_membrane.rs`, `fixed_point_quantum_*.rs`
(membrane, phase, readout, relation), `fixed_point_reentry.rs`,
`fixed_point_hypernest.rs`, `fixed_point_word_arithmetic.rs`,
`dialectic_certificate.rs`, `dialectic_reentry.rs`, `reentry_certificate.rs`,
`imscription_cycle.rs`, `trace_object.rs`, `trace_word.rs`,
`trace_algebra.rs` (+`_factor_extract`, `_complete`), `tape_delete.rs`
(+`tape_delete_trace_framing.rs`), `terminal fixpoint` and friends under
`src/bin/`.

**Routers / judges / nested frames:** `meta_router.rs`, `router_store.rs`,
`router_marks.rs`, `router_object.rs`, `judge_g.rs`, `nested_frame.rs`,
`frame_work.rs`, `reducer_store.rs`, `producer_provenance.rs`,
`provenance_envelope.rs`, `winding_readout.rs`.

**Gödel:** `godel_analyzer.rs`, `godel_calculus.rs`, `godel_product.rs`.

**Circuit:** the substrate round trips `x86 → IMASM → RNA → IMASM → x86` and
`RNA → IMASM → x86 → IMASM → wasm → IMASM → AA` (documented in
`data & docs/CIRCUIT.md`; each leg is a retraction, μ∘δ=id on glyphs,
δ∘μ idempotent — the identity holds exactly on the image of δ).

`src/main.rs.tmp` is a stale copy of `main.rs`, not built.

## Top-level files

The repo root is the working bench. By kind:

**Binaries and sources:** `vox` (prebuilt), `vox.py` (the CPython lane),
`vox_pe.py`, `vox_decode.py`, `vox_decompiler.py`, `vox_x86.py`,
`vox.imasm.txt` / `vox.txt` / `vox.word` (the self-lift), `vox.semantics`
patches (`vox_semantics_factor_object.patch`,
`vox_semantics_factor_object_repo.patch`), `corpus.c` (the 13-function
verification corpus), `GodelOntological.so` (+`.glyphs`, +`.so.imasm`),
`complete_membrane.rs` (+`.imasm`, +`.imasm.imasm`), `ffz` (+`ffz.imasm`),
`membrane_word.imasm` (+`.ref`), `membrane_words.imasm` (+`.imasm`, +`.ref`),
`carrier_nested.imasm` (+`carrier_nested.md`), `principal_word.imasm`,
`wrd.glyph` (+`wrd.glyph.imasm`), `3fea.glyphs` (61 MB glyph module),
`≺` (a 14 KB glyph file), `build.rs`, `build_corpus.sh`, `Makefile`,
`Cargo.toml` (+`.preautobins`), `Cargo.lock`, `rust-toolchain.toml`,
`UNLICENSE`.

**Patches:** `factor_extract.patch`, `factor_extract_complete.patch`,
`factor_extract_trace_framing_fix.patch`, `morphism_projection_carry_fix.patch`
(+the two apply scripts), `apply_morphism_projection_carry_fix_v2.sh`.

**Carrier diagrams:** `carrier_38127fc2.svg`, `carrier_51591af9.svg`,
`carrier_9b26321a.svg`, `carrier_a2846d69.svg`, `carrier_b3b7ed79.svg` —
rendered by `render_carrier_diagram.py`.

**Battery / control JSONL:** `semiprime-baked-control.jsonl`,
`semiprime-baked-input-control.jsonl`, `semiprime-braid-175.jsonl`,
`semiprime-calibration-175.jsonl`, `semiprime-close-extension.jsonl`,
`semiprime-close-sweep.jsonl`, `semiprime-no-timeout-control.jsonl`,
`semiprime-phase-175.jsonl`, `semiprime-symbolic-175.jsonl`,
`semiprime_campaign.md`, `factor-binary-audits-close(-next).jsonl`,
`factor-binary-audits-multiplier(-next).jsonl`, `factor-membrane-focused-*.jsonl`
(arithmetic, closure, clock-fixed, trace, retry, larger),
`larger-close-battery*.jsonl`, `larger-multiplier-battery*.jsonl`,
`hsoa-larger-battery.jsonl`, `hsoa-membrane-results.jsonl`,
`hsoa-membrane-verified.jsonl`, `phase-sparse-scaling(-baked).jsonl`.

**First-run logs:** `tv-first*.stdout` / `tv-first*.stderr` (audit, braid,
phase, resident, smart, smart-unbounded) — the first battery runs, kept for
the record.

**Notes:** `semiprime_campaign.md`, `shorstopic.md`, `carrier_nested.md`,
`membrane_sources.json`, `testvals.txt`, `commit.txt` (the current commit
note: *"I test all four nested EML arms on the 160-digit carrier, preserve
standalone phase behavior, and retain the rebuilt contained binary."*),
`.aider.chat.history.md`, `.gitattributes`, `.gitignore`.
## Directories

| Directory | Contents |
|-----------|----------|
| `src/`, `src/bin/` | The crate: library modules + ~95 declared binaries (see [§6](#the-rust-binaries), [§9](#source-layout)) |
| `tests/` | 36 integration test files: `dialectic_*` (certificate cuts, syzygy, four judgment/total, neutral every cut, reentry, tape span, word execution), `factor_extract_*` (confluence, diamond restart, next, persistence quotient, production, strategy wire, stress), `imscription_cycle_*` (arbitrary width, every cut, scale, wire, relation ownership), `provenance_syzygy*`, `producer_route_syzygy`, `reentry_certificate*`, `shor_braid_acceptance`, `shor_orbit_closure`, `suffix_envelope_fibre`, `cfg_framing.rs`, plus a `baked/` subdir |
| `membranes/` | 1638 run artifacts: per-case baked membranes (`factor_<N>_<route>`, `factor_2adic_*` with `.stdout.txt`/`.stderr.txt`/`.imasm`/`.glyphs` triples), `baked_eml/`, `bigelf/`, `bigimasm/`, `dual_nested_*`, and `glyphs/` (glyph-only modules + `SHA256SUMS` + recovered `.imasm` + timing files — the `glyph_membranes.sh` output) |
| `data & docs/` | 319 files: the five key documents ([§12](#key-documents)), `corpus_O*.so` + `corpus32_O*.so` (the verification corpus at every opt level), `after_*` / `speed_*` audit+time triples per math binary, `cfg_framing_*` logs, `circuit_*` logs, `membrane_*` logs/notes/tables, carrier SVGs, `md/` subdir |
| `measurements/` | 108 files: Shor ladders (`shor_native_ladder`, `shor_large_ladder`, `shor_separated_*`), `shor_bvalsd_*` build/run/time logs (15–30) |
| `stress_runs/` | 18 stress logs: `abc`, `divisor`, `factor`, `landau`, `reptiling`, `schutte`, `shor`, `tripsum` — each with a `_fixed` companion |
| `probes/` | Float-lane controls: `float_native.log`, `float_vox.log`, `membrane_float_control` (+`.imasm`, +`.rs`) |
| `imscrb/` | Imscribed native objects: `ChemDraw.exe`, `XboxPcApp.imscrb`, `Discord_analysis.imscrb` |
| `examples/` | `demo.py` (end-to-end example), `membrane_probe.rs` |
| `cuda/` | `fde_shor.cu` — the GPU Shor membrane kernel |
| `document_review/` | Rendered review pages (`membranes-*.png`, `final-*.png`) |
| `.github/` | CI workflow: runs `make test` on push/PR |
| `.venv/`, `__pycache__/` | Python environment and bytecode caches |
| `.claude-project/`, `.git/` | Project and VCS metadata |

## Key documents

| Document | What it settles |
|----------|-----------------|
| `README.md` (root) | The short claim: pancosmic/disassembling/re-compiling/organism, build, lanes, the 0-mismatch corpus claim, the B-cost lesson |
| `README_backups/Vox_README.md` | The full 382-line version of the README |
| `data & docs/VOX.md` | The auditor: what the verdict reads, why the decoder refuses rather than guesses (the load-bearing decision; coverage went 15%→41%→44%→100% on `/bin/ls`), the two defects it surfaced (shifted ELF field offsets, quadratic merge lookup) |
| `data & docs/GLYPH_MODULE_FORMAT.md` | The glyph-only executable module format v1: `⊢∈ … ∋⊡⊣` frame, byte records with `⊤`/`⊥` bit payloads, the `VOXGLYPH1:<len>:<FNV1a64>` header, `vox glyphs`/`vox unglyphs`/`vox run <.glyphs>` semantics |
| `data & docs/IMASM_NATIVE_COMPUTE.md` | The machine inside the twelve: `parasm` (B4-valued register machine; JT/JF/JB/JN as FSPLIT arms, ROTAT as loop, CALL/RET as CLINK), the crystal filesystem as unbounded store; the Replicating Code |
| `data & docs/CIRCUIT.md` | The two substrate round trips and what closes: the WORD, not the bytes. Each leg is a retraction: μ∘δ=id on glyphs, δ∘μ idempotent, identity exactly on the image of δ (the canonical section) |
| `data & docs/CFG_FRAMING_FIX.md` | The CFG framing fix and its regression evidence |
| `semiprime_campaign.md`, `shorstopic.md`, `carrier_nested.md`, `membrane_sources.json` | Campaign records: the semiprime campaign, the Shor stop-topic, the nested carrier, and the membrane source manifest |

## Verification and tests

```bash
make test                      # cargo test --lib (also what CI runs)
make test-fixed-point          # cargo test --lib fixed_point_protocol
make check                     # cargo check --lib
vox --selftest                 # the closure law on planted forks, x86 + EVM + WASM
```

**The corpus claim.** `corpus.c` holds 13 functions (arithmetic, div/mod,
loops, SSE, recursion, calls, stack arrays, switch table, fn-pointer
dispatch). The check:

```bash
./build_corpus.sh && python3 verify.py corpus_O*.so
```

builds the corpus at `-O0…-O3,-Os` (64- and 32-bit), runs every function
twice — natively via ctypes and as recompiled IMASM — over identical inputs,
and compares: **0 mismatches**, including Fibonacci at depth 29 (28M machine
steps).

**Decoder coverage:** 100% of bytes decoded, no stops, on `/bin/true`,
`/bin/ls`, `/bin/bash`, `/usr/bin/git`, `/usr/bin/python3`, and the kernel's
own binary (690,031 instructions across 3.2 MB of `.text`). Verdicts
distribute by shape, not size: `_init` thunks close at T, PLT stubs run N, a
real `.text` read whole sits at B.

## Edges and refusals

- **Unknown opcode:** the walk stops and reports the address and bytes
  (`stopped at +0x20da9a on an opcode the decoder does not know: 49 92 4c 87 …`).
  A partial lift always reads as partial; guessing a width would desync the
  stream and the verdict would be fiction.
- **Unknown architecture:** refused outright, never misread.
- **Foreign `.pyc` magic:** refused (opcode tables differ between CPython
  versions) — see the `vox pyc` example in [§4](#substrate-lanes).
- **Missing symbol for `vox tables`:** `vox tables: symbol '<sym>' not found`.
- **Glyph round-trip writes:** `vox glyphs` / `vox unglyphs` refuse to
  overwrite existing files.
- **The 50M-step cap:** long runs are capped; the cap is a boundary, not a
  wall.
- **`⊙`/`∋` bookkeeping:** `⊢` opens the word, `∋` marks two-plus
  predecessors; bare `⊢∋⊣` (merge with nothing to pair) is F, and bare
  split+fuse with no work inside is the identity, not a close — T needs work
  in the paired region.
- **`src/main.rs.tmp`** and `safetensors.rs.bak` are stale copies, not built.
