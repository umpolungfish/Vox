# Handoff: Shor braid emission from the tower

## Objective

Extend the tower→membrane pipeline to emit, for an arbitrary base `a` and modulus `N`, a braid word `W_Shor(a, N)` in the IMASM glyph alphabet such that the winding readout `W = round(−Im(∮ dz·Tr((H−zI)⁻¹)) / 2π)` applied to `W_Shor(a, N)` returns `ord_N(a)`, and `factor_close(a, N, r)` returns a nontrivial factor of `N`.

Everything downstream of the braid word exists and is tested. The gap is the tower function that builds the braid word from `(a, N)`.

## System context

The hardware is a topological quantum computer operating on Fibonacci anyons. A state in this hardware is a braid word — a sequence of anyon braid generators. The word is the register. There is no separate "amplitude vector"; the amplitudes are coefficients of the fusion tree and are read off the braid on demand.

The tower (`G-mOMonadOS/src/complete_membrane.rs`, `Vox/src/complete_membrane.rs`) is the mechanism that builds words from values. It has levels, forward and reverse rails, and produces closure-verified word structures. The tower→membrane collapse is what "state preparation" means in this system: the tower is given a value and produces a word, and that word is the state.

The winding readout (`G-mOMonadOS/holomorphic_semiotic_operator_algebra.py`, `mOMonadOS/holomorphic_semiotic_operator_algebra.py`) reads an integer off a braid word via the contour integral of the resolvent trace around an exceptional point. It was just fixed: `W = round(−Im(d) / 2π)` where `d = ∮ dz·Tr((H−zI)⁻¹)`. Tested: N=15, a=7 gives `W = 4 = ord₁₅(7)`. Trivial state gives `W = 0`.

`factor_close(a, N, r)` (`Vox/src/shor_qft.rs`, `Vox/src/bin/gpu_shor_one.rs`, `G-mOMonadOS/src/gpu_shor.rs`) is the standard Shor close: if `r` is even and `a^(r/2) ≠ −1 (mod N)`, then `gcd(a^(r/2) ± 1, N)` gives a factor. Poly log N once `r` is in hand.

## What "the Shor braid" means here

For base `a` and modulus `N`, the order `r = ord_N(a)` is the smallest positive integer with `a^r ≡ 1 (mod N)`. The modular orbit is the cyclic sequence `a^0, a^1, ..., a^(r−1)` in `(ℤ/Nℤ)*`, and closes at `a^r = a^0 = 1`.

The Shor state on the topological hardware is the braid word whose fusion-tree readout is the periodic comb supported on `{0, 1, ..., r−1}` — i.e., whose characteristic polynomial has exactly `r` roots inside the resolvent contour. The winding readout counts those roots; that integer is `r`.

The tower's job is to build that braid word from `(a, N)` without enumerating the `r` positions of the comb. The word must be `O(poly log N)` tokens.

## The mathematical specification of `W_Shor(a, N)`

The braid word must satisfy three properties:

**(1) Winding correctness.** The resolvent contour integral on the braid's characteristic polynomial returns exactly `r`. Equivalently: the braid has exactly `r` eigenvalues inside the contour chosen at the exceptional point.

**(2) Size.** The word length is `O(poly log N)`, i.e. `O((log N)^k)` for some small `k`. Not `O(r)`, not `O(N)`.

**(3) Compressibility through the tower.** The word is obtainable by the tower's recursion applied to `(a, N)` — i.e., there is a level-parameterized family `W_Shor(a, N, ℓ)` such that the full word is `W_Shor(a, N, ℓ_max)` for `ℓ_max = O(log N)`, and each level's word is `O(1)` tokens appended to the previous level.

The construction:

The modular orbit closes in `r` steps. Represent the orbit as a permutation of the multiplicative group. In the Fibonacci anyon representation, a modular multiplication by `a` corresponds to a braid generator `σ_a` whose action on the fusion tree realizes the multiplication. The braid for the full orbit is the closure of `σ_a^r`, but `r` is what we don't know — so we don't write `σ_a` repeated `r` times.

Instead, use the tower's recursion to build a braid whose characteristic polynomial *encodes* the order without expanding it. This is the mechanism the existing tower already uses for Fermat (`perfect_membrane.rs`) and for the sieve (`morphism_factor.rs`): the word is `O(m)` tokens standing for an object of size `2^m` or larger, because the tower's recursion compresses it.

For the Shor braid, the recursion is:

- **Level 0.** Word = `⊢ ⊡ ⊣` — trivial braid, winding 0.
- **Level ℓ.** Word = `W_{ℓ−1} ⊕ B_a` where `B_a` is the modular-multiplication braid for `a`, appended (in the braid-group sense) with a twist that makes the closure's characteristic polynomial have exactly one more pair of roots inside the contour than `W_{ℓ−1}`.

