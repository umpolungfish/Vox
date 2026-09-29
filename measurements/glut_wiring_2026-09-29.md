# Glut computation and wiring

## Recorded baseline before repairs

The current library exports `glut_system` from `src/glut_system.rs` and
`glut_system_perfected` from `src/glut_system.rs.work`. The files are identical.
The toroidal executable imports the former. Its glut call receives the numeral
tape directly; the previously constructed width sweeps and phase base are not
arguments to that call.

The glut advances factor prefixes through the exact multiplication diagonal
and carry. At each bit it retains every pair whose product bit matches the
source. `frame_superpose` batches these advances and deduplicates identical
prefix pairs. `glut_crystal` checks full product equality at the terminal
width. `readout` takes the first crystal pair.

The probe below links the current Vox library. It compares complete surviving
states under frame widths 2 through 8 and mutates the carry of a completed
state as a control for the prefix verifier.

```bash
cargo build --lib
rustc --edition=2021 -L dependency=target/debug/deps --extern vox=target/debug/libvox.rlib measurements/glut_wiring_probe.rs -o target/debug/glut_wiring_probe
./target/debug/glut_wiring_probe
cargo test --lib glut_system -- --nocapture
```

| N | Crystal pairs in readout order | Selected pair | Toroidal glut gate |
|---:|---|---|---|
| 35 | 1×35, 5×7, 35×1, 7×5 | 1×35 | rejects |
| 56 | 28×2, 4×14, 2×28, 14×4, 56×1, 8×7, 1×56, 7×8 | 28×2 | accepts |
| 8051 | 1×8051, 97×83, 83×97, 8051×1 | 1×8051 | rejects |

For these inputs every tested frame width returns the identical final state
vector. The odd-input live-state counts double at each advanced bit: 35 reaches
32 states at position 6; 8051 reaches 4096 at position 13. The width sweep
groups the same transitions. Its current deduplication preserves those distinct
prefixes. Product-equality rejection occurs after all source bits have advanced.

For all three inputs, changing a completed state's carry by one still passes
`verify_product_bit`. That method recomputes the prefix product; it does not
check the stored carry consumed by the next advance.

The focused library test invocation passes ten tests and fails the state-count
test in both exports. The first assertion expects one state after an advance
from the odd seed; the actual vector has two. The factor tests check product
equality, allowing the selected 1×N pair.

The next wiring change is to return a proper pair from the crystal while
retaining trivial pairs in the crystal data. The reverse frame rail needs to
verify the retained carry and constrain the live prefixes before the next
frame descends. A mutation control must reject a changed carry; a readout
control must distinguish product reconstruction from proper factor extraction.
The toroidal certificate currently receives `build_resident_trace()`, a fixed
trajectory built from the resident word after extraction. A glut execution
trajectory would connect that certificate to the actual frame transitions.


## Repaired computation

Both public glut module names now use `src/glut_system.rs`. Readout selects a
proper exact pair and records its actual frame ancestry. Reverse replay checks
all multiplication columns and the stored carry. Prefix products exceeding N
are rejected because extending a nonnegative prefix cannot decrease its product.
No live-state quota or truncation is applied.

Carry and column sums are growing IMASM bit tapes. Frames start at one cell,
grow when the population contracts or stays constant, and shrink when it grows.
Their extent follows the remaining source cells rather than a configured ceiling.
The sweep closes at the first exact proper pair, which can differ from the
smallest factor available after complete enumeration. For 56 it closes at 4×14;
for 35 and 8051 the tested closures are 5×7 and 83×97.

Executed checkpoints, including dynamic carry tapes, form the carrier trace.
The existing transport has an eight-bit record length field; long payloads fold
across an unlimited number of records. `verify_glut_trace` replays the decoded
computation. `verify_glut_reentry_certificate` binds the original carrier trace
to this replay before checking passive re-entry. The toroidal glut branch uses
these functions instead of a constructed resident trace.

The toroidal width verification covers the source extent, and gematria sums use
bit-tape addition. Host positions remain allocation indexes, not numeral widths.
Physical memory and execution cost remain constraints: this exact live-state
algorithm can still expand substantially.

## Execution instrument repair

The static membrane exposed missing x86 SSE packing instructions in the lift.
The decoder and VM now implement PACKSSDW, PACKSSWB and PACKUSWB with signed
saturation and source ordering. The retained control's native and lifted outputs
agree through factor selection, trace copy, certificate transport and verification.
The lifted control exits zero after 5,088,673 VM steps. Historical failed lift
artifacts remain alongside the repaired control output.

## Verification artifacts

`glut_library_tests_2026-09-29.stdout` records the complete library suite.
Tests cover exact factor census through 128, altered carry and prefixes,
congruence aliases, actual frame replay, certificate source binding, transport
beyond machine numeral width, and carry folding beyond 128 bits.
`membranes/glut_dynamic_8051/` retains the final baked membrane and its native
and lifted execution outputs. Earlier probe and membrane outputs above remain
baseline records.

The final full suite passes 249 tests. Native and lifted 8051 outputs match
except elapsed time and the VM footer; the lifted membrane exits zero after
18,626,751 steps. The final native execution takes approximately 4 ms on this
run. These timings describe the retained 8051 case.
