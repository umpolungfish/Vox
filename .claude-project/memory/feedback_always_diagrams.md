---
name: feedback-always-diagrams
description: Never pass --no-diagram on an ob3ect/auto.py invocation — Lando always wants the diagram generated
metadata: 
  node_type: memory
  type: feedback
  originSessionId: c1c0eede-24ef-4d16-a15c-42b993d77470
  modified: 2026-08-27T14:29:17.771Z
---

Every `auto.py` command prepared or run should omit `--no-diagram`, full
stop. Corrected directly: "i always want diagrams, always."

**Why:** no exceptions given, no scoping by ob3ect type or purpose — this is
a blanket preference, not situational. Passing `--no-diagram` for
speed/verbosity reasons (as done routinely this session) was never asked
for and dropped output he wants by default.

**How to apply:** when preparing or running any `auto.py --desc-file ...`
(or churn/zoom) command, never include `--no-diagram`. If output length is a
concern for a terminal report, that's a separate question from whether the
diagram gets generated at all — generate it regardless.
