# Membrane words: decode, nest, dissolve

A guide to `membrane_dissolve.py`, a small standalone tool for the IMASM glyph
words in the Vox repo. It needs only Python 3 and has no dependencies.

Run it as:

```
python3 -I membrane_dissolve.py <command> [args]
```

`-I` (isolated mode) stops Python from importing stray modules from the current
directory. Quote any word you pass in, since the shell can mangle the glyphs.

## 1. The glyphs

Constants come from `src/vox.rs`.

| Glyph | Name | Role in a cell-binary word |
|---|---|---|
| `⊢` | VINIT | Opens a word (the skin). |
| `⊣` | TANCH | Closes a word (the skin). |
| `≻` | AFWD | First glyph of every bit frame. |
| `⋈` | CLINK | Second glyph of every bit frame; also separates lanes in a word of words. |
| `∈` | FSPLIT | Opens a membrane. One per frame, so it is the "crossing" count. |
| `∋` | FFUSE | Closes a membrane. |
| `⊥` | EVALF | The bit 1. |
| `⊤` | EVALT | The bit 0. |
| `⊙` | IMSCRIB | First of the three closing glyphs. |
| `⊡` | IFIX | Winding record. Second of the three closing glyphs. |
| `≺` | AREV | Not used by cell-binary words. |
| `⊞` | ENGAGR | Not used by cell-binary words. |

## 2. Cell-binary words

A number is a skin around one frame per bit, least significant bit first:

```
⊢  [≻ ⋈ ∈ BIT ∋] × k  ⊙ ⊡ ⊣
```

`BIT` is `⊥` for 1 and `⊤` for 0. Zero is encoded as one `⊤` frame.

The number 5 is `101` in binary, so its bits-le are `1,0,1`:

```
⊢≻⋈∈⊥∋≻⋈∈⊤∋≻⋈∈⊥∋⊙⊡⊣
```

The strict decoder mirrors `decode_cell` in `src/godel_calculus.rs`. It accepts a
word only if it starts with `⊢`, ends with `⊙⊡⊣`, and the middle is whole
five-glyph frames each holding exactly `⊥` or `⊤`.

## 3. Commands

### `enc <natural>`

Prints the cell-binary word for a number.

```
$ python3 -I membrane_dissolve.py enc 5
⊢≻⋈∈⊥∋≻⋈∈⊤∋≻⋈∈⊥∋⊙⊡⊣
```

### `dec <word>`

Strict decode. Prints the value, the frame count and the bits-le string. It
exits with an error if the word is malformed.

```
$ python3 -I membrane_dissolve.py dec "⊢≻⋈∈⊥∋≻⋈∈⊤∋≻⋈∈⊥∋⊙⊡⊣"
value   5
frames  3
bits-le 101
```

### `nest <n> [<n> ...]`

Builds a word of words: each value becomes a complete inner word inside an
`∈ … ∋` cell, the cells are joined by `⋈`, and the whole thing is wrapped in an
outer skin.

```
⊢ ∈ ⋈ ( ∈ WORD ∋ ⋈ )×k ∋ ⊙ ⊡ ⊣
```

```
$ python3 -I membrane_dissolve.py nest 5 6 7
⊢∈⋈∈⊢≻⋈∈⊥∋≻⋈∈⊤∋≻⋈∈⊥∋⊙⊡⊣∋⋈∈⊢≻⋈∈⊤∋≻⋈∈⊥∋≻⋈∈⊥∋⊙⊡⊣∋⋈∈⊢≻⋈∈⊥∋≻⋈∈⊥∋≻⋈∈⊥∋⊙⊡⊣∋⋈∋⊙⊡⊣
```

Note: the repo has no generator for this shape that I found. `nest` is rebuilt
from the three-copy example of 1234567890 used in the conversation that produced
this tool, and it reproduces that example exactly.

### `lanes <word>`

Decodes every inner word of a word of words.

```
$ python3 -I membrane_dissolve.py lanes "$(python3 -I membrane_dissolve.py nest 5 6 7)"
lane 0  value 5  frames 3
lane 1  value 6  frames 3
lane 2  value 7  frames 3
```

It fails if anything other than `⋈` sits between lanes, or if the `∈`/`∋`
brackets are unbalanced.

### `dissolve <word>`

Applies the repo's dissolution rule (`collapse_to_nested` in
`src/fixed_point_hypernest.rs`) and prints the result plus a before/after count
of each mark.

```
$ python3 -I membrane_dissolve.py dissolve "$(python3 -I membrane_dissolve.py nest 1 2)"
⊢∈⋈≻⋈⊥⊙⊡⋈≻⋈⊤≻⋈⊥⊙⊡⋈∋⊙⊡⊣

mark              before   after
VINIT                  3       1
FSPLIT                 6       1
FFUSE                  6       1
TANCH                  3       1
IFIX (winding)         3       3
IMSCRIB                3       3
CLINK                  6       6
AFWD                   3       3
EVALT                  1       1
EVALF                  2       2
```

### `shift <word> <s>` and `unshift <word> <s>`

`shift` prepends `s` `⊤` frames after `⊢`, which multiplies the value by 2^s.
`unshift` removes `s` leading `⊤` frames and fails if the word doesn't start with
that many. Zero is the exception, covered in section 5.

```
$ python3 -I membrane_dissolve.py shift "⊢≻⋈∈⊥∋≻⋈∈⊤∋≻⋈∈⊥∋⊙⊡⊣" 2
⊢≻⋈∈⊤∋≻⋈∈⊤∋≻⋈∈⊥∋≻⋈∈⊤∋≻⋈∈⊥∋⊙⊡⊣
```

