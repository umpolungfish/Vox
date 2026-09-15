---
name: erdos_straus_ladder
description: The Erdős–Straus ladder — where the surviving class stands, what is proved, and the next rung
metadata:
  type: project
---

Erdős–Straus, worked 2026-08-11 with the instruments first. All of it is in
`p4rakernel/p4ramill/Imscribing/Millennium/Erdos/StrausGreedyFamily.lean`,
sorry-free on propext / Classical.choice / Quot.sound, library green at 8592 jobs.
The reading instrument is `straus` in mOMonadOS (`straus <n>`, `sweep lo hi`,
`nest <n>`, `census lo hi`, `frontier lo hi`), and `nesting greedy` in `src/nesting.rs`.

**The ladder is complete, proved 2026-08-11.** Every representation of 4/n is a
rung: given 4/n = 1/x + 1/y + 1/z, take r = 4x − n and M = nx, then u = ry − M
and v = rz − M satisfy uv = M² and both congruences, since (M+u)(M+v) = r²yz =
M((M+u)+(M+v)). `ladder_complete` and `ladder_cross`, with
`threeUnit_of_closedAtRungSq` for the converse. So the rung and the divisor are a
COORDINATE SYSTEM on the representations, not a method that finds some of them,
and a value with no rung has no representation at all. `ClosedAtRungSq` carries
both congruences — the version taking the second as a hypothesis had the
hypothesis doing v_condition_free's work.

**The frame.** `nesting greedy` puts 4/n in the ATTRACTED bin with a finite
budget — greedy removal always arrives, so the conjecture is a claim about the
budget being three. `imasm check` on the two-fork form answers B and names the
remedy: COMMIT one arm, do not search it. Committing gives the ladder.

**The ladder.** For n = 4k+1 and any r ≡ 3 (mod 4), a = (n+r)/4 is an integer and
4/n − 1/a = r/(n·a), so the remainder's numerator is CHOSEN. r = 3 is the greedy
step; higher rungs are the committed arms.

**The criterion.** r/M = 1/b + 1/c iff some u with u·v = M² has r ∣ M+u and
r ∣ M+v — and the second condition is FREE (`v_condition_free`, in ZMod r). One
congruence, one divisor. `straus_practical` writes it with no division at all:
M = u·w makes v = u·w².

**The reduction.** `ClosedAtRung n r` := ∃ a u w, 4a = n+r ∧ n·a = u·w ∧ r ∣ n·a+u.
`straus_of_closedAtRung` turns that into the three unit fractions.
`EveryNClosed` → Erdős–Straus for the whole surviving class.

**Proved families**, each reading the rung off n rather than searching:
- `straus_one_shot` — n ≡ 5 (mod 8): u = 2 at rung 3, price ZERO. 3 ∣ n²+8 holds
  identically for 3∤n, and 2 ∣ M is exactly n ≡ 5 (mod 8). HALF the class.
- `straus_prime_family` — p ∣ n with p ≡ 3 (mod 4) is itself a rung; u = M and the
  two later denominators coincide.
- `straus_divisor_family` — r ∣ n(n+1): u = a, 4/n = 1/a + 1/b + 1/(nb).
- `straus_n_family` — r ∣ n(n+4): u = n, and c = a·b.
- `straus_of_divisor` — the greedy rung, d ≡ 2 (mod 3) dividing n(n+3)/4.

**Measured.** 5 ≤ n ≤ 200000, 33333 values in the class: every one closes.
Rung ceiling, measured with the CURRENT criterion (u ∣ M², cap 1024): over
5 ≤ n ≤ 4·10⁶ the least closing rung NEVER EXCEEDS 71 (first at 1430641); records
r = 3, 7, 11, 23, 31, 59, 71 at n = 5, 49, 1129, 1201, 21169, 118801, 1430641,
ratios 3.0, 3.5, 2.2, 4.6, 2.6, 3.3, 2.1, and the 2–4 million block tops out at
55 with ratio 1.4. The ratio is largest at SMALL n,
so C·n^(1/4) fits with C just over 4.6, the binding constant coming from n=1201.
`straus ceiling <lo> <hi>` reads it; `RungGrowthFourthRoot` states it. The older
figures (51 at 35809, 75 at 196561) came from the u ∣ M criterion and are
superseded — with M² those values close lower. `straus census` reads 50.2% one-shot / 49.8% iterated / 0% dead.
n = 2521 is the one value needing u ∣ M² rather than u ∣ M; it closes at rung 23.

