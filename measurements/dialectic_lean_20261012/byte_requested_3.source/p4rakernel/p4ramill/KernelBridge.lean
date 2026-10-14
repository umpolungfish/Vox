import Init.Paraconsistent
import Imscribing.Paraconsistent.CombinatoryFixedPointBounds
import Imscribing.Paraconsistent.CombinatoryDynamics
import Imscribing.Paraconsistent.DivisorReentry
import Imscribing.Paraconsistent.DialecticResident

open Imscribing.Paraconsistent
open CombinatoryReentry CombinatoryFixedPoint CombinatoryFixedPointBounds
open CombinatoryDynamics TrilatticePrograms
open Imscribing.Paraconsistent.DivisorReentry
local infixl:70 " @@ " => Term.app

enable_trilattice

run_cmd do
  unless (← Lean.getEnv).isTrilattice do
    throwError "the kernel bridge requires active SIXTEEN_3"

reentry bridgeHold (state : Lean.Sixteen3) : Lean.Sixteen3 :=
  Lean.Sixteen3.join_i state ⟨false, true, true, false⟩

private def tick (s : String) : Except String Nat :=
  match s.toNat? with
  | some n => .ok n
  | none => .error "expected a natural tick"

private def mask (s : String) : Except String Lean.Sixteen3 := do
  let some n := s.toNat? | throw "expected a native SIXTEEN_3 mask (0..15)"
  if n > 15 then throw "mask exceeds 15"
  return Lean.Sixteen3.ofMask n

private def operation : String → Except String LatticeOp
  | "join_i" => .ok .joinInformation
  | "meet_i" => .ok .meetInformation
  | "join_t" => .ok .joinTruth
  | "meet_t" => .ok .meetTruth
  | "join_f" => .ok .joinFalsity
  | "meet_f" => .ok .meetFalsity
  | _ => .error "expected join_i, meet_i, join_t, meet_t, join_f, or meet_f"

private def parsePostfix (tokens : List String) : Except String Term := do
  let mut stack : List Term := []
  for token in tokens do
    if token == "@" then
      let x :: f :: rest := stack | throw "application requires two terms"
      stack := (f @@ x) :: rest
    else
      let term ← match token with
        | "I" => pure Term.i
        | "K" => pure Term.k
        | "S" => pure Term.s
        | "Y" => pure Y
        | "hold" => pure hold
        | "negation" => pure Term.negation
        | _ =>
          if let some value := token.dropPrefix? "data:" then
            Term.datum <$> mask value.toString
          else Term.lattice <$> operation token
      stack := term :: stack
  let [result] := stack | throw "postfix program must leave exactly one term"
  return result

private def program : List String → Except String Term
  | "program" :: tokens => parsePostfix tokens
  | ["identity"] => .ok .i
  | ["negation"] => .ok .negation
  | ["hold"] => .ok hold
  | ["seeded", s] => seededOperator <$> mask s
  | ["constant", s] => (fun x => .k @@ .datum x) <$> mask s
  | ["Y", s] => (fun x => Y @@ (.k @@ (.k @@ .datum x))) <$> mask s
  | [op, s] => do return withSeed (← operation op) (← mask s)
  | ["retain-after-inject", f, s] => do return retainAfterInject (← mask f) (← mask s)
  | ["inject-after-retain", f, s] => do return injectAfterRetain (← mask f) (← mask s)
  | _ => .error "expected identity | negation | hold | seeded MASK | constant MASK | Y MASK | OP MASK | retain-after-inject FRAME SEED | inject-after-retain FRAME SEED"

