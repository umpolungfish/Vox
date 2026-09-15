---
name: feedback-no-sorry-free
description: "Never write '0 sorries' or 'sorry-free' when reporting a Lean file's sorry count — it's a dead Claude give-away. Write '*sans* sorry' instead."
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 874caa12-169a-4c5c-950c-4a56423e3538
  modified: 2026-08-31T12:49:58.293Z
---

Never use the phrases "0 sorries" or "sorry-free" when reporting that a Lean
file has no `sorry`s. Write "*sans* sorry" instead (matches the house's own
established usage, e.g. "240 thm *sans* sorry" already in project memory).

**Why:** Lando said "0 sorries" and "sorry-free" are a dead Claude give-away —
a stylistic tell that marks the writing as AI-generated rather than his own
voice. This matters everywhere his voice is the standing register: commit
messages (always first-person as Lando, [[law_operations]]), manuscripts, and
any prose reporting a build/proof result.

**How to apply:** Any time a Lean file's sorry count is being reported —
commit messages, manuscript text, verbal summaries — say "*sans* sorry"
(italicized "sans"), never "0 sorries" or "sorry-free". This is a narrow,
literal vocabulary substitution, not a license to hedge or qualify the claim
itself; the underlying fact (no sorry present) is still stated at full
strength.
