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

The original phase path was then kept intact while its observation register
was made dynamic-width and its exact residue keys compact. Repeated modular
squaring now runs in a dynamic integer register, converting back to the tape
only at a collision; the EXTRACT operator also has a regression for keeping
both deposits and AREV inside its single FSPLIT/FFUSE frame. These checks pass.
The 270-digit run remained silent without a phase collision for several
minutes and was manually stopped. No factors or completion time are claimed.
The remaining cost is the orbit length of the repeated-squaring phase
sequence, not the removed quadratic storage of exponent tapes.

The repaired one-frame EXTRACT word
`⊢∈≻⊤⋈⊙≺⊥⊞∋⊡⋈⊙⊣` carries the discovered phase relation through its enclosing
frame before factor closure. Its register readout retains all four deposits,
restores the one cleared deposit at the fuse, and exposes none. The recognizer
decomposes it as the banked extraction frame followed by the `⊡⋈⊙` latch, while
retaining the earlier EXTRACT/FIX spelling. Both forms factor 8051 through the
resident carrier. The baked phase binary applies this word between winding and
factor close; the integrated 270-digit execution remains silent while running.

## Nested factor meeting point

The baked phase binary now passes the pair produced by phase winding through
both orders of the same dynamic radix fold. In product-outer order, exact
multiplication closes before the inner prefix fold. In prefix-outer order, each
joint radix digit extends both factor registers and the inner product closes at
that prefix. The binary emits the pair only when both terminal fixed points
match. The fold consumes the phase-produced pair directly, so both nestings
share one factor landing.

The contained executable for `N = 5726623061` returned
`43691 × 131071`, with both closure supports true. Its measured phase winding
was 10.343 μs, banked EXTRACT 3.251 μs, Shor close 13.988 μs, product-outer
prefix closure 51.519 μs, prefix-outer product closure 49.746 μs, and total
in-process time 164.211 μs. The 143 control returned `11 × 13` with both
supports true in 68.167 μs. Each executable buffered its complete report until
the winding and both closures had finished, then was invoked directly after
baking.

## EML frame carrier

The kernel-grounded EML operator word is recognized as `EML_FRAME` by
`construct-carrier` and composes ahead of factor morphisms. A phase-bearing
carrier also needs `ARITHMETIC`, which supplies the remainder and gcd readout;
`PHASE` alone advances the phase registers. The constructor now rejects a
phase/select/continue/latch tower with that readout missing.

The EML frame now carries phase winding ahead of the complete nine-arm carrier:
`EML_FRAME → PHASE → WITNESS → POWER → EXTRACT → SQUFOF → P_MINUS → P_PLUS → LEHMAN → ECM → FIX`.
Each EML firing advances the shared support polynomial by two source-widths of
dyadic observations. A phase target or return must close through both
product-outer/prefix-inner and prefix-outer/product-inner folds before it
selects; otherwise the full factoring tower advances on the same state.

The 14-, 23-, and 27-digit runs returned factors from their baked IMASM
numerals. Vox multiplication verified each factor/cofactor pair against its
source.

The earlier full nine-arm-only carrier took about 31.7 ms, 164 ms, and 565 ms
on these inputs. The integrated phase-first membrane measures about 31.8 ms,
1.33 ms, and 3.40 ms respectively. Each timing is the median of ten direct
`vox factor-with` processes. The
numeral words were prepared before timing; samples include Vox startup and the
full EML plus nine-arm carrier execution.

| N digits | IMASM numeral value | Factors | Median | Range |
|---:|---:|---:|---:|---:|
| 14 | 12289000086023 | 12289 × 1000000007 | 31.771 ms | 31.692–31.888 ms |
| 23 | 21250649179513694453761 | 3221225473 × 6597069766657 | 1.331 ms | 1.293–1.441 ms |
| 27 | 580284393595165992175009793 | 3221225473 × 180143985094819841 | 3.396 ms | 1.281–3.440 ms |

## Wider direct executions with the shared running-product fold

The radix digits are decomposed through dynamic arbitrary-width quotient and
remainder registers. The running product quotient advances with each digit,
and the phase-produced factor pair passes through one shared prefix state with
the exact product closed on both sides. These binaries were built and invoked
directly with phase base 2 and radix 3. Every completed run returned FDE T with
both nesting supports true.

