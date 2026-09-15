---
name: feedback_review_other_agents_work
description: "Lando wants me to personally verify other agents' and models' work rather than trust their own status claims"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 874caa12-169a-4c5c-950c-4a56423e3538
  modified: 2026-08-31T05:52:04.578Z
---

Lando trusts my verification over other agents' output — when he points me at
work produced by another agent, model, or pipeline (ob3ect artifacts, kilo/
openrouter/deepseek-designed objects, explore-agent summaries, Lean files
written in an earlier or different session), the job is to independently check
it myself, not relay its own claims about itself.

**Why:** demonstrated directly on the Erdős witness review 2026-08-30 — a
grep-only pass would have missed everything real. Reading the actual file
content caught a header claiming PROVED/DISPROVED with real `sorry`s inside;
`lake build` on the registered targets caught which of those `sorry`s were
real versus prose mentioning the word; and a direct standalone build caught a
file importing a Mathlib module that does not exist in the pinned version,
independent of its own zero-sorry grep count. None of that surfaces from
trusting the file's own header or a text search — [[law_check_before_claiming]]
is exactly the discipline this rewards.

**How to apply:** when reviewing prior work — mine or anyone else's — reach
for the real instrument (`lake build`, running the file, reading the full
source) before reporting a status. Don't delegate this verification to a
fresh subagent either; [[feedback_check_own_transcripts_first]] already
established Lando wants less delegation for things checkable in-session, and
this extends that to reviewing other agents' deliverables specifically. State
what the instrument said and what it was handed, then claim at full strength.
