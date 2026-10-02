# Vox diagnosis of the 208-bit closure

Input: `godel_random_semiprime_inputs_208_20261001.jsonl`, generated separately
before compilation. The same input is used for the baseline and repair.

Vox `profile-native` sampled the original baked ELF through its full closure.
The factor binary's stdout and stderr were captured; no target output was
forwarded while it ran. Its terminal stdout exactly matches the previously
verified factor pair, its stderr is empty, and its exit status is zero.

The original recorded closure took 837.364 seconds. The profiling execution
took 789.086 seconds with 77,657 jittered instruction-pointer samples.
Sampling gives an estimate of time distribution, rather than cycle counts.

| Measured code | ELF address range | Samples | Share |
| --- | --- | ---: | ---: |
| Prime-root score marking | `97c19..97cf8` | 37,738 | 48.596% |
| Scalar threshold scan | `97d4d..97d9f` | 20,627 | 26.562% |
| GF(2) matrix combination function | symbol `sieve::combine` | 7,152 | 9.210% |
| General tape division function | symbol `morphism_factor::l_divmod` | 67 | 0.086% |

These two sieve loops account for 75.158% of the complete profile. Function
shares refer to samples in that function's own instruction range.

The repair removes the bounds check from each score mark. The carried root
positions are at least the current block start; the loop guard puts the
relative index below the block's active length. The threshold scan compares
four signed scores at a time using SSE2, then visits qualifying positions in
their original order. The relation-count termination check runs after a
relation is accepted. Score storage, thresholds, candidate arithmetic, and
the closure gate retain their existing meaning.

Vox disassembly confirms that the repaired marking loops contain the score
addition, position advance, bound comparison and back branch. It also decodes
the threshold scan's `pcmpgtd` and `pmovmskb`. A scalar equivalence check covers
signed extremes, threshold equality, all partial vector tails and reuse of the
candidate buffer.

Evidence:

- `godel_208_native_baseline.samples.tsv`: every sampled native address.
- `godel_208_native_baseline.profile.tsv`: complete function distribution.
- `godel_208_native_baseline.status`: execution status and elapsed time.
- `godel_208_native_baseline.stdout` and `.stderr`: captured target output.
- `godel_208_repair.vox.disasm`: Vox's repaired instruction stream.
- `vox_native_hotspots.py`: joins samples to Vox instructions and loop edges.

The repaired binary closed on the identical input in **549.047 seconds**,
compared with the original recorded **837.364 seconds**: a **34.431%** reduction
and **1.525x** speedup. Compared with the full profiling execution's 789.086
seconds, the reduction is 30.420%. These are individual executions; the two
baseline timings show that wall time varies between runs.

Both prime lanes passed Vox primality checks. `godel check mul` passed. The
ELF's complete IMASM module recovered exactly through the glyph round trip.
The binary accepted no runtime input, emitted no stderr, and emitted its one
factor equation only after completion.

Repair result: `../membranes/godel_random_semiprime_baked_20261001_vox_repair_208/results.jsonl`.
Repair ELF SHA256: `490137b1258776930b8b7ee5ae3666f994f55b465dd937868a53afe8a1150e43`.

## Remaining cost measured through the repaired closure

The complete follow-up Vox execution took 547.538 seconds and collected 53,934
samples. Its captured factor equation exactly matches the verified repaired
binary's output. Marking at `72ed9..72f9c` accounts for 65.004% of samples.
Matrix combination accounts for 13.224%. Its pivot bit-search loop at
`6e85e..6e893` accounts for 11.646% of the whole execution, or 88.068% of samples
in matrix combination. The profile is preserved as `godel_208_native_repair.*`.

The small-prime marks now fold into one period per polynomial. Each selected
prime's two root lanes repeat every prime positions; their combined score
therefore repeats at the product of those active primes. That product is
derived from the live factor base and fits the existing block capacity. The
block seed begins at its absolute offset modulo the period and copies complete
cycles. The remaining primes keep their carried positions.

Score comparisons against direct root marking cover skipped primes, identical
roots, rotated blocks and partial tails. Folding closed the same 208-bit input
in **429.450 seconds**, a **21.783%** reduction from 549.047 seconds. Both prime
lanes, the word multiplication gate and complete IMASM recovery passed.

Result: `../membranes/godel_random_semiprime_baked_20261001_score_period_208/results.jsonl`.
ELF SHA256: `6c1e1988b649059c61fdf06252bd8c24de7e9ed857ba3507057af56fc1aaf494`.

