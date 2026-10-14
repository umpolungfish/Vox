import Imscribing.Paraconsistent.Belnap
import Init.Internal.Order.Basic
import Init.Paraconsistent

enable_trilattice

/-!
# Tick-indexed semantics for paraconsistent observations

This module models recursive observations as signals indexed by natural ticks
and exposes Lean's chain-complete fixed-point construction as a Y combinator
for monotone functionals over partial values. Undefined computation is `none`;
the equation theorem is part of Lean's logic. The FDE layer below carries
dialetheic fixed points through the kernel's SIXTEEN_3 powerset representation.
Negation closure retains both poles, computes every seeded negation fixed point,
and connects periodic observations with feedback that settles after two ticks.
-/

namespace Imscribing.Paraconsistent.Temporal.Semantics

open Imscribing.Paraconsistent

universe u v w

/-- A discrete-time stream of four-valued observations. -/
abbrev Signal (α : Type u) := Nat → α

/-- Read the next tick. -/
def next {α : Type u} (s : Signal α) : Signal α := fun n => s (n + 1)

/-- Apply a truth-value operation at every tick. -/
def map {α : Type u} {β : Type v} (f : α → β) (s : Signal α) : Signal β :=
  fun n => f (s n)

/-- A one-tick delay with an explicit initial observation. -/
def delay {α : Type u} (initial : α) (s : Signal α) : Signal α
  | 0 => initial
  | n + 1 => s n

/-- Compose signal transformers. -/
def compose {α : Type u} {β : Type v} {γ : Type w}
    (f : Signal β → Signal γ) (g : Signal α → Signal β) :
    Signal α → Signal γ := fun s => f (g s)

/-- Guarded feedback: the next output is computed from the previous tick. -/
def feedback {α : Type u} (step : α → α) (initial : α) : Signal α
  | 0 => initial
  | n + 1 => step (feedback step initial n)

/-- The guarded fixed-point equation: expose the seed now and feed the
    transformed trace back only after one tick of delay. -/
theorem feedback_unfold {α : Type u} (step : α → α) (initial : α) :
    feedback step initial = delay initial (map step (feedback step initial)) := by
  funext n
  cases n <;> rfl

theorem next_delay {α : Type u} (initial : α) (s : Signal α) :
    next (delay initial s) = s := by
  funext n
  rfl

theorem map_compose (f g : Belnap → Belnap) (s : Signal Belnap) :
    map f (map g s) = map (f ∘ g) s := by
  rfl

theorem compose_assoc {α : Type u} {β : Type v} {γ : Type w} {δ : Type _}
    (f : Signal γ → Signal δ) (g : Signal β → Signal γ) (h : Signal α → Signal β) :
    compose (compose f g) h = compose f (compose g h) := rfl

namespace Combinators

/-- Scott's least-fixed-point combinator for partial functions. A functional
    needs only to preserve the approximation order; no structural recursion
    measure is supplied. The chain-complete function space includes `none` as
    the undefined approximation. -/
noncomputable def Y {α : Type u} {β : Type v}
    (F : (α → Option β) → α → Option β)
    (hF : Lean.Order.monotone F) : α → Option β :=
  Lean.Order.fix F hF

theorem Y_unfold {α : Type u} {β : Type v}
    (F : (α → Option β) → α → Option β)
    (hF : Lean.Order.monotone F) :
    Y F hF = F (Y F hF) :=
  Lean.Order.fix_eq hF

theorem Y_unfold_apply {α : Type u} {β : Type v}
    (F : (α → Option β) → α → Option β)
    (hF : Lean.Order.monotone F) (x : α) :
    Y F hF x = F (Y F hF) x :=
  congrFun (Y_unfold F hF) x

/-- The identity combinator. -/
def I {α : Type u} (x : α) : α := x

/-- The constant-function combinator. -/
def K {α : Type u} {β : Type v} (x : α) (_ : β) : α := x