**The price-zero layer, 2026-08-11.** The marker said the next rung for
n ≡ 1 (mod 8) was u = 2 at rung 11, on the grounds that n² ≡ 3 (mod 11) is a QR.
It is a QR, but u = 2 needs 2 ∣ M², and a = (n+11)/4 is odd for n ≡ 1 (mod 8),
so that route is closed. The real next rung was one level up: the whole family of
monomial divisors u = nⁱaʲ, which admissibility caps at i,j ≤ 2. Using 4a ≡ n
(mod r), the congruence r ∣ M+u becomes 4ʲ + 4n^(i+j−2) ≡ 0, and the nine pairs
give exactly three live outcomes — r ∣ n, r ∣ n+1, r ∣ n+4 — with the rest
demanding r ∣ 8, r ∣ 5 or r ∣ n²+4, all dead for r ≡ 3 (mod 4). That is the
price-zero layer, and it is complete: no divisor built from n and a reaches
further.

`straus_of_priceZero`, `straus_scaling` (multiplicative descent) and
`straus_frontier_mod_24` are in StrausGreedyFamily.lean, sorry-free on the
standard three, library green at 8592 jobs. `straus frontier <lo> <hi>` is the
new instrument verb and `classify` now reports all three as one-shots.

**Measured, 5 ≤ n ≤ 200000.** 32062 of 33333 close at price zero (96.2%); the
census over 5..4000 moves from 50.2% one-shot to 93.4%. The 1271 left are all
n ≡ 1 (mod 24) — proved, not observed — and equivalently are the n where n, n+4
and (n+1)/2 are products of primes ≡ 1 (mod 4). Descent through a proper divisor
takes 639 more, leaving a frontier of 632, beginning 193, 313, 457, 673, 1009,
1153, 1201. The manuscript is ig-docs/manuscripts_erdos/erdos_straus_price_zero.

**The walk has a graded coordinate, and it is coverage.** Reading the rung walk
as a nesting needs a residual, and the obvious one — the least residue of M+u
from zero — collapses to {0, 1/r}: a bool wearing a decimal, which showed itself
as identical tables for 1201 and 2521. The graded quantity is the SET the
divisors reach, {M+u mod r : u ∣ M²}, whose size against r is a density. Over
the 162 frontier values below 20000 it separates: 0.734 mean at the closing rung
and never under 0.533, against 0.366 mean at a failing rung, over 0.533 only 3
times in 89. The walk is dissipative in that coordinate — the reached set grows
into the target. `straus defect <n>` reads it. Structurally the reached set is
the image of the primes of n·a in Z/r, so the rung closes when those primes
generate enough of it; that is now proved, below.

**The searched rung is a group condition, proved 2026-08-11.** The divisors of
M² reduce mod r to a reachable set R, closure is −M ∈ R, and M ∈ R always — so
the content is −1 ∈ H, the subgroup of (Z/r)* generated by the primes of M = na.
`straus_of_neg_one_divisor` is the clean half: t ∣ M with t ≡ −1 (mod r) closes
the rung by u = M·t, no exponent bookkeeping. It is not an equivalence, and
`SubgroupExceedsReach` names the reason — H allows any exponent, divisors of M²
cap them at twice those of M. The budget gap THINS as n grows — 19 of 379 rungs
walked below 50000 (5.0%), 21 of 850 to 200000 (2.5%), 15 of 974 to 400000
(1.5%): more primes in M, larger reachable set, so the group criterion and the
real one converge. What is exceptional is failing that way at EVERY rung: 2521 does — searched to
rung 4000, no M = 2521(2521+r)/4 carries a divisor at −1, so its divisor
genuinely lives in M². 196561 gets a cofactor at rung 75, 1201 at rung 23.
`straus cof <n> <maxrung>` reads the height.
Elsewhere the gap costs one rung. Every other failing rung fails because the
subgroup lacks −1. `straus budget` reads it. Library green at 8592 jobs, and the manuscript carries the
section.

**The cascade, 2026-08-11.** The frontier's witnesses are cofactors: M = u·w
with w ≡ −1 (mod r), so M+u = u(w+1). `closedAtRung_of_cofactor` is that, and at
the greedy rung it collapses to one prime — every frontier value has M ≡ 1
(mod 3) (`frontier_M_mod_three`), so a divisor at 2 exists exactly when a prime
of M sits at 2 (mod 3). `closedAtRung_three_of_prime` takes 873 of the 1271
frontier values by itself before the shift family was found. Cascading the same
test up the ladder leaves exactly two values that no cofactor of M reaches with r ≤ 51 — 2521 at rung
23, and 196561 at rung 27, where M = 7²·17·59·196561 and the divisor is 7⁴·17,
the fourth power existing only after squaring. Both are witnessed in Lean;
`straus cascade <lo> <hi>` reads it. So the exponent budget is exercised twice
below 200000 and the frontier is a cascade with a known shape, not a wall.