### `pipe add|mul|braid <a> <b> <s>` and `pipe unbraid <n> <k>`

Runs encode → shift → operation → unshift → decode and compares the result with
the plain computation. See section 5 for how much each operation unshifts.

```
$ python3 -I membrane_dissolve.py pipe mul 1234567890 987654321 3
encode     a 31 frames, b 30 frames
shift      by 3: a 34 frames, b 33 frames
mul        result 67 frames  (value 78036883912086572160)
unshift    by 2*3 = 6: 61 frames
decode     1219326311126352690
expected   1219326311126352690  PASS
```

```
$ python3 -I membrane_dissolve.py pipe unbraid 1234567890 5
encode     n 31 frames
shift      by 5: 36 frames
unbraid    lanes 17 and 18 frames
unshift    odd shift: lanes swapped; unshift first lane by 3, second by 2
decode     gamma 38412, lambda 10521
expected   gamma 38412, lambda 10521  PASS
```

### `selftest`

Runs 23 checks and prints `PASS` or `FAIL` for each. It covers the repo's own
collapse test vectors, an encode/decode round trip, the values printed by
`godel encode 1234567890`, the strict decoder's rejections, the three-copy
nested example, and the shift and pipeline round trips.

## 4. How dissolution works

`collapse_to_nested` keeps exactly four marks and dissolves every other copy of
them:

- the first `⊢`
- the first `∈`
- the last `∋`
- the last `⊣`

Everything else is left alone, including every `⊡` winding record.

What follows from the rule:

- **One skin, one membrane.** After dissolving, each of `⊢ ∈ ∋ ⊣` appears once.
- **Winding is preserved.** The `⊡` count does not change. The repo's doc
  comment says this explicitly: the depth is kept while the boundary count drops
  to one of each.
- **It also dissolves the bit cells.** The `∈ ∋` inside each `≻⋈∈⊥∋` frame are
  interior boundaries too, so they go as well. The frames become `≻⋈⊥`.
- **Lanes stay separate.** The bit marks survive in order, but the lanes are
  separated by `⊙⊡⋈`, not merged into one numeral.
- **The result is not a cell-binary word.** The strict decoder rejects it.
- **It is idempotent.** Dissolving a dissolved word changes nothing.

Caveat: the repo wrote this function for its hypernest words. Applying it to a
word of cell-binary words is a reading of "dissolves the inner membranes to a
single one". If your system dissolves these differently, change
`collapse_to_nested` in the script.

## 5. Frame shift and the pipeline

A frame shift by `s` prepends `s` `⊤` frames, which multiplies the value by 2^s.
The operations (`add`, `mul`, `braid`, `unbraid`) work at the value level, as
`godel check` does: they decode their inputs, compute on naturals and re-encode.
Only the shift and unshift act on words. The pipeline
`encode → shift → op → unshift → decode` therefore returns the plain result, as
long as each operation is unshifted by the amount it produced:

| Operation | Inputs | Result carries | Unshift by |
|---|---|---|---|
| `add` | both shifted by `s` | `s` | `s` |
| `mul` | both shifted by `s` | `2s` (shifts add) | `2s` |
| `braid` | both shifted by `s` | `2s` (the interleave doubles it) | `2s` |
| `unbraid` | one input shifted by `k = 2s` | each lane shifted by `s` | `s` per lane |
| `unbraid` | one input shifted by `k = 2s+1` | lanes swapped: first lane `s+1`, second `s` | `s+1` and `s`, then swap back |

So operations commute with a frame shift only in a graded sense: the shift
amount changes as it passes through the operation, and an odd shift exchanges
the two lanes of an unbraid.

Edge cases:

- **Zero** has no shifted word, because `encode(0)` is one `⊤` frame and
  `0 << s` is still `0`. `shift` refuses it, `unshift` returns a zero word
  unchanged, and `pipe unbraid` leaves a zero lane alone.
- **Dissolution commutes with a per-lane shift.** Shifting every lane of a word
  of words by `s` and then dissolving gives the same word as dissolving first
  and inserting `s` dissolved `⊤` frames (`≻⋈⊤`) at the start of each lane.
  This was checked as a one-off over 500 random cases and is not a command.
- The value-level rows follow from exponent and bit-index arithmetic, so the
  tests confirm the implementation against the repo's `braid_values` and
  `unbraid_value`. They are ports of the Rust source, not runs of the compiled
  binary.

## 6. Verification status

- The `dissolve` output matches a verbatim Rust copy of `collapse_to_nested` on
  the 493-glyph three-copy word.
- The collapse test vectors come from the repo's own unit test.
- The codec matches the numbers printed by `godel encode 1234567890`: 31 frames,
  support `{1,4,6,7,9,17,18,20,23,24,27,30}`.
- The pipelines pass 400 random round trips each for `add`, `mul` and `braid`
  (shifts 0 to 9) and 600 for `unbraid` (shifts 0 to 11), plus the zero-lane
  cases for n = 1, 2, 3.
- Not covered: the `affine-edit` and `product-fusion` families in
  `src/godel_calculus.rs`, and the `⊢⊙∈…` hypernest words, beyond the one
  collapse test vector. Also not covered: regrouping frames at another width
  (the `analyze` sweep over widths 2 to 8), moving a frame boundary, and
  word-level arithmetic that carries across frame groups.

## 7. The repo's own `godel` binary

For arithmetic and factoring, use the Rust binary. Its dispatch (in
`src/bin/godel.rs`) covers `encode`, `decode`, `check add|mul`, `unbraid`,
`braid`, `relation`, `analyze`, `lte2`, `product`, `separate`, `factor`, `shiab`,
`frame` and `selftest`. `godel help` after `cargo build --release` is the
authoritative list.
