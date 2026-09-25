# Baked frame-factor scaling run

Each executable was rebuilt from `frame_factor_build.sh <N> <frame-width>` with `RUSTFLAGS=-D warnings`, then run directly. The only baked input is the compiler-produced `cell-binary` IMASM numeral for N. The report is emitted after the frame walk completes. Every fixture returned the listed pair and printed `closure closed`.

| N decimal digits | Factor bit widths | N | Returned factors | Frame width | Runtime | Binary |
|---:|---:|---:|---:|---:|---:|---|
| 11 | 17 × 17 | 10002200057 | 100003 × 100019 | 8 | 0.148 s | `frame_factor_1532da89708f509c4d96` |
| 13 | 20 × 20 | 1000036000099 | 1000003 × 1000033 | 8 | 1.622 s | `frame_factor_9e9dc21fafae368e5d04` |
| 14 | 14 × 30 | 10007000070049 | 10007 × 1000000007 | 8 | 0.043 s | `frame_factor_d62a36cacb88adfbfe6d` |
| 15 | 17 × 30 | 100003000700021 | 100003 × 1000000007 | 8 | 0.302 s | `frame_factor_449a0b5466066f9c72ea` |
| 16 | 20 × 30 | 1000003007000021 | 1000003 × 1000000007 | 8 | 2.591 s | `frame_factor_8ea05ac35e1f608afb60` |
| 17 | 24 × 30 | 10000019070000133 | 10000019 × 1000000007 | 8 | 29.859 s | `frame_factor_e57d7d5e1842f3c4a4ec` |
| 23 | 14 × 61 | 23074570993201435367657 | 10007 × 2305843009213693951 | 8 | 0.110 s | `frame_factor_8cdbe764c7516ab99526` |
| 24 | 17 × 61 | 230591218450397036181853 | 100003 × 2305843009213693951 | 8 | 0.792 s | `frame_factor_3f74d4c490d3785468f5` |
| 31 | 14 × 89 | 6194032986564400205457768044777 | 10007 × 618970019642690137449562111 | 8 | 0.164 s | `frame_factor_811624a191ec7e89ce52` |
| 43 | 14 × 127 | 1702602822888915601938994848284852246010089 | 10007 × 170141183460469231731687303715884105727 | 8 | 0.262 s | `frame_factor_212df6193784cbada649` |
| 43 | 14 × 127 | 1702602822888915601938994848284852246010089 | 10007 × 170141183460469231731687303715884105727 | 2 | 0.280 s | `frame_factor_94687891bcd06f2be301` |
| 44 | 17 × 127 | 17014628769597304580863925433499558225017181 | 100003 × 170141183460469231731687303715884105727 | 8 | 2.175 s | `frame_factor_56b7099d5f0ffd6ac7c5` |
| 45 | 20 × 127 | 170141693884019613139382498777795253379317181 | 1000003 × 170141183460469231731687303715884105727 | 8 | 21.373 s | `frame_factor_a2015a1f1c2016c55e4e` |
| 46 | 24 × 127 | 1701415067287178066232275939217611659068008813 | 10000019 × 170141183460469231731687303715884105727 | 8 | 233.063 s | `frame_factor_8387d1707a9b2f3befa3` |

## Readout

The route closes on inputs up to 43 decimal digits when the smaller factor is 14 bits. The 14-bit by 30-bit semiprime runs in 43 ms, below the 50 ms reference. Increasing the larger factor from 30 to 127 bits raised runtime from 43 ms to 287 ms. The smaller-factor width is the sharper cost driver in this set: at 24 bits, the 17-digit input took 29.859 s. Balanced 17-bit and 20-bit factors took 148 ms and 1.622 s respectively.

The wider 127-bit-factor sweep closes at 44, 45, and 46 decimal digits, with shorter-factor widths 17, 20, and 24 bits. Direct executable times rise from 2.175 s to 21.373 s to 233.063 s. Frame multiplication now returns each group product at the sum of its source-group origins and normalizes the accumulated carries. The shorter-factor extraction still branches over both values of each unresolved bit, so that search remains the dominant width-dependent cost and does not meet a sub-second target.

The inverse-convolution route reads each target cell from the active evaluation frame. It fixes the odd anchor `p₀=q₀=1`, shifts a candidate `pₖ` into the current support position, cancels `bitₖ(PQ) XOR pₖ` against the source cell to recover `qₖ`, then advances the carry-normalized product tape. Frame widths 2–8 are tested as lossless regroupings of the same source word. The semiprime specialization rejects shorter-factor candidates divisible by 3, 5, or 7 when the longer support still has at least eight unresolved cells; the corresponding negative residue reads are also applied at the terminal complementary factor.