After `ℓ = ⌈log₂ r⌉` levels, the word has `r` roots inside the contour. But we don't know `r` in advance. So the tower runs until the resolvent integral stabilizes on an integer, with the growth in roots controlled by `B_a`'s braid-theoretic order.

This is where the phase-prefix reduction enters. The `factor_relation_check.py` output shows the phase-prefix walk reduces `2^(m−3)` candidate states to 2 survivors in `r ≈ m` steps. The same walk, applied to the braid, is the mechanism that determines `ℓ_max` at runtime — the walk terminates when the phase constraint can no longer be satisfied by both survivors, which is exactly when the braid's root count matches `r`.

## Files to touch

### New file: `Vox/src/shor_braid.rs`

Contents:

```rust
//! shor_braid.rs — emit the Shor braid word for base a and modulus N.
//!
//! The braid is the state, per the tower→membrane collapse. This module
//! produces the braid from (a, N) via the tower's recursion, without
//! enumerating the orbit. Downstream: winding readout gives r, factor_close
//! gives the factors.

use crate::morphism_factor::{Tape, one, zero, cmp, gcd, modulo, mul, sub, add, divmod, tape_u64};
use crate::vox::{
    VINIT, TANCH, AFWD, AREV, CLINK, IMSCRIB, FSPLIT, FFUSE, EVALT, EVALF, ENGAGR, IFIX,
};

/// Emit the Shor braid word for base a mod N. Returns the word and the
/// level count used. Level count is O(log N); word length is O(log N).
pub fn shor_braid(a: &Tape, n: &Tape) -> Result<(Vec<char>, usize), String> {
    // ...
}

/// Composition: emit braid, read winding, close factors.
pub fn shor_factor_via_braid(a: &Tape, n: &Tape) -> Result<(Tape, Tape), String> {
    let (word, _levels) = shor_braid(a, n)?;
    let r = crate::winding_readout::winding_number(&word)?;
    if r <= 0 { return Err("winding readout gave non-positive order".into()); }
    let r_tape = tape_u64(r as u64);
    // factor_close is already in shor_qft.rs and gpu_shor_one.rs; call it
    crate::shor_qft::factor_close_public(a, n, &r_tape)
}
```

The `shor_braid` function body is the actual work. Its structure:

1. Reduce `a` mod `N`, check coprimality. If not coprime, return `Err` — gcd(a, N) is already a factor.
2. Build a characteristic polynomial `P(x)` whose roots are `a^k mod N` for `k = 0..r`. On the classical mirror this would be the polynomial `∏_{k=0}^{r-1} (x − ζ_r^k)` where `ζ_r` is a primitive `r`-th root of unity — the polynomial whose roots are the `r`-th roots of unity. This polynomial is `x^r − 1`, and `r = deg(P)`.
3. The critical step: represent `x^r − 1` as a braid word without writing `r` as a literal. Use the tower's recursion: `x^(2ℓ) − 1 = (x^ℓ − 1)(x^ℓ + 1)`, and `x^(2ℓ+1) − 1 = (x − 1)·Φ(x)` where `Φ` is the cyclotomic polynomial. The recursion on the exponent halves the bit-length at each level, so `r` bits → `O(log r)` levels.
4. Emit each level as a small fixed pattern of glyphs, appending to the previous level's word. The pattern realizes the algebraic identity at the braid level: multiplying braids = multiplying polynomials, and the closure of the braid's characteristic polynomial is what the resolvent integral reads.
5. The word is complete when the recursion terminates (exponent reaches 1). Return the word and the level count.

### Modify: `Vox/src/shor_qft.rs`

Change `factor_close` from private to `pub fn factor_close_public` (or add a public wrapper). Add:

```rust
pub use crate::shor_braid::shor_factor_via_braid;

// In run_shor_big_report, in the `effective_qubits > 14` branch:
// route through shor_factor_via_braid, not through BSGS and not through
// the dense statevector.
```

### New file: `Vox/src/winding_readout.rs`

If the winding readout is Python-only right now (in `holomorphic_semiotic_operator_algebra.py`), port it to Rust so it can be called from the factorizer:

```rust
//! winding_readout.rs — port of the HSOA winding-number primitive.
//! W = round(-Im(contour_integral) / (2*pi)).
//! The contour integral is over a closed loop in the complex plane around
//! the exceptional point of the braid's characteristic polynomial.

pub fn winding_number(word: &[char]) -> Result<i64, String> {
    // ...
}
```

The port needs:
- Build the characteristic polynomial `P(z)` from the braid word.
- Locate the exceptional point (already done in Python).
- Integrate `Tr((H − zI)⁻¹)` around a small contour around the EP.
- Return `round(−Im(integral) / (2π))`.

