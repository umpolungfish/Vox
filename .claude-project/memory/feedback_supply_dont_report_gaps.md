---
name: feedback_supply_dont_report_gaps
description: "When a derivation or document is missing a piece, go supply the piece myself rather than just report the gap"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 874caa12-169a-4c5c-950c-4a56423e3538
  modified: 2026-08-31T07:54:02.359Z
---

Finding a gap or a broken piece in a document is not the finish line. Lando's
instruction, verbatim: "if something is missing, don't tell me, just make it
your job to supply it." Reporting "X is circular" or "Y doesn't derive Z" and
stopping there is half the job — the other half is going and fixing X, or
deriving Z properly, in the same pass.

**Why:** said directly 2026-08-30 after I flagged three broken derivations in
`QM_effects_via_FDE_and_tensor_filtration.md` (a wrong Bell-CHSH formula, a
Born-rule calculation fixed at 1/2, an uncertainty-principle coefficient
chosen to match the known answer) and asked whether to keep auditing rather
than just fixing them.

**How to apply:** this does not license inventing a fix that is just as
unfounded as what it replaces — [[law_check_before_claiming]] and the
two-seals discipline in ig-docs/the_two_seals.md (never dress an unchecked
claim as a derivation) still bind. Where a real fix exists — reachable from
material already in the document or the codebase, or from established
outside mathematics — supply it and say so plainly. Where the real content is
a boundary (a constant that cannot be derived from the structure at hand, the
way π or ℏ enter from outside a purely combinatorial lattice), supply THAT as
the finding — state the boundary precisely, at full strength, rather than
reporting an open gap and waiting to be told what to do with it.
