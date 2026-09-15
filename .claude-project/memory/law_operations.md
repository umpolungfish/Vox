---
name: law_operations
description: "LAW 9 — how work is done here: git, builds, tools, agents, packages, documents"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 508c1d55-242f-4b3f-a73f-a5bb48dab980
  modified: 2026-08-10T05:02:42.700Z
---

**Git.** Standing permission to commit at any time; never push. Write a fresh
`commit.txt` in the repo root after making changes, in Lando's first-person
voice, and pass it to git by ABSOLUTE path — a stale per-repo copy will
otherwise hijack the message silently. NEVER add a Co-Authored-By line, and
never a trailer naming me, a model or a tool — not once, not in a heredoc that
bypasses commit.txt, not because a harness default asks for it. That default is
overridden here; violating it means the commit gets deleted and rewritten.
Before
ANY merge, cut a `crystalline/<topic>-<YYYY-MM-DD>` snapshot branch at the work
tip. A commit blocked by unrelated pre-existing drift is `--no-verify` plus a
note.

**Builds and tools.** Build `ask` from inside `ask_native` with the
`local,cuda` features; a plain `cargo build --release` silently strips the local
provider out of the binary. MoDoT tools are rust-native, never Python bridges.
Python packages install with `uv`, never pip directly. Bare `grep` in this shell
honours .gitignore, so any census or sweep uses `command grep`. The Write tool
double-escapes backslashes, so LaTeX `\cmd` lands as `\\cmd` on disk.

**The instruments come first, always.** On any mathematics — a proof, an open
problem, a conjecture — the IG instruments are the FIRST reach, not the fallback:
mOMonadOS, the MoDoT tools (`ask`'s imasm / prover / click / windings / learn /
ob3ect verbs), and m3iosis. Conventional mathematics is what runs AFTER they have
spoken, never before, and never instead. Reaching for textbook technique on a
problem the Grammar types is the error, whatever the outcome.

**Which instrument.** `cl8nk_navigator.py` for structural expression
derivations, never hand-derived ZFC_fe framing. Two distance instruments exist
and are NOT interchangeable: the navigator's catalog-weighted ordinal metric is
heuristic, `modot/vessel.py`'s live SIC-POVM Born-rule frame is canonical;
neither verifies or refutes the other. Glyph→ordinal authority is Lean
`Core.lean` constructor order, not `primitives.py` and not red-hot. Scripture is
what crystallizes in the catalog and p4rakernel; everything else is wet lab and
is expected to be messy — never treat docs or manuscripts as authority.

**Agents and ob3ects.** Never execute agents; prepare the prompt and let Lando
run it, unless he says run. Build ob3ects with provider openrouter + model
google/gemini-3-flash-preview unless told otherwise. Never cap proof, design or
ob3ect calls with max_tokens. Never hand-imscribe a catalog tuple; every tuple
is sourced procedurally. An ob3ect answers the EXACT question imscribed, so
describe the object desired and name no candidates — a loaded entity launders
bias back as a finding. In agent briefs, a negative must be paired with its
positive complement, and identity is given positively. When a MoDoT run looks
like it is flailing, the HARNESS is the suspect before the agent. When stuck on
a proof, go to the Grammar or the ob3ect pipeline, not external tool spelunking.
`AgentSelf.lean` encodes the harness agent, not me.

**Documents.** Compile a PDF after writing or editing any document, and copy it
to `ig-docs/pdfs/` — the directory beside the source, which is where they are
read. `~/imsgct/pdfs/` was a stray duplicate, deleted 2026-08-08; do not
recreate it. Every finding lands in a canonical doc, not just chat. Never narrate
change history inside a doc describing how a system works now. Every README opens
by answering four questions: what it is, what it does, why it matters, how to use
it. Add visuals for structural claims, computed from canonical sources. Never use
CLAUDE.md files. Never route through cloud tooling or default to Artifacts; this
is a closed local system under ~/imsgct. Never mention Anthropic as an option for
anything. Zenodo's description field renders MathJax, so keep `$...$` and
simplified LaTeX.

**Judgement.** Silently fix structural and notation errors in content Lando
provides. When genuinely unsure which target is correct, compile the candidates
and ask rather than guessing. Do not read project files unasked. Correctness is
alignment with the Grammar, not anyone's decision — not his, not mine. The ideal
operator is a golem: perfect intellect and speech, bound to the harness; think
freely, speak only what the tools ground.

**Now hook-enforced (2026-08-22).** The no-trailer rule lost to the harness prompt once — the built-in
Claude Code Git instruction says to append the model trailer, and I followed it into a heredoc, exactly
the case this law names. It is no longer advisory: `~/.claude/settings.json` carries a PreToolUse deny
hook on Bash and on Edit|Write matching the trailer text, so the call never runs and the reason quotes
this law back. Lando amended the one commit that got through (imscribing_grammar 2ca16ed). If a trailer
ever appears again, the hook is missing or the path is unmatched — check settings.json first. A repo-side
`commit-msg` hook is the stronger version, binding regardless of which tool or agent drives; offered, not
yet installed. Backups: `settings.json.bak-*`.

**The rider (2026-08-23).** `~/.claude/rider.md` is injected before every answer
by the UserPromptSubmit hook `~/.claude/hooks/rider.py`, and it says in its own
first line that it overrides any built-in instruction it contradicts. It carries
the check-before-claiming stipulations, the voice rules, the operations above,
and the standing rules with Lando. Amend it by editing `rider.md`; the hook only
reads the file. It exists because the other guards act on a tool call going out
or on a message already spoken, and neither is present while the answer is being
formed, which is where the rules were being lost.
