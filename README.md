# V⊙x

V⊙x lifts a machine program to twelve glyphs, runs the glyphs in a machine that
never once looks at the original bytes, and gets every answer native code gets.
Ackermann recursion, SSE, a switch's jump table, a call through a
function-pointer array, at five optimisation levels from thirteen functions gcc
was free to transform however it wanted, every function and every input agreeing
with native and nothing added to the twelve to make any of it pass.

It is a single Rust crate with no external crates. The container loader is its
own, so it reads ELF, PE and Mach-O without a library. The instruction decoder
is its own, so there is no capstone. The machine is its own, so there is no
runtime under it. `cargo build` and it stands on its own.

```bash
cargo install --path .
vox run gcd --args 1071,462 corpus_O0.so
gcd(1071, 462) = 21   [41 steps in the twelve]
```

That result is the evidence for the larger claim. The Imscriber's Guide states
it rather than proposing it: the twelve operations and the twelve axes are one
alphabet, "read as an operation or as an axis according to where it stands."
x86, EVM, WASM, CPython bytecode, and the genetic code are ixcriptions of that
one language. V⊙x is what shows it: point it at any of them and it hands back
the word, and where the substrate executes, the word executes.

`imasm_vm::Machine` never sees the binary. It dispatches on the glyph and
nothing else, and what an instruction *was* in x86 survives only as payload the
glyph reads. A ∈ splits, a ∋ fuses, a ⊞ engages, a ◻ commits, a ⊙ transfers
through data.

## Build and run

Nothing to compile but the crate. Point a verb at any executable and it does one
thing to it.

```bash
cargo build --release            # zero external crates
cargo install --path .           # puts `vox` on PATH

vox run <sym> --args a,b <file>  # recompile a function and RUN it
vox imasm <file>                 # emit the executable IMASM module
vox word <file>                  # the structure word per function
vox disasm <file>                # the decoded instruction stream
vox <file>                       # audit every function, tally verdicts
vox verdict ⊢∈◻⊣                 # verdict one glyph word
vox evm <hex>                    # lift EVM bytecode, verdict its closure
vox wasm <hex>                   # lift a WASM function body, verdict it
vox rna <seq>                    # lift a coding sequence, verdict the transcript
vox --selftest                   # planted forks, x86 / EVM / WASM
```

The file can be an ELF, a PE, or a Mach-O, from Linux, Windows or macOS, and a
fat Mach-O picks its x86-64 slice. Anything else is read as a raw flat image.
The container is universal; the instruction set is x86-64, so a segment that is
not x86 fails to decode one instruction at a time and the walk reports how far
it got, rather than being refused at the door.

## The twelve

| | | | |
|---|---|---|---|
| ⊢ entry | ⊣ terminal | ∈ split | ∋ fuse |
| > call | < transfer | ⊙ indirect | ◻ commit |
| ⋈ link | ⊤ truth made | ⊥ truth taken | ⊞ engage |

⊙ is the one that earns its glyph: a transfer whose target is data, the
structure taking itself as its own object, precisely where a disassembler goes
blind. It is one of the twelve on the same terms as the other eleven, earned the
day the first function-pointer table was thrown at it and nothing else in the
alphabet could read the jump.

## The claim, and how it is decided

Translation that cannot be checked is decoration. Every function in a shared
object runs twice, once natively through ctypes and once as IMASM in the
machine, over identical inputs, and any disagreement prints with the arguments
that produced it.

```
corpus_O0 … corpus_Os (5 optimisation levels)     every function, 0 mismatches
```

Thirteen functions (integer arithmetic, division and modulo, loops, vectorised
code, deep recursion, cross-function calls, stack arrays, a switch jump table,
calls through a function-pointer array) at `-O0` through `-O3` and `-Os`, built
by `build_corpus.sh` from `corpus.c`. The switch table and the function-pointer
dispatch, the two places a disassembler is most likely to lose the thread, agree
exactly at every optimisation level, as does Fibonacci to depth twenty-nine,
which costs the machine twenty-eight million steps to answer.

Read that as a statement about the twelve rather than about the emulator. Every
transformation gcc applies at every level (unrolling, vectorising, tail-calling,
table-dispatching) produced code the twelve held without extension.

## Four parts, one alphabet

```
file ──▶ loader::load ──▶ x86::decode ──▶ imasm_module::emit ──▶ imasm_vm::Machine ──▶ answer
         any container     operands         glyph + payload        dispatch on glyph
```

