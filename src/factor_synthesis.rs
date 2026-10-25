//! N-only synthesis retaining dialectic and GLUT state across return edges.
use alloc::{string::String, vec, vec::Vec};
use crate::{glut_system::PreparedGlutMembrane, morphism_factor as mf};
use core::cmp::Ordering;

pub struct FactorSynthesis {
    source: Vec<char>,
    glut: Option<PreparedGlutMembrane>,
    turns: usize,
    returned:Option<FactorReturn>,
    morphisms: GfeMorphismMachine,
    circuit_braid: crate::gfe_circuit_braid::GfeCircuitBraidState,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GfeMorphismApplication {
    pub position: usize,
    pub glyph: char,
    pub operator: &'static str,
    pub observation: Option<bool>,
}

/// Executes the requested GFE word against one retained resident. Split arms
/// are cloned carrier handles over the same Rc register, so their fusion checks
/// establish identity of the carried state rather than equality of summaries.
struct GfeMorphismMachine {
    source_anchor: Option<Vec<char>>,
    resident_anchor: Option<usize>,
    branch_handles: Vec<PreparedGlutMembrane>,
    branches: crate::reentry_powerset::Held<usize>,
    support: Option<bool>,
    refutation: Option<bool>,
    engaged: bool,
    fused: usize,
    monad_laws_verified: bool,
    latched: bool,
    applications: Vec<GfeMorphismApplication>,
}
impl GfeMorphismMachine {
    fn new() -> Self {
        Self { source_anchor: None, resident_anchor: None, branch_handles: Vec::new(),
            branches: crate::reentry_powerset::Held::new(),
            support: None, refutation: None, engaged: false, fused: 0,
            monad_laws_verified: false, latched: false,
            applications: Vec::new() }
    }
    fn record(&mut self, position: usize, glyph: char, operator: &'static str) {
        self.applications.push(GfeMorphismApplication { position, glyph, operator, observation: None });
    }
    fn record_observation(&mut self, position: usize, glyph: char, operator: &'static str, value: bool) {
        self.applications.push(GfeMorphismApplication {
            position, glyph, operator, observation: Some(value),
        });
    }
    fn prepare(&mut self, source: &[char], carrier: &mut PreparedGlutMembrane,
        torus_axes: Option<[usize; 12]>) -> Result<(), String> {
        if self.applications.len() != 0 { return Err("GFE word preparation ran twice".into()); }
        self.source_anchor = Some(source.to_vec());
        self.record(0, '⊣', "TANCH: bind source boundary");
        self.resident_anchor = Some(carrier.resident_identity());
        self.record(1, '⊣', "TANCH: bind resident boundary");
        if self.source_anchor.as_deref() != Some(source) || self.resident_anchor != Some(carrier.resident_identity()) {
            return Err("GFE IMSCRIB identity failed at the prepared boundary".into());
        }
        self.record(2, '⊙', "IMSCRIB: retain source and resident identity");
        let mut branch_handles = vec![carrier.clone(), carrier.clone(), carrier.clone()];
        let branches = crate::reentry_powerset::bind(
            crate::reentry_powerset::unit(0usize),
            |root| {
                if root != 0 { return Err(String::from("FSPLIT lost its resident root")); }
                Ok([1usize, 2usize].into_iter().collect())
            },
        )?;
        self.branch_handles.append(&mut branch_handles);
        self.branches = branches;
        self.monad_laws_verified = verify_resident_branch_monad(&self.branches)?;
        self.record(3, '∈', "FSPLIT: open first shared-state branch");
        self.record(4, '∈', "FSPLIT: open second shared-state branch");
        if let Some(axes) = torus_axes {
            carrier.apply_catalyst_order_axes(axes)?;
            self.record(5, '≻', "AFWD: apply singularity bit-braid and QFT1+HTSX amplitudes to live resident priorities");
        } else {
            carrier.apply_catalyst_order()?;
            self.record(5, '≻', "AFWD: permute live correlation priorities");
        }
        if self.source_anchor.as_deref() != Some(source) || self.resident_anchor != Some(carrier.resident_identity()) {
            return Err("GFE IMSCRIB identity changed under AFWD".into());
        }
        self.record(6, '⊙', "IMSCRIB: verify transformed resident identity");
        self.engaged = true;
        self.record(7, '⊞', "ENGAGR: open support/refutation evidence pair");
        Ok(())
    }
    fn close(&mut self, source: &[char], carrier: &mut PreparedGlutMembrane,
        p: &[char], q: &[char]) -> Result<(), String> {
        if self.applications.len() != 8 || self.latched || !self.engaged {
            return Err("GFE word reached return in an invalid morphism state".into());
        }
        carrier.return_catalyst_order()?;
        self.record(8, '≺', "AREV: restore the retained correlation priorities");
        let product_closed = mf::mul(p, q) == source;
        if !product_closed { return Err("GFE EVALT received an open exact product".into()); }
        self.support = Some(product_closed);
        self.record_observation(9, '⊤', "EVALT: deposit exact product support", product_closed);
        let conflicts = carrier.stats().correlation_conflicts;
        self.refutation = Some(conflicts > 0);
        self.record_observation(10, '⊥', "EVALF: deposit retained correlation-conflict observation", conflicts > 0);
        if !self.monad_laws_verified || self.branches.len() != 2
            || self.branch_handles.len() != 3 || self.branches.iter().any(|id|
            self.branch_handles[*id].resident_identity() != carrier.resident_identity()) {
            return Err("GFE FFUSE branches lost their common resident".into());
        }
        let fused = crate::reentry_powerset::bind(self.branches.clone(), |id| {
            if self.branch_handles[id].resident_identity() != carrier.resident_identity() {
                return Err(String::from("FFUSE branch escaped its resident arena"));
            }
            Ok(crate::reentry_powerset::unit(0usize))
        })?;
        let terminal = crate::reentry_powerset::multiply(
            crate::reentry_powerset::unit(fused),
        );
        if terminal != crate::reentry_powerset::unit(0usize) {
            return Err("powerset multiplication failed to fuse the resident branches".into());
        }
        self.fused += 1;
        self.record(11, '∋', "FFUSE: rejoin first branch at shared resident");
        self.fused += 1;
        self.record(12, '∋', "FFUSE: rejoin second branch at shared resident");
        if self.fused != 2 || self.support != Some(true) || self.refutation.is_none() {
            return Err("GFE IFIX lacks fused branches and both evidence coordinates".into());
        }
        Ok(())
    }