/-- The substitution combinator. -/
def S {α : Type u} {β : Type v} {γ : Type w}
    (f : α → β → γ) (g : α → β) (x : α) : γ := f x (g x)

theorem I_apply {α : Type u} (x : α) : I x = x := rfl

theorem K_apply {α : Type u} {β : Type v} (x : α) (y : β) : K x y = x := rfl

theorem S_apply {α : Type u} {β : Type v} {γ : Type w}
    (f : α → β → γ) (g : α → β) (x : α) : S f g x = f x (g x) := rfl

theorem S_K_K_identity {α : Type u} (x : α) :
    S (K (α := α) (β := α → α))
      (K (α := α → α) (β := α) I) x = x := rfl

/-- A Y-style fixed-point unfolding indexed by ticks. Each stage applies the
    functional once to the preceding approximation. -/
def yUnfolding {α : Type u} (f : α → α) (seed : α) : Signal α :=
  feedback f seed

theorem yUnfolding_step {α : Type u} (f : α → α) (seed : α) (n : Nat) :
    yUnfolding f seed (n + 1) = f (yUnfolding f seed n) := rfl

/-- If adjacent approximants agree, their shared value is a fixed point. -/
theorem yUnfolding_stable_is_fixed {α : Type u} (f : α → α) (seed : α) (n : Nat)
    (h : yUnfolding f seed (n + 1) = yUnfolding f seed n) :
    f (yUnfolding f seed n) = yUnfolding f seed n := by
  calc
    f (yUnfolding f seed n) = yUnfolding f seed (n + 1) :=
      (yUnfolding_step f seed n).symm
    _ = yUnfolding f seed n := h

end Combinators

namespace FDE

/-- The kernel's Belnap FOUR values, used as the atoms of SIXTEEN_3. -/
abbrev Value := _root_.Paraconsistent.Belnap

def implication (p q : Value) : Value := _root_.Paraconsistent.Belnap.bimply p q

def designated (p : Value) : Bool :=
  match p with
  | .T | .B => true
  | .N | .F => false

theorem B_implies_B : implication .B .B = .B := by decide

theorem B_modus_ponens_B :
    _root_.Paraconsistent.Belnap.band .B (implication .B .B) = .B := by decide

/-- Under the fork's material implication `¬p ∨ q`, designated premises at
    `p = B`, `q = F` do not designate the conclusion. -/
theorem material_implication_MP_counterexample :
    designated .B = true ∧
    designated (implication .B .F) = true ∧
    designated .F = false := by decide

theorem contradiction_is_contained :
    _root_.Paraconsistent.Belnap.band .B (_root_.Paraconsistent.Belnap.bnot .B) = .B ∧
    _root_.Paraconsistent.Belnap.B ≠ _root_.Paraconsistent.Belnap.F := by decide

private def atoms : List Value := [.N, .T, .F, .B]

def contains (x : Lean.Sixteen3) : Value → Bool
  | .N => x.hasN
  | .T => x.hasT
  | .F => x.hasF
  | .B => x.hasB

/-- Lift a FOUR operation to SIXTEEN_3 by taking the direct image of the
    cartesian product of the two evidence sets. -/
def liftBinary (op : Value → Value → Value)
    (x y : Lean.Sixteen3) : Lean.Sixteen3 := Id.run do
  let hasResult (r : Value) := atoms.any fun a =>
    contains x a && atoms.any (fun b => contains y b && op a b == r)
  return ⟨hasResult .N, hasResult .T, hasResult .F, hasResult .B⟩

def liftNegation (x : Lean.Sixteen3) : Lean.Sixteen3 :=
  ⟨x.hasN, x.hasF, x.hasT, x.hasB⟩

/-- Lifted negation is an involution on the powerset carrier. -/
theorem liftNegation_involutive (x : Lean.Sixteen3) :
    liftNegation (liftNegation x) = x := by
  cases x
  rfl

