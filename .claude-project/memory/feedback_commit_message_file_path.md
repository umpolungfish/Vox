---
name: feedback-commit-message-file-path
description: git -C <repo> commit -F <relative path> reads the file inside <repo>, and ig-docs holds a stale tracked commit.txt that silently supplies the wrong message
metadata:
  type: feedback
---

`git -C <repo> commit -F <path>` resolves a **relative** `-F` path inside `<repo>`,
not the working directory. `~/imsgct/ig-docs/commit.txt` is a tracked leftover from
2026-08-17 whose first line is "The additive energy bound gets its own paper", so
`git -C ig-docs commit -F commit.txt` succeeds with that message instead of the
intended one — silently, since it does not fail.

**Why:** it has produced at least three mislabelled commits in ig-docs
(`b1df73c0`, `a8c0e1a7`, `b4ff1f3e`), and because the commit succeeds, an
`|| fallback` never fires and `git log -1` looks like someone else committed. I
wrongly blamed a concurrent session for it once; see [[law_check_before_claiming]]
and [[law_never_manufacture_an_adversary]].

**How to apply:** always pass `-F` as an absolute path, or write the message file
into the repo being committed. After any scripted commit, check `git log -1` shows
the message you wrote. History is never rewritten here
([[law_never_rewrite_history]]) — a wrong message gets corrected by a following
commit, so it is cheaper to get right the first time.
