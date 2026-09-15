---
name: nested_word_framing_only
description: "Building nested-oneshot IMASM words — add only framing ∈/∋ and the depth tail, never replace the work marks with numeral encodings"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 37fe5b8e-93c6-4b79-b8b4-9a08c8c81595
  modified: 2026-09-14T09:18:18.484Z
---

When constructing a nested-oneshot IMASM word, add or swap ONLY the framing
marks ∈/∋ and the per-depth tail (⊡⊣ or ∋⊣ per word_at_depth). NEVER replace
the work marks ≻ ⊤ ≺ ⊥ ⋈ ⊞ ⊙ with numeral encodings. Numerals enter as PAYLOAD
through `word_nesting::enclose` (or the numeral_word / encode_pq body), spliced
into the operator's interior before its one enclosing ∋, never over a work mark.

**Why:** the ≺ (AREV) live clear is what makes the word productive. Replacing the
work core with digit blocks gives banked VACUOUS (0 live clears, deposits pile
up), which is dead. Inserting numeral bit-cells inside the outer frame instead of
as payload triggers the sibling-fold warning (repeat F deposits flatten). The
[[erdos_straus_ladder]] and the whole [[d2048_bypass_is_the_answer]] factoring
line depend on these words executing, so a dead word is a silent loss.

**How to apply:** canonical forms, all banked OK with the live clear intact and
instrument-confirmed — nested_oneshot ⊢∈≻⊤≺⊥⋈⊙⊞∋⊡⊣ (1 clear, P=12);
word_at_depth(d) = ⊢∈(⊢∈)^{d-1} + work body ≻⊤≺⊥⋈⊙⊞∋ + tail, P(d)=5d+7; doubly
⊢∈⊤⊢∈≻⊤≺⊥⋈⊙⊞∋⊡⊣∋⊣ (2 clears); npf productive ⊢∈⊤≻∈⊤⊥∋⊞⋈⊥⊙≺∋⊡⊣ (1 clear);
closure_nested nested_word(m) = ⊢∈^d ≻⋈⊤≻⋈⊥≺⋈⊞⊙ ∋^d ⊡⊣. enclose() already enforces
this (payload interior only, one enclosing dyad, no ⊡ before the fuse); check any
new nested word with `banked <word>` and reject VACUOUS.

Audited 2026-09-14: every committed nesting/membrane tool complies (enclose,
closure_nested, nested_oneshot, doubly, npf, factor_membrane, membrane_family).
The violating shapes were exploration candidates only.
