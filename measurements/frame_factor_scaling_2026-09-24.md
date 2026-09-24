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
| 43 | 14 × 127 | 1702602822888915601938994848284852246010089 | 10007 × 170141183460469231731687303715884105727 | 8 | 0.287 s | `frame_factor_212df6193784cbada649` |
| 43 | 14 × 127 | 1702602822888915601938994848284852246010089 | 10007 × 170141183460469231731687303715884105727 | 2 | 0.280 s | `frame_factor_94687891bcd06f2be301` |

## Readout

The route closes on inputs up to 43 decimal digits when the smaller factor is 14 bits. The 14-bit by 30-bit semiprime runs in 43 ms, below the 50 ms reference. Increasing the larger factor from 30 to 127 bits raised runtime from 43 ms to 287 ms. The smaller-factor width is the sharper cost driver in this set: at 24 bits, the 17-digit input took 29.859 s. Balanced 17-bit and 20-bit factors took 148 ms and 1.622 s respectively.

The inverse-convolution route reads each target cell from the active evaluation frame. It fixes the odd anchor `p₀=q₀=1`, shifts a candidate `pₖ` into the current support position, cancels `bitₖ(PQ) XOR pₖ` against the source cell to recover `qₖ`, then advances the carry-normalized product tape. Frame widths 2–8 are tested as lossless regroupings of the same source word. The semiprime specialization rejects shorter-factor candidates divisible by 3, 5, or 7 when the longer support still has at least eight unresolved cells; the corresponding negative residue reads are also applied at the terminal complementary factor.

The run supports exact factor-pair closure, not a primality certificate. These fixtures were constructed from known primes. A mistyped candidate, `100003000210021`, was excluded: it is not `100003 × 1000000007` and the membrane returned `421 × 237536817601`. This is a useful reminder that exact product closure alone does not establish semiprimality.
