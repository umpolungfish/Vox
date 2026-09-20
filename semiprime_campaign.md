# Semiprime execution campaign

## HSOA membrane integration

The project's working architectural description is a fractal holomorphic
semiotic operator algebraic P-system. Executable checks below specify which
properties have been measured in this implementation.

`python3 hsoa_membrane_check.py` builds baked-input phase membranes and compares
their entire spectra with `G-mOMonadOS/hsoa_shor_state.py`'s NumPy Fourier
readout. The external reference state is prepared from modular powers without
calling `true_period`. Factors close from the Rust recovered winding, rather
than a reference order. The native controls recover 4, 6, and 12 for the
15, 21, and 35 cases and emit verified pairs. The 175-bit balanced case at
four register qubits remains a singleton observation with no inferred winding.
This is a wide-arithmetic negative control, not a 175-bit factoring result.

Each executable is lifted with the existing `vox imasm`, serialized with
`vox glyphs`, and recovered with `vox unglyphs`. Exact module recovery is
checked before `vox run`; native stdout/stderr and VM exit must all agree.
Failures are retained and make the battery exit nonzero. Logs and executable
modules live under `membranes/hsoa/` in unique run directories.

Current integration status: all four native checks, exact serialized-module
recoveries, and complete glyph-module executions pass. Native and VM stdout and
stderr agree exactly, and every VM process exits zero. The successful run is
`membranes/hsoa/1789930626756584634/results.jsonl`, with a tracked summary in
`hsoa-membrane-verified.jsonl`. The wide case remains unresolved spectrally in
both executions; this agreement does not assert a factorization.

The previous failure at `0x19543` was caused by missing packed-word shift
decoding: bytes inside `psrlw` were misread as a conditional branch. Packed
word/dword shift execution was also absent. The decoder and VM now preserve
instruction boundaries, shift each lane independently, saturate oversized
logical shifts to zero, and sign-fill arithmetic right shifts. Regression tests
cover boundary counts, huge register counts, and full-width byte shifts.
All 82 library tests pass. Earlier failure records remain in
`hsoa-membrane-results.jsonl` and their original local run directories.

The experimental `phase_partners.rs` resident observes modular powers at dyadic
exponents. Equal residues produce a return-exponent witness, replayed against
both exponents and the modulus. This may be a multiple of the minimal order.
Small controls and corrupted-witness rejection pass; no wide factoring or
Fourier-sampling claim is attached to this experimental path.

Run `python3 semiprime_campaign.py --output results.jsonl`.
The Python harness requires SymPy. Output files are created exclusively and
each completed case is flushed immediately. The default minimum width is 175
bits, with five samples per family. There is no default execution timeout.
`--seconds` explicitly opts into an external measurement timeout.

Each case builds and retains a separate executable. The canonical Rust
`emit_numeral` produces the embedded IMASM modulus and base words at build time.
Register widths, where used, are also embedded IMASM numerals. Execution has
no numeric arguments, stdin input, or environment input. Expected factors remain
in the parent process. Success requires both expected factors in the producer readout.
The build/copy pair is locked so concurrent builders cannot exchange case binaries.
The recorded execution time excludes compilation. Binaries remain under
`target/baked-semiprime/`; the log records their paths, hashes, and input words.
SymPy primality checks above 64 bits are probable-prime checks, not attached
primality certificates. Seeds and exact inputs are retained for reproduction.

## Baked phase observation controls

`python3 baked_phase_check.py` builds separate observation binaries for an
arithmetic control and every rotation of the blueprint on the balanced 175-bit
input, plus a second register width at the original cut. The execution word is
read from the supplied ob3ect JSON and baked into each binary alongside its
IMASM input numerals. The report records the blueprint hash and rotated word.
The external harness checks Fourier queries against independent modular powers,
and confirms that conflicting runtime arguments, stdin, and environment cannot
change a baked case. Reports and binaries are retained
under `target/baked-phase/`.