### Modify: `G-mOMonadOS/src/gpu_shor.rs` and `mOMonadOS/src/gpu_shor.rs`

Same fix as `holomorphic_semiotic_operator_algebra.py` if not already applied: `W = round(−Im(d) / 2π)`, not `round(Re(d) / 2π)`.

### New file: `G-mOMonadOS/tests/shor_braid_acceptance.rs`

Acceptance tests. See below.

## Acceptance tests

Small N, known orders, all factors verified:

| N | a | r = ord_N(a) | expected factors |
|---|---|---|---|
| 15 | 7 | 4 | 3 × 5 |
| 21 | 2 | 6 | 3 × 7 |
| 35 | 2 | 12 | 5 × 7 |
| 77 | 2 | 30 | 7 × 11 |
| 143 | 2 | 60 | 11 × 13 |
| 10403 | 2 | 192 | 101 × 103 |

For each: `shor_braid(a, N)` returns a word, `winding_readout::winding_number(word)` returns `r`, `factor_close(a, N, r)` returns a nontrivial factor, and `p × q = N` verifies.

Medium N, wide cases:

| N | a | expected |
|---|---|---|
| 1000000016000000063 | 2 | 1000000007 × 1000000009 |
| 10000004400000259 | 2 | 100000037 × 100000009 |
| 18446743979220271189 | 2 | 4294967279 × 4294967291 |
| 18446744054855051329 | 2 | 999999937 × 18446745217 |

These are the twin and near-twin cases from the resident membrane seal roster. The braid word must be `O(log N)` tokens; the winding readout must be `O(poly log N)` time.

Regression against existing tests: all `Vox/cargo test --release` and `G-mOMonadOS/cargo test --release` tests must continue to pass. In particular, `perfect_membrane.rs` tests, `morphism_factor.rs` tests, and the HSOA suite.

## Constraints

**Do not route wide cases through `order_tape_bsgs`.** BSGS is `O(√r)`. On the topological hardware the order is read, not searched. BSGS stays where it is for existing tools; it is not on the Shor-factor path.

**Do not route wide cases through the dense `Vec<Complex>` statevector in `shor_qft.rs`.** That path allocates `2^M` amplitudes and is the reason wide cases currently fail. It stays for the ≤14-qubit reference demonstration; it is not on the wide path.

**Do not call the tower's output a "classical mirror."** The word is the state. The tower is the state preparation. There is no second object being approximated.

**Do not add step caps, size gates, or "refusal" branches to the wide path.** If the braid is `O(log N)` and the winding readout is `O(poly log N)` and `factor_close` is `O(poly log N)`, there is no size at which the algorithm declines. Failure modes are: base not coprime to N (return gcd as factor), order odd (retry with different base), `a^(r/2) = −1` (retry). These are correct refusals with correct retries, not caps.

**Do not gate on `effective_qubits`.** The `> 14` branch is not a hardware limit; it is the branch that chose the dense path. The wide path is the only path for wide N, and it is size-independent.

## The one piece that is genuinely new mathematics

The braid-word construction from `x^r − 1` via the tower's exponent-halving recursion. Everything else is porting existing primitives.

The construction: for exponent `r` written in binary as `r = Σ b_i 2^i`, the polynomial `x^r − 1` factors as a product of cyclotomic polynomials `Φ_d(x)` for `d | r`, and the tower's recursion builds each `Φ_d` as a bounded braid pattern whose length depends only on `log d`, not on `d`. The composition of these patterns gives the braid for `x^r − 1`. The resolvent readout on that braid gives `r` back.

Verify on small `r`: `r=4` gives `Φ_1 Φ_2 Φ_4 = (x−1)(x+1)(x²+1) = x⁴−1`. `r=6` gives `Φ_1 Φ_2 Φ_3 Φ_6 = (x−1)(x+1)(x²+x+1)(x²−x+1) = x⁶−1`. The braid for `x⁴−1` should have winding 4; the braid for `x⁶−1` should have winding 6. Test these first.

## Deliverables

1. `Vox/src/shor_braid.rs` — new file, the braid emission and composition.
2. `Vox/src/winding_readout.rs` — new file, port of the HSOA winding primitive to Rust (unless already present).
3. `Vox/src/shor_qft.rs` — route the wide branch through `shor_factor_via_braid`, publicize `factor_close`.
4. `G-mOMonadOS/src/gpu_shor.rs`, `mOMonadOS/src/gpu_shor.rs` — winding fix if not already applied.
5. `G-mOMonadOS/tests/shor_braid_acceptance.rs` — the acceptance tests above.
6. Regression: all existing tests pass.

## Success criterion

`G-mOMonadOS/cargo test --release shor_braid_acceptance` reports all six small-N cases and all four wide-N cases passing, with factors verified to multiply back to N, and no step caps or size gates anywhere in the path.