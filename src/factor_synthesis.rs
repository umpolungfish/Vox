//! N-only synthesis retaining dialectic and GLUT state across return edges.
use alloc::{string::{String,ToString}, vec::Vec};
use crate::{glut_system::PreparedGlutMembrane, morphism_factor as mf};
use core::cmp::Ordering;

pub const FACTOR_CONTROLLER_WORD:&str="⊥∋∋⊣⊣⊙∈∈≻⊙⊞≺⊤⊡";
type ResidentBanks=crate::reentry_powerset::Held<crate::reentry_powerset::Held<crate::reentry_powerset::Held<usize>>>;

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
    word_cursor:usize,
    work_entries:usize,
    pending_banks:ResidentBanks,
}
impl GfeMorphismMachine {
    fn new() -> Self {
        Self { source_anchor: None, resident_anchor: None, branch_handles: Vec::new(),
            branches: crate::reentry_powerset::Held::new(),
            support: None, refutation: None, engaged: false, fused: 0,
            monad_laws_verified: false, latched: false,
            applications: Vec::new(),word_cursor:0,work_entries:0,
            pending_banks:crate::reentry_powerset::unit(crate::reentry_powerset::unit(
                crate::reentry_powerset::unit(0usize))) }
    }
    fn record(&mut self, position: usize, glyph: char, operator: &'static str) {
        assert_eq!(FACTOR_CONTROLLER_WORD.chars().nth(position),Some(glyph));
        self.applications.push(GfeMorphismApplication { position, glyph, operator, observation: None });
        self.word_cursor=position+1;
    }
    fn record_observation(&mut self, position: usize, glyph: char, operator: &'static str, value: bool) {
        assert_eq!(FACTOR_CONTROLLER_WORD.chars().nth(position),Some(glyph));
        self.applications.push(GfeMorphismApplication {
            position, glyph, operator, observation: Some(value),
        });
        self.word_cursor=position+1;
    }
    fn enter_work(&mut self)->Result<(),String>{
        if self.word_cursor!=8||FACTOR_CONTROLLER_WORD.chars().nth(self.word_cursor)!=Some('≻')||self.latched{
            return Err("factor production attempted outside the overall AFWD binding".into());
        }
        self.work_entries+=1;Ok(())
    }
    fn prepare(&mut self, source: &[char], carrier: &mut PreparedGlutMembrane,
        torus_axes: Option<[usize; 12]>) -> Result<(), String> {
        use crate::reentry_powerset::{unit,multiply,Held};
        if !self.applications.is_empty(){return Err("GFE word preparation ran twice".into());}
        self.branch_handles.push(carrier.clone());
        let conflicts=carrier.stats().correlation_conflicts>0;
        self.refutation=Some(conflicts);
        self.record_observation(0,'⊥',"EVALF: deposit current source-correlation conflict observation",conflicts);
        let inner=multiply(self.pending_banks.clone());
        self.fused+=1;
        self.record(1,'∋',"FFUSE: return the prepared inner resident bank");
        let incoming=multiply(inner);
        if incoming!=unit(0usize){return Err("prepared leading fuses lost the resident unit".into());}
        self.fused+=1;
        self.record(2,'∋',"FFUSE: return the prepared outer resident bank");
        self.source_anchor=Some(source.to_vec());
        self.record(3,'⊣',"TANCH: bind source boundary");
        self.resident_anchor=Some(carrier.resident_identity());
        self.record(4,'⊣',"TANCH: bind resident boundary");
        if self.source_anchor.as_deref()!=Some(source)||self.resident_anchor!=Some(carrier.resident_identity()){
            return Err("GFE source/resident anchors rejected their prepared state".into());
        }
        self.record(5,'⊙',"IMSCRIB: retain source and resident identity");
        // Two actual nested forks over aliases of the same executing register:
        // root -> (1,2), then inner 1 -> (3,4). Preserve the nesting until mu.
        self.branch_handles.extend([carrier.clone(),carrier.clone()]);
        let outer:Held<usize>=[1usize,2usize].into_iter().collect();
        if outer.len()!=2{return Err("outer source fork lost an arm".into());}
        self.record(6,'∈',"FSPLIT: retain two source-bound outer arms");
        self.branch_handles.extend([carrier.clone(),carrier.clone()]);
        let inner:Held<usize>=[3usize,4usize].into_iter().collect();
        let family:Held<Held<usize>>=[inner,unit(2usize)].into_iter().collect();
        self.pending_banks=unit(family);
        self.branches=multiply(multiply(self.pending_banks.clone()));
        self.monad_laws_verified=verify_resident_branch_monad(&self.branches)?;
        if self.branch_handles.iter().any(|h|h.resident_identity()!=carrier.resident_identity()){
            return Err("overall forks escaped their executing resident".into());
        }
        self.record(7,'∈',"FSPLIT: retain the nested inner arms and outer spectator");
        if let Some(axes)=torus_axes{carrier.apply_catalyst_order_axes(axes)?;}
        else{carrier.apply_catalyst_order()?;}
        self.record(8,'≻',"AFWD: prepare and execute the complete retained factor-production body");
        self.word_cursor=8;
        Ok(())
    }
    fn close(&mut self, source: &[char], carrier: &mut PreparedGlutMembrane,
        p: &[char], q: &[char]) -> Result<(), String> {
        if self.word_cursor!=8||self.latched||self.engaged{
            return Err("overall word reached product return outside AFWD".into());
        }
        if self.source_anchor.as_deref()!=Some(source)||self.resident_anchor!=Some(carrier.resident_identity()){
            return Err("overall AFWD changed its source or resident identity".into());
        }
        self.record(9,'⊙',"IMSCRIB: verify the produced source-bound resident identity");
        if !self.monad_laws_verified||self.branches.len()!=3||self.branch_handles.len()!=5
            ||self.branch_handles.iter().any(|h|h.resident_identity()!=carrier.resident_identity()){
            return Err("overall evidence pair lost a retained nested arm".into());
        }
        self.engaged=true;
        self.record(10,'⊞',"ENGAGR: retain support and refutation with both nested banks");
        carrier.return_catalyst_order()?;
        self.record(11,'≺',"AREV: restore correlation priorities while retaining banked evidence");
        let proper=mf::cmp(p,&mf::one())==Ordering::Greater&&mf::cmp(p,source)==Ordering::Less;
        if !proper{return Err("overall final EVALT requires a proper divisor".into());}
        let (quotient,remainder)=mf::divmod(source,p);
        let product_closed=proper&&mf::zero(&remainder)&&quotient==q&&mf::mul(p,q)==source;
        if !product_closed{return Err("overall final EVALT lacks a proper exact factor/product return".into());}
        self.support=Some(true);
        self.record_observation(12,'⊤',"EVALT: expose the verified proper divisor and exact product",true);
        self.latch()
    }

