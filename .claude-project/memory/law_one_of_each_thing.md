---
name: law_one_of_each_thing
description: LAW 4 — there is exactly one of each thing; a second copy is drift, and the copy I edited may not be the one being read
metadata:
  type: feedback
---

There is one Grammar, one canonical catalog, one source for any generated
artifact, one directory that is read. A second copy is drift, never a second
valid convention.

Two failures follow from breaking it, and both happened today:
- **I edited a file nobody was reading.** Corrections were reported as applied
  while the read location carried a build days old. When an edit has no visible
  effect, or a rendered artifact persists, FIRST find every copy and confirm
  which one is being viewed.
- **A divergent duplicate went unnoticed.** Two source trees held the same paper;
  only one was fixed.

**How to apply:** before claiming something is fixed, locate every copy of it and
check the one the user actually opens, by timestamp and by content. Fix the
generator and regenerate, never hand-patch the output. Cross-compare against the
canonical location before deleting or moving. Strip the old form rather than
adding a compatibility shim, freezing a branch first if the old state matters.