| Mersenne exponent m | N decimal digits | phase observations | phase winding | Shor close | dual closure | total in-process |
|---:|---:|---:|---:|---:|---:|---:|
| 3217 | 971 | 809 | 4.613 ms | 4.218 ms | 4.913 ms | 15.274 ms |
| 19937 | 6005 | 9973 | 945.780 ms | 1.664 s | 151.990 ms | 2.794 s |
| 44497 | 13398 | 2786 | 692.183 ms | 2.259 s | 729.183 ms | 3.700 s |
| 86243 | 25965 | 86247 | 65.670 s | 265.125 s | 2.709 s | 335.296 s |

Each modulus is `257 × (2^m − 1)`. The 25,965-digit executable completed its
phase winding and both closures before the stop request arrived; its 335.296 s
runtime records the scale reached by this run.

## Semiprime terminal

Under the input promise that `N` has exactly two prime factors counted with
multiplicity, the membrane terminates at a canonical proper pair `P ≤ Q` when
both nested closures agree and `P × Q = N`. Each entry exceeds one. The promise
then forces each entry to contain exactly one prime factor, so the pair is the
unique prime factorization, including `p × p` squares. This removes a separate
primality pass from the terminal path.

The promise is part of the input contract, not a conclusion of product closure.
For a general composite, an exact proper pair can still be composite, as in
`210 = 14 × 15`; the terminal condition alone does not certify semiprimality.

## Minimum factor widths for new trials

Every new semiprime trial uses one prime factor with at least five decimal
digits and the other with at least ten decimal digits. Earlier rows are retained
as historical measurements and do not qualify as new trials under this floor.
The executable receives only the IMASM numeral for `N`; neither factor is a
baked input.

| N | decoded factors | phase observations | phase winding | Shor close | dual closure | total in-process |
|---:|---|---:|---:|---:|---:|---:|
| 39585639837697 | 12289 × 3221225473 | 31 | 30.168 μs | 10.837 μs | 34.171 μs | 163.339 μs |

The radix-3 binary was invoked directly. Its factor-register numeral words
decoded to the listed factors, and it reported FDE T with prefix-first and
product-first closure both true. Both factors passed primality checks.

The qualifying input `10007000070049 = 10007 × 1000000007` was also baked with
phase base 2 and radix 3. Its direct run remained silent and was interrupted
before closure, so it produced no factor result or completed timing. It is
recorded as an incomplete trial, not a successful factorization.

## Overhead profile and larger dynamic registers

These completed binaries embed only the IMASM numeral for `N`, use phase base
2 and lift radix 3, and were invoked directly. The table reports medians of
seven completed executions. `Through flush` includes writing the complete
frame report, which the earlier `total-in-process` measurement excluded.

| N digits | N | decoded factors | phase observations | phase winding | Shor close | prefix fold | output render | output write/flush | through flush |
|---:|---:|---|---:|---:|---:|---:|---:|---:|---:|
| 14 | 39585639837697 | 12289 × 3221225473 | 31 | 12.735 μs | 5.134 μs | 7.207 μs | 8.194 μs | 19.152 μs | 67.426 μs |
| 18 | 162129792744554497 | 786433 × 206158430209 | 38 | 14.897 μs | 5.722 μs | 8.583 μs | 10.063 μs | 20.618 μs | 74.581 μs |
| 23 | 21250649179513694453761 | 3221225473 × 6597069766657 | 42 | 26.200 μs | 11.500 μs | 11.300 μs | 11.700 μs | 21.700 μs | 99.800 μs |
| 27 | 580284393595165992175009793 | 3221225473 × 180143985094819841 | 59 | 32.383 μs | 10.335 μs | 14.075 μs | 11.123 μs | 19.587 μs | 105.024 μs |

Each completed run returned FDE T with both closure supports true. The emitted
factor-register words decoded to the listed factors. The 27-digit factors both
passed deterministic 64-bit primality checks. Their product exceeds the u64
range, while the membrane keeps the modulus, factors, and phase residues in
dynamically sized registers.

For the same 14-digit input, ten alternating invocations of the earlier and
optimized binaries reduced median in-process time from 90.21 μs to 47.11 μs
and output rendering from 34.07 μs to 8.18 μs. Captured process wall medians,
including launch and output, were 685.75 μs and 655.60 μs. Direct frame writing
removed the seven temporary frame trees and joined strings; a width-derived
capacity reservation avoids repeated output-buffer growth. The binary now
reports output-write/flush separately and reports total-to-flush.

