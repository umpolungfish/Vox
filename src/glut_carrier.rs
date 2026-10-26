//! Carrier assembled from bound GLUT component ports over one resident register.
use super::{
    fold::{Port, Resident},
    GlutExecution,
};
use alloc::{rc::Rc, string::String, vec::Vec};
use core::cell::{Cell, RefCell};

#[derive(Clone)]
pub struct PreparedGlutMembrane {
    nested: Vec<char>,
    collapsed: Vec<char>,
    body: Vec<(char, Option<Port>)>,
    resident: Rc<RefCell<Resident>>,
    returned: Option<crate::phase_word::Readout<Rc<RefCell<Resident>>>>,
}
/// The canonical component wiring, shared by resident preparation and device
/// compilation. Building this program performs no factor production.
pub struct GlutComponentProgram {
    pub nested: Vec<char>,
    pub collapsed: Vec<char>,
    body: Vec<(char, Option<Port>)>,
}
impl GlutComponentProgram {
    pub fn compile() -> Result<Self, String> {
        // Skin -> GLUT -> component protocols. Consume each component's source
        // and terminal ports; retain its work and dyad inside the GLUT frame.
        let mut nested: Vec<char> = "⊢∈∈".chars().collect();
        let mut body = Vec::new();
        for port in Port::ordered() {
            let word: Vec<char> = port.word().chars().collect();
            if word.first() != Some(&'⊢') || word.last() != Some(&'⊣') {
                return Err("GLUT component lacks source and terminal ports".into());
            }
            let mut bound = false;
            for &mark in &word[1..word.len() - 1] {
                nested.push(mark);
                if matches!(mark, '∈' | '∋') {
                    continue;
                }
                if mark == '⊡' {
                    return Err("GLUT component fixes before its enclosing fuse".into());
                }
                let binding = if !bound && mark == port.trigger() {
                    bound = true;
                    Some(port)
                } else {
                    None
                };
                body.push((mark, binding));
            }
            if !bound {
                return Err("GLUT component has no bound working port".into());
            }
        }
        nested.extend("∋∋⊡⊣".chars());
        let (_, open, overfused) = crate::vox::pairing(&nested);
        if !open.is_empty() || !overfused.is_empty() {
            return Err("assembled GLUT protocol has unmatched boundaries".into());
        }
        let collapsed = crate::fixed_point_hypernest::collapse_for_execution(&nested);
        let mut expected: Vec<char> = "⊢∈".chars().collect();
        expected.extend(body.iter().map(|&(mark, _)| mark));
        expected.extend("∋⊡⊣".chars());
        if collapsed != expected {
            return Err("GLUT dissolution changed connected component work".into());
        }
        Ok(Self {
            nested,
            collapsed,
            body,
        })
    }
    /// Bindings are body addresses and original Port constructor indices.
    pub fn component_bindings(&self) -> Vec<(usize, u32)> {
        self.body.iter().enumerate().filter_map(|(address, &(_, port))|
            port.map(|port|(address,port as u32))).collect()
    }
}
impl PreparedGlutMembrane {
    pub fn prepare_dialectic(n: &[char]) -> Result<Self, String> {
        Self::prepare_dialectic_with_mask(n, None)
    }
    pub fn prepare_dialectic_gfe(
        n: &[char],
        mask: &crate::geometer_factor_extractor::SmallPrimeOrbitMask,
    ) -> Result<Self, String> {
        Self::prepare_dialectic_with_mask(n, Some(mask))
    }
    fn prepare_dialectic_with_mask(
        n: &[char],
        mask: Option<&crate::geometer_factor_extractor::SmallPrimeOrbitMask>,
    ) -> Result<Self, String> {
        let program = GlutComponentProgram::compile()?;
        let mut resident=Resident::with_dialectic(n, mask)?;
        resident.catalyst=Some(super::CatalystCycle::canonical()?);
        Ok(Self { nested: program.nested, collapsed: program.collapsed, body: program.body,
            resident: Rc::new(RefCell::new(resident)), returned: None })
    }
    pub fn prepare(n: &[char]) -> Result<Self, String> {
        let program=GlutComponentProgram::compile()?;
        let mut resident=Resident::new(n, |_| {});
        resident.catalyst=Some(super::CatalystCycle::canonical()?);
        Ok(Self {nested:program.nested,collapsed:program.collapsed,body:program.body,
            resident:Rc::new(RefCell::new(resident)),returned:None})
    }
    pub fn resident_identity(&self)->usize{Rc::as_ptr(&self.resident) as usize}
    pub fn prepare_toroidal_division(&mut self)->Result<(),String>{
        self.resident.borrow_mut().prepare_toroidal_division()
    }
    pub fn toroidal_snapshot(&self)->serde_json::Value{
        let mut value=self.resident.borrow().toroidal_snapshot();
        if let Some(object)=value.as_object_mut(){
            object.insert("resident_identity".into(),serde_json::json!(self.resident_identity()));
        }
        value
    }
    pub fn act_toroidal_division(&mut self,port:usize)
        ->Result<(Option<(crate::belnap_word::BelnapWord,crate::belnap_word::BelnapWord)>,serde_json::Value,crate::belnap::B4),String>{
        self.resident.borrow_mut().act_toroidal_division(port)
    }
    pub fn toroidal_learning_context(&self)->serde_json::Value{
        self.resident.borrow().toroidal_learning_context()
    }
    pub fn square_polynomial_actions(&self)->Vec<serde_json::Value>{
        self.resident.borrow().square_polynomial_actions()
    }
    pub fn reenter_toroidal_division(&mut self,before:u8,after:u8,kind:u8)->Result<serde_json::Value,String>{
        self.resident.borrow_mut().reenter_toroidal_division(before,after,kind)
    }
    pub fn bind_toroidal_support(&mut self,p:crate::belnap_word::BelnapWord,
        q:crate::belnap_word::BelnapWord,evidence:crate::belnap::B4)->Result<(),String>{
        self.resident.borrow_mut().bind_toroidal_support(p,q,evidence)
    }
    pub fn toroidal_decisions(&self)->Vec<super::ToroidalDecision>{
        self.resident.borrow().toroidal_decisions()
    }
    pub fn bind_square_torus(&mut self) -> Result<(), String> {
        self.resident.borrow_mut().bind_square_torus()
    }
    pub fn square_torus_observations(&self) -> Vec<super::SquareTorusObservation> {
        self.resident.borrow().square_torus_observations()
    }
    pub fn belnap_relation_conflicts(&self) -> Vec<super::BelnapRelationConflict> { self.resident.borrow().belnap_relation_conflicts() }
    pub fn belnap_conflicts(&self) -> Vec<super::BelnapConflict> { self.resident.borrow().belnap_conflicts() }
    pub fn stats(&self)->super::FoldStats{self.resident.borrow().stats.clone()}
    /// Bind an N-only divisor to the executing register and close its exact
    /// multiplication, carry checkpoints and factor-phase return in place.
    pub fn bind_source_candidate(&mut self, p: Vec<char>, q: Vec<char>,
        producer: &'static str) -> Result<(), String> {
        self.resident.borrow_mut().bind_source_candidate(p, q, producer)
    }
    pub fn catalyst_observations(&self)->Vec<super::CatalystObservation>{
        self.resident.borrow().catalyst.as_ref().map_or_else(Vec::new,|c|c.observations.clone())
    }
    pub fn return_catalyst_order(&mut self) -> Result<(), String> {
        self.resident.borrow_mut().return_catalyst_order()
    }
    /// Apply the forward MoDoT support permutation to the shared resident.
    /// Called by the GFE word's AFWD morphism after its two branch handles exist.
    pub fn apply_catalyst_order(&mut self) -> Result<(), String> {
        self.resident.borrow_mut().bind_catalyst_order()
    }
    /// Apply a source-bound axis permutation derived from the GFE torus braid.
    pub fn apply_catalyst_order_axes(&mut self, axes: [usize; 12]) -> Result<(), String> {
        self.resident.borrow_mut().bind_catalyst_order_axes(axes)
    }
    /// Apply the multi-strand horn-torus bit-twist braid permutation directly
    /// to the resident correlation graph.
    pub fn apply_horn_torus_twist(&mut self, strands: usize) -> Result<[usize; 12], String> {
        let braid = crate::pwoat_torus::BitBraid::torus_twist(strands);
        let axes = braid.axis_permutation();
        self.apply_catalyst_order_axes(axes)?;
        Ok(axes)
    }
    pub fn nested_word(&self) -> &[char] {
        &self.nested
    }
    pub fn membrane_word(&self) -> &[char] {
        &self.collapsed
    }
    /// Return through every component while retaining the same source register.
    /// This advances work only; readout still requires `execute` and its bank checks.
    pub fn advance(&mut self) -> bool {
        self.advance_observed(|_| {})
    }
    /// An organism-selected component acts on this same retained register.
    /// Verification and the factor-phase boundary consume any produced pair.
    pub fn advance_component(&mut self, index: usize) -> Result<bool, String> {
        let port = *Port::ORDER.get(index).ok_or("invalid resident component")?;
        let mut resident = self.resident.borrow_mut();
        resident.activate(port, |_| {});
        resident.activate(Port::Verify, |_| {});
        resident.activate(Port::Quantum, |_| {});
        Ok(resident.finished() && resident.quantum_phase_closed())
    }
    pub fn component_names() -> Vec<String> {
        Port::ORDER.iter().map(|port| alloc::format!("{port:?}")).collect()
    }
    /// Keep the complete resident producing cycle while permitting an
    /// organism-selected starting component. Every component still acts.
    pub fn advance_component_cycle(&mut self, start: usize) -> Result<bool, String> {
        if start>=Port::ORDER.len(){return Err("invalid resident cycle anchor".into());}
        let mut resident=self.resident.borrow_mut();
        for offset in 0..Port::ORDER.len(){resident.activate(Port::ORDER[(start+offset)%Port::ORDER.len()],|_|{});}
        resident.activate(Port::Verify,|_|{});
        resident.activate(Port::Quantum,|_|{});
        Ok(resident.finished() && resident.quantum_phase_closed())
    }
    /// Execute the existing first component on the same resident object.
    pub fn advance_frames(&mut self) -> Option<super::GlutFrameObservation> {
        let mut resident = self.resident.borrow_mut();
        resident.activate_frames(|_| {});
        resident.frame_observation()
    }
    /// Execute the Square component with the work assigned by Frames, retaining
    /// its phase state and any emitted pair in the same resident register.
    pub fn advance_square_observed(&mut self, observe: impl FnMut(&super::FoldStats)) {
        self.resident.borrow_mut().activate_square(observe);
    }
    pub fn advance_observed(&mut self, mut observe: impl FnMut(&super::FoldStats)) -> bool {
        let mut resident = self.resident.borrow_mut();
        for port in Port::ordered() {
            resident.activate(port, &mut observe);
        }
        resident.finished() && resident.quantum_phase_closed()
    }
    pub fn execute(mut self) -> Result<Option<GlutExecution>, String> {
        self.execute_retained()
    }
    /// Keep the complete resident object and its enclosing observations after
    /// readout. Re-reading a closed return does not run its components again.
    pub fn execute_retained(&mut self) -> Result<Option<GlutExecution>, String> {
        if self.returned.is_some() {
            return Ok(self.resident.borrow().execution());
        }
        let binding = Rc::new(Cell::new(None));
        let address = Rc::new(Cell::new(None));
        let walk = CarrierWalk {
            body: self.body.clone(),
            position: 0,
            prefix: 0,
            tail: None,
            binding: binding.clone(),
            address: address.clone(),
            resident: self.resident.clone(),
        };
        let returned = crate::phase_word::execute_addressed_iter_with(
            walk,
            &self.resident,
            |_, resident| {
                if let Some(port) = binding.get() {
                    resident.borrow_mut().activate(port, |_| {});
                }
                Ok(())
            },
            |lane| {
                address
                    .get()
                    .map(|slot| slot * 4 + lane.trailing_zeros() as usize)
            },
        )?;
        if returned.exposed != 0 || returned.surviving.is_empty() {
            return Err("GLUT return lost its enclosing bank".into());
        }
        if returned
            .surviving
            .iter()
            .any(|d| !Rc::ptr_eq(&self.resident, &d.observation))
        {
            return Err("GLUT return changed its resident register".into());
        }
        let resident = self.resident.borrow();
        if !resident.finished() {
            return Err("GLUT fused before its verification return".into());
        }
        if !resident.quantum_phase_closed() {
            return Err("GLUT fused before its quantum factor-phase return".into());
        }
        let execution = resident.execution();
        drop(resident);
        self.returned = Some(returned);
        Ok(execution)
    }
    /// Deposit IDs, lane supports and actual resident addresses at readout.
    pub fn retained_readout(&self) -> Option<crate::phase_word::Readout<usize>> {
        let read = self.returned.as_ref()?;
        Some(crate::phase_word::Readout {
            register: read.register,
            surviving: read.surviving.iter().map(|deposit| crate::phase_word::Deposit {
                id: deposit.id, lane: deposit.lane,
                observation: Rc::as_ptr(&deposit.observation) as usize,
            }).collect(),
            cleared: read.cleared, restored: read.restored, exposed: read.exposed,
            deposits: read.deposits, seeded: read.seeded, inert: read.inert,
            restore_supports: read.restore_supports.clone(),
        })
    }
}

