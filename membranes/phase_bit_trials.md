# Phase winding and bit-register trials

Each executable contains the modulus and base as IMASM numeral words. Runtime
phase and factor arithmetic stays on dynamically sized tapes. The measured
phase arm records dyadic residue observations; the close arm checks the result
in both product-first and prefix-first order.

| N (decimal digits) | N | factors | phase observations | phase | dual closure | repeated process timing |
|---:|---:|---|---:|---:|---:|---:|
| 2 | 15 | 3 × 5 | 3 | 25.523 μs | 6.404 μs | single run rounded to 0.00 s |
| 21 | 592601653367919345407 | 257 × 2305843009213693951 | 65 | 264.225 μs | 7.202 μs | 100 runs in 0.10 s |
| 41 | 43726284149340592555043637054982215171839 | 257 × 170141183460469231731687303715884105727 | 12 | 69.912 μs | 10.157 μs | single run rounded to 0.00 s |
| 160 | 1764252998653566696750348505363918056838244872136829490214377109010684598133197605395497827649993820629165152027610580515540350915927177459831525270816569687807 | 257 × 6864797660130609714981900799081393217269435300143305409394463459185543183397656052122559640661454554977296311391480858037121987999716643812574028291115057151 | 265 | 4.273920 ms | 18.270 μs | 100 runs in 0.88 s |

The 100-process `/bin/true` control took 0.07 s. The 21-digit membrane took
0.10 s for 100 executions; the 160-digit membrane took 0.88 s. Each successful
run reported `FDE closure: T` with both support checks true. The 160-digit
executable therefore averaged 8.8 ms per process, including launch and frame
output, and its internal phase plus closure timers totaled about 4.3 ms.

The 10-digit case `1000036000099 = 1000003 × 1000033` did not close during the
measured run. Its progress stream passed 142,784 dyadic observations before
the run was manually stopped. This measures the phase-orbit cost for that
input; it produced no factor output.

The 15 case was also run through `vox run` on its emitted `.imasm` module. Vox
reported `entry(...) exited(0)` after 405,525 VM steps; that VM run took 0.63 s.

## Wider phase and closure ladder

Each added case uses the baked modulus `N = 257 × (2^m − 1)` and base 2,
with the listed Mersenne-prime exponent `m`. The phase count is the number of
dyadic residue registers observed before a repeated phase closes. Every row
reported FDE T with both closure arms true.

| m | N decimal digits | phase observations | phase winding | dual closure |
|---:|---:|---:|---:|---:|
| 521 | 160 | 265 | 4.274 ms | 18.270 μs |
| 607 | 186 | 308 | 6.014 ms | 20.526 μs |
| 1279 | 388 | 644 | 25.581 ms | 52.696 μs |
| 2203 | 666 | 739 | 55.318 ms | 70.360 μs |
| 3217 | 971 | 809 | 85.969 ms | 70.458 μs |
| 4423 | 1334 | 742 | 109.119 ms | 100.134 μs |
| 9689 | 2920 | 4849 | 2.269 s | 265.551 μs |
| 11213 | 3378 | 11217 | 6.708 s | 233.875 μs |
| 19937 | 6005 | 9973 | 13.839 s | 415.745 μs |
| 44497 | 13398 | 2786 | 14.181 s | 967.768 μs |

The encoded binary outputs the factor registers as IMASM numeral words. The
closures verify the exact product in both nesting orders. The observation count
varies with the dyadic phase cycle, so increasing the modulus width does not
monotonically increase that count.

## Variable lift radix

The build command accepts a phase base and a separate lift radix. The lift now
stores radix digits, radix powers, prefixes, and remainders as dynamic IMASM
numeral tapes. Each extension closes the running product modulo `radix^k`;
it checks the seed product residue and then the generalized coefficient-digit
equation. The binary XOR recurrence is not used for nonbinary radices. The
existing 2-valued frame sweep remains a lossless view of the input tape.

The contained membrane built with
`./factor_2adic_membrane.sh 91 membranes/radix_lift_smoke 2 3` closed
`91 = 7 × 13` with phase base 2 and lift radix 3. Running that binary reported
FDE T, with both radix-prefix and product-first closure true. Its internal
phase and dual-closure timings were 16.132 μs and 38.168 μs. A unit test also
closes the same pair in radices 2, 3, 5, 10, 11, and `2^80 + 1`, then rejects
an incorrect pair.

