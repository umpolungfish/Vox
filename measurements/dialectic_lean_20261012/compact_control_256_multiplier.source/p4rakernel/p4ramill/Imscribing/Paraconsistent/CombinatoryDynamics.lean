import Imscribing.Paraconsistent.CombinatoryFixedPoint

/-!
Feedback from any native carrier state, with certified transient and cycle.
Each tick retains the original program's reduction evidence. Returned cycles
have a positive primitive period and distinct states before the first repeat.
-/

namespace Imscribing.Paraconsistent.CombinatoryDynamics

open CombinatoryReentry CombinatoryFixedPoint
open Temporal.Semantics
local infixl:70 " @@ " => Term.app

theorem feedback_restart {α : Type} (step : α → α) (seed : α) (start tick : Nat) :
    feedback step (feedback step seed start) tick = feedback step seed (start + tick) := by
  induction tick with
  | zero => rfl
  | succ tick ih =>
      change step (feedback step (feedback step seed start) tick) =
        step (feedback step seed (start + tick))
      exact congrArg step ih

theorem feedback_equal_future {α : Type} (step : α → α) (seed : α)
    (left right : Nat) (h : feedback step seed left = feedback step seed right)
    (tick : Nat) :
    feedback step seed (left + tick) = feedback step seed (right + tick) := by
  induction tick with
  | zero => exact h
  | succ tick ih =>
      change step (feedback step seed (left + tick)) =
        step (feedback step seed (right + tick))
      exact congrArg step ih

def DistinctBefore {program : Term} (table : ProgramTable program)
    (seed : Lean.Sixteen3) (stop : Nat) : Prop :=
  ∀ i j : Fin stop, feedback table.interpret seed i.val =
    feedback table.interpret seed j.val → i = j

instance {program : Term} (table : ProgramTable program) (seed : Lean.Sixteen3)
    (stop : Nat) : Decidable (DistinctBefore table seed stop) :=
  inferInstanceAs (Decidable (∀ i j : Fin stop,
    feedback table.interpret seed i.val = feedback table.interpret seed j.val → i = j))

/-- A first repeated state, with the complete program table and closure proof. -/
structure Cycle (program : Term) (seed : Lean.Sixteen3) where
  table : ProgramTable program
  entry : Nat
  period : Nat
  positive : 0 < period
  bounded : entry + period ≤ 16
  closes : feedback table.interpret seed (entry + period) =
    feedback table.interpret seed entry
  distinct : DistinctBefore table seed (entry + period)

def certifyCycle {program : Term} (table : ProgramTable program) (seed : Lean.Sixteen3)
    (entry period : Nat) : Option (Cycle program seed) :=
  if hpositive : 0 < period then
    if hbounded : entry + period ≤ 16 then
      if hcloses : feedback table.interpret seed (entry + period) =
          feedback table.interpret seed entry then
        if hdistinct : DistinctBefore table seed (entry + period) then
          some ⟨table, entry, period, hpositive, hbounded, hcloses, hdistinct⟩
        else none
      else none
    else none
  else none

/-- Search the seventeen observations from tick zero through tick sixteen. -/
def findCycle {program : Term} (table : ProgramTable program) (seed : Lean.Sixteen3) :
    Option (Cycle program seed) := Id.run do
  for stop in [1:17] do
    for entry in [0:stop] do
      if feedback table.interpret seed stop == feedback table.interpret seed entry then
        if let some cycle := certifyCycle table seed entry (stop - entry) then
          return some cycle
  return none

def analyzeOrbit (fuel : Nat) (program : Term) (seed : Lean.Sixteen3) :
    Option (Cycle program seed) := do
  let table ← tabulate fuel program
  findCycle table seed

def Cycle.orbit {program : Term} {seed : Lean.Sixteen3} (cycle : Cycle program seed) :
    Signal Lean.Sixteen3 := feedback cycle.table.interpret seed