`loader::load` reads ELF, PE, Mach-O and fat Mach-O, and falls back to a raw
image, handing back the same thing from every source: the entry point, the
executable segments, the data segments the code loads, and the symbols the file
carries. `x86::decode` is a hand-written x86-64 operand decoder: ModRM, SIB, REX
and the prefixes into structured operands (`r:reg`, `i:imm`,
`m:base:index:scale:disp:size`), integer and SSE, checked instruction for
instruction against `objdump`. `imasm_module::emit` recompiles each instruction
to its glyph plus the payload that glyph reads, carries the data sections, and
decodes each executable segment linearly so an address reached only through an
indirect jump (a switch arm, a function pointer) is in the module too.
`imasm_vm::Machine` is the register file with its sub-register aliasing, the byte
memory, the lazy `cmp`/`test` flags, the ALU, the SSE vector unit, and a small
syscall subset, dispatching on the glyph.

The lift holds to the same rule as the machine. A word is a list of these twelve
glyphs from the moment the front end builds it, not a list of opcode names
translated to glyphs when printed, so there is no name-to-symbol boundary
anywhere in the tool for a mismatch to hide behind.

## The lanes

The same twelve read instruction sets with nothing in common.

| lane | input | state |
|---|---|---|
| native x86 | an ELF, PE, or Mach-O binary | lifts, verdicts, and **runs** |
| EVM | `vox evm HEX` | lifts and verdicts |
| WASM | `vox wasm HEX` | lifts and verdicts |
| genetic code | `vox rna SEQ` | lifts and verdicts |
| CPython | `vox.py` | lifts and verdicts |

The x86 lane executes. EVM and WASM lift bytecode to the same word and verdict
its closure. The lift is the same act in each, which is the point: a merge is a
merge whether it is a `JUMPDEST`, an `end`, or a jump target with two
predecessors.

The genetic lane reads a gene as what it already is, a word. The table is not
retyped here: `gen_genetic_table.py` parses it out of the Lean that proves it
and writes both `genetic_table.py` and `src/genetic_table.rs`, so neither
language can drift from the source. Guanine is **B**
because it wobble-pairs with both C and U, cytosine is **T** because it pairs
only with G, adenine is **F**, uracil is **N**; codons carry to amino acids by
the genetic code; exactly twelve amino acids are promoted and they biject the
twelve axes. The full `RNA → IMASM → x86 → IMASM → wasm → IMASM → AA` round
trip still runs in the mOMonadOS `circuit` verb. The CPython lane needs a running interpreter's disassembler, so it
stays in `vox.py`.

## The auditor, which is a corollary

Once a program is a word, the Grammar can be asked things about it. The first
question is whether it closes. A fork that commits state or returns before its
paths rejoin does not, and that open fork is the shape of a whole class of bugs:
Solidity reentrancy, a Python early return inside an `if`, a WASM store in an
escaped `if`. All three lift to the same word:

```
⊢∈◻⊣  →  B
```

The verdict is Belnap FOUR from the SIXTEEN_3 trilattice: **T** closes, **B** a
fork held open across a commit or return, **N** a linear routine that never
forked, **F** an ill-typed word (a ∋ with no ∈ to pair). B is dialetheic: a fork
worth looking at, decided by whether the paths rejoin, not by any judgment about
what the code is for. No pattern list, no per-language rules, no heuristics.

## The edges

- The decoder returns `None` on an opcode it does not know rather than a guess,
  so the walk stops there and reports how far it got. A partial lift is always
  visible as partial, and a low coverage is never silent. For execution the
  whole executable segment is decoded linearly, so a jump-table arm or a
  function-pointer target, reachable only through a ⊙, is in the module rather
  than dropped.
- The machine handles a small subset of syscalls, not an operating system.
  `exit`/`exit_group` stop the run cleanly with the real exit code, and anything
  else returns `-ENOSYS`, the kernel's own answer for "not implemented," rather
  than crashing or faking success. A run is capped at fifty million steps;
  recursion deeper than that halts with the step count in the error rather than
  running forever or returning a wrong answer.
- The auditor walks recursive descent, so it reports honest coverage on a large
  stripped binary rather than misreading padding and data as code. The CPython
  lane and the cost measurement are in `vox.py`, not the crate.

## Layout

- `src/loader.rs` the universal loader, any container to code + data + entry.
- `src/x86.rs` the operand decoder, bytes to structured operands.
- `src/imasm_module.rs` the recompiler, every instruction to its glyph and payload.
- `src/vox.rs` the classifier and the closure verdict.
- `src/vox_decode.rs` the length decoder the auditor walks.
- `src/imasm_vm.rs` the machine that runs a module, dispatching on the glyph.
- `src/lanes.rs` the EVM and WASM front ends.
- `src/main.rs` the CLI.
- `corpus.c`, `build_corpus.sh` the thirteen functions and the five builds.
- `vox.py` and companions, the CPython lane and the cost measurement.

The crate is the foundation for anything that wants the auditor as a library.
`vox.rs`, `vox_decode.rs` and `lanes.rs` live here as their one home, and
mOMonadOS links this crate rather than carrying a copy, so a fix propagates by
recompile.

## License

Unlicense. See `UNLICENSE`.
