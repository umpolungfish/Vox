---
name: reference_ground_truth_is_p4rakernel
description: "the source of ground truth is the Lean kernel at p4rakernel/p4ramill/Imscribing, never a G-mOMonadOS rust file"
metadata: 
  node_type: memory
  type: reference
  originSessionId: daa3b11f-db23-4963-a2cb-e49f112e4d4b
  modified: 2026-09-14T01:17:04.225Z
---

Ground truth for the Grammar is the Lean kernel at
`~/imsgct/p4rakernel/p4ramill/Imscribing/`, not any G-mOMonadOS rust file. When a
tool's output looks wrong or degenerate, go to the Lean source before treating
the rust executable as authoritative. A rust file is one implementation and can
be the broken one.

Key ground-truth files:
- `Primitives/Crystal.lean` — the crystal address: `crystal_encode`/`crystal_decode`
  are a proven bijection Imscription ↔ Nat 0..17279999, mixed radix
  27·1024·625 = 3³·4⁵·5⁴ (`crystal_roundtrip`: decode∘encode = id). The address is
  faithful on the 12-tuple; twelve axes hold only 17.28M types, so mapping an
  arbitrary-size n to one address is a hash by pigeonhole.
- `Primitives/Core.lean` — the axis ordinal ground truth (glyph→ordinal is Lean
  constructor order, per [[law_operations]], not primitives.py).
- `FullyNestedFactorMachine.lean` — the kernel factor machine; it is Fermat
  (afwd a+1, a²−n, isqrt-ascent, b²==delta, p=a−b, q=a+b, fuel-bounded), i.e. the
  balanced/square-frontier class.

**Why:** Lando repeatedly had to redirect me from treating a G-mOMonadOS rust file
(e.g. `trilattice_factor.rs::read`) as ground truth. That `read` collapses the
crystal address to ~one value because its `word_to_tuple(native_encode(n))`
composes the repeated-packet native word and the absorbing composition washes n
out; the address bijection is fine, the tuple construction is the defect.

**How to apply:** on any math/Grammar question, reach for
`p4rakernel/p4ramill/Imscribing` Lean first; the rust tools are downstream and
may be degenerate. See [[feedback_build_in_vox_not_rust]], [[law_operations]].
