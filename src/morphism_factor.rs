//! Factoring over IMASM numeral tapes.
//!
//! A numeral is never decoded into a machine integer.  Its payload is a tape
//! of EVALT/EVALF marks, least significant cell first.  Arithmetic consumes
//! and produces those tapes through the full-adder/full-subtractor tables.

use crate::vox::{AREV, AFWD, CLINK, EVALF, EVALT, FFUSE, FSPLIT, IFIX, IMSCRIB, TANCH, VINIT};
use alloc::format;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

type Tape = Vec<char>;

const PHASE: &[char] = &[VINIT, FSPLIT, AFWD, EVALT, EVALF, FFUSE, TANCH];
const ARITHMETIC: &[char] = &[VINIT, FSPLIT, CLINK, EVALT, EVALF, FFUSE, TANCH];
const BRANCH: &[char] = &[VINIT, FSPLIT, EVALT, EVALF, FFUSE, TANCH];
const SELECT: &[char] = &[VINIT, FSPLIT, IMSCRIB, FFUSE, TANCH];
const CONTINUE: &[char] = &[VINIT, AFWD, CLINK, TANCH];
const FIX: &[char] = &[VINIT, IMSCRIB, IFIX, TANCH];

fn bit(mark: char) -> Result<bool, String> {
    match mark {
        EVALT => Ok(false),
        EVALF => Ok(true),
        _ => Err(format!("non-numeral mark {mark}")),
    }
}

fn mark(v: bool) -> char {
    if v {
        EVALF
    } else {
        EVALT
    }
}

fn trim(mut a: Tape) -> Tape {
    while a.len() > 1 && a.last() == Some(&EVALT) {
        a.pop();
    }
    if a.is_empty() {
        a.push(EVALT);
    }
    a
}

pub fn parse_numeral(word: &str) -> Result<Tape, String> {
    let c: Vec<char> = word.chars().collect();
    if c == [VINIT, IMSCRIB, IFIX, TANCH] {
        return Ok(vec![EVALT]);
    }

    // Properly-nested single-frame numeral: ⊢ ∈ [⊤/⊥ bits LSB-first] ≺ ∋ ⊡ ⊣
    // Entry ∈ and exit ∋ are the SAME frame; AREV ≺ sits INSIDE the frame so the
    // deposited marks bank and survive the reversal — the matched puncture.
    if c.len() >= 6
        && c.first() == Some(&VINIT)
        && c[1] == FSPLIT
        && c[c.len() - 1] == TANCH
        && c[c.len() - 2] == IFIX
        && c[c.len() - 3] == FFUSE
        && c[c.len() - 4] == AREV
    {
        let mut out = Vec::new();
        for &m in &c[2..c.len() - 4] {
            bit(m)?;
            out.push(m);
        }
        return Ok(trim(out));
    }
    if c.len() < 9 || c.first() != Some(&VINIT) || c[c.len() - 3..] != [IMSCRIB, IFIX, TANCH] {
        return Err("expected a native IMASM numeral word".into());
    }
    let body = &c[1..c.len() - 3];
    if body.len() % 5 != 0 {
        return Err("broken native numeral cell".into());
    }
    let mut out = Vec::with_capacity(body.len() / 5);
    for cell in body.chunks(5) {
        if cell[0] != AFWD || cell[1] != CLINK || cell[2] != FSPLIT || cell[4] != FFUSE {
            return Err("broken native numeral cell".into());
        }
        bit(cell[3])?;
        out.push(cell[3]);
    }
    Ok(trim(out))
}

pub fn emit_numeral(tape: &[char]) -> String {
    if tape.len() == 1 && tape[0] == EVALT {
        return [VINIT, IMSCRIB, IFIX, TANCH].iter().collect();
    }
    let mut out = String::new();
    out.push(VINIT);
    for &b in tape {
        out.extend([AFWD, CLINK, FSPLIT, b, FFUSE]);
    }
    out.extend([IMSCRIB, IFIX, TANCH]);
    out
}

