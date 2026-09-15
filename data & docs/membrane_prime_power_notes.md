# Prime-power membranes

Source: `/home/mrnob0dy666/imsgct/G-mOMonadOS/src/erdos_walks.rs`,
`lcm_to_n` and `row_gcd`/`binom`.
SHA-256: `4f44b2ab90a38b1948f29372a3f3df55edb0f07719cd654efd30930945aff490`.

`src/prime_power_membrane.rs` prepares a sieve once through the largest baked
input. Each prime marks its powers with its base. The interior binomial-row GCD
is that base at a prime power, and one otherwise. Rows zero and one have empty
interiors, whose GCD is zero, matching the source kernel.

The same prime-power events feed a cumulative LCM table: its running product
increases by p precisely at p^k. Requested inputs select prepared entries in
their original order, including duplicates. Preparation is O(L log log L) time
and O(L) storage; each subsequent query is O(1). This is algorithmic sharing,
not a claim of zero execution time. The executable prepares these tables when
run; compilation bakes the numeric input words, not precomputed answers.

LCM arithmetic is widened from the source's unchecked u64 to checked u128.
Overflow is recorded at an entry and every later entry; smaller queries remain
usable. The explicit table limit is 1,000,000. Neither membrane claims results
for arbitrary-size inputs.

Controls retain the original saturating-u64 binomial construction through its
source's tested range (30), then independently construct exact Pascal rows
through 120. LCM uses independent repeated GCD/LCM through 200, including the
overflow transition. Boundary, duplicate, descending and out-of-domain queries
are tested. The timing control repeats rows 2..30 and LCM queries 0..40 1,000
times; preparation and readout are reported separately. Its LCM baseline uses
checked u128 arithmetic and is not a byte-for-byte timing of the original u64.

```sh
cargo test --release --bin binomial_one --bin lcm_one -- --nocapture
./membrane_one.sh binomial 0 1 2 8 9 12 25 27 30 81 120 81
./membrane_one.sh lcm 0 1 10 20 40 80 100 10
```

Full compilation, lifting and Vox execution use the existing launcher, including
zero VM exit and exact native/Vox stdout and stderr comparison. Measurements
are recorded in `membrane_wave4_checks.log`, `membrane_binomial_vox.log`,
`membrane_lcm_vox.log` and the per-payload `membranes/` directories.

## Wide-integer execution diagnosis

The original LCM executable and the now-passing executable have identical SHA-256:
`b120f88bdf68bd060e059b1c35e5950c93e1dec1b31b31dfd80044a334de31b4`.
The source remains checked u128 with standard Rust decimal formatting.

The decoder lacked the two-register SHRD/SHLD instructions emitted for wide
shifts. Their four opcodes now consume complete operands and immediate or CL
counts, and their lifted operations are classified as arithmetic. Execution
defines masked counts, concatenated shifts, carry, count-one overflow, sign,
zero and parity. Zero counts preserve flags and destination. Architecturally
undefined 16-bit counts above 16 and overflow for counts above one use documented
deterministic behavior, not a claimed hardware guarantee.

MUL previously computed both product limbs but set flags as a comparison with
zero. That lost overflow at the compiler's checked-multiply gate. MUL now sets
CF/OF from the nonzero high limb; all IMUL forms set them from signed truncation.
The regression controls cover 8/16/32/64-bit products, signed and unsigned
overflow, shift widths 16/32/64, masked counts 0..255, and decoder boundaries.

The archived `formatter_failure_*` directory preserves the original halt.
`bit_readout_failure_*` preserves a rejected formatting workaround, which has
been removed from the source. No reduction to u64 was made.

`membrane_instruction_controls.log` records the targeted instruction tests.
The full library run in `membrane_wide_regression.log` records 46 passes and
one failure in `morphism_factor::tests::scout_reads_the_shape_and_routes` at an
Option unwrap. That file had unrelated working-tree edits and was not modified
for this diagnosis. This run is not reported as a fully passing library suite.

Follow-up: `membrane_factor_routing_fix.log` records all 47 library tests passing.
The stale scout assertion expected a standalone rho result after rho had moved
into the sieve. The revised control verifies the HARD handoff and the exact
full-route factors 1000003 and 1000000007, including their product. The unused
scout rho budget was removed and trial logs now report the actual bound.
