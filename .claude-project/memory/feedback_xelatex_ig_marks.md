---
name: feedback_xelatex_ig_marks
description: "Building ig-docs .tex with the 12 IG marks — unicode-math loads AFTER amsmath, \\square is undefined, and marks render BLANK if math setup is broken"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: a96222c2-d499-4dd9-ad88-b18ea8a0f63f
  modified: 2026-08-26T13:47:31.279Z
---

Building an ig-docs manuscript that uses the twelve IG marks (⊢⊣≻≺⋈⊤∈∋⊙⊥⊞⊡) in math mode with
xelatex + unicode-math + TeX Gyre Pagella Math.

**Why:** wrong load order / undefined mark macros make the marks render BLANK, not error out — a PDF
builds "successfully" (grep '^!' clean) while every mark is invisible. Caught only by grepping the log
for "Undefined control sequence" and "Missing character", which you MUST do, not just check a PDF appeared.

**How to apply:**
- Load `unicode-math` + `\setmathfont{TeX Gyre Pagella Math}` AFTER amsmath/amsthm/mathtools, not before.
- Do NOT `\usepackage{amssymb}` with unicode-math — \eth clash ("Command \eth already defined").
- `\square` is UNDEFINED under unicode-math. The Winding mark is now ⊡ (U+22A1), not the old
  ◻ (U+25FB, which needed a `\mdlgwhtsquare` workaround because Pagella lacked it). ⊡ needs no
  workaround: `\boxdot` compiles clean under this exact stack, verified directly. \boxplus,
  \bowtie, \odot, \vdash, \dashv, \in, \ni, \top, \bot, \succ, \prec all work too.
- Marks must be in math mode ($...$); raw ⊙/⊗ in text (author line) need $\odot$/$\otimes$.
- After building: `grep -c 'Undefined control sequence'` AND `grep -c 'Missing character'` must both be 0.
- Build: `xelatex` ×2-3 passes; ig-docs/build.py only INDEXES, it does not compile.
See [[behavioral_requirements_quicklist]] (commit.txt absolute path, no trailer).
