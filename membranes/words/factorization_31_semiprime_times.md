# 31-step membrane: increasing semiprime timings

Each product was baked into a fresh ELF, run natively, lifted to IMASM, run by
Vox, and compared against native output. Vox time is execution-only. The final
case was bounded at 90 seconds.

| Factors | N | Vox time | VM steps | Result |
|---:|---:|---:|---:|---|
| 101 × 103 | 10,403 | 0.08 s | 11,032 | PASS, F=0 |
| 1,009 × 1,013 | 1,022,117 | 0.07 s | 17,133 | PASS, F=0 |
| 10,007 × 10,009 | 100,160,063 | 0.11 s | 76,120 | PASS, F=0 |
| 100,003 × 100,019 | 10,002,200,057 | 0.54 s | 663,220 | PASS, F=0 |
| 1,000,003 × 1,000,033 | 1,000,036,000,099 | 4.56 s | 6,519,186 | PASS, F=0 |
| 10,000,019 × 10,000,079 | 100,000,980,001,501 | 20.72 s | 18,482,703 | PASS, F=0 |

All six output diffs are empty. The sixth case was previously bounded out at 90
seconds. After nesting the parity and primality gates outside the close-
semiprime Fermat arm, it closes in 20.72 seconds under Vox; MPQS/QS remains
behind that arm for distant composites.

## Compact resident ABI

The same 100,000,980,001,501 case was then baked into the `no_std` resident
payload (`factorization_31_resident`): direct syscalls, no libc startup,
allocator, formatter, or panic runtime. Vox completed it in **0.00 s** at the
timer resolution, using **1,943 VM steps**, with `F=0`. The payload returned
the factors in folded hexadecimal form:

```text
0x989693 x 0x9896cf
```
