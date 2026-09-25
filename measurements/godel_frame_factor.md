# Interframe support unbraiding through a Gödel evaluation frame

The executable specification in `co_creative_frame_check.lean` now includes
cell-local carry normalization and signed contribution cancellation. Given an
odd candidate word, `complementWord` reads the residual parity, cancels that
candidate's contribution when the next complementary bit is set, and shifts
the even residual with its signed carry. It returns a complementary word and
the remaining residual after consuming the source width. It does not choose
the candidate. These operations are currently Lean definitions; the compiled
IMASM extractor does not yet execute them.

Kernel checks establish the value identities for normalization, cancellation,
and even-residual shifting. Concrete inverse checks include exact products,
a nonzero terminal residual for a nondivisor, and word widths crossing 64,
128, and 256 bits. Those are complementary-word unit tests, not RSA factoring
results or evidence of N-only candidate selection.

`./frame_factor_build.sh <decimal-N> [frame-width]` compiles a contained binary
with the source value imscribed as an IMASM numeral and the frame width baked
into it. The executable takes no arguments. Its evaluation frame regroups the
same source support at the selected width, then transports the recovered factor
supports back through that frame before exact product closure.

At each support cell, the odd anchor seeds `p₀=q₀=1`. The active source cell
selects the next complementary cell after cancellation of the candidate
convolution contribution against the running product's resolved bit. Updating
the product tape propagates and normalizes carries. In the returned factor
frames, a group at width `u` and index `i` begins at source position `u·i`; a
group at width `v` and index `j` begins at `v·j`. Their local product is shifted
to `u·i + v·j`, then added with carry normalization. The transported products
are checked against the source across widths 2–8.

This address transport is used for product closure after the prefix search. The
prefix search still branches over admissible shorter-factor bits; the frame
address map does not yet select those bits.

For the baked input `10007000070049` at frame width 8, direct execution returns
`10007 × 1000000007`, reconstructs `10007000070049`, and prints `closure
closed`. Five direct runs took 42–43 ms each. The semiprime arm applies negative
residue reads for 3, 5, and 7 at the completed shorter support when the longer
support has at least eight cells remaining, then applies them to the terminal
complementary support.

The reported wall time includes writing the encoded source and factor words.
The factor values are returned from the tape membrane and are not build inputs.
Both factor extraction and return closure are exercised at widths 2 through 8,
including partial final groups with zero-padding restored.

The cross-width convolution test multiplies `10007` and `1000000007` as grouped
supports for every left and right width from 2 through 8. Every local product
returns at the sum of its two group origins, and the normalized support equals
the whole-tape product. The 43-digit baked case
`1702602822888915601938994848284852246010089` returns the same factors and
closes in 262 ms with transported frame multiplication included.
