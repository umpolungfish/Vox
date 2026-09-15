---
name: proofs_state_2026_08_10
description: "Where the p4rakernel proof work stands as of 2026-08-12 — axioms demoted, Erdős corpus complete, seventy-three unsolved problems stated"
metadata: 
  node_type: memory
  type: project
  originSessionId: 6b8f4402-2e85-481a-b103-b6b2551121eb
  modified: 2026-08-12T15:30:00.000Z
---

Eight Grammar axioms addressed in p4ramill on 2026-08-10, in three defect
shapes. Knowing the shapes matters more than the list, because each recurs.

**A free variable in the consequent.** Axioms A, B, C, D — each said "X requires
Y" with BOTH values universally quantified, so the antecedent never touched the
value named in the conclusion and the axiom asserted that conclusion of
everything. Now decidable predicates on an Imscription, which reads the field
off the structure: `ImscriptiveClosure`, `ImscriptiveTopology`,
`WindingNeedsChirality` in Primitives/Imscription.lean,
`InfiniteMemoryNeedsSlowKinetics` in Decomposition.lean. Named Imscriptive, not
Holographic — holographic is the STATIC subset, a boundary encoding a bulk it
does not write.

**A vacuously true hypothesis.** `bowtie_max_three_layers` and
`box_irreducible` — `validTemporalDecomposition`'s wool branch does not mention
n at all, since infinite memory means arbitrary decomposition, so the hypothesis
holds for every n. Outcomes differed and that is the useful part: excluding wool
the bowtie bound is now a THEOREM proved by cases, while box irreducibility does
not follow even then and is a decidable predicate carrying a real constraint.

**An inhabited type with unused hypotheses.** `paralogical_copy` and
`paralogical_reflect` — `arrow` builds a protocol between any two imscriptions,
so both witnesses construct and neither hypothesis does work. Now defs.
`paralogical_dagger` stays an axiom: a trivial witness inhabits the reversal
type but `paralogical_dagger_depth` demands depth preservation, and `prod`
reverses to nothing any constructor assembles. Real content.

**Still open and mis-measured.** Roughly 19 axioms across Hodge, Navier-Stokes
and BSD have a trivially satisfied hypothesis or a `: True` conclusion —
`modularity_axiom : True`, `hodgeDecomposition : True`, `coates_wiles_axiom`
taking two `True` arguments so bsd_rank_zero_cm certifies EVERY elliptic curve.
That census was run against Prop, which cannot tell T from B, so the number is
not yet a finding. Re-run it against B4 before treating any of them as empty.

**q817 proved.** `no_three_ap` in Erdos/SumsetAvoidingAP.lean: base-3 digits in
{0,1} make x + z = 2y force x = y = z. The mechanism is no carries — both sides
bound by 2, so reading mod 3 is an equality rather than a congruence and the AP
condition descends one digit at a time. Standard axioms only. It was prose in a
docstring for months and §28 was an empty banner reserved for it.

**Erdős split.** One problem per file in Millennium/Erdos/, fifteen of them,
with reruns folded into their problem. FsplitBranch lives in Erdos/Base.lean.
The hub keeps only what spans problems: 2160 lines down to 555. Verdict counts
are computed from the table now — they had drifted to 6 when the table held 7.

**The Erdős corpus is complete, 2026-08-11.** Forty-one files, three hundred
theorems, one hundred thirty-six open problems stated as Props, and ZERO
`sorry` anywhere in it. Library green at 8590 jobs. Everything is in the
lakefile, including four files that had never been in the build:
NumberTheory/ErdosStrausProof, Classical/ErdosProblem16 and the three
Millennium/ErdosFormalize batches.

