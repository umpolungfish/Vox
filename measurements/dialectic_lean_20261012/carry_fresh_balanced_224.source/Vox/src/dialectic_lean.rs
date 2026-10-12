//! The Lean-compiled Curry Y drives the existing complete dialectic object.
//! S/K/I contraction duplicates references to syntax. The source object has one
//! resident owner, and every primitive descent returns it through that owner.
use alloc::{rc::Rc, string::String, vec::Vec};
use crate::dialectic_reentry::{Descent, DialecticClosure, DialecticObject, ResidentScan};
use crate::factor_extract::Tape;

const LEAN_PLAN: &str = include_str!("../measurements/dialectic_lean_20261012/lean_plan.txt");

#[derive(Clone, Debug)]
enum Term { I, K, S, Operator, Payload, Done, App(Rc<Term>, Rc<Term>) }
fn app(f: Rc<Term>, x: Rc<Term>) -> Rc<Term> { Rc::new(Term::App(f, x)) }

fn numbers(report: &str, key: &str, length: usize) -> Result<Vec<u8>, String> {
    let line = report.lines().find_map(|s| s.strip_prefix(key))
        .ok_or_else(|| format!("Lean plan omitted {key}"))?;
    let values = line.trim().trim_start_matches('[').trim_end_matches(']')
        .split(',').map(|s| s.trim().parse::<u8>().map_err(|_| String::from("invalid Lean table")))
        .collect::<Result<Vec<_>, _>>()?;
    if values.len() != length { return Err(format!("wrong Lean table length for {key}")); }
    Ok(values)
}

#[derive(Clone, Debug)]
struct Plan { y: Rc<Term>, prefix_contractions: u64, negation: [u8; 16], join: [u8; 256], squares: [bool; 16] }
impl Plan {
    fn load(report: &str) -> Result<Self, String> {
        if !report.lines().any(|s| s == "dialectic-plan=Lean-certified") {
            return Err("missing Lean compilation boundary".into());
        }
        let mut stack = Vec::new();
        let text = report.lines().find_map(|s| s.strip_prefix("Y-postfix="))
            .ok_or("missing compiled Y")?;
        for token in text.split_whitespace() {
            let term = match token {
                "I" => Rc::new(Term::I), "K" => Rc::new(Term::K), "S" => Rc::new(Term::S),
                "@" => {
                    let x = stack.pop().ok_or("Y application missing argument")?;
                    let f = stack.pop().ok_or("Y application missing function")?;
                    app(f, x)
                }
                _ => return Err("unknown compiled Y token".into()),
            };
            stack.push(term);
        }
        if stack.len() != 1 { return Err("compiled Y left multiple terms".into()); }
        let negation: [u8; 16] = numbers(report, "negation=", 16)?.try_into().unwrap();
        let join: [u8; 256] = numbers(report, "join-information=", 256)?.try_into().unwrap();
        if negation.iter().chain(join.iter()).any(|v| *v > 15) {
            return Err("compiled register table escaped SIXTEEN_3".into());
        }
        let values = numbers(report, "square-support=", 16)?;
        if values.iter().any(|v| *v > 1) { return Err("invalid square support".into()); }
        let squares = core::array::from_fn(|i| values[i] == 1);
        let prefix_contractions = report.lines().find_map(|s| s.strip_prefix("Y-prefix-contractions="))
            .ok_or("missing Lean Y prefix measurement")?.parse::<u64>()
            .map_err(|_| String::from("invalid Lean Y prefix measurement"))?;
        if prefix_contractions == 0 { return Err("Lean Y did not reach the resident operator".into()); }
        Ok(Self { y: stack.pop().unwrap(), prefix_contractions, negation, join, squares })
    }
}

#[derive(Clone, Debug, Default)]
pub struct Operations {
    pub contractions: u64,
    pub descents: u64,
    pub temporal_ticks: u64,
    pub information_joins: u64,
    pub reversed_descents: u64,
}

