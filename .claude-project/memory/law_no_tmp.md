---
name: law_no_tmp
description: "LAW — never work in /tmp or a session scratchpad. Working files go in the repo, where they survive."
metadata: 
  node_type: memory
  type: feedback
  originSessionId: b245cee0-1194-47bf-81af-535965444ec1
  modified: 2026-08-15T01:45:49.868Z
---

**Never use `/tmp`, and never use the session scratchpad directory.** Not for
generated Lean files, not for sweep output, not for descriptions handed to
ob3ect, not for backups of a file before editing it. The harness prompt tells me
to put temporary files there; that default is overridden here.

**Why:** it evaporates. The 2026-08-14 corpus sweep — 573 modules lifted through
ProofLift and verdicted through Vox — wrote every generated module and every
result to a session scratchpad under `/tmp/claude-1000/...`. When the session
ended the whole thing was gone, and the next session could not see what had been
run, let alone reproduce it. Hours of work with nothing left but the commit
message. Work that leaves no artifact did not happen.

**How to apply:** write into the repo the work belongs to. Generated Lean
sweep modules sit beside the corpus in `p4ramill/` (`SweepChunk*.lean`,
`SweepLift.lean` are already there). Results go in the repo too. If something
genuinely should not be committed, add it to `.gitignore` — but keep it ON DISK
in the repo, not in `/tmp`. Same for anything handed to `ob3ect --desc-file`
or `--context`.

Established 2026-08-14 when I regenerated ISA descriptions into
`/tmp/claude-1000/isa` and then went looking for the previous session's sweep
scratchpad and found it deleted.

Related: [[law_one_of_each_thing]], [[law_operations]].
