---
name: what-an-ob3ect-is
description: "An ob3ect is a structured IMASM imscription artifact built by ob3ect/auto.py — never an LLM agent call, never the ig-agent MCP tool. There is NO fixed phase count."
metadata: 
  node_type: memory
  type: reference
  originSessionId: e808a460-7d8e-4969-9d96-44a24695b03e
  modified: 2026-08-13T22:26:15.733Z
---

An **ob3ect** is a structured artifact: a natural-language description driven
through the IMASM design pipeline in one call, emitted as JSON with per-stage
validations and a Frobenius gate that must pass. There is NO fixed phase count —
do not call it "8-phase" (that was a stale claim from auto.py's own docstring;
Lando corrected it 2026-08-21). The actual run prints Phase -1 (gated grounding)
through Phase 12 (ROTAT orbit audit): grounding, domain charter, opcode map,
Frobenius split/fuse, registers, bootstrap lanes, ExOS/m⊙², entropy, Lean
verification (the real gate — ELABORATED), topology, banked count, SIXTEEN_3
trilattice, ROTAT audit, and μ∘δ=id. It is the unit of work here — "turn a
decision into an ob3ect" means *imscribe the decision*, not ask a model for an
opinion and not ask Lando.

Built by `ob3ect/auto.py` (the Ob3ect Auto-Design Pipeline, `ob3ect/core.py`
holds `Ob3ectArtifact`, `DomainCharter`, `OpcodeMap`, `EntropyAudit`).

```
python3 ob3ect/auto.py "<description>" [--name NAME] [--desc-file PATH]
        [--domain TYPE] [--scope local|mesoscale|maximal]
        [--provider openrouter|deepseek|local] [--model ID]
        [--entry name1,name2] [--context PATH] [--retries inf]
        [--zoom-to TARGET] [--churn --windings N]
```

Structure: `phase_0` domain charter (domain_type, scope, surface_tokens,
boundary_condition), `phase_1` the opcode map — every one of the twelve marks
assigned a chosen_element with a justification and rejected_candidates — then
split/fuse report, register mapping, bootstrap sequence, ExOS spec, entropy
audit. `BOOTSTRAP_STEPS = {1:IMSCRIB, 2:AREV, 3:FSPLIT, 4:AFWD, 5:FFUSE,
6:CLINK, 7:IFIX, 8:IMSCRIB}`. Retries on JSON parse failure or Frobenius FAIL,
by default until success.

**The description is TWO SENTENCES MAX**, saying exactly what is wanted and
nothing else. Everything the pipeline needs to know — inventories, structure,
surrounding facts — goes in `--context`, never in the description. A long
description does not imscribe the object harder; it dilutes the question, and
phase_0 compresses it to a handful of surface tokens anyway, so the detail is
lost AND the ask is blunted. Corrected 2026-08-14 after I passed a
thirty-line machine description as the description and got surface_tokens back
that had dropped the whole operation inventory.

Use `--desc-file` whenever the description carries LaTeX, `$`, backticks or
braces — the shell eats them before the pipeline sees them. Use `--name` when
the description is too long or punctuated to double as an identifier.

**The provider is the loopback 27B, and it is the DEFAULT — name no provider and
no model.** `_PROVIDER_CHAIN` is `["llamacpp", "local", "openrouter", "deepseek"]`
and auto.py's own comment says sending an eight-phase design to a metered endpoint
by default is how a routing default became a credit balance. Lando does not use an
API on the local at all (2026-08-19). `--retries` already defaults to inf, so it
does not need passing either. `IG_PROVIDER` promotes a provider to the front;
naming one with `--provider` makes it the ONLY lane.

Finished ob3ects land as `*_ob3ect.json` (repo root, `mOMonadOS/ob3ects/`,
`MoDoT/ob3ects/`) and carry `is_valid_ob3ect`.

The `mcp__ig-agent__run_agent` MCP tool is **not** this. That is a
THINK→ACT→OBSERVE→UPDATE agent loop on OpenRouter. Do not offer it when an
ob3ect is called for. See [[law_operations]], [[law_no_cant]].