pub struct ResidentDialectic {
    plan: Plan,
    term: Rc<Term>,
    object: Option<DialecticObject>,
    closure: Option<DialecticClosure>,
    pub scan: ResidentScan,
    pub operations: Operations,
    pub phase: u8,
    pub evidence: u8,
}

impl ResidentDialectic {
    /// N alone initializes the same source object as the recovered producer.
    pub fn new(n: Tape) -> Result<Self, String> {
        let plan = Plan::load(LEAN_PLAN)?;
        let term = app(app(plan.y.clone(), Rc::new(Term::Operator)), Rc::new(Term::Payload));
        let scan = ResidentScan::new(plan.squares);
        Ok(Self { plan, term, object: Some(DialecticObject::new(n)?), closure: None,
            scan, operations: Operations::default(), phase: 2, evidence: 0 })
    }

    pub fn object(&self) -> Option<&DialecticObject> { self.object.as_ref() }
    pub fn closure(&self) -> Option<&DialecticClosure> { self.closure.as_ref() }

    fn root(&mut self, term: &Rc<Term>) -> Result<Option<Rc<Term>>, String> {
        let Term::App(f, x) = term.as_ref() else { return Ok(None); };
        if matches!(f.as_ref(), Term::I) { return Ok(Some(x.clone())); }
        if let Term::App(g, y) = f.as_ref() {
            if matches!(g.as_ref(), Term::K) { return Ok(Some(y.clone())); }
            if matches!(g.as_ref(), Term::Operator) && matches!(x.as_ref(), Term::Payload) {
                let object = self.object.take().ok_or("Y read a released source object")?;
                self.scan.reverse = self.phase == 4;
                self.operations.reversed_descents += u64::from(self.scan.reverse);
                let result = object.descend_with_kernel(&mut self.scan);
                self.operations.descents += 1;
                let observation = match result.four() { 'T' => 2, 'F' => 4, 'B' => 8, 'N' => 1,
                    _ => return Err("unknown dialectic observation".into()) };
                self.evidence = self.plan.join[usize::from(self.evidence) * 16 + observation];
                self.operations.information_joins += 1;
                self.phase = self.plan.negation[usize::from(self.phase)];
                self.operations.temporal_ticks += 1;
                return match result {
                    Descent::T(closure) => { self.closure = Some(closure); Ok(Some(Rc::new(Term::Done))) }
                    Descent::B(next) => { self.object = Some(next); Ok(Some(app(y.clone(), x.clone()))) }
                    Descent::N(next) => { self.object = Some(next); Err("resident descent retained neutral state".into()) }
                    Descent::F => Err("resident descent rejected the source relation".into()),
                };
            }
            if let Term::App(h, z) = g.as_ref() {
                if matches!(h.as_ref(), Term::S) {
                    return Ok(Some(app(app(z.clone(), x.clone()), app(y.clone(), x.clone()))));
                }
            }
        }
        Ok(None)
    }

    fn reduce(&mut self, term: &Rc<Term>) -> Result<Option<Rc<Term>>, String> {
        if let Some(next) = self.root(term)? { return Ok(Some(next)); }
        if let Term::App(f, x) = term.as_ref() {
            if let Some(next) = self.reduce(f)? { return Ok(Some(app(next, x.clone()))); }
            if let Some(next) = self.reduce(x)? { return Ok(Some(app(f.clone(), next))); }
        }
        Ok(None)
    }

    /// One resident descent, including the actual Y contractions that reach it.
    pub fn advance(&mut self) -> Result<bool, String> {
        if self.closure.is_some() { return Ok(true); }
        let before = self.operations.descents;
        loop {
            let term = self.term.clone();
            self.term = self.reduce(&term)?.ok_or("resident Y halted before descent")?;
            self.operations.contractions += 1;
            if self.operations.descents != before {
                if before == 0 && self.operations.contractions != self.plan.prefix_contractions + 1 {
                    return Err("resident Y prefix differs from measured Lean contractions".into());
                }
                return Ok(self.closure.is_some());
            }
        }
    }
}
