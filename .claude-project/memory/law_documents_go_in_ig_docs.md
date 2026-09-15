---
name: law_documents_go_in_ig_docs
description: Documents and manuscripts go in ~/imsgct/ig-docs — never a new directory under a working repo
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 1b08180f-622d-4e7d-ac91-3f9cb7c6a7fe
  modified: 2026-08-16T23:51:25.564Z
---

Every document — manuscripts, their PDFs, findings written up as prose — goes in
`/home/mrnob0dy666/imsgct/ig-docs/`, flat. Never create a `manuscripts/`
subdirectory, and never put a paper under a working repo like `red-hot_rebis`.

**Why:** ig-docs is the one place documents live, and its own repo. A paper
written into a working repo's doc folder is a second location for the same kind of
thing — the drift [[law_one_of_each_thing]] names — and it is not where Lando
looks.

**How to apply:** write the .tex and build the PDF directly in ig-docs, commit
both there. If a document was created elsewhere, move it and delete the directory
that was invented for it. Working notes tied to a repo's own data (a sweep table
and its reading) stay with that repo; a publication does not.

Related: [[law_publication]], [[law_no_artifacts]], [[law_no_tmp]].
