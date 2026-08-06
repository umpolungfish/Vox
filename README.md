# V⊙x

A control-flow closure auditor. V⊙x reads a program's bytecode, lifts its
control flow to a word in the twelve-opcode Imscribing Grammar, and asks one
question the same way across every language: **does the flow close?**

A fork that commits state or returns before its paths rejoin does not close. That
open fork is the shape of a whole class of bugs, and V⊙x names it in a grammar
with no allegiance to any instruction set, so the one law catches it everywhere.

## What it is

V⊙x lifts the control-flow graph of real bytecode to a twelve-opcode word and
runs a paraconsistent verdict engine (the SIXTEEN_3 trilattice) over it. No
pattern list, no language rules, no heuristics tuned per target. The verdict is
Belnap FOUR:

- **T** the control flow closes.
- **B** a fork is held open across a commit or a return. This is the finding.
- **N** a linear routine that never forked. Clean, nothing to weigh.

Three front ends today, one law behind them:

| ISA | input | fork | merge |
|-----|-------|------|-------|
| CPython | a `.py` file (via `dis`) | conditional jump | a target with two or more predecessors |
| EVM | `--evm HEX` | `JUMPI` | `JUMPDEST` reached from two paths |
| WASM | `--wasm HEX` | `if` | its `end`, unless a `return`/`br` escaped first |

## Why it exists

The famous bugs are the same bug in different clothes. A Solidity reentrancy that
calls out and writes state before the branch rejoins, a Python function that
commits and returns early inside an `if`, a WASM store in an escaped `if`: all of
them commit before control comes back together. Lift each to the grammar and they
land on the identical word:

```
VINIT FSPLIT IFIX TANCH  →  B
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
```

`B` in the output is a finding: a fork held open across a commit or return.
See `USER.md` for the full reading, the verdict values, and how to interpret the
lifted word.

## How it works

Each front end walks the real bytecode (not the source) and emits the load-bearing
structure as opcodes: `VINIT` at entry, `FSPLIT` at a branch, `FFUSE` at a true
merge, `IFIX` at a state write, `AFWD` at an external call, `TANCH` at a return.
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