`PhaseObservationFrame` implements payload banking, live-register clearing, and
fusion restoration. Its regression checks exact payload and spectrum preservation
and rejects a changed live payload at fusion.
`phase_word.rs` implements sequential dispatch of all twelve marks over
observation-bearing deposits and nested frames. Each deposit keeps an identity,
lane, and actual modular-observation payload; fusion unions identities so an
uncleared deposit is not counted twice. Seeding T creates no observation.
The executable now uses this interpreter in place of a hardcoded sequence of
frame-method calls. Unit tests reproduce all twelve supplied rotation readouts,
including exposed clears and vacuous seeded landings, and test nested restoration
and malformed words. The phase binary has eleven passing unit tests.
All fourteen baked cases passed, and their register landings agree with the
independent existing `IMASM16_3_Machine`. The run is retained at
`target/baked-phase/1789929213605065499/results.jsonl`.

This interpreter implements the sequential register/weight transport question.
Graph ancestry and closure remain separate judgments. Its lane labels mark
transported observations; they do not establish a modular period. Acquisition
still uses the existing sparse modular walk and enumerates the register.
Non-enumerating acquisition and order extraction from it remain unfinished.

`semiprime-baked-input-control.jsonl` records the 175-bit close-pair factoring
control with embedded modulus and base. `phase-sparse-scaling-baked.jsonl`
records the initial baked-modulus/width scaling run, before the base was also
moved into an embedded numeral. Its binary hashes identify that earlier build.

Families currently implemented are close pairs, q near 2p, independently
sampled balanced pairs, and unbalanced pairs with p near one third of the
total width. Product widths are checked exactly. Close pairs use small gaps
and therefore principally measure arithmetic width. The multiplier family
currently probes k near two, not a sweep of multiplier depth.

The probe measures producer execution. Certificate and passive-extraction
timings, peak memory, persistent timeout checkpoints, and controlled
p-minus-one smoothness families remain to be added. Timeout means unfinished
within the selected budget. It does not establish a factoring limit.

`semiprime-calibration-175.jsonl` is an exploratory calibration log. Its first
resident close-pair result is a harness false failure: the initial probe omitted
the resident remainder. That defect was corrected and the case rerun in
`semiprime-close-sweep.jsonl`. The calibration crossed a probe rebuild and
must not be treated as a uniform binary benchmark.

`semiprime-close-sweep.jsonl` contains the corrected five-sample sweep from
175 through 512 bits for both producers, with the executable SHA-256 recorded.
All cases passed. Results include process startup and independent oracle
comparison outside the producer. Concurrent exploratory probes can affect timings.

The extension log covers five close-pair inputs each at 768 and 1024 bits.
Both producers passed every input. The 175-bit balanced calibration timed out
on both producers at sixty seconds; the multiplier-aligned dialectic case
closed in four descents while the resident timed out.

The structural battery now also exercises repeated neutral descent after
serialization at every checkpoint of the twelve-descent control:
`cargo test --release --test dialectic_neutral_every_cut`.
`dialectic_certificate_cuts` rejects every internal checkpoint deletion and
every checkpoint duplication, while verifying legitimate suffix restarts.

Next batteries cover exact Fermat interval boundaries, checkpoint splicing
between valid certificates, and an independent resident/dialectic provenance
comparison. The existing support projection comparison alone does not establish
independent producer agreement.

Direct producer options now include `phase`, `braid`, and `symbolic`.
The braid probe acquires the order, transports it in a braid, reads winding,
and closes factors. The symbolic probe invokes register construction directly,
then peak extraction and factor closure. Its current constructor computes the
order using a baby-step/giant-step table before forming a peak; removing the
braid cutoff does not change that algorithm. The phase probe invokes the nested
factor tower. These are local CPU execution paths.

The three additional 175-bit logs preserve the earlier sixty-second experiment.
Both braid inputs exhausted the former internal orbit cap. Phase and symbolic
inputs timed out. These logs precede removal of the braid cap.

The braid orbit now terminates on modular closure or invalid input. The scanned
API takes only N and returns the selected base as a tape; its former max-base
and orbit-cap arguments have been removed. No repository callers used that API.
The residue domain determines scan exhaustion. Regression coverage explicitly
crosses the former cutoff and checks factor reconstruction.

## Independent phase observations

