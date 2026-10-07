//! Register-bound descent ∈⊤⊥⊞∋ over native numeral tapes.
//!
//! A recurrence image collision retains both predecessors. Their squares have
//! the same residue because f(x)=x²+c. The GCD stage reads the hidden factor
//! collision; no unknown prime is supplied to the cyclic stage.

use crate::godel_calculus::{check, encode_cell_binary, Nat, Operator};
use crate::godel_product::normalize_support_product;
use crate::morphism_factor as arithmetic;
use alloc::collections::BTreeMap;
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

/// Produce preimages of the first repeated recurrence value. The computational
/// channel advances modulo N; the collision channel retains its other preimage.
/// A same-preimage collision and a global sign collision are retained as failed
/// attempts. Only the later GCD stage selects a nontrivial factor.
fn cyclic_split(
    n: &[char],
    seed: &[char],
    c: &[char],
    budget: usize,
) -> (Option<(Tape, Tape)>, usize) {
    let mut predecessors = BTreeMap::<Tape, Tape>::new();
    let mut x = seed.to_vec();
    for step in 1..=budget {
        let next = arithmetic::mul_mod_add(&x, &x, c, n);
        if let Some(y) = predecessors.get(&next) {
            return (Some((x, y.clone())), step);
        }
        predecessors.insert(next.clone(), x);
        x = next;
    }
    (None, budget)
}

pub fn run(
    source: Tape,
    seed: Tape,
    constant: Tape,
    word: &str,
    limits: Limits,
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
        evidence: Four::N,
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
        let observations = match &collision {
            Some((x, y)) => format!("\"seed\":\"{}\",\"c\":\"{}\",\"steps\":{},\"computational_channel\":\"{}\",\"leakage_channel\":\"{}\",\"collision\":\"recurrence_image\"",
                arithmetic::dec_of(&seed), arithmetic::dec_of(&constant), steps,
                arithmetic::dec_of(x), arithmetic::dec_of(y)),
            None => format!("\"seed\":\"{}\",\"c\":\"{}\",\"steps\":{},\"collision\":null",
                arithmetic::dec_of(&seed), arithmetic::dec_of(&constant), steps),
        };
        event(&mut report, start, '∈', "cyclic_split", observations);
        if collision.is_none() {
            report.events.last_mut().unwrap().status = "unclosed";
        }
        let squares = collision.as_ref().map(|(x, y)| {
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
                "\"square_congruence\":{},\"x_squared_mod_n\":{},\"y_squared_mod_n\":{}",
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
        let g = collision.as_ref().map(|(x, y)| {
            let diff = if arithmetic::cmp(x, y) == Less {
                arithmetic::sub(y, x)
            } else {
                arithmetic::sub(x, y)
            };
            arithmetic::gcd(diff, source.clone())
        });
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
        assert_eq!(
            arithmetic::mul_mod_add(&x, &x, &tape(1), &source),
            arithmetic::mul_mod_add(&y, &y, &tape(1), &source)
        );
        // A modulo-p collision alone cannot supply the square relation:
        // 1 and 7 coincide modulo 3, but their squares differ modulo 15.
        assert_ne!(
            arithmetic::mul_mod(&tape(1), &tape(1), &tape(15)),
            arithmetic::mul_mod(&tape(7), &tape(7), &tape(15))
        );
    }
}