The run supports exact factor-pair closure, not a primality certificate. These fixtures were constructed from known primes. A mistyped candidate, `100003000210021`, was excluded: it is not `100003 × 1000000007` and the membrane returned `421 × 237536817601`. This is a useful reminder that exact product closure alone does not establish semiprimality.

## Interframe candidate wiring, 2026-09-24 (bounded version, superseded below)

The prior executable flattened each frame back to the source and then entered
the two-branch factor-bit recursion. I added a tape-native support-polynomial
read at the first eight dyadic phase states. The width-8 evaluation proposes a
proper gcd; widths 2–7 must return the identical residue, and the proposed pair
must then close through the carry-normalized inverse-convolution frame and all
cross-width product-return checks. This gives the phase/frame read a real role
in candidate formation instead of only checking a pair after the search.

The N-only binaries below were rebuilt with warnings denied and run directly.
Twenty executions per binary were timed with `perf_counter_ns`; the table gives
the median and maximum wall times, including process startup and complete
readout. The source numeral is baked in IMASM form; neither factor is a build
input.

| Decimal digits | Factor bits | N | Returned factors | Phase index | Median / max | Binary |
|---:|---:|---:|---:|---:|---:|---|
| 43 | 14 × 127 | 1702602822888915601938994848284852246010089 | 10007 × 170141183460469231731687303715884105727 | 1 | 1.824 / 2.202 ms | `frame_factor_212df6193784cbada649` |
| 11 | 17 × 17 | 10002200057 | 100003 × 100019 | none | 152.848 / 177.765 ms | `frame_factor_1532da89708f509c4d96` |
| 44 | 17 × 127 | 17014628769597304580863925433499558225017181 | 100003 × 170141183460469231731687303715884105727 | 1 | 1.798 / 2.540 ms | `frame_factor_56b7099d5f0ffd6ac7c5` |
| 45 | 20 × 127 | 170141693884019613139382498777795253379317181 | 1000003 × 170141183460469231731687303715884105727 | 1 | 1.856 / 2.873 ms | `frame_factor_a2015a1f1c2016c55e4e` |
| 46 | 24 × 127 | 1701415067287178066232275939217611659068008813 | 10000019 × 170141183460469231731687303715884105727 | 1 | 1.965 / 2.984 ms | `frame_factor_8387d1707a9b2f3befa3` |
| 19 | 30 × 30 | 1000000016000000063 | not closed after 5 s | none | timed out; silent | `frame_factor_688766701f761ca6937c` |

The wider asymmetric fixtures now close under 3 ms because their support
polynomial has a proper factor target at phase index 1. This is not yet a
width-independent result. This support read is a frame-consistent candidate
probe, not yet the full operation-transport map between distinct evaluation
frames. The balanced 19-digit RSA fixture
`1000000016000000063 = 1000000007 × 1000000009` produces no target in the
first eight phase states; its fallback bit-branch route was still running
after a 5-second timeout and emitted no partial report. Its contained binary
is `frame_factor_688766701f761ca6937c`. The 11-digit balanced fixture
`10002200057` also misses the phase probe, then closes by fallback in a
152.848 ms median (177.765 ms maximum). The current phase/frame selector
therefore explains the dramatic speedup for the 127-bit-factor set but does
not solve the balanced case. The next wiring task is to derive additional
candidate states from interframe operation transport, not to extend the same
factor-bit branch search.

## Removal of the phase-state cutoff, 2026-09-24

The earlier implementation's `1..=8` loop was an arbitrary guard I added. It
was not implied by the codec, frame widths, or phase recurrence. I removed it.
The phase counter is now an IMASM numeral tape; support reads occur whenever
that counter has exactly one set bit (1, 2, 4, 8, ...), with no terminal count.
If no support read closes, the phase walk continues until its modular state
repeats, then attempts the paired half-step closure. The state map is dynamic;
there is no fixed state-count or bit-width limit in this path.

Regression `N=4629` reaches its support target at phase index 16, beyond the
old cutoff. Rebuilt N-only binaries for the 43–46 digit asymmetric fixtures
still close at index 1, with twenty-run medians of 1.745, 1.798, 1.856, and
1.907 ms respectively. The rebuilt `10002200057` fallback closes in 320 ms.
The balanced 19-digit case remains unresolved after an external 5-second
timeout; this timeout was imposed by the test invocation, not by the membrane.

