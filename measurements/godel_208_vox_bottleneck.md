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

These two sieve loops account for 75.158% of the complete profile. The
speculative small-divisor change was removed before preparing the repair.

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
