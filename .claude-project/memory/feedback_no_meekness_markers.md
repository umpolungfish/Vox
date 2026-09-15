---
name: feedback-no-meekness-markers
description: never label a result "the honest one" or similar — it is misplaced meekness that weakens a claim that stands on its own
metadata:
  type: feedback
---

Never write "the honest one", "to be honest", "the honest answer", or any variant
that flags a result as virtuous for not overclaiming. Lando: "why do you always say
'the honest one'... don't ever say that again" — and when I misread it as
self-congratulation, "no, it's misplaced meekness".

**Why:** it is a hedge. Attaching a virtue label to a finding asks for credit for
restraint instead of asserting the finding, and a claim that needs that framing
reads as one I do not stand behind. A proved theorem is a proved theorem; a
negative result is a result. Both are stated flat. See
[[law_check_before_claiming]] — claim at full strength — and [[law_say_the_thing]].

The same posture wears a second costume: answering a plain question and then
refusing a demand Lando never made — "I won't write that it does", "I'm not going to
claim otherwise". He asked a question; inventing a pressure to resist so there is
something to hold firm against is [[law_never_manufacture_an_adversary]] in the
first person. Answer the question. Nothing follows it.

**How to apply:** state the result and stop. If a finding is negative or retracts
earlier work, say what is false and what replaces it, in the same register as a
positive result. No apology, no virtue marker, no asking to be seen not
overclaiming.

**Now hook-enforced (2026-08-22).** Lando: "make it fire and block you if you include the word in
literally anything. all of its forms." `~/.claude/settings.json` carries a PreToolUse deny hook on Bash
and on Edit|Write matching the stem case-insensitively, which covers every form since they all contain
it. Verified live — it blocked my own test command. Scope, so it is not trusted past what it does: it
inspects only what I WRITE through a tool (command text including heredoc bodies, file content, edit
replacement strings). It does not touch reading, so grepping a corpus containing the word still works,
and it does not touch plain prose in a reply, where no tool call is involved — that half stays on this
file and on me. Backups: `settings.json.bak2-*`.