theorem Cycle.program_tick {program : Term} {seed : Lean.Sixteen3}
    (cycle : Cycle program seed) (tick : Nat) :
    Steps (program @@ .datum (cycle.orbit tick)) (.datum (cycle.orbit (tick + 1))) :=
  cycle.table.reduces_at (cycle.orbit tick)

theorem Cycle.periodic {program : Term} {seed : Lean.Sixteen3}
    (cycle : Cycle program seed) (tick : Nat) :
    cycle.orbit (cycle.entry + tick + cycle.period) = cycle.orbit (cycle.entry + tick) := by
  calc
    cycle.orbit (cycle.entry + tick + cycle.period) =
        cycle.orbit (cycle.entry + cycle.period + tick) := by rw [Nat.add_right_comm]
    _ = cycle.orbit (cycle.entry + tick) :=
      feedback_equal_future cycle.table.interpret seed _ _ cycle.closes tick

/-- Reduce a future offset to one primitive lap of the certified cycle. -/
theorem Cycle.orbit_mod {program : Term} {seed : Lean.Sixteen3}
    (cycle : Cycle program seed) (offset : Nat) :
    cycle.orbit (cycle.entry + offset) =
      cycle.orbit (cycle.entry + offset % cycle.period) := by
  induction offset using Nat.strongRecOn with
  | ind offset ih =>
      if h : offset < cycle.period then
        rw [Nat.mod_eq_of_lt h]
      else
        have hle : cycle.period ≤ offset := Nat.le_of_not_gt h
        calc
          cycle.orbit (cycle.entry + offset) =
              cycle.orbit (cycle.entry + (offset - cycle.period) + cycle.period) := by
                rw [Nat.add_assoc, Nat.sub_add_cancel hle]
          _ = cycle.orbit (cycle.entry + (offset - cycle.period)) :=
            cycle.periodic (offset - cycle.period)
          _ = cycle.orbit (cycle.entry + (offset - cycle.period) % cycle.period) :=
            ih _ (Nat.sub_lt_of_pos_le cycle.positive hle)
          _ = cycle.orbit (cycle.entry + offset % cycle.period) := by
            rw [Nat.mod_eq_sub_mod hle]

/-- Preserve transient ticks and reduce later ticks to the stored cycle phase. -/
def Cycle.phaseTick {program : Term} {seed : Lean.Sixteen3}
    (cycle : Cycle program seed) (tick : Nat) : Nat :=
  if tick < cycle.entry then tick
  else cycle.entry + (tick - cycle.entry) % cycle.period

theorem Cycle.phaseTick_lt {program : Term} {seed : Lean.Sixteen3}
    (cycle : Cycle program seed) (tick : Nat) : cycle.phaseTick tick < 16 := by
  unfold Cycle.phaseTick
  split
  · rename_i h
    exact Nat.lt_of_lt_of_le (Nat.lt_trans h (Nat.lt_add_of_pos_right cycle.positive))
      cycle.bounded
  · exact Nat.lt_of_lt_of_le
      (Nat.add_lt_add_left (Nat.mod_lt _ cycle.positive) cycle.entry) cycle.bounded

theorem Cycle.orbit_phaseTick {program : Term} {seed : Lean.Sixteen3}
    (cycle : Cycle program seed) (tick : Nat) :
    cycle.orbit (cycle.phaseTick tick) = cycle.orbit tick := by
  unfold Cycle.phaseTick
  split
  · rfl
  · rename_i h
    have hle : cycle.entry ≤ tick := Nat.le_of_not_gt h
    calc
      cycle.orbit (cycle.entry + (tick - cycle.entry) % cycle.period) =
          cycle.orbit (cycle.entry + (tick - cycle.entry)) :=
        (cycle.orbit_mod (tick - cycle.entry)).symm
      _ = cycle.orbit tick := congrArg cycle.orbit (Nat.add_sub_of_le hle)

/-- A finite readout of a certified feedback signal. The constructor caches the
    native states in an array; arbitrary ticks use a phase and one array lookup. -/
