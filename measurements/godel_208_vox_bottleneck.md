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