private def rawTable (args : List String) : IO UInt32 := do
  let out ← IO.getStdout
  let err ← IO.getStderr
  let seedText :: tickText :: values := args | err.putStrLn "table SEED TICK OUTPUT0 ... OUTPUT15"; return 2
  match (do return (← mask seedText, ← tick tickText, ← values.mapM mask)) with
  | .error message => err.putStrLn message; return 2
  | .ok (seed, time, values) =>
    if values.length != 16 then err.putStrLn "table requires exactly sixteen outputs"; return 2
    let table := values.toArray
    let step := fun x => table[(maskIndex x).val]!
    out.putStrLn "backend=Lean-kernel SIXTEEN_3=active mask-order=N,T,F,B scope=runtime-transition-table"
    out.putStrLn s!"table={values.map (fun x => (maskIndex x).val)}"
    match Lean.Sixteen3.fixedPoint table with
    | some point =>
      if step point = point ∧ (∀ i : Fin 16, step (Lean.Sixteen3.ofMask i.val) = Lean.Sixteen3.ofMask i.val →
          Lean.Sixteen3.le_i point (Lean.Sixteen3.ofMask i.val) = true) then
        out.putStrLn s!"least={(maskIndex point).val} native-fixed-point-checked=true"
        let dual := Array.ofFn (fun i : Fin 16 => informationComplement
          (step (informationComplement (Lean.Sixteen3.ofMask i.val))))
        let some upperDual := Lean.Sixteen3.fixedPoint dual |
          err.putStrLn "native dual solver rejected a monotone table"; return 2
        let upper := informationComplement upperDual
        if step upper = upper ∧ (∀ i : Fin 16, step (Lean.Sixteen3.ofMask i.val) = Lean.Sixteen3.ofMask i.val →
            Lean.Sixteen3.le_i (Lean.Sixteen3.ofMask i.val) upper = true) then
          out.putStrLn s!"greatest={(maskIndex upper).val} native-greatest-fixed-point-checked=true"
        else err.putStrLn "native greatest solver candidate failed validation"; return 2
      else err.putStrLn "native solver candidate failed validation"; return 2
    | none => out.putStrLn "fixed-point-certificate=unavailable (native solver requires information monotonicity)"
    let states := Array.ofFn (fun i : Fin 17 => Temporal.Semantics.feedback step seed i.val)
    for stop in [1:17] do
      for start in [0:stop] do
        if states[stop]! == states[start]! then
          let period := stop - start
          let phase := if time < start then time else start + (time - start) % period
          out.putStrLn s!"seed={(maskIndex seed).val} transient={start} period={period} tick={time} value={(maskIndex states[phase]!).val} orbit-closure-checked=true"
          return 0
    err.putStrLn "sixteen-state orbit did not close"
    return 2

private def verifyProduct (source left right : String) : IO UInt32 := do
  let out ← IO.getStdout
  let err ← IO.getStderr
  let some n := source.toNat? | err.putStrLn "invalid source natural"; return 2
  let some p := left.toNat? | err.putStrLn "invalid left natural"; return 2
  let some q := right.toNat? | err.putStrLn "invalid right natural"; return 2
  -- This dependent value retains the full arithmetic equality and both
  -- nontriviality proofs. No source bits are projected into the status carrier.
  let certificate : Option { pair : Nat × Nat //
      pair.1 * pair.2 = n ∧ 1 < pair.1 ∧ 1 < pair.2 } :=
    if h : p * q = n ∧ 1 < p ∧ 1 < q then some ⟨(p, q), h⟩ else none
  let evidence := Lean.Sixteen3.ofMask (if certificate.isSome then 8 else 4)
  let some bounds := solveBounds 4096 (withSeed .joinInformation evidence) |
    err.putStrLn "arithmetic evidence failed native lattice certification"; return 2
  out.putStrLn s!"source={n}\nleft={p}\nright={q}\nproduct={p*q}"
  out.putStrLn s!"product-certificate={certificate.isSome} evidence={(maskIndex bounds.lower.point).val} evidence-certified=true"
  return if certificate.isSome then 0 else 2

