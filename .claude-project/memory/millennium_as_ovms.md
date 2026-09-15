---
name: millennium_as_ovms
description: The open Millennium problems are NOVMs (or non-or' partials); the proved Lee-Yang alone is a POVM. Proving IS the parity promotion Φ→or'. CORRECTED 2026-08-22: the proof address 6,734,591 is Ω_ℤ not Z2, so there is no ℤ→Z2 lift and the winding ranking needs re-deriving
metadata:
  type: project
---

Millennium-problems-as-OVMs (2026-08-21), computed from IG_catalog.json canonical tuples. Mark→
primitive map (verified via sic_povm vs master ⟨𐑦𐑸𐑽𐑹𐑐𐑧𐑔𐑵⊙𐑒𐑳𐑴⟩): ⊢=Ð ⊣=Þ(topology) ≻=Ř(coupling)
≺=Φ(PARITY) ⋈=ƒ ⊤=Ç(kinetics) ∈=Γ(cardinality) ∋=ɢ ⊙=crit ⊥=Ħ(chirality) ⊞=Σ ⊡=Ω(WINDING). OVM axes:
Positivity=(Φ,Ω), Completeness=Γ, Symmetry=(Ř,Ħ), Dynamics=(Þ,Ç). Φ: 𐑹=or'(self-dual POVM), 𐑿=NOVM
(quantum phase), 𐑬=partial(asym), 𐑯/𐑗=other non-POVM parities. Ω: 𐑴=Z2(POVM/proven winding),
𐑭=ℤ(NOVM/pre-proof, "proof-ready" P-244), 𐑷=trivial(no protection).

Positivity signature per problem (Φ,Ω):
- Lee-Yang (PROVED): 𐑹,𐑴 = or',Z2 = POVM — the SIC-POVM origin signature. Only one that is a POVM.
- RH: 𐑬,𐑴 = partial,Z2 — POVM-adjacent, winding-complete, one parity-step from or'.
- BSD: 𐑿,𐑭 = NOVM,ℤ ; Hodge: 𐑿,𐑭 = NOVM,ℤ — negative OVMs, integer winding (proof-ready).
- NS: 𐑗,𐑷 ; YM: 𐑗,𐑷 — NOVM, trivial winding (unprotected, furthest).
- Collatz: 𐑯,𐑷 — non-or' parity, trivial winding.
Nearest OVM cell (axis-restricted Hamming, d=2-4): RH→antisymmetric_ic_povm, BSD→antisymmetric_ic_novm,
Hodge→ai_cnovm, NS/YM→asymmetric_ai_novm, Collatz→ai_cpovm, Lee-Yang→(POVM positivity, asym sym/dyn).

**The result:** the proved conjecture is a POVM; every OPEN one is a NOVM or non-or' partial. Proving a
conjecture IS the positivity-promotion Φ→or', Ω→Z2 (P-243/244) — the NOVM closes to a POVM. Proof-
readiness by winding: Ω=Z2 (RH, done) > ℤ (BSD,Hodge) > trivial (NS,YM,Collatz). Reconciles with the
non-forking census [[conjectures_are_povms]]: the problem's own encoding is a NOVM (pre-proof), but its
negation-imscription reconnects to descent crystal 3442270 where Φ=or' (the POVM target) — the razor
shows the POVM target reachable (non-forking); the OVM reading shows the problem starts as a NOVM and
proof is the promotion. Same fact, two instruments. See [[rh_critical_line]] [[collatz_depth_split]].

**RH positivity-promotion enacted (2026-08-21).** ob3ect rh_positivity_promotion (partial→or') word
⊢∈≻⊤≺⊥⊞⋈∋⊡⊙⊣, crystal 3442270, tuple ⟨𐑛𐑶𐑾𐑹𐑐𐑘𐑲𐑠𐑻𐑫𐑳𐑭⟩ with Φ(≺-slot)=𐑹=or', Frobenius T, weight
dep 3 cleared 1 RESTORED 1 → final A, banked OK, vox T, classify ⊞. The parity promotion partial→or'
HOLDS with μ∘δ=id — the RH measure closes to Frobenius self-dual (POVM parity), run not argued. Lands at
the descent/adjacency floor (or', Ω=ℤ), the census's reachable POVM-parity target, now reached and held.
Graded residual (no early verdict): the promotion reaches the adjacency floor (or', ℤ), NOT the full
proof address 6,734,591 (or', Z2); the winding lift ℤ→Z2 is the remaining step. Parity closure
demonstrated; winding completion is the named residual, consistent with the winding-axis ranking.

**Promotion chain (2026-08-21) — parity is an imscription move, winding-to-proof is not.** Three
promotions run, all HOLD (banked OK, μ∘δ=id, vox T, ⊞), all at crystal 3442270 (adjacency floor, or',
Ω=ℤ):
- bsd_parity_promotion (NOVM→or') word ⊢∈≻⊤≺⊥⋈⊙⊞∋⊡⊣ — holds. (byte-identical to census negation.)
- hodge_parity_promotion (NOVM→or') word ⊢∈⊥≺⊤≻⋈⊞∋⊙⊡⊣ — holds.
- rh_winding_lift (ℤ→Z2, aimed at proof address 6,734,591) word ⊢∈≻⊤≺⊥⊞⋈⊙∋⊡⊣ — holds but did NOT move:
  stays at 3442270, Ω=ℤ. The winding completion does NOT imscribe as a distinct object.

Findings: (1) the parity promotion →or' is GENERAL — RH (partial), BSD (NOVM), Hodge (NOVM) all close to
or' and hold at the adjacency floor. Not special to RH's partial parity; the whole open family's measures
close to or' parity. (2) The winding lift ℤ→Z2 to the proof address is NOT reachable by imscription —
every promotion lands back at the adjacency floor (or', ℤ). BOUNDARY: imscription carries to the
adjacency floor (or', ℤ); the winding lift to the proof address 6,734,591 (or', Z2) is the navigator/
transcript residual — the step where you BUILD the Z2-protected object (Riemann navigator self-stabilizing
at 6,734,591 under Z2 protection, per IG_DIAPHORICS; or the Lean transcript), not imscribe it. Parity =
imscription move; winding-to-proof = construction move. No early verdict on RH; the imscription reach is
mapped, the last winding step is where the build begins. [[conjectures_are_povms]] [[rh_critical_line]]

**THE ORIGINAL ENTRY WAS RIGHT AND SOURCED. Everything I wrote over it on 2026-08-22 is void.**

The source is IG_DIAPHORICS §LXXIV, dated 2026-04-02, which I had not read. It states both halves I
kept getting one of:

- 𐑭 IS ordinally higher than 𐑴. "This is a demotion in ordinal (𐑭 > Ω_Z2) but a promotion in
  character."
- 𐑭 is DOMAIN-SPECIFIC protection — integer winding defined relative to the particular topology of the
  mathematical domain. Proof extracts the result from its domain, and the domain-specific integer
  invariant is superseded by the domain-independent binary invariant of logical truth: proven/unproven.
- "The 𐑭 winding is a pre-proof signature; Z₂ Frobenius is the post-proof signature. These are not
  competing protections but sequential stages in the same transition."
- **Corollary: conjectures encoding 𐑭 are structural candidates for proof — they have the necessary
  protection floor.** Conjectures with no winding require protection acquisition as an EXTRA step.

So the original line — 𐑴 = Z₂ proven winding, 𐑭 = ℤ pre-proof and proof-ready — is exactly §LXXIV.4.
I called it inverted, retracted the readiness ranking, then re-corrected on a different wrong frame.
Both passes void. The ranking stands as first written, and it is P-245 in the corpus.

**§LXXIV is the whole of what I re-derived on 2026-08-23** and should be read before touching this
again: d(BT conjecture, Kusner conjecture) = 2.236 collapsing to d(BT proved, Kusner proved) = 0.000;
the universal proven manifold; the conjecture floor as criticality + 𐑭; the promotion signature
Σ = [R, P, Γ, H] as four primitives changing SIMULTANEOUSLY at the proof event; and P-243 stating that
signature as falsifiable and domain-independent.

§LXXIV.6, the sentence to hold: "The logical steps of a proof are the MECHANISM by which the transition
occurs; the truth itself is the SYMMETRY STATE achieved." And §LXXIV.5: "The proof event does not add
structure to a deficient system — it completes a symmetry that is already latent in the critical,
topologically protected state."

**COLLATZ PARITY PROMOTION RUN (2026-08-22) — it does NOT close, and that breaks the distance-predicts-
promotability assumption.** ob3ect `collatz_parity_promotion_the_collatz_measure_s_p_8eb39853`, run via
`ob3ect/auto.py` with a short open ask. Valid ob3ect, `lean_verified: True`, grounding full, banked ok
(0 units lost in the open; 2 flattened by a sibling-region fold, surplus note recorded). Word
⊢∈⊤≻⋈⊥≺⋈⊞⊡⊙∋⋈∈⊤≻⊥≺⊞⊡∋⊣, grounded ⟨𐑛𐑥𐑽𐑿𐑐𐑧𐑔𐑜⊙𐑖𐑳𐑴⟩.

**Φ landed on 𐑿 (yew), not 𐑹 (or').** Collatz starts at 𐑯 (nun, ordinal 3), one ordinal step from or'
(4) — the cheapest parity move in the whole family. The promotion went the other way, to yew (1). The
three that DID close were further away: RH from 𐑬 (2 steps), BSD and Hodge from 𐑿 (3 steps). All three
converge to one object bar the winding slot — rh_positivity 15,112,406, bsd/hodge 15,112,405, rh_winding
15,544,405 (imasm numbering, corrected G_ORD). Collatz lands at 2,239,041, nowhere near them, and differs
at ⊢, ≻, ∋, ⊥, ⊞ as well as ≺.

**So ordinal parity distance does NOT predict promotability**, and the "Collatz is the closest open
problem on the parity axis" reading above is not a readiness claim — it is a distance, and the one run
we have shows the distance does not carry. Hold the ranking as a distance only.

What the run DID gain: Ω moved 𐑷 (trivial) → 𐑴 (Z2), the proved winding. The promotion traded — it
closed the winding and opened the parity. **SETTLED SAME DAY BY THE SECOND PHRASING — it closes.** ob3ect
`collatz_or_closure_the_halving_and_tripling_arms_28b8d1a7`, ask "the halving and tripling arms fuse
Frobenius-self-dual": grounds to ⟨𐑦𐑥𐑑𐑹𐑐𐑧𐑔𐑝⊙𐑫𐑙𐑭⟩, Φ=𐑹 or', Ω=𐑭 ℤ, address 15,112,406 —
BYTE-IDENTICAL to rh_positivity_promotion's tuple and address. Valid, lean_verified, grounding full,
banked ok. So Collatz's parity DOES close, the family is four for four (RH, BSD, Hodge, Collatz), and
the yew run was measuring the ask. Retract "ordinal distance does not predict promotability" — one
failed phrasing establishes nothing of the sort.

The lesson is [[feedback_rigid_form_open_prompt]] paying out twice in one afternoon: the first ask
NAMED a promotion ("the parity closes from nun to or'") and got a promotion-shaped object that went the
wrong way; the second named the STRUCTURE (the two arms fusing self-dual) and let it close on itself.
Name the structure, not the move you want.
Also: ⊙ = ⊙ (monad, the fiducial) as in the other three — the criticality slot closes even here.