fn cmp(a: &[char], b: &[char]) -> core::cmp::Ordering {
    let a = trim(a.to_vec());
    let b = trim(b.to_vec());
    if a.len() != b.len() {
        return a.len().cmp(&b.len());
    }
    for i in (0..a.len()).rev() {
        if a[i] != b[i] {
            return bit(a[i]).unwrap().cmp(&bit(b[i]).unwrap());
        }
    }
    core::cmp::Ordering::Equal
}

fn add(a: &[char], b: &[char]) -> Tape {
    let mut out = Vec::new();
    let mut carry = false;
    for i in 0..a.len().max(b.len()) {
        let x = a
            .get(i)
            .copied()
            .map(bit)
            .transpose()
            .unwrap()
            .unwrap_or(false);
        let y = b
            .get(i)
            .copied()
            .map(bit)
            .transpose()
            .unwrap()
            .unwrap_or(false);
        out.push(mark(x ^ y ^ carry));
        carry = (x && y) || (x && carry) || (y && carry);
    }
    if carry {
        out.push(EVALF);
    }
    trim(out)
}

fn sub(a: &[char], b: &[char]) -> Tape {
    let mut out = Vec::new();
    let mut borrow = false;
    for i in 0..a.len() {
        let x = bit(a[i]).unwrap();
        let y = b
            .get(i)
            .copied()
            .map(bit)
            .transpose()
            .unwrap()
            .unwrap_or(false);
        out.push(mark(x ^ y ^ borrow));
        borrow = (!x && (y || borrow)) || (y && borrow);
    }
    trim(out)
}

fn mul(a: &[char], b: &[char]) -> Tape {
    let mut acc = vec![EVALT];
    for (i, &m) in b.iter().enumerate() {
        if bit(m).unwrap() {
            let mut row = vec![EVALT; i];
            row.extend_from_slice(a);
            acc = add(&acc, &row);
        }
    }
    trim(acc)
}

fn divmod(n: &[char], d: &[char]) -> (Tape, Tape) {
    let mut q = vec![EVALT; n.len()];
    let mut r = vec![EVALT];
    for i in (0..n.len()).rev() {
        r.insert(0, n[i]);
        r = trim(r);
        if cmp(&r, d) != core::cmp::Ordering::Less {
            r = sub(&r, d);
            q[i] = EVALF;
        }
    }
    (trim(q), trim(r))
}

fn modulo(n: &[char], d: &[char]) -> Tape {
    divmod(n, d).1
}

fn mod_add(a: &[char], b: &[char], n: &[char]) -> Tape {
    modulo(&add(a, b), n)
}

fn mod_mul(a: &[char], b: &[char], n: &[char]) -> Tape {
    modulo(&mul(a, b), n)
}

fn abs_diff(a: &[char], b: &[char]) -> Tape {
    if cmp(a, b) == core::cmp::Ordering::Less {
        sub(b, a)
    } else {
        sub(a, b)
    }
}

fn gcd(mut a: Tape, mut b: Tape) -> Tape {
    while !zero(&b) {
        let r = modulo(&a, &b);
        a = b;
        b = r;
    }
    trim(a)
}

fn rho_step(x: &[char], c: &[char], n: &[char]) -> Tape {
    mod_add(&mod_mul(x, x, n), c, n)
}

fn one() -> Tape {
    vec![EVALF]
}
fn two() -> Tape {
    vec![EVALT, EVALF]
}
fn zero(a: &[char]) -> bool {
    trim(a.to_vec()) == [EVALT]
}

struct State {
    n: Tape,
    candidate: Tape,
    remainder: Tape,
    x: Tape,
    y: Tape,
    phase: Tape,
    divisor: Tape,
    exhausted: bool,
    selected: Option<Tape>,
}

