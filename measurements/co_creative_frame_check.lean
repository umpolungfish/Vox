import Imscribing.Paraconsistent.SixteenThreeTrilattice

/- Executable obligations for the supplied co-creative frame overview.
   Count coordinates are readouts; the original ordered tape is retained. -/
namespace CoCreativeFrameCheck

example : Fintype.card Reg16_3 = 16 := by decide
example (x : Reg16_3) : Reg16_3.invol (Reg16_3.invol x) = x :=
  Reg16_3.invol_involutive x

-- Any lattice join is idempotent, so the proposed top-coordinate join
-- cannot turn the two top counts 2,2 into R's top count 1.
theorem addition_coordinate_mismatch
    (join : Nat → Nat → Nat) (idem : ∀ n, join n n = n) :
    join 2 2 ≠ 1 := by
  rw [idem]
  decide

-- Under the usual order on counts, the overview's multiplication and
-- subtraction disagree with its declared S and T in these coordinates.
example : min (1 : Nat) 2 ≠ 2 := by decide
example : ((2 : Int) - 1, (2 : Int) - 2) = (1, 0) := by decide
example : ((2 : Int) - 1, (2 : Int) - 2) ≠ (2, 1) := by decide

def numeral : List Bool → Nat
  | [] => 0
  | bit :: rest => (if bit then 1 else 0) + 2 * numeral rest

def counts (tape : List Bool) : Nat × Nat :=
  (tape.count true, tape.count false)

-- Same punctum counts and frame, distinct cell-binary numerals.
example : counts [true, false, true] = counts [false, true, true] := by decide
example : numeral [true, false, true] = 5 := by decide
example : numeral [false, true, true] = 6 := by decide
theorem counts_lose_position : ¬ Function.Injective counts := by
  intro h
  have bad := h (show counts [true, false, true] =
    counts [false, true, true] by decide)
  have different : ([true, false, true] : List Bool) ≠ [false, true, true] := by decide
  exact different bad

-- A reversible evaluation frame transports the operation as well as operands.
-- This is the return-map obligation for a pair produced in that frame.
def frameMul {Frame : Type} (e : Nat ≃ Frame) (u v : Frame) : Frame :=
  e (e.symm u * e.symm v)

theorem factor_pair_returns {Frame : Type} (e : Nat ≃ Frame)
    (n : Nat) (u v : Frame) (closed : frameMul e u v = e n) :
    e.symm u * e.symm v = n := by
  exact e.injective closed

end CoCreativeFrameCheck