impl crate::membrane_interface::Membrane for PreparedGlutMembrane {
    type Input = Vec<char>;
    type Readout = Option<GlutExecution>;
    type Continuation = Self;
    type Error = String;

    fn prepare(input: Self::Input) -> Result<Self, Self::Error> {
        Self::prepare(&input)
    }

    fn advance(
        &mut self,
    ) -> Result<crate::membrane_interface::MembraneProgress, Self::Error> {
        Ok(if PreparedGlutMembrane::advance(self) {
            crate::membrane_interface::MembraneProgress::ReadyToClose
        } else {
            crate::membrane_interface::MembraneProgress::Open
        })
    }

    fn close(
        mut self,
    ) -> Result<
        crate::membrane_interface::MembraneClosure<Self, Self::Continuation, Self::Readout>,
        Self::Error,
    > {
        match self.execute_retained()? {
            Some(readout) => Ok(crate::membrane_interface::MembraneClosure::Closed {
                membrane: Some(self),
                readout: Some(readout),
            }),
            None => Ok(crate::membrane_interface::MembraneClosure::Open(self)),
        }
    }
}

/// The return edge repeats component work within the same open skin. Register
/// state survives each pass. Only the verified return releases fuse/fix/readout.
struct CarrierWalk {
    body: Vec<(char, Option<Port>)>,
    position: usize,
    prefix: usize,
    tail: Option<usize>,
    binding: Rc<Cell<Option<Port>>>,
    address: Rc<Cell<Option<usize>>>,
    resident: Rc<RefCell<Resident>>,
}
impl Iterator for CarrierWalk {
    type Item = char;
    fn next(&mut self) -> Option<char> {
        self.binding.set(None);
        self.address.set(None);
        if self.prefix < 2 {
            let mark = ['⊢', '∈'][self.prefix];
            self.prefix += 1;
            return Some(mark);
        }
        if self.position == self.body.len() && self.tail.is_none() {
            if self.resident.borrow().finished() {
                self.tail = Some(0);
            } else {
                self.position = 0;
            }
        }
        if let Some(index) = self.tail {
            self.tail = Some(index + 1);
            return ['∋', '⊡', '⊣'].get(index).copied();
        }
        self.address.set(Some(self.position));
        let (mark, port) = self.body[self.position];
        self.position += 1;
        self.binding.set(port);
        Some(mark)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn toroidal_views_share_the_actual_resident_through_cloned_handles(){
        let source=crate::morphism_factor::decimal_to_tape("211060082272597355141159798675733025589").unwrap();
        let mut carrier=PreparedGlutMembrane::prepare(&source).unwrap();
        carrier.prepare_toroidal_division().unwrap();
        let alias=carrier.clone();let initial=alias.toroidal_snapshot();
        carrier.act_toroidal_division(0).unwrap();
        assert_eq!(carrier.toroidal_snapshot(),alias.toroidal_snapshot());
        assert_ne!(alias.toroidal_snapshot(),initial);
        assert_eq!(alias.toroidal_snapshot()["resident_identity"],serde_json::json!(carrier.resident_identity()));
        assert_eq!(alias.resident.borrow().source(),source);
        assert!(Rc::ptr_eq(&carrier.resident,&alias.resident));
    }
    #[test]
    fn retained_gpu_emission_keeps_its_register_and_observations_after_readout() {
        // Post-production control from the retained 200-bit CUDA emission in
        // membrane_gpu_wiring_20261013/gpu_receipt.json. No producer runs here.
        let tape = |s| crate::morphism_factor::decimal_to_tape(s).unwrap();
        let n = tape("752801617958890719975170228002419627780316816610845271542051");
        let p = tape("867641410929014337335571954827");
        let q = tape("867641410929014337335571955913");
        let mut prepared = PreparedGlutMembrane::prepare(&n).unwrap();
        prepared.prepare_toroidal_division().unwrap();
        let resident = prepared.resident.clone();
        resident.borrow_mut().accept_emitted_pair_for_return_test(p.clone(), q.clone());
        let first = prepared.execute_retained().unwrap().unwrap();
        let read = prepared.retained_readout().unwrap();
        assert_eq!((first.p.clone(), first.q.clone()), (p, q));
        assert_eq!(read.exposed, 0);
        assert!(!read.surviving.is_empty());
        assert_eq!(read.surviving.iter().fold(0, |mask, d| mask | d.lane), read.register);
        assert!(read.surviving.iter().all(|d| d.observation == Rc::as_ptr(&resident) as usize));
        assert_eq!(resident.borrow().source(), n);
        assert_eq!(resident.borrow().execution(), Some(first.clone()));
        assert!(prepared.toroidal_snapshot()["arms"].as_array().unwrap().iter().any(|arm|
            arm["signed_product_height"]["magnitude"]["numeral"]=="0"));
        assert_eq!(prepared.execute_retained().unwrap(), Some(first));
        let repeated = prepared.retained_readout().unwrap();
        assert_eq!(read.surviving, repeated.surviving);
        assert_eq!(read.restored, repeated.restored);
        assert_eq!(read.restore_supports, repeated.restore_supports);
        assert!(Rc::ptr_eq(&resident, &prepared.resident));
    }
    #[test]
    fn random_2048_source_is_resident_in_the_component_derived_carrier() {
        let n = crate::morphism_factor::parse_numeral(
            include_str!("../tests/fixtures/random_rsa_2048.imasm").trim(),
        )
        .unwrap();
        let prepared = PreparedGlutMembrane::prepare(&n).unwrap();
        let slots: Vec<_> = prepared.body.iter().filter_map(|&(_, p)| p).collect();
        assert_eq!(slots, Port::ORDER);
        assert_eq!(
            prepared
                .membrane_word()
                .iter()
                .filter(|&&m| m == '∈')
                .count(),
            1
        );
        assert_eq!(
            prepared
                .membrane_word()
                .iter()
                .filter(|&&m| m == '∋')
                .count(),
            1
        );
        assert_eq!(
            prepared
                .membrane_word()
                .iter()
                .filter(|&&m| m == '⊡')
                .count(),
            1
        );
        assert!(prepared.nested_word().len() > prepared.membrane_word().len());
        assert_eq!(prepared.resident.borrow().source(), n);
        let id = prepared.resident.clone();
        let binding = Rc::new(Cell::new(None));
        let mut walk = CarrierWalk {
            body: prepared.body,
            position: 0,
            prefix: 0,
            tail: None,
            binding: binding.clone(),
            address: Rc::new(Cell::new(None)),
            resident: prepared.resident,
        };
        let mut engaged = Vec::new();
        for _ in 0..2 {
            while let Some(mark) = walk.next() {
                assert!(
                    mark != '⊡' && mark != '∋',
                    "unfinished return must keep the skin open"
                );
                if let Some(port) = binding.get() {
                    engaged.push(port);
                }
                if walk.position == walk.body.len() {
                    break;
                }
            }
        }
        assert_eq!(engaged, [Port::ORDER, Port::ORDER].concat());
        assert!(Rc::ptr_eq(&id, &walk.resident));
        assert!(!walk.resident.borrow().finished());
    }
}
