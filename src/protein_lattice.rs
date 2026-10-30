//! Membership-resolved mechanical responses on the native powerset of FOUR.
//! Constitutive potentials are explicit inputs. Information fusion counts a
//! shared member once; different physical sources keep distinct addresses.
//! Lean laws: Biochem/PrimaLogicaMechanicalGlut.LatticeInteraction.
use alloc::vec::Vec;
use crate::{belnap::B4, belnap_relation::BelnapSet};

#[derive(Clone, Copy, Debug, Default)]
pub struct Response {
    pub energy: f64,
    pub radial_derivative: f64,
}

/// Complete native-lattice interaction spectrum. Samples may contain arbitrary
/// cooperative responses; their finite differences retain all sixteen coefficients.
/// The four inverse accumulations are checked in the canonical Lean module.
#[derive(Clone, Copy, Debug)]
pub struct Spectrum {
    pub coefficients: [Response; 16],
}

impl Spectrum {
    pub fn from_samples(mut samples: [Response; 16]) -> Self {
        for bit in [1usize, 2, 4, 8] {
            for mask in 0..16 {
                if mask & bit != 0 {
                    samples[mask].energy -= samples[mask ^ bit].energy;
                    samples[mask].radial_derivative -= samples[mask ^ bit].radial_derivative;
                }
            }
        }
        Self { coefficients: samples }
    }

    pub fn reconstruct(&self) -> [Response; 16] {
        let mut samples = self.coefficients;
        for bit in [8usize, 4, 2, 1] {
            for mask in 0..16 {
                if mask & bit != 0 {
                    samples[mask].energy += samples[mask ^ bit].energy;
                    samples[mask].radial_derivative += samples[mask ^ bit].radial_derivative;
                }
            }
        }
        samples
    }

    pub fn evaluate(&self, state: BelnapSet) -> Response {
        self.reconstruct()[state.mask() as usize]
    }
}

pub fn response(state: BelnapSet, family: &[Response; 4]) -> Response {
    let mut total = Response::default();
    for value in [B4::N, B4::T, B4::F, B4::B] {
        if state.contains(value) {
            total.energy += family[value as usize].energy;
            total.radial_derivative += family[value as usize].radial_derivative;
        }
    }
    total
}

#[derive(Clone, Copy, Debug)]
pub struct Term {
    pub source: usize,
    pub member: B4,
    pub response: Response,
}

#[derive(Debug, Default)]
pub struct Interaction {
    pub terms: Vec<Term>,
}

impl Interaction {
    pub fn insert(&mut self, source: usize, member: B4, value: Response) {
        // Re-observing one physical source replaces its current response.
        // Distinct sources remain distinct even when their FOUR member agrees.
        if let Some(term) = self.terms.iter_mut().find(|t| t.source == source) {
            *term = Term { source, member, response: value };
        } else {
            self.terms.push(Term { source, member, response: value });
        }
    }

    pub fn family(&self) -> (BelnapSet, [Response; 4]) {
        let mut state = BelnapSet::EMPTY;
        let mut family = [Response::default(); 4];
        for term in &self.terms {
            state = state.union(BelnapSet::singleton(term.member));
            family[term.member as usize].energy += term.response.energy;
            family[term.member as usize].radial_derivative += term.response.radial_derivative;
        }
        (state, family)
    }

    pub fn evaluate(&self) -> Response {
        let (state, family) = self.family();
        response(state, &family)
    }
}

#[derive(Debug)]
pub struct PairReadout {
    pub atoms: (usize, usize),
    pub distance: f64,
    pub state: BelnapSet,
    pub family: [Response; 4],
    pub terms: Vec<Term>,
}

#[cfg(feature = "protein-cli")]
impl PairReadout {
    pub fn json(&self) -> serde_json::Value {
        let samples=core::array::from_fn(|mask|
            response(BelnapSet::from_mask(mask as u8).unwrap(),&self.family));
        let spectrum=Spectrum::from_samples(samples);
        serde_json::json!({"atoms":[self.atoms.0,self.atoms.1],"distance":self.distance,
            "native_membership_mask":self.state.mask(),"members":["N","T","F","B"],
            "family":self.family.iter().map(|r|serde_json::json!({
                "energy":r.energy,"radial_derivative":r.radial_derivative})).collect::<Vec<_>>(),
            "interaction_coefficients":spectrum.coefficients.iter().map(|r|serde_json::json!({
                "energy":r.energy,"radial_derivative":r.radial_derivative})).collect::<Vec<_>>(),
            "terms":self.terms.iter().map(|t|serde_json::json!({"source":t.source,
                "member":t.member.name(),"energy":t.response.energy,
                "radial_derivative":t.response.radial_derivative})).collect::<Vec<_>>()})
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cooperative_force_survives_when_singleton_responses_vanish() {
        let samples=core::array::from_fn(|mask| {
            let cooperative=mask&6==6;
            Response{energy:if cooperative{7.0}else{0.0},
                     radial_derivative:if cooperative{-3.0}else{0.0}}
        });
        let spectrum=Spectrum::from_samples(samples);
        for mask in 0..16 {
            let expected=if mask==6{7.0}else{0.0};
            assert_eq!(spectrum.coefficients[mask].energy,expected);
            let actual=spectrum.evaluate(BelnapSet::from_mask(mask as u8).unwrap());
            assert_eq!(actual.energy,samples[mask].energy);
            assert_eq!(actual.radial_derivative,samples[mask].radial_derivative);
        }
        assert_eq!(spectrum.evaluate(BelnapSet::singleton(B4::T)).energy,0.0);
        assert_eq!(spectrum.evaluate(BelnapSet::NUMERAL).energy,7.0);
    }

    #[test]
    fn balanced_forces_retain_positive_strain_and_source_identity() {
        let mut pair = Interaction::default();
        pair.insert(10, B4::T, Response { energy: 2.0, radial_derivative: 3.0 });
        pair.insert(11, B4::F, Response { energy: 5.0, radial_derivative: -3.0 });
        pair.insert(10, B4::T, Response { energy: 2.0, radial_derivative: 3.0 });
        assert_eq!(pair.terms.len(), 2);
        let (state, family) = pair.family();
        assert_eq!(state, BelnapSet::NUMERAL);
        assert_eq!(pair.evaluate().energy, 7.0);
        assert_eq!(pair.evaluate().radial_derivative, 0.0);
        assert_eq!(response(BelnapSet::singleton(B4::B), &family).energy, 0.0);
        pair.insert(12, B4::T, Response { energy: 1.0, radial_derivative: 2.0 });
        assert_eq!(pair.evaluate().energy, 8.0);
        assert_eq!(pair.terms.len(), 3);
    }
}