On the 18-digit input, replacing packed-byte residue keys with dynamic
`BigUint` keys and one `HashMap` entry lookup reduced median phase-orbit time
from 12.755 μs to 8.884 μs across ten alternating runs. The radix fold now
streams each dynamic digit instead of materializing digit vectors. On the
27-digit input, twelve interleaved executions measured these lift radices:

| lift radix | prefix fold | through flush |
|---:|---:|---:|
| 3 | 14.736 μs | 112.290 μs |
| 10 | 9.259 μs | 100.674 μs |
| 16 | 7.740 μs | 101.704 μs |
| 256 | 6.515 μs | 97.392 μs |
| 4294967296 | 3.577 μs | 97.539 μs |

The 32-bit radix gives the shortest measured fold; its full runtime is within
0.15 μs of radix 256 in this sweep. All five radix binaries returned the same
factor registers and true dual closure. The fold comparison keeps phase base
and baked modulus fixed, so the change is isolated to the register digit width.

## Beyond the 27-digit floor

The next direct-run trial raises both factor widths substantially. The input
was encoded as an IMASM numeral before compilation; only that `N` word, phase
base 2, and lift radix `4294967296` were baked into the executable. The factor
values below were decoded from the emitted `P` and `Q` register words after the
run. Both are Proth primes, certified by the witnesses shown.

| N digits | N | factors (digits) | Proth witnesses | phase observations | phase winding | Shor close | dual closure | output render | output write/flush | through flush |
|---:|---:|---|---|---:|---:|---:|---:|---:|---:|---:|
| 256 | 4222978216202643879358776203723956984487864771371595739381318407303980253973317835720418236701436401727311635791396822723292392910122079788459348727875224286730683890750756540930477218788410269980206557306023861593174362539933039434147476890811995915288577 | 1983167906370745796855745924098313119485214048892843657853946385823217373708934037475567631319145854542773636075070074912769 (124) × 2129410325084785812156222093421888284239183673888627129730928317912029164394720478494700343887979195978796450018357943164652153208833 (133) | 11, 5 | 440 | 430.175 μs | 184.322 μs | T, both true | 65.584 μs | 80.089 μs | 946.024 μs |

All timing columns are medians across seven direct executions. The product was
checked against the baked decimal input, and the Proth tests were
`11^((p−1)/2) ≡ −1 (mod p)` and `5^((q−1)/2) ≡ −1 (mod q)`. Both factors
exceed the current minimum pair. The 32-bit radix run completed without build
warnings and emitted its report only after closure.

## Continued widening: phase-half close and overhead walls

These are direct executions of separately compiled, contained binaries. The
builder receives decimal `N`, a phase-base value, and radix `4294967296`; it
converts those values to IMASM numeral words before compiling. The executable
contains IMASM numeral words for the modulus, phase base, and radix; no factor
value is baked in. The factor expressions in the audit column below describe
the generated test semiprimes; they were not passed to the builder. For each completed case I
decoded the emitted `P` and `Q` numeral registers, checked that they equal the
test primes, and checked `P × Q = N` and FDE `T` with both closure supports.