    fn latch(&mut self) -> Result<(), String> {
        if self.latched || self.fused != 2 || self.support != Some(true)
            || self.refutation.is_none() || !self.monad_laws_verified
        {
            return Err("GFE IFIX lacks fused branches, monad laws, or complete evidence".into());
        }
        self.latched = true;
        self.record(13, '⊡', "IFIX: latch terminal readout");
        Ok(())
    }
}

/// Execute the powerset monad laws over this resident's actual FSPLIT branch
/// identifiers. The split arrow returns the two retained handles; the return
/// arrow preserves each handle until FFUSE validates their common resident.
fn verify_resident_branch_monad(
    branches: &crate::reentry_powerset::Held<usize>,
) -> Result<bool, String> {
    use crate::reentry_powerset::{bind, multiply, unit};
    let root = 0usize;
    let split = |id| {
        if id == root { Ok([1usize, 2usize].into_iter().collect()) }
        else { Err(String::from("monad split received a non-root resident handle")) }
    };
    let identity = |id| Ok::<_, String>(unit(id));
    let left_unit = bind(unit(root), split)? == *branches;
    let right_unit = bind(branches.clone(), identity)? == *branches;
    let left = bind(bind(unit(root), split)?, identity)?;
    let right = bind(unit(root), |id| bind(split(id)?, identity))?;
    let associativity = left == right;
    let nested: crate::reentry_powerset::Held<
        crate::reentry_powerset::Held<crate::reentry_powerset::Held<usize>>,
    > = unit(unit(branches.clone()));
    let multiply_associativity = multiply(multiply(nested.clone()))
        == multiply(nested.into_iter().map(multiply).collect());
    let multiplication_units = multiply(unit(branches.clone())) == *branches
        && multiply(branches.iter().copied().map(unit).collect()) == *branches;
    if left_unit && right_unit && associativity && multiply_associativity && multiplication_units {
        Ok(true)
    } else {
        Err(String::from("resident powerset monad law check failed"))
    }
}

#[derive(Clone)]
pub struct FactorReturn {
    pub p: Vec<char>,
    pub q: Vec<char>,
    pub producer: &'static str,
    pub turns: usize,
    pub membrane:Option<PreparedGlutMembrane>,
    pub braid_dots: Vec<FactorBraidDot>,
    pub circuit_braid: crate::gfe_circuit_braid::GfeCircuitBraidState,
    pub catalyst_order_applied: bool,
    pub catalyst_inverse_returned: bool,
    pub morphism_applications: Vec<GfeMorphismApplication>,
    pub powerset_monad_laws_verified: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FactorBraidDot {
    pub reentry: u8,
    pub p_value: Vec<char>,
    pub q_value: Vec<char>,
    pub source_value: Vec<char>,
    pub product_closed: bool,
}

pub fn build_factor_braid_dots(
    source: &[char], p: &[char], q: &[char],
) -> Result<Vec<FactorBraidDot>, String> {
    let (mut p_value, mut q_value, mut source_value) = (mf::one(), mf::one(), mf::one());
    let mut braid_dots = Vec::with_capacity(3);
    for reentry in 1..=3 {
        p_value = mf::mul(&p_value, p);
        q_value = mf::mul(&q_value, q);
        source_value = mf::mul(&source_value, source);
        let product_closed = mf::mul(&p_value, &q_value) == source_value;
        if !product_closed {
            return Err("factor braid failed exact re-entry product closure".into());
        }
        braid_dots.push(FactorBraidDot {
            reentry,
            p_value: p_value.clone(),
            q_value: q_value.clone(),
            source_value: source_value.clone(),
            product_closed,
        });
    }
    Ok(braid_dots)
}

impl FactorSynthesis {
    pub fn circuit_braid_state(&self) -> &crate::gfe_circuit_braid::GfeCircuitBraidState {
        &self.circuit_braid
    }
    pub fn powerset_monad_laws_verified(&self) -> bool {
        self.morphisms.monad_laws_verified
    }