/// The interior of each operator motif (its marks between VINIT and TANCH).
/// Dispatch in `apply_morphism` keys on the whole operator word; the carrier
/// constructor recognises a motif by this interior wherever it appears inside
/// a larger operator word.
const PHASE_I: &[char] = &[FSPLIT, AFWD, EVALT, EVALF, FFUSE];
const ARITHMETIC_I: &[char] = &[FSPLIT, CLINK, EVALT, EVALF, FFUSE];
const BRANCH_I: &[char] = &[FSPLIT, EVALT, EVALF, FFUSE];
const SELECT_I: &[char] = &[FSPLIT, IMSCRIB, FFUSE];
const CONTINUE_I: &[char] = &[AFWD, CLINK];
const FIX_I: &[char] = &[IMSCRIB, IFIX];

/// Name of an operator motif, for reporting a constructed tower.
pub fn morphism_name(operator: &[char]) -> &'static str {
    if operator == PHASE { "PHASE" }
    else if operator == ARITHMETIC { "ARITHMETIC" }
    else if operator == BRANCH { "BRANCH" }
    else if operator == SELECT { "SELECT" }
    else if operator == CONTINUE { "CONTINUE" }
    else if operator == FIX { "FIX" }
    else { "?" }
}

/// Automated carrier constructor. Read an operator ob3ect word and decompose
/// its interior into the ordered sequence of factoring morphisms it realises.
/// Each recognised motif interior emits its whole operator word, so the result
/// is a tower that `execute_nested` can drive exactly like the fixed one in
/// `factor`. Marks the Grammar defines as carry rather than dispatch (AREV the
/// involution, ENGAGR as hold, and a lone IMSCRIB seed) are carried across
/// without emitting an operator. An interior mark that begins no motif and is
/// not a carry mark is refused, naming the position.
pub fn construct_carrier(operator_word: &str) -> Result<Vec<&'static [char]>, String> {
    let c: Vec<char> = operator_word.chars().collect();
    if c.first() != Some(&VINIT) || c.last() != Some(&TANCH) {
        return Err("operator word needs VINIT ⊢ and TANCH ⊣ interfaces".into());
    }
    let body = &c[1..c.len() - 1];
    // Longest interior first so PHASE/ARITHMETIC win over BRANCH, and FIX (⊙⊡)
    // wins over a lone IMSCRIB carry.
    let motifs: [(&[char], &[char]); 6] = [
        (PHASE_I, PHASE),
        (ARITHMETIC_I, ARITHMETIC),
        (BRANCH_I, BRANCH),
        (SELECT_I, SELECT),
        (CONTINUE_I, CONTINUE),
        (FIX_I, FIX),
    ];
    let mut tower: Vec<&'static [char]> = Vec::new();
    let mut i = 0;
    'scan: while i < body.len() {
        for (interior, operator) in motifs.iter() {
            if body[i..].starts_with(interior) {
                tower.push(operator);
                i += interior.len();
                continue 'scan;
            }
        }
        // Carry marks: carried across the register, they select no morphism.
        if matches!(body[i], AREV) || body[i] == '⊞' || body[i] == IMSCRIB {
            i += 1;
            continue;
        }
        return Err(format!(
            "operator mark {} at interior position {} begins no factoring morphism",
            body[i], i
        ));
    }
    Ok(tower)
}

