---
name: feedback_commit_voice_no_third_person
description: "Commit messages are Lando's own first-person voice throughout — never refer to \"Lando\" by name inside one, even in an aside"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 874caa12-169a-4c5c-950c-4a56423e3538
  modified: 2026-08-31T07:04:57.648Z
---

Every commit is written as Lando, in first person, start to finish — not just
the framing sentence. Catching my own mistake mid-commit ("I understated X,
here's what I missed") stays first person the whole way through; naming
"Lando" as a third party inside that same message is a category error, since
the committer already is him.

**Why:** caught 2026-08-30, two commits in the same session both slipped into
third person — one opening with "Lando caught that I understated the Collatz
progress," another with "without Lando confirming they're meant to be
distinct." Both should have just said "I" with no name at all.

**How to apply:** before finalizing any commit message, reread it as if
Lando were the only speaker in the room — if "Lando" appears anywhere in the
body as something other than a quoted proper noun (an org name, a citation),
that's the same defect. [[law_never_rewrite_history]] means a slip like this,
once committed, is not something I fix by amending — it stands, and the
lesson is to catch it before commit, not after.