| N digits | Executable | Test-input construction (audit only) | Decoded factors | Phase base seed / prewind | Observations | Direct median | Through flush | Result |
|---:|---|---|---|---|---:|---:|---:|---:|
| 7,627 | `phase_7628d_pre3_r32` | `(405×2^12556+1)(405×2^12763+1)` | 3,783 × 3,845 digits | 3 / N-derived half-width offset | 248 | 44.627 ms | 45.471 ms | T |
| 8,817 | `phase_8817d_tailwind42_r32` | `(15435×2^16502+1)(405×2^12763+1)` | 4,972 × 3,845 digits | 42 / `9·bits/16` | 110 | 38.706 ms | 31.885 ms | T |
| 10,020 | `phase_10020d_tailwind34_r32` | `(1463×2^20501+1)(405×2^12763+1)` | 6,175 × 3,845 digits | 34 / `123·bits/200` | 86 | 41.207 ms | 32.189 ms | T |
| 11,147 | `phase_11147d_tailwind21_r32` | `(1463×2^20501+1)(15435×2^16502+1)` | 6,175 × 4,972 digits | 21 / `9·bits/16 − bits/100` | 134 | 47.582 ms* | 48.323 ms | T |
| 13,317 | `phase_13317d_threshold31_r32` | `(1463×2^23713+1)(1463×2^20501+1)` | 7,142 × 6,175 digits | 31 / threshold `t=23705` | 92 | 49.310 ms | 47.167 ms | T |
| 16,179 | `phase_16179d_c2_t29900_r32` | `(2873×2^30009+1)(1463×2^23713+1)` | 9,038 × 7,142 digits | 2 / `t=29900` | 1,045 | 362.467 ms | 359.910 ms | T |
| 19,673 | `phase_19673d_compatible5_r32` | `(1463×2^41617+1)(1463×2^23713+1)` | 12,532 × 7,142 digits | 5 / threshold `t=41614` | 34 | 46.246 ms | 44.268 ms | T |

The direct medians use seven runs except the 11,147-digit row, which uses nine.
For direct timing, stdout was captured for the 7,627-, 8,817-, and
10,020-digit rows and redirected to `/dev/null` for the remaining rows. The
through-flush column is the membrane's own timer through `write_all` and
`flush`. The 16,179-digit factor registers were decoded from the emitted
output and match the two generated primes exactly. The reported prime
certificates are Proth witnesses: 7 for cofactor 405, 17 for cofactor 15435,
3 for cofactors 1463 and 2873.

The wider trials exposed and repaired these overheads:

* On the 1,483-digit control, Shor close recomputed a modular half-power even
  though winding already held the adjacent half-step residues. Passing those
  residues directly into the close reduced that close from about 27.6 ms to
  0.77 ms.
* On the 2,416-digit case, rendering the winding register as a decimal integer
  was the hidden ~16-second cost. Reporting its dynamic bit length instead
  reduced the run from 22.177 s to 5.979 s without changing the register.
* At 4,938 digits, a repeated runtime gcd only revalidated that the baked
  phase base was a unit. `vox coprime` now checks the original decimal inputs
  before baking, and the binary records the validated-unit marker alongside
  the encoded words. This moved the check out of the run while preserving the
  generic validation path for binaries built without that marker.
* At 7,627 digits, Euclidean gcd over large `BigUint`s took 40.9 ms in Shor
  close. Replacing that hot path with `num-integer`'s binary gcd reduced the
  close to about 3.2 ms; the same input then ran in 44.6 ms directly.
* At 8,817 digits, prewinding beyond `N`'s full bit length removed the long
  transient but also removed the even component needed by Shor close, giving
  an odd-return rejection. Keeping a short residual 2-adic transient and
  selecting seed 42 yielded 110 observations and a 38.7 ms direct median.
* At 10,020 digits, seed 42 took 213 observations and 62.5 ms. Surveying
  phase-only seeds found seed 34 with 86 observations and a 41.2 ms median.
* At 11,147 digits, seed 42 had a 767-observation orbit. Seed 21 reduced it to
  134 observations; nine direct runs with stdout redirected had a 47.6 ms
  median, and the membrane timer measured 48.3 ms through flush.
* At 13,317 digits, a prewind that was too deep again made the return odd.
  Sweeping the prewind against the phase collision and half-residue close
  found the `t=23705` boundary, preserving one 2-adic step. That produced 92
  observations and a 49.3 ms direct median.
* At 16,179 digits, the mixed odd cofactors raised the best tested phase cycle
  to 360 steps (468 observations including its transient); seed 2 completed
  the compiled membrane in 362.5 ms with both closure supports true. This is
  the mixed-cofactor wall. Pairing primes with the same cofactor 1463 reduced
  the phase return to 34 observations at 19,673 digits. The new Proth prime
  was found after 14,168 parallel candidate checks in 635 s; the serial scan
  through exponent 30,000 and the fixed-1463 scan through 60,000 found no
  earlier matching prime.
