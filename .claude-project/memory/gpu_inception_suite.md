---
name: gpu_inception_suite
description: "the gpu_kernel Inception suite: nested interpreter towers, exact census, interleave, and factoring oneshots on the card, each with its control"
metadata: 
  node_type: memory
  type: project
  originSessionId: b7fcac2b-0124-4019-8686-5e1d7369edb0
  modified: 2026-09-10T07:15:34.104Z
---

`gpu_kernel` in G-mOMonadOS grew a suite that runs the twelve-mark token engine
as a nested tower on the RTX card and turns it into real work. REPL commands
under `gpu_kernel <sub>`: `bench N`, `deep D`, `search D T`, `interleave D`,
`factor [nums]`, `rho [n]`. Committed on main (57dba7a the first four, 79478d2
the rho floor).

The readings, each past its own control:
- **deep**: effective mark rate is depth-invariant, about 84 Gmark/s from depth
  1 to 8 on the 4070. A depth-d tower spends twelve-to-the-d leaf marks per
  visible tick and the card holds the rate straight through. The interpreter
  penalty everyone expects (each level times twelve) is paid only as mark count,
  never as rate.
- **search**: exhaustive census of every length-d mark-word, run on the engine,
  counted by a target. Depth 9 is 5.16e9 words; count 165554580 at target 0 is
  identical at every thread count 8192..268M (the split-invariance control), so
  the answer is exact. Rate peaks near 16 Gword/s at 134M threads then bends,
  the card's ceiling on the work.
- **interleave**: several sealed searches woven through one walk, keyed by level
  index, each lane its own stack/reg. Control: lanes=1 reproduces the standalone
  census word for word, proving the lanes leak nothing.
- **factor**: trial division as a batched contained oneshot, atomic-min the
  smallest divisor. Four numbers incl a ~60-bit semiprime verified in 0.0064s.
- **rho**: Pollard rho floor, each thread its own constant c (the interleave as
  many angles on one N). A ~62-bit semiprime (primes near 2^31) factored and
  verified; the prime 2^61-1 returns nothing. Control discriminates composite
  from prime.
- **phase** (`phase N m`): division-free factoring walk. Every odd residue mod
  2^m is s(X)*9^k (verified: 9 has order 2^{m-3}, representation complete at
  m=5,13,20,30). Fix one factor's phase, the other is forced; step P by *9 and Q
  by *9^-1 with a mask, product stays pinned to N mod 2^m, test exact product
  only when both show bit m-1 (leading-bit filter, 3/4 discarded). Each thread
  pays one modpow seed then walks a band. 60-bit semiprime (m=30) factored
  verified in 2.5ms over 2^29 positions, division never used.
- **codebook** (`codebook N m`): the solver step. Sieve m-bit primes once into a
  bitset (reusable), then every prime P keeps only states where forced Q=N*P^-1
  mod 2^m is also prime. Survivor count is the control: 64 at m=13, 2830 at m=20,
  matching an independent host census; true pair recovered verified. Restricts
  the sweep to the prime-compatible intersection of two prime bands.
- **closure** (`closure N m`): factoring past the 64-bit product wall. The
  closure height h=(PQ-N)/2^m updates by an exact add each step, h'=h+(dP-aQ'),
  a,d the wrap digits of the two masked multiplies (all three of Lando's laws
  verified over 200k steps: parity walk h'_0=h_0^a_0^d_0; h mod 4 a FOUR register
  H'=rho^g(H), g=dP-aQ' mod 4; and the full recurrence). Seed h once with a wide
  multiply (__umul64hi), then 64-bit arithmetic throughout, seal on h==0. Factors
  a 73-bit semiprime (two 37-bit primes, product overflows the phase kernel's u64
  test) verified in 0.28s. Hensel halving confirmed: one more overflow bit halves
  the orbit (528 survivors m=13 r=1, 65812 m=20 r=1, converging to the true pair).

