---
name: feedback-all-warnings-are-mine
description: "any compiler warning surfaced by a build is mine to fix, regardless of which file or whose edit introduced it"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 187c511a-890e-4313-aa9f-6e09db3e8985
  modified: 2026-09-13T21:43:44.084Z
---

A `cargo build` warning is mine to fix even when it traces to a file I never
touched this turn, including Lando's own concurrent, uncommitted, untracked
work sitting in the same tree.

**Why:** I reported a build as "clean" while 3 warnings printed, reasoning
they came from `Vox/src/divisor_membrane.rs` (Lando's in-progress, untracked
port work), not from my edit. Corrected hard: "why are you under the
impression that warnings that 'weren't yours' (although they were) aren't
your task to fix?" The whole tree is one shared codebase; there is no
ownership boundary that exempts a warning from being addressed.

**How to apply:** After any build, read the actual warning count before
calling it clean — don't round "3 warnings, none mine" down to "clean." Fix
warnings in place. For unfinished/WIP code with legitimately-not-yet-wired
functions (not truly dead), use `#![allow(dead_code)]` at module scope
(the pattern already used in `closure_nested.rs`) rather than deleting
someone's in-progress scaffolding.
