# 31-step membrane: baked large-value tests

Each value was compiled into `factorization_31_one`, executed natively, lifted
to IMASM, executed by Vox, and compared byte-for-byte with the native output.
The reported time is Vox execution only.

| Baked N | Resident result | Vox time | VM steps | F verdicts |
|---:|---|---:|---:|---:|
| 4,294,967,291 | prime | 0.35 s | 434,704 | 0 |
| 1,152,921,504,606,846,976 | 2⁶⁰ = 2 × 2 × 2 × 2 × 2 × 36,028,797,018,963,968 | 0.08 s | 18,827 | 0 |
| 1,000,036,000,099 | 1,000,003 × 1,000,033 | 4.98 s | 6,519,162 | 0 |
| 18,446,744,073,709,551,615 | 3 × 5 × 17 × 257 × 641 × 439,125,228,929 | 0.40 s | 450,426 | 0 |

All four runs exited 0, reconstructed the same output as native execution, and
closed the final product boundary. The artifacts and `output.diff` files are
under `membranes/factorization_31/large_<N>/`.
