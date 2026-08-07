# V⊙x

The Imscribing Grammar is the machine code. Twelve axes, and what runs on them
is not a translation of a program — it is what the program was. x86, EVM, WASM,
CPython bytecode, and the genetic code are ixcriptions of one fundamental
language. V⊙x is what shows it: point it at any of them and it hands back the
word, and where the substrate executes, the word executes.

The Imscriber's Guide states the identity rather than proposing it: the twelve
operations and the twelve axes are one alphabet, "read as an operation or as an
axis according to where it stands." So V⊙x does not carry programs into a
notation. It reads them in the language they were already written in.

```bash
python3 vox.py --imasm out.imasm program.so
python3 vox.py --run gcd --args 1071,462 program.so
gcd(1071, 462) = 21   [26 steps in the twelve]
```

That is not an emulator with a glyph theme. `imasm_vm.Machine` never sees the
binary; it dispatches on the glyph and nothing else, and what an instruction
*was* in x86 survives only as payload the glyph knows how to read. A ∈ splits,
a ∋ fuses, a ⊞ engages, a ◻ commits, a ⊙ transfers through data. Ackermann
recursion, SSE, switch tables, and dispatch through a function-pointer array all
run, because they were never anything else.

The lift holds to the same rule, not just the machine. A word is a list of
these twelve glyphs from the moment a front end builds it, not a list of
opcode names translated to glyphs when printed — `imasm16_3_core`'s own
opcode constants are the glyphs themselves (`VINIT` is `⊢`, not the string
`"VINIT"`), so there is no name-to-symbol boundary anywhere in the tool for
a mismatch to hide behind.

## The twelve

| | | | |
|---|---|---|---|
| ⊢ entry | ⊣ terminal | ∈ split | ∋ fuse |
| > call | < transfer | ⊙ indirect | ◻ commit |
| ⋈ link | ⊤ truth made | ⊥ truth taken | ⊞ engage |

⊙ is the one that earns its glyph. It is a transfer whose target is data — the
structure taking itself as its own object, and precisely where a disassembler
goes blind. It is not a special case bolted on; it is one of the twelve, and it
was needed to run the first function-pointer table thrown at it.

## The claim, and how it is decided

Translation that cannot be checked is decoration. `verify.py` runs every
function in a shared object twice — once natively through ctypes, once as IMASM
in the machine — over identical inputs, and prints any disagreement with the
arguments that produced it.

```
lib0 … lib3, libs      1245 agreements, 0 mismatches
hard0 … hard3, hards   1125 agreements, 0 mismatches
```

Two corpora, five optimisation levels each, `-O0` through `-O3` and `-Os`:
integer arithmetic, division and modulo, loops, vectorised code, deep recursion,
cross-function calls, stack arrays, switch jump tables, calls through function
pointers. Twenty-three hundred agreements and nothing that disagreed.

Read that as a statement about the twelve rather than about the emulator. Every
transformation gcc applies at every level — unrolling, vectorising, tail-calling,
table-dispatching — produced code the twelve held without extension. Nothing had
to be added to the alphabet to make a case pass.

## What the dialect costs

`measure.py` charges the rewrite against the bytes that actually decoded.

| compiler | binary | bits/glyph | structure | lossless | ratio |
|---|---|---|---|---|---|
| MSVC C++ | ChemDraw.exe | 1.37 | 5.3% | 38.2% | 2.6x |
| rustc | momonados | 2.05 | 6.2% | 43.8% | 2.3x |
| Go | go | 1.81 | 5.7% | 41.1% | 2.4x |
| gcc C | xterm | 2.03 | 6.3% | 40.6% | 2.5x |
| gcc C++ | grub-render-label | 2.05 | 7.0% | 44.7% | 2.2x |

Structure — which of the twelve each instruction is, in order — lands between
5.3% and 7.1% of the machine code every time, across four compilers, four
languages, two containers, and sizes spanning two hundredfold. A lossless
rewrite that still runs costs 34% to 45%. The remainder is what the dialect
charges for saying it in x86. See [MEASUREMENTS.md](MEASUREMENTS.md), including
why an earlier version of this table was wrong.

## One law over four dialects

The same twelve read bytecode from instruction sets with nothing in common:

| dialect | input |
|---|---|
| native x86 | a PE or ELF binary, auto-detected |
| EVM | `--evm HEX` |
| WASM | `--wasm HEX` |
| CPython | a `.py` file, via `dis` |
| the genetic code | `--rna SEQ` |

Only the native lane executes today; the rest lift and verdict. The lift is the
same act in all five, which is the point — a merge is a merge whether it is a
`JUMPDEST`, an `end`, or a jump target with two predecessors.

