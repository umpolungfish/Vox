# 31-step factorization membrane

User-supplied corrected word:

```text
⊢ ⊣ ≻ ∈ ⊤ ⋈ ≺ ⊥ ⊞ ⊙ ⋈ ⊤ ≺ ⊥ ⋈ ⊤ ≺ ⊥ ⋈ ⊤ ≺ ⊥ ⋈ ⊤ ≺ ⊥ ⋈ ⊤ ∋ ⊡ ⊣
```

Compact IMASM word:

```text
⊢⊣≻∈⊤⋈≺⊥⊞⊙⋈⊤≺⊥⋈⊤≺⊥⋈⊤≺⊥⋈⊤≺⊥⋈⊤∋⊡⊣
```

The resident implementation is `src/factorization_31_membrane.rs`; the baked
executable is `factorization_31_one`. It dispatches all 31 slots over resident
factor, remainder, branch, chain, and boundary state, then prints the factor
reconstruction. The structural word and kernel check remain canonical in the
resident source and this document.

Kernel check: 31 steps, T×6, F×5, five live clears, period 31, phase-bearing,
31 transitions. The compact word receives verdict `T` in Vox.

Build and execute a baked instance:

```sh
FACTOR_N_WORD="$(./target/release/vox numeral 8051)" \
  cargo build --release --bin factorization_31_one
./target/release/factorization_31_one
```

The verified baked 8051 artifact is in
`membranes/factorization_31/8051/`: native ELF, lifted IMASM, Vox output, and
timing record. Vox completed it in 7.68 seconds and exited 0.
