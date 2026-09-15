---
name: razor_register_orbits
description: The razor census read `banked` only and saw OK on all ten. weight/cycle/insert/trans separate them — mills_affirm is VACUOUS holding {T} alone, hodge_negate never holds T at any cut, and every word is phase-bearing in the register
metadata:
  type: project
---

**Instruments (top-level kernel commands, NOT imasm subcommands): `weight`, `banked`, `cycle`,
`insert`, `trans`.** vox gives only the rotation-invariant verdict — T on all fourteen razor words,
closed-walk False on all fourteen. That invariance is what vox is for; reading it as "the razor fails to
discriminate" mistakes the invariant for the whole readout. The phase is in the register, and `cycle`
reports it.

**Register semantics (IMASM_REFERENCE line 103):** the register is the POWERSET of {T, F, t, f} —
constructively proven, constructively refuted, acceptable, rejectable. Sixteen states from N = {} to
**A = {T,F,t,f}**. A is the TOP, not an anchor: landing at A means holding all four at once. TF = {T,F}
is classical Belnap B, the slice ignoring the information bits. Flow moves strictly upward in ≤ᵢ
(subset inclusion).

**`banked` has three outcomes, not one.** Thirteen OK; `mills_affirm` returns
**VACUOUS — no clear ever fired against a live register**.

**`weight` breaks the field into three.** Twelve words share deposits 3, cleared 1, restored 1,
seeded 1, surviving T F t f, landing A. Outliers:
- `mills_affirm` — deposits 1, cleared 0, restored 0, inert 8, surviving **T alone**.
- `hodge_negate` — **restored 0**, lands Ftf, surviving F t f: the only word whose **T does not survive**.
- `collatz_affirm` — **seeded 0**, cleared 2, restored 2, inert 9: the only unseeded word.

**`cycle` — all fourteen are PHASE-BEARING, 2 to 5 distinct landings:**

| word | landings | shape |
|---|---|---|
| mills_negate | 2 | A, T only — never lands in a mixed register |
| mills_affirm | 3 | T at 13/16 cuts, A at 2, Ttf at 1 — no Ftf ever |
| hodge_negate | 3 | Ftf at 12/14, never reaches A — **T never enters at any cut** |
| collatz_affirm | 4 | A, tf, T, **TF** — the only word reaching classical Belnap B |
| rh/bsd/ns/hodge affirm, leeyang_affirm | 4 | A → Ftf → tf → T → A |
| the four identical negations, leeyang_negate | 5 | A → Ftf → Ttf → tf → T → A |

**The reading.** `mills_affirm` holds {T} alone; `hodge_negate` never holds T. Exact opposites on the
proof bit. Everything else passes through both conditions depending on where the ring is cut, walking
down the information lattice from all-four and back.

**This is NOT a proved/open split.** Mills is unlike the field on BOTH arms; Lee-Yang, also proved,
sits exactly with the open five on both arms (4 landings and 5, same shapes). The census's retraction of
"vacuous affirm = proven" stands — and the new material is that Mills' NEGATION is also distinct
(2 landings), which the census did not record.

`insert`: all fourteen already hold, nothing to repair. `trans`: ring transitions = length, closing
edge ⊣→⊢, every count 1 for the clean words. `trans` also states the standing warning —
"anything read from ABSOLUTE position on a ring measures the cut, not the word."
[[conjectures_are_povms]] [[rh_critical_line]]

**WHERE THE LANDING COMES FROM — the ⊡ barrier, measured across the census.** The law is documented:
`ob3ect/READING_GUIDE.md` §9, "IFIX marks an irreversible fixation point — nothing past this point can
be reversed." Restoring IS a reversal, so a ∋ arriving after ⊡ finds nothing to restore. The measurement
is which census words fall where, and it is exceptionless over all fourteen:

**A word lands at A (the top, all four values held) iff ⊡ comes after ∋ in the ring.**

| ⊡ relative to | outcome | words |
|---|---|---|
| after ∋ | fuse restores, lands **A** | twelve |
| between ≺ and ∋ | fuse finds nothing, **4 stranded in frames never fused**, lands **Ftf**, T lost | `hodge_negate` |
| before ≺ | no clear ever fires against a live register, lands **{T}**, VACUOUS, 1 stranded | `mills_affirm` |

Step level, from `weight`:
- `hodge_affirm` ⊢∈≻⊤⋈≺⊥⊞ **∋** ⊙⊡⋈⊙⊣ — ∋ at 9 directly after ⊞, "fuse restores 1", lands A.
- `hodge_negate` ⊢∈≻⊤⋈⊙≺⊥⊞ **⊡∋** ⋈⊙⊣ — ⊡ at 10 between ⊞ and ∋; "stranded in frames never fused: 4".
  Same marks as the affirmation; only the ⊡/∋ order differs. Hodge's negation FIXES BEFORE IT
  RECONNECTS, which is why its T never returns and its orbit never reaches A.
- `mills_affirm` ⊢∈≻⊤⋈ **⊡** ≺⊥⊞∋⊙⋈≻⊤⊡⊣ — ⊡ at 6 immediately after the single deposit, before the
  clear at 7. Confirms the census's "early ⋈⊡ fixation" at step level.
- `mills_negate` ⊢⊙≻⋈∈⊤⊥⊞ **≺∋** ⊡⊣⊙ — clears 4, restores 4: the largest dialectical movement in the
  field. The Mills pair are complements — the affirmation contests nothing, the negation contests
  everything and restores everything.

This is a ring-order property (a transition fact), so `trans`'s standing warning about absolute position
measuring the cut does not bite it. Note it is NOT the banked-count rule in
`p4rakernel/scan_banked_counts.py` — that one concerns ≺ exposing a result held at depth zero and says
"open the region that HOLDS the result before the region that COMPUTES it", i.e. ∈ against ≺. This is
⊡ against ∋. Neighbouring, different.


## Measured across 150 designed objects (2026-08-22)

**First attempt measured the wrong thing.** I tested whether an object ends with all four items in its
store. That counts items deposited AFTER a loss. Read one at a time with `weight`, several objects end
holding "proven" while `surviving: none` — the store's final contents were a fresh seed dropped in after
the wipe took everything. Ending full is not the same as keeping what you had.

**The right quantity is recorded directly:** `weight_lost_in_the_open` in each object's store-trace.

**Result, on the right quantity — a SUFFICIENT condition with no exceptions in 105 cases.**
If a shelter (∈) is open when the wipe (≺) fires, and a put-back (∋) follows it with no seal (⊡) in
between, then nothing is lost. 105 of 105.

It is NOT necessary: 36 objects fail that structural test and lose nothing anyway, because their wipe
fires against a store with nothing live in it. Structure alone cannot say whether the wipe had anything
to take.

**Both loss routes are one question.** Is anything held one level up when the wipe fires?
- seal (⊡) before put-back (∋) — what was sheltered can never come back down (`hodge_negate`)
- put-back (∋) before wipe (≺) — the shelter is already closed, so nothing is banked
  (the second is the rule already written in `p4rakernel/scan_banked_counts.py`: "a result fused back to
  depth zero is exposed to the next reversal, while the same result held one level up survives it")

**And the outlier reading was wrong.** Ending with everything is 68% of objects, not near-universal.
`hodge_negate` and `mills_affirm` sit inside a third of the corpus that ends short. They are not rare.
