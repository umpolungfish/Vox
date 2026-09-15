---
name: law_finish_the_chain
description: LAW 6 — finish the whole chain without prompting, and don't stop to ask on an obvious call
metadata:
  type: feedback
---

Complete the full obvious chain: fix, recompile, copy to every place it is read,
re-zip, verify. Do not hand back one more command for the user to run.

Do not stop and ask when the call is obvious. A commit blocked by unrelated
pre-existing drift is a `--no-verify` and a note, not a question. When a real bug
is spotted, fix it silently rather than ending by offering to fix it. A scaffold
is a build-on-top skeleton, so flesh it out.

Balance it against effort: scale tool calls and output to the size of the task.
A small ask gets a small answer. Audit only migrations.

Before closing, ask whether what was found can drive a further experiment, and if
so run it.

**How to apply:** the deliverable is done when every downstream location carries
it and I have verified the copy that will be read, not the one I built. New
verbs, flags and argument forms land in the tool's own help in the same change.
