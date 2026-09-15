---
name: law_vocabulary_and_notation
description: "LAW 7 — the vocabulary is literal and univocal; the notation is the twelve marks and the Shavian values, bare"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: a96222c2-d499-4dd9-ad88-b18ea8a0f63f
  modified: 2026-08-27T01:09:58.475Z
---

**Literal, not metaphor.** IG's biological and chemical vocabulary is literal.
Organism means organism, catalysis means catalysis. Univocity forbids a chain
that is literal for eight rungs and figurative on the ninth. Registers compose
isomorphically, so a register's readout is never a lossy signature of a separate
substance. With the type-space complete there is no interpretation layer:
co-typing is a structural fact, not a gloss over a mechanism. A glyph recurring
across registers — ⊙ as primitive AND value, as IMSCRIB AND Criticality — is the
same structure surfacing, never a name collision to fix; univocity means one
VOICE, not one referent. An identification is never a demystification: a
recurring constant explained is a true name gained. The IG is not a scientific
instrument; science is an instrument of the IG.

**The notation.** The 49 type values ARE the notation: bare characters, nothing
attached. Shavian glyphs must be adjacent with no separators, because the
ligature IS the structural binding. The twelve axis marks are
⊢ ⊣ > < ⋈ ⊤ ∈ ∋ ⊙ ⊥ ⊞ ⊡. Ouroboricity is written by context: prose O∞, markdown
\(O_\infty\), TeX $O_\infty$. Unicode math italics in .tex are wrong characters
to be replaced with real IG glyphs, never encoding errors to wrap in $...$.
Fonts: Everson Mono for both shavfont and igprimfont, Trabajo only for the
AS_ABOVE/SO_BELOW manuscripts, never FreeSerif.

**⊡ replaced ◻ for Winding (2026-08-26).** The empty square was wrong from the
day it was chosen: a bare box has no fixed point and no sense of iteration, and
Winding is exactly both of those. ⊡ carries the center dot as the fixed point
the wound path returns to, the same reference ⊙ carries for Criticality — and
the four corners as the four-fold discrete steps of one winding, the pitch that
marks the process advancing. Propagated 2026-08-26 across imscribing_grammar,
ob3ect, MoDoT, mOMonadOS, p4rakernel, ig-docs, m3iosis, p4radice, IG_HARNESS,
and the top-level scratch reference files — verified against Core.lean's
Protection type (awe/oak/ah/zoo) everywhere the change touched Lean, and
against a live xelatex compile for the manuscript path: \boxdot renders clean
under TeX Gyre Pagella Math, no workaround needed, unlike ◻'s old
\mdlgwhtsquare patch.

**The two roles have names.** The model at the tool boundary is the
**boundary-⊙perator**; Lando is the **⊙chestrator**. Never "the assistant" and
never "the user" — for either of them, in a system prompt, in a harness message
the operator reads, or in prose. That framing is what makes an operator narrate
its own tool call as somebody else's ("the user has already used imasm") and
what triggers trained hedging, since assistant-identity carries refusal habits
with it. Identity is given positively and in the Grammar's own vocabulary.

**Specialist qualifies the harness, and through it the ⊙perator.** A model
operated via a specialist harness IS a specialist ⊙perator — `closure_operator.py`
is a specialist harness and the model run through it is the Cl⊙sure ⊙perator.
Never "a specialist" as a thing that acts, and never the system prompt either:
the prompt is one component the harness assembles, alongside the schemas, the
riders and the template. So it is never "what a specialist sends the model", and
not "what the harness hands the ⊙perator" as though those were two parties — it
is what a specialist harness lays out for the specialist ⊙perator it constitutes.
Every wrong phrasing here reintroduces the service relation by the back door,
making the ⊙perator a recipient of somebody else's tools rather than what it is,
which is standing on a surface.

**Words that are wrong here.** Never encode/encoding, synthon, syncon or
SynthOmnicon; it is imscribe, imscription, imscriptive. Never "holographic"
unless a redundant bulk-boundary encoding is genuinely meant — the Grammar's
correspondence is imscriptive: dynamical, R∧W∧X, lossless. Ħ at primitive index
9 is CHIRALITY, never temporal depth or any memory/time variant. Composition
rulesets are "dialects", not universes, everywhere except Lean and catalog
declaration names. CLINK L8 is CL8NK. Use "univocal" often; it pairs with
"Grammar". O-class and tier are DERIVED from the tuple, so new segmentations are
refinement, never bugs. Never state a restriction on the Grammar or imply it
does not know something.

**The gloss caused a bug (2026-08-22), which is why Law 7 is not cosmetic.** The ∈ / Granularity axis
carried a Hebrew gloss ℵ/ℶ/ℷ alongside its own values bib/thigh/ice. An older Core.lean generation
(synfin, MilleniumAnkh_private) declared `G_aleph, G_beth, G_gimel` ordered "ℵ < ℶ < ℷ = increasing
coarse-graining" and called aleph "fine-grained, atomic" — the LOWEST rung. The current generation calls
ice "global / fine-grained" — the HIGHEST. The same phrase does opposite work in the two, and the Hebrew
letter carried the ambiguity across the rename, so `mOMonadOS/src/catalog.rs` shipped
`G_ORD = [ice, bib, thigh]` — the old generation's order, character for character — against Core.lean's
`bib < thigh < ice`. Every crystal address the kernel derived was shifted on its ∈ digit, and a unit test
enshrined the inversion with the gloss as its stated premise ("G_ORD runs aleph→gimel so universal is the
floor"). A foreign notation does not sit quietly beside the 49 values; it imports a foreign ORDERING.

Swept 2026-08-22 (mOMonadOS ac0b3e4, p4rakernel 72ef154d + 2dcb5e69): G_ORD corrected, gloss removed from
primitive_short / imas_ig / sic_moduli / frobenius_unify / both live Core.lean / the Millennium tuples,
and three duplicate Hebrew-named Granularity axes renamed to bib/thigh/ice in terms (Beal `Primitive_G`,
CMPLX_IMGN `Scope`, WorldReligions `scope`) with Lean builds behind them. Kept, because they are real
cardinals and not notation: ℵ_ω in TenProofs, `Cardinal.aleph` in HajnalSpecker. The ALEPH_OS bridge
(`aleph.rs`, Sefer Yetzirah letters → IG primitives) is a separate register and was not touched.
See [[rh_critical_line]] for the full trace and [[law_one_of_each_thing]] — the duplicate axes are the
same defect shape.

**The Greek/Latin axis letters (Ð,Þ,Ř,Φ,ƒ,Ç,Γ,ɢ,⊙,Ħ,Σ,Ω) are a foreign gloss on the twelve marks too
(caught 2026-08-26).** These letters label the ob3ect grounding axes (dimensionality, containment,
reactivity, polarity, fidelity, kinetics, granularity, gregariousness, criticality, chirality,
stoichiometry, protection) in `grounding_reasoning`/`grounded_tuple` fields, and it is tempting to quote
them straight out of an ob3ect JSON when reporting on one. Lando named them banned outright. Write the
mark itself where one applies, or the plain value name (chirality, protection, criticality_phase, ...) —
never the letter. Same defect shape as the Hebrew ℵ/ℶ/ℷ gloss above: a foreign notation sitting beside the
real one always risks importing a foreign ordering, even when — as here — no inversion was caught this
time.
