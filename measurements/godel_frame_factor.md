# Full-value factor transport through a Gödel frame

`./frame_factor_build.sh <decimal-N> [frame-width]` compiles a self-contained
binary with the source value as an IMASM numeral and the selected frame width
baked into it. Running the emitted binary takes no arguments. It partitions the
complete LSB-first tape into IMASM-coded joint states, reconstructs that whole
tape for the factor membrane, sends every returned factor through the same
frame and back, then multiplies the returned tapes and checks exact closure on
the baked source.

For frame width 5, the baked input `10007000070049` executed as:

```text
factors      10007 x 1000000007
product      10007000070049
closure      closed
real         0.00 s at `/usr/bin/time` resolution
```

The reported wall time includes writing the encoded source and factor words.
The factor values are returned from the tape membrane and are not build inputs.
Both factor extraction and return closure are exercised at widths 2 through 8,
including partial final groups with zero-padding restored.