This removes one arbitrary software cap, not all host assumptions. The binary
is native Rust, and its frame grouping, vector addressing, and dynamic
`BTreeMap` storage use host data structures. The arithmetic numerals and phase
counter are IMASM tapes, but the full control flow is not yet one homogeneous
IMASM instruction stream. The phase lane is a classical modular-squaring
recurrence, not a quantum measurement. The balanced case shows that removing
the cutoff alone does not meet the under-one-second target.

## Full-state phase reads and bounded-storage cycle closure, 2026-09-24

I removed the remaining sparse checkpoint rule: the support polynomial is now
read at every phase register, not just phase indices 1, 2, 4, 8, ... . I also
replaced the stored-orbit map with Brent cycle detection. The phase counter,
cycle power, and cycle length remain dynamically sized IMASM tapes; the cycle
detector retains a constant number of tapes rather than one entry per phase.

To remove repeated tape fold/unfold and modular reductions from the hot read,
the frame polynomial evaluator now uses fused multiply-add modular reduction.
Its single-limb fast path has a dynamic multi-limb fallback, and the generic
path folds the support, phase, and modulus once and keeps the frame powers and
accumulator in dynamically sized limbs until the readout boundary. The fixed
frame sweep remains widths 2–8 as specified by the analyzer; it is not a phase
count limit.

| N | Width | Result | Phase index | Direct binary time |
|---:|---:|---|---:|---:|
| 1000000016000000063 | 8 | 1000000007 × 1000000009; exact product closure | 325692 | 0.58 s |
| 23074570993201435367657 | 8 | 10007 × 2305843009213693951; exact product closure | 1 | median 1.182 ms; max 1.529 ms |
| 17014628769597304580863925433499558225017181 | 8 | 100003 × 170141183460469231731687303715884105727; exact product closure | 1 | median 1.526 ms; max 1.986 ms |
| 580284393595165992175009793 | 8 | no completion/readout | not emitted | stopped by explicit 60 s timeout |

The 19-digit balanced case now reaches its late support target under the
one-second reference without skipping phase states or retaining the orbit.
The wider 90-bit fixture did not close within the requested one-minute runtime
limit. This is evidence that the old eight-state cutoff and orbit-sized map
were unnecessary implementation constraints, but removing them does not
establish width-independent sub-second closure. The binary still has native
Rust control flow and is still a deterministic single-base phase walk; it is
not a pure IMASM instruction stream or a physical quantum execution.

## IMASM carrier composition

`frame_factor_carrier.imasm` is the executable carrier word: its seven ordered
motifs are PHASE, ARITHMETIC, BRANCH, SELECT, CONTINUE, FIX, and UNBRAID. The
input is supplied to the carrier as a native numeral word, so decimal digits do
not enter the factor operation. `vox construct-carrier` reads the seven motifs;
`vox factor-with <carrier-word> <numeral-word>` returns one factor word.

| Input | Factor pair | Factor-word closure | Factor-with wall time |
|---|---|---|---:|
| 8,051 | 83 × 97 | returned 97; `vox verify` returned `p*q == N: true` | under 1 ms |
| 1,000,036,000,099 | 1,000,003 × 1,000,033 | returned 1,000,033; `vox verify` returned `p*q == N: true` | 6 ms |
| 580,284,393,595,165,992,175,009,793 | 3,221,225,473 × 180,143,985,094,819,841 | returned factor matched one input prime; `vox verify` returned `p*q == N: true` | 1.012 s |

The carrier word carries the operation ordering, and each numeral remains an
IMASM tape across the call. These runs verify the composed PHASE-to-UNBRAID
route on the listed inputs.

## Corrected phase-return wiring

The earlier 90-bit timeout was from the support-polynomial candidate route.
For this fixture, the phase-return branch closes directly: retain the complete
dyadic winding relation, bank and restore it through EXTRACT, derive both
factor-register seeds from the relation's two half-step residues, then require
both nested product/prefix closures to agree before emitting the pair. The
compiled input contains only the IMASM numeral for N, phase base 2, and lift
radix 4294967296; the two factors are not build inputs.

| N | Width | Extracted factors | Phase observations | Winding | Dual closure | Total to flush |
|---:|---:|---|---:|---:|---:|---:|
| 580284393595165992175009793 | 89 | 3221225473 × 180143985094819841 | 59 | 115.789 µs | T; both directions true | 195.974 µs |

The standalone native executable completed and emitted the two IMASM numeral
tapes; their decoded values multiply exactly to N. `vox <binary>` audited the
complete 1,163,141-byte image with zero F verdicts. The equivalent
`vox run <binary>.imasm` instruction-by-instruction emulator did not complete
within 60 seconds, so that emulator timing is not conflated with direct native
execution. This result confirms the corrected route on this 89-bit instance;
it does not establish a width-independent runtime bound.
