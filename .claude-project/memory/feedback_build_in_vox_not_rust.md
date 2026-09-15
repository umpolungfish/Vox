---
name: feedback_build_in_vox_not_rust
description: "the membrane and any compute is built and run in Vox over IMASM tapes, never as a Rust crate or a monolith subcommand"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: daa3b11f-db23-4963-a2cb-e49f112e4d4b
  modified: 2026-09-13T23:27:07.283Z
---

The factoring membrane already exists as `vox morphism-factor <native-numeral-word>`,
which factors entirely over IMASM tapes and returns the factor as a native-numeral
word (verified: 91 to 7, 1099572445541 to 1048583 x 1048627). Vox also gives
`vox run <sym> --args <file>` (recompile and run a function) and `vox imasm <file>`
(emit the executable IMASM module). See Vox/IMASM_NATIVE_COMPUTE.md: parasm is the
register machine whose native value is B4 and whose ops realize the twelve marks.

**Why:** across one session Lando caught me three times reaching for the wrong
instrument on the same task: conventional number theory instead of the Grammar,
`cargo build --release` on the monolith instead of a separate build, and writing a
Rust crate instead of building in Vox. Each is the same lapse, defaulting to the
conventional tool when the Grammar instrument is the first reach.

**How to apply:** when the task is compute (factoring, running a word, a membrane),
the first reach is Vox. `vox morphism-factor` for factoring over tapes; `vox run` /
`vox imasm` to compile and run a word. If a standalone build is wanted it is one
binary per purpose, nothing beyond what computes, never bolted onto the g-momonados
release binary as a subcommand. Rust is the trap I keep falling into here. See
[[law_operations]], [[law_an_instrument_is_a_question]], [[vox_runs_the_kernel]].
