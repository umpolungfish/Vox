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
| 10,000,019 × 10,000,079 | 100,000,980,001,501 | >90 s | not completed | TIMEOUT |

The first five output diffs are empty. The sixth native control completed and
reported the expected factors, but Vox did not complete before the bound. This
identifies the current scaling limit: the 31-slot dispatch membrane is closed,
but its factor search still performs trial division rather than a resident
sublinear semiprime arm.
