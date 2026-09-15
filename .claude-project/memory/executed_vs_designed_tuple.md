---
name: executed_vs_designed_tuple
description: Two different measurements give a tuple — `imasm derive` RUNS a word and witnesses its control flow; an ob3ect's grounded_tuple READS a description against the axes. Their crystal addresses are never comparable
metadata:
  type: feedback
---

Two operations both produce a twelve-mark tuple and they are not the same thing:

- **Executed** — `imasm derive <word>` in mOMonadOS runs the structure word as a program and reports the
  structural witness of its control flow.
- **Designed** — an ob3ect's `grounded_tuple` (from `auto.py`'s grounding phase) reads the DESCRIPTION
  against the twelve axes, one mark at a time, with a `grounding_reasoning` string per mark saying why.

**Their crystal addresses are not comparable.** Measured 2026-08-22 on the same word ⊢∈≻⊤≺⊥⊞⋈∋⊡⊙⊣:
executed gives ⟨𐑛𐑶𐑾𐑹𐑐𐑘𐑲𐑠𐑻𐑫𐑳𐑭⟩ at 3,444,190; designed gives ⟨𐑦𐑥𐑑𐑹𐑐𐑧𐑔𐑝⊙𐑫𐑙𐑭⟩ at 15,112,406. Same word,
two numbers, both correct about different things.

**Why:** I compared the four promotion ob3ects by their grounded addresses (15112405/6), said Collatz
"lands on the same object as RH", and only caught it when a reproducer asserting those numbers against
the kernel failed on every row. The razor census compares EXECUTED words; the promotion family I was
reading compares DESIGNED tuples. Both are real measurements; setting their addresses side by side is
the error.

**How to apply:** when comparing ob3ects, say which level. Compare the parity slot (or any single mark)
across levels freely — that is what agreed for four of five promotions. Never compare addresses across
levels. When building a reproducer, assert the EXECUTED value, since that is what the kernel can check;
carry the designed value as a separate labelled column.

**The levels can disagree, and the disagreement is informative.** The Collatz promotion asked as a named
move ("the parity closes from nun to or'") executes to or' but on the fork 16404190, while its
description grounds to 𐑿 yew. The control flow closes; the reading does not. Asked as a structure ("the
halving and tripling arms fuse Frobenius-self-dual") both levels close. See
[[feedback_rigid_form_open_prompt]] — the failure was measuring the ask.

Related trap, same day and same shape: two NUMBERINGS of the one type space (kernel plain mixed-radix vs
crystal_navigator's boundary-cell factoring). Self-encode type = 16,840,174 executed-numbering =
6,734,591 boundary-numbering. See [[rh_critical_line]]. Both traps are guarded in the crystallized branch
`millennium_ovms`. [[law_one_of_each_thing]] [[law_check_before_claiming]]
