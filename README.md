# V⊙x

The Pancosmic Disassembling Re-Compiling Organism. Every word names something you can run. Single Rust crate, zero external crates.

**Pancosmic.** One lift, every substrate: native x86-64 (ELF/PE/Mach-O, both widths), EVM bytecode, WASM function bodies, CPython `.pyc`, and coding sequences (12 promoted amino acids biject the 12 axes - a gene is already a word). A merge is a merge whether `JUMPDEST`, `end`, or two-predecessor jump target.

**Disassembling.** Own loader, decoder, machine - no capstone/pefile/runtime. Refuses unknown architectures rather than misreading them.

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

### Membrane numeral factor verification

`factor-membrane` accepts a decimal or `0x` integer, the canonical
g-mOMonadOS hex-digit word, or its native binary word. It emits both canonical
encodings, factors the decoded value with Vox, and checks the result. Supply a
candidate pair with `--factors P Q` to verify its product. Both CLI entrypoints
also invoke `g-momonados trilattice_factor read` on N, p, and q so period, cuts, dialect
register, and type hash are measured for the current input. Set
`VOX_TRILATTICE_FACTOR` if that executable is not on `PATH`.
The shift-faithful domain is the integers greater than one that are odd and
non-Mersenne; inputs and factor witnesses are checked against that declared
domain. The native word for a k-bit value has `5k + 4` glyphs, and its unique
`⊢` marker makes its literal ROTAT period equal that full length. This also
holds for even and Mersenne values, so those classes are not excluded because
of a shorter ROTAT orbit. The register records the Mersenne/non-Mersenne class
(`001000011100` / `111111111111`); parity is checked separately.

```bash
cargo run --release --bin vox -- factor-membrane 143 --factors 11 13
cargo run --release --bin vox -- factor-membrane --native-word '⊢≻⋈∈⊥∋≻⋈∈⊥∋≻⋈∈⊤∋≻⋈∈⊥∋⊙⊡⊣' --factors 11 13
cargo run --release --bin factor_membrane -- 143 --factors 11 13
```

The binary multiplication trace distinguishes nonzero carry columns from the
sum of carry values. The exact identity is
`sum(carry_out[k]) = popcount(p) * popcount(q) - popcount(N)`; the difference
`popcount(N) - popcount(p) - popcount(q)` is not a carry count. Per-input
period and cut residuals are reported as measured values for each input.

## Verdicts and B-cost

T closes, B holds a fork open across the cycle, N never forked, F ill-typed (∋ with no ∈). T needs work inside the paired region - bare split+fuse is μ∘δ=id and verifies nothing. B-cost lesson from self-lift: early return = fork that never rejoins; ∈-surplus rises with exits (0.28/5.26/12.91/15.00 at 0/1/2/4+ exits), so `vox self` reports the residual after exits are paid.

## Claim (checked, not decorated)

Every function in `corpus.c` (13 fns: arithmetic, div/mod, loops, SSE, recursion, calls, stack arrays, switch table, fn-pointer dispatch) runs twice - natively via ctypes and as IMASM - over identical inputs at `-O0..-O3,-Os` (`build_corpus.sh`): **0 mismatches**, incl. Fibonacci depth-29 (28M machine steps).

## Lanes, auditor, layout

Lanes: x86 (lifts+verdicts+runs), EVM/WASM/genetic (lift+verdict); genetic table generated from Lean (`gen_genetic_table.py` → both languages, no drift); CPython lane in `vox.py`. Auditor corollary: open fork across commit/return (Solidity reentrancy, early return, WASM escaped-`if`) all lift to `⊢∈⊡⊣ → B` - no patterns, no heuristics. Edges: unknown opcode → `None` + distance walked (never silent); tiny syscall subset (`-ENOSYS`); 50M-step cap; recursive-descent audit with honest coverage. Sources: `loader/x86/imasm_module/imasm_vm/vox/vox_decode/lanes/main.rs` + `corpus.c` + `vox.py`; mOMonadOS links this crate. Unlicense.

Full 382-line version: `README_backups/Vox_README.md`.

## Native numeral encoding — commuting structure

Every number is a word; the codec is exact, prefix-free, and bit-order symmetric. Every identity below was witnessed live (26/26 PASS over 1..2000, 64/127-bit randoms, and all 21 BIGBREAK cells; kernel compose witnesses in §4).

### 1. The word shape