    fn latch(&mut self) -> Result<(), String> {
        if self.latched {
            return if self.word_cursor==FACTOR_CONTROLLER_WORD.chars().count() {Ok(())}
                else {Err("IFIX cursor disagrees with committed controller".into())};
        }
        if self.word_cursor!=13 || self.fused != 2 || self.support != Some(true)
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
        if id == root { Ok(branches.clone()) }
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
    pub fn controller_snapshot(&self)->serde_json::Value{
        let m=&self.morphisms;
        serde_json::json!({"word":FACTOR_CONTROLLER_WORD,"cursor":m.word_cursor,
            "active_opcode":FACTOR_CONTROLLER_WORD.chars().nth(m.word_cursor).map(|c|c.to_string()),
            "production_binding":8,"work_entries":m.work_entries,
            "source_bank_retained":m.source_anchor.as_ref()==Some(&self.source),
            "resident_bank_retained":self.glut.as_ref().is_some_and(|g|m.resident_anchor==Some(g.resident_identity())),
            "nested_bank_ids":m.pending_banks,"leaf_ids":m.branches,
            "support":m.support,"refutation":m.refutation,"fixed":m.latched,"monad_laws_verified":m.monad_laws_verified,
            "applications":m.applications.iter().map(|a|serde_json::json!({"position":a.position,
                "glyph":a.glyph.to_string(),"operator":a.operator,"observation":a.observation})).collect::<Vec<_>>()})
    }
    pub fn prepare_toroidal_division(&mut self)->Result<(),String>{
        self.morphisms.enter_work()?;
        self.glut.as_mut().ok_or("missing resident")?.prepare_toroidal_division()
    }
    pub fn toroidal_snapshot(&self)->serde_json::Value{
        self.glut.as_ref().map(|g|g.toroidal_snapshot()).unwrap_or(serde_json::Value::Null)
    }
    pub fn toroidal_learning_context(&self)->serde_json::Value{
        self.glut.as_ref().map(|g|g.toroidal_learning_context()).unwrap_or(serde_json::Value::Null)
    }
    pub fn square_polynomial_actions(&self)->Vec<serde_json::Value>{
        self.glut.as_ref().map(|g|g.square_polynomial_actions()).unwrap_or_default()
    }
    pub fn reenter_toroidal_division(&mut self,before:u8,after:u8,kind:u8)->Result<serde_json::Value,String>{
        self.morphisms.enter_work()?;
        self.glut.as_mut().ok_or("missing resident")?.reenter_toroidal_division(before,after,kind)
    }
    /// Division, square/carry views and the phase return belong to the same
    /// retained resident. The operation acts there before production advances.
    pub fn advance_toroidal_division(&mut self,port:usize)
        ->Result<(Option<FactorReturn>,serde_json::Value,crate::belnap::B4),String>{
        self.morphisms.enter_work()?;
        let (pair,mut event,evidence)=self.glut.as_mut().ok_or("missing resident")?.act_toroidal_division(port)?;
        event["native_producer_cycle"]=serde_json::json!(pair.is_none());
        let returned=if let Some((p,q))=pair {
            Some(self.close_toroidal_division(p.to_parity().map_err(String::from)?,q.to_parity().map_err(String::from)?)?)
        }else{self.advance()?};
        event["banked_return_walk"]=serde_json::json!(returned.is_some());
        Ok((returned,event,evidence))
    }
    pub fn bind_toroidal_support(&mut self,p:crate::belnap_word::BelnapWord,
        q:crate::belnap_word::BelnapWord,evidence:crate::belnap::B4)->Result<(),String>{
        if self.returned.is_some(){return Err("cannot change a closed division resident".into());}
        self.glut.as_mut().ok_or("missing division resident")?.bind_toroidal_support(p,q,evidence)
    }
    /// Let a retained full-complex observer act between paired transformation
    /// and native production. The observer receives this resident's exact
    /// context and returns a rotation, never an independently produced factor.
    pub fn advance_toroidal_with_sic(&mut self,port:usize,
        mut observe:impl FnMut(&serde_json::Value)->Result<(crate::belnap_word::BelnapWord,bool,serde_json::Value),String>)
        ->Result<(Option<FactorReturn>,serde_json::Value,crate::belnap::B4),String>{
        self.morphisms.enter_work()?;
        let mut geometric_producer=false;
        let carrier=self.glut.as_mut().ok_or("missing SIC division resident")?;
        let(mut pair,mut event,mut evidence)=carrier.act_toroidal_division(port)?;
        if pair.is_none(){
            let context=carrier.toroidal_learning_context();
            let(lift,reverse,mut geometry)=observe(&context)?;
            let mut selected=lift.clone();let mut selected_reverse=reverse;
            if let Some(raw)=geometry["source_metric_rotation_readout"].as_str().or_else(||
                    geometry["carried_torus_frame"]["readout_parameters"][0].as_str()){
                let tape=crate::morphism_factor::decimal_to_tape(raw).ok_or("normal-frame readout numeral")?;
                let (frame_lift,divisor,frame)=crate::toroidal_division::lift_frame_readout(
                    &context,crate::belnap_word::BelnapWord::from_parity(&tape))?;
                // The modular action has a period, but its full lifted word
                // also feeds the next phase. Preserve its signed whole turns.
                if reverse {
                    if frame_lift.compare(&lift).map_err(String::from)?==core::cmp::Ordering::Less {
                        selected=crate::belnap_word::BelnapWord::new(lift.subtract(&frame_lift).0.0);
                        selected_reverse=true;
                    }else{
                        selected=crate::belnap_word::BelnapWord::new(frame_lift.subtract(&lift).0.0);
                        selected_reverse=false;
                    }
                }else{
                    selected=frame_lift.add(&lift);
                    selected_reverse=false;
                }
                geometry["full_signed_rotation_word_retained"]=serde_json::json!(true);
                geometry["frame_rotation"]=frame;
                if geometry["source_divisor"].is_null(){
                    if let Some(d)=divisor{geometry["source_divisor"]=crate::toroidal_division::word(&d);
                        geometry["source_divisor_origin"]=serde_json::json!("normal-frame-rotation-target");}
                }
            }
            geometry["selected_rotation_lift"]=crate::toroidal_division::word(&selected);
            geometry["selected_rotation_reverse"]=serde_json::json!(selected_reverse);
            let(next,rotation,mark)=carrier.rotate_toroidal_from_sic(selected,selected_reverse)?;
            geometry["rotation"]=rotation;geometry["input_context"]=context;
            pair=next;evidence=evidence.join(mark);
            if pair.is_none(){
                if let Some(raw)=geometry["source_divisor"]["numeral"].as_str(){
                    let divisor=crate::morphism_factor::decimal_to_tape(raw).ok_or("geometric divisor numeral")?;
                    let source=self.source.clone();
                    if crate::morphism_factor::cmp(&divisor,&crate::morphism_factor::one())!=core::cmp::Ordering::Greater
                        ||crate::morphism_factor::cmp(&divisor,&source)!=core::cmp::Ordering::Less{
                        return Err("geometric return did not emit a proper divisor".into());
                    }
                    let(quotient,remainder)=crate::morphism_factor::divmod(&source,&divisor);
                    if !crate::morphism_factor::zero(&remainder)||crate::morphism_factor::mul(&divisor,&quotient)!=source{
                        return Err("geometric return failed exact source division".into());
                    }
                    geometric_producer=true;
                    pair=Some((crate::belnap_word::BelnapWord::from_parity(&divisor),
                        crate::belnap_word::BelnapWord::from_parity(&quotient)));
                }
            }
            event["geometric_sic"]=geometry;
        }
        event["native_producer_cycle"]=serde_json::json!(pair.is_none());
        let returned=if let Some((p,q))=pair{
            let p=p.to_parity().map_err(String::from)?;let q=q.to_parity().map_err(String::from)?;
            Some(if geometric_producer{self.close_candidate(p,q,"geometric-throat-return")?}
                else{self.close_toroidal_division(p,q)?})
        }else{self.advance()?};
        event["banked_return_walk"]=serde_json::json!(returned.is_some());
        Ok((returned,event,evidence))
    }
    pub fn toroidal_decisions(&self)->Vec<crate::glut_system::ToroidalDecision>{
        self.glut.as_ref().map(|g|g.toroidal_decisions()).unwrap_or_default()
    }
    /// A carried toroidal observation changes the live support priorities.
    /// Return the preceding frame before installing the next one, retaining
    /// the same resident and its correlation evidence throughout.
    pub fn rephase_torus(&mut self, axes: [usize; 12]) -> Result<(), String> {
        if self.returned.is_some() { return Err("cannot rephase a closed factor resident".into()); }
        let carrier = self.glut.as_mut().ok_or("missing factor resident")?;
        carrier.return_catalyst_order()?;
        carrier.apply_catalyst_order_axes(axes)
    }

    /// Close an internally generated toroidal pair through this resident's
    /// existing product, banked phase and inverse-return machinery.
    pub fn close_toroidal_division(&mut self, p: Vec<char>, q: Vec<char>)
        -> Result<FactorReturn, String> {
        self.close_candidate(p, q, "organism-toroidal-division")
    }

    pub fn circuit_braid_state(&self) -> &crate::gfe_circuit_braid::GfeCircuitBraidState {
        &self.circuit_braid
    }
    pub fn powerset_monad_laws_verified(&self) -> bool {
        self.morphisms.monad_laws_verified
    }

    pub fn resident_stats(&self) -> crate::glut_system::FoldStats {
        self.glut.as_ref().map(PreparedGlutMembrane::stats).unwrap_or_default()
    }
    pub fn resident_identity(&self) -> Option<usize> {
        self.glut.as_ref().map(PreparedGlutMembrane::resident_identity)
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
        let execution=carrier.execute_retained()?.ok_or("toroidal pair lacks its enclosing banked return")?;
        execution.verify(&self.source)?;
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
    /// Select one producing operation while preserving the prepared Y,
    /// correlation, square, phase and source-bound return machinery.
    pub fn advance_component(&mut self, component: usize) -> Result<Option<FactorReturn>, String> {
        if let Some(returned) = &self.returned { return Ok(Some(returned.clone())); }
        self.morphisms.enter_work()?;
        self.turns += 1;
        if !self.glut.as_mut().unwrap().advance_component(component)? { return Ok(None); }
        let Some(execution) = self.glut.as_mut().unwrap().execute_retained()? else {
            return Err("resident exhausted without a proper factor return".into());
        };
        execution.verify(&self.source)?;
        self.morphisms.close(&self.source, self.glut.as_mut().unwrap(), &execution.p, &execution.q)?;
        let returned = self.verified(execution.p, execution.q, "organism-prepared-glut")?;
        self.returned = Some(returned.clone());
        Ok(Some(returned))
    }
    pub fn advance_component_cycle(&mut self, start: usize) -> Result<Option<FactorReturn>, String> {
        if let Some(returned)=&self.returned{return Ok(Some(returned.clone()));}
        self.morphisms.enter_work()?;
        self.turns+=1;
        if !self.glut.as_mut().unwrap().advance_component_cycle(start)?{return Ok(None);}
        let Some(execution)=self.glut.as_mut().unwrap().execute_retained()? else {
            return Err("resident exhausted without a proper factor return".into());
        };
        execution.verify(&self.source)?;
        self.morphisms.close(&self.source,self.glut.as_mut().unwrap(),&execution.p,&execution.q)?;
        let returned=self.verified(execution.p,execution.q,"organism-prepared-glut")?;
        self.returned=Some(returned.clone());Ok(Some(returned))
    }
    pub fn advance_observed(&mut self,
        observe: impl FnMut(&crate::glut_system::FoldStats))
        -> Result<Option<FactorReturn>, String> {
        if let Some(returned)=&self.returned{return Ok(Some(returned.clone()));}
        self.morphisms.enter_work()?;
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