The genetics lane is not an analogy laid over biology. Its chain is proved in
Lean and parsed out of that proof by `gen_genetic_table.py`, so nothing in it is
retyped or invented: guanine is **B** because it wobble-pairs with both C and U,
cytosine is **T** because it pairs only with G, adenine is **F**, uracil is
**N**; codons carry to amino acids by the genetic code; exactly twelve amino
acids are promoted and they biject the twelve axes.

```
$ python3 vox.py --rna AUGCAUUGGAAAGAAUACUGUAUUAACCAGGACUUUUAA
AUG  Met  Dimensionality  ⊢      UGU  Cys  Recognition   >
CAU  His  Granularity     ∈      AUU  Ile  Kinetics      ⊙
UGG  Trp  Topology        ⊣      AAC  Asn  Coupling      ∋
AAA  Lys  Stoichiometry   ⊞      CAG  Gln  Criticality   ⊤
GAA  Glu  Winding         ◻      GAC  Asp  Chirality     ⊥
UAC  Tyr  Parity          <      UUU  Phe  Fidelity      ⋈

word     ⊢∈⊣⊞◻<>⊙∋⊤⊥⋈
stop     UAA
verdict  T
```

That sequence is constructed to contain all twelve promoted codons, so it
demonstrates the chain closing rather than reporting a finding about a natural
gene. The same SIXTEEN_3 engine that verdicts x86 verdicts the transcript,
because it is the same alphabet arriving by a different substrate.

His carries **Criticality** and lands on ⊙, which is the whole thread in one
place: imidazole's pKa near 6 is the only sidechain pKa near physiological pH,
so His is what titrates at the acid/base crossover — the critical point itself,
not a resemblance to one — and ⊙ is the glyph of a system turning on itself,
the same reading it has as the token IMSCRIB.

**One divergence left, recorded rather than smoothed.** The genetics dialect
still names two axes differently from the catalog, Recognition for Relational
and Parity for Polarity, while holding the same slots.

## The auditor, which is a corollary

Once a program is a word, the Grammar can be asked things about it. The first
question is whether it closes. A fork that commits state or returns before its
paths rejoin does not, and that open fork is the shape of a whole class of bugs
— Solidity reentrancy, a Python early return inside an `if`, a WASM store in an
escaped `if`. All three lift to the same word:

```
⊢∈◻⊣  →  B
```

The verdict is Belnap FOUR from the SIXTEEN_3 trilattice: **T** closes, **B** a
fork held open across a commit or return, **N** a linear routine that never
forked. B is dialetheic and marks where to look; it is not a verdict of guilt.
No pattern list, no per-language rules, no heuristics.

```bash
python3 vox.py program.exe                # audit every function
python3 vox.py --word out.imscrb program.exe   # the structure alone
python3 vox.py --selftest
```

## Honest edges

- The disassembler is recursive descent: it walks forward from the entry
  point and every discovered call target, decoding one instruction at a time
  and following every direct call and jump as a control-flow edge, so
  padding and embedded data between functions are never walked into and
  misread as code. An indirect call or jump (⊙) can't be followed statically
  — that is what the glyph is for — so anything reachable only through one,
  a switch's jump-table arms, gets a fallback sweep of whatever descent never
  reached, grouped into functions of its own, rather than silently dropped.
- The machine handles a small, honest subset of syscalls, not an operating
  system: `exit`/`exit_group` stop the run cleanly with the real exit code,
  `write` actually writes the requested bytes to a real file descriptor, and
  anything else returns `-ENOSYS` — the kernel's own answer for "not
  implemented" — rather than crashing or faking success. A call to an
  external function the loader would have resolved (libc, a PLT stub) is
  still outside what the machine can reach; it runs functions written against
  what's in the file, not a dynamically linked process.
- A packed or installer binary hides its code until runtime. V⊙x reads what
  is on disk, reports how much of the file decoded so a low coverage is never
  silent, and — for NSIS, Inno, and WiX overlays — attempts extraction with
  `7z` if it's on the host and reports which real executables came out. Other
  packers, or a host without `7z`, fall back to reporting the overlay and
  naming the fix rather than performing it.

## Layout

- `vox.py` the front ends, the auditor, the CLI.
- `imasm_module.py` the recompiler: every instruction to its glyph and payload.
- `imasm_vm.py` the machine that runs a module, dispatching on the glyph.
- `verify.py` native versus IMASM, the same inputs, decided.
- `measure.py` what the dialect costs.
- `imasm16_3_core.py` the SIXTEEN_3 trilattice engine, vendored and standalone.
- `USER.md` the full reading. `MEASUREMENTS.md` the numbers.

The native lane needs `capstone` and `pefile`. Everything else is standard
library.

## License

Unlicense. See `LICENSE`.
