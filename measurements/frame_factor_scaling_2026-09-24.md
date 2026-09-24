# Baked frame-factor scaling run

Each executable was compiled from `frame_factor_build.sh <N> <frame-width>` with `RUSTFLAGS=-D warnings`, then run directly. The only baked input is the compiler-produced `cell-binary` IMASM numeral for N. The reported time is the direct executable's wall time, including its final report output. Every valid fixture returned the listed pair and printed `closure closed`.

| N decimal digits | Factor bit widths | N | Returned factors | Frame width | Runtime | Binary |
|---:|---:|---:|---:|---:|---:|---|
| 11 | 17 × 17 | 10002200057 | 100003 × 100019 | 8 | 0.093 s | `frame_factor_1532da89708f509c4d96` |
| 13 | 20 × 20 | 1000036000099 | 1000003 × 1000033 | 8 | 1.017 s | `frame_factor_9e9dc21fafae368e5d04` |
| 14 | 14 × 30 | 10007000070049 | 10007 × 1000000007 | 8 | 0.037–0.040 s (8 runs) | `frame_factor_d62a36cacb88adfbfe6d` |
| 15 | 17 × 30 | 100003000700021 | 100003 × 1000000007 | 8 | 0.271 s | `frame_factor_449a0b5466066f9c72ea` |
| 16 | 20 × 30 | 1000003007000021 | 1000003 × 1000000007 | 8 | 2.209 s | `frame_factor_8ea05ac35e1f608afb60` |
| 17 | 24 × 30 | 10000019070000133 | 10000019 × 1000000007 | 8 | 21.854 s | `frame_factor_e57d7d5e1842f3c4a4ec` |
| 23 | 14 × 61 | 23074570993201435367657 | 10007 × 2305843009213693951 | 8 | 0.099 s | `frame_factor_8cdbe764c7516ab99526` |
| 24 | 17 × 61 | 230591218450397036181853 | 100003 × 2305843009213693951 | 8 | 0.734 s | `frame_factor_3f74d4c490d3785468f5` |
| 31 | 14 × 89 | 6194032986564400205457768044777 | 10007 × 618970019642690137449562111 | 8 | 0.161 s | `frame_factor_811624a191ec7e89ce52` |
| 43 | 14 × 127 | 1702602822888915601938994848284852246010089 | 10007 × 170141183460469231731687303715884105727 | 8 | 0.295 s | `frame_factor_212df6193784cbada649` |
| 43 | 14 × 127 | 1702602822888915601938994848284852246010089 | 10007 × 170141183460469231731687303715884105727 | 2 | 0.307 s | `frame_factor_94687891bcd06f2be301` |

## Readout

The route closes on inputs up to 43 decimal digits when the smaller factor is 14 bits. Increasing the larger factor from 30 to 127 bits raised runtime from 0.037 s to 0.295 s. The smaller-factor width is the sharper cost driver in this set: at 24 bits, the 17-digit input took 21.854 s. Balanced 20-bit factors took 1.017 s for the 13-digit control.

The run supports exact factor-pair closure, not a primality certificate. These fixtures were constructed from known primes. A mistyped candidate, `100003000210021`, was excluded: it is not `100003 × 1000000007` and the membrane returned `421 × 237536817601`. This is a useful reminder that exact product closure alone does not establish semiprimality.
