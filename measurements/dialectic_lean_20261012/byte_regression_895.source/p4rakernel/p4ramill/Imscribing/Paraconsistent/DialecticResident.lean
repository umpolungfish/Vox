import Imscribing.Paraconsistent.CombinatoryDynamics

namespace Imscribing.Paraconsistent.DialecticResident
open CombinatoryReentry CombinatoryDynamics Temporal.Semantics

/-- Compile the existing certified Curry Y without introducing a second definition. -/
def emitPostfix : Term → List String
  | .i => ["I"]
  | .k => ["K"]
  | .s => ["S"]
  | .var 37 => ["OP"]
  | .var 38 => ["PAY"]
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

def prefixTrace : Nat → Term → List (List String)
  | 0, term => [emitPostfix term]
  | fuel + 1, term =>
    emitPostfix term :: match term with
    | .app (.app (.var 37) _) (.var 38) => []
    | _ => match reduceOnce term with
      | none => []
      | some next => prefixTrace fuel next.val

def residentPrefixTrace : List (List String) :=
  prefixTrace 256 (.app (.app Y (.var 37)) (.var 38))

/-- The primitive retains its entire payload at a tail re-entry, as divisorScan
    does for the existing divisor functional. The payload type is unrestricted. -/
inductive CompactState (α β : Type) where
  | active : α → CompactState α β
  | closed : β → CompactState α β
  | neutral : α → CompactState α β
  | invalid : CompactState α β

def compactStep {α β : Type} (step : α → CompactState α β) :
    CompactState α β → CompactState α β
  | .active object => step object
  | terminal => terminal

theorem compact_reentry {α β : Type} (step : α → CompactState α β)
    (object next : α) (h : step object = .active next) :
    compactStep step (.active object) = .active next := h

theorem compact_closed_retains {α β : Type} (step : α → CompactState α β)
    (closure : β) : compactStep step (.closed closure) = .closed closure := rfl

theorem compact_feedback_unfold {α β : Type} (step : α → CompactState α β)
    (object : α) (tick : Nat) :
    Combinators.yUnfolding (compactStep step) (.active object) (tick + 1) =
    compactStep step (Combinators.yUnfolding (compactStep step) (.active object) tick) := rfl

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


/-- One-byte support rejects residuals before full-width square-root work. -/
def squareSupport256 (residue : Fin 256) : Bool :=
  (List.finRange 256).any (fun root => root.val * root.val % 256 == residue.val)

theorem square_support_256_complete (root : Fin 256) :
    squareSupport256 ⟨root.val * root.val % 256, Nat.mod_lt _ (by decide)⟩ = true := by
  unfold squareSupport256
  apply List.any_eq_true.mpr
  exact ⟨root, List.mem_finRange root, by simp⟩

theorem full_width_square_support_256 (root : Nat) :
    squareSupport256 ⟨root * root % 256, Nat.mod_lt _ (by decide)⟩ = true := by
  simpa only [← Nat.mul_mod] using
    square_support_256_complete ⟨root % 256, Nat.mod_lt _ (by decide)⟩

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
