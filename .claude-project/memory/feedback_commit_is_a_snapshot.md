---
name: feedback-commit-is-a-snapshot
description: "a commit takes everything sitting in the tree, never a subset carved out by who or what touched each file"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: b7fcac2b-0124-4019-8686-5e1d7369edb0
  modified: 2026-09-10T14:01:08.579Z
---

Lando does not distinguish "files I edited this turn" from "other in-flight
changes sitting in the working tree" when committing. A commit is a snapshot
of the tree at that moment, full stop.

**Why:** I staged only the two files I had personally edited
(`src/gpu_kernel.rs`, `src/repl.rs`) and left two other modified files
(`factor_relation_check.py`, `measurements/factor_relation_controls.log`)
uncommitted, reasoning that they belonged to someone else's in-flight work
and shouldn't be misattributed. Lando: "i do not make such distinctions, a
commit is a snapshot." There is no ownership boundary to protect; whatever
is sitting modified belongs in the next commit.

**How to apply:** before committing, `git add -A` (or otherwise stage
everything shown modified in `git status`), not a hand-picked list matching
only the current task. Write the commit message to cover the whole diff
actually staged, not just the piece I was asked to do. This does not
override [[law_never_rewrite_history]] or the no-push rule; it only means
don't leave unrelated-looking hunks stranded uncommitted out of a
misplaced sense of attribution.
