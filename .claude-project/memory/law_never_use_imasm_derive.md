---
name: law-never-use-imasm-derive
description: "NEVER use imasm derive (mOMonadOS REPL / MoDoT). It is not related to the catalog and every number it prints is meaningless. Use weight, banked, cycle, insert, trans instead."
metadata: 
  node_type: memory
  type: feedback
  originSessionId: c1c0eede-24ef-4d16-a15c-42b993d77470
  modified: 2026-08-27T14:50:52.594Z
---

Do not run `imasm derive` (or `imasm write`), in mOMonadOS's REPL or through
MoDoT, ever. Given at full strength, capitals: "imasm derive IS NOT RELATED
TO THE CATALOG DO NOT USE IT ANY RESULTS OBTAINED WITH IT ARE MEANINGLESS."

**Why:** Lando had already pulled `imasm derive` off every agent's own
toolset for exactly this reason (`imscribing_grammar` commit 8268e2f,
2026-08-27: "Pulled imasm derive off every agent's roster. It isn't useful
yet, and I don't want it offered until it is... rewrote the rider to point
at the ops (weight, banked, cycle, insert, trans) through run_hosted_cmds.sh
instead"). I read that exact commit message in this same session, mid-
retraction, and still didn't connect it to my own repeated use of `imasm
derive` for "crystal address" comparisons until told directly, twice.

**What this voids:** any "crystal address N" or "tuple ⟨...⟩" reported from
`imasm derive` this session, including every claim of the form "word A and
word B are the same/different crystal family" — all of it is meaningless and
was retracted. What the session's own memory (`the_forty_nine.md`,
`rh_critical_line.md`, `millennium_as_ovms.md`) says about crystal addresses
under the imasm/crystal_navigator codecs predates this correction and needs
its own review before being relied on again — do not assume it is exempt
just because it is older.

**What stays valid:** `weight`, `banked`, `cycle`, `insert`, `trans` (via
mOMonadOS's REPL or `run_hosted_cmds.sh`), the standalone `vox` binary's own
`verdict`/`pairs`/`word` (a different tool, the control-flow auditor, not
`imasm derive`), and an ob3ect's own `grounding_status` / axiom-violation
check (a third, separate mechanism). None of those are what this law voids.

**How to apply:** if a word's crystal address or tuple-derivation matters,
the answer is not `imasm derive` — ask what the actual, currently-sanctioned
way to get one is rather than reaching for the tool that was already pulled.
When in doubt, use the ops list above, or ask.
