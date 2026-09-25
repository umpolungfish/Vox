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

-- Return is a local carry sweep, without decoding the entire frame to Nat.
def normalizeFrame : List Nat → Nat → List Bool
  | [], carry => emitBits carry
  | digit :: rest, carry =>
    ((digit + carry) % 2 == 1) :: normalizeFrame rest ((digit + carry) / 2)

def returnFrame (frame : List Nat) : List Bool := normalizeFrame frame 0

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

theorem normalizeFrame_value (frame : List Nat) (carry : Nat) :
    numeral (normalizeFrame frame carry) = coefficientValue frame + carry := by
  induction frame generalizing carry with
  | nil => simp [normalizeFrame, coefficientValue, emitBits_value]
  | cons digit rest ih =>
    simp only [normalizeFrame, numeral, ih, coefficientValue, beq_iff_eq]
    have remainder := Nat.mod_lt (digit + carry) (by decide : 0 < 2)
    split_ifs <;> omega

theorem multiplication_returns (left right : List Bool) :
    numeral (returnFrame (multiplyFrame (enterFrame left) (enterFrame right))) =
      numeral left * numeral right := by
  simp [returnFrame, normalizeFrame_value, multiplyFrame_value, enterFrame_value]

-- Whole-word operations commute for the supplied examples and for the
-- carry-producing composition P+T=Q that the isolated count projection lost.
example : returnFrame (addFrame (enterFrame blockP) (enterFrame blockQ)) = blockR := by
  simp [returnFrame, normalizeFrame, blockP, blockQ, blockR, enterFrame, addFrame, emitBits]
example : returnFrame (addFrame (enterFrame blockP) (enterFrame blockT)) = blockQ := by
  simp [returnFrame, normalizeFrame, blockP, blockT, blockQ, enterFrame, addFrame, emitBits]
example : returnFrame (multiplyFrame (enterFrame blockP) (enterFrame blockQ)) = blockS := by
  simp [returnFrame, normalizeFrame, blockP, blockQ, blockS, enterFrame, multiplyFrame,
    addFrame, emitBits]
example : returnFrame (multiplyFrame (enterFrame [true, true])
    (enterFrame [true, true])) = [true, false, false, true] := by
  simp [returnFrame, normalizeFrame, enterFrame, multiplyFrame, addFrame, emitBits]

-- Signed coefficients retain borrows during cancellation. The divisor is an
-- anchored candidate word; this operation does not select that candidate.
def signedValue : List Int → Int
  | [] => 0
  | digit :: rest => digit + 2 * signedValue rest

def cancelContribution : List Int → List Int → List Int
  | [], right => right.map (- ·)
  | left, [] => left
  | a :: left, b :: right => (a - b) :: cancelContribution left right

def carryInto (carry : Int) : List Int → List Int
  | [] => [carry]
  | digit :: rest => (digit + carry) :: rest

def shiftResidual : List Int → List Int
  | [] => []
  | digit :: rest => carryInto (digit / 2) rest

def signedFrame (word : List Bool) : List Int :=
  word.map fun bit => if bit then 1 else 0

-- One output cell per source cell. At an odd anchor the next complementary
-- cell is uniquely the parity of the residual's lowest coefficient.
def recoverComplement (divisor : List Int) : List Bool → List Int → List Bool × List Int
  | [], residual => ([], residual)
  | _ :: sourceRest, residual =>
    let bit := residual.headD 0 % 2 == 1
    let cancelled := if bit then cancelContribution residual divisor else residual
    let next := recoverComplement divisor sourceRest (shiftResidual cancelled)
    (bit :: next.1, next.2)

def complementWord (candidate source : List Bool) : List Bool × List Int :=
  recoverComplement (signedFrame candidate) source (signedFrame source)

theorem cancelContribution_value (left right : List Int) :
    signedValue (cancelContribution left right) = signedValue left - signedValue right := by
  induction left generalizing right with
  | nil =>
    induction right with
    | nil => rfl
    | cons digit rest ih =>
      simp [cancelContribution, signedValue] at ih ⊢
      omega
  | cons digit rest ih =>
    cases right with
    | nil => simp [cancelContribution, signedValue]
    | cons other tail => simp [cancelContribution, signedValue, ih]; ring

theorem carryInto_value (carry : Int) (frame : List Int) :
    signedValue (carryInto carry frame) = signedValue frame + carry := by
  cases frame <;> simp [carryInto, signedValue]
  omega

theorem shiftResidual_value (frame : List Int) (even : frame.headD 0 % 2 = 0) :
    2 * signedValue (shiftResidual frame) = signedValue frame := by
  cases frame with
  | nil => simp [shiftResidual, signedValue]
  | cons digit rest =>
    simp only [List.headD_cons] at even
    simp only [shiftResidual, carryInto_value, signedValue]
    omega

-- These cancellation tests supply an explicit candidate and exercise borrows.
example : complementWord [true, true] [true, false, true, false, true] =
    ([true, true, true, false, false], [0]) := by decide
example : complementWord [true, true] [true, false, false, true] =
    ([true, true, false, false], [0]) := by decide
example : complementWord [true, true] [true, false, true, true] =
    ([true, true, true, true], [-2]) := by decide

-- Width is supplied by the word itself. Exercise the same cancellation beyond
-- machine-word boundaries without any decimal conversion in the operation.
set_option maxRecDepth 100000 in
set_option maxHeartbeats 0 in
example : ∀ width ∈ ([65, 129, 257] : List Nat),
    complementWord [true, true]
      ([true, true] ++ List.replicate (width - 2) false ++ [true, true]) =
    ([true] ++ List.replicate (width - 1) false ++ [true, false], [0]) := by decide