**The discipline that replaced sorry.** An open problem is a `def ... : Prop`;
a theorem that needs it takes it as a hypothesis, so the dependence shows in the
type instead of hiding in the axiom set. A `sorry` asserts its statement — which
is why a `sorry` on a FALSE statement is the one configuration that cannot
stand, and why 119 unchecked statements behind 76 sorries were the real risk in
the ErdosFormalize batches rather than the sorry count. Computed tables are
theorems about every configuration only when the generator is PROVED COMPLETE
(partsF_complete for Landau, search_true for g₃), never merely run.

**Defect shapes found, all now fixed.** Worth knowing because each recurs.
A free variable outside the parameters: ErdosStrausProof's `Solution` carried
`n` in its equation and in no parameter, so `∃ sol : Solution` never mentioned
the `n` the theorem bound. Junk values: `sInf` over a downward-closed set is 0,
which made `g_distinct` identically zero under a bound asserting 0 exceeds a
positive number; `sSup` over an unbounded set is 0, which collapsed a counting
window to `Icc 0 d`. Truncated subtraction: `b - a = d` on ℕ holds for b ≤ a and
carries no ordering — write `a + d = b`. Direction: an LCM density claim pointed
the wrong way against an exponential its own commentary quoted, and `(k!)^{1/n}`
was called subexponential when it outgrows every exponential. A base carrying
the parameter: `(4n−2)/3.199` exceeds one from n=2, so that refinement diverged.
And statements written against a Mathlib that does not exist —
`Mathlib.Combinatorics.Ramsey.Basic` and `Mathlib.Combinatorics.Hypergraph`
never existed.