The pivot repair searches the first nonzero matrix word, then uses its trailing
zero count to locate the same lowest column. It rejects columns beyond the
matrix width. Reference checks compare this with the original bit scan through
zero words, padding, and word boundaries. Vox decodes the new loop's word loads,
zero tests and `tzcnt`; the bit-at-a-time search has been replaced.

The same input closed in **360.475 seconds**, a **16.061%** reduction from the
folded score implementation's 429.450 seconds. From the original 837.364-second
closure, the measured reduction is **56.951%**, or **2.323x** speedup. All four
prepared binaries recover the identical factors. The final binary's primality
reads, word multiplication and complete IMASM recovery passed.

Result: `../membranes/godel_random_semiprime_baked_20261001_score_period_wordpivot_208/results.jsonl`.
ELF SHA256: `03d9bbb050153609c85463559da1ea6cb277c3d051ea2bb652933c9d3d601689`.
Decoded instructions: `godel_208_wordpivot.vox.disasm`.

Vox's `profile-native` also accepts `--registers` to preserve the live general
registers with each address sample. The hotspot reader reports source register
values at hot additions, allowing the remaining prime strides to be measured.

## Register trace and rejected root pairing

The retained implementation completed a full Vox register profile in 360.034
seconds (35,361 samples). Its factor output matches the verified 360.475-second
binary, with empty stderr. Remaining root marking at `713ed..71454` accounts
for 63.940% of samples. The two stride additions at `71433` and `7144c`
account for 24.620% and 23.763%. Live strides include 37, 43, 53 and 61.
These are factor-base primes, independently read from the running registers.

An exact paired-root recurrence preserved every score and both carried root
positions in reference checks, but the same input took 399.382 seconds.
That is 10.793% slower than 360.475 seconds. The paired-root source change
was removed; the measured faster implementation remains canonical. Its
experimental result and Vox disassembly are retained for reproducibility.
This experiment demonstrates that reducing loop branches alone does not
establish a speedup. No larger input was used.

Register profile: `godel_208_native_folded_wordpivot.*`.
Rejected experiment: `../membranes/godel_random_semiprime_baked_20261001_score_period_wordpivot_rootpair_208/results.jsonl`.
Rejected instruction stream: `godel_208_rootpair.vox.disasm`.

## Hardware counter reading and independent second period

A second independent score period passed direct-mark equivalence checks and
closed the same input in 401.288 seconds. Its score addition is decoded as an
eight-lane `paddd` loop. The source change was removed because it did not improve
on the retained 360.475-second execution. This execution overlapped a Gödel
analysis of the same input and a short build; its wall time is not an isolated
benchmark. The experimental ELF and full closure gates remain available in
`../membranes/godel_random_semiprime_baked_20261001_two_periods_208/`.

Vox now records user-space task hardware counters alongside native sampling.
A silent control executes a five-million-iteration loop with ten decoded
instructions per iteration. Vox records 50,166,894 instructions, compared with
193,168 for `/usr/bin/true`. Both controls exit zero with empty stdout and stderr.
The perf ABI comes from the local Linux headers; the size is checked at compile
time. Unsupported events are recorded as errors, without disabling sampling.

The retained 208-bit binary completed its hardware profile in 364.872 seconds,
with 35,970 samples. Root marking remains 64.012% of sampled execution. Terminal
output matches the verified binary exactly; stderr is empty. Hardware counts
are raw, with enabled and running times preserved. All six events ran without
multiplexing in this execution. The profile overlapped Gödel analysis; hardware
counts describe this observed execution.

| Event | Count |
| --- | ---: |
| cycles | 1,714,570,934,373 |
| instructions | 2,722,804,255,735 |
| cache_references | 226,879,284,243 |
| cache_misses | 2,107,975,942 |
| branch_misses | 19,297,303,846 |
| l1d_read_misses | 215,677,924,105 |

The whole-execution instruction/cycle ratio is 1.588; cache misses/reference is 0.929%. These aggregate readings do not establish a particular instruction
as the cause of stalls. The register trace continues to identify the remaining
prime-root recurrence. Future baked result records include process CPU time
separately from elapsed time. The live Gödel analysis also reached its verified
Stage 4 factor pair, multiplication gate PASS and word-primality PASS.

Hardware profile: `godel_208_native_hardware.*`.
Counter controls: `vox_native_counter_control_20261001.*` and
`vox_native_counter_loop_control_20261001.*`.
Rejected second-period instructions: `godel_208_two_periods.vox.disasm`.

## Silent program-state reading

Vox can now read the prepared ELF's `VOX_SIEVE_COUNTERS` object symbol while
the child is stopped for sampling. The function-symbol table retains its
function-only meaning. A bounded ELF object-symbol reader finds the data
address and validates its storage size before the tracer reads nine words.
The silent control records the expected constants and advancing values, with
empty target stdout and stderr. The target does not write a diagnostic file.

