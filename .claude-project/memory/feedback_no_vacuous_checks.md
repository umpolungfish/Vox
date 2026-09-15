---
name: feedback-no-vacuous-checks
description: A check gated on a real condition can still be vacuous if its consequent has no reachable False path — find the real discriminant or drop it
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 04b0296d-f897-49b9-ba61-7601465021fd
  modified: 2026-09-09T01:48:48.559Z
---

Gating a check on a real, per-entry-varying condition is not enough. If the
check's own consequent has no code path that ever produces the opposite
verdict, it is still vacuous — the gating just hides that.

**Why:** in `Gate2_BalneologicalHeap.evaluate()`
([[sudovoynichese_checkpoint]]), `cold_process` fired whenever K=cold
maceration and was hardcoded `True`. Gating it on K alone looked like a
real condition (K varies per entry) but the literal `True` behind it never
had a `False` branch anywhere in the code. Same for `vessel_split = True`
in the volatile-exempt branch — a check key with a value that can never be
anything else. Lando's instruction after the first fix landed ("make sure
nothing is hardcoded") wasn't answered by the first pass, which fixed two
of three vacuous checks but left these two. Caught by direct execution
across thousands of runs and checking which check keys ever come back with
both values.

**How to apply:** after fixing a vacuous check, grep the surrounding gate
or validator for every other `= True`/`= False` literal assigned into a
checks/results dict, not just the one flagged. For each: either it has a
real, cited failure mode (keep it, argue why from the source spec, and
verify empirically that both outcomes occur across a real sample), or it
doesn't (drop it from the checks dict entirely rather than dressing an
inert always-pass up as a check — record it as a plain reason string if
it's worth noting at all). A key that is always `True` across a large
random sample is not automatically wrong, but it needs a citation for why
it's *supposed* to always hold, not just an absence of a counterexample in
the code.
