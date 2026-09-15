# Seven-box object calculus

The 31-step resident membrane treats the search stages as objects, not as
independent unlabelled arrows:

```text
B1 parity ───────► B2 primality ─────► B3 Fermat ─────► B4 MPQS
  ▲                  ▲                   ▲               ▲
  │                  │                   │               │
B7 boundary ◄────── B6 folds ◄───────── B5 QS ◄─────────┘
```

For every `i < j`, the right rail is the composite

```text
f(i,j) = f(j-1) ∘ ... ∘ f(i)
```

and the left rail is

```text
r(j,i) = v(i) ∘ ... ∘ v(j-1)
```

The adjacent objects are:

| Step | Forward object map | Reverse/verification map |
|---|---|---|
| B1 → B2 | strip powers of two | parity check |
| B2 → B3 | pass the unfactored composite | primality check |
| B3 → B4 | pass residual to Fermat/MPQS boundary | close-semiprime check |
| B4 → B5 | pass residual to QS fallback | congruence-of-squares check |
| B5 → B6 | pass residual to remainder folds | smoothness check |
| B6 → B7 | multiply factors | retraction to remainder folds |

The source-level object paths are `forward_rail(i,j)` and
`reverse_rail(j,i)` in `factorization_31_membrane.rs`. The resident dispatcher
nests the decisive gates outside the Fermat arm, then MPQS and QS, before the
remainder folds and final product boundary.
