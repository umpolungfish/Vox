# V⊙x

A disassembler that recompiles. V⊙x reads a program's bytecode and rewrites it
as a word in the twelve-opcode Imscribing Grammar — one glyph per instruction,
nothing dropped — so a binary compiled for any instruction set comes back out in
a single alphabet that belongs to none of them.

```bash
python3 vox.py --imasm out.imasm program.exe
```

Two lifts run over the same disassembly. The **recompiler** (`--imasm`) is
total: every decoded instruction lands on exactly one of the twelve axes, in
order, per function, labelled by address, so the emitted module is the program
rewritten rather than a summary of it. The **auditor** (the default) keeps only
the control-flow skeleton, because a closure verdict does not need the
arithmetic — a fork that commits state or returns before its paths rejoin does
not close, and that open fork is the shape of a whole class of bugs.

The auditor is the smaller thing. It falls out of the recompile, because once a
program is a word you can ask the Grammar anything about it, and *does it close*
is only the first question.

## How small

`measure.py` recompiles a binary and measures what came out. On a 596 KB x86-64
build: the structure is 1.9% of the code bytes, the operands another 11.8%, and
a lossless recompile that still runs lands at 13.7% — 7.3x smaller than the
original, 2.3x smaller than zlib on the machine code, and still executable. See
[MEASUREMENTS.md](MEASUREMENTS.md), including why the 1.9% is not a compression
figure.

## What it is

V⊙x lifts the control-flow graph of real bytecode to a twelve-opcode word and
runs a paraconsistent verdict engine (the SIXTEEN_3 trilattice) over it. No
pattern list, no language rules, no heuristics tuned per target. The verdict is
Belnap FOUR:

- **T** the control flow closes.
- **B** a fork is held open across a commit or a return. This is the finding.
- **N** a linear routine that never forked. Clean, nothing to weigh.

Four front ends today, one law behind them:

| ISA | input | fork | merge |
|-----|-------|------|-------|
| CPython | a `.py` file (via `dis`) | conditional jump | a target with two or more predecessors |
| EVM | `--evm HEX` | `JUMPI` | `JUMPDEST` reached from two paths |
| WASM | `--wasm HEX` | `if` | its `end`, unless a `return`/`br` escaped first |
| native x86 | a PE or ELF binary (auto-detected) | conditional `jcc` | a jump target reached from two paths |

The native lane needs `capstone` and `pefile` (`pip install capstone pefile`).
The other three lanes are standard library only.

## Why it exists

The famous bugs are the same bug in different clothes. A Solidity reentrancy that
calls out and writes state before the branch rejoins, a Python function that
commits and returns early inside an `if`, a WASM store in an escaped `if`: all of
them commit before control comes back together. Lift each to the grammar and they
land on the identical word:

```
⊢∈◻⊣  →  B
```

One bug shape, three unrelated bytecode formats, one verdict. The leak lives in
the topology of the control flow, not in the syntax, so V⊙x reads it off the
topology.

## How to use it

```bash
python3 vox.py --selftest                 # EVM + WASM vulnerable/safe pairs
python3 vox.py examples/demo.py           # every function in a Python file
python3 vox.py --evm 600160075755005b00   # an EVM bytecode string
python3 vox.py --wasm 20000440410141003602000f0b0b   # a WASM function body
python3 vox.py some_program.exe           # a native x86 PE binary (auto-detected)
```

`B` in the output is a finding: a fork held open across a commit or return.
See `USER.md` for the full reading, the verdict values, and how to interpret the
lifted word.

## How it works

Each front end walks the real bytecode (not the source) and emits the load-bearing
structure in the alphabet: ⊢ at entry, ∈ at a branch, ∋ at a true merge, ◻ at a
state write, > at an external call, ⊣ at a return.
Everything else is noise and is dropped. The resulting word runs through the
SIXTEEN_3 engine, which computes the μ∘δ closure verdict: a fork that does work
on its arms and rejoins closes (T); a fork whose commit or return escapes before
the rejoin stays open (B).

Reading the real bytecode matters. CPython 3.12 duplicates a common continuation
into both arms of an `if`/`else`, so a merge you wrote in the source can be
compiled away. V⊙x reports what actually executes.

## Layout

- `vox.py` the auditor and its three front ends.
- `imasm16_3_core.py` the SIXTEEN_3 trilattice verdict engine (vendored, standalone).
- `examples/` sample targets.

## License

Unlicense. See `LICENSE`.
