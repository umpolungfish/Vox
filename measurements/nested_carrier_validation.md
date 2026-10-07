# Nested carrier validation

The native nine-stage carrier dispatches WITNESS, POWER, EXTRACT, SQUFOF,
P_MINUS, P_PLUS, LEHMAN, ECM, and FIX in that order. Its eight split/rejoin
frames are nested:

```text
⊢∈⊤≺⊥∈⊤⊞⊥∈≻⊤≺⊥⊞⋈∈⊤≺⊞⊥∈⊙⊞⋈∈⊙≺⋈∈≻⋈⊤⊥∈⊙≻⋈∋∋∋∋∋∋∋∋⊙⊡⊣
```

The ordinary execution-projection audit checks the word against all nine
dispatched stages. Removing SQUFOF fails that audit. The executor compares the
source tape returned by each executed stage with its bound source. A mismatch
closes the carrier and clears its selected output. Single-stage descent uses
the same check without a minimum tower depth.

The complete-membrane tests exercise forward/reverse tape maps independently.
The transformed-register tests preserve widths through 4,194,304 cells; their
large-register readout uses tape widths to avoid repeated decimal conversion.
The sieve tests include the pending polynomial multiplier change and exact
divisor/product checks.

Validation commands, all passing:

```text
cargo test --release --lib morphism_factor::tests -- --test-threads=1
cargo test --release --lib complete_membrane::tests
cargo test --release --lib perfect_membrane::tests -- --test-threads=1
cargo test --release --lib sieve:: -- --test-threads=1
```

These run 36 carrier tests, four complete-membrane tests, three transformed
register tests, and 17 sieve tests. The carrier control closes 8051 and verifies
its returned pair through the tape product check.

The current large-source target is:

```text
233108530344407544527637656910680524145619812480305449042948611968495918245135782867888369318577116418213919268572658314913060672626911354027609793166341626693946596196427744273886601876896313468704059066746903123910748277606548649151920812699309766587514735456594993207
```

Recorded attempts before this corrected nine-stage mapping reached their
240-second carrier and 600-second source-routed timeouts without returning
factors. The bounded moat resolver exhausted 1,000,000 nodes on the same source.
Those attempts do not measure the corrected mapping. The target remains active
for execution diagnostics.