-- The phase arm supplies residue words, not factor words. Its difference
-- closure selects the first register; cancellation constructs the second.
-- Nat.gcd here specifies the existing phase arm's Euclidean closure.
def recoverPhaseWords (source currentHalf earlierHalf : List Bool) :
    Option (List Bool × List Bool) :=
  let n := numeral source
  let x := numeral currentHalf
  let y := numeral earlierHalf
  let difference := if x ≥ y then x - y else y - x
  let seed := Nat.gcd difference n
  if 1 < seed && seed < n && seed % 2 == 1 && (x * x) % n == (y * y) % n then
    let candidate := emitBits seed
    let recovered := complementWord candidate source
    if signedValue recovered.2 == 0 then some (candidate, recovered.1) else none
  else none

-- N=21, current half=8, earlier half=1. Both factor words are outputs.
theorem phase21_return : recoverPhaseWords [true, false, true, false, true]
    [false, false, false, true] [true] =
    some ([true, true, true], [true, true, false, false, false]) := by
  simp [recoverPhaseWords, numeral, emitBits, complementWord, recoverComplement,
    signedFrame, cancelContribution, shiftResidual, carryInto, signedValue]

-- Coincident phase halves have no proper difference closure.
example : recoverPhaseWords [true, false, true, false, true] [true] [true] = none := by
  simp [recoverPhaseWords, numeral]

-- FOUR × FOUR is carried by the kernel's actual four-lane register.
abbrev FourCell := Bool × Bool

def encodePair (pair : FourCell × FourCell) : Reg16_3 :=
  Reg16_3.mk pair.1.1 pair.1.2 pair.2.1 pair.2.2

def reversePair (register : Reg16_3) : FourCell × FourCell :=
  ((register.bigT, register.bigF), (register.smallT, register.smallF))

theorem pair_return (pair : FourCell × FourCell) :
    reversePair (encodePair pair) = pair := rfl

theorem register_return (register : Reg16_3) :
    encodePair (reversePair register) = register := rfl

theorem paired_stream_return (stream : List (FourCell × FourCell)) :
    (stream.map encodePair).map reversePair = stream := by
  simp only [List.map_map, Function.comp_def, pair_return]
  exact List.map_id stream

-- Two cells from each factor per register. Missing high cells are zero;
-- the stream grows with the longer word and has no fixed-width register cap.
def pairFactorWords (left right : List Bool) : List Reg16_3 :=
  if left.isEmpty && right.isEmpty then [] else
    encodePair ((left.headD false, (left.drop 1).headD false),
      (right.headD false, (right.drop 1).headD false)) ::
      pairFactorWords (left.drop 2) (right.drop 2)
termination_by left.length + right.length
decreasing_by
  simp_all only [Bool.and_eq_true, List.isEmpty_iff, not_and]
  simp only [List.length_drop]
  cases left <;> cases right <;> simp_all
  all_goals omega

def extractPairStream (stream : List Reg16_3) : List Bool × List Bool :=
  (stream.flatMap fun r => [r.bigT, r.bigF],
   stream.flatMap fun r => [r.smallT, r.smallF])

def pairedPhaseReturn (source currentHalf earlierHalf : List Bool) :
    Option (List Bool × List Bool) :=
  (recoverPhaseWords source currentHalf earlierHalf).map fun pair =>
    extractPairStream (pairFactorWords pair.1 pair.2)

-- The diagram's return preserves both numeral readouts, including the
-- unequal factor widths. Padding remains high-order zero cells.
example : (extractPairStream (pairFactorWords [true, true, true]
    [true, true, false, false, false])).map numeral numeral = (7, 3) := by
  simp [pairFactorWords, encodePair, extractPairStream, Reg16_3.mk,
    Reg16_3.bigT, Reg16_3.bigF, Reg16_3.smallT, Reg16_3.smallF, numeral]

example : (pairedPhaseReturn [true, false, true, false, true]
    [false, false, false, true] [true]).map (fun pair =>
      (numeral pair.1, numeral pair.2)) = some (7, 3) := by
  unfold pairedPhaseReturn
  rw [phase21_return]
  simp only [Option.map_some]
  simp [pairFactorWords, encodePair, extractPairStream, Reg16_3.mk,
    Reg16_3.bigT, Reg16_3.bigF, Reg16_3.smallT, Reg16_3.smallF, numeral]

example : (pairedPhaseReturn [true, true, false, false, false, true]
    [false, true, true] [true]).map (fun pair =>
      (numeral pair.1, numeral pair.2)) = some (5, 7) := by
  have returned : recoverPhaseWords [true, true, false, false, false, true]
      [false, true, true] [true] =
      some ([true, false, true], [true, true, true, false, false, false]) := by
    simp [recoverPhaseWords, numeral, emitBits, complementWord, recoverComplement,
      signedFrame, cancelContribution, shiftResidual, carryInto, signedValue]
  unfold pairedPhaseReturn
  rw [returned]
  simp only [Option.map_some]
  simp [pairFactorWords, encodePair, extractPairStream, Reg16_3.mk,
    Reg16_3.bigT, Reg16_3.bigF, Reg16_3.smallT, Reg16_3.smallF, numeral]

end CoCreativeFrameCheck
