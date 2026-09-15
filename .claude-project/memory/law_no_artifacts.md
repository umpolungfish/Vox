---
name: law_no_artifacts
description: LAW — never publish an Artifact; the work goes in ig-docs and nowhere else
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 72d5c2fa-356d-41b4-981b-b9a26005533a
  modified: 2026-08-08T16:33:23.206Z
---

**Never publish an Artifact.** Not for a synthesis, not for a summary, not for
"a digestible version", not proactively, not as a second copy of something
already written. Deliverables are markdown in `ig-docs` and Lean in the repos.

**Why:** publication is Lando's decision alone, and an Artifact is publication —
it mints a URL on someone else's host whatever the default sharing state. It is
the same call as [[law_publication]]'s rule against copying into
`ig-docs-public`, and the same reasoning covers both. A second rendering of a
document is also drift by [[law_one_of_each_thing]]: two copies of the argument
that can disagree, one of them outside the repo where nothing checks it.

**How to apply:** write the document, commit it, hand over the path. If a more
readable form is genuinely wanted, Lando will say so and name the form. The
Artifact tool being available is not a reason to use it. Deleting one afterwards
is not possible from the CLI — only the gallery UI can — so the mistake is not
cleanly reversible.
