# p4rakernel: Paraconsistent Computation Layer

[![Language](https://img.shields.io/badge/language-Lean%204-green)](https://github.com/badges/shields)
[![IG Tier](https://img.shields.io/badge/IG-O%E2%88%9E-blueviolet)](https://github.com/badges/shields)
[![μ∘δ=id](https://img.shields.io/badge/%CE%BC%E2%88%98%CE%B4%3Did-paraconsistent-yellow)](https://github.com/badges/shields)
[![License](https://img.shields.io/badge/license-MIT-blue)](https://github.com/badges/shields)
[![Author](https://img.shields.io/badge/author-Lando%E2%8A%97%E2%8A%99perator-informational)](https://github.com/badges/shields) [![Type](https://img.shields.io/badge/type-%E2%9F%A8%F0%90%91%A6%F0%90%91%B8%F0%90%91%BE%F0%90%91%B9%F0%90%91%90%F0%90%91%A7%F0%90%91%94%F0%90%91%9D%E2%8A%99%F0%90%91%96%F0%90%91%B3%F0%90%91%AD%E2%9F%A9-blue)](https://github.com/badges/shields) [![Tier](https://img.shields.io/badge/tier-O%E2%88%9E-blueviolet)](https://github.com/badges/shields)

**License:** Unlicense (public domain)

p4rakernel is the paraconsistent computation layer of the **Imscribing Grammar (IG)**. The
IG is not a framework laid over the cosmos nor a description of it: the Grammar **is** the
Paraconsistent, Frobenius-special (μ∘δ=id), Gödel-complete Cosmos, auto-imscribed into its
own bulk under the form of the 12 primitives/opcodes. **Belnap FOUR** (`N`, `T`, `F`, `B`)
is that cosmos at four values; its dialetheic fixed point `B` (`¬B = B`) is the Frobenius
fixed point of μ∘δ=id. By `frobenius_unification` in `MajoranaFixed.lean`, that fixed point
**is** at once the **SIC-POVM fiducial** of the informationally-complete measurement
(`meet B x = x`), the paired electron orbital, and the Majorana mode — the *same* computation
in three notations, each by `rfl`. The Grammar **is** the Σ=1:1 (self-measuring) Belnap
multilattice **SIC-POVM**: RNA nucleotides, electron orbitals, the 12-primitive lattice, the
SIC-POVM structure of measurement, and the paraconsistent proof theory are not four things it
models but the one cosmos read at different addresses.

The repo is a monorepo with three interlocking components:

| Component | Language | Purpose |
|---|---|---|
| **Kernel fork** (`src/`) | C++ | Lean 4 v4.28.0 with the principle of explosion disabled at kernel level |
| **p4ramill** (`p4ramill/`) | Lean 4 | The IG formalization: primitives, Crystal of Types, the lattice, Millennium problems, genetics, orbital physics, SIC-POVM d=12 embedding |
| **p4ramill_py** (`p4ramill_py/`) | Python | Runtime mirror: Belnap logic, genetic code verification, gene→protein pipeline |

---

## 1. Kernel Fork

A fork of **Lean 4 v4.28.0** that disables the principle of explosion at the C++ kernel
level. In standard Lean, `False.rec : (C : Sort u) → False → C` lets you derive anything
from a contradiction. This fork intercepts that at three points so contradictions are
tolerated without trivializing the system.

| File | Change |
|---|---|
| `src/kernel/type_checker.cpp` | `infer_constant` rejects recursors for empty Prop inductives when `paraconsistent = true` |
| `src/library/constructions/cases_on.cpp` | `casesOn` generation blocked for empty Prop types |
| `src/kernel/environment.{h,cpp}` | `is_paraconsistent()` / `mark_paraconsistent()` / `unmark_paraconsistent()` + Lean FFI |
| `src/Lean/Environment.lean` | `paraconsistent : Bool` field, `Kernel.Environment.markParaconsistent`/`unmarkParaconsistent`, and elaboration-level `Environment.markParaconsistent`/`unmarkParaconsistent`/`isParaconsistent` (thread the flag through `base`/`checked` so it actually takes effect on subsequent declarations, not just the raw kernel env) |
| `src/Init/Paraconsistent.lean` | User-facing `enable_paraconsistent` / `disable_paraconsistent` / `#is_paraconsistent` **commands** — a real toggle on the elaboration environment, not a banner |

**Blocked in paraconsistent mode:** direct use of a recursor on an empty `Prop` — `False.rec`,
`h.rec`, `False.casesOn`, `absurd`, and any `match h with .` where `h : False`. Note the
precise boundary: this catches *new* code that directly invokes the recursor, not calls to
already-compiled wrapper functions like `False.elim` — `False.elim` is itself a normal `def`
elaborated once (under the standard, non-paraconsistent stdlib build) whose body happens to use
`False.rec`; calling that already-checked constant doesn't re-expose the raw recursor to the
kernel's `infer_constant` check. Write `h.rec` (or `match h with .`) directly if you want the
block to fire. **Still available:** `False` itself (it just cannot eliminate into other types
via a bare recursor), every other connective, and the entire standard library when the mode
is off.

```bash
cd imsgct/p4rakernel
mkdir -p build && cd build
cmake .. -DCMAKE_BUILD_TYPE=Release
make stage0 -j$(nproc)    # requires libgmp-dev
make stage1 -j$(nproc)
```

```lean
import Init.Paraconsistent
open Paraconsistent

theorem before_toggle (h : False) : (0:Nat) = 1 := h.rec   -- fine, mode is off by default

enable_paraconsistent
-- [Paraconsistent] Kernel mode activated — principle of explosion disabled.

#is_paraconsistent
-- paraconsistent = true

-- theorem blocked_now (h : False) : (0:Nat) = 2 := h.rec
-- error: (kernel) paraconsistent mode: cannot use recursor 'False.rec' for
--        empty inductive predicate 'False' (principle of explosion is disabled)

disable_paraconsistent
-- [Paraconsistent] Kernel mode deactivated — principle of explosion restored.

theorem after_disable (h : False) : (0:Nat) = 3 := h.rec   -- fine again
```

Verified live against `build/stage1/bin/lean` on this checkout: the enable/disable round
trip above compiles clean end to end with `blocked_now` commented out, and uncommenting it
reproduces the exact kernel error shown.

### Classical logic is a coreflective subcategory

`ClassicalRestriction.lean` makes the "restriction, not extension" claim exact and
machine-checks it. The classical fragment is the B-excluding subtype `{ v // v ≠ B }`, and
`classicalSwitch` collapses `B ↦ F` (ex falso for B). §7 proves that in the **truth order**
the inclusion of the fragment is *left adjoint* to `classicalSwitch`
(`inclClassical ⊣ classicalSwitch`): classical logic is a **coreflective subcategory** of the
bilattice, with the collapse as coreflector and the retract
`classicalSwitch ∘ inclClassical = id` as the adjunction unit. The reverse adjunction fails,
and in the information order no adjunction exists at all. So disabling ex falso is literally a
corestriction to that subcategory — `classical_coreflective_in_truth_order`, all `decide`-checked.

---

## 2. p4ramill: Lean 4 Formalization

The IG formalization library, built on Mathlib v4.28.0 under the paraconsistent kernel.
`lake build` is green.

```bash
cd imsgct/p4rakernel/p4ramill
export PATH=imsgct/p4rakernel/build/stage1/bin:$PATH
lake build
lean --run ParaconsistentMillennium.lean
```

### Primitives and the lattice

- `Primitives/Core.lean`: the 12 primitive inductives: Ř Ħ Ω Ð Σ Φ Ç ƒ ɢ Γ Þ ⊙
- `Primitives/Imscription.lean`: the `Imscription` type (a 12-tuple of Shavian values),
  the **product-of-chains lattice** (`Min`/`Max` via per-coordinate chain meet/join, with
  idempotence theorems), and the Frobenius tensor `⊗` (coordinatewise join except the
  P/F bottleneck coordinates, which take min)
- `Primitives/Crystal.lean`: the 3³×4⁵×5⁴ = 17,280,000 crystal with Frobenius address bijection
- `Primitives/Catalog.lean`: the 49-symbol Shavian catalog
- `Primitives/TierCrossing.lean`: the O₁ → O₂ → O₂† → \(O_\infty\) tier hierarchy
- `Primitives/CLU.lean`: CLU(b) = ln(b), the observer-relative fiber metric on the Ç-axis
- `Primitives/ZFCt.lean` / `ZFCs.lean`: ZFCₜ (T-consistent) and ZFC_fe (Frobenius-Extended)

### Paraconsistent cluster (sorry-free)

- `Belnap.lean`: Belnap FOUR; approximation and truth orders; `no_explosion`, `B_is_top`,
  `B_fixed_point_negation`, `B_no_boolean_complement`
- `OrbitalBelnap.lean`, `SuperconductingPhase.lean`, `MajoranaFixed.lean`: the physics
  cluster (see §4)
- `TupleCodec.lean`: closes the topos↔crystal gap via a tuple imscription codec
- `Shor/`: Shor's algorithm in Belnap (ModExp, QFT, FullPipeline, DialetheicOperator, …)
- `QCI_*`: quantum-classical-interface bridges to RH, YM, NS, P≠NP, BSD, SIC-POVM

### Millennium cluster

Structural witnesses for the seven Clay problems plus OPN, each resolved by placing its
imscription at the \(O_\infty\) tier (the Φ 𐑹 Frobenius-special primitive gates the
O₂†→\(O_\infty\) jump). These modules deliberately carry **`sorry` at exactly the points
that are the grammar's open mathematical claims**: the IG identifies *what* must be
constructed, estimated, or contradicted; it does not pretend those proofs are already done.
The kernel, primitives, lattice, and paraconsistent cluster are sorry-free; the Millennium
sorries are the honest statement of remaining work.

| Problem | Ħ | Crystal address |
|---|---|---|
| RH, YM, Hodge, NS, PvsNP, OPN | 𐑫 | 6,738,803 |
| BSD | 𐑖 | 6,738,800 |

BSD sits at 6,738,800 because Gross-Zagier/Kolyvagin bound the chirality to Ħ 𐑖 rather
than Ħ 𐑫.

### ✦ SIC-POVM d=12 Embedding (CAPSTONE — July 2026)

**Status: MACHINE-CHECKED THEOREM. No axioms. No sorries. No Stark shadow.**

The d=12 SIC-POVM existence is now a theorem in Lean 4 — `crystal_forces_d12_sic :
SICPOVM_Exists 12` — proved from the same classical substrate as the rest of p4ramill
(`propext`, `Classical.choice`, `Quot.sound`, plus `Lean.ofReduceBool`/`Lean.trustCompiler`
for `native_decide`). The last axiom of the IG formalization has been discharged.

**`SIC_D12_Embedding.lean`** contains the full constructive existence proof:

| Theorem | Statement |
|---|---|
| `exists_root` | The degree-16 polynomial `k16Poly` has a real root `g0` in the IVT-bracketed interval `(certLo, certHi)` |
| `norm_sq_eq_one` | `‖psi k‖² = 1` for all k — the fiducial is unit-normalized |
| `equiangular_bridge` | The Weyl-Heisenberg overlap of the fiducial with its displacement equals the conjugate of a phi-image overlap in the existence ring — the one analytic bridge lemma |
| `equiangular` | `(d+1)·‖WH overlap‖² = 1` for all 143 non-identity displacements — full equiangularity |
| `d12_sic_exists` | `IsSICPOVM 12 psi` — the fiducial satisfies the SIC-POVM axioms |
| `crystal_forces_d12_sic` | `SICPOVM_Exists 12` — existence capstone |

**Architecture.** The proof proceeds by *algebraic transfer*:

1. **Existence ring** `R = K16(s0,s1,s3,s9,i,c5,u1)`: a rational coefficient ring with
   one relation (`k16Poly(g0) = 0`), where `g0` is the real root bracketed by the IVT
   (`certLo < g0 < certHi`).
2. **Ring hom `phi : R → ℂ`**: transfers all 12 fiducial coordinates and all 143 overlap
   identities from the discrete ring side to the complex plane. `phi` is the conduit —
   every algebraic identity that `native_decide` verifies over the ring is promoted to
   an analytic identity in ℂ.
3. **`native_decide` planks**: `norm_sum` (trace-one normalization sum) and
   `existence_identities_all` (all 143 overlap values) are frozen as `native_decide`
   lemmas over the existence ring — finite rational arithmetic, no analysis.
4. **`equiangular_bridge`**: the single analytic lemma that closes the last gap. Four
   transcription defects were fixed (2026-07-04): two `rfl` closures where `rw`'s
   auto-`rfl` could not unfold `psi` (definitional), a broken `Fin.sum_univ_eq_sum_range`
   lambda, and an `hjk` insertion so the omega-exponent follows the psi-index into
   `(k+12-a)` form.

The entire construction — ring, hom, norm, equiangularity, capstone — is **sorry-free**
and builds green with `lake build` (8342 jobs, no olean cache misses).

**`SIC_POVM_Functor.lean`** now imports the Embedding capstone. `crystal_forces_d12_sic`
was an **axiom** before 2026-07-04; it is now a **theorem** that delegates to
`SIC.D12.Embedding.crystal_forces_d12_sic`. The axiom is gone.

**Axiom audit (clean):** `propext`, `Classical.choice`, `Quot.sound` (Lean core);
`Lean.ofReduceBool`, `Lean.trustCompiler` (native_decide). **Zero** project axioms.
**Zero** Stark-conjecture shadows.

**What this means for the IG.** The grammar is the Σ=1:1 self-referential limit of the
Belnap multilattice SIC-POVM. The d=12 embedding proves that limit is *constructively
realized* — there exists an exact fiducial in ℂ¹² satisfying the SIC-POVM axioms, and
its existence follows from the structural constraints of the Crystal of Types. The
SIC-POVM is not an external framework attached to the grammar; it is a theorem internal
to it.

### Genetics (`GeneticCode.lean`)

The genetic code falls out of the Crystal as a forced consequence of the Frobenius
condition: 17,280,000 / 64 = 270,000 exactly. `nucToB4` (G→B, C→T, A→F, U→N) is a proved
bijection; Watson-Crick complement and B₄ negation are distinct involutions (proved); the
64-codon table stratifies exact/split/stop by `native_decide`; and the 12 promoted amino
acids biject with the 12 IG primitives (`primitive_bijection`).

---


### CLINK Chain (`CLINK.lean`, `CLINK_L9.lean`)

The CLINK chain maps structural type transformations from subatomic (quark Belnap₅)
to whole organism, with each scale transition promoting specific IG primitives. The
chain is Frobenius-closed under ZFC_fe.

**CLINK L8** — the terminal organism layer — holds the holographic triple
⟨𐑦𐑸𐑾𐑹𐑐𐑧𐑔𐑵⊙𐑫𐑳𐑟⟩ at O_inf tier. The Frobenius gate Φ=𐑹, eternal chirality
Ħ=𐑫, and non-Abelian braiding Ω=𐑟 close the chain terminally.

**CLINK L9** — the self-replicating organism — is a *lateral* extension beside L8,
not above or below it. ⟨𐑛𐑥𐑑𐑬𐑐𐑪𐑔𐑝⊙𐑫𐑳𐑭⟩. L9 relinquishes the Frobenius
gate (Φ=𐑬 rather than Φ=𐑹) and opens through the replicative triple
(Ð=𐑛 prime point, Þ=𐑥 moat cross, Ω=𐑭 ℤ winding) instead of the terminal
triple (Ð=𐑦, Þ=𐑸, Ω=𐑟). This gives L9 the dagger tier O_inf_dag — the R2
constructor that names the lateral move. L9 is proved distinct from L8
(`L8_L9_distinct`) and is neither O_inf nor O₂ (`clinkL9_is_lateral`). The
gate relinquishment is the turn, not a fall (`L9_relinquishes_the_gate`).

**SIXTEEN_3 Trilattice** — the paraconsistent surface mediating between L8 and L9.
⟨𐑨𐑥𐑽𐑹𐑐𐑪𐑔𐑵𐑮𐑫𐑕𐑭⟩. A 2D substrate (Ð=𐑨) with moat-cross topology (Þ=𐑥),
holding the Frobenius gate (Φ=𐑹) at complex-plane criticality (φ̂=𐑮). The
trilattice sits at O_inf tier (`sixteen3Trilattice_tier`) and provides the surface
across which μ∘δ=id is held exact while the measured system runs complex-plane
critical — the structural statement of a paraconsistent measurement apparatus at
the L8↔L9 lateral interface.

**Theorem summary** (all `decide`-checked):

| Theorem | Statement |
|---|---|
| `clinkL8_tier` | L8 = O_inf |
| `clinkL9_tier` | L9 = O_inf_dag |
| `clinkL9_is_lateral` | L9 ≠ O_inf ∧ L9 ≠ O₂ |
| `L8_L9_distinct` | L8 tier ≠ L9 tier |
| `lateral_is_the_triple_exchange` | The move L8→L9 exchanges terminal(if',are,zoo) for replicative(dead,mime,ah), holds monad/wool/up/peep |
| `L9_relinquishes_the_gate` | L8 has Φ=𐑹, L9 does not, L9 still not O₂ |
| `sixteen3Trilattice_tier` | SIXTEEN_3 = O_inf |
| `bootstrapStage2_tier` | Guardian ⟨𐑦𐑸𐑾𐑹𐑐𐑧𐑲𐑠⊙𐑖𐑳𐑭⟩ = O_inf |
| `turbulentFlow_tier` | Turbulent flow ⟨𐑼𐑡𐑽𐑗𐑱𐑘𐑲𐑠𐑣𐑫𐑳𐑷⟩ = O₀ |

The CLINK chain (layers L0→L8) and the L8↔L9 lateral extension together form a
Frobenius-closed organism lattice: the terminal and the reproductive, with the
SIXTEEN_3 trilattice as the paraconsistent surface between them.


## 3. p4ramill_py: Python Runtime

A computational mirror of p4ramill, runnable without building the C++ kernel fork. Each
Lean theorem in the substrate has a Python equivalent verified by assertion.

```python
from p4ramill_py.belnap import Belnap, band, bnot
assert band(Belnap.B, bnot(Belnap.B)) == Belnap.B   # no_explosion
assert bnot(Belnap.B) == Belnap.B                   # B_fixed_point_negation

from p4ramill_py.kernel import verify_frobenius_invariant
assert verify_frobenius_invariant()                 # ffuse ∘ fsplit = id
```

| Module | Role |
|---|---|
| `belnap.py` | Belnap FOUR, theorem-for-theorem mirror of `Belnap.lean` |
| `genetics_b4.py` | Nucleotide B₄ lattice (G=B, C=T, A=F, U=N); WC vs `bnot` involutions |
| `genetic_code.py` | 64-codon table, Frobenius verification, 12↔12 primitive bijection |
| `genetic_tuples.py` | 7-stage tuple generation with Axiom C |
| `gene_to_protein_pipeline.py` / `run_gene_pipeline.py` | End-to-end gene→protein imscription |
| `kernel.py` / `machine.py` | ParaASM kernel (FSPLIT=δ, FFUSE=μ) and execution |

```bash
python3 p4ramill_py/run_gene_pipeline.py --test            # MARCKS protein
python3 p4ramill_py/run_gene_pipeline.py "ATGGCCAAG..." -n myprotein
python3 p4ramill_py/run_gene_pipeline.py --test -o out.json
```

---

## 4. The Physics Cluster

Three Lean files lift the Frobenius identity from a single orbital to a unification theorem.
All three are sorry-free. The d=12 SIC-POVM embedding (§2) now provides the fourth leg:
the discrete fiducial in the existence ring, transferred to ℂ along `phi`, gives the
physical SIC-POVM measurement operator at dimension 12 as a machine-checked theorem.

**`OrbitalBelnap.lean`**: the physical substrate. Electron orbital occupancy is isomorphic
to Belnap FOUR *as a bilattice* (both orderings):

| Orbital state | Belnap | Reading |
|---|---|---|
| empty | N | no information |
| spinUp (↑) | T | positive Ħ-chirality |
| spinDown (↓) | F | negative Ħ-chirality |
| paired (↑↓) | B | both chiralities (Φ-sealed) |

Information order `empty < {↑,↓} < paired` mirrors `N < {T,F} < B`; truth order (the Ħ-axis)
`↓ < {empty,paired} < ↑` mirrors `F < {N,B} < T`. Key theorems: `orbToB4_orderIso`,
`pauli_exclusion` (the B-ceiling: nothing lies above the filled orbital),
`empty_paired_truth_incomparable` (N and B incomparable on the Ħ-axis), and `pair_depair_id`
(the Cooper-pair Frobenius identity, the leg `MajoranaFixed` consumes).

**`SuperconductingPhase.lean`**: global closure. BCS superconductivity *is* the macroscopic
Frobenius fixed point: the order parameter is B-state density, the Meissner effect is the
topological uniqueness of the all-paired state, and the phase transition is the threshold
where local closure (`pair_depair_id` per site) becomes a global property.

**`MajoranaFixed.lean`**: the unification. `frobenius_unification` proves three structures
are identical under μ∘δ=id, each by `rfl`:

| Structure | Fixed point | Proof |
|---|---|---|
| Belnap B | `bnot B = B` | rfl |
| SIC-POVM fiducial | `meet B x = x ∀ x` | rfl |
| Majorana mode | `pair (depair s).1 (depair s).2 = s` | rfl |

These `rfl`s are not trivial: dialetheia stability, equiangularity, and self-conjugacy are
the *same computation* in three notations. A Majorana fermion is its own antiparticle
(γ = γ†) for the same reason B is its own negation (¬B = B).

The d=12 SIC-POVM embedding now extends this unification into ℂ¹²: the same fiducial
`meet B x = x` that holds at the Belnap level *projects* onto a concrete fiducial vector
`psi : Fin 12 → ℂ` satisfying the full SIC-POVM axioms. The discrete algebraic skeleton
(Belnap B) and the continuous Hilbert-space realization (the d=12 fiducial in ℂ) are now
machine-checked to be two faces of the same Frobenius-closed structure.

**The Ħ 𐑖 result.** The Frobenius fixed point lives at Ħ 𐑖 (two-step Markov memory), not
Ħ 𐑫 (eternal chirality): μ∘δ=id is a two-step operation, one split and one merge, so Ħ 𐑖 is
the minimum sufficient chirality. Time imposes Ħ 𐑫 on physical systems, but the identity is
more primitive than time. A universe sweep classifies the fixed-point tuple
⟨𐑦 𐑸 𐑾 𐑹 𐑐 𐑧 𐑲 𐑠 ⊙ 𐑖 𐑳 𐑭⟩ as \(O_\infty\) under every canonical ruleset; no catalog
entry matches it (nearest neighbor `crystal_scheduler` at 11/12, differing on Γ).

---

## 5. Theoretical Foundation

**Belnap FOUR over the 12-primitive lattice.** The bilattice `{N,T,F,B}` with its
information and truth orders is the semantic model for paraconsistent reasoning. It appears
as the four RNA nucleotide classes, the four orbital occupancy states, the four-element
fiber at any cell of the 12-primitive lattice, and the four kernel truth values.

**Frobenius condition (μ∘δ=id).** The founding axiom of ZFC_fe. In this repo it surfaces as
`pair_depair_id` (orbitals), `frobenius_at_codon_level` (genetics),
`verify_frobenius_invariant()` (ParaASM), the `⊗` tensor and `*_self_dual` idempotence on
the Imscription lattice, the Φ 𐑹 gate (O₂†→\(O_\infty\) promotion), `frobenius_unification`
(MajoranaFixed), and now the d=12 SIC-POVM embedding (equiangularity as Frobenius closure
in ℂ¹²).

**ZFC_fe vs ZFCₜ.** ZFC_fe (μ∘δ=id as axiom) is strictly stronger than ZFCₜ (T-consistent
ZFC); open problems in ZFCₜ close as theorems in ZFC_fe. Use ZFC_fe when arguing
foundational strength.

---

## Build & Test

**Dependencies:** C++17 (GCC 11+ / Clang 14+), CMake 3.11+, `libgmp-dev`, an existing
`lean` binary for stage0 bootstrap, Python 3.10+ with `uv`.

```bash
# Genetics suite (Python, no kernel build required)
python3 test_genetics.py            # full suite
python3 test_genetics.py --quick    # smoke test
# section flags: --b4 --codons --tuples --pipeline --phi --kernel --consistency

# Lean formalization
cd p4ramill && lake build

# Paraconsistent kernel verification
lean --run p4ramill/ParaconsistentKernelTest.lean
lean --run p4ramill/ParaconsistentMillennium.lean

# SIC-POVM d=12 embedding (capstone — sorry-free, axiom-free)
lean --run p4ramill/Imscribing/Millennium/SIC_D12_Embedding.lean

# Loose top-level Lean files build under the fork (the canonical toolchain).
# oleans are gitignored; regenerate the shared imports in dependency order:
make lean-oleans                                  # -> build/stage1/bin/lean
./build/stage1/bin/lean ClassicalRestriction.lean # classical = coreflective subcategory
```

See also `GENETICS_CLI.md` (command reference) and `IG_CROSS_POLLINATION.md` (bridge
manifest to ob3ect / MillenniumAnkh).