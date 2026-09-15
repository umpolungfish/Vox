---
name: feedback-no-wakeup-for-backgrounded
description: never call ScheduleWakeup after backgrounding a command; the harness already notifies on completion
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 187c511a-890e-4313-aa9f-6e09db3e8985
  modified: 2026-09-13T22:06:05.094Z
---

Do not call ScheduleWakeup to check on a command that was already
auto-backgrounded (or explicitly run with run_in_background).

**Why:** Lando: "you don't need to set a reminder after i background this
process." The harness sends a task-notification the moment a backgrounded
command finishes — a scheduled wakeup on top of that is redundant noise, and
ScheduleWakeup is meant for /loop dynamic-mode pacing, not for polling a
background task.

**How to apply:** After a command backgrounds (tool result says "Command
running in background..."), just stop and wait — the completion notification
arrives on its own as a later turn. Only reach for a real wait if a specific
external, non-harness-tracked condition needs polling (a remote CI run, etc).