```
example:  n = 42,  bits b₀ b₁ b₂ b₃ b₄ b₅ = 0 1 0 1 0 1    (LSB → MSB)

  ⊢ │ ≻⋈∈⊤∋ │ ≻⋈∈⊥∋ │ ≻⋈∈⊤∋ │ ≻⋈∈⊥∋ │ ≻⋈∈⊤∋ │ ≻⋈∈⊥∋ │ ⊙⊡⊣
      cell(b₀)  cell(b₁)  cell(b₂)  cell(b₃)  cell(b₄)  cell(b₅)
      └────────────────── the word of 42 ─────────────────────────┘

  cell(b) =  ≻⋈∈⊤∋   if b = 0
             ≻⋈∈⊥∋   if b = 1

  rules:   bits run LSB → MSB (cell 0 = b₀)
           the final cell is always ⊥ (the leading 1 bit)  →  prefix-free
           decode = inverse of encode (round-trip exact)
```

### 2. Shift σ and reversal R — the one non-commuting pair

```
              σ : prepend one ⊤-cell          (enc(2n) = σ(enc(n)))
   W(n)  ──────────────────────────────────►  W(2n)
     │                                         │
     │ R : reverse the cell order              │ R : reverse the cell order
     ▼                                         ▼
   R(W(n)) ────────────────────────────────►  R(W(2n))
              σ′ : append one ⊤-cell

   Read around the square:   R ∘ σ = append ∘ R
   reversal turns a PREPEND into an APPEND.
   That anticommutation is the ONLY non-commuting pair in the ring.
   Everything else commutes with everything:
     popcount ∘ {encode, σ, R}  are all equal   (popcount(enc(n)) = popcount(n))
     R conjugates the support:   support_MSB = { m−1−i : i ∈ support_LE }
```

### 3. Interlace Γ / deinterlace Λ — the braid pair

```
  input lanes (equal bit length):

        lane p :   p₀    p₁    p₂    p₃
        lane q :   q₀    q₁    q₂    q₃

                   Γ : weave — one cell of p, then one of q, alternating
                   ▼
        woven  :   p₀    q₀    p₁    q₁    p₂    q₂    p₃    q₃
                   pos 0    1     2     3     4     5     6     7
                   │
  ┌────────────────┴────────────────────────────┐
  │ Λ_even : keep even positions (0,2,4,6)      │ Λ_odd : keep odd positions (1,3,5,7)
  ▼                                             ▼
  lane p (recovered)                        lane q (recovered)

   Λ_even ∘ Γ = id      Λ_odd ∘ Γ = id        (projections recover each lane)
   Γ ∘ (Λ_even, Λ_odd) = id                    (re-weave recovers the input; even length)
   Γ ∘ swap = swap_pairs ∘ Γ                   (swap lanes ↔ swap each adjacent pair: S₂-equivariance)

   number level:   Γ#(p,q) = P₄(p) + 2·P₄(q),    P₄(n) = Σᵢ bᵢ(n)·4ⁱ,    P₄(2n) = 4·P₄(n)
                   ⇒  Γ#(2p,2q) = 4·Γ#(p,q)      (weaving commutes with doubling both lanes)

   v₂(n) = the initial run of ⊤-cells in enc(n)    (the file's composite.decomp-k / shift-factor)
```

### 4. Compose rule — live kernel witnesses (the defect commutes)

```
   ⊙> native_numeral compose 7 13               ⊙> native_numeral compose 13 7
   period(7)  = 20                              period(13) = 25
   period(13) = 25                              period(7)  = 20
   period(91) = 40                              period(91) = 40
   d = 20 + 25 − 40 = 5   rule (5 or 10): OK    d = 25 + 20 − 40 = 5   rule (5 or 10): OK

   Swap the factors: the two single periods exchange places, period(91) is
   fixed, and the defect is unchanged   →   d(p,q) = d(q,p).
   (d(p,q) = period(p) + period(q) − period(p·q) always lands in {5,10}.)
```

Full list of twelve: **(1)** codec involution dec∘enc = id = enc∘dec · **(2)** prefix-free unique decode · **(3)** shift σ commutes with popcount · **(4)** R² = id, R∘σ = append∘R · **(5)** support/polynomial conjugacy under R · **(6)** Γ/Λ braid pair · **(7)** Γ# = P₄ + 2P₄ homomorphism · **(8)** v₂ = initial ⊤-run · **(9)** compose-defect commutative (live) · **(10)** Belnap binary-gcd commutative (mirror N/T/F/B trace) · **(11)** kernel invariants commute with every cell operation (frame-sweep widths 2..8 preserve all bits; type hash 16389838/17280000 constant across all 21 cells; sieve certificate 53 primes / aperture 2⁸) · **(12)** orbit readout invariant under presentation (word, binary, bits-le, support, polynomial all pass the same asserts per cell).

μ∘δ = id
