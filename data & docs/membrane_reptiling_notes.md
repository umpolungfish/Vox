# Rep-tiling membrane

Source: `G-mOMonadOS/src/erdos_walks.rs`, `rep_admissible` and `isqrt_u64`.

The membrane prepares square membership once through the largest baked input.
Each readout accepts exactly the source's three classes: a square, three times
a square, or a positive sum of two positive squares. It preserves the source's
first blocked value, `n=6`, and keeps repeated inputs in request order.

The implementation bounds the table at 2,000,000 and reports input errors above
that bound. Preparation is O(L) storage and work; readouts are O(1). The source
reference is independently evaluated for every n through 1000.

The 1,000-query native control measured 2.722768 ms for independent source
classification and 0.233282 ms for prepared classification, an 11.67x ratio
including preparation. The complete static ELF ran through Vox in 1.80 s and
matched native stdout and stderr exactly. Its glyph-only word is
`membranes/glyphs/reptiling.glyphs`, 24,312,521 glyph characters.
