import Imscribing.Paraconsistent.SixteenThreeTrilattice
import Mathlib.Tactic

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

-- The supplied commuting square has A = D1 + 1, B = D2 + 1,
-- D2 = D1 + 2, and A+B = D1*D2. Its arithmetic valuation is unique.
theorem supplied_square_value (x : Nat)
    (closure : (x + 1) + (x + 2 + 1) = x * (x + 2)) :
    x = 2 ∧ x * (x + 2) = 8 := by
  have square : x * x = 4 := by nlinarith
  have hx : x = 2 := by nlinarith
  subst x
  decide

-- Factoring the product frame of that same example returns its two
-- specified predecessor valuations in the source frame.
example : (2 + 1) + (4 + 1) = 2 * 4 := by decide

-- Block 2, copied as LSB-first cells: false = top, true = bottom.
def blockP : List Bool := [false, false, true]
def blockQ : List Bool := [true, false, false, true]
def blockR : List Bool := [true, false, true, true]
def blockS : List Bool := [false, false, true, false, false, true]
def blockT : List Bool := [true, false, true]

example : (numeral blockP, numeral blockQ, numeral blockR,
    numeral blockS, numeral blockT) = (4, 9, 13, 36, 5) := by decide
example : numeral blockP + numeral blockQ = numeral blockR := by decide
example : numeral blockP * numeral blockQ = numeral blockS := by decide
example : numeral blockQ - numeral blockP = numeral blockT := by decide

-- The three displayed count readings hold on the supplied examples.
example : (counts blockP).1 + (counts blockQ).1 = (counts blockR).1 := by decide
example : (counts blockP).2 * (counts blockQ).2 = (counts blockS).2 := by decide
example : (counts blockQ).1 - (counts blockP).1 = (counts blockT).2 := by decide

-- Composing the supplied operands tests the proposed general additive square.
-- The numeral path closes, but the count projection does not preserve addition.
theorem block_composition_count_mismatch :
    numeral blockP + numeral blockT = numeral blockQ ∧
    (counts blockP).1 + (counts blockT).1 ≠ (counts blockQ).1 := by decide

-- Q and T have the same bottom count and different top counts. A scalar
-- bottom-to-top arrow therefore needs the retained word as additional input.
theorem count_frame_requires_word :
    ¬ ∃ shift : Nat → Nat, ∀ word : List Bool,
      shift (counts word).1 = (counts word).2 := by
  rintro ⟨shift, rule⟩
  have hq := rule blockQ
  have ht := rule blockT
  change shift 2 = 2 at hq
  change shift 2 = 1 at ht
  omega

/- Full-word evaluation frame. Each coefficient retains its original binary
   position. Addition and convolution may produce coefficients larger than one;
   their carries are normalized only on return to the numeral word. -/
def coefficientValue : List Nat → Nat
  | [] => 0
  | head :: tail => head + 2 * coefficientValue tail

def enterFrame (word : List Bool) : List Nat :=
  word.map fun bit => if bit then 1 else 0

def addFrame : List Nat → List Nat → List Nat
  | [], right => right
  | left, [] => left
  | a :: left, b :: right => (a + b) :: addFrame left right

def multiplyFrame : List Nat → List Nat → List Nat
  | [], _ => []
  | a :: left, right =>
    addFrame (right.map (a * ·)) (0 :: multiplyFrame left right)

def emitBits (n : Nat) : List Bool :=
  if n = 0 then [] else (n % 2 == 1) :: emitBits (n / 2)
termination_by n

def returnFrame (frame : List Nat) : List Bool := emitBits (coefficientValue frame)

theorem enterFrame_value (word : List Bool) :
    coefficientValue (enterFrame word) = numeral word := by
  induction word with
  | nil => rfl
  | cons bit rest ih =>
    change (if bit then 1 else 0) + 2 * coefficientValue (enterFrame rest) =
      (if bit then 1 else 0) + 2 * numeral rest
    rw [ih]

theorem addFrame_value (left right : List Nat) :
    coefficientValue (addFrame left right) =
      coefficientValue left + coefficientValue right := by
  induction left generalizing right with
  | nil => simp [addFrame, coefficientValue]
  | cons a rest ih =>
    cases right with
    | nil => simp [addFrame, coefficientValue]
    | cons b tail => simp [addFrame, coefficientValue, ih]; omega

theorem scaleFrame_value (a : Nat) (word : List Nat) :
    coefficientValue (word.map (a * ·)) = a * coefficientValue word := by
  induction word with
  | nil => simp [coefficientValue]
  | cons b rest ih => simp [coefficientValue, ih]; ring

theorem multiplyFrame_value (left right : List Nat) :
    coefficientValue (multiplyFrame left right) =
      coefficientValue left * coefficientValue right := by
  induction left with
  | nil => simp [multiplyFrame, coefficientValue]
  | cons a rest ih =>
    simp [multiplyFrame, addFrame_value, scaleFrame_value, coefficientValue, ih]
    ring

theorem emitBits_value (n : Nat) : numeral (emitBits n) = n := by
  induction n using Nat.strong_induction_on with
  | h n ih =>
    rw [emitBits]
    split_ifs with hn
    · simp [numeral, hn]
    · have smaller : n / 2 < n := by omega
      simp only [numeral, ih (n / 2) smaller, beq_iff_eq]
      have remainder := Nat.mod_lt n (by decide : 0 < 2)
      split_ifs <;> omega

theorem multiplication_returns (left right : List Bool) :
    numeral (returnFrame (multiplyFrame (enterFrame left) (enterFrame right))) =
      numeral left * numeral right := by
  simp [returnFrame, emitBits_value, multiplyFrame_value, enterFrame_value]

-- Whole-word operations commute for the supplied examples and for the
-- carry-producing composition P+T=Q that the isolated count projection lost.
example : returnFrame (addFrame (enterFrame blockP) (enterFrame blockQ)) = blockR := by
  simp [returnFrame, blockP, blockQ, blockR, enterFrame, addFrame, coefficientValue, emitBits]
example : returnFrame (addFrame (enterFrame blockP) (enterFrame blockT)) = blockQ := by
  simp [returnFrame, blockP, blockT, blockQ, enterFrame, addFrame, coefficientValue, emitBits]
example : returnFrame (multiplyFrame (enterFrame blockP) (enterFrame blockQ)) = blockS := by
  simp [returnFrame, blockP, blockQ, blockS, enterFrame, multiplyFrame, addFrame,
    coefficientValue, emitBits]
example : returnFrame (multiplyFrame (enterFrame [true, true])
    (enterFrame [true, true])) = [true, false, false, true] := by
  simp [returnFrame, enterFrame, multiplyFrame, addFrame, coefficientValue, emitBits]

end CoCreativeFrameCheck