    pub fn resident_stats(&self) -> crate::glut_system::FoldStats {
        self.glut.as_ref().map(PreparedGlutMembrane::stats).unwrap_or_default()
    }

    /// No factor widths, seeds, factors, orders or external witnesses enter here.
    /// Derives the multi-strand horn-torus bit-twist braid permutation from N
    /// so that the resident correlation graph inherits the horn-torus braid permutation.
    pub fn new(n: &[char]) -> Result<Self, String> {
        let m = n.len().div_ceil(2);
        let bit_braid = crate::pwoat_torus::BitBraid::torus_twist(m);
        let torus_axes = bit_braid.axis_permutation();
        Self::new_with_torus_braid(
            n,
            Some(torus_axes),
            crate::geometer_factor_extractor::SmallPrimeOrbitMask::prepare()?,
        )
    }

    pub(crate) fn new_with_torus_braid(n: &[char],
        torus_axes: Option<[usize; 12]>,
        small_prime_orbit_mask: crate::geometer_factor_extractor::SmallPrimeOrbitMask,
    ) -> Result<Self, String> {
        if n.is_empty() || n.iter().any(|mark| !matches!(mark, '⊤' | '⊥')) {
            return Err("source must be a nonempty canonical bit tape".into());
        }
        let source = mf::trim(n.to_vec());
        if mf::cmp(&source, &mf::one()) != Ordering::Greater {
            return Err("proper factor production requires N > 1".into());
        }
        let torus_axes = torus_axes.unwrap_or_else(||
            crate::pwoat_torus::BitBraid::torus_twist(source.len().div_ceil(2)).axis_permutation());
        let circuit_braid = crate::geometer_factor_extractor::prepare_source_circuit_braid(
            &source, torus_axes,
        )?;
        // Y descent and the product/carry fold must re-enter through one
        // resident. The ordinary prepared carrier runs an independent square
        // lane beside that descent; the dialectic carrier keeps its frontier,
        // support graph, and exact factor/product boundary in the same state.
        let mut glut = PreparedGlutMembrane::prepare_dialectic_gfe(
            &source, &small_prime_orbit_mask,
        )?;
        let mut morphisms = GfeMorphismMachine::new();
        morphisms.prepare(&source, &mut glut, Some(circuit_braid.axis_order))?;
        glut.bind_square_torus()?;
        Ok(Self { source, glut: Some(glut), turns: 0, returned:None, morphisms, circuit_braid })
    }

    /// Bind a source-derived divisor into this prepared resident's live
    /// factor/product/carry support, then close the GFE return.
    pub(crate) fn close_candidate(&mut self, p: Vec<char>, q: Vec<char>,
        producer: &'static str) -> Result<FactorReturn, String> {
        if mf::mul(&p, &q) != self.source {
            return Err("source-derived candidate failed exact product closure".into());
        }
        let carrier = self.glut.as_mut().unwrap();
        if !carrier.stats().catalyst_order_applied {
            return Err("GFE AFWD did not reach the live correlation graph".into());
        }
        carrier.bind_source_candidate(p.clone(), q.clone(), producer)?;
        self.morphisms.close(&self.source, carrier, &p, &q)?;
        let returned = self.verified(p, q, producer)?;
        self.returned = Some(returned.clone());
        Ok(returned)
    }

