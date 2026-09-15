# Membrane stress results

Date: 2026-09-15. The stress command exercised the complete static executable
pipeline: bake numeric IMASM inputs, compile, lift, run the native control, run
the lifted module in Vox, require an in-VM exit code of zero, and compare native
and Vox stdout/stderr exactly.

## Summary

| Membrane | Stress payload | Result | Vox time | VM steps |
|---|---|---|---:|---:|
| ABC | c=2,9,32,81,200,400,600,800,1000,1000,2,0 | PASS | fixed run exited 0 | 56,369,755 |
| Divisor ring | 1,2,3,97,360,65536,2^50,360,2,1 | PASS | 0.67 s | 542,838 |
| Shor/QFT | a=2,N=15,8 qubits | PASS | 0.22 s | 166,194 |
| Schütte | n=23, k=1,2,3,4,5,2,1 | PASS | 0.21 s | 198,583 |
| Landau | 0,1,2,10,15,30,45,45,1,0 | PASS | 0.18 s | 106,000 |
| Distinct triple sums | limit 24 | PASS | 10.82 s | 11,934,728 |
| Factor | 8051 | PASS | 7.48 s | 7,113,290 |
| Rep-tiling | 1,2,3,4,5,6,7,8,9,10,18,20,25,26,100,1000,1000,6,1 | PASS | 1.78 s | 2,732,851 |

All eight payloads produced exact native/Vox agreement for every requested
output, including repeats, descending values, wide values, an empty ABC cutoff,
and the Schütte counterexample case.

## ABC fix

The failing sequence was reproduced and then fixed in the instruction path. The
lifted x86 decoder was missing the SSE2 `pcmpgtd` and `pandn` opcodes. It skipped
the compare and lost instruction alignment at the following packed operation,
which caused the VM to jump into an unmapped guest allocation region. Both
instructions are now implemented in the decoder, Vox classifier, and VM:

- `pcmpgtd`: signed 32-bit lane comparison, all-ones or zero per lane.
- `pandn`: Intel semantics, `(~destination) & source`.

The focused SIMD tests pass, and the original controls now complete:

| Case | Result |
|---|---|
| `abc 1 10 0` | PASS, `cutoff=0 empty`, 19,219 steps |
| `abc 1 10 0 2 9 32 1000` | PASS, exact agreement |
| `abc 1 10 1000 0` | PASS, exact agreement, 56,251,689 steps |
| Full stress sequence with zero last | PASS, exact agreement, 56,369,755 steps |

## Controls and records

Raw logs are in `stress_runs/`. The fixed matrix is recorded in the
`*_fixed.log` files, including `stress_runs/abc_fixed.log`.

The existing unit suite remains separate from this stress matrix. The focused
SIMD tests and the complete eight-membrane stress matrix now pass.