/// Factor N using a carrier constructed from an arbitrary operator ob3ect word,
/// rather than the fixed tower in `factor`. The operator word is decomposed by
/// `construct_carrier`; the tower must be factoring-complete (able to advance a
/// candidate, decide, continue, and fix) or the missing morphisms are named.
pub fn factor_with(operator_word: &str, n_word: &str) -> Result<String, String> {
    let tower = construct_carrier(operator_word)?;
    let has = |op: &[char]| tower.iter().any(|t| *t == op);
    let mut missing = Vec::new();
    if !has(PHASE) && !has(ARITHMETIC) { missing.push("PHASE or ARITHMETIC (advance)"); }
    if !has(SELECT) { missing.push("SELECT (decide)"); }
    if !has(CONTINUE) { missing.push("CONTINUE (step the candidate)"); }
    if !has(FIX) { missing.push("FIX (latch)"); }
    if !missing.is_empty() {
        return Err(format!(
            "operator word does not carry factoring; missing: {}",
            missing.join(", ")
        ));
    }
    let n = parse_numeral(n_word)?;
    if cmp(&n, &two()) == core::cmp::Ordering::Less {
        return Err("numeral has no non-trivial factor".into());
    }
    let (_, even) = divmod(&n, &two());
    if zero(&even) {
        return Ok(emit_numeral(&two()));
    }
    let mut state = State {
        n,
        candidate: add(&two(), &one()),
        remainder: vec![EVALT],
        x: two(),
        y: two(),
        phase: one(),
        divisor: one(),
        exhausted: false,
        selected: None,
    };
    let tower_refs: Vec<&[char]> = tower.iter().map(|t| *t).collect();
    loop {
        execute_nested(&tower_refs, &mut state);
        if let Some(ref selected) = state.selected {
            return Ok(emit_numeral(selected));
        }
    }
}

fn apply_morphism(operator: &[char], state: &mut State) {
    // Dispatch is read from the operator word itself. Each operator therefore
    // remains both the boundary and the action performed at that boundary.
    if operator == PHASE {
        state.exhausted =
            cmp(&mul(&state.candidate, &state.candidate), &state.n) == core::cmp::Ordering::Greater;
        state.x = rho_step(&state.x, &state.phase, &state.n);
        state.y = rho_step(
            &rho_step(&state.y, &state.phase, &state.n),
            &state.phase,
            &state.n,
        );
    } else if operator == ARITHMETIC && !state.exhausted {
        state.remainder = divmod(&state.n, &state.candidate).1;
        state.divisor = gcd(abs_diff(&state.x, &state.y), state.n.clone());
    } else if operator == BRANCH && !state.exhausted {
        state.remainder = trim(state.remainder.clone());
    } else if operator == SELECT {
        if state.exhausted {
            state.selected = Some(state.n.clone());
        } else if cmp(&state.divisor, &one()) == core::cmp::Ordering::Greater
            && cmp(&state.divisor, &state.n) == core::cmp::Ordering::Less
        {
            state.selected = Some(state.divisor.clone());
        } else if zero(&state.remainder) {
            state.selected = Some(state.candidate.clone());
        }
    } else if operator == CONTINUE && state.selected.is_none() {
        state.candidate = add(&state.candidate, &two());
        if cmp(&state.divisor, &state.n) == core::cmp::Ordering::Equal {
            state.phase = add(&state.phase, &one());
            state.x = two();
            state.y = two();
            state.divisor = one();
        }
    } else if operator == FIX {
        if let Some(value) = state.selected.take() {
            state.selected = Some(trim(value));
        }
    }
}

fn execute_nested(operators: &[&[char]], state: &mut State) {
    if let Some((operator, continuation)) = operators.split_first() {
        apply_morphism(operator, state);
        execute_nested(continuation, state);
    }
}

/// Execute the nested phase → arithmetic → branch → selection → continuation
/// → fixation tower. Each boundary is itself an IMASM operator word; the live
/// state passed through all boundaries consists only of numeral tapes.
pub fn factor(word: &str) -> Result<String, String> {
    let n = parse_numeral(word)?;
    if cmp(&n, &two()) == core::cmp::Ordering::Less {
        return Err("numeral has no non-trivial factor".into());
    }
    let (_, even) = divmod(&n, &two());
    if zero(&even) {
        return Ok(emit_numeral(&two()));
    }
    let mut state = State {
        n,
        candidate: add(&two(), &one()),
        remainder: vec![EVALT],
        x: two(),
        y: two(),
        phase: one(),
        divisor: one(),
        exhausted: false,
        selected: None,
    };
    let tower: [&[char]; 6] = [PHASE, ARITHMETIC, BRANCH, SELECT, CONTINUE, FIX];
    loop {
        execute_nested(&tower, &mut state);
        if let Some(ref selected) = state.selected {
            return Ok(emit_numeral(selected));
        }
    }
}