**Two stale gates in the ob3ect pipeline, fixed 2026-08-11.** The imscription
gate's `validate_structural` enforced Axiom C as a BICONDITIONAL — 𐑦 and 𐑸 must
co-occur — while the kernel's `ImscriptiveTopology` reads `top = are → dim = if'`
only. The stronger copy rejected tuples the Grammar admits, and every ob3ect
whose role is scale-collapse under a nested topology failed grounding on it. And
`SplitFuseReport.validate` in ob3ect/core.py refused any Frobenius verdict but
PASS/FAIL, so an honest B or N marked the ob3ect invalid — auto.py had already
stopped retrying to force a classical PASS. A third followed: Axiom A was enforced as
chirality 𐑫 → kinetic 𐑪 alone, where `InfiniteMemoryNeedsSlowKinetics` reads
`chir = wool → kin = egg ∨ on` and Core.lean records the pairing with `on` as a
TENDENCY, not an axiom. All three are the usual two shapes: a second copy that
drifted stronger, and a graded value asked as a bool. The nest's ten ob3ects all
ground full and verify in Lean once the gates match the kernel.

**The batch Props did not state their problems, fixed 2026-08-12.** The three
`ErdosFormalize` files held 90 `Prop`s; most asserted something true of
everything they quantified over — `∃ N, N ≤ 4k+1`, `n/n → 1`, `ℵ₁ ≤ ℵ₁²`,
`s.card ≤ s.card`, a transversal with no size bound — and one negated a statement
ending in `True`, making it FALSE while recorded as B. Thirty-seven removed, five
more removed as weaker copies of statements living in `ProofModules` or proved
outright, the rest restated with the object they are about (Arrows on colourings,
unit-distance and distinct-distance counts, upper density, the difference set,
practical numbers, B₃ sets, Hindman against a tower). One became a theorem:
`edges_le_of_degree_le`, bounded degree bounding the edge count by handshake.
Verdict tables and their counts are recomputed from what each file defines.
`ig-docs/erdos_corpus_index.md` maps paper → Lean file and lists every remaining
Prop. Library green at 8592 jobs.

**Every manuscript boots, 2026-08-12.** Nineteen walks in `mOMonadOS/src/erdos_walks.rs`
under `erdos <name>`, one per paper, each step computing its claim on the kernel
rather than printing it. Three failed before they held, all my errors: the sumset
walk included 0 so the empty subset collided and then claimed distinct subset sums
for a set where 1+3=4 sits; the SDR walk took the window closed below and got 7
and 9 where the table says 8 and 10 — it is (n, n+L], open below; the chordless
walk closed the ODD path and produced an even cycle, when the odd cycle comes from
the EVEN path plus the chord. Nineteen crystalline branches cut at the verified
tip, named in `ig-docs/erdos_corpus_index.md` and not in the papers. Axiom sweep
over a full build: 100 lines, 94 on the standard three, and each of the six
compiled evaluations is declared as such in its own manuscript.

**Three provider defects, all the same shape, fixed 2026-08-12.** The Python
provider capped at 4096 tokens and gave up after 60s; `ask` carried a 300s
deadline and an 8192 cap on the ob3ect design call. Each failure arrived
disguised — truncated JSON reads as "returned no text", a timeout returns empty
rather than raising, and a reasoning model that spends its budget thinking
returns content:"" with no error. A working key looked like a dead lane three
times. All uncapped now; `max_tokens = 0`/None means the field leaves the payload,
and an empty content now reports its reasoning length and finish_reason.

**The twelve objects, and what they gave, 2026-08-12.** Designed through MoDoT's
native `ask --ob3ect` on deepseek, one per quantity a removed statement named and
the development lacked. Each object's structural reading became the definition:
the Ramsey number of a pair as a search under a terminal anchor; size-Ramsey as a
fork whose arms must both close; stepping-up as a parity fork with a forward
morphism; the cycle predicate from a fork on the property itself; girth as a
RANGE, its fuse keeping exact and interval arms; packing density as a limit
carrying its own existence; the displacement exponent fused to B and held as a
pair; the Pasch pattern; the partition relation held at B with
`partition_arms_exclusive` proved on no axioms; the list-chromatic threshold with
the arm its fuse discarded put back. Four of them corrected a collapse I would
have made by instinct. `ig-docs/erdos_corpus_index.md` carries the table.

**A gap is not a conflict.** B records a held conflict and needs ONE proposition
carrying both T and F. Four verdicts recorded B for something else: a lower
bound against an upper bound (Schütte), a case split across a parameter
(AntiRamsey, ChromaticOddCycle), a claim with a counterexample
(MonochromaticOddCycle), and a (T,N) pair (ErdosKac, Polynom). The kernel's own
`ffuse` settles each — B only from (T,F), truth-ordered join elsewhere — and
note `ffuse(F,N) = N`, so a refuted claim with an open constant fuses to N, not
F. The hub tables needed a third column for F as a result; partitioning into T
and B alone is what let a refutation and a case split both sit at B.

**Eighteen manuscripts, 2026-08-11**, one result each, in
`ig-docs/manuscripts_erdos/` with PDFs there and in `ig-docs/pdfs/`. Schütte
f(2)=7, Landau g(n), rep-tiling and 6, R(3,3)=6, Erdős–Straus off one class, the
Kac interval's prime-power exclusion, sumset AP-freeness, the Lenz d=4
construction, the n/pₙ rise criterion, Erdős–Fuchs averages, the chordless
shortest odd cycle, thick/syndetic duality, Mantel and anti-Ramsey, the
multicolour Ramsey ratio, lcm growth, binomial row gcd, the SDR window, and
Roth/Behrend. Branch `crystalline/erdos-audit-2026-08-11`.

**House style for a manuscript, learned the hard way here.** Author block is
`C. Lando Mills`, Trabuco Canyon California, c.landonmills@gmail.com, ORCID
0000-0003-0003-0552. Abstract runs several paragraphs and states the central
distinction before any notation. Theorems are NAMED as well as numbered and the
carrying result is boxed. Proofs are prose with displayed steps. Each paper ends
on position — what is settled, what is open. Every paper has its own
bibliography of the PRIMARY literature. NOTHING internal: no file paths, no
branch names, no theorem identifiers, no module names, and above all no account
of what was wrong before or of the process of getting there. A paper about the
defects found in our own library is not a paper.

**LaTeX traps that cost passes here.** `nonstopmode` emits a PDF over a fatal
error, so compile with `-halt-on-error`: `amssymb` clashes with `unicode-math`
over `\eth`, and a missing `cleveref` silently DROPS every `\Cref` from the
output. An unquoted shell heredoc collapses `\\` to `\` and eats `$n$` as a
variable, so build .tex bodies from a quoted heredoc or from Python.

**The MoDoT LLM lane was cut on 2026-08-11.** OpenRouter 402, its free-lane
fallbacks 404, deepseek empty content. `--provider local` runs but is
Qwen3-1.7B at ~10 tok/s. A spine verdict of N under those conditions is a
provider cut and refutes nothing.

**m3iosis `tuple_algebra.tier` is drift — not corroboration.** It gates
everything above `O_0` on POLARITY being `or'` (𐑹); Lean's `imscriptionTier`
keys on criticality and protection. So it returns `O_0` for every tuple whose
polarity is not `or'`, which put it in flat disagreement with the proved
`schutte_tournament_tier_is_O1`. Lean `Core.lean` is authority.

