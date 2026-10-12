import Imscribing.Paraconsistent.CombinatoryReentry

namespace Imscribing.Paraconsistent.DivisorReentry
open CombinatoryReentry
local infixl:70 " @@ " => Term.app

abbrev δ (N k : Nat) (contradictory : Bool := false) : Lean.Sixteen3 :=
  divisorEvidence N k contradictory
abbrev π_T := Lean.Sixteen3.assertsTrue
abbrev π_F := Lean.Sixteen3.assertsFalse

/-- The recursive program is the existing unrestricted Curry Y applied to Φ_N. -/
def factorProgram (N k : Nat) (contradictory : Bool := false) : Term :=
  Y @@ .divisor N contradictory @@ .number k

def factorsProgram (N : Nat) (contradictory : Bool := false) : Term :=
  Y @@ .divisors N contradictory @@ .number 2 @@ .numbers []

/-- Execute contractions to a normal form without a caller-supplied fuel bound. -/
partial def evaluate (program : Term) : Term :=
  match reduceOnce program with
  | none => program
  | some next => evaluate next.val

/-- Count Y contractions and sample the residual syntax at powers of two. -/
partial def termSize : Term → Nat
  | .app f x => 1 + termSize f + termSize x
  | .numbers values => values.length + 1
  | _ => 1

partial def evaluateProfileAux (program : Term) (steps nextSample : Nat)
    (samples : List (Nat × Nat)) : Nat × List (Nat × Nat) × Term :=
  match reduceOnce program with
  | none => (steps, samples.reverse, program)
  | some next =>
      let nextSteps := steps + 1
      if nextSteps == nextSample then
        evaluateProfileAux next.val nextSteps (nextSample * 2)
          ((nextSteps, termSize next.val) :: samples)
      else
        evaluateProfileAux next.val nextSteps nextSample samples

def evaluateProfile (program : Term) : Nat × List (Nat × Nat) × Term :=
  evaluateProfileAux program 0 1 []

def factorFrom (N d : Nat) (contradictory : Bool := false) : Nat :=
  match evaluate (factorProgram N d contradictory) with
  | .number k => k
  | _ => panic! "divisor re-entry did not return a numeral"

def factor (N : Nat) (contradictory : Bool := false) : Nat :=
  factorFrom N 2 contradictory

def factors (N : Nat) (contradictory : Bool := false) : List Nat :=
  match evaluate (factorsProgram N contradictory) with
  | .numbers values => values.filter (fun k => 1 < k && k < N)
  | _ => panic! "divisor re-entry did not return its accumulator"

/-- Contradictory evidence retains truth designation exactly for divisors. -/
theorem designation_iff (N k : Nat) (b : Bool) :
    π_T (δ N k b) = true ↔ k ∣ N := by
  by_cases h : k ∣ N
  · cases b <;> simp [δ, divisorEvidence, π_T, Lean.Sixteen3.assertsTrue, h]
  · simp [δ, divisorEvidence, π_T, Lean.Sixteen3.assertsTrue, h]

theorem contradictory_divisor_retains_both (N k : Nat) (h : k ∣ N) :
    π_T (δ N k true) = true ∧ π_F (δ N k true) = true := by
  simp [δ, divisorEvidence, π_T, π_F,
    Lean.Sixteen3.assertsTrue, Lean.Sixteen3.assertsFalse, h]

/-- Adding a falsity-only observation preserves the truth projection for all 16 states. -/
theorem falsity_join_preserves_truth (x : Lean.Sixteen3) :
    π_T (Lean.Sixteen3.join_i x ⟨false, false, true, false⟩) = π_T x := by
  simp [π_T, Lean.Sixteen3.join_i, Lean.Sixteen3.assertsTrue]

/-- The compiled program has the unrestricted Y fixed-point equation. -/
theorem factor_fixedpoint (N : Nat) (b : Bool) :
    Conversion (Y @@ .divisor N b) (.divisor N b @@ (Y @@ .divisor N b)) :=
  Y_fixedpoint (.divisor N b)

/-- A seeded nontrivial divisor returns itself after certified Y unfolding. -/
theorem seeded_reduces (N d : Nat) (b : Bool)
    (hd : 1 < d) (hN : d < N) (hdiv : d ∣ N) :
    Steps (factorProgram N d b) (.number d) := by
  have ha : acceptsDivisor N d b = true := (designation_iff N d b).mpr hdiv
  have hstop : ¬d ≥ N := Nat.not_le.mpr hN
  exact ((Y_unfolds (.divisor N b)).left (.number d)).trans
    (by simpa [factorBody, hstop, hd, ha] using
      Steps.single (Step.factor N b (unfoldState (.divisor N b)) d))

/-- Prime/trivial sentinel is N; the caller can discard it. -/
theorem terminal_reduces (N k : Nat) (b : Bool) (h : k ≥ N) :
    Steps (factorProgram N k b) (.number N) :=
  ((Y_unfolds (.divisor N b)).left (.number k)).trans
    (by simpa [factorBody, h] using
      Steps.single (Step.factor N b (unfoldState (.divisor N b)) k))

end Imscribing.Paraconsistent.DivisorReentry
