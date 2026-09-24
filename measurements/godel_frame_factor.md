# Interframe support unbraiding through a Gödel evaluation frame

`./frame_factor_build.sh <decimal-N> [frame-width]` compiles a contained binary
with the source value imscribed as an IMASM numeral and the frame width baked
into it. The executable takes no arguments. Its evaluation frame regroups the
same source support at the selected width, then transports the recovered factor
supports back through that frame before exact product closure.

At each support cell, the odd anchor seeds `p₀=q₀=1`. The active source cell
selects the next complementary cell after cancellation of the candidate
convolution contribution against the running product's resolved bit. Updating
the product tape propagates and normalizes carries. The source is regrouped at
widths 2–8, and the same recovered factors close through each regrouping.

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