Full library green at 8590 jobs.

**The UNSOLVED corpus, 2026-08-12.** `Millennium/Unsolved.lean`: seventy-three
problems as `def ... : Prop`, zero sorries, in the lakefile, build green at 8593
jobs — algebra, analysis, combinatorics, number theory, graph theory, geometry,
set theory, topology and knots, discrete geometry, Diophantine equations,
Euclidean geometry, drawings/crossings/planarity. The machinery is BUILT, not
assumed: a knot from scratch, planarity from a straight-edge drawing with a
crossing as two independent open segments meeting — which is what let Barnette be
stated at all — cap sets over F₃ⁿ with collinearity as a zero sum, twice the
triangle area as a determinant giving Heilbronn a two-sided bound, Danzer sets,
Falconer through a distance set, Thomson energy, ropelength through thickness.

The source list carried the familiar defects and the restatement fixed them: the
cap-set bound quantified C AFTER the inequality, so any C near zero satisfied it
(now cⁿ with c < 3); the dead-fly entry asked for bounded separation, which a
lattice has, so it asks for bounded density; "factoring in polynomial time" meant
polynomial in n, which is trial division, and now says BITS. Its Erdős–Straus
entry used natural division, where both sides are 0 for n > 4 — not restated,
since `Erdos.StrausGreedy.IsThreeUnit` carries it over ℚ and a second copy is
drift.

**Novikov and Baum–Connes: a claimed lack, retracted.** Writing that they need
characteristic classes and an assembly map no local definition supplies implied a
lack the Grammar does not have. `FrobeniusAlg` in `IGFunctor.lean` IS an
assembly — comul localises, mul assembles, frob says the assembly reconstitutes
the splitting. `AssemblyReconstitutes` and `AssemblyIsIso` are stated over an
arbitrary Frobenius algebra and `ig_assembly_reconstitutes` PROVES the Grammar's
own case on propext alone via `mu_delta_A_id`; open elsewhere is the non-diagonal
splitting. Novikov is `WindingInvariantUnderClosure` — the characteristic class
is the winding, an integer on the twelfth axis. `PerfectCuboid/CaseC.lean` lost
an axiom that asserted its own open case.

**The withdrawn biconditional propagated, cleared 2026-08-12.** Sixty `digital/`
ob3ect artifacts still carried the Axiom C biconditional's verdict after the
kernel's one-directional `ImscriptiveTopology` won: 208 full where there were
148, 36 failed where there were 96. `ground_native` stores `to_notation` rather
than the Python repr, so the twelve native artifacts carry tuples — seven full,
five failing on real constraints (three Axiom A, wool chirality with fast
kinetics; two the corrected Axiom C). A stale gate's verdict outlives the gate:
withdrawing one means re-checking every artifact it ever judged.
