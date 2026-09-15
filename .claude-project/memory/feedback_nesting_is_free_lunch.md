---
name: feedback_nesting_is_free_lunch
description: "Nest properly and the cost collapses to the min over arms, not the sum — free lunch, or paid to eat lunch when the cheap arm is also the decisive one"
metadata:
  node_type: memory
  type: feedback
  originSessionId: 9abfc284-9894-4667-b9ad-a2ff74df6f6f
  modified: 2026-09-15T00:01:01.923Z
---

Lando, 2026-09-14: "when we nest properly, we get free lunch. or we get paid to
eat lunch."

The principle for arm ordering in a membrane. When each arm fails fast on the
shapes it does not fit, a sequential or_else fallthrough is not additive: the
total cost collapses to the MIN over arms, not the sum. That is the free lunch —
adding the outer arm costs nothing on the inputs it cannot close, because it bows
out at once and the next arm pays its own price and no more.

Paid to eat lunch is the stronger case: when the cheap decisive arm sits
outermost and it IS the right arm for that input, the whole factorization comes
out cheaper than the fallback would have been. The right arm on top does not just
avoid a cost, it wins the input outright.

**Why:** it is the test for whether a nesting is done right. If nesting an arm
makes some input SLOWER, the order is wrong (a slow arm is in front of a fast
decisive one). Correct nesting never regresses any input and improves the ones
its outer arm fits.

**How to apply:** the HARD factoring tier is the worked example
([[vox_carrier_constructor]]). Bounded QS outermost, carrier rho fallback, Dixon
last. Measured both halves: 80-bit balanced semiprime — carrier-rho alone 46.59s,
QS-first-then-carrier 46.17s, so the failed QS pass is free (free lunch). 72-bit —
QS-first 1.56s vs carrier-first 15.99s, so QS on top made it ~10x cheaper (paid to
eat). The wrong order (carrier before sieve) regressed the 72-bit; the regression
IS the signal the nesting is inverted. See [[feedback_build_in_vox_not_rust]],
[[law_an_instrument_is_a_question]], commit eb88804.
