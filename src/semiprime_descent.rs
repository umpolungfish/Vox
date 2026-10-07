//! Register-bound descent ∈⊤⊥⊞∋ over native numeral tapes.
//!
//! Two quadratic orbit channels detect a strict hidden-factor collision by
//! GCD. The square stage lifts that observed divisor into a verified congruence.
//! No unknown prime or external factor producer is supplied to the search.

use crate::godel_calculus::{check, encode_cell_binary, Nat, Operator};
use crate::godel_product::normalize_support_product;
use crate::morphism_factor as arithmetic;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::cmp::Ordering::{Greater, Less};

pub const DESCENT: &str = "∈⊤⊥⊞∋";
pub const PROTOCOL: &str = "⊢⊙∈⊤⊥⊞∋≻⋈≺⊡⊣";
type Tape = Vec<char>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Four {
    N,
    T,
    F,
    B,
}

impl Four {
    pub fn join(self, other: Self) -> Self {
        let bits = |v| match v {
            Self::N => 0,
            Self::T => 1,
            Self::F => 2,
            Self::B => 3,
        };
        match bits(self) | bits(other) {
            0 => Self::N,
            1 => Self::T,
            2 => Self::F,
            _ => Self::B,
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::N => "N",
            Self::T => "T",
            Self::F => "F",
            Self::B => "B",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Limits {
    pub attempts: usize,
    pub steps_per_attempt: usize,
    pub total_steps: usize,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            attempts: 64,
            steps_per_attempt: 100_000,
            total_steps: 1_000_000,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Event {
    pub index: usize,
    pub symbol: char,
    pub attempt: usize,
    pub operation: &'static str,
    pub status: &'static str,
    pub observations: String,
}

#[derive(Clone, Debug)]
pub struct Pair {
    pub p: Tape,
    pub q: Tape,
    pub product: Tape,
}

#[derive(Clone, Debug)]
pub struct Report {
    pub source: Tape,
    pub word: String,
    pub events: Vec<Event>,
    pub evidence: Four,
    pub attempts: usize,
    pub steps: usize,
    pub pair: Option<Pair>,
    pub reason: &'static str,
}

fn natural(tape: &[char]) -> Nat {
    Nat::from_bits_le(tape.iter().map(|g| *g == '⊥').collect())
}

/// Exact split/product retraction for this arithmetic carrier, independently
/// checked by the Gödel codec and support-polynomial carry synthesis.
pub fn verify_pair(source: &[char], p: &[char], q: &[char]) -> Result<Pair, String> {
    let one = arithmetic::one();
    for factor in [p, q] {
        if arithmetic::cmp(factor, &one) != Greater || arithmetic::cmp(factor, source) != Less {
            return Err("factor is outside the strict interior of the source".to_string());
        }
    }
    let product = arithmetic::mul(p, q);
    let n = natural(source);
    let pn = natural(p);
    let qn = natural(q);
    let checked = check(
        &encode_cell_binary(&pn),
        Operator::Mul,
        &encode_cell_binary(&qn),
        &encode_cell_binary(&n),
    )
    .map_err(|e| e.to_string())?;
    let (synthesized, _, _) = normalize_support_product(&pn, &qn);
    if product != source || !checked.valid || synthesized != n {
        return Err("factor product does not reconstruct the retained source".to_string());
    }
    Ok(Pair {
        p: p.to_vec(),
        q: q.to_vec(),
        product,
    })
}

/// Validate this register's stage order. Boundaries are optional; they do not
/// constrain words in other registers or impose a universal head/tail rule.
pub fn word_parts(word: &str) -> Result<(Vec<char>, usize), String> {
    let glyphs: Vec<char> = word.chars().filter(|g| !g.is_whitespace()).collect();
    let stages: Vec<char> = DESCENT.chars().collect();
    let starts: Vec<usize> = glyphs
        .windows(stages.len())
        .enumerate()
        .filter_map(|(i, window)| (window == stages).then_some(i))
        .collect();
    if starts.len() != 1 {
        return Err("semiprime descent needs one ordered ∈⊤⊥⊞∋ region".to_string());
    }
    let start = starts[0];
    if glyphs[..start].iter().any(|g| !matches!(g, '⊢' | '⊙'))
        || glyphs[start + 5..]
            .iter()
            .any(|g| !matches!(g, '≻' | '⋈' | '≺' | '⊡' | '⊣'))
    {
        return Err(
            "this descent register binds initialization before its region and sealing after it"
                .to_string(),
        );
    }
    Ok((glyphs, start))
}

fn event(
    report: &mut Report,
    index: usize,
    symbol: char,
    operation: &'static str,
    observations: String,
) {
    report.events.push(Event {
        index,
        symbol,
        attempt: report.attempts,
        operation,
        status: "completed",
        observations,
    });
}

fn difference(x: &[char], y: &[char]) -> Tape {
    if arithmetic::cmp(x, y) == Less {
        arithmetic::sub(y, x)
    } else {
        arithmetic::sub(x, y)
    }
}

/// Detect a modulo-factor collision between slow and fast orbit channels.
/// Repeated-image predecessors are retained as a complete traversal safeguard.
/// Detection computes a GCD witness; final severing and release remain later.
fn cyclic_split(
    n: &[char],
    seed: &[char],
    c: &[char],
    budget: usize,
) -> (Option<(Tape, Tape)>, usize) {
    let mut previous_image: Option<(Tape, Tape)> = None;
    let mut x = seed.to_vec();
    let mut fast = seed.to_vec();
    for step in 1..=budget {
        let next = arithmetic::mul_mod_add(&x, &x, c, n);
        if let Some((image, predecessor)) = &previous_image {
            if image == &next {
                return (Some((x, predecessor.clone())), step);
            }
        }
        previous_image = Some((next.clone(), x));
        x = next;
        fast = arithmetic::mul_mod_add(&fast, &fast, c, n);
        fast = arithmetic::mul_mod_add(&fast, &fast, c, n);
        let probe = arithmetic::gcd(difference(&x, &fast), n.to_vec());
        if arithmetic::cmp(&probe, &arithmetic::one()) == Greater
            && arithmetic::cmp(&probe, n) == Less
        {
            return (Some((x, fast)), step);
        }
        if x == fast && step >= 2 {
            return (Some((x, fast)), step);
        }
    }
    (None, budget)
}

/// Lift a strict collision divisor into a full-source square congruence.
/// Odd N: X=(g+N/g)/2, Y=|g-N/g|/2, so X²-Y²=N.
/// Even N: X=N/2+1, Y=N/2-1, so X²-Y²=2N and gcd(X-Y,N)=2.
fn square_lift(n: &[char], g: &[char]) -> Result<(Tape, Tape), String> {
    let (q, rem) = arithmetic::divmod(n, g);
    if !arithmetic::zero(&rem)
        || arithmetic::cmp(g, &arithmetic::one()) != Greater
        || arithmetic::cmp(g, n) != Less
    {
        return Err("square lift requires a strict measured divisor".to_string());
    }
    let (half, parity) = arithmetic::divmod(n, &arithmetic::two());
    if arithmetic::zero(&parity) {
        return Ok((
            arithmetic::add(&half, &arithmetic::one()),
            arithmetic::sub(&half, &arithmetic::one()),
        ));
    }
    let (x, rx) = arithmetic::divmod(&arithmetic::add(g, &q), &arithmetic::two());
    let (y, ry) = arithmetic::divmod(&difference(g, &q), &arithmetic::two());
    if !arithmetic::zero(&rx) || !arithmetic::zero(&ry) {
        return Err("odd-source square lift parity failed".to_string());
    }
    Ok((x, y))
}

pub fn run(
    source: Tape,
    seed: Tape,
    constant: Tape,
    word: &str,
    limits: Limits,
) -> Result<Report, String> {
    run_with_evidence(source, seed, constant, word, limits, Four::N)
}

fn run_with_evidence(
    source: Tape,
    seed: Tape,
    constant: Tape,
    word: &str,
    limits: Limits,
    evidence: Four,
) -> Result<Report, String> {
    if arithmetic::cmp(&source, &arithmetic::two()) == Less {
        return Err("source must be an integer at least two".to_string());
    }
    if limits.attempts == 0 || limits.steps_per_attempt == 0 || limits.total_steps == 0 {
        return Err("descent budgets must be positive".to_string());
    }
    let (glyphs, start) = word_parts(word)?;
    let mut report = Report {
        source: source.clone(),
        word: glyphs.iter().collect(),
        events: Vec::new(),
        evidence,
        attempts: 0,
        steps: 0,
        pair: None,
        reason: "budget_exhausted",
    };
    for (i, g) in glyphs[..start].iter().copied().enumerate() {
        event(
            &mut report,
            i,
            g,
            if g == '⊢' {
                "bind_source"
            } else {
                "retain_source"
            },
            "\"factor_extraction\":false".to_string(),
        );
    }
    let mut seed = arithmetic::modulo(&seed, &source);
    let mut constant = arithmetic::modulo(&constant, &source);
    for attempt in 1..=limits.attempts {
        if report.steps == limits.total_steps {
            break;
        }
        report.attempts = attempt;
        let budget = limits
            .steps_per_attempt
            .min(limits.total_steps - report.steps);
        let (collision, steps) = cyclic_split(&source, &seed, &constant, budget);
        report.steps += steps;
        let probe = collision
            .as_ref()
            .map(|(x, y)| arithmetic::gcd(difference(x, y), source.clone()));
        let strict_probe = probe.as_ref().is_some_and(|g| {
            arithmetic::cmp(g, &arithmetic::one()) == Greater && arithmetic::cmp(g, &source) == Less
        });
        let observations = match &collision {
            Some((x, y)) => format!("\"seed\":\"{}\",\"c\":\"{}\",\"steps\":{},\"computational_channel\":\"{}\",\"leakage_channel\":\"{}\",\"collision\":\"{}\",\"collision_gcd\":\"{}\",\"distinct_mod_source\":{}",
                arithmetic::dec_of(&seed), arithmetic::dec_of(&constant), steps,
                arithmetic::dec_of(x), arithmetic::dec_of(y), if strict_probe { "hidden_factor" } else { "recurrence_image" }, arithmetic::dec_of(probe.as_ref().unwrap()), x != y),
            None => format!("\"seed\":\"{}\",\"c\":\"{}\",\"steps\":{},\"collision\":null",
                arithmetic::dec_of(&seed), arithmetic::dec_of(&constant), steps),
        };
        event(&mut report, start, '∈', "cyclic_split", observations);
        if collision.is_none() {
            report.events.last_mut().unwrap().status = "unclosed";
        }
        let lifted = if strict_probe {
            Some(square_lift(&source, probe.as_ref().unwrap())?)
        } else {
            collision.clone()
        };
        let squares = lifted.as_ref().map(|(x, y)| {
            (
                arithmetic::mul_mod(x, x, &source),
                arithmetic::mul_mod(y, y, &source),
            )
        });
        let congruent = squares.as_ref().map(|(x2, y2)| x2 == y2);
        event(
            &mut report,
            start + 1,
            '⊤',
            "square_congruence",
            format!(
                "\"lifted_from_collision\":{},\"X\":{},\"Y\":{},\"square_congruence\":{},\"x_squared_mod_n\":{},\"y_squared_mod_n\":{}",
                strict_probe,
                lifted.as_ref().map(|(x,_)| format!("\"{}\"", arithmetic::dec_of(x))).unwrap_or("null".to_string()),
                lifted.as_ref().map(|(_,y)| format!("\"{}\"", arithmetic::dec_of(y))).unwrap_or("null".to_string()),
                congruent
                    .map(|v| if v { "true" } else { "false" })
                    .unwrap_or("null"),
                squares
                    .as_ref()
                    .map(|(x2, _)| format!("\"{}\"", arithmetic::dec_of(x2)))
                    .unwrap_or("null".to_string()),
                squares
                    .as_ref()
                    .map(|(_, y2)| format!("\"{}\"", arithmetic::dec_of(y2)))
                    .unwrap_or("null".to_string())
            ),
        );
        if squares.is_none() {
            report.events.last_mut().unwrap().status = "not_run";
        }
        let g = lifted
            .as_ref()
            .map(|(x, y)| arithmetic::gcd(difference(x, y), source.clone()));
        let deposit = match &g {
            Some(g)
                if congruent == Some(true)
                    && arithmetic::cmp(g, &arithmetic::one()) == Greater
                    && arithmetic::cmp(g, &source) == Less =>
            {
                Four::T
            }
            Some(_) => Four::F,
            None => Four::N,
        };
        event(
            &mut report,
            start + 2,
            '⊥',
            "gcd_severing",
            format!(
                "\"gcd\":{},\"deposit\":\"{}\",\"hidden_factor_collision\":{}",
                g.as_ref()
                    .map(|g| format!("\"{}\"", arithmetic::dec_of(g)))
                    .unwrap_or("null".to_string()),
                deposit.label(),
                deposit == Four::T
            ),
        );
        if g.is_none() {
            report.events.last_mut().unwrap().status = "not_run";
        }
        let before = report.evidence;
        report.evidence = report.evidence.join(deposit);
        let after = report.evidence;
        event(&mut report, start + 3, '⊞', "knowledge_join",
            format!("\"proposition\":\"attempt_nontrivial_factor\",\"before\":\"{}\",\"deposit\":\"{}\",\"after\":\"{}\",\"retry\":{}",
                    before.label(), deposit.label(), after.label(), deposit != Four::T));
        if deposit == Four::T {
            let g = g.expect("support requires a measured gcd");
            let (q, rem) = arithmetic::divmod(&source, &g);
            if !arithmetic::zero(&rem) {
                return Err("nonzero cofactor remainder".to_string());
            }
            let pair = verify_pair(&source, &g, &q)?;
            event(&mut report, start + 4, '∋', "verify_and_fuse",
                format!("\"p\":\"{}\",\"q\":\"{}\",\"product\":\"{}\",\"godel_product_verified\":true,\"support_polynomial_product_verified\":true,\"mu_delta_source_return\":true",
                    arithmetic::dec_of(&pair.p), arithmetic::dec_of(&pair.q), arithmetic::dec_of(&pair.product)));
            report.pair = Some(pair);
            report.reason = "verified";
            break;
        }
        constant = arithmetic::modulo(&arithmetic::add(&constant, &arithmetic::one()), &source);
        if arithmetic::zero(&constant) {
            seed = arithmetic::modulo(&arithmetic::add(&seed, &arithmetic::one()), &source);
        }
    }
    if report.pair.is_some() {
        let mut forwarded = report.pair.clone();
        let mut linked = None;
        let mut latch: Option<(Pair, Four)> = None;
        for (i, g) in glyphs.iter().copied().enumerate().skip(start + 5) {
            let operation = match g {
                '≻' => "advance_verified_pair",
                '⋈' => "link_product_witness",
                '≺' => "return_source",
                '⊡' => "latch_pair",
                '⊣' => "release_pair",
                _ => unreachable!(),
            };
            match g {
                '≻' => forwarded = report.pair.clone(),
                '⋈' => {
                    let pair = forwarded.as_ref().unwrap();
                    linked = Some((
                        encode_cell_binary(&natural(&pair.p)),
                        encode_cell_binary(&natural(&pair.q)),
                    ));
                }
                '≺' => {
                    let pair = forwarded.as_ref().unwrap();
                    verify_pair(&source, &pair.p, &pair.q)?;
                }
                '⊡' => latch = Some((forwarded.as_ref().unwrap().clone(), report.evidence)),
                '⊣' => {
                    let pair = latch
                        .as_ref()
                        .map(|(p, _)| p)
                        .unwrap_or_else(|| forwarded.as_ref().unwrap());
                    verify_pair(&source, &pair.p, &pair.q)?;
                }
                _ => unreachable!(),
            }
            event(
                &mut report,
                i,
                g,
                operation,
                format!(
                    "\"factor_extraction\":false,\"source_return\":{},\"linked\":{},\"latched\":{}",
                    forwarded.as_ref().unwrap().product == source,
                    linked.is_some(),
                    latch.is_some()
                ),
            );
        }
    }
    Ok(report)
}

/// Traverse every (seed, constant) residue pair without a fixed attempt limit.
/// Each finite attempt retains only a fixed number of recurrence tapes;
/// the observer receives its trace before it is dropped. Counters use tapes.
///
/// Completeness for semiprimes: for N=pq != 4 choose an odd cofactor q and
/// seed p, y=N-p, c=y-p² (mod N). Then f(p)=f(y)=y, so the second step yields
/// gcd(|p-y|,N)=gcd(2p,N)=p. For N=4, seed 0,c=2 yields 0,2 and gcd=2.
/// Every residue pair is visited within N² attempts. No factors are used to
/// construct the traversal; p appears only in this termination proof.
/// No practical runtime bound is implied. Non-semiprime inputs are not covered.
pub fn until_closed<F>(
    source: Tape,
    seed: Tape,
    constant: Tape,
    word: &str,
    steps_per_attempt: usize,
    mut observe: F,
) -> Result<Report, String>
where
    F: FnMut(&Report, &[char], &[char]) -> Result<(), String>,
{
    if steps_per_attempt < 2 {
        return Err("complete cyclic traversal needs at least two steps per attempt".to_string());
    }
    word_parts(word)?;
    if arithmetic::cmp(&source, &arithmetic::two()) == Less {
        return Err("source must be at least two".to_string());
    }
    let mut seed = arithmetic::modulo(&seed, &source);
    let mut constant = arithmetic::modulo(&constant, &source);
    let mut evidence = Four::N;
    let mut attempts = arithmetic::decimal_to_tape("0").unwrap();
    let mut steps = attempts.clone();
    loop {
        let report = run_with_evidence(
            source.clone(),
            seed.clone(),
            constant.clone(),
            word,
            Limits {
                attempts: 1,
                steps_per_attempt,
                total_steps: steps_per_attempt,
            },
            evidence,
        )?;
        attempts = arithmetic::add(&attempts, &arithmetic::one());
        steps = arithmetic::add(
            &steps,
            &arithmetic::decimal_to_tape(&report.steps.to_string()).unwrap(),
        );
        evidence = report.evidence;
        observe(&report, &attempts, &steps)?;
        if report.pair.is_some() {
            return Ok(report);
        }
        constant = arithmetic::modulo(&arithmetic::add(&constant, &arithmetic::one()), &source);
        if arithmetic::zero(&constant) {
            seed = arithmetic::modulo(&arithmetic::add(&seed, &arithmetic::one()), &source);
        }
    }
}

impl Report {
    pub fn json(&self) -> String {
        let events: Vec<String> = self
            .events
            .iter()
            .map(|e| {
                format!(
            "{{\"i\":{},\"symbol\":\"{}\",\"attempt\":{},\"kind\":\"{}\",\"status\":\"{}\",{}}}",
            e.index, e.symbol, e.attempt, e.operation, e.status, e.observations)
            })
            .collect();
        let pair = self
            .pair
            .as_ref()
            .map(|p| {
                format!(
                    "[\"{}\",\"{}\"]",
                    arithmetic::dec_of(&p.p),
                    arithmetic::dec_of(&p.q)
                )
            })
            .unwrap_or("null".to_string());
        format!("{{\"backend\":\"semiprime-descent\",\"source\":\"{}\",\"word\":\"{}\",\"status\":\"{}\",\"evidence\":\"{}\",\"attempts\":{},\"steps\":{},\"factors\":{},\"events\":[{}]}}",
            arithmetic::dec_of(&self.source), self.word, self.reason, self.evidence.label(),
            self.attempts, self.steps, pair, events.join(","))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn complete_parameter_traversal_closes_and_retains_evidence() {
        for n in [4, 6, 9, 15, 25, 35, 49, 77, 143, 8051] {
            let report = until_closed(tape(n), tape(0), tape(0), DESCENT, 2, |r, attempts, _| {
                assert!(arithmetic::cmp(attempts, &tape(n * n)) != Greater);
                assert!(r.steps <= 2);
                Ok(())
            })
            .unwrap();
            assert_eq!(report.pair.unwrap().product, tape(n));
            assert_eq!(report.evidence, Four::B);
        }
        assert!(until_closed(tape(15), tape(0), tape(0), DESCENT, 1, |_, _, _| Ok(())).is_err());
    }

    #[test]
    fn complete_traversal_witness_proof_controls() {
        for p in [2, 3, 5, 7, 11, 13] {
            for q in [3, 5, 7, 11, 13] {
                let n = tape(p * q);
                let x = tape(p);
                let y = arithmetic::sub(&n, &x);
                let square = arithmetic::mul_mod(&x, &x, &n);
                let c = arithmetic::modulo(&arithmetic::sub(&arithmetic::add(&y, &n), &square), &n);
                let (pair, count) = cyclic_split(&n, &x, &c, 2);
                assert_eq!(count, 2);
                let (a, b) = pair.unwrap();
                assert_eq!(a, y);
                assert_eq!(b, x);
                let diff = arithmetic::sub(&a, &b);
                assert_eq!(arithmetic::gcd(diff, n), tape(p));
            }
        }
    }

    fn tape(n: u64) -> Tape {
        arithmetic::decimal_to_tape(&n.to_string()).unwrap()
    }
    fn extract(n: u64) -> Report {
        run(
            tape(n),
            tape(2),
            tape(1),
            PROTOCOL,
            Limits {
                attempts: 256,
                steps_per_attempt: 10_000,
                total_steps: 100_000,
            },
        )
        .unwrap()
    }
    #[test]
    fn four_join_retains_failure_when_support_arrives() {
        assert_eq!(Four::F.join(Four::F), Four::F);
        assert_eq!(Four::F.join(Four::T), Four::B);
        for a in [Four::N, Four::T, Four::F, Four::B] {
            assert_eq!(a.join(a), a);
            for b in [Four::N, Four::T, Four::F, Four::B] {
                assert_eq!(a.join(b), b.join(a));
            }
        }
    }
    #[test]
    fn semiprimes_reconstruct_only_inside_descent() {
        for n in [4, 6, 9, 15, 21, 25, 35, 49, 77, 143, 8051, 10403, 1_022_117] {
            let report = extract(n);
            let pair = report
                .pair
                .as_ref()
                .unwrap_or_else(|| panic!("{}", report.json()));
            assert_eq!(pair.product, tape(n));
            let fuse = report
                .events
                .iter()
                .find(|e| e.operation == "verify_and_fuse")
                .unwrap();
            assert_eq!(fuse.symbol, '∋');
            assert!(report
                .events
                .iter()
                .filter(|e| e.symbol == '⊤')
                .all(|e| e.observations.contains("true")));
            assert!(!report
                .events
                .iter()
                .filter(|e| matches!(e.symbol, '⊢' | '⊙' | '≻' | '⋈' | '≺' | '⊡' | '⊣'))
                .any(|e| e.operation == "gcd_severing" || e.operation == "verify_and_fuse"));
        }
    }
    #[test]
    fn failed_attempts_survive_success() {
        let report = run(
            tape(8051),
            tape(0),
            tape(0),
            PROTOCOL,
            Limits {
                attempts: 256,
                steps_per_attempt: 10_000,
                total_steps: 100_000,
            },
        )
        .unwrap();
        assert!(report.pair.is_some());
        assert!(report.events.iter().any(
            |e| e.operation == "knowledge_join" && e.observations.contains("\"deposit\":\"F\"")
        ));
        assert_eq!(report.evidence, Four::B);
    }
    #[test]
    fn prime_and_budget_do_not_release_factors() {
        let report = extract(101);
        assert!(report.pair.is_none());
        assert_eq!(report.evidence, Four::F);
        assert!(!report
            .events
            .iter()
            .any(|e| e.symbol == '∋' || e.symbol == '⊣'));
        let report = run(
            tape(8051),
            tape(2),
            tape(1),
            DESCENT,
            Limits {
                attempts: 1,
                steps_per_attempt: 1,
                total_steps: 1,
            },
        )
        .unwrap();
        assert_eq!(report.evidence, Four::N);
        assert!(report.pair.is_none());
    }
    #[test]
    fn product_tampering_and_stage_reordering_are_rejected() {
        assert!(verify_pair(&tape(15), &tape(3), &tape(7)).is_err());
        assert!(verify_pair(&tape(15), &tape(1), &tape(15)).is_err());
        assert!(word_parts("∈⊥⊤⊞∋").is_err());
        assert!(word_parts(DESCENT).is_ok());
    }
    #[test]
    fn wide_source_and_imscribed_input_keep_exact_product() {
        let source =
            arithmetic::decimal_to_tape("340282366920938463463374607431768211454").unwrap();
        // Parameters derived solely from the source and seed produce two
        // preimages of a fixed recurrence value, without a factor oracle.
        let constant = arithmetic::sub(&source, &tape(6));
        let report = run(
            source.clone(),
            tape(2),
            constant,
            DESCENT,
            Limits {
                attempts: 1,
                steps_per_attempt: 4,
                total_steps: 4,
            },
        )
        .unwrap();
        let pair = report.pair.unwrap();
        assert_eq!(pair.p, tape(2));
        assert_eq!(pair.product, source);
        assert_eq!(natural(&pair.q).bits_le().len(), 127);
    }
    #[test]
    fn semiprime_census_and_cycle_channel_control() {
        let primes = [2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47];
        for (i, p) in primes.iter().copied().enumerate() {
            for q in primes[i..].iter().copied() {
                let report = extract(p * q);
                assert!(report.pair.is_some(), "{}", report.json());
            }
        }
        let source = tape(8051);
        let (pair, _) = cyclic_split(&source, &tape(2), &tape(1), 10_000);
        let (x, y) = pair.unwrap();
        let observed = arithmetic::gcd(difference(&x, &y), source.clone());
        assert!(arithmetic::cmp(&observed, &tape(1)) == Greater);
        assert!(arithmetic::cmp(&observed, &source) == Less);
        let (x, y) = square_lift(&source, &observed).unwrap();
        assert_eq!(
            arithmetic::mul_mod(&x, &x, &source),
            arithmetic::mul_mod(&y, &y, &source)
        );
        // A modulo-p collision alone cannot supply the square relation:
        // 1 and 7 coincide modulo 3, but their squares differ modulo 15.
        assert_ne!(
            arithmetic::mul_mod(&tape(1), &tape(1), &tape(15)),
            arithmetic::mul_mod(&tape(7), &tape(7), &tape(15))
        );
    }

    #[test]
    fn early_factor_collision_lifts_the_original_counterexample() {
        let report = run(
            tape(15),
            tape(1),
            tape(6),
            DESCENT,
            Limits {
                attempts: 1,
                steps_per_attempt: 1,
                total_steps: 1,
            },
        )
        .unwrap();
        assert!(report.pair.is_some());
        assert_eq!(report.steps, 1);
        let split = report.events.iter().find(|e| e.symbol == '∈').unwrap();
        assert!(split.observations.contains("\"collision_gcd\":\"3\""));
        assert!(split.observations.contains("\"distinct_mod_source\":true"));
        let square = report.events.iter().find(|e| e.symbol == '⊤').unwrap();
        assert!(square
            .observations
            .contains("\"lifted_from_collision\":true"));
        assert!(square.observations.contains("\"X\":\"4\",\"Y\":\"1\""));
        assert!(square.observations.contains("\"square_congruence\":true"));
    }

    #[test]
    fn square_lift_covers_even_odd_and_prime_square_sources() {
        for (n, g) in [(4, 2), (6, 3), (15, 3), (49, 7), (77, 11), (143, 13)] {
            let (x, y) = square_lift(&tape(n), &tape(g)).unwrap();
            assert_eq!(
                arithmetic::mul_mod(&x, &x, &tape(n)),
                arithmetic::mul_mod(&y, &y, &tape(n))
            );
            let severed = arithmetic::gcd(difference(&x, &y), tape(n));
            assert!(verify_pair(
                &tape(n),
                &severed,
                &arithmetic::divmod(&tape(n), &severed).0
            )
            .is_ok());
        }
        for g in [1, 4, 15] {
            assert!(square_lift(&tape(15), &tape(g)).is_err());
        }
    }

    #[test]
    fn lifted_descent_has_no_fixed_source_bit_width() {
        for bits in [128, 522, 4096] {
            // 2*(2^(bits-1)-1). Only source and source-derived cycle parameters
            // enter execution. This control checks width, not primality proof.
            let mut source = alloc::vec!['⊥'; bits];
            source[0] = '⊤';
            let c = arithmetic::sub(&source, &tape(6));
            let report = run(
                source.clone(),
                tape(2),
                c,
                DESCENT,
                Limits {
                    attempts: 1,
                    steps_per_attempt: 2,
                    total_steps: 2,
                },
            )
            .unwrap();
            let pair = report.pair.unwrap();
            assert_eq!(pair.p, tape(2));
            assert_eq!(pair.product, source);
            assert_eq!(pair.q.len(), bits - 1);
        }
    }
}