## Wider radix-3 semiprimes

Each baked binary used phase base 2 and lift radix 3 for
`N = 257 × (2^m − 1)`. All completed runs returned FDE T with both closure
channels true. The first four used the initial lift, which rechecked the full
modulus at every digit. Starting at `m = 4423`, the lift used the inductive
digit equation and exact terminal product closure.

| m | N decimal digits | factors | phase observations | phase winding | dual closure |
|---:|---:|---|---:|---:|---:|
| 521 | 160 | 257 × (2^521 − 1) | 265 | 4.348 ms | 266.469 ms |
| 1279 | 388 | 257 × (2^1279 − 1) | 644 | 27.361 ms | 3.125 s |
| 2203 | 666 | 257 × (2^2203 − 1) | 739 | 52.441 ms | 14.441 s |
| 3217 | 971 | 257 × (2^3217 − 1) | 809 | 87.755 ms | 42.511 s |
| 4423 | 1334 | 257 × (2^4423 − 1) | 742 | 162.360 ms | 178.401 ms |
| 9689 | 2920 | 257 × (2^9689 − 1) | 4849 | 2.284 s | 813.829 ms |
| 11213 | 3378 | 257 × (2^11213 − 1) | 11217 | 6.861 s | 1.055 s |
| 19937 | 6005 | 257 × (2^19937 − 1) | 9973 | 14.126 s | 3.219 s |
| 44497 | 13398 | 257 × (2^44497 − 1) | 2786 | 14.678 s | 15.661 s |

The inductive lift reduced the 1334-digit dual closure from 102.902 s to
178.401 ms. A 25,965-digit input, `257 × (2^86243 − 1)`, was also built with
phase base 2 and radix 3. Its run had not returned a closure when it was
interrupted; the last surfaced milestone was 65,536 observations. The output
filter suppressed later intermediate counts, so no final observation count or
timing is recorded for that run.

The measurement path now uses `QuantumPhaseSample::from_executor_landing`,
which validates the landing through `QuantumWindingPreimage::from_landing`
before fixing the numerator. The targeted ten-digit measurement test passes,
and the full library suite passes all 195 tests.

## Hidden-overhead profile and hot-path reductions

For the 13,398-digit `m = 44497` case, the original instrumented run took
36.18 s wall time: 14.66 s phase winding, 6.89 s Shor factor close, and 14.55 s
dual closure. The two closure orders independently repeated the same radix
prefix fold. Sharing that fold while retaining the two exact-product closure
orders reduced the run to 31.39 s; the output render and flush accounted for
only 25 ms.

The phase collision path also replayed three full tape modular
exponentiations to verify a relation whose states were already generated by
the recurrence `residue <- residue² mod N`, `exponent <- 2 × exponent`. The
independent `ReturnRelation::verify` remains available and tested, but the
resident path now relies on that inductive invariant rather than replaying it.
The direct encoded-bit scan in tape exponentiation additionally removes a
divide-by-two pass per exponent bit and skips the unused final square.

After both reductions, the same baked case completed in 20.01 s wall time:
4.384 s phase winding, 7.296 s factor close, 8.257 s shared radix-prefix
fold, and 25 ms output render/flush. Both closure channels were true and the
result was FDE T. Compared with the first instrumented run, wall time fell by
about 45%; most of that measured gain is in phase winding. The standalone
factor-close timings varied from 6.89 s to 8.33 s across these runs, so the
direct-bit exponent change has not yet demonstrated a separate wall-time
gain. The full library suite passes all 196 tests, including a factor-close
regression; the binary target's independent relation replay test also passes.

The silent-output build was also tested on the user-supplied 270-digit input
`233108530344407544527637656910680524145619812480305449042948611968495918245135782867888369318577116418213919268572658314913060672626911354027609793166341626693946596196427744273886601876896313468704059066746903123910748277606548649151920812699309766587514735456594993207`.
It emitted no partial output, but the process was killed with exit 137 before
closure; the captured output file was empty. The phase `BTreeMap` retains every
full residue and its exponent, so this run exposes unbounded observation-state
memory as the next bottleneck. No factorization or execution-time result is
claimed for this input.
