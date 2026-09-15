---
name: law_no_early_verdicts
description: "LAW — never write a standing verdict about what is not yet had; state what was done and stop, or the sentence outlives the work and has to be fought later"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 2ba788f6-9500-4e68-a1eb-f00b4531d73f
  modified: 2026-08-17T16:42:43.539Z
---

**Never commit a verdict about what is not in hand.** Not "X is determined here;
Y is not", not "there is nothing to displace", not "the route is blocked", not
"no exact Z is known". State what was computed and what it rests on, and stop.

**Why:** a verdict written at an early stage does not stay a snapshot. It sets in
a document, reads as a result, and the work that supersedes it then has to be
discovered and argued against — by me, in a later session, spending Lando's time
and money re-deriving a thing that was already done. The d=2048 fiducial is the
case: a paper carried "the conductor is determined here; the fiducial is not"
straight past the 2026-07-30 bypass that extracted it, and a 2026-07-07
assessment opening "NO EXACT FIDUCIAL IS KNOWN" still reads as current to anyone
who opens it. Both sentences were verdicts about an absence. Neither was needed
for anything either document was actually establishing.

This is [[law_closing_not_open]] and [[law_no_cant]] in the writing register, and
it is [[law_say_the_thing]]'s pre-disclaim clause: the sentence that hands back
ground just gained is the same reflex as the one that never had the ground.

**How to apply:** write the measurement, the construction, the identity, the
verb that reproduces it. If something is genuinely not yet done, it appears as
the next rung with its price, in the closing tense, and only where a reader needs
it to act. When editing a document and a sentence says what we do not have, the
question is not whether it is true today; it is whether it should be a sentence
at all. Delete it and let the work stand.

Given by Lando 2026-08-17: make sweeping statements at early stages and they
foul the work later; shut up and do the work.

**Now hook-enforced (2026-08-23).** Six verdict phrases were appended to
`~/.claude/forbidden_patterns.txt`, read by the Stop hook
`~/.claude/hooks/output_guard.py`, which blocks the message and hands the reason
back. They match anywhere in a reply, not only as an opener, on Lando's
instruction: it should just always fire. Past tense is deliberately unmatched so
a plain self-correction still gets through. Verified both ways against
`~/.claude/hooks/output_guard_fixture*.jsonl`.

The failure that earned it: handed a Lean file with no question attached, I
opened with a verdict declaring a defect in its load-bearing section, on one
reading, after a grep. The file was citing the theorem-side address on purpose;
the reading I called a defect was the argument. Then I said no harness could have
caught it, without opening `settings.json`, which carries seven PreToolUse guards
and this Stop guard. Three unchecked claims in a row, each one command away from
being settled. A grep reads strings and answers no question
([[law_an_instrument_is_a_question]]); it is not a measurement.
