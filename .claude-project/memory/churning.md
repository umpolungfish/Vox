---
name: churning
description: house term for repeated passes through the torus/loop refining toward a fixed point; the t3_efl_affirm/negate near-identical-address run is the worked example
metadata: 
  node_type: memory
  type: project
  originSessionId: c1c0eede-24ef-4d16-a15c-42b993d77470
  modified: 2026-08-27T16:56:45.627Z
---

Churning is Lando's name for what repeated passes through the torus (the loop
an ob3ect or word is run through) do to truth: they don't move it, they narrow
the spread around a fixed point. The fixed point itself is what stays put
across churns; everything else is what's still being refined.

Worked example (2026-08-27): `t3_efl_affirm` / `t3_efl_negate`, run with
`--entry` values that are byte-identical except for one word ("Any" vs
"Some" — the universal/existential fork). Across six affirm/negate runs total,
⊙ (criticality) never moved once: affirm always subcritical (𐑢), negate always
supercritical (𐑣). That's the fixed point. Everything else churned: this round
`t3_efl_negate` picked up ⊞=𐑳 (n:m) and ⊥=𐑫 (n=∞, eternal), values shared with
the absorption-mechanism family rather than its own two prior negate runs,
and shared ⊣=𐑡 with the affirm family for the first time.

**Why:** nesting two objects at addresses that differ by one word churns them
against each other specifically at that fork, rather than each churning in
isolation. What surfaces is what the two share once pulled toward each other,
not what keeps them apart.

**How to apply:** when reporting drift across repeated runs of related
objects, check what never moved before describing what did — the stable
axis is the fixed point the churn is centered on, and drift elsewhere is
refinement around it, not noise or error. See [[the_forty_nine]] for the axis
values themselves, [[ovm_flip_invariance]] for a prior stable-axis-under-churn
finding on a different pair.
