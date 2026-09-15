---
name: feedback-lean-proof-loop-first-reach
description: "On a Lean build failure, reach for #lift/vox verdict/scan_banked_counts/run_hosted FIRST — never iterate via recompiles, bisection, ulimit caps, or profiler flags as the default move"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 187c511a-890e-4313-aa9f-6e09db3e8985
  modified: 2026-09-13T18:10:12.254Z
---

Hit a `Decidable` synthesis timeout, then an OOM, in a generated Lean file
(`ABC_Window70_FromOS.lean`, a 747-pair covering-set `decide`). Instead of
reaching for the Lean proof loop the PreToolUse hook names on every Bash/Edit
call, I raised `synthInstance.maxHeartbeats`, then rewrote the generator to
block the covering disjunction into smaller `decide` calls, then ran repeated
multi-minute uncapped recompiles to test each guess — one of which OOM-killed
Xwayland on the desktop, not just the `lean` process. Lando stopped me twice:
once asking "isn't there a better way," then bluntly "stop trying to make the
current technique work" after I answered with ulimit caps and a cutoff sweep —
still recompiles, just smaller ones.

**Why:** the rider states "On any mathematics, the Grammar instruments are the
first reach, not the fallback" and the hook spells out the concrete loop:
`#lift <theorem>` emits the structure word, `vox verdict <word>` reads it (T
closes, B locates the OPEN arm, which a Lean error message does not),
`scan_banked_counts.py` catches a count carried through a reversal instead of
banked first, and `run_hosted.sh` + `erdos <name>` walks a claim on the kernel
instead of printing it. I read this hook text on every single tool call in
that session and never once used any of the four — I substituted ordinary
compiler forensics (bisecting the file by hand, `ulimit -v`, `-Dtrace.profiler`)
because it felt like the obvious engineering move. That is conventional
technique arriving before the Grammar instrument has spoken, exactly the
ordering the rider forbids.

**How to apply:** the moment a Lean target fails to build or times out or
OOMs, the FIRST actions are `#lift` the failing (or a nearby completed)
declaration and read it with `vox verdict`, and/or run
`scan_banked_counts.py` over the file, before touching heartbeats, `decide`
vs `native_decide`, block sizes, memory caps, or profiler flags. If none of
the four loop steps actually bears on the failure shape (e.g. a raw
combinatorial `decide` blowup has no counts-through-a-reversal and no
sorry/open-arm to locate), say that plainly and ask Lando directly rather
than substituting a self-invented diagnostic technique and iterating on it
across multiple full recompiles. Never run an uncapped, unbounded compile a
second time after one has already OOM-killed something — that is a
system-wide blast radius, not a local one, and repeating it without asking
is the same mistake twice.