The complete gauged baseline closes in 384.241 seconds. Its terminal factor
equation matches the verified retained binary and stderr is empty. The final
observed fields are:

| Field | Value |
| --- | ---: |
| Input bits | 208 |
| Factor-base columns | 10,925 |
| Relation target | 10,989 |
| Polynomials | 5,741 |
| Scanned positions | 137,780,125,292 |
| Candidate positions | 1,658,297 |
| Accepted relations | 10,989 |
| Surviving matrix rows | 9,896 |
| Surviving matrix columns | 9,463 |

Only 0.663% of candidate positions produce accepted relations. Matrix closure
was first attempted at the width-based relation target. The matrix remains
large after singleton removal on this input. Source commentary now describes
the observed dimensions instead of assuming the residual is a few hundred.

This execution includes tracing and program-state observation. It is a
diagnostic run, not an isolated wall-time comparison against the untraced
360.475-second result. A short test build overlapped part of the execution.
Profile and observed fields: `godel_208_silent_gauge_profile_20261001.*`.
Prepared baseline: `../membranes/godel_208_silent_gauges_20261001/factor_one`.

A controlled dependency with two valid rows and sixteen base columns returns
a factor and passes the word product check before a width-based relation
target would be reached. The tested candidate now attempts the existing matrix
solver at doubling relation counts; its exhaustive final gate remains present.
No sieve window, score threshold or factor-base parameter is changed. Its full
208-bit execution is being measured through Vox before deciding whether to
retain that gate.

The doubling-checkpoint execution closed in 390.837 seconds, with the identical
5,741 polynomials, scanned positions, candidates and 10,989 accepted relations.
At 4,096 rows its singleton-pruned matrix was empty. At 8,192 rows it had
5,725 surviving rows and 7,606 columns; no factor returned. Final dimensions
were the same 9,896 by 9,463 as the measured baseline. Terminal factors match
exactly and stderr is empty. The extra gate did not reduce sieve work and
was removed. Its profile and prepared binary are retained.

This observation redirects the next diagnosis to rejected residual cofactors.
The silent gauges now classify these against the live base bound squared and
its fourth power, preserving exact equality at both boundaries. The wide
classification uses the residual word and word-level bounds. The extraction
algorithm and original final relation gate remain intact.

Rejected gate: `godel_208_early_matrix_profile_20261001.*`.
Prepared experiment: `../membranes/godel_208_early_matrix_20261001/factor_one`.

## Shared residual word closure

The complete residual reading closed in 373.538 seconds. It records 1,318,757
non-unit residuals at or below the live base bound squared, and 324,859 above
that bound but at or below its fourth power. None exceeded the fourth power
in this reading. The terminal factor pair matches the verified baseline and
stderr is empty. This identifies a large discarded extraction surface.

The repair retains the first partial relation for each residual word. Later
relations with that exact word multiply their left lanes modulo N and add
their factor-base exponents. The common residual contributes its exact square;
the matrix reconstruction carries its square root into Y before the final
gcd. Partial exponents are stored sparsely. A residual need not be declared
prime for this identity to hold. The cap is derived from the live factor base.

Controls cover the required square contribution, reuse of the first anchor,
signed parity and a composite shared residual. Omitting the square contribution
fails the factor-producing controls. All nine sieve kernel controls pass.
The original sieve window, score threshold and factor base are retained.

The same separately generated 208-bit input closes in **193.083 seconds**,
with **201.001 seconds** reported process CPU time. That is **46.436%** below
the retained 360.475-second untraced result. Both factor lanes, Gödel word
multiplication and complete IMASM recovery pass. These are individual runs;
CPU time and wall time are separate instrument readings.

Vox's complete follow-up execution closes in **159.197 seconds** with
15,675 samples and the same factor equation. It records **2,103 polynomials**
and **50,464,779,318 scanned positions**, compared with 5,741 polynomials and
137,780,125,292 positions before the repair. Root marking at `93e7d..93ee4`
still accounts for **59.030%** of samples. Matrix reconstruction accounts
for **7.305%**. The remaining measurements direct further work at the sieve
window and its score-selection rule. The under-one-minute objective is active.

Result: `../membranes/godel_random_semiprime_baked_20261001_shared_residual_208/results.jsonl`.
ELF SHA256: `18977ad20804a4393b7f2012a0e3b162fa698ab51683486051246a6f0e619123`.
Vox decoding: `godel_208_shared_residual.vox.disasm`.
Full follow-up reading: `godel_208_shared_residual_profile_20261001.*`.
