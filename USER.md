# V⊙x user guide

V⊙x answers one question about a program: does its control flow close? This guide
covers how to run it, how to read the verdict, and what a finding means.

## Install

Nothing to install. Python 3.10 or newer, standard library only. The verdict
engine is vendored in `imasm16_3_core.py`.

```bash
git clone <this repo> vox && cd vox
python3 vox.py --selftest
```

## Running

### A Python file

```bash
python3 vox.py path/to/module.py
```

V⊙x imports the module, disassembles every top-level function, lifts each one, and
prints a line per function:

```
FUNCTION                B4  WORD
guarded_use             T   VINIT IFIX FSPLIT IFIX FFUSE TANCH
linear                  N   VINIT TANCH
reentrant               B   VINIT FSPLIT IFIX TANCH TANCH   <-- FINDING
```

Importing the module runs its top-level code. Point V⊙x at code you trust to
import, or at a copy with side effects removed.

### EVM bytecode

```bash
python3 vox.py --evm 600160075755005b00
```

Pass the runtime bytecode as a hex string, with or without a `0x` prefix. V⊙x
parses the opcode stream, skips `PUSH` operands, and resolves each jump target
from the `PUSH` immediately before it, which is the form every compiler emits.

### WASM bytecode

```bash
python3 vox.py --wasm 20000440410141003602000f0b0b
```

Pass a single function body as a hex string: the raw instruction bytes, not a
whole module. WASM control is structured, so `if` opens a fork and its `end`
closes it unless a `return` or `br` escaped the branch first.

### Self-test

```bash
python3 vox.py --selftest
```

Runs a vulnerable and a safe example on both EVM and WASM and asserts the
verdicts. Use it to confirm the engine is wired correctly.

## Reading the verdict

The verdict is one of four Belnap values, not a boolean. A boolean would collapse
a four-valued distinction into two.

| Verdict | Meaning | Finding? |
|---------|---------|----------|
| **T** | The flow forks, does work, and rejoins. It closes. | no |
| **B** | A fork is held open across a commit or a return. | yes, triage it |
| **N** | The routine never forked. Linear, nothing to weigh. | no |
| **F** | The flow is a refutation on its own terms. | inspect |

A finding is **B**: the fork opened alternatives and then a state write (`IFIX`)
or a return (`TANCH`) escaped before the alternatives rejoined (`FFUSE`). That is
the structural signature of reentrancy, an unhandled path, and resource leaks.

**B is not a verdict of guilt.** It is dialetheic: both closed and open are live.
It marks the spot to look, and you resolve it to a definite T or F by reading the
call at that fork. V⊙x tells you where; it does not tell you it is exploitable.

## Reading the lifted word

The `WORD` column is the control flow in the twelve opcodes:

| Opcode | Glyph | Bytecode role |
|--------|-------|---------------|
| VINIT | ⊢ | function entry |
| FSPLIT | ∈ | a branch opens a fork |
| FFUSE | ∋ | a true merge, the paths rejoin |
| AFWD | > | work, an external call |
| IFIX | ◻ | a state write, irreversible |
| TANCH | ⊣ | a return |

Read the word left to right. A closed word forks (`FSPLIT`), does work, and fuses
(`FFUSE`) before it commits or returns. A finding forks and then commits or
returns with no `FFUSE` between: the fork dangles.

Worked example, the reentrant case in all three languages:

```
VINIT FSPLIT IFIX TANCH   →   B
```

Enter, fork, commit state, return. No fuse. The commit happened while the fork was
still open, so a re-entrant call slips into that window. The Solidity, Python, and
WASM versions of this bug all produce this same word.

## The closure law

μ∘δ = id. A fork (δ, `FSPLIT`) that does work on its arms and fuses back (μ,
`FFUSE`) is the identity: the decision was made and resolved. A fork that never
fuses before a commit is open, and open is where bugs live. V⊙x computes this
over the SIXTEEN_3 trilattice, so the verdict is a property of the structure, not
of any language.

## What it does not do

V⊙x reports structure, not exploitability. A `B` marks a fork that holds open
across a commit; whether that window is reachable and profitable is triage you do
by hand. It reads real bytecode, so a merge present in source but compiled away
will not appear. It does not model data flow, only control flow.

## Extending it

The lift is a small table per ISA in `vox.py`. To add a front end, parse the
target's instruction stream and map its branch, merge, state-write, call, and
return opcodes to `FSPLIT`, `FFUSE`, `IFIX`, `AFWD`, `TANCH`. The verdict engine
does the rest and needs no changes.
