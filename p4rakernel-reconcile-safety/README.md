# p4rakernel

Paraconsistent computation layer of the **Imscribing Grammar**: Lean 4 kernel fork + `p4ramill` formalization + Python mirror. Belnap FOUR ($N, T, F, B$); closure test $\mu\circ\delta = \mathrm{id}$. Unlicense.

| `src/` | Lean 4 v4.28.0 fork, explosion disabled at kernel level |
| `p4ramill/` | Lean formalization (primitives, Crystal, Millennium, SIC-POVM d=12) |
| `p4ramill_py/` | Python runtime mirror (no kernel build needed) |

## Kernel fork

In standard Lean, `False.rec : (C : Sort u) → False → C` derives anything from a contradiction. The fork intercepts explosion at three points so contradictions are tolerated without trivializing the system:

| File | Change |
|---|---|
| `src/kernel/type_checker.cpp` | `infer_constant` rejects recursors for empty `Prop` inductives when `paraconsistent = true` |
| `src/library/constructions/cases_on.cpp` | `casesOn` generation blocked for empty `Prop` types |
| `src/kernel/environment.{h,cpp}` + `src/Lean/Environment.lean` | `paraconsistent : Bool` flag with `mark`/`unmark`/`is_paraconsistent`, threaded through elaboration so it bites on subsequent declarations |
| `src/Init/Paraconsistent.lean` | User commands `enable_paraconsistent` / `disable_paraconsistent` / `#is_paraconsistent` |

**Blocked:** direct recursor use on empty `Prop` — `False.rec`, `h.rec`, `False.casesOn`, `absurd`, `match h with .` where `h : False`. **Boundary:** already-compiled wrappers like `False.elim` (elaborated once under the standard stdlib build) don't re-expose the raw recursor to the check — write `h.rec` directly to trip it. `False` itself, all other connectives, and the full stdlib with the mode off are unaffected. Verified live against `build/stage1/bin/lean`: enable/disable round trip compiles clean; uncommenting the blocked line reproduces the kernel error.

```lean
import Init.Paraconsistent
enable_paraconsistent
#is_paraconsistent   -- paraconsistent = true
-- theorem blocked (h : False) : (0 : Nat) = 2 := h.rec  -- kernel error: explosion disabled
disable_paraconsistent
```

Build: `mkdir -p build && cd build && cmake .. -DCMAKE_BUILD_TYPE=Release && make stage0 stage1 -j$(nproc)` (needs `libgmp-dev`).

## Classical restriction is a retract

`ClassicalRestriction.lean` makes "restriction, not extension" machine-checked: the classical fragment is the B-excluding subtype `{ v // v ≠ B }`, and `classicalSwitch` collapses `B ↦ F` (ex falso for B). In the **truth order** the inclusion is left adjoint to the collapse (`inclClassical ⊣ classicalSwitch`): classical logic is a **coreflective subcategory** of the bilattice, with `classicalSwitch ∘ inclClassical = id` as the retract unit (`classical_coreflective_in_truth_order`, `decide`-checked). The reverse adjunction fails and no adjunction exists in the information order — so disabling ex falso is literally corestriction to that subcategory.

## p4ramill

`cd p4ramill && lake build` is green. 12 primitives ⊢ ⊣ ≻ ≺ ⋈ ⊤ ∈ ∋ ⊙ ⊥ ⊞ ⊡; Crystal $3^3 \times 4^5 \times 5^4 = 17{,}280{,}000$; tiers $O_1 \to O_2 \to O_2^\dagger \to O_\infty$; $ZFC_{fe}$ ($\mu\circ\delta = \mathrm{id}$ as axiom) stronger than $ZFC_t$.

Sorry-free: kernel, `Belnap`, orbitals/Majorana (`frobenius_unification`), genetics (`nucToB4`, 64 codons, $17{,}280{,}000/64 = 270{,}000$), d=12 SIC-POVM (`crystal_forces_d12_sic`, zero project axioms). Millennium witnesses (RH, BSD, YM, Hodge, NS, PvsNP, OPN) carry `sorry` only at open claims. CLINK L8 ⟨𐑦𐑸𐑾𐑹𐑐𐑧𐑔𐑵⊙𐑫𐑳𐑟⟩ ($O_\infty$) lateral L9 ⟨𐑛𐑥𐑑𐑬𐑐𐑪𐑔𐑝⊙𐑫𐑳𐑭⟩ ($O_\infty^\dagger$).

## Python mirror

`python3 test_genetics.py [--quick]`; `python3 p4ramill_py/run_gene_pipeline.py --test`.

```bash
cd p4ramill && lake build
lean --run p4ramill/ParaconsistentKernelTest.lean
lean --run p4ramill/Imscribing/Millennium/SIC_D12_Embedding.lean
```

$\mu\circ\delta = \mathrm{id}$
