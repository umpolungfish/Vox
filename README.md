# V⊙x

The Pancosmic Disassembling Re-Compiling Organism. Every word of that is meant
literally, and each one names something you can run.

**Pancosmic.** One lift, every substrate. Native x86 at both widths, out of ELF,
PE or Mach-O; EVM bytecode; a WASM function body; CPython, straight out of a
`.pyc` with no interpreter running; and a coding sequence, where the twelve
promoted amino acids biject the twelve axes so a gene is already a word. A merge
is a merge whether it is a `JUMPDEST`, an `end`, a jump target with two
predecessors, or a codon.

**Disassembling.** Its own container loader, its own instruction decoder, its own
machine. No capstone, no pefile, no runtime underneath. It reads the machine
field and refuses an architecture it cannot decode rather than misreading one as
another.

**Re-Compiling.** The lift is not a description of the program, it is the
program. V⊙x recompiles native code into an executable IMASM module and runs the
glyphs in a machine that never once looks at the original bytes, returning every
answer native code returns — Ackermann recursion, SSE, a jump table, a call
through a function-pointer array, at five optimisation levels and both widths.

**Organism.** It closes on itself. `vox self` points the lifter at its own image
and reads the lifter: every function, in phase, F zero.

```bash
cargo install --path .
vox run gcd --args 1071,462 corpus_O0.so
vox self
vox pyc module.pyc
```

## What the verdict means, and what B costs

T closes, B holds a fork open across the cycle, N never forked, F is ill-typed.
Pairing is cyclic, because a word is a loop and ROTAT is the cyclic shift, and T
additionally requires work inside the paired region: a split and fuse with
nothing between them is μ∘δ=id and verifies nothing.

Reading its own image taught it what B costs. An early return IS a fork that
leaves and never rejoins, so the surplus of ∈ over ∋ rises with the number of
exits — a mean of 0.28 at no exit, 5.26 at one, 12.91 at two, 15.00 at four or
more, and above two exits essentially every function carries surplus. Ranking
candidates by raw surplus therefore ranks by how many ways a function can return.
`vox self` reports the residual, which is the surplus left once the exits are
paid for, and that is the part worth reading.

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
glyph reads. A ∈ splits, a ∋ fuses, a ⊞ engages, a ⊡ commits, a ⊙ transfers
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
vox verdict ⊢∈⊡⊣                 # verdict one glyph word
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
| > call | < transfer | ⊙ indirect | ⊡ commit |
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
⊢∈⊡⊣  →  B
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

## Baked numerical membranes

### Resident circuits

`qft_circuit.sh <levels>` builds a QFT circuit with `2^levels` ports, lifts its
complete executable into IMASM, and prepares it once inside Vox. Depth is a
build parameter. The direct-transform checking runner accepts depths 1 through
12; the circuit's connection layout is generated from the selected depth.

```text
input gates -> stored permutation -> level 1 -> ... -> level d -> output
     ^                                                           |
     +-------------- sampled feedback gate <----------------------+
```

Connections are compile-time data. Preparation initializes the shared phase
table and signal buffers. Activation changes a gate pattern and propagates
through those existing connections, using fixed guest storage. Feedback selects
the preceding activation's normalized output as the source. Closing all input
gates clears the signals. A 64-bit gate pattern repeats across the ports when
the circuit has more than 64 ports.

```sh
./qft_circuit.sh 3
./qft_circuit.sh 8
./target/release/vox circuit membranes/qft_circuit/256/payload.elf.imasm --stdin
```

With `--stdin`, preparation finishes before gate changes are read. Enter one
hexadecimal mask per line, optionally followed by `:1` to select feedback, then
EOF to finish. For example, `1`, `0123456789abcdef`, `ffffffffffffffff:1`, and
`0` activate, change the pattern, feed back, and clear. The same VM remains
resident throughout. Each output is checked against a direct transform after
the activation timer stops. Logs separate loading, preparation and activation;
the default run also compares five cold and resident runs with identical gates.

### Complete process membranes

`membrane_one.sh` builds static musl executables from the copied
`G-mOMonadOS` numerical modules. Parameters become IMASM numeral words before
compilation. Vox lifts the complete executable, including its data, into a saved
`.imasm` module and runs that module with no input arguments. The launcher
requires successful VM exit and exact stdout/stderr agreement with the native
control. Modules, executables, output comparisons and elapsed Vox timings live
under `membranes/<kind>_one/<parameters>/`. The installed Rust musl target is
required. `membrane_speed_chart.md` distinguishes native kernel benchmarks from
complete interpreted executions.

```sh
./membrane_one.sh abc 1 10 9 32 1000

./membrane_one.sh divisor 1125899906842624 360 97

./membrane_one.sh shor 2 21 12

./membrane_one.sh schutte 23 2 3

./membrane_one.sh landau 10 15 45

./membrane_one.sh tripsum 24
./membrane_one.sh factor 8051
```

ABC takes an epsilon numerator and nonzero denominator, followed by cutoffs.
Its outer window membrane holds one radical table, logarithm table, and scan of
cumulative maxima. Each cutoff reads that scan, preserving request order and
duplicate cutoffs. Cutoffs through 1,000,000 are accepted by the hosted entry.

The divisor membrane takes unsigned 64-bit integers. One factorization generates
the divisors in exponent order. The numerical sort, ring bonds, and spectrum
consume that prepared state. Inputs zero and one return a trivial result.

The Shor membrane takes a base, modulus, and index-qubit count (1 through 14).
The modular orbit advances by multiplication. A shared roots-of-unity table and
bit-reversal permutation feed the nested Fourier stages, reducing the transform
from quadratic work in register size M to O(M log M), with O(M) storage.
Continued fractions still consume the computed probability peaks.

The Schütte membrane takes a vertex count (1 through 63), followed by subset
sizes. It prepares the quadratic-residue graph once. Each subset intersects
precomputed dominator bitsets; only masks of the requested cardinality are
visited, in increasing order. The result carries the number examined and the
first failing subset, when present. The kernel is extracted from the Schütte
section of `erdos_walks.rs`.

The modules are compiled by their hosted binary targets; the `vox` library
retains its standalone `no_std` interface and has no added dependencies.
Source provenance is recorded in `membrane_sources.json`.

Controls compare ABC maxima and attaining triples with separate original window
enumerations, and divisor membership and ring order with trial division and
exponent sorting. Run them with:

```sh
cargo test --release --bin abc_one --bin divisor_one membrane -- --nocapture
cargo test --release --bin shor_one --bin schutte_one -- --nocapture
```

The recorded controls in `membrane_checks.log` measured ABC cutoffs
200/400/600/800/1000 at 138.32 ms for separate windows and 13.05 ms for one
membrane. For `2^50`, trial divisor enumeration took 210.60 ms and the prepared
full ring analysis took 21.17 microseconds. These are execution timings for
those inputs. The wrapper's build time is separate.

`membrane_wave2_checks.log` records a dense 1,024-amplitude QFT control at
25.35 ms for the direct transform and 38.77 microseconds for the nested transform,
including phase-table setup. Complex amplitudes agree within the test tolerance;
norm preservation and extracted periods have separate controls. The Schütte
control at 23 vertices and subset size 2 takes 11.24 ms in the original search
and 4.38 microseconds with the prepared graph, including setup. All 253 pairs
are checked. These measurements cover the named kernels and inputs.

The Landau membrane prepares the largest partition LCM for every capacity through
the greatest requested N. Each prime encloses its alternative powers, and every
choice reads the preceding prime stage. Results use checked `u128` arithmetic.
Multiple requested N values consume the same table in request order.

The distinct-triple-sum membrane retains the sums belonging to the active search
branch. A candidate adds only triples containing that candidate. Its trail is
rolled back on leaving the branch, preserving the enclosing sums and the original
search order. It emits the largest set whose three-element subsets have distinct
sums, including the same first witness as the source search.

Both kernels are extracted from `erdos_walks.rs`. Controls in
`membrane_wave3_checks.log` compare Landau values through 32 and triple-sum
witnesses through 18 against the original exhaustive code. At N=45, partition
descent takes 4.25 ms and the complete Landau table 5.85 microseconds. At limit
24, the original triple-sum search takes 8.51 ms and the nested search 1.04 ms;
both emit `[1, 2, 3, 11, 17, 20, 23]`. Reproduce the controls with:

```sh
cargo test --release --bin landau_one --bin tripsum_one -- --nocapture
```

## Source layout

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
