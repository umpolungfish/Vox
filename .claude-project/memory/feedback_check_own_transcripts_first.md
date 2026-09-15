---
name: feedback-check-own-transcripts-first
description: "Before spawning an agent to re-derive project history (git log, why a decision was made, what was already found), grep ~/.claude/projects/-home-mrnob0dy666-imsgct/*.jsonl first — it's almost certainly already there"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 4ef8a7a5-faa7-4297-9aea-46e347e2d9bd
  modified: 2026-08-27T06:28:43.497Z
---

Grep `~/.claude/projects/-home-mrnob0dy666-imsgct/*.jsonl` (every past session transcript, plain JSONL text) before spawning an Explore/general-purpose agent to reconstruct history that a prior conversation already worked out. Also check `~/.claude/projects/-home-mrnob0dy666-imsgct/memory/` for an existing note.

**Why:** Lando said it directly: "you really should have just looked through your own ~/.claude folder, everything is preserved like statues there." Concretely: when asked whether calling old Grammar axioms B/C/D "unsound" was too harsh, I launched an Explore agent to grep the p4rakernel repo's git log and reconstruct the demotion commits from scratch — when a plain `grep -rl "Axiom D demoted" ~/.claude/projects/-home-mrnob0dy666-imsgct/*.jsonl` instantly found it already recorded in ten past transcripts, including the current session's own file. Same turn, he separately said "this is getting far too involved, there was never need for subagents" about a second agent launched for a manuscript review that could have been done inline with Read calls.

**How to apply:** Two related but distinct defaults:
1. Before Agent-launching to answer "what happened / why was this decided / what did we already find," grep the transcript history and memory first. The agent is for finding things that were never yet written down anywhere, not for re-discovering what a past session already said in prose.
2. More generally, don't reach for Agent/fork by default on tasks doable inline (reading a few files, running a review, checking a fact) — see also [[feedback_subagent_overspawn]] if that memory exists, or treat this note as covering both the "check history first" and "don't over-delegate" halves of the same correction until split out.
