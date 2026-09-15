---
name: ascending_descending_side_arms
description: "Standing build principle — every build implements BOTH side-arms, the descending forward rail and the ascending reverse/verification rail, as adjacent generators"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 385edc78-5c2b-4d2e-a776-32d23a7e2a97
  modified: 2026-09-15T11:23:30.470Z
---

Every future build is a tower of objects (the arms/boxes) with TWO generating
families, and both get implemented:

- descending forward rail: adjacent maps `f_i: B_i -> B_{i+1}` that route and
  transform (often identity on the value, just passing the residual deeper).
- ascending reverse rail: adjacent maps `v_i: B_{i+1} -> B_i` that verify and
  close (the checks: parity, primality, closeness, congruence, smoothness, ...).

Any long-range side-arrow is a COMPOSITE of adjacent generators, not its own
formula: right-side `f_{i,j}=f_{j-1}∘…∘f_i`, left-side `r_{j,i}=v_i∘…∘v_{j-1}`.
So a tower of n boxes needs 2(n-1) adjacent maps, not n^2 wired arrows.

**Why:** closure is the round trip. Descent is routing; the real content is in
the ascending verification arm. `r∘f` closing on a box is that box closing on
the value (μ∘δ=id), and the first box whose down-then-up round trip verifies
short-circuits the rest (first-close-wins). Build only the descending arm and
there is nothing to close against; the ascending arm is half the structure, not
decoration.

**How to apply:** in every build (the GPU-native resident engine, each ported
factoring/number-theory arm, any membrane), implement the ascending and
descending side-arms as adjacent generators and let long rails be composites.
Watch the type seam: forward maps may act on `n` while reverse maps act on a
pair `(p,q)`, so `r∘f` type-checks only where the domains meet (the product
boundary); above that the forward/reverse pair are adjoint checks on either side
of the same box. Worked example: the factor membrane's seven arms (parity,
primality, Fermat close-semiprime, MPQS, QS, remainder folds, product boundary).

See [[factor_membrane_two_arm_tree]] and [[gpu_native_gmomonados]].
