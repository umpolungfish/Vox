---
name: vox_carrier_constructor
description: "vox construct-carrier / factor-with build a factoring carrier tower from any operator ob3ect word, in Vox/src/morphism_factor.rs"
metadata: 
  node_type: memory
  type: project
  originSessionId: 37fe5b8e-93c6-4b79-b8b4-9a08c8c81595
  modified: 2026-09-15T05:45:12.320Z
---

The automated carrier constructor lives in `Vox/src/morphism_factor.rs` (the vox
crate at ~/imsgct/Vox, NOT the g-momonados one), per [[feedback_build_in_vox_not_rust]].

`factor()` runs a fixed tower of six operator words: PHASE ∈≻⊤⊥∋, ARITHMETIC
∈⋈⊤⊥∋, BRANCH ∈⊤⊥∋, SELECT ∈⊙∋, CONTINUE ≻⋈, FIX ⊙⊡. Dispatch reads the operator
word itself (apply_morphism). The carrier is the parasm register machine (native
value B4, ops = the twelve marks); the number rides as a native_numeral tape.

New: `construct_carrier(word)` decomposes any operator word's interior into the
ordered morphisms it realises by matching those motif interiors; AREV ≺, ENGAGR
⊞ (hold), and a lone IMSCRIB ⊙ are carry marks skipped without emitting an
operator; any other unmatched mark is refused with its position named.
`factor_with(word, n_word)` builds the tower, checks it is factoring-complete
(advance + SELECT + CONTINUE + FIX) or names what is missing, then runs the same
nested loop. CLI: `vox construct-carrier <operator-word>` prints the tower;
`vox factor-with <operator-word> <n-word>` runs it. Binary: Vox/target/release/vox.

There are eight morphisms. The seventh, EXTRACT (∈≻⊤≺⊥⊞⋈∋, carrying AREV ≺ and
ENGAGR ⊞ in the evaluate frame), is the instant one-frame factorizer; a tower
carrying it needs only a FIX to be complete. Added 81caa6d after wrongly refusing
it first as a non-factoring mark; the extractor is a factorizer by design.

EXTRACT now forks three arms per round and fuses on the first to close (9bdcf4e):
the square frontier (a from ceil(sqrt N), factor when a*a-N is a square), rho, and
trial. So a balanced near-root semiprime is flat in width: 60 to 120-bit all
close in about a thirtieth of a second, past the old rho ceiling. isqrt over tapes
seeds/tests the frontier.

The ninth, ECM (∈⊙≻⋈∋), is Lenstra elliptic-curve as a nestable morphism
(c9f3cbc): one curve per round from a rising seed, factor when the group law hits
a non-invertible slope (mod_inv returns the gcd as Err). Discovery rides
curve-order smoothness, independent of p-1's. Tape layer gained mod_sub, mod_inv
(extended Euclid, coeffs mod N), affine ec_double/ec_add/ec_scalar, stage-1
scalar lcm(1..~47). Reach is B1 (here ~47, small factors); larger B1 + stage 2
widen the same arm. Extra arm, nests with a complete one. ECM-only carrier
⊢∈⊙≻⋈∋⊙⊡⊣ factors 143/8051/100160063; four-arm ⊢∈≻⊤≺⊥⊞⋈∋∈⊙⊞⋈∋∈⊙≻⋈∋⊙⊡⊣ is
EXTRACT->P_MINUS->ECM->FIX.

The eighth, P_MINUS (∈⊙⊞⋈∋), is Pollard p-1 as a nestable morphism (99ef247):
raise an accumulator through rising exponents, gcd(a-1,N). It catches a factor p
whenever p-1 is smooth, at any size and gap, the slice frontier and rho miss. It
is an extra arm, not standalone (loops on a non-smooth predecessor), so it nests
with a complete arm. Carrier EXTRACT->P_MINUS->FIX (⊢∈≻⊤≺⊥⊞⋈∋∈⊙⊞⋈∋⊙⊡⊣) closes a
97-bit N with a 45-bit smooth-predecessor factor and a ~2^51 gap in 0.09s where
EXTRACT alone runs past 40s. pow_mod over tapes does the powering. The residual
hard slice (both factors large, gap large, neither predecessor smooth) is the
sub-exponential core (sieve / bigger-B1 ECM), the next nest, not a wall.

Nesting order is load-bearing, both directions. ECM as the costly arm fires only
on power-of-two rounds (3869430) so the cheap arms carry the walk; certifying
999983 dropped 5.6s to 0.22s. A WITNESS arm (Miller-Rabin over tapes, frame
∈⊤≺⊥∋) nested FIRST (0765eb0) selects a prime at once instead of walking trial to
sqrt(N): 1000000007 went 8.7s to 0.06s, a 19-digit prime closes in 0.17s. The
full carrier is WITNESS->EXTRACT->P_MINUS->ECM->FIX, cheap decisive test
outermost, exhaustive walk deeper.

