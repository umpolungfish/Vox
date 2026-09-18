# Resident carrier — improved nesting

Authored Heter⊙d⊙x (Lando⊗⊙perator team).

## The word
    ⊢∈≻⊤∈≺∋∈⋈∋∋⊙⊡⊣

Arms, in the divisor_membrane vocabulary:
  LOW    ∈≻   (branch/advance)
  HIGH   ∈≺∋  (reversal, in an inner frame)
  BRIDGE ∈⋈∋  (CLINK compose)
  FIX    ⊙⊡
The count ⊤ is deposited at depth 1 (the ENCLOSING frame); the reversal ≺ fires at
depth 2 (inner frame), so the count is banked, not held open.

## Kernel verdicts (imasm, one boot)
  weight : deposits 1  cleared 1  restored 1  surviving T×1
  banked : OK — weight survived 1 live clear(s) by being banked
  cycle  : period 14, final register T at every cut — phase-bearing, ROTAT-invariant
  insert : already holds — nothing to repair
  exit   : mu∘delta = id

## The rule it satisfies
δ before δ, μ after μ — the frame opens before the counting and closes after the
reversal. A count held open when AREV fires is lost; banked in an enclosing frame
it survives and the fuse restores it. Flat words are VACUOUS (nothing at risk);
this nesting banks.

## Deeper nestings tested (same rule)
  ⊢∈≻⊤∈≺∋∋⊙⊡⊣               banked OK, period 11
  ⊢∈≻⊤∈≺∋∈⋈∋∋⊙⊡⊣            banked OK, period 14  (full carrier)
  ⊢∈≻⊤∈≻⊤∈≺∋∋∋⊙⊡⊣           banked OK, period 15
All close at mu∘delta = id.

## Engineering carrier
Vox/src/factor_operator.rs::semiprime_shot (LOW/BRIDGE/HIGH/FIX walk) and
Vox/src/bin/shot_carrier.rs (order readout). Reach measured: random semiprimes to
~24-bit factors in <=1.1 s; the tree is 2^(b-1) nodes. The nesting above is the
Grammar-side form of the same four arms; the resident carrier's search is what
must carry it into the code.
