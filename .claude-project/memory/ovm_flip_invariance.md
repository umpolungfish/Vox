---
name: ovm_flip_invariance
description: OVM master §2.5's positivity-flip invariance (all flips cost 3.11) is theoretical and wrong as stated; measured on registered catalog entries the PURE flip (≺𐑹→𐑿, ⊡𐑴→𐑭) is invariant at 0.700/1.4727, and the 𐑷 transitions are compound moves, not flips
metadata:
  type: project
---

**Closes part of OVM_MASTER_MATHEMATICS §10.2 (distance calibration), which was open because "none of
the 24 OVM types are registered".** They are now — 16 of 18 checked are in IG_catalog.json, so the
master's §10.1 is stale and its distances became checkable for the first time.

Measured with the grammar's own instrument (`imscrbgrmr.cli distance`), not a hand-rolled metric:

| pair | d(sym) | d(Mahalanobis) | marks moved |
|---|---|---|---|
| SIC-POVM / SIC-NOVM | 0.700 | 1.4727 | ≺𐑹→𐑿, ⊡𐑴→𐑭 |
| SIC-O-POVM / SIC-O-NOVM | 0.700 | 1.4727 | ≺𐑹→𐑿, ⊡𐑴→𐑭 |
| A⁻-IC-POVM / A⁻-IC-NOVM | 0.700 | 1.4727 | ≺𐑹→𐑿, ⊡𐑴→𐑭 |
| AI-CPOVM / AI-CNOVM | 2.400 | 3.4380 | ≺, ∈, ∋, ⊡𐑷→𐑭 |
| S-PC-POVM / S-PC-NOVM | 1.400 | 2.9454 | ⊣, ≺, ∈, ∋, ⊞, ⊡𐑷→𐑭 |

**The theorem as written is false; restricted, it is exact.** §2.5 asserts all four flips cost 3.11.
No pair measures 3.11. Three measure 0.700/1.4727 to four decimals — those are the ones that move
exactly two marks, one from each dual pair (≻↔≺ and ⊞↔⊡), leaving from 𐑴. The two that break it both
leave from 𐑷 and move four and six marks. They are compound transitions, not flips.

**Corrected statement:** the PURE positivity flip — ≺ 𐑹→𐑿 with ⊡ 𐑴→𐑭 — has invariant cost across
symmetry class and across dynamics. Nothing else in the taxonomy is that operation.

**THE REACHABILITY HALF IS THE PLANTING RULE RE-DERIVED. Not new.** Lando caught this immediately.
`P_doublebarpipe` is the old name for ≺=𐑹, exact self-duality, and the core theory states the rule in
several places: P-438 "O₂→O_∞ crossing requires planting, not achievement"; §23 Frobenius
Non-Synthesizability, "one cannot assemble Cantor and Gödel and get the grammar by composition — the
grammar must be recognized whole, or not at all"; IG_ONTICS §2492, "the method appropriate to O_∞ is
planting, not composition". Saying that 𐑷→𐑭 never appears as a pure two-mark move and needs a phase
transition rather than tuning is that rule in OVM coordinates. Cite the planting rule; do not restate
it as a finding.

**What is actually new here is only the CALIBRATION** — that no measured pair costs 3.11, that three
measure 0.700/1.4727 identically, and that §2.5's theorem holds only when restricted to the pure
two-mark flip. That part answers §10.2 and was not computable before the cells were registered.

**The confirmation, stated as confirmation:** The algebra says an
OVM at ⊡=𐑷 cannot reach the Grammar's 𐑭 by winding alone and needs a topological phase transition.
Nowhere in the registered catalog does 𐑷→𐑭 occur as a pure two-mark move — every instance drags four
to six marks. The theoretical taxonomy costed it as one uniform operation; the entries say it is a
different kind of move.

Still open in §10: 10.3 (type-convergence proof — and note its proposed test is self-defeating, since
the claim is that A⁻-IC-POVM-O and SIC-O-POVM have IDENTICAL tuples, so there are not two entries to
compare; the four antisymmetric-oscillating cells are unregistered), 10.4, 10.5, 10.6, 10.7, 10.9.
[[conjectures_are_povms]] [[millennium_as_ovms]]
