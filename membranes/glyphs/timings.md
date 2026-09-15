# Glyph-only word timings

Five sequential executions per word, 2026-09-15. All 30 executions exited zero
inside Vox and matched the previously native-verified stdout/stderr exactly.

| Word | Median | Minimum | Maximum | VM steps | Peak RSS, maximum (KiB) |
|---|---:|---:|---:|---:|---:|
| [Binomial](binomial.glyphs) | 0.33 s | 0.31 s | 0.81 s | 98,978 | 114,048 |
| [LCM](lcm.glyphs) | 0.29 s | 0.29 s | 0.32 s | 70,563 | 114,304 |
| [Divisor](divisor.glyphs) | 0.50 s | 0.49 s | 0.55 s | 246,788 | 127,232 |
| [Landau](landau.glyphs) | 0.28 s | 0.28 s | 0.30 s | 54,503 | 114,560 |
| [Schütte](schutte.glyphs) | 0.26 s | 0.25 s | 0.26 s | 35,062 | 113,792 |
| [Shor](shor.glyphs) | 2.13 s | 2.12 s | 2.18 s | 1,958,339 | 125,568 |

Wall time includes process startup, file reading, glyph decoding, module parsing,
and execution. Compilation and glyph generation are excluded. File caches were
not flushed. These are fresh processes, not resident activation timings or
cold-storage benchmarks. `/usr/bin/time` reports time at 0.01-second resolution.
The slower first binomial run is retained, not discarded.

Raw elapsed seconds and maximum RSS in KiB are stored in
`NAME.timing-1.time` through `NAME.timing-5.time`; corresponding `.stdout` and
`.stderr` files preserve each execution. Step counts exclude glyph decoding
and parsing, which occur before the instruction loop.

Measured command:

```sh
/usr/bin/time -f '%e %M' ./target/release/vox run membranes/glyphs/NAME.glyphs
```

| Word | Run 1 | Run 2 | Run 3 | Run 4 | Run 5 |
|---|---:|---:|---:|---:|---:|
| Binomial | 0.81 | 0.33 | 0.33 | 0.32 | 0.31 |
| LCM | 0.32 | 0.29 | 0.29 | 0.29 | 0.29 |
| Divisor | 0.52 | 0.49 | 0.50 | 0.49 | 0.55 |
| Landau | 0.28 | 0.29 | 0.28 | 0.30 | 0.28 |
| Schütte | 0.26 | 0.25 | 0.26 | 0.26 | 0.26 |
| Shor | 2.18 | 2.12 | 2.13 | 2.12 | 2.16 |
| Rep-tiling | 1.95 | — | — | — | — |

All run values above are seconds.
