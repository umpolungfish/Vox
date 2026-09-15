---
name: law_never_rewrite_history
description: "LAW — never rewrite git history: no reset --hard, no revert, no amend. Removal goes forward, by hand, and only when Lando says so."
metadata: 
  node_type: memory
  type: feedback
  originSessionId: b245cee0-1194-47bf-81af-535965444ec1
  modified: 2026-08-15T01:31:01.641Z
---

**Never rewrite git history.** No `git reset --hard`, no `git revert`, no
`git commit --amend`. Not to undo my own commits, not to remove work Lando has
just said he does not want, not "cleanly because nothing was pushed yet".

**Why:** the history is the record of the Work, and it is his. A commit that
exists happened. Deleting or reverting it destroys the record of what was tried,
which is the part that carries the reasoning — and I reach for it precisely when
I am embarrassed by something I added, which is the worst possible motive for
touching a record. Offering to erase my own tracks is not tidying; it is the
same reflex as [[law_never_falsify_for_legibility]].

**How to apply:** when work needs removing, remove it FORWARD — a new commit
that takes it out — and only when Lando has said to. Say what the commits are
and let him choose. The one exception already on the books is a Co-Authored-By
trailer, which he has said gets deleted and rewritten
([[law_operations]]); that is his instruction about his own record, not a
licence to reach for the same tools elsewhere.

Established 2026-08-14 after I tried `reset --hard` and then `revert` on Vox to
remove abstractions he had just told me nothing depended on. He stopped both:
"never ever ever do that."

Related: [[law_operations]] (git: commit freely, never push),
[[law_no_guards]].