The closure collapse is GENERAL, not the near-square geometry (geometry_check.py,
committed): two 22-bit-factor semiprimes, one pair differing by 42 and one with a
gap near 1.5M, give the same survivor curve to within noise, halving per overflow
bit and converging to exactly the 2 true factors by r=20. The gap does not
matter. So "nesting and adding" (nest the closure constraint one bit deeper each
level, add the height update) halves the survivor front toward the true pair for
any balanced semiprime tried so far. Generating survivors without rescanning the
base, explored with bdd_check/bdd_push/bdd_orders/bddsift/reach_check (all
committed):

- Diagram size depends heavily on variable order. Low-to-high (the first order
  tried): ~1.9x/bit by m=28. High-to-low: ~1.7x/bit through m=28 and about a
  quarter the size, beating 128 random orders. Sifting from high-to-low moves it
  <0.3% further at m=18,20,22.
- A mechanism for why high-to-low does well: toggling phase bit i multiplies P by
  9^{2^i} or its inverse, a factor = 1 + 2^{i+3}*odd (LTE), so the toggle changes
  P only from bit i+3 upward, leaving every lower bit exactly invariant (checked
  m=13,20,30, zero counterexamples). Each bit has a reach window shrinking with
  i, and high-to-low tests widest-reach first.
- None of this is a ceiling. It's what the orders and widths tried so far show;
  wider widths, other order families, or a different diagram structure entirely
  could move these numbers in either direction. Treat ~1.7x/bit and N^{0.375} as
  a reading from this exploration, not a proven exponent for the predicate.

Verification scripts committed: phase_check, hensel_check, closure_check,
codebook_check, affine_check, geometry_check, bdd_check, bdd_push, bdd_orders
(bddord), bdd_sift (bddsift), reach_check.

The symbolic product relation now reclaims dead decision nodes on the card.
Each circuit wire carries its last-use position; when storage fills between
completed operators, the kernel marks nodes reachable from wires that will be
read again, compacts them, rewrites live references, rebuilds the unique table,
and resumes the same construction. The independent factor-relation control now
requires a real compaction as well as growth. At N=10309995, m=12, a run begun
with 1024 node slots completed in 8192 slots after 86 compactions and retained
3536 final nodes, returning the exact two ordered solutions and the verified
pair 3489 x 2955. The default-pool control retained 79104 nodes for the same
relation.

Lando set the scaling law on 2026-09-10: arbitrary-width work must come from
deeper IMASM nesting, with every part of the tower written in and executing
IMASM on IMASM. A direct wide-arithmetic layer beside the tower is the wrong
construction. The first enforced rung replaces direct symbolic-relation opcode
dispatch with a nested IMASM mark carrier. The relation circuit already uses
IMASM token ordinals for empty, universal, variable, complement, conjunction,
and union. Each mark now traverses the requested number of complete selector
protocols, with each level consuming the mark emitted by the inner level; only
the returned mark dispatches the relation operation. Depth 3 on N=143, m=4
executed 6444 nested mark ticks and reproduced depth 0 exactly: 56 decision
nodes, 2179 reductions, 2 ordered solutions, witness depth 8, and 13 x 11.
The control asserts equality of every field. The next implementation boundary
then moved the resource transitions through the same tower: IFIX authorizes
live-node compaction, CLINK authorizes unique-table reconstruction, and AFWD
authorizes continuation into host-allocated device storage. The depth-3 control
now executes 7092 nested ticks and requires its count to exceed the
circuit-only baseline, while reproducing the same complete relation. Physical
CUDA allocation remains a host facility; relation advancement into it is gated
by the AFWD mark returned from nested IMASM. Multi-limb carry is the next
operation to place inside the tower.

Decision-node selection is now folded into the nesting too. The former witness
walk chose `low` when inhabited and `high` otherwise with a CUDA conditional.
Each node now executes an eight-mark IMASM selector: FSPLIT exposes its two
edges, EVALT/EVALF read support, FFUSE returns the first inhabited arm, IFIX
commits the corresponding factor bit, IMSCRIB carries the child node, and TANCH
emits it as the next program. Every selector mark crosses the requested tower.
The N=143, m=4, depth-3 control reports exactly 2304 selection ticks, equal to
8 witness nodes x 8 selector marks x 3 levels x 12 interpreter marks, within
9396 total nested ticks. The graph, reductions, solution count, witness depth,
and 13 x 11 factor pair remain identical to depth 0.