**The shift family, and the frontier is primes.** Any divisor d of n divides
M = n·a, so a rung r ≡ 3 (mod 4) dividing d+1 closes by the cofactor form —
`closedAtRung_of_shift`, read off the factorisation of n alone, and NOT of
monomial type, so the exhaustion theorem does not reach it. It halves the
frontier, 1271 → 638, and descent takes it to 624 of which 622 are prime; the
composites survive only when every factor does. `ShiftCovered` is in the Lean
frontier definition and in `on_frontier`. The cascade on the smaller set: 313 at
rung 3, 202 at 7, 55 at 11, 15, 8, 19, 2, 5, 3 — everything but 2521 and 196561
by rung 51.

**The rung reads off k·n+1, and the circularity is gone.** Every cofactor is
w = k·r − 1, so r = (w+1)/k, and w ∣ M becomes a condition on n alone: 4k·a =
k·n + w + 1, so w ∣ k·n+1 with gcd(w,4k)=1 forces w ∣ a. `closedAtRung_of_kshift`.
Each k needs only the factorisation of k·n+1 — nothing about a, which was what
made the divisor search circular. k=1 is the old divisor family. The reparametrization is exact — every
cofactor is k·r − 1 — but k reaches (M+1)/r, so k is the WRONG AXIS to search on;
read rungs and take k = (w+1)/r from each cofactor. Over the 624 frontier values
below 200000: least k ≤ 2 for 314, ≤ 8 for 565, ≤ 32 for 613, tail to k = 12813
at n = 66529, rung 135. Only even k occur since w must be odd where M is. With
rungs to 200, exactly ONE value below 200000 needs a divisor of M² rather than of
M: 2521. `straus kceiling <lo> <hi>` reads it. Out to 10⁶ the frontier thins — 486 in 200001–400000
(1.5%, 484 prime), 444 in 400001–600000 (1.3%, 440), 395 in 600001–800000 (1.2%,
393), 390 in 800001–1000000 (1.2%, 388) — each closed entirely by a cofactor of M
at a rung ≤ 200, two composites per block and each a product of frontier primes. Largest least-k anywhere: 15083 at n = 297889, rung 79. `straus kshift <lo> <hi> <K>` and `straus reach`
read it.

**Every Erdős manuscript boots, 2026-08-12.** `erdos <name>` in mOMonadOS runs
one walk per paper, `straus` among them: the price-zero layer covering 3248 of
3333 below 20000, and 2521 at rung 23 with no cofactor of M there. See
[[proofs_state_2026_08_10]] for the three walks that caught my errors.

**The nest of creatures, 2026-08-11.** Ten imscriptions in
`ob3ect/nests/straus/` as stepping stones to the rung bound, built on kilo with
poolside/laguna-s-2.1:free (the openrouter lane was 402; Lando supplied a KILO
key). Building them exposed two stale gates, both fixed: `validate_structural`
enforced Axiom C as a BICONDITIONAL where Lean's `ImscriptiveTopology` is
one-directional, and `SplitFuseReport.validate` demanded a PASS/FAIL where the
verdict is B4. `ob3ect/revalidate_straus.py` re-checks built artifacts against
the corrected gates without regenerating them.

**The squeeze.** `rung_three_residue`: a frontier value that survives rung 3
under the cofactor form has every prime of n(n+3)/4 at 1 (mod 3). With the
frontier condition that is four multiplicative constraints at once — n and n+4
from primes ≡ 1 (mod 4), n and (n+3)/4 from primes ≡ 1 (mod 3). That is the
statement a density argument attacks.

**The three statements, ordered.** Weakest: every n has some rung — by
`ladder_complete` that IS the conjecture on the class. Middle: every n has a rung
where M rather than M² carries a divisor at −1 — strictly stronger, and 2521
inhabits the gap. Strongest: `RungBounded B`, a closing rung below B n, which by
`straus_class_of_rungBounded` gives the conjecture outright and is consistent
with C·n^(1/4) on everything measured. The bound is the one to attack, since
below it the rungs are finitely many and each is decidable.

**The next rung.** A bound on the least closing rung is now a concrete statement:
how soon, walking r ≡ 3 (mod 4), does a rung arrive whose n(n+r)/4 carries a
divisor at −1. At rung 3 that is "some prime factor ≡ 2 (mod 3)", a density
statement about the primes of n(n+3)/4, and proving that density is bounded below
on the frontier is the shape a real bound would take. The M²-only cases are rare
enough to be enumerated — two below 200000 — so the cofactor form carries the
argument and the exponent budget is a correction term, not the obstacle.

**How this was found, and it matters.** The Fixed-Point Nesting Rule's cross-axis
line — conservative maps populate only {one-shot, no-closure}, the basin class is
empty — is what exposed the M² correction: a congruence test is conservative, so a
value that will not close is never "nearly" closing, and a persistent no-closure
points at the STABILISER SET being wrong. It was. Reaching for a search instead of
the rule cost hours; the rule cost minutes. See [[law_operations]] on instruments
first.