    /// Fixes the immutable terminal value only after geometric, phase, SIC and
    /// exact-product readout coordinates have all been assembled.
    pub(crate) fn latch_terminal_readout(
        &mut self,
        readout: &mut crate::geometer_factor_extractor::GeometerFrameReadout,
    ) -> Result<(), String> {
        if readout.final_readout_fixed || !readout.product_closed
            || mf::mul(&readout.p, &readout.q) != self.source
        {
            return Err("GFE terminal readout is already fixed or its exact product is open".into());
        }
        self.morphisms.latch()?;
        if let Some(returned) = self.returned.as_mut() {
            returned.morphism_applications = self.morphisms.applications.clone();
        }
        readout.morphism_applications = self.morphisms.applications.clone();
        readout.final_readout_fixed = true;
        Ok(())
    }

    fn verified(&self, p: Vec<char>, q: Vec<char>, producer: &'static str)
        -> Result<FactorReturn, String> {
        if mf::cmp(&p, &mf::one()) != Ordering::Greater
            || mf::cmp(&q, &mf::one()) != Ordering::Greater
            || mf::mul(&p, &q) != self.source {
            return Err("synthesis return failed exact proper product verification".into());
        }
        let braid_dots = build_factor_braid_dots(&self.source, &p, &q)?;
        let stats = self.glut.as_ref().map(PreparedGlutMembrane::stats)
            .unwrap_or_default();
        Ok(FactorReturn {
            p, q, producer, turns: self.turns, membrane: self.glut.clone(), braid_dots,
            circuit_braid: self.circuit_braid.clone(),
            catalyst_order_applied: stats.catalyst_order_applied,
            catalyst_inverse_returned: stats.catalyst_inverse_returned,
            morphism_applications: self.morphisms.applications.clone(),
            powerset_monad_laws_verified: self.morphisms.monad_laws_verified,
        })
    }

    /// One inclusive return: dialectic descent and all resident GLUT ports.
    /// A turn is structural work, not a fixed wall-clock quantum.
    pub fn advance(&mut self) -> Result<Option<FactorReturn>, String> {
        self.advance_observed(|_| {})
    }
    pub fn advance_observed(&mut self,
        observe: impl FnMut(&crate::glut_system::FoldStats))
        -> Result<Option<FactorReturn>, String> {
        if let Some(returned)=&self.returned{return Ok(Some(returned.clone()));}
        self.turns += 1;
        if self.glut.as_mut().unwrap().advance_observed(observe) {
            let execution = self.glut.as_mut().unwrap().execute_retained()?;
            if let Some(execution) = execution {
                execution.verify(&self.source)?;
                // The remainder of the word consumes the retained inverse,
                // evaluates both evidence coordinates, rejoins both handles,
                // and fixes the terminal value.
                self.morphisms.close(&self.source, self.glut.as_mut().unwrap(), &execution.p, &execution.q)?;
                let returned=self.verified(execution.p,execution.q,"prepared-glut")?;
                self.returned=Some(returned.clone());return Ok(Some(returned));
            }
            return Err("resident exhausted without a proper factor return".into());
        }
        Ok(None)
    }
}

impl crate::membrane_interface::Membrane for FactorSynthesis {
    type Input = Vec<char>;
    type Readout = FactorReturn;
    type Continuation = Self;
    type Error = String;

    fn prepare(input: Self::Input) -> Result<Self, Self::Error> {
        Self::new(&input)
    }

    fn advance(
        &mut self,
    ) -> Result<crate::membrane_interface::MembraneProgress, Self::Error> {
        Ok(if FactorSynthesis::advance(self)?.is_some() {
            crate::membrane_interface::MembraneProgress::ReadyToClose
        } else {
            crate::membrane_interface::MembraneProgress::Open
        })
    }

    fn close(
        self,
    ) -> Result<
        crate::membrane_interface::MembraneClosure<Self, Self::Continuation, Self::Readout>,
        Self::Error,
    > {
        match self.returned.clone() {
            Some(readout) => Ok(crate::membrane_interface::MembraneClosure::Closed {
                membrane: Some(self),
                readout,
            }),
            None => Ok(crate::membrane_interface::MembraneClosure::Open(self)),
        }
    }
}