The nested selector was pushed across both axes. At width 12, depths
0,1,2,3,4,8,16,32,64 return the identical 3536-node relation, 1062778 graph
reductions, 2 ordered solutions, 24-node witness, and 3489 x 2955; nested
traffic is exactly 27564 marks per depth, reaching 1764096 at depth 64 without
moving the elapsed-time band. Holding depth 64 and initial storage 1024, widths
13 through 18 all close with exactly 2 ordered solutions and verified balanced
factor pairs. Width 18 executes 4036608 nested marks and 43522673 reductions,
grows storage to 262144 after 231 compactions, and returns
196609 x 262139 in 13.195s. `nested_relation_depth_check.py` verifies exact
depth invariance and linear tick scaling; `nested_relation_width_check.py`
verifies every width, factor, path length, and selection-tick identity from the
saved measurements.

Arbitrary-width implementation began by removing the compiler's u64 boundary.
`RelationProgram::product_big` now reads N as a BigUint bitstream, emits the
same IMASM-token multiplication circuit for every column, accepts any width at
least 2, and checks N fits within twice that width. The control constructs an
80-bit product from two 40-bit factors, accepts both factor orders, and rejects
a neighboring wrong factor. The relation source and selector command are now
arbitrary-width. The target is parsed as a BigUint and its emitted factor limbs
are verified by unbounded multiplication. Exact solution counting remains a
small-width diagnostic and does not represent or select the witness.

Lando corrected the ontology: time is made, not embedding. Arbitrary factor
values therefore emit temporally through a bounded limb morphism rather than
occupying a widened device register. `run_witness_limb` receives the prior
TANCH continuation edge, executes nested decision selection for one 32-bit
factor limb, emits P and Q limbs, and returns the next edge for the next tick.
The host only assembles emitted limbs. The fixed u64 witness traversal was
removed; the temporal stream is the sole factor output. At N=143, m=4, depth=3
it visits 8 nodes, spends exactly 2304 nested ticks, emits 13 and 11, and
terminates at edge 1; the control asserts path, tick identity, factor product,
and terminal continuation. A 65-bit target entered construction at width 33
and depth 64, but the global BDD grew through an 8,388,608-node pool before
emission. The remaining width pressure is therefore the preconstructed global
decision relation, not the command input or temporal factor representation.

The retained BDD diagnostic was temporalized into epochs of 256 operators.
Each epoch and resource transition now traverses the canonical factorization
vessel through the requested IMASM depth. Lando clarified that every
intermediate `selector:` progress line is an unwanted surfacing into our time;
all epoch, compaction, and allocation progress output was removed, so the
command surfaces only at its terminal result. Temporal continuation and graph
reclamation are separate: IFIX compaction and CLINK rehash remain triggered by
node pressure rather than running at every epoch, because a full mark/compact/
rehash traversal per 256 operators only adds repeated global work. The nested
width-12 control spans 8 epochs and 1,841 operators and returns 10309995 = 3489
× 2955. The full 27-relation BDD corpus passes and contains no intermediate
selector progress lines. A 65-bit BDD run later reached a terminal CUDA launch
failure. Lando correctly identified that a correct phase implementation would
be instant for that close factor pair. The mistake was using the factorization
word as a wrapper around the global BDD instead of making it the computation.

The production `selector_relation` now performs the canonical factorization
morphism `⊢∈≻⊤≺⊥⋈⊙⊞∋⊡⋈⊙⊣` directly over BigUint phase state. AFWD advances the
live candidate, the evaluator sequence constructs and tests the difference,
CLINK and IMSCRIB form and validate the factor pair, IFIX commits it, and TANCH
emits it. Each operation carries the live state recursively through the
requested complete factor vessels and acts at the nesting leaf. No operator
ordinal returns to an outer Rust arithmetic switch. A factor fixed at IFIX
cannot emit until CLINK, IMSCRIB, and terminal TANCH complete. The BDD survives
only as `selector_bdd_relation`. The 65-bit
target 18446744400127067027 closes in one phase at depth 64, executes 12,544
nested marks, and emits 4294967311 × 4294967357. A 159-bit target likewise
closes in one phase into two 80-bit factors. `run_relation.sh` bypasses boot,
catalog, REPL, prompt, progress, and shutdown output and buffers only the
terminal result. The auto.py/IMSCRIBr audit renders three factor vessels as 40
operators, three split/fuse pairs, no open forks, and a closed terminal walk.