/-- A powerset state is fixed by lifted negation exactly when it contains T and
    F together or omits both. The N and B memberships remain independent. -/
theorem liftNegation_fixed_iff (x : Lean.Sixteen3) :
    liftNegation x = x ↔ x.hasT = x.hasF := by
  cases x with
  | mk hasN hasT hasF hasB =>
    cases hasN <;> cases hasT <;> cases hasF <;> cases hasB <;> decide

/-- Every choice of N membership, paired T/F membership, and B membership gives
    a fixed point. These three independent bits describe the whole fixed set. -/
def negationFixedState (hasN hasPair hasB : Bool) : Lean.Sixteen3 :=
  ⟨hasN, hasPair, hasPair, hasB⟩

theorem negationFixedState_fixed (hasN hasPair hasB : Bool) :
    liftNegation (negationFixedState hasN hasPair hasB) =
      negationFixedState hasN hasPair hasB := by
  cases hasN <;> cases hasPair <;> cases hasB <;> rfl

theorem every_negation_fixed_state_has_form (x : Lean.Sixteen3)
    (h : liftNegation x = x) :
    x = negationFixedState x.hasN x.hasT x.hasB := by
  have hpair := (liftNegation_fixed_iff x).mp h
  cases x with
  | mk hasN hasT hasF hasB => simp_all [negationFixedState]

/-- The fixed states of SIXTEEN_3 negation are exactly three independent bits:
    N membership, the paired T/F membership, and B membership. -/