The wide Shor entry and the campaign's `symbolic` producer now use
`observe_order`: modular evolution through the FDE carrier builds the branch
state, a QFT produces frequencies, and continued fractions propose orders
checked by modular exponentiation. Register width grows after unresolved
observations, without an execution budget. The old `SymbolicRegister` remains
as an explicitly labelled legacy reference; production no longer calls it.

The new implementation materializes a register of 2^qubits complex amplitudes.
This repairs the order-first dependency but does not provide a compact quantum
state implementation. Physical address and allocation failures remain errors.
This is a CPU simulation conditioned on the output branch f(x)=1, not sampling
a physical quantum device. No large-input speedup has been established.

`semiprime_probe observed N qubits` probes an explicitly selected register.
For N=34708385599522211756186926321308977827402135941491069 and qubits=16,
the executed probe returned branch_population=1 and unresolved phase register.
Full spectra on small controls agree with the independent direct Fourier
implementation. A singleton branch is tested to yield a flat spectrum and
no extracted order. The wide-entry regression requires observed-phase output.

## Sparse phase scaling measurement

`SparsePhaseBranch` retains only observed branch positions and evaluates a
requested Fourier probability directly from their phase sum. It accepts no
known order. Every frequency agrees with the dense QFT on the tested controls.
It is a probability-query implementation, not an independent peak locator.
The production order extractor still uses the dense implementation.

`python3 phase_scaling_check.py` records the fixed 175-bit balanced input at
16, 18, and 20 register bits. The recorded construction times are approximately
0.37, 1.50, and 6.03 seconds. Each run retains one position with a vector
capacity of 32 bytes (not total process memory). Construction executes exactly
65536, 262144, and 1048576 modular steps. The branch is a singleton and its
Fourier probabilities are uniform, so these observations expose no period.

The sparse representation removes the dense amplitude allocation for phase
queries. Construction still scales with register length, which doubles with
each added register bit. Completing scalable extraction requires a further
algorithm that obtains informative observations and locates their peaks without
enumerating that domain or supplying the order in advance. This campaign has
not established such an algorithm; no general 175-bit factoring result follows
from the sparse-memory measurement.

## Escalating baked execution battery

`hsoa_membrane_check.py --stages 175:4 256:6 512:8` generates deterministic
balanced semiprimes and bakes the base, modulus, and observation width into
canonical IMASM input words. Expected factors remain outside the executable.
Each compiled binary receives no arguments, an empty environment, and empty
stdin. Its complete Fourier spectrum is compared with independently constructed
modular observations and NumPy's FFT. Full module serialization must recover
exactly, and execution via Vox must exit successfully and reproduce both output
streams exactly. Stages stop on a failed check.

All three stages pass. Their branch populations are one and their factorization
status is unresolved. The original factor-producing controls also pass. The
report separates factored, return-without-factors, and unresolved results from
execution agreement. Execution timings include process startup and loading,
and exclude compilation. The display names are "Compiled binary execution"
and "Serialized membrane via Vox VM".

`larger-close-battery.jsonl` records two successful close-factor cases at each
of 175, 256, 512, 1024, and 2048 bits. `larger-close-battery-next.jsonl` records
successful 4096- and 8192-bit close-factor cases. These use the dialectic
producer with the modulus baked into each retained executable; the external
oracle checks both factors. These measurements establish wide arithmetic and
closure for the close-factor family. They do not establish factoring arbitrary
balanced semiprimes of those sizes.

`larger-multiplier-battery.jsonl` records two successful cases at each of 175,
256, 512, and 1024 bits with q the next prime above 2p.
`larger-multiplier-battery-next.jsonl` records successful 2048-, 4096-, and
8192-bit cases in that family. Every case closes after four descents. The
8192-bit case takes 24.935 seconds of compiled binary execution, including
startup. Prime generation and compilation are excluded. The full release
regression suite passes alongside these batteries.

Case directories use a hash of all numerical inputs, retaining those inputs
in the JSONL report. This avoids filesystem component-length failures for
large decimal moduli.

The local harnesses disable Python's decimal-conversion digit ceiling for
their generated test integers. A 16384-bit decimal conversion is verified.
The factor campaign stops after a failed or timed-out case unless
`--keep-going` is explicitly requested. No timeout is applied by default.