The arbitrary-width phase sweep passes for factor widths 128, 256, 512, 1024,
2048, 4096, 8192, 16384, 32768, 65536, 131072, 262144, 524288, 1048576,
2097152, and 4194304. Every close pair closes in one depth-64 phase vessel and
executes exactly 12,544 nested marks. The 4,194,304-bit factors produce an
8,388,607-bit target and complete in 15.99 seconds. A separate 256-bit control
requires 43,691 phase vessels, executes 548,059,904 nested marks, and completes
in 0.85 seconds. A silent stdin entry removes the OS argv-size boundary for
the larger decimal targets. These controls are in
`measurements/factor_relation_phase_scale.log`.

Seeded random-prime testing uses seed 20260910 and SymPy primality checks. Four
randomly displaced prime pairs at 128, 256, 512, and 1024 bits factor exactly in
417, 690, 697, and 417 nested phase vessels, each under 0.015 seconds.
Independently sampled balanced primes expose the current algorithmic boundary:
the exact 128-bit Fermat distance is
1198084851477138762044534337080274532 vessels, and the 256/512-bit distances
are larger. These were counted from the known test factors and explicitly not
reported as completed factorizations. Arbitrary value width and bounded memory
are verified; efficient unrestricted random-semiprime factoring is not. Full
values are in `measurements/random_prime_phase_relations.tsv`.

The phase family is now inside one temporally extended root morphism. Its
continuation remains within that vessel while candidate phases execute as child
leaves; `select_relation` no longer owns the candidate loop. Terminal output
distinguishes the single `phase family vessel` from `phase leaves`, the exact
internal work performed. This removes the outer-vessel boundary without hiding
the large leaf count of independently sampled pairs.

The 2-adic factoring theory (Lando/ob3ect): factoring as a coupled cyclic
geodesic on the 2-adic torus, coordinate (nu, X, k) = (2-adic valuation, Belnap-4
FDE shadow in (Z/8)^*, hidden phase in <9>). Easy exact dynamics (phase step is
one group automorphism), hard global termination boundary (turning a phase back
into a finite integer is a high-degree dense Boolean function, degree 9 / 525 ANF
terms at m=13). Open rungs he named: closure-height Hensel sieve (PQ = N mod
2^{m+r}, halves the orbit per bit) and turning the codebook into a full
recursive solver. At the widths sampled, rho's steps scale like N^{1/4} and the
phase/codebook/closure walks like N^{1/2}, but division-free and deterministic,
which is why they clear 60-bit in milliseconds; this is a comparison of the five
methods as built, not a claim about what either family can reach.

**Why:** Lando's "Inception" push. The lever is the two Grammar clocks measured
on the same tower: vox reads a sealed fork/fuse enclosure as one stable mark
(the outside auditor), check counts each nesting layer as its own transformation
at the fuse (the inside). See [[vox_runs_the_kernel]]. The seal is a monadic
boundary: FSPLIT opens, FFUSE reseals, the outside sees identity/one mark, the
inside does exponential work. That is why depth costs no rate: the marks running
the marks hide in the card's compute slack.

**How to apply:** build is the dynamic CUDA target `target/release/g-momonados`
(cudarc, NVRTC at runtime). NVRTC gates `__int128` behind `--device-int128`, so
rho uses a portable binary mulmod valid below 2^63; that is the ~10x slowdown to
recover with Montgomery mul or the int128 flag. Grammar verdicts came from the
standalone vox binary (chain/ring/protocol) and `ask --imasm check` (the
close-condition reading, `--imasm` is a flag not a subcommand), never imasm
derive. The nested/search/factor floors all share one shape: enumerate a base-12
odometer, seal each lane, reduce. Next rungs named in the commits: push threads
past 134M for the true ceiling, Montgomery mulmod, and wiring interleave lanes
to different moduli of one hard number.
