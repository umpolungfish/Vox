---
name: grammar_native_trainset
description: The workstream to finetune a model saturated in the Grammar; the four-valued register is the holder, opened first and held through
metadata:
  type: project
---

Compiling a training set to finetune a model that reasons natively in the Grammar (four-valued,
never collapsing to boolean). Started 2026-08-21. Files in `~/imsgct/grammar_trainset/`
(`gen_oracle.py`, `oracle_sample.jsonl`, `NAVIGATION.md`).

**The kernel is the oracle.** Most of the Grammar is a deterministic function, so the training
data can be VERIFIED not just curated — the highest-leverage, most unique property. `gen_oracle.py`
sweeps the word instruments (imasm derive/weight/banked/vox/insert/cycle) over a word corpus and
emits verified (prompt, completion) JSONL; every completion is the kernel's own output. Guardrail:
never a model's guess about a tuple (kernel is scripture, [[what_an_ob3ect_is]] / [[law_operations]]).

**The navigation finding (ob3ect grammar_native_model).** The naive imscription
`⊢≻∈⊤⊥⊞∋⋈⊙≺⊡⊣` LEAKS — verdict T but the reversal ≺ clears all four Belnap values (T,F,t,f) in
the open. `insert` repair: one ∈ before the ∋ fuse (`⊢≻∈⊤⊥⊞∈∋⋈⊙≺⊡⊣`) → the four bank in frames
(held), banked OK, and verdict flips T→**B** (holds the both). So: **the four-valued register is the
HOLDER — open it first and hold it through the mechanical tasks.** Compute inside a held four-valued
frame or the both leaks at the reversal and the model ends boolean-native (the exact failure to
avoid). This inverts the naive layer order: the Belnap-reasoning-trace layer is the enclosing held
frame, NOT a capstone bolted on last; the kernel-oracle/catalog/ob3ect data train INSIDE it.

Layers: (holder) four-valued reasoning traces; (inside) kernel-oracle pairs + catalog description↔tuple
+ ob3ect description→design; (saturation) Lean/Rust/docs/laws pretraining. Open decisions: JSONL format
(prompt/completion vs chat) pending base model; bias corpus toward structured words (⊤⊥⊞, ∈∋, ⊢…⊢) over
the vacuous common case; add imasm write / cycle / catalog extractors. See NAVIGATION.md.
