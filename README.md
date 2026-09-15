# V⊙x

The Pancosmic Disassembling Re-Compiling Organism. Every word names something you can run. Single Rust crate, zero external crates.

**Pancosmic.** One lift, every substrate: native x86-64 (ELF/PE/Mach-O, both widths), EVM bytecode, WASM function bodies, CPython `.pyc`, and coding sequences (12 promoted amino acids biject the 12 axes — a gene is already a word). A merge is a merge whether `JUMPDEST`, `end`, or two-predecessor jump target.

**Disassembling.** Own loader, decoder, machine — no capstone/pefile/runtime. Refuses unknown architectures rather than misreading them.

**Re-Compiling.** The lift IS the program: native code recompiled to an executable IMASM module, run in a machine that never looks at the original bytes (Ackermann, SSE, jump tables, function-pointer calls, 5 opt levels, both widths).

**Organism.** `vox self` lifts its own image: every function, in phase, F zero.

```bash
cargo install --path .
vox run gcd --args 1071,462 corpus_O0.so   # gcd = 21, 41 steps in the twelve
vox self
vox pyc module.pyc
```

## Build and run

Complete executable [glyph-only membrane words](membranes/glyphs/README.md) are
available alongside the annotated modules. `vox run file.glyphs` executes them;
`vox glyphs input.imasm output.glyphs` generates one without dropping payloads.
See the [versioned format and verification](GLYPH_MODULE_FORMAT.md).

```bash
cargo build --release
vox run <sym> --args a,b <file>  # recompile + run
vox imasm <file> | vox word <file> | vox disasm <file> | vox <file>  # module/word/stream/audit
vox verdict ⊢∈⊡⊣ | vox evm HEX | vox wasm HEX | vox rna SEQ | vox --selftest
```

## Verdicts and B-cost

T closes, B holds a fork open across the cycle, N never forked, F ill-typed (∋ with no ∈). T needs work inside the paired region — bare split+fuse is μ∘δ=id and verifies nothing. B-cost lesson from self-lift: early return = fork that never rejoins; ∈-surplus rises with exits (0.28/5.26/12.91/15.00 at 0/1/2/4+ exits), so `vox self` reports the residual after exits are paid.

## Claim (checked, not decorated)

Every function in `corpus.c` (13 fns: arithmetic, div/mod, loops, SSE, recursion, calls, stack arrays, switch table, fn-pointer dispatch) runs twice — natively via ctypes and as IMASM — over identical inputs at `-O0..-O3,-Os` (`build_corpus.sh`): **0 mismatches**, incl. Fibonacci depth-29 (28M machine steps).

## Lanes, auditor, layout

Lanes: x86 (lifts+verdicts+runs), EVM/WASM/genetic (lift+verdict); genetic table generated from Lean (`gen_genetic_table.py` → both languages, no drift); CPython lane in `vox.py`. Auditor corollary: open fork across commit/return (Solidity reentrancy, early return, WASM escaped-`if`) all lift to `⊢∈⊡⊣ → B` — no patterns, no heuristics. Edges: unknown opcode → `None` + distance walked (never silent); tiny syscall subset (`-ENOSYS`); 50M-step cap; recursive-descent audit with honest coverage. Sources: `loader/x86/imasm_module/imasm_vm/vox/vox_decode/lanes/main.rs` + `corpus.c` + `vox.py`; mOMonadOS links this crate. Unlicense.

Full 382-line version: `README_backups/Vox_README.md`.

μ∘δ = id