def negationFixedEquiv : (Bool × Bool × Bool) ≃
    {x : Lean.Sixteen3 // liftNegation x = x} where
  toFun bits :=
    ⟨negationFixedState bits.1 bits.2.1 bits.2.2,
      negationFixedState_fixed bits.1 bits.2.1 bits.2.2⟩
  invFun x := (x.1.hasN, x.1.hasT, x.1.hasB)
  left_inv bits := by
    cases bits with
    | mk hasN rest => cases rest with | mk hasPair hasB => rfl
  right_inv x := by
    rcases x with ⟨⟨hasN, hasT, hasF, hasB⟩, hfixed⟩
    have hpair : hasT = hasF := by
      simpa using (liftNegation_fixed_iff _).mp hfixed
    simp [negationFixedState, hpair]

/-- Hold each observation together with its negation by information join. -/
def negationClosure (x : Lean.Sixteen3) : Lean.Sixteen3 :=
  Lean.Sixteen3.join_i x (liftNegation x)

theorem negationClosure_fixed (x : Lean.Sixteen3) :
    liftNegation (negationClosure x) = negationClosure x := by
  rcases x with ⟨n, t, f, b⟩
  cases n <;> cases t <;> cases f <;> cases b <;> rfl

theorem negationClosure_idempotent (x : Lean.Sixteen3) :
    negationClosure (negationClosure x) = negationClosure x := by
  rcases x with ⟨n, t, f, b⟩
  cases n <;> cases t <;> cases f <;> cases b <;> rfl

theorem negationClosure_negation (x : Lean.Sixteen3) :
    negationClosure (liftNegation x) = negationClosure x := by
  rcases x with ⟨n, t, f, b⟩
  cases n <;> cases t <;> cases f <;> cases b <;> rfl

theorem negationClosure_extensive (x : Lean.Sixteen3) :
    Lean.Sixteen3.le_i x (negationClosure x) = true := by
  rcases x with ⟨n, t, f, b⟩
  cases n <;> cases t <;> cases f <;> cases b <;> rfl

theorem negationClosure_monotone (x y : Lean.Sixteen3) :
    Lean.Sixteen3.le_i x y = true →
      Lean.Sixteen3.le_i (negationClosure x) (negationClosure y) = true := by
  rcases x with ⟨xn, xt, xf, xb⟩
  rcases y with ⟨yn, yt, yf, yb⟩
  cases xn <;> cases xt <;> cases xf <;> cases xb <;>
    cases yn <;> cases yt <;> cases yf <;> cases yb <;> decide

/-- Closure is the least negation-fixed state containing its input. -/
theorem negationClosure_le_iff (x y : Lean.Sixteen3)
    (hy : liftNegation y = y) :
    Lean.Sixteen3.le_i (negationClosure x) y = true ↔
      Lean.Sixteen3.le_i x y = true := by
  revert hy
  rcases x with ⟨xn, xt, xf, xb⟩
  rcases y with ⟨yn, yt, yf, yb⟩
  cases xn <;> cases xt <;> cases xf <;> cases xb <;>
    cases yn <;> cases yt <;> cases yf <;> cases yb <;> decide

/-- Feed negation back while retaining a fixed seed at each unfolding. -/
def negationSeededStep (seed x : Lean.Sixteen3) : Lean.Sixteen3 :=
  Lean.Sixteen3.join_i (liftNegation x) seed

theorem negationSeededStep_closure_fixed (seed : Lean.Sixteen3) :
    negationSeededStep seed (negationClosure seed) = negationClosure seed := by
  rcases seed with ⟨n, t, f, b⟩
  cases n <;> cases t <;> cases f <;> cases b <;> rfl

theorem negationSeededStep_fixed_iff (seed x : Lean.Sixteen3) :
    negationSeededStep seed x = x ↔
      liftNegation x = x ∧ Lean.Sixteen3.le_i seed x = true := by
  rcases seed with ⟨sn, st, sf, sb⟩
  rcases x with ⟨xn, xt, xf, xb⟩
  cases sn <;> cases st <;> cases sf <;> cases sb <;>
    cases xn <;> cases xt <;> cases xf <;> cases xb <;> decide

/-- A closed formula for the least information fixed point of every seeded
    negation map, available directly as a computable function. -/
theorem negationSeededStep_least (seed y : Lean.Sixteen3)
    (hy : negationSeededStep seed y = y) :
    Lean.Sixteen3.le_i (negationClosure seed) y = true := by
  have hfixed := (negationSeededStep_fixed_iff seed y).mp hy
  exact (negationClosure_le_iff seed y hfixed.1).mpr hfixed.2

def singleton (x : Value) : Lean.Sixteen3 :=
  match x with
  | .N => ⟨true, false, false, false⟩
  | .T => ⟨false, true, false, false⟩
  | .F => ⟨false, false, true, false⟩
  | .B => ⟨false, false, false, true⟩

def designated16 (x : Lean.Sixteen3) : Bool := x.hasT || x.hasB

theorem liftNegation_singleton (p : Value) :
    liftNegation (singleton p) = singleton (_root_.Paraconsistent.Belnap.bnot p) := by
  cases p <;> decide

theorem liftBimply_B_B :
    liftBinary _root_.Paraconsistent.Belnap.bimply (singleton .B) (singleton .B) =
      singleton .B := by decide

theorem liftBimply_B_F :
    liftBinary _root_.Paraconsistent.Belnap.bimply (singleton .B) (singleton .F) =
      singleton .B := by decide

theorem B_modus_ponens_in_SIXTEEN_3 :
    liftBinary _root_.Paraconsistent.Belnap.band (singleton .B)
      (liftBinary _root_.Paraconsistent.Belnap.bimply (singleton .B) (singleton .B)) =
        singleton .B := by decide

theorem material_implication_MP_counterexample_in_SIXTEEN_3 :
    designated16 (singleton .B) = true ∧
    designated16 (liftBinary _root_.Paraconsistent.Belnap.bimply
      (singleton .B) (singleton .F)) = true ∧
    designated16 (singleton .F) = false := by
  decide

theorem SIXTEEN_3_contradiction_contained :
    liftBinary _root_.Paraconsistent.Belnap.band (singleton .B)
      (liftNegation (singleton .B)) = singleton .B ∧
    singleton .B ≠ singleton .F := by
  decide

end FDE

namespace NegationSignals

/-- Read each successive negation as its own tick. -/
def orbit (seed : Lean.Sixteen3) : Signal Lean.Sixteen3 :=
  feedback FDE.liftNegation seed

theorem orbit_period_two (seed : Lean.Sixteen3) (n : Nat) :
    orbit seed (n + 2) = orbit seed n := by
  change FDE.liftNegation (FDE.liftNegation (orbit seed n)) = orbit seed n
  exact FDE.liftNegation_involutive _

theorem orbit_closure (seed : Lean.Sixteen3) (n : Nat) :
    FDE.negationClosure (orbit seed n) = FDE.negationClosure seed := by
  induction n with
  | zero => rfl
  | succ n ih =>
      change FDE.negationClosure (FDE.liftNegation (orbit seed n)) =
        FDE.negationClosure seed
      rw [FDE.negationClosure_negation, ih]

/-- Holding any two adjacent ticks recovers the same negation closure. -/
theorem adjacent_ticks_hold_closure (seed : Lean.Sixteen3) (n : Nat) :
    Lean.Sixteen3.join_i (orbit seed n) (orbit seed (n + 1)) =
      FDE.negationClosure seed := by
  exact orbit_closure seed n

/-- Unfold the retained-seed feedback map from information bottom. -/
def seededFeedback (seed : Lean.Sixteen3) : Signal Lean.Sixteen3 :=
  feedback (FDE.negationSeededStep seed) Lean.Sixteen3.none

theorem seededFeedback_one (seed : Lean.Sixteen3) :
    seededFeedback seed 1 = seed := by
  rcases seed with ⟨n, t, f, b⟩
  cases n <;> cases t <;> cases f <;> cases b <;> rfl

theorem seededFeedback_two (seed : Lean.Sixteen3) :
    seededFeedback seed 2 = FDE.negationClosure seed := by
  rcases seed with ⟨n, t, f, b⟩
  cases n <;> cases t <;> cases f <;> cases b <;> rfl

/-- Every retained-seed feedback trace settles after at most two unfoldings. -/
theorem seededFeedback_settled (seed : Lean.Sixteen3) (n : Nat) :
    seededFeedback seed (n + 2) = FDE.negationClosure seed := by
  induction n with
  | zero => exact seededFeedback_two seed
  | succ n ih =>
      change FDE.negationSeededStep seed (seededFeedback seed (n + 2)) =
        FDE.negationClosure seed
      rw [ih, FDE.negationSeededStep_closure_fixed]

end NegationSignals

namespace KernelReentry

open Lean

/-- Native re-entry computes the least information fixed point of negation on
    the SIXTEEN_3 powerset. Its value is the empty subset. -/
reentry negationReentry (x : Sixteen3) : Sixteen3 :=
  FDE.liftNegation x

/-- Kernel-native re-entry feeds SIXTEEN_3 negation back together with the
    B evidence. The least information fixed point is the singleton B state. -/
reentry dialetheicNegationReentry (x : Sixteen3) : Sixteen3 :=
  Sixteen3.join_i (FDE.liftNegation x) (FDE.singleton .B)

/-- A T seed re-enters as the subset containing both T and F. -/
reentry truthNegationReentry (x : Sixteen3) : Sixteen3 :=
  FDE.negationSeededStep (FDE.singleton .T) x

/-- The unknown FOUR member stays present as its own negation-fixed singleton. -/
reentry unknownNegationReentry (x : Sixteen3) : Sixteen3 :=
  FDE.negationSeededStep (FDE.singleton .N) x

theorem negationReentry_value :
    negationReentry = Sixteen3.none := rfl

theorem negationReentry_fixed :
    FDE.liftNegation negationReentry = negationReentry := rfl

theorem dialetheicNegationReentry_value :
    dialetheicNegationReentry = FDE.singleton .B := rfl

theorem dialetheicNegationReentry_fixed :
    Sixteen3.join_i (FDE.liftNegation dialetheicNegationReentry)
      (FDE.singleton .B) = dialetheicNegationReentry := rfl

theorem truthNegationReentry_value :
    truthNegationReentry = Sixteen3.ofBelnap false true true false := rfl

theorem truthNegationReentry_is_closure :
    truthNegationReentry = FDE.negationClosure (FDE.singleton .T) := rfl

theorem unknownNegationReentry_is_closure :
    unknownNegationReentry = FDE.negationClosure (FDE.singleton .N) := rfl

theorem dialetheicNegationReentry_is_closure :
    dialetheicNegationReentry = FDE.negationClosure (FDE.singleton .B) := rfl

end KernelReentry

namespace DialetheicY

open Lean.Order

/-- A recursive stream functional that seeds the first tick with the FDE glut
    and obtains every later tick from its predecessor. -/
def BStreamFunctional (s : Nat → Option FDE.Value) : Nat → Option FDE.Value
  | 0 => some .B
  | n + 1 => s n

theorem BStreamFunctional_monotone : monotone BStreamFunctional := by
  intro s t h n
  cases n with
  | zero => exact PartialOrder.rel_refl
  | succ n => exact h n

/-- The Y-combinator fixed point of the B-seeded recursive stream. -/
noncomputable def BStream : Nat → Option FDE.Value :=
  Combinators.Y BStreamFunctional BStreamFunctional_monotone

theorem BStream_unfold :
    BStream = BStreamFunctional BStream :=
  Combinators.Y_unfold BStreamFunctional BStreamFunctional_monotone

theorem BStream_zero : BStream 0 = some .B := by
  calc
    BStream 0 = BStreamFunctional BStream 0 := congrFun BStream_unfold 0
    _ = some .B := rfl

theorem BStream_succ (n : Nat) : BStream (n + 1) = BStream n := by
  calc
    BStream (n + 1) = BStreamFunctional BStream (n + 1) :=
      congrFun BStream_unfold (n + 1)
    _ = BStream n := rfl

theorem BStream_all_B (n : Nat) : BStream n = some .B := by
  induction n with
  | zero => exact BStream_zero
  | succ n ih => rw [BStream_succ, ih]

end DialetheicY

namespace UntypedLambda

/-- Terms of the untyped lambda calculus, represented with named variables. -/
inductive Term where
  | var : Nat → Term
  | app : Term → Term → Term
  | lam : Nat → Term → Term
  deriving DecidableEq, Repr

open Term

/-- Free occurrence of a variable. A binder removes its name from the body. -/
def HasFree (name : Nat) : Term → Prop
  | .var other => name = other
  | .app f x => HasFree name f ∨ HasFree name x
  | .lam binder body => binder ≠ name ∧ HasFree name body

def Closed (t : Term) : Prop := ∀ name, ¬ HasFree name t

/-- Substitution with shadowing. The Y laws below apply to closed arguments,
    for which substitution cannot capture a free variable. -/
def subst (target : Nat) (replacement : Term) : Term → Term
  | .var name => if name = target then replacement else .var name
  | .app f x => .app (subst target replacement f) (subst target replacement x)
  | .lam binder body =>
      if binder = target then .lam binder body
      else .lam binder (subst target replacement body)

theorem subst_irrelevant (target : Nat) (replacement : Term) (t : Term)
    (h : ¬ HasFree target t) : subst target replacement t = t := by
  induction t with
  | var name =>
      by_cases heq : name = target
      · simp [HasFree, heq] at h
      · simp [subst, heq]
  | app f x ihf ihx =>
      have hf : ¬ HasFree target f := by
        intro hfree
        exact h (Or.inl hfree)
      have hx : ¬ HasFree target x := by
        intro hfree
        exact h (Or.inr hfree)
      simp [subst, ihf hf, ihx hx]
  | lam binder body ih =>
      by_cases heq : binder = target
      · simp [subst, heq]
      · have hbody : ¬ HasFree target body := by
          intro hfree
          exact h ⟨heq, hfree⟩
        simp [subst, heq, ih hbody]

/-- Contract a beta-redex at the root of a term. -/
def beta : Term → Option Term
  | .app (.lam binder body) argument => some (subst binder argument body)
  | _ => none

/-- The self-application seed `λx. f (x x)`, with `f` at name zero and
    `x` at name one. -/
def seed : Term :=
  .lam 1 (.app (.var 0) (.app (.var 1) (.var 1)))

/-- The untyped Curry fixed-point combinator `λf. D D`. -/
def Y : Term := .lam 0 (.app seed seed)

def recursiveHalf (F : Term) : Term :=
  .lam 1 (.app F (.app (.var 1) (.var 1)))

def unfoldState (F : Term) : Term :=
  .app (recursiveHalf F) (recursiveHalf F)

theorem Y_first_beta (F : Term) :
    beta (.app Y F) = some (unfoldState F) := by
  simp [beta, Y, seed, unfoldState, recursiveHalf, subst]

theorem Y_second_beta (F : Term) (hF : Closed F) :
    beta (unfoldState F) = some (.app F (unfoldState F)) := by
  have hstable : subst 1 (recursiveHalf F) F = F :=
    subst_irrelevant 1 (recursiveHalf F) F (hF 1)
  have hcontract :
      subst 1 (recursiveHalf F)
        (.app F (.app (.var 1) (.var 1))) =
        .app (subst 1 (recursiveHalf F) F)
          (.app (recursiveHalf F) (recursiveHalf F)) := rfl
  change some (subst 1 (recursiveHalf F)
      (.app F (.app (.var 1) (.var 1)))) =
    some (.app F (unfoldState F))
  rw [hcontract, hstable]
  rfl

/-- Every closed untyped functional unfolds through Y's self-application to
    the equation `Y F ↦ F (Y F)` in two root beta steps. -/
theorem Y_unfolds (F : Term) (hF : Closed F) :
    ∃ middle, beta (.app Y F) = some middle ∧
      beta middle = some (.app F middle) := by
  exact ⟨unfoldState F, Y_first_beta F, Y_second_beta F hF⟩

/-- The closed identity functional `λx. x`. Its Y unfolding has a genuine
    two-state root-reduction cycle: the recursive application unfolds to the
    functional applied to itself, then the identity returns that unfolding. -/
def identityFunctional : Term := .lam 2 (.var 2)

theorem identityFunctional_closed : Closed identityFunctional := by
  intro name hfree
  change 2 ≠ name ∧ name = 2 at hfree
  exact hfree.1 hfree.2.symm

theorem identity_beta (t : Term) :
    beta (.app identityFunctional t) = some t := by
  simp [beta, identityFunctional, subst]

def identityYState : Term := unfoldState identityFunctional

def identityYNext : Term := .app identityFunctional identityYState

theorem identityYState_step : beta identityYState = some identityYNext :=
  Y_second_beta identityFunctional identityFunctional_closed

theorem identityYNext_step : beta identityYNext = some identityYState :=
  identity_beta identityYState

/-- A tick-indexed signal alternating between the two root-redex states of
    the untyped Y unfolding. This records divergence as a periodic signal. -/
def identityYLoop : Signal Term
  | 0 => identityYState
  | 1 => identityYNext
  | n + 2 => identityYLoop n

theorem identityYLoop_period_two (n : Nat) :
    identityYLoop (n + 2) = identityYLoop n := rfl

theorem identityYLoop_step (n : Nat) :
    beta (identityYLoop n) = some (identityYLoop (n + 1)) := by
  cases n with
  | zero => exact identityYState_step
  | succ n =>
      cases n with
      | zero => exact identityYNext_step
      | succ n =>
          simpa [identityYLoop] using identityYLoop_step n

def StableTerm (s : Signal Term) : Prop := ∃ t, ∀ n, s n = t

theorem identityYLoop_not_stable : ¬ StableTerm identityYLoop := by
  rintro ⟨t, ht⟩
  have h0 : t = identityYState := (ht 0).symm
  have h1 : t = identityYNext := (ht 1).symm
  have heq : identityYState = identityYNext := h0.symm.trans h1
  have hne : identityYState ≠ identityYNext := by
    decide
  exact hne heq

end UntypedLambda

/-- An oscillator started at true and negated once on every tick. -/
def liarOscillator : Signal Belnap := feedback bnot Belnap.T

theorem liarOscillator_step (n : Nat) :
    liarOscillator (n + 1) = bnot (liarOscillator n) := rfl

theorem bnot_involutive (v : Belnap) : bnot (bnot v) = v := by
  cases v <;> rfl

theorem liarOscillator_period_two (n : Nat) :
    liarOscillator (n + 2) = liarOscillator n := by
  change bnot (bnot (liarOscillator n)) = liarOscillator n
  exact bnot_involutive _

/-- The liar trace stays on the classical true/false slice while it oscillates. -/
def Oscillating (s : Signal Belnap) : Prop :=
  ∀ n, (s n = .T ∨ s n = .F) ∧ s (n + 1) = bnot (s n)

theorem liarOscillator_values (n : Nat) :
    liarOscillator n = .T ∨ liarOscillator n = .F := by
  induction n with
  | zero => exact Or.inl rfl
  | succ n ih =>
      change bnot (liarOscillator n) = .T ∨ bnot (liarOscillator n) = .F
      rcases ih with h | h
      · rw [h]
        exact Or.inr rfl
      · rw [h]
        exact Or.inl rfl

theorem liarOscillator_oscillates : Oscillating liarOscillator := by
  intro n
  exact ⟨liarOscillator_values n, liarOscillator_step n⟩

def Stable (s : Signal Belnap) : Prop := ∃ v, ∀ n, s n = v

theorem liarOscillator_not_stable : ¬ Stable liarOscillator := by
  rintro ⟨v, hv⟩
  have h0 : liarOscillator 0 = .T := rfl
  have h1 : liarOscillator 1 = .F := rfl
  have ht : v = .T := (hv 0).symm.trans h0
  have hf : v = .F := (hv 1).symm.trans h1
  have : Belnap.T = Belnap.F := ht.symm.trans hf
  exact (by decide : Belnap.T ≠ Belnap.F) this

/-- Stable unknown evidence is distinct from an alternating classical trace. -/
def undetermined : Signal Belnap := fun _ => .N

theorem undetermined_stable : Stable undetermined :=
  ⟨.N, fun _ => rfl⟩

theorem undetermined_negation_fixed (n : Nat) :
    bnot (undetermined n) = undetermined n := rfl

/-- Belnap's glut is a stable, self-negating observation. -/
def glut : Signal Belnap := fun _ => .B

theorem glut_stable : Stable glut := ⟨.B, fun _ => rfl⟩

theorem glut_negation_fixed (n : Nat) : bnot (glut n) = glut n := rfl

/-- A relation is a tick bisimulation when it matches the current readout and
    remains related after both signals advance by one tick. -/
def IsBisimulation {α : Type u} (R : Signal α → Signal α → Prop) : Prop :=
  ∀ ⦃s t⦄, R s t → s 0 = t 0 ∧ R (next s) (next t)

theorem bisimilar_signals_agree {α : Type u} (R : Signal α → Signal α → Prop)
    (hstep : IsBisimulation R) {s t : Signal α} (hrel : R s t) :
    ∀ n, s n = t n := by
  intro n
  induction n generalizing s t hrel with
  | zero => exact (hstep hrel).1
  | succ n ih =>
      have hnext : R (next s) (next t) := (hstep hrel).2
      exact ih hnext

end Imscribing.Paraconsistent.Temporal.Semantics
