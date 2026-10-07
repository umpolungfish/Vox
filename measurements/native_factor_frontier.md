# Current native factor frontier

Vox's native shape-routed factor command closes both original sources:

| Source | Factors | Elapsed seconds |
| --- | --- | --- |
| 229513619370652772473594096727489823787 | 15058366252086023423 × 15241601613910701269 | 0.19 |
| 271690685666312585018220346622515128917 | 15726440087894854657 × 17276044937559748181 | 0.22 |

Each pair passes the membrane product check and both resident primality
checks return `prime`. These are native-route executions, separate from
the ququart-only phase-progress measurements.

The independently retained 208-bit source
`266175155279792434973506259190185380993589458872977057421959357`
closes in 35.14 seconds with the full-L2 score block. Its pair is
`13419294329574030763782528217097 × 19835257260375005453724574658581`.
Both primality checks return `prime` and the product check passes.

Vox's bounded native profile identifies the root-stride additions at
`0x77102` and `0x77123` as the hottest instructions in that source-baked
ELF. At approximately 24 seconds the sieve has collected 7,298 of its
10,989 requested relations. Coefficient A and target A both have 81 bits.

A half-L2 score block passes all thirteen sieve controls but closes the
same source in 41.35 seconds. The two executions emit identical factor
equations. The smaller block is rejected and the full-L2 implementation
remains canonical. Its generated experimental disassembly has been removed.
Timings are individual executions, not aggregate benchmark estimates.

The native 208-bit execution remains above the requested 30-second bound.
Further work targets root marking and relation collection without changing
the retained product and primality checks.

## Larger retained inputs

The fastest measured 208-bit source-baked executable is retained in
`native_factor_current/membrane`, with its source, checksum and elapsed reading
in `native_factor_current/manifest.json`. The superseded ququart debug
binaries and their generated disassemblies have been removed; measurement
records and source preparations remain. The rejection-control membrane is
preserved as a control.

The next independently generated fixtures were compiled from their source
numeral alone and executed without runtime input or supplied factor witnesses:

| Width | Source | 30-second execution |
| --- | --- | --- |
| 216 | 66907153346687236573189133122442567577365561337596755157374226593 | timeout, empty factor output |
| 224 | 24937384895847894358120264124930833426221437163545452015667815601231 | timeout, empty factor output |

The process timer records 30.02 seconds for the 216-bit observation and
30.00 seconds for the 224-bit observation. Their
stdout and stderr are retained as `native216_limit.*` and `native224_limit.*`.
The temporary candidate executable has been removed after both observations;
the saved fastest membrane's checksum remains unchanged.

## Larger-width score scale

The polynomial score scale uses the full product `A × M²`. Checked
machine-word multiplication retains the narrow fast path; an overflowing
product is multiplied on the folded tapes before taking its bit length.
The regression compares both paths with full tape multiplication, including
products at and above the 128-bit boundary. All fourteen sieve controls pass.

The corrected 224-bit candidate's Vox trace records 89-bit A and target A,
candidate magnitudes of 134 to 135 bits, and 960 accepted relations out of
11,205 at 24.665519 seconds. The root-stride additions at `0x77223` and
`0x77244` remain the hottest sampled instructions in its regenerated
disassembly. Bounded sieve-counter traces now flush regularly so interrupted
profiles retain complete rows.

The saved 208-bit fastest membrane remains unchanged. The next target is
relation collection on the larger source with the full score scale preserved.
The standalone corrected execution reaches the 30-second bound with empty
factor output. Its temporary binary and disassembly have been removed after
the reading; the native samples and relation-counter trace remain.

## Live-bound selection experiment

Using the expanded `eff_bound` instead of the requested `base_bound` in
candidate-score slack passes all fourteen sieve controls, but the standalone
224-bit execution still times out at 30.00 seconds. Vox records 9,928 candidates
and 941 relations at 24.769594 seconds, compared with 4,757 candidates and 960
relations at 24.665519 seconds for the preceding candidate. The extra candidate
work does not improve relation yield. The experimental change is reverted.

The live-bound run, counter traces and timing records are retained under
`native224_live_bound*`; its temporary executable and disassembly are removed.
The saved fastest 208-bit membrane remains unchanged. The next inspection is
the shared-residual pairing step, where accepted partial candidates wait for
an identical residual before contributing a complete relation.

## Residual closure paths

Exact-square residuals now carry their verified square root directly into
matrix reconstruction without waiting for a duplicate. All fifteen release
sieve controls pass. The first tape-root implementation collects 874 relations
at 24.640455 seconds and the standalone run times out at 30.00 seconds.

The small-residual root check now uses exact integer Newton steps with a tape
fallback for wider values. Vox's expanded closure counters record, at
24.915597 seconds on the 224-bit source: 919 smooth closures, zero square
closures, 78 shared-residual closures, and 3,850 unmatched residuals. The
complete relation count is 997 of 11,205. Square closure does not improve
this source; the measurements distinguish it from the unmatched-partial path.
The counter trace is `native224_closure_paths_profile.sieve.tsv`.

Superseded temporary candidate binaries and disassemblies are removed after
inspection. The saved fastest 208-bit membrane remains unchanged.

## Small prime-power scoring experiment

Lifted roots through modulus 65,536 exactly match repeated polynomial
divisibility in the regression. All sixteen sieve controls pass. Direct
prime-power marking collects 808 relations at 24.808560 seconds. Folding the
dense marks into the score period improves this to 871 at 24.628370 seconds,
but both trail the 997-relation baseline. Runtime prime-power marking is
removed; its exact scoring regression and traces remain for future work.
The saved fastest membrane remains unchanged, and superseded experimental
executables and disassemblies are removed.