Two more arms (a31457a): POWER (∈⊤⊞⊥∋), one-shot perfect-power test (is N=a^b?
select base a), ipow/iroot over tapes; P_PLUS (∈⊙≺⋈∋), Williams p+1 (Lucas
V-sequence to stage-1 scalar, gcd(V-2,N)), catches p where p+1 is smooth, lucas_v
over tapes. LEHMAN (∈≻⋈⊤⊥∋) multiplier-Fermat, mid-range factor N^(1/3)..N^(2/3), is_square +
per-k window; and SQUFOF (∈⊤≺⊞⊥∋) Shanks square forms, one-shot forward-then-
reverse cycle (square tested on Q_i at top of even step, NOT after advancing;
that was the trap), sign-tracked Q recurrence, validated on 8051/11111/2021/
1234567/2027651281. Full membrane now nine arms:
WITNESS->POWER->EXTRACT->SQUFOF->P_MINUS->P_PLUS->LEHMAN->ECM->FIX (word
⊢∈⊤≺⊥∋∈⊤⊞⊥∋∈≻⊤≺⊥⊞⋈∋∈⊤≺⊞⊥∋∈⊙⊞⋈∋∈⊙≺⋈∋∈≻⋈⊤⊥∋∈⊙≻⋈∋⊙⊡⊣). 15 crate tests.
Cadence (d392234): a shared round counter in factor_with's loop; P_MINUS/P_PLUS/
LEHMAN fire on a stride (round%4==0/1/2), SQUFOF delayed to fire once after round
256 (its N^(1/4) cycle is heavy), ECM power-of-two. Costly arms nested deep in
time so the cheap EXTRACT walk carries the rounds; fixed a 4-digit factor of a
60-bit N taking 6.5s.

Bit-register folding (41bd257): the tape is still the canonical numeral but
cmp/add/sub/mul/divmod/modulo fold it into u64 Limbs, compute with hardware words
(u128 product carries, shift-subtract division), unfold at the boundary. Every arm
routes through mod_mul so all speed up: a 19-digit prime certifies 23s -> 0.05s,
2^63-25 (was timeout) 0.05s, test suite 6.3s -> 1.26s.

Transport program (bringing G-mOMonadOS factoring tools into Vox on the folded
kernel): divisor_membrane now shares the folded primitives from morphism_factor
(8d59357, deleted its duplicate arithmetic); factor_operator ported (the CL9NK
moat resolver, 2-adic winding DFS builds p,q bit-by-bit, node count = moat width;
`vox factor-operator resolve|full <N>`, no cap). morphism_factor exports the
shared kernel: pub trim/cmp/add/sub/mul/divmod/modulo/tape_u64/one/two/zero/isqrt.
Remaining G-mOMonadOS candidates to port+lift+build out: native_numeral two-arm
engine (hensel_unbraid), prime_winding/oneshot_prime_winder, shor_qft/belnap_shor.

Diagrams: render_carrier_diagram.py <sequence> [name] [out.svg] prints the ASCII
circuit and writes the SVG (reuses IMSCRIBr imscr_wiring + render_wiring_svg_v3,
the ob3ect/auto.py path); before/after carrier SVGs in ig-docs. Principle: every factoring
method is a nestable morphism keyed by its own frame; adding one only tightens the
constraint; cheap decisive tests outermost, costly arms deeper and throttled; the
constructor builds whatever tower the word spells. 13 crate tests.

Verified: a word whose interior is the six motifs in order rebuilds the full
tower and factors 8051 to 97, identical to `vox morphism-factor`. The instant
semiprime extractor word ⊢⊙∈≻⊤≺⊥⊞⋈∋⊙⊡⊣ decomposes to EXTRACT then FIX and
factors on the carrier built from itself: 8051 to 97, 143 to 11, 100160063 to
10007. See [[factor_membrane_two_arm_tree]].

