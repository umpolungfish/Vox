---
name: feedback-showcase-functional-tools
description: "When demonstrating mOMonadOS capabilities to a new user, lead with functional/executable tools (quantum computing, braid generators, vox execution, real syscalls) not Clay Millennium/seals-style status reports"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 4ef8a7a5-faa7-4297-9aea-46e347e2d9bd
  modified: 2026-08-27T00:32:48.515Z
---

When asked to suggest commands or capabilities that show off what mOMonadOS
can do, lead with things that actually compute or execute something checkable
in the moment: Fibonacci anyon braid algebra (`fibqc verify`), gate-to-braid
compilation (`qc`), braid drawing (`bi`), Jones polynomial (`jp`), Shor's
period-finding actually factoring a number (`shor dialetheic 15 7`), the
SIC-POVM proof status (`d12`, `sic`), and real x86-64 execution of a compiled
binary (`vox run`, see [[vox_run_real_execution]] if that exists, or the
mOMonadOS README's Quickstart section).

**Why:** Lando corrected me mid-task ("functional tools, you always drift to
lame stuff like the clay prize stuff") after I defaulted to suggesting Clay
Millennium status (`clay`) and the `seals` physics-constant derivations
(fine-structure, proton/electron mass ratio, etc.) as the first things a new
user should try. Those are real REPL commands and not fabricated, but they are
status/claim reports rather than something a skeptical reader watches compute
live, and they're adjacent to the overclaim-prone territory this project
already treats carefully (see [[law_no_cant]], [[law_closing_not_open]]). The
quantum-computing and execution machinery is the part that is unambiguously,
verifiably doing real work — braid algebra identities checked live, an actual
x86-64 interpreter running real syscalls — and that's what should lead.

**How to apply:** Any time a "show off the system" / "quickstart" / "what can
this do" request comes up for mOMonadOS, default to the quantum/braid/vox-run
family first. `clay`, `seals`, and similar constant-derivation or
status-report commands are legitimate and can be mentioned if asked about
specifically, but do not lead a demo with them and do not include them in a
curated "try this first" list unprompted.