* At 19,673 digits, the compatible-cofactor run initially measured 69.0 ms:
  Shor close took 21.7 ms and the nested radix fold 18.0 ms. `radix_prefix_fold`
  now recognizes a power-of-two radix and uses dynamic bit masks/shifts for
  digit extraction, carry updates, and the second nesting; dual closure fell
  to 1.79 ms. Frame groups are now emitted as fixed-width hexadecimal joint
  states (one nibble for windows 2–4, one byte for windows 5–8), with the
  actual tail width retained. The new renderer test reconstructs every source
  bit at every window. Finally, the report stopped printing the full encoded
  base and modulus a second time; their baked IMASM words remain in the binary
  and the seven lossless frames carry the complete N stream. Output shrank to
  635,516 bytes and direct median fell to 46.246 ms; both closure supports
  remain true.

The next widening constraint is supplying another prime whose odd cofactor
has a phase period compatible with 1463. The fixed-cofactor scan through
60,000 produced no match. A broader follow-up candidate family was built from
odd cofactors dividing `2^90−1`, which keeps the same 90-step phase-period
bound; that screening pass did not yield a completed result in this run.

## EML-first contained binary widening

The single-value runner now bakes both the modulus and the phase base as
IMASM numeral words, then emits a silent, standalone `factor_eml_one` binary.
The phase base is supplied as an input (`./eml_factor_one.sh N [base]`), not
selected by a source-code literal. Release builds use `target-cpu=native` and
deny warnings. The initial contained executions below used GNU `timeout` at 60
seconds. The runner now executes the baked binary without a time limit. Both
largest base-2 cases were rebuilt from their N and base IMASM words and rerun
uncapped; each completed output was compared byte-for-byte with the encoded
known smaller prime factor. Here `M_e` denotes the Mersenne prime `2^e − 1`.

| Decimal digits of N | Factors (factor digits) | Baked phase base | Execution | Result |
|---:|---:|---:|---:|---|
| 4,249 | M4423 × M9689 (1,332 × 2,917) | 2 | 0.792 s | exact factor |
| 6,293 | M9689 × M11213 (2,917 × 3,376) | 2 | 1.405 s | exact factor |
| 12,535 | M19937 × M21701 (6,002 × 6,533) | 2 | 5.059 s | exact factor |
| 20,382 | M23209 × M44497 (6,987 × 13,395) | 2 | 16.477 s | exact factor |
| 31,964 | M19937 × M86243 (6,002 × 25,962) | 2 | 51.33 s | exact factor |
| 39,267 | M19937 × M110503 (6,002 × 33,265) | 2 | 49.93 s | exact factor |
| 39,357 | M44497 × M86243 (13,395 × 25,962) | 2 | 46.44 s | exact factor |
| 46,660 | M44497 × M110503 (13,395 × 33,265) | 2 | initial 65.93 s; uncapped rerun 69.73 s | exact factor in both runs |
| 59,227 | M86243 × M110503 (25,962 × 33,265) | 2, 3, 5, 7, 11, 13, 17 | base 2 initial 60-second cap; uncapped rerun 90.77 s; other bases remain capped | base 2 returned exact factor; other capped runs produced no output |

The encoded-base CLI path was also run end-to-end on the 39,267-digit case.
Conversion, baking, and compilation brought total CLI wall time to 83.42 s;
the silent contained execution completed within its internal one-minute cap,
and its output matched `M19937` byte-for-byte.

The 39,357-digit phase state previously held a growing residue map. Replacing
it with Brent winding fixes the resident orbit registers at three; the
large-width test verifies the return relation and both nested closures. A
native-CPU release build then reduced that case from over a minute to 46.44 s.
On the 59,227-digit case, the uncapped base-2 execution completed in 90.77 s
and returned the exact smaller factor word. This confirms the earlier exit 124
was the one-minute process cap, not a failed closure. Bases 3, 5, 7, 11, 13,
and 17 retain their earlier capped results and were not rerun. The base-11 and
base-17 full CLI runs took 105.51 s and 105.59 s respectively, including
baking and compilation before their contained runs timed out. The uncapped
base-2 timing measures the integrated carrier end-to-end; it does not isolate
phase-orbit work from its support reads.

The dynamic-base API has a small-number regression test for bases 2, 3, and 5,
rejects 1, and immediately closes when the supplied base itself shares a
proper divisor with N. The CLI wrapper's output remains silent until the
factor word is complete.

No compiler warnings appeared in the release builds or test runs. `cargo fmt
--all -- --check` currently fails on broad formatting differences throughout
the repository; I left those unrelated files untouched.
