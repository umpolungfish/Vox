//! The Lean-compiled Curry Y drives the existing complete dialectic object.
//! S/K/I contraction duplicates references to syntax. The source object has one
//! resident owner, and every primitive descent returns it through that owner.
use alloc::{rc::Rc, string::String, vec::Vec};
use crate::resident_y::Term;
use crate::dialectic_reentry::{Descent, DialecticClosure, DialecticObject, ImscriptionLattice, ResidentScan};
use crate::factor_extract::Tape;

use crate::resident_y::Plan;
pub use crate::resident_y::ResidentY;
const LEAN_PLAN: &str = ResidentY::certified_program();

#[derive(Clone, Debug, Default)]
pub struct Operations {
    pub contractions: u64,
    pub descents: u64,
    pub temporal_ticks: u64,
    pub information_joins: u64,
    pub reversed_descents: u64,
}

#[derive(Clone, Debug)]
pub struct ExecutionRecord {
    pub source: Vec<char>,
    pub word: Vec<char>,
    pub boundary: Vec<char>,
    pub span: Vec<char>,
    pub frontier: Option<Vec<char>>,
    pub verdict: char,
    pub lattice: ImscriptionLattice,
    pub phase: u8,
    pub evidence: u8,
    pub operations: Operations,
    pub closure_pair: Option<(Vec<char>,Vec<char>)>,
}

pub struct ResidentDialectic {
    plan: Plan,
    driver: Option<ResidentY>,
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
        let driver = Some(ResidentY::from_plan(&plan));
        let scan = ResidentScan::new(plan.squares);
        Ok(Self { plan, driver, object: Some(DialecticObject::new(n)?), closure: None,
            scan, operations: Operations::default(), phase: 2, evidence: 0 })
    }

    pub fn object(&self) -> Option<&DialecticObject> { self.object.as_ref() }
    pub fn closure(&self) -> Option<&DialecticClosure> { self.closure.as_ref() }

    /// Bind the current factor-cell domains into the Y return before its first
    /// descent. The support is carried with this same N-bound object.
    pub fn bind_factor_support(
        &mut self,
        p: Vec<Option<char>>,
        q: Vec<Option<char>>,
        depth: usize,
    ) -> Result<(), String> {
        let source = self.object.as_ref().ok_or("dialectic source object is closed")?.n.clone();
        if p.iter().chain(&q).flatten().any(|mark| !matches!(mark, '⊤' | '⊥')) {
            return Err("dialectic factor support contains a non-parity mark".into());
        }
        self.scan.factor_support = Some(crate::dialectic_reentry::FactorSupport {
            source, p, q, depth,
        });
        Ok(())
    }

    /// Return the current non-Lehman type-lattice lower boundary in the
    /// arithmetic word's native numeral representation.
    pub fn midpoint_frontier(&self) -> Result<Option<Vec<char>>, String> {
        let Some(object) = self.object.as_ref() else { return Ok(None); };
        let execution = crate::dialectic_reentry::decode_imasm_execution(object.word())?;
        if execution.lattice() == crate::dialectic_reentry::ImscriptionLattice::Lehman {
            return Ok(None);
        }
        Ok(Some(crate::morphism_factor::add(object.boundary(), object.span())))
    }

    pub fn execution_record(&self, frontier: Option<Vec<char>>) -> Result<ExecutionRecord, String> {
        let (source,word,boundary,span,closure_pair)=if let Some(object)=self.object.as_ref() {
            (object.n.clone(),object.word().to_vec(),object.boundary().to_vec(),object.span().to_vec(),None)
        } else if let Some(closure)=self.closure.as_ref() {
            (closure.carrier.n.clone(),closure.word().to_vec(),closure.boundary().to_vec(),
             closure.span().to_vec(),Some((closure.carrier.p.clone(),closure.carrier.q.clone())))
        } else {
            return Err("dialectic record lost both its resident object and closure".into());
        };
        let execution=crate::dialectic_reentry::decode_imasm_execution(&word)?;
        Ok(ExecutionRecord {source,word,boundary,span,frontier,verdict:execution.four(),
            lattice:execution.lattice(),phase:self.phase,evidence:self.evidence,
            operations:self.operations.clone(),closure_pair})
    }

    fn descend_term(&mut self) -> Result<Option<Rc<Term>>, String> {
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
            Descent::B(next) => { self.object = Some(next); Ok(Some(Rc::new(Term::Resume))) }
            Descent::N(next) => { self.object = Some(next); Err("resident descent retained neutral state".into()) }
            Descent::F => Err(self.scan.failure.clone().unwrap_or_else(|| "resident descent rejected the source relation".into())),
        };
    }

    /// Execute the same certified Y driver around the resident dialectic operator.
    pub fn advance(&mut self) -> Result<bool, String> {
        let mut driver = self.driver.take().ok_or("resident Y driver is already executing")?;
        let result = driver.advance(|| {
            let term = self.descend_term()?.ok_or("dialectic operator omitted its return")?;
            Ok(matches!(term.as_ref(), Term::Done))
        });
        self.operations.contractions = driver.contractions;
        self.driver = Some(driver);
        result
    }
}