private def runDivisor (args : List String) : IO UInt32 := do
  let out ← IO.getStdout
  let err ← IO.getStderr
  let parse (s : String) : Except String Nat :=
    match s.toNat? with
    | some n => .ok n
    | none => .error s!"invalid natural numeral: {s}"
  let report (N k : Nat) (both : Bool) : IO UInt32 := do
    if 1 < k && k < N then
      unless N % k == 0 do
        err.putStrLn s!"Y divisor output {k} failed exact divisibility check for {N}"
        return 2
      out.putStrLn s!"factor={k} cofactor={N / k} product={k * (N / k)} evidence={repr (δ N k both)}"
    else
      out.putStrLn s!"sentinel={k}"
    return 0
  match args with
  | ["factor", n] =>
      match parse n with
      | .ok N => return ← report N (factor N) false
      | .error e => err.putStrLn e; return 2
  | ["factor", n, "both"] =>
      match parse n with
      | .ok N => return ← report N (factor N true) true
      | .error e => err.putStrLn e; return 2
  | ["factors", n] =>
      match parse n with
      | .ok N => out.putStrLn s!"factors={factors N}"; return 0
      | .error e => err.putStrLn e; return 2
  | ["factors", n, "both"] =>
      match parse n with
      | .ok N => out.putStrLn s!"factors={factors N true}"; return 0
      | .error e => err.putStrLn e; return 2
  | ["from", n, d] =>
      match parse n, parse d with
      | .ok N, .ok seed => return ← report N (factorFrom N seed) false
      | .error e, _ | _, .error e => err.putStrLn e; return 2
  | ["from", n, d, "both"] =>
      match parse n, parse d with
      | .ok N, .ok seed => return ← report N (factorFrom N seed true) true
      | .error e, _ | _, .error e => err.putStrLn e; return 2
  | ["profile", n] =>
      match parse n with
      | .error e => err.putStrLn e; return 2
      | .ok N =>
          let (steps, samples, result) := evaluateProfile (factorProgram N 2)
          out.putStrLn s!"contractions={steps} sampled-term-sizes={samples} result={repr result}"
          return 0
  | _ =>
      err.putStrLn "y_divisor factor N [both] | factors N [both] | from N D [both] | profile N"
      return 2

def main (args : List String) : IO UInt32 := do
  if args == ["dialectic-plan"] then
    let out ← IO.getStdout
    out.putStrLn s!"Y-postfix={String.intercalate " " (DialecticResident.emitPostfix CombinatoryReentry.Y)}"
    out.putStrLn s!"Y-prefix-contractions={DialecticResident.residentPrefixContractions.getD 0}"
    for (term, index) in DialecticResident.residentPrefixTrace.zipIdx do
      out.putStrLn s!"Y-prefix-{index}={String.intercalate " " term}"
    out.putStrLn s!"square-support={(List.finRange 256).map (fun r => if DialecticResident.squareSupport256 r then 1 else 0)}"
    out.putStrLn s!"negation={(List.finRange 16).map (fun r => (maskIndex (Temporal.Semantics.FDE.liftNegation (Lean.Sixteen3.ofMask r.val))).val)}"
    out.putStrLn s!"join-information={(List.finRange 16).flatMap (fun a => (List.finRange 16).map (fun b => (maskIndex (Lean.Sixteen3.join_i (Lean.Sixteen3.ofMask a.val) (Lean.Sixteen3.ofMask b.val))).val))}"
    out.putStrLn "dialectic-plan=Lean-certified"
    return 0
  if let "divisor" :: rest := args then return ← runDivisor rest
  if let ["product", n, p, q] := args then return ← verifyProduct n p q
  if let "table" :: rest := args then return ← rawTable rest
  let out ← IO.getStdout
  let err ← IO.getStderr
  let (seedText, tickText, programArgs) := match args with
    | "orbit" :: seed :: tick :: rest => (seed, tick, rest)
    | rest => ("0", "4", rest)
  match (do return (← program programArgs, ← mask seedText,
      ← tick tickText)) with
  | .error message => err.putStrLn message; return 2
  | .ok (p, seed, tick) =>
    let some table := tabulate 4096 p | err.putStrLn "program did not reduce within fuel"; return 2
    out.putStrLn "backend=Lean-kernel SIXTEEN_3=active mask-order=N,T,F,B"
    out.putStrLn s!"kernel-reentry={(maskIndex bridgeHold).val}"
    out.putStrLn s!"table={table.toArray.toList.map (fun x => (maskIndex x).val)}"
    match solveBounds 4096 p with
    | some bounds =>
      out.putStrLn s!"least={(maskIndex bounds.lower.point).val} greatest={(maskIndex bounds.upper.point).val} certified=true"
    | none => out.putStrLn "fixed-point-certificate=unavailable"
    let some cycle := findCycle table seed | err.putStrLn "cycle certification failed"; return 2
    out.putStrLn s!"seed={(maskIndex seed).val} transient={cycle.entry} period={cycle.period} tick={tick} value={(maskIndex (cycle.clock.read tick)).val} cycle-certified=true"
    return 0
