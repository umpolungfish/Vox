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
guarded_use             T   ⊢◻∈◻∋⊣
linear                  N   ⊢⊣
reentrant               B   ⊢∈◻⊣⊣   <-- FINDING
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

### A native binary

```bash
python3 vox.py some_program.exe
```

V⊙x auto-detects a PE binary by its `MZ` magic, disassembles the executable
sections (needs `capstone` and `pefile`), splits the code into functions at the
entry point and at call targets, and lifts each function. It prints the verdict
distribution and lists the functions that hold a fork open, with their addresses:

```
file 41,475,671 B  |  code 26,624 B (read)  |  overlay 41,406,551 B (not code)  |  NSIS installer
  note: this file is mostly an appended payload, not program. V⊙x read the stub;
  extract it (e.g. 7z x) to scan the real code inside.
native PE: 94 functions   verdicts {'B': 38, 'T': 25, 'N': 31}
38 B-finding(s): fork(s) holding open across a commit/return.
  0x40128c     ⊢∈>∈⊣◻◻∋◻◻∈∋◻⊣
  ...
```

The header is the first thing to read. A 40 MB installer is almost all appended
payload: the real program is a small stub (here 26 KB), and the rest is the
compressed application, which is data, not code, so V⊙x leaves it alone. To audit
the app itself, extract the installer and point V⊙x at the unpacked binaries.

Native code forks and returns constantly, so B is common and mostly benign here.
It is a map of where control does not cleanly rejoin, ranked by the machine, for
you to triage. This is a linear sweep with a call-target split, not recursive
descent, so a region with few internal calls can be lumped into one long word.
Every word is printed in full, in the alphabet, and every finding is listed:
nothing is truncated, because a word cut short is a different word. A packed
binary hides its real code until runtime; V⊙x reads what is on disk.

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

## Recompiling

`--imasm OUT` rewrites a native binary — PE or ELF, auto-detected — as an IMASM module: a labelled word per
function, in address order.

```
; ⊙ program.exe
; 408 words   66585 glyphs
0x140001000
⊢<⊣⊣⊣⊣◻◻⋈⊣⊣⋈⋈⊤∈⋈<⋈⊣⊤∈⋈⊤∈⋈⊞⊤∈∋⋈⊤∈⊤∈∋⊞⊞⊤∈∋⊞⊣⊤∈◻∋⋈⊣
```

This lift is total — every decoded instruction gets a glyph:

| Glyph | Instruction |
|-------|-------------|
| ⊢ | function entry |
| ⊣ | a terminal: `ret`, `int3`, `ud2`, `hlt` |
| ∈ | a conditional branch |
| ∋ | a merge, two paths rejoining |
| > | a direct call |
| < | an unconditional transfer: `jmp`, a tail call |
| ⊙ | an **indirect** call or jump — the target is data, the structure taking itself as its own object, and exactly where a linear disassembler goes blind |
| ◻ | a write to memory, irreversible |
| ⋈ | data movement between named slots: `mov`, `lea`, `movzx`, `push`, `pop`, `xchg` |
| ⊤ | a truth produced: `cmp`, `test` |
| ⊥ | a truth consumed: `setcc`, `cmovcc` |
| ⊞ | engagement: everything that computes on values |

`--word` emits exactly this and nothing else, and it does not execute — it is
the structure, which is what MEASUREMENTS.md is taken over.

## Executing

`--imasm` emits the module that runs: the same glyph per instruction, each
carrying the payload its axis needs, plus the initialised data sections the code
reads. `imasm_vm.Machine` executes it, dispatching on the glyph alone.

```bash
python3 vox.py --run gcd --args 1071,462 lib.so
gcd(1071, 462) = 21   [26 steps in the twelve]
```

Payload by glyph — the glyph decides how its fields are read:

| Glyph | Payload |
|-------|---------|
| ∈ | a condition and a target |
| > / < / ⊙ | a target; ⊙ carries whether it is a call or a jump, because a call must still leave a return address |
| ⋈ | the two slots, and the move's kind |
| ◻ | the memory reference, the source, the width |
| ⊤ | the two things compared, and whether by difference or by conjunction |
| ⊥ | the condition, and the slot it lands in |
| ⊞ | the operation and its operands, integer or vector |

Operands are normalised at emit time so the machine never parses assembly:
`r:rax` a register, `i:0x10` an immediate, `m:base:index:scale:disp:size` a
memory reference. `rip` is a slot like any other, because a rip-relative
reference reads the address of the next instruction.

### Verification

`verify.py LIB.so ...` runs every function twice — natively through ctypes and
as IMASM in the machine — over the same inputs, and prints any disagreement with
the arguments that caused it.

```
lib0.so ... lib3.so, libs.so        1245 agreements, 0 mismatches
hard0.so ... hard3.so, hards.so     1125 agreements, 0 mismatches
```

Two corpora at five optimisation levels each: integer arithmetic, division and
modulo, loops, vectorised code (`-O3` emits SSE), deep recursion, cross-function
calls, stack arrays, switch jump tables, and calls through a function-pointer
table. The last is what ⊙ is for.

## Auditing

The `WORD` column is the control flow in the twelve opcodes:

| Glyph | Bytecode role |
|-------|---------------|
| ⊢ | function entry |
| ∈ | a branch opens a fork |
| ∋ | a true merge, the paths rejoin |
| > | work, an external call |
| ◻ | a state write, irreversible |
| ⊣ | a return |

The word is written in the alphabet and nothing else. Read it left to right. A
closed word forks (∈), does work, and fuses (∋) before it commits or returns. A
finding forks and then commits or returns with no ∋ between: the fork dangles.

Worked example, the reentrant case in all three languages:

```
⊢∈◻⊣   →   B
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
