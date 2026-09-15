# Membrane verification tables, 2026-09-15

87 tests passed, zero failures, using `cargo test --release --all-targets`.
Six payloads were compiled, lifted and executed through Vox, then each saved
module was executed five more times. All 36 VM executions exited zero and
matched native stdout and stderr exactly.

## Regression suite

| Target | Passed | Failed |
|---|---:|---:|
| Library, including factor routing and instruction definitions | 47 | 0 |
| ABC | 16 | 0 |
| Binomial-row GCD | 3 | 0 |
| Divisor ring | 3 | 0 |
| Landau | 3 | 0 |
| Cumulative LCM | 3 | 0 |
| Resident QFT circuit | 1 | 0 |
| Schütte | 3 | 0 |
| Shor/QFT | 4 | 0 |
| Distinct triple sums | 3 | 0 |
| Vox CLI | 1 | 0 |
| **Total** | **87** | **0** |

`factor_bench`, `factor_one` and `perfect_one` also compiled as test targets;
they contain no target-local tests. The library includes factor controls.
Full record: [regression log](membrane_test_tables_regression.log).

## Complete IMASM execution

Median and range use five sequential reruns of each saved module. Times are
whole-process wall time from `/usr/bin/time`, at 0.01-second resolution, including
module reading, loading and execution. Compilation and lifting are excluded.
These are not kernel-only timings or native/Vox speedup ratios.

| Membrane and complete IMASM | Baked inputs | Median | Range | VM steps | Output checks |
|---|---|---:|---:|---:|---:|
| [Binomial-row GCD](membranes/binomial_one/0_1_2_8_9_12_25_27_30_81_120_81/payload.elf.imasm) | 0,1,2,8,9,12,25,27,30,81,120,81 | 0.14 s | 0.14–0.15 s | 99,220 | 6/6 |
| [Cumulative LCM](membranes/lcm_one/0_1_10_20_40_80_100_10/payload.elf.imasm) | 0,1,10,20,40,80,100,10 | 0.12 s | 0.12–0.13 s | 70,751 | 6/6 |
| [Divisor ring](membranes/divisor_one/1_2_97_360_65536/payload.elf.imasm) | 1,2,97,360,65536 | 0.30 s | 0.30–0.31 s | 246,940 | 6/6 |
| [Landau](membranes/landau_one/0_1_10_15_30/payload.elf.imasm) | 0,1,10,15,30 | 0.10 s | 0.10–0.12 s | 54,631 | 6/6 |
| [Schütte](membranes/schutte_one/7_1_2_3/payload.elf.imasm) | 7 vertices; k=1,2,3 | 0.09 s | 0.08–0.10 s | 35,160 | 6/6 |
| [Shor/QFT](membranes/shor_one/2_21_12/payload.elf.imasm) | a=2, N=21, 12 qubits | 1.89 s | 1.85–1.97 s | 1,958,437 | 6/6 |
| [Rep-tiling](membranes/reptiling_one/1_2_3_4_5_6_7_8_9_10_18_20_25_26_100_1000_6/payload.elf.imasm) | 1,2,3,4,5,6,7,8,9,10,18,20,25,26,100,1000,6 | 1.80 s | single verified run | 2,717,665 | 1/1 |

Each module directory retains `native.stdout`, `native.stderr`, the first
`vox.stdout`/`vox.stderr` pair and empty `output.diff`. Five reruns are retained
as `recheck-1` through `recheck-5`, each with `.stdout`, `.stderr` and `.time`.

## Boundary and numerical results

| Control | Verified result |
|---|---|
| Empty binomial interiors | h(0)=h(1)=0 |
| Prime powers versus mixed factors | h(8)=2, h(9)=3, h(25)=5, h(30)=1, h(81)=3 |
| Repeated/descending input | Repeated row 81 and final LCM query 10 agree with their earlier readouts |
| Wide LCM | LCM(1..80)=32433859254793982911622772305630400 |
| Checked overflow | LCM(1..100) reports `overflow_u128`; subsequent query 10 still returns 2520 |
| Landau values | g(0)=g(1)=1, g(10)=30, g(15)=105, g(30)=4620 |
| Schütte graph | n=7 passes k=1 and k=2; k=3 emits counterexample mask 7 |
| Shor recovery | Period 6; factors 7 × 3 = 21 |

Native agreement checks the lifted execution against the same compiled
algorithm. Independent mathematical/control comparisons are in the regression
tests; native agreement alone is not an independent proof of the algorithm.
