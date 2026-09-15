---
name: sudovoynichese-checkpoint
description: "Current state of IMSCRIBr/pseudo_voynich_v3.py and its standalone SudoVoynichese package — what's shipped, what's open"
metadata: 
  node_type: memory
  type: project
  originSessionId: 04b0296d-f897-49b9-ba61-7601465021fd
  modified: 2026-09-09T01:48:38.154Z
---

`~/imsgct/IMSCRIBr/pseudo_voynich_v3.py` is a generator that reproduces the
Voynich manuscript's statistical fingerprint in Shavian glyphs, zero
character overlap with the source EVA transcription. It's packaged
standalone at `~/imsgct/SudoVoynichese/sudovoynichese.py` (own git repo,
own README, Unlicense) — the two files are kept byte-identical by direct
copy after every fix, verified with `diff`, committed separately in each
repo's own voice.

**Shipped this session:**
- Fixed interlinear-transcription over-counting in `parse_ivtff` (each
  transcriber's parallel reading was being counted as independent text).
- Fixed KL-divergence and repetition-rate sample-size mismatches in
  verification; corpus-wide Match table now reads 92-99% across runs.
- `Primitive` enum's values are now the actual IG glyphs (`D.value ==
  '⊢'`), not an auto-int with the mark in a comment — attribute names stay
  Latin since Python rejects Sm-category characters as identifiers.
- Section dividers switched to the literal glyph string `# ⊢⊡⊙⊙⊙…⊙⊡⊣`.
- Fixed `Gate2_BalneologicalHeap.evaluate()`, which was structurally
  vacuous: `vessel_capacity` checked `n_ops <= 16` against a generator that
  only ever draws `n_ops` from `randint(3, 16)`, and `vessel_fused`/
  `vessel_split` were hardcoded `True` with no reachable `False` path. Also
  found the volatility check compared K's distillation value against
  `grammar['P']`, the wrong axis. Fixed by implementing ENGINE.md's actual
  `heap_folio = folios[folio_number % 20]` selection against twenty
  authored (FSPLIT, FFUSE) capacity pairs, gating volatility on K plus the
  `volatilis` field (generated per entry, previously never reached gate
  context), and gating the `cold_process` claim on whether this entry's own
  `n_ops` actually produces a Calefac step (`n_ops >= 5`) rather than firing
  on K alone. Confirmed with direct execution (2000-5000 run samples):
  `vessel_capacity` and `vessel_fused` now come back both True and False;
  `cold_process` comes back True only, which is now accurate rather than
  coincidental, since ENGINE.md itself argues that entailment always holds
  once applicable.

**Open, found but not chased:** the IMASM structural-fingerprint comparison
(`classifier.py`'s `coarse_key()`, distinct from the statistical Match
table) shows three of six sections — astronomical, cosmological,
pharmaceutical — diverging from the real corpus specifically on
`frobenius_order` and/or `dialetheia_complete`, not just start/end-token
noise like the two sections that come back `FROB+DIAL`. The synthetic text
over-produces dialetheia-completeness in astronomical/pharmaceutical and an
inverted frobenius order (fuse→split) in cosmological. Not yet
investigated — flagged to Lando, waiting on direction.

See [[feedback_no_vacuous_checks]] for the standard applied to the Gate 2
fix.