HARD-tier nesting + sieve reach (2026-09-14, eb88804 + 99df176). `smart_factor`'s
HARD branch (scout returns None) nests, cheap-decisive outermost: bounded QS pass
first, then the nine-arm carrier's rho via `run_carrier_rounds(tower, n,
HARD_CARRIER_ROUNDS)` (the bounded loop factor_with now shares, one copy), then
Dixon last. Inverting it (carrier before sieve) regressed the 72-bit 1.6s→16s —
that regression IS the signal the order is wrong, see [[feedback_nesting_is_free_lunch]].
`sieve_params` widened the single-polynomial window with N above 64 bits
(1.5M + (bits-64)·1.5M) and raised the base cap to 60000; the base bound was
already near-optimal, the window was the limiter. Reach: MPQS (`sieve::mpqs`, d0d563c)
is now the primary hard arm and closes balanced semiprimes to ~124 bits (112-bit
~5s, 123-bit ~19s), faster than single-poly everywhere they overlap (79-bit 1.3s
vs 2.5s); single-poly QS is the fallback, then carrier rho, then Dixon. MPQS:
g(x)=Ax²+2Bx+C, A a product of 3 base primes, B by CRT so B²≡N (mod A), sign on a
phantom -1 column so `combine` is untouched, base sized to reach A's primes, all
in machine ints exact while (Ax+B)²<2^127. Carried wide (6655044): N
stays on the tapes, N mod p off the tape, C=(B²-N)/A on the tapes; only the per-x
inner loop is machine-word (g~M·√2N and Ax+B fit i128 past 200 bits). A is k primes
each near a_target^(1/k), k rising with N so per-prime size stays in the base; base
bound tracks the sieve optimum by a width table. Self-initialized (fe8a6c0): one modular
inverse per A shared across its 2^(k-1) sign-varied B siblings, each recomputing
roots directly from the cached inverse (a Gray-code additive update was tried
first and was WRONG — siblings past the first got bad roots, yielded nothing,
2^(k-1)-fold wasted work; direct recompute costs the same one multiply and is
correct). Plus offset position prepared once per sibling and reused by sieve+
candidate division, log buffer cleared/reused in the A frame (took 160-bit 50s→19s;
Astra found this half during a usage-limit gap, verified by me). Reach now ~200 bits:
140-bit ~3s, 160-bit ~19s, 180-bit ~118s, 199-bit ~273s, all correct, 37 tests.
The linear algebra is NOT the frontier bottleneck (combine ~3s of a 20s 160-bit
run); relation collection dominates, so block Lanczos only matters past 200 bits.
(earlier pre-self-init: 140-bit ~6s, 160-bit ~22s through the front end); 180-bit is the frontier, held by the dense
GF(2) elimination (next rung: block Lanczos/Wiedemann sparse solver). Two bugs
found by measurement, both relation-independence: A-sets collided across polys
(fix: step distinct k-combinations in lex order via next_combination); duplicate
relations counted toward need while distinct fell below base width so every
dependency was a trivial duplicate pair (fix: a seen BTreeSet keeps only distinct).
LESSON: a relation count is meaningless until it is a DISTINCT-relation count above
the base width. `vox mpqs <N>` runs it alone; `--features mpqs_debug` prints
relation/distinct/dependency counts. Technique
writeup: ig-docs/factoring_membrane_nesting_technique.md (cb71b6c9). Caveat, HARD-LEARNED: this box is WSL2 and factoring timings are
worthless unless the machine is quiet and runs are SERIAL. A control of three
identical runs (same N, base, window) back-to-back read 242.78s, 31.67s, 6.03s as
concurrent background jobs drained — a 40x spread on identical input. Running many
`vox mpqs` jobs at once (or hung procs: two gen_hard.py loops another session)
makes every reading a contention artifact. Before ANY timing: kill stray vox/procs,
confirm `/proc/loadavg` ~0, run one at a time, and repeat the point to see the
spread. The FACTORS are always trustworthy (correctness is load-independent);
the seconds are not until serialized. On a quiet box 180-bit closes in single-digit
seconds, not the ~118s reported under self-inflicted load. 36 crate tests.

Committed 4ecb11a and 81caa6d on branch notation/prec-succ-marks (Vox is its own git repo,
with a stale tracked commit.txt like ig-docs; always commit -F an absolute path
outside it per [[feedback_commit_message_file_path]]).

NO CAP on N (3ae3a70, supersedes all bit-ceiling talk above). The sqrt(2N) guard
and the u128 `?` bail on C are gone. C, g(x), Ax+B compute in a machine word while
the value fits one (a `wide` flag picks the fast path once per run from N's width)
and fall to signed tape arithmetic (`sadd`) when they would overflow — no bit
ceiling. 112-bit ~1s on the fast path (no regression); a 200-bit balanced semiprime
that used to abort at the word boundary now runs to closure. Above ~200 the value
is tape-carried and keeps going. The heavy stage at large N is now the DENSE GF(2)
combine (width ~7000 at 184, ~9000 at 200), so block Lanczos/Wiedemann is the real
next lever, arriving here not at 256 because the base is wide. Also: Astra committed
b4bec45 (resident circuits / runtime gates) on branch notation/prec-succ-marks;
coordinate on shared files.
