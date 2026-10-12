import Imscribing.Paraconsistent.CombinatoryDynamics

namespace Imscribing.Paraconsistent.DialecticResident
open CombinatoryReentry CombinatoryDynamics Temporal.Semantics

/-- Compile the existing certified Curry Y without introducing a second definition. -/
def emitPostfix : Term → List String
  | .i => ["I"]
  | .k => ["K"]
  | .s => ["S"]
  | .app f x => emitPostfix f ++ emitPostfix x ++ ["@"]
  | _ => []

def headCount : Nat → Term → Option Nat
  | 0, _ => none
  | fuel + 1, term =>
    match term with
    | .app (.app (.var 37) _) (.var 38) => some 0
    | _ => match reduceOnce term with
      | none => none
      | some next => (headCount fuel next.val).map Nat.succ

def residentPrefixContractions : Option Nat :=
  headCount 256 (.app (.app Y (.var 37)) (.var 38))

/-- A source residual can be square only on this retained low-word support. -/
def squareSupport (residue : Fin 16) : Bool :=
  (List.finRange 16).any (fun root => root.val * root.val % 16 == residue.val)

theorem square_support_complete (root : Fin 16) :
    squareSupport ⟨root.val * root.val % 16, Nat.mod_lt _ (by decide)⟩ = true := by
  revert root
  decide

theorem full_width_square_support (root : Nat) :
    squareSupport ⟨root * root % 16, Nat.mod_lt _ (by decide)⟩ = true := by
  simpa only [← Nat.mul_mod] using
    square_support_complete ⟨root % 16, Nat.mod_lt _ (by decide)⟩

/-- The full residual/carry relation is preserved by one boundary advance. -/
theorem residual_advance (a n residual : Nat) (h : a * a = n + residual) :
    (a + 1) * (a + 1) = n + (residual + 2 * a + 1) := by
  simp only [Nat.add_mul, Nat.mul_add, Nat.mul_one, Nat.one_mul]
  omega

theorem residual_retreat (a n residual : Nat) (ha : 1 ≤ a)
    (h : a * a = n + residual) (hn : n ≤ (a - 1) * (a - 1)) :
    (a - 1) * (a - 1) = n + (residual - (2 * a - 1)) := by
  have hbase : (a - 1) * (a - 1) = n + ((a - 1) * (a - 1) - n) := by omega
  have hnext := residual_advance (a - 1) n ((a - 1) * (a - 1) - n) hbase
  have hstep : a - 1 + 1 = a := by omega
  rw [hstep] at hnext
  omega

/-- Resident payloads are parameters to the existing certified Y equation. -/
theorem resident_unfold (operator payload : Term) :
    Steps (.app (.app Y operator) payload)
      (.app (.app operator (unfoldState operator)) payload) :=
  (Y_unfolds operator).left payload

/-- Negation moves the two designated single-pole phases through one orbit. -/
theorem two_phase_orbit :
    FDE.liftNegation (Lean.Sixteen3.ofMask 2) = Lean.Sixteen3.ofMask 4 ∧
    FDE.liftNegation (Lean.Sixteen3.ofMask 4) = Lean.Sixteen3.ofMask 2 := by
  decide

end Imscribing.Paraconsistent.DialecticResident