structure Clock (program : Term) (seed : Lean.Sixteen3) where
  cycle : Cycle program seed
  value : Fin 16 → Lean.Sixteen3
  agrees : ∀ i, value i = cycle.orbit i.val

def Cycle.clock {program : Term} {seed : Lean.Sixteen3}
    (cycle : Cycle program seed) : Clock program seed :=
  let states := Array.ofFn (fun i : Fin 16 => cycle.orbit i.val)
  { cycle := cycle
    value := fun i => states[i.val]'(by rw [Array.size_ofFn]; exact i.isLt)
    agrees := fun i => Array.getElem_ofFn _ }

def Clock.read {program : Term} {seed : Lean.Sixteen3}
    (clock : Clock program seed) (tick : Nat) : Lean.Sixteen3 :=
  clock.value ⟨clock.cycle.phaseTick tick, clock.cycle.phaseTick_lt tick⟩

theorem Clock.read_eq_orbit {program : Term} {seed : Lean.Sixteen3}
    (clock : Clock program seed) (tick : Nat) :
    clock.read tick = clock.cycle.orbit tick :=
  (clock.agrees _).trans (clock.cycle.orbit_phaseTick tick)

theorem Clock.program_tick {program : Term} {seed : Lean.Sixteen3}
    (clock : Clock program seed) (tick : Nat) :
    Steps (program @@ .datum (clock.read tick)) (.datum (clock.read (tick + 1))) := by
  rw [clock.read_eq_orbit, clock.read_eq_orbit]
  exact clock.cycle.program_tick tick

theorem Clock.restart {program : Term} {seed : Lean.Sixteen3}
    (clock : Clock program seed) (start tick : Nat) :
    feedback clock.cycle.table.interpret (clock.read start) tick =
      clock.read (start + tick) := by
  rw [clock.read_eq_orbit, clock.read_eq_orbit]
  exact feedback_restart clock.cycle.table.interpret seed start tick

theorem Cycle.no_earlier_return {program : Term} {seed : Lean.Sixteen3}
    (cycle : Cycle program seed) (phase : Fin cycle.period) (hphase : 0 < phase.val) :
    cycle.orbit (cycle.entry + phase.val) ≠ cycle.orbit cycle.entry := by
  intro heq
  have hindices := cycle.distinct
    ⟨cycle.entry + phase.val, Nat.add_lt_add_left phase.isLt cycle.entry⟩
    ⟨cycle.entry, Nat.lt_add_of_pos_right cycle.positive⟩ heq
  exact (Nat.ne_of_gt (Nat.lt_add_of_pos_right hphase)) (congrArg Fin.val hindices)

theorem Cycle.fixed_of_period_one {program : Term} {seed : Lean.Sixteen3}
    (cycle : Cycle program seed) (hperiod : cycle.period = 1) :
    cycle.table.interpret (cycle.orbit cycle.entry) = cycle.orbit cycle.entry := by
  change cycle.orbit (cycle.entry + 1) = cycle.orbit cycle.entry
  rw [← hperiod]
  exact cycle.closes

theorem Cycle.moves_of_period_gt_one {program : Term} {seed : Lean.Sixteen3}
    (cycle : Cycle program seed) (hperiod : 1 < cycle.period) :
    cycle.table.interpret (cycle.orbit cycle.entry) ≠ cycle.orbit cycle.entry :=
  cycle.no_earlier_return ⟨1, hperiod⟩ Nat.zero_lt_one

def Cycle.transient {program : Term} {seed : Lean.Sixteen3}
    (cycle : Cycle program seed) : List Lean.Sixteen3 :=
  (List.range cycle.entry).map cycle.orbit

def Cycle.loop {program : Term} {seed : Lean.Sixteen3}
    (cycle : Cycle program seed) : List Lean.Sixteen3 :=
  (List.range cycle.period).map fun phase => cycle.orbit (cycle.entry + phase)

def Cycle.isStationary {program : Term} {seed : Lean.Sixteen3}
    (cycle : Cycle program seed) : Bool := cycle.period == 1

end Imscribing.Paraconsistent.CombinatoryDynamics
