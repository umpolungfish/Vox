//! N-only synthesis retaining dialectic and GLUT state across return edges.
use alloc::{string::String, vec::Vec};
use crate::{dialectic_reentry::{Descent, DialecticObject},
    glut_system::PreparedGlutMembrane, morphism_factor as mf};
use core::cmp::Ordering;

pub struct FactorSynthesis {
    source: Vec<char>,
    dialectic: Option<DialecticObject>,
    glut: Option<PreparedGlutMembrane>,
    turns: usize,
}

pub struct FactorReturn {
    pub p: Vec<char>,
    pub q: Vec<char>,
    pub producer: &'static str,
    pub turns: usize,
}

impl FactorSynthesis {
    /// No factor widths, seeds, factors, orders or external witnesses enter here.
    pub fn new(n: &[char]) -> Result<Self, String> {
        if n.is_empty() || n.iter().any(|mark| !matches!(mark, '⊤' | '⊥')) {
            return Err("source must be a nonempty canonical bit tape".into());
        }
        let source = mf::trim(n.to_vec());
        if mf::cmp(&source, &mf::one()) != Ordering::Greater {
            return Err("proper factor production requires N > 1".into());
        }
        Ok(Self { dialectic: DialecticObject::new(source.clone()).ok(),
            source, glut: None, turns: 0 })
    }

    fn verified(&self, p: Vec<char>, q: Vec<char>, producer: &'static str)
        -> Result<FactorReturn, String> {
        if mf::cmp(&p, &mf::one()) != Ordering::Greater
            || mf::cmp(&q, &mf::one()) != Ordering::Greater
            || mf::mul(&p, &q) != self.source {
            return Err("synthesis return failed exact proper product verification".into());
        }
        Ok(FactorReturn { p, q, producer, turns: self.turns })
    }

    /// One inclusive return: dialectic descent and all resident GLUT ports.
    /// A turn is structural work, not a fixed wall-clock quantum.
    pub fn advance(&mut self) -> Result<Option<FactorReturn>, String> {
        self.advance_observed(|_| {})
    }
    pub fn advance_observed(&mut self,
        observe: impl FnMut(&crate::glut_system::FoldStats))
        -> Result<Option<FactorReturn>, String> {
        self.turns += 1;
        if let Some(object) = self.dialectic.take() {
            match object.descend() {
                Descent::T(closed) => return self.verified(closed.carrier.p,
                    closed.carrier.q, "dialectic").map(Some),
                Descent::B(next) | Descent::N(next) => self.dialectic = Some(next),
                Descent::F => {},
            }
        }
        if self.glut.is_none() {
            self.glut = Some(PreparedGlutMembrane::prepare(&self.source)?);
        }
        if self.glut.as_mut().unwrap().advance_observed(observe) {
            let execution = self.glut.take().unwrap().execute()?;
            if let Some(execution) = execution {
                execution.verify(&self.source)?;
                return self.verified(execution.p, execution.q, "prepared-glut").map(Some);
            }
            return Err("resident exhausted without a proper factor return".into());
        }
        Ok(None)
    }
}
