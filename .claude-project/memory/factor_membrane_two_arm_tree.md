---
name: factor_membrane_two_arm_tree
description: "factor_membrane recurse is the μ∘δ two-arm tree; δ splits even/odd bit lanes, μ (word multiply) closes on the value, both branches descend to primes"
metadata: 
  node_type: memory
  type: project
  originSessionId: 37fe5b8e-93c6-4b79-b8b4-9a08c8c81595
  modified: 2026-09-14T11:15:09.777Z
---

`factor_membrane recurse <N>` builds a two-arm co-evolution tree, not a
single-spine dyadic walk. Two distinct fusions are in play and must not be
confused:

- δ (dyadic split, deinterlace_word): δ(V) = (even-cell bit lane, odd-cell bit
  lane). p = Σ b_{2j} 2^j, q = Σ b_{2j+1} 2^j. Γ (interlace_words) is its exact
  inverse, so Γ∘δ = id always on a canonical encode word. The lanes almost never
  satisfy p·q = V; that is expected, they are representation descendants.
- μ (native word multiply): the arithmetic fusion. The real factor pair is where
  μ(p,q) = p·q = V and syzygy_preserves holds. `syzygy_preserves` reduces to
  μ closing plus the Γ/Λ codec commuting (automatic for canonical words).

recurse now: at each node, print the δ seed with Γ∘δ=id verified, co-evolve the
two arms to the μ-fixed point via `propagate_word_states` (Hensel low arm coupled
to range-prune high arm through the μ-bridge, lifting p and q together), then
descend BOTH children. Leaves are the prime factorization; the tree is the exact
multiscale ancestry. 360 → full 2·2·2·3·3·5 tree; 15959 seeds at δ=(111,113) and
is a single PRIME leaf (no false 111×113 claim); 100160063 → 10007 × 10009.

The earlier recurse walked only the even lane and called the split
"representation-only" (discarding the odd sibling). That was the wrong object:
follow both branches under the μ relation. Committed eb71bba.

Also in that line: `membrane_family run` self-bounds on the band (a up to N/2)
with no step cap; the old max_steps=200000 ceiling was removed. See
[[nested_word_framing_only]].