/// Verify p·q == N entirely over IMASM numeral tapes: parse three words,
/// multiply p and q with the tape full-adder, compare the product to N, and
/// emit both the product and N back as IMASM words. This is mu circ delta = id
/// for the tower's own arithmetic at full RSA width.
pub fn verify(p_word: &str, q_word: &str, n_word: &str) -> Result<String, String> {
    let p = parse_numeral(p_word)?;
    let q = parse_numeral(q_word)?;
    let n = parse_numeral(n_word)?;
    let prod = trim(mul(&p, &q));
    let ok = cmp(&prod, &n) == core::cmp::Ordering::Equal;
    Ok(format!(
        "p*q == N: {ok}\nproduct: {}\nN:       {}",
        emit_numeral(&prod),
        emit_numeral(&n)
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn numeral(mut n: u64) -> String {
        if n == 0 {
            return emit_numeral(&[EVALT]);
        }
        let mut t = Vec::new();
        while n != 0 {
            t.push(mark(n & 1 == 1));
            n >>= 1;
        }
        emit_numeral(&t)
    }
    #[test]
    fn factors_marks_without_numeric_state() {
        assert_eq!(factor(&numeral(143)).unwrap(), numeral(11));
        assert_eq!(factor(&numeral(127)).unwrap(), numeral(127));
        let f = factor(&numeral(8051)).unwrap();
        assert!(f == numeral(83) || f == numeral(97));
    }
    #[test]
    fn phase_family_reaches_a_forty_bit_semiprime() {
        let p = 1_000_003u64;
        let q = 1_000_033u64;
        let f = factor(&numeral(p * q)).unwrap();
        assert!(f == numeral(p) || f == numeral(q));
    }
    #[test]
    fn phase_family_reaches_a_sixty_bit_semiprime() {
        let p = 1_000_000_007u64;
        let q = 1_000_000_009u64;
        let f = factor(&numeral(p * q)).unwrap();
        assert!(f == numeral(p) || f == numeral(q));
    }

    // A single operator word whose interior is the six motif interiors in
    // canonical order. The carrier constructor recovers the full tower.
    const FULL: &str = "⊢∈≻⊤⊥∋∈⋈⊤⊥∋∈⊤⊥∋∈⊙∋≻⋈⊙⊡⊣";

    #[test]
    fn constructor_recovers_the_full_tower() {
        let tower = construct_carrier(FULL).unwrap();
        let names: Vec<&str> = tower.iter().map(|t| morphism_name(t)).collect();
        assert_eq!(names, ["PHASE", "ARITHMETIC", "BRANCH", "SELECT", "CONTINUE", "FIX"]);
    }

    #[test]
    fn constructed_carrier_factors_like_the_fixed_tower() {
        let f = factor_with(FULL, &numeral(8051)).unwrap();
        assert!(f == numeral(83) || f == numeral(97));
        assert_eq!(f, factor(&numeral(8051)).unwrap());
    }

    #[test]
    fn incomplete_operator_word_is_named_not_run() {
        // BRANCH + SELECT + FIX but no advance/continue: refused, missing named.
        let w = "⊢∈⊤⊥∋∈⊙∋⊙⊡⊣";
        let err = factor_with(w, &numeral(8051)).unwrap_err();
        assert!(err.contains("missing"));
        assert!(err.contains("CONTINUE"));
    }

    #[test]
    fn evaluate_frame_with_arev_is_not_a_factoring_morphism() {
        // The instant semiprime extractor carries AREV ≺ inside its evaluate
        // frame (∈≻⊤≺⊥…), which no factoring morphism has; the constructor
        // names the exact obstruction rather than silently absorbing it.
        let err = construct_carrier("⊢⊙∈≻⊤≺⊥⊞⋈∋⊙⊡⊣").unwrap_err();
        assert!(err.contains("begins no factoring morphism"));
    }
}
