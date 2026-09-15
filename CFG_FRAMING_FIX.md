# Runtime control-flow framing correction

The four recurring F words came from `_Exit`, `_Unwind_GetDataRelBase`,
`_Unwind_GetTextRelBase` and `_Unwind_Resume`. All four remain in the audit and
now read T in all six membrane ELF files. No verdict rules were changed.

## Corrections

- Decoded instructions retain their actual fall-through address. Sorted-list
  adjacency is no longer treated as an execution edge across missing bytes.
- Exact known non-returning runtime contracts (`abort`, `_Exit`, `_exit`,
  `exit`, `__stack_chk_fail`) remove their call continuations. Unknown calls
  retain continuations; substring/name-fragment matching is not used.
- Iterative CFG traversal identifies real DFS feedback edges. Each gets an
  explicit loop frame, opened at the header and closed at the latch. Backward
  address order alone does not imply a loop: backward shared tails remain joins.
- Forward joins emit one fuse per excess incoming edge, retaining join arity.
- Reverse-postorder emission places shared tails after their contributing
  branches. Address-order emission had put fuses before the work inside the
  branch. Unreachable padding is not promoted to an execution path.
- Structural audit and `vox word` use the same framing implementation.

The corrected `_Exit` word is:

```text
⊢⋈⋈⊙⋈∈⋈⊙≺∋
```

Its frame represents the syscall-retry loop. The T verdict describes the
region's closure/work, not a guarantee that the retry loop terminates.
The old malformed word still yields F in the regression test.

## Installed-command verification

The corrected release binary was installed at
`/home/mrnob0dy666/.cargo/bin/vox`. These results were obtained using that
installed `vox`, not only a repository-local build:

| Membrane | T | B | N | F |
|---|---:|---:|---:|---:|
| Binomial | 147 | 921 | 1770 | 0 |
| LCM | 147 | 921 | 1770 | 0 |
| Divisor | 154 | 962 | 1872 | 0 |
| Landau | 147 | 922 | 1769 | 0 |
| Schütte | 147 | 919 | 1771 | 0 |
| Shor | 156 | 936 | 1816 | 0 |

All 96 release-target tests pass. Five new integration controls cover exact
`_Exit` bytes, backward shared tails/non-returning calls, address gaps,
three-way join arity, unchanged verdict rejection, and the four actual runtime
functions across all six binaries. Every one of those 24 runtime readings is T.

All six existing glyph-only words were executed again with identical stdout,
stderr and step counts. The executable module representation and payloads were
not rewritten for this framing correction.

Records: `cfg_framing_tests.log`, `cfg_framing_full_tests.log`,
`cfg_installed_*.audit`, `cfg_framing_*.stdout`, `cfg_framing_*.stderr`.

## Audit timing after the fix

Five fresh executions of the installed `vox` command per ELF, including file
read, decode, CFG walk, framing, verdicting and report formatting:

| Membrane | Median | Minimum | Maximum | Current fallback functions |
|---|---:|---:|---:|---:|
| Binomial | 0.10 s | 0.08 s | 0.14 s | 1,635 |
| LCM | 0.08 s | 0.08 s | 0.08 s | 1,727 |
| Divisor | 0.09 s | 0.09 s | 0.11 s | 1,786 |
| Landau | 0.09 s | 0.08 s | 0.09 s | 1,643 |
| Schütte | 0.08 s | 0.08 s | 0.08 s | 1,635 |
| Shor | 0.09 s | 0.09 s | 0.09 s | 1,773 |

The prior F=4 audit did not save wall-clock samples, so a numeric before/after
speedup cannot be claimed. The available structural comparison is favorable:
binomial fallback functions fell from 1,719 to 1,635, while correctness changed
from F=4 to F=0. Raw timing files are `speed_NAME_RUN.time` and reports are
`speed_NAME_RUN.audit`.