**Extension 2026-08-13 — covering progressions from `kshift`.** Added to
StrausGreedyFamily.lean, all green on the three standard axioms: `straus_mod20`
(+ `closedAtRung_three_of_mod20`) covers the whole class `n ≡ 17 (mod 20)` at
rung 3 via the `k=2, w=5` shift; seven more covering families
`straus_cover_mod{44,52,68,84,92,108,116}` from `(k,w,r)` triples the kernel's
`kshift` engine returns, each closing an infinite congruence class at a fixed
rung; and five frontier-prime witnesses `straus_{193,313,457,673,1009}` (the
values the price-zero + one-shot families miss), formalized straight from the
`straus <n>` iterated-rung output as `norm_num` checks. Pipeline: kernel emits →
Lean verifies. The open core is unchanged — `EveryNClosedSq`/`RungBounded`, the
conjecture on the surviving class, still a Prop. Covering families never sum to
the full class; that gap IS the open problem. `straus census 5..400` = 0
no-closure, 90.9% one-shot.

**2026-08-13 batches 2–3 + second file.** Straus now carries 125 covering-family (through batch 9)
theorems `straus_cover_mod{20..1620}` (each an infinite congruence class closed at
a fixed rung via `closedAtRung_of_kshift`, `(k,w,r)` from the kernel `kshift`
engine) and 55 frontier-prime witnesses `straus_{193..18313}` (norm_num from
`straus <n>`). Separately, `SquarefreePlusPowerOfTwo.lean` `counterexample_density`
now PROVES the stated `CounterexampleDensity` (≥ x/3528 non-representable below x,
via the injective mod-1764 `bad` family) — a stated open Prop closed, standard
axioms. Cross-file Prop hunt: `JointLogarithmicGrowth` (SubgroupOrders) is
ψ(n)/n→1 (PNT) + Landau, not tool-reachable; repArith/APFree/ClassesBipartite are
definitional and already used in proved theorems. Roster of all three repos'
tools in `ig-docs/tool_roster_three_repos.md`; per-file checklist in
`ig-docs/erdos_completion_checklist.md`.

**2026-08-13 coverage-bound advance.** Landed `qnr_pow_eq_neg_one` (Euler
criterion): at every odd prime rung r, a QNR prime factor p of M gives
p^(r/2)=−1, so −1 ∈ ⟨primes of M⟩ ⊂ (ZMod r)ˣ — the general form of the rung-3
p≡2(mod 3) criterion, verified, standard axioms. Reached via the ob3ect pipeline
(context ig-docs/straus_ctx/coverage_measured.md), which decomposed the coverage
bound as FSPLIT on "−1 ∈ subgroup" with the QNR arm affirmative. With
closedAtRung_of_full_coverage + the measured dissipative coverage walk (→1, closes
in finite budget by the Fixed-Point Nesting Rule), the ONE remaining frontier is
the exponent budget: produce a divisor ∏p_i^{f_i} (0≤f_i≤2e_i) of M² at −M by rung
B(n)~n^(1/4). Kernel-measured gap thins with n (19/379 <50000 → 15/974 <400000):
a counting/pigeonhole surjectivity threshold — coverage totals once #distinct
primes of M clears a bound relative to r. Next experiment.

**2026-08-13 exponent-budget arm.** Landed `negMReachable_of_primitiveRoot`: a
prime p|M that is a primitive root mod r (powers hit every nonzero residue) with
p^(r-1)|M² makes −M a divisor of M², i.e. NegMReachable — the kernel's exact
closing criterion. With qnr_pow_eq_neg_one this closes the single-prime case end
to end (a primitive root is a QNR). Verified, standard axioms. Remaining core of
RungBounded: the MULTI-GENERATOR case — powers of several primes of M jointly fill
(ZMod r)ˣ when no single prime generates. The pigeonhole/counting surjectivity
threshold (coverage totals once #distinct primes of M clears a bound vs r). Next.

**2026-08-13 multi-gen + second file.** Straus coverage now has 5 lemmas:
qnr_pow_eq_neg_one, negMReachable_of_{primitiveRoot,twoGen,multiGen},
prod_dvd_of_pairwise_coprime — the full pigeonhole coverage step (any pairwise-
coprime prime family of M covering (ZMod r)ˣ within budget closes the rung).
Remaining for RungBounded: the number-theoretic existence that SOME rung ≤ B(n)
has M's primes generating within budget. SEPARATE: landed lcmUpTo_ge_quadratic in
SubgroupOrders.lean (n(n-1) ≤ lcm(1..n), coprime consecutive divisors) — the
kernel `erdos <name>` walks compute formalizable cores for every checklist file
(schutte f(2)=7, mantel, ramsey33 R(3,3)=6, reptiling class, roth, kac, sdr,
lenz, syndetic, oddcycle, chordless); instruments-first, formalize each.
