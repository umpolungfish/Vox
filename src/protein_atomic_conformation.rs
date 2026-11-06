//! Whole-chain atom realization on the retained source conformation.
use alloc::{string::{String,ToString},vec::Vec};
use crate::{fold3d::{self,RamaEntry},peptide_geometry as v};
#[derive(Clone)]
struct SourceGeometry{radii:Vec<f64>,excluded:Vec<Vec<bool>>}
impl SourceGeometry{
    fn prepare(chain:&[String],terminal:bool)->Self{
        let chain=chain.iter().map(String::as_str).collect::<Vec<_>>();let mut radii=Vec::new();
        for aa in &chain{radii.extend(["N","CA","C","O"].iter().map(|a|crate::peptide_closure::radius(a)));radii.extend(fold3d::sidechain_topology(aa).iter().map(|a|crate::peptide_closure::radius(a.0.trim())));}
        if terminal{radii.push(crate::peptide_closure::radius("OXT"));}
        let mut excluded=alloc::vec![alloc::vec![false;radii.len()];radii.len()];
        for(i,row)in crate::peptide_closure::steric_exclusions_with_terminal(&chain,terminal).iter().enumerate(){for &j in row{excluded[i][j]=true;}}
        Self{radii,excluded}
    }
}
#[derive(Clone)]
pub(crate) struct AtomicConformation { chain:Vec<String>,steps:Vec<RamaEntry>,chi:Vec<RamaEntry>,offsets:Vec<v::Point>,fork_masks:Vec<u8>,reverse_masks:Vec<u8>,events:Vec<serde_json::Value>,contact_frames:Option<serde_json::Value>,active_gram:Option<[f64;3]>,active_surface:Option<crate::protein_atomic_surface::AtomicSurface>,geometry:SourceGeometry,conditional_contact:bool,terminal_carboxyl:bool }
#[cfg(feature="hosted")]
#[derive(Clone)]
struct PreparedTransition{steps:Vec<RamaEntry>,chi:Vec<RamaEntry>,offsets:Vec<v::Point>,contact_frames:Option<serde_json::Value>,surface:Option<crate::protein_atomic_surface::AtomicSurface>,event:serde_json::Value}
#[cfg(feature="hosted")]
fn prepared_transitions()->&'static std::sync::Mutex<alloc::collections::BTreeMap<String,PreparedTransition>>{
    static STORE:std::sync::OnceLock<std::sync::Mutex<alloc::collections::BTreeMap<String,PreparedTransition>>>=std::sync::OnceLock::new();
    STORE.get_or_init(Default::default)
}
#[cfg(test)]
mod conflict_tests {
    use super::*;
    #[test]
    fn carried_geometric_conflict_keeps_energy_and_dynamics() {
        let mut molecule=crate::peptide_closure::molecular_constraint_source(&["Ala","Ser"]);
        molecule["residue_identities"]=serde_json::json!(["Ala","Ser"]);
        molecule["source_codons"]=serde_json::json!(["GCU","UCU"]);
        molecule["native_rotamer_fork_masks"]=serde_json::json!([1,2,12]);
        molecule["native_rotamer_reverse_masks"]=serde_json::json!([2,1,12]);
        let mut state=AtomicConformation::prepare(&molecule).unwrap();
        // A finite retained deformation creates explicit source bond conflicts.
        state.offsets[4]=(10.0,0.0,0.0);
        let target=state.positions();
        let points=state.evaluate(&state.steps,&state.chi,&target,false).2;
        assert!(!state.assess_pose(&points,&state.steps).issues.is_empty());
        assert!(state.admitted(&state.steps,&state.chi,&target));
        // A controlled compression reservoir must contribute energy/gradient
        // without becoming a separate coordinate-admission veto.
        state.geometry.radii.fill(20.0);
        let (energy,gradient,_)=state.evaluate(&state.steps,&state.chi,&target,false);
        assert!(energy>0.0&&energy.is_finite());
        assert!(gradient.iter().any(|p|v::dot(*p,*p)>0.0));
        assert!(state.admitted(&state.steps,&state.chi,&target));
        state.offsets[4].0=f64::NAN;
        assert!(!state.admitted(&state.steps,&state.chi,&target));
    }
}
impl AtomicConformation {
    pub fn prepare(molecule:&serde_json::Value)->Result<Self,String>{
        let chain=molecule["residue_identities"].as_array().ok_or("atomic sequence missing")?.iter().map(|v|v.as_str().unwrap().to_string()).collect::<Vec<_>>();
        let path=molecule["source_codons"].as_array().ok_or("atomic source codons missing")?.iter().map(|c|{
            crate::genetic::nuc_b4(c.as_str().unwrap().chars().next().unwrap()).ok_or("atomic source codon is not bound")
        }).collect::<Result<Vec<_>,_>>()?;
        let fork_masks=molecule["native_rotamer_fork_masks"].as_array().ok_or("native conformational fork missing")?.iter().map(|v|v.as_u64().unwrap() as u8).collect::<Vec<_>>();
        if fork_masks.len()<2{return Err("native conformational fork has insufficient arms".into());}
        let reverse_masks=molecule["native_rotamer_reverse_masks"].as_array().ok_or("native reverse fork missing")?.iter().map(|v|v.as_u64().unwrap() as u8).collect::<Vec<_>>();
        let offsets=alloc::vec![(0.0,0.0,0.0);molecule["atoms"].as_array().unwrap().len()];
        let terminal_carboxyl=molecule["terminal_carboxyl_boundary"].is_object();
        let geometry=SourceGeometry::prepare(&chain,terminal_carboxyl);
        if offsets.len()!=geometry.radii.len(){return Err("molecular atom source differs from native pose layout".into());}
        let steps=fold3d::rama_steps(&path);Ok(Self{chain,chi:steps.clone(),steps,offsets,fork_masks,reverse_masks,events:Vec::new(),contact_frames:None,active_gram:None,active_surface:None,geometry,conditional_contact:false,terminal_carboxyl})
    }
    #[cfg(feature="hosted")]
    pub(crate) fn resume_saved(value:&serde_json::Value,molecule:&serde_json::Value,budget:usize)->Result<Self,String>{
        if !["native-atomic-contact-frame-cycle/v1","native-atomic-contact-frame-cycle/v2","native-atomic-contact-frame-cycle/v3","native-atomic-contact-frame-cycle/v4","native-atomic-contact-frame-cycle/v5"].iter().any(|k|value["kernel"]==*k){return Err("saved atomic kernel is incompatible".into());}
        let mut state=Self::prepare(molecule)?;
        state.conditional_contact=value["source_contact_operand"]=="conditional candidate proton";
        let operand_before=state.conditional_contact;
        if let Ok(operand)=std::env::var("VOX_CONTACT_OPERAND"){
            state.conditional_contact=match operand.as_str(){"conditional"=>true,"shared"=>false,_=>return Err("VOX_CONTACT_OPERAND must be shared or conditional".into())};
        }

        let torsions:Vec<[f64;2]>=serde_json::from_value(value["torsions"].clone()).map_err(|e|e.to_string())?;
        let chi:Vec<[f64;2]>=serde_json::from_value(value["sidechain_torsions"].clone()).map_err(|e|e.to_string())?;
        if torsions.len()!=state.steps.len()||chi.len()!=state.chi.len(){return Err("saved torsion inventory differs".into());}
        for(i,row)in torsions.iter().enumerate(){state.steps[i].phi=row[0];state.steps[i].psi=row[1];state.chi[i].phi=chi[i][0];state.chi[i].psi=chi[i][1];}
        state.offsets=serde_json::from_value(value["retained_internal_displacements"].clone()).map_err(|e|e.to_string())?;
        let boundary_lift=state.terminal_carboxyl&&value["terminal_carboxyl_present"]!=true&&state.offsets.len()+1==state.geometry.radii.len();
        if boundary_lift{state.offsets.push((0.0,0.0,0.0));}
        if state.offsets.len()!=state.geometry.radii.len(){return Err("saved displacement inventory differs".into());}
        state.events=value["events"].as_array().ok_or("saved atomic history missing")?.clone();
        state.contact_frames=if value["directional_contact_frames"].is_null(){None}else{Some(value["directional_contact_frames"].clone())};
        if let Some(contact)=&state.contact_frames{
            let binding:serde_json::Value=serde_json::from_str(include_str!("../../measurements/protein_frame_transport_20261101/native_hydrogen_contact_interface.json")).map_err(|e|e.to_string())?;
            if contact["source_artifact_sha256"]!=binding["artifact_sha256"]{return Err("saved contact source binding differs".into());}
        }
        state.active_gram=serde_json::from_value(value["active_source_gram"].clone()).map_err(|e|e.to_string())?;
        state.active_surface=if value["source_cycle_surface_state"].is_null(){None}else{Some(crate::protein_atomic_surface::AtomicSurface::from_saved(&value["source_cycle_surface_state"],state.offsets.len())?)};
        let chain=state.chain.iter().map(String::as_str).collect::<Vec<_>>();
        let points=crate::protein_relax::independent_pose_points(&chain,&state.steps,&state.chi,&state.offsets);
        let atoms=value["atoms"].as_array().ok_or("saved atoms missing")?;
        if !(points.len()==atoms.len()||boundary_lift&&points.len()==atoms.len()+1)||points.iter().zip(atoms).any(|(p,a)|[p.0,p.1,p.2].iter().enumerate().any(|(k,x)|a["position"][k].as_f64().map(f64::to_bits)!=Some(x.to_bits()))){return Err("saved atomic coordinates do not reproduce bitwise".into());}
        let target=state.positions();
        let source_step=state.events.last().and_then(|e|e["source_step"].as_u64()).unwrap_or(0) as usize;
        let boundary_preparation=if state.terminal_carboxyl&&!state.admitted(&state.steps,&state.chi,&target){
            let mut steps=state.steps.clone();let mut chi=state.chi.clone();let report=state.restore_native_clearance(&mut steps,&mut chi,&target);state.steps=steps;state.chi=chi;Some(report)
        }else{None};
        if boundary_lift{
            let chain=state.chain.iter().map(String::as_str).collect::<Vec<_>>();let points=crate::protein_relax::independent_pose_points(&chain,&state.steps,&state.chi,&state.offsets);
            state.contact_frames=Some(crate::protein_contact_frames::execute(molecule,&points,source_step,state.active_gram,state.contact_frames.as_ref())?);
        }
        let refinement=state.refine_after_admission(&target,source_step,budget);
        if state.contact_frames.is_some(){
            let chain=state.chain.iter().map(String::as_str).collect::<Vec<_>>();
            let points=crate::protein_relax::independent_pose_points(&chain,&state.steps,&state.chi,&state.offsets);
            state.contact_frames=Some(crate::protein_contact_frames::execute(molecule,&points,source_step,state.active_gram,state.contact_frames.as_ref())?);
        }
        state.events.push(serde_json::json!({"source_step":source_step,"operation":"resume retained atomic refinement","source_operator_advanced":false,"source_boundary_lift":boundary_lift,"old_coordinates_verified_before_boundary_preparation":true,"boundary_preparation":boundary_preparation,"contact_operand_before":if operand_before{"conditional candidate proton"}else{"shared donor proton"},"contact_operand_after":if state.conditional_contact{"conditional candidate proton"}else{"shared donor proton"},"contact_operand_scope":"controlled realization of the source quadratic; parent ancestry and other operand state retained","coupled_refinement":refinement}));
        Ok(state)
    }
    pub fn positions(&self)->Vec<[f64;3]>{fold3d::build_backbone(&self.steps).iter().map(|b|[b.ca.0,b.ca.1,b.ca.2]).collect()}
    fn evaluate(&self,steps:&[RamaEntry],chi:&[RamaEntry],_target:&[[f64;3]],fit:bool)->(f64,Vec<v::Point>,Vec<v::Point>){
        let chain=self.chain.iter().map(String::as_str).collect::<Vec<_>>();let points=crate::protein_relax::independent_pose_points(&chain,steps,chi,&self.offsets);
        let radii=&self.geometry.radii;let excluded=&self.geometry.excluded;
        let mut gradient=alloc::vec![(0.0,0.0,0.0);points.len()];let mut energy=0.0;
        for i in 0..points.len(){for j in i+1..points.len(){if excluded[i][j]{continue;}
            let d=v::sub(points[i],points[j]);let square=v::dot(d,d);let minimum=radii[i]+radii[j]-crate::peptide_closure::CLASH_OVERLAP;
            if square<minimum*minimum{let r=libm::sqrt(square).max(1e-9);let error=(minimum-r).max(0.0);energy+=10.0*error*error;let g=v::scale(d,-20.0*error/r);gradient[i]=v::add(gradient[i],g);gradient[j]=v::sub(gradient[j],g);}
        }}
        if fit{if let (Some(source),Some(gram))=(self.contact_frames.as_ref(),self.active_gram){
            match if self.conditional_contact{crate::protein_contact_frames::conditional_chain_response(source,&points,gram,false)}else{crate::protein_contact_frames::chain_response(source,&points,gram,false)}{
                Ok((value,force,_))=>{energy+=value;for i in 0..gradient.len(){gradient[i]=v::add(gradient[i],force[i]);}}
                Err(_)=>return (f64::INFINITY,gradient,points),
            }
        }}
        if fit{if let Some(surface)=&self.active_surface{
            match surface.potential_gradient(&points){Ok((value,g))=>{energy+=value;for i in 0..gradient.len(){gradient[i]=v::add(gradient[i],g[i]);}},Err(_)=>return(f64::INFINITY,gradient,points)}
        }}
        (energy,gradient,points)
    }
    pub fn act(&mut self,source_step:usize,mark:char,target:&[[f64;3]],molecule:&serde_json::Value,gram:Option<[f64;3]>,cycles:&[serde_json::Value])->Result<(),String>{
        #[cfg(feature="hosted")]
        let refinement_budget=std::env::var("VOX_POST_ADMISSION_STEPS").ok().and_then(|s|s.parse::<usize>().ok()).unwrap_or(512);
        #[cfg(not(feature="hosted"))]
        let refinement_budget=512usize;
        // Reuse only an equal complete numerical action input. Histories and
        // enclosing object/bank identities remain on their original carriers.
        #[cfg(feature="hosted")]
        let transition_key={
            let rama=|rows:&[RamaEntry]|rows.iter().map(|r|serde_json::json!([r.phi.to_bits(),r.psi.to_bits(),r.ss,r.conf.to_bits()])).collect::<Vec<_>>();
            serde_json::to_string(&serde_json::json!({"step":source_step,"refinement_budget":refinement_budget,"conditional_contact":self.conditional_contact,"chain":self.chain,
                "steps":rama(&self.steps),"chi":rama(&self.chi),"offsets":self.offsets.iter().map(|p|[p.0.to_bits(),p.1.to_bits(),p.2.to_bits()]).collect::<Vec<_>>(),
                "previous_contact_frames":self.contact_frames,"target":target,"molecule":molecule,
                "gram":gram.map(|g|g.map(f64::to_bits)),"prior_surface":self.active_surface.as_ref().map(|s|s.report()),"source_cycles":cycles,"mark":mark.to_string()})).map_err(|e|e.to_string())?
        };
        #[cfg(feature="hosted")]
        if let Some(prepared)=prepared_transitions().lock().unwrap().get(&transition_key).cloned(){
            self.steps=prepared.steps;self.chi=prepared.chi;self.offsets=prepared.offsets;self.contact_frames=prepared.contact_frames;
            self.active_gram=gram;self.active_surface=prepared.surface;let mut event=prepared.event;
            event["prepared_numeric_transition_reused"]=serde_json::json!(true);
            self.events.push(event);
            if std::env::var_os("VOX_FOLD_TRACE").is_some(){std::eprintln!("protein source {source_step}: applied equal-input prepared numeric transition; enclosing history retained");}
            return Ok(());
        }
        if mark=='⋈'{
            let chain=self.chain.iter().map(String::as_str).collect::<Vec<_>>();let points=crate::protein_relax::independent_pose_points(&chain,&self.steps,&self.chi,&self.offsets);
            if self.active_surface.is_none(){self.active_surface=crate::protein_atomic_surface::AtomicSurface::prepare(molecule,cycles,&points)?;}
            if let Some(surface)=&mut self.active_surface{surface.advance(source_step,&points)?;}
        }
        let prepare_contact=self.contact_frames.is_none()&&gram.is_some();
        if prepare_contact{
            let chain=self.chain.iter().map(String::as_str).collect::<Vec<_>>();
            let points=crate::protein_relax::independent_pose_points(&chain,&self.steps,&self.chi,&self.offsets);
            self.contact_frames=Some(crate::protein_contact_frames::execute(molecule,&points,source_step,gram,None)?);
        }
        self.active_gram=gram.filter(|_|self.contact_frames.is_some());
        let before=self.evaluate(&self.steps,&self.chi,target,true).0;let steric_before=self.evaluate(&self.steps,&self.chi,target,false).0;let mut accepted=0;let mut offered=Vec::new();
        // The reverse branch acts on the whole downstream backbone, retaining
        // its covalent frames. This crosses torsion barriers that a local
        // gradient cannot cross while every intermediate trial is held.
        for residue in 0..self.chain.len().saturating_sub(1){
            let base=self.steps.clone();let fit=self.admitted(&base,&self.chi,target);
            let mut best=self.evaluate(&base,&self.chi,target,fit).0;let mut best_pose=base.clone();let mut selected=0;let mut rows=Vec::new();
            for arm in 0..2*self.fork_masks.len(){
                let slot=arm%self.fork_masks.len();let reversed=arm>=self.fork_masks.len();
                let angle=360.0*slot as f64/self.fork_masks.len() as f64+if reversed{180.0}else{0.0};
                let mask=if reversed{self.reverse_masks[slot]}else{self.fork_masks[slot]};
                let mut trial=base.clone();trial[residue].psi+=angle;
                let energy=self.evaluate(&trial,&self.chi,target,fit).0;let admitted=mask!=0&&(!fit||self.admitted(&trial,&self.chi,target));
                rows.push(serde_json::json!({"arm":arm,"native_membership":mask,"rotation_degrees":angle,"potential":energy,"admitted":admitted}));
                if admitted&&energy<best-1e-14{best=energy;best_pose=trial;selected=arm;}
            }
            offered.push(serde_json::json!({"residue":residue,"coordinate":"backbone psi","source_backbone_torsions":base.iter().map(|s|[s.phi,s.psi]).collect::<Vec<_>>(),"source_sidechain_torsions":self.chi.iter().map(|s|[s.phi,s.psi]).collect::<Vec<_>>(),"arms":rows,"selected_arm":selected}));
            self.steps=best_pose;
        }
        for residue in 0..self.chain.len().saturating_sub(2){
            let base=self.steps.clone();let fit=self.admitted(&base,&self.chi,target);
            let before=self.evaluate(&base,&self.chi,target,fit).0;let mut trial=base.clone();
            trial[residue].psi+=180.0;trial[residue+1].psi+=180.0;
            let energy=self.evaluate(&trial,&self.chi,target,fit).0;let admitted=!fit||self.admitted(&trial,&self.chi,target);
            let selected=admitted&&energy<before-1e-14;
            offered.push(serde_json::json!({"residues":[residue,residue+1],"coordinate":"correlated backbone reversals","source_backbone_torsions":base.iter().map(|s|[s.phi,s.psi]).collect::<Vec<_>>(),"source_sidechain_torsions":self.chi.iter().map(|s|[s.phi,s.psi]).collect::<Vec<_>>(),"source_native_reverse_masks":self.reverse_masks,"potential_before":before,"potential_after":energy,"admitted":admitted,"selected":selected}));
            if selected{self.steps=trial;}
        }
        for residue in 0..self.chain.len(){
            let topology=fold3d::sidechain_topology(&self.chain[residue]);
            if topology.len()<=1{continue;}
            let base=self.chi.clone();let fit=self.admitted(&self.steps,&base,target);
            let mut best=self.evaluate(&self.steps,&base,target,fit).0;let mut selected=[0,0];let mut rows=Vec::new();let mut best_pose=base.clone();
            let proline=self.chain[residue]=="Pro";
            let arms=if proline{2}else{2*self.fork_masks.len()};let second=if !proline&&topology.iter().any(|a|a.1!="CA"&&a.1!="CB"){arms}else{1};
            for first in 0..arms{for second_arm in 0..second{
                let phase=|arm:usize|if proline{180.0*arm as f64}else{360.0*(arm%self.fork_masks.len()) as f64/self.fork_masks.len() as f64+if arm>=self.fork_masks.len(){180.0}else{0.0}};
                let mask=|arm:usize|if proline{if arm==0{self.fork_masks[0]}else{self.reverse_masks[0]}}else if arm>=self.fork_masks.len(){self.reverse_masks[arm%self.fork_masks.len()]}else{self.fork_masks[arm]};
                let angles=[phase(first),phase(second_arm)];let masks=[mask(first),mask(second_arm)];
                let mut trial=base.clone();trial[residue].phi+=angles[0];trial[residue].psi+=angles[1];
                let score=self.evaluate(&self.steps,&trial,target,fit).0;
                let admissible=(!fit||self.admitted(&self.steps,&trial,target))&&masks.iter().all(|m|*m!=0);
                rows.push(serde_json::json!({"arms":[first,second_arm],"native_memberships":masks,"rotation_degrees":angles,"potential":score,"admitted":admissible}));
                if admissible&&score<best-1e-14{best=score;selected=[first,second_arm];best_pose=trial;}
            }}
            offered.push(serde_json::json!({"residue":residue,"source_backbone_torsions":self.steps.iter().map(|s|[s.phi,s.psi]).collect::<Vec<_>>(),"source_sidechain_torsions":base.iter().map(|s|[s.phi,s.psi]).collect::<Vec<_>>(),"arms":rows,"selected_arms":selected,"generator":"held geometric angle assignment to native fork masks; AREV representation unproved"}));
            self.chi=best_pose;
        }
        let chain=self.chain.iter().map(String::as_str).collect::<Vec<_>>();
        for _ in 0..512{
            let fit=self.admitted(&self.steps,&self.chi,target);
            if fit{break;}
            let(current,gradient,points)=self.evaluate(&self.steps,&self.chi,target,fit);
            let torque=self.source_torques(&gradient,&points,fit);
            let maximum=torque.iter().flatten().fold(0.0_f64,|m,p|m.max(p.abs()));if maximum<1e-10{break;}
            let mut scale=8.0/maximum;let mut improved=false;
            for _ in 0..14{
                let mut trial=self.steps.clone();let mut chi=self.chi.clone();for(i,g)in torque.iter().enumerate(){trial[i].phi-=scale*g[0];trial[i].psi-=scale*g[1];chi[i].phi-=scale*g[2];chi[i].psi-=scale*g[3];}
                let energy=self.evaluate(&trial,&chi,target,fit).0;
                let admissible=!fit||self.admitted(&trial,&chi,target);
                if admissible&&energy<current-1e-14{self.steps=trial;self.chi=chi;accepted+=1;improved=true;break;}scale*=0.5;
            }if !improved{break;}
        }
        let frames=crate::protein_relax::independent_displacement_frames(&chain,&self.steps,&self.chi);
        let mut side_indices=Vec::new();let mut offset=0;for aa in &chain{let end=offset+4+fold3d::sidechain_topology(aa).len();side_indices.extend(offset+4..end);offset=end;}
        let mut accepted_displacements=0;
        for _ in 0..256{
            let(current,gradient,_)=self.evaluate(&self.steps,&self.chi,target,false);if current<1e-16{break;}
            let maximum=side_indices.iter().map(|&i|libm::sqrt(v::dot(gradient[i],gradient[i]))).fold(0.0_f64,f64::max);if maximum<1e-10{break;}
            let previous=self.offsets.clone();let mut scale=0.02/maximum;let mut improved=false;
            for _ in 0..20{
                self.offsets=previous.clone();
                for &i in &side_indices{let g=gradient[i];let old=previous[i];self.offsets[i]=(old.0-scale*v::dot(g,frames[i][0]),old.1-scale*v::dot(g,frames[i][1]),old.2-scale*v::dot(g,frames[i][2]));}
                let(energy,_,points)=self.evaluate(&self.steps,&self.chi,target,false);
                let evidence=self.assess_points(&points);
                if energy<current-1e-16&&!evidence.issues.iter().any(|i|matches!(i.kind,"inventory"|"finite")){
                    accepted_displacements+=1;improved=true;break;
                }scale*=0.5;
            }
            if !improved{self.offsets=previous;break;}
        }
        drop(chain);
        let coupled_refinement=self.refine_after_admission(target,source_step,refinement_budget);
        let chain=self.chain.iter().map(String::as_str).collect::<Vec<_>>();
        let current=self.evaluate(&self.steps,&self.chi,target,true).0;let steric_after=self.evaluate(&self.steps,&self.chi,target,false).0;
        let points=crate::protein_relax::independent_pose_points(&chain,&self.steps,&self.chi,&self.offsets);
        self.contact_frames=Some(crate::protein_contact_frames::execute(molecule,&points,source_step,gram,self.contact_frames.as_ref())?);
        self.events.push(serde_json::json!({"source_step":source_step,"potential_before":before,"potential_after":current,"steric_before":steric_before,"steric_after":steric_after,"accepted_torsion_updates":accepted,
            "native_rotamer_offers":offered,
            "accepted_native_bound_displacements":accepted_displacements,
            "source_contact_gram_drives_chain":self.active_gram.is_some(),
            "source_cycle_surface_drives_chain":self.active_surface.is_some(),
            "post_admission_coupled_refinement":coupled_refinement,
            "contact_component_prepared_before_chain_action":prepare_contact,
            "admission":"finite source inventory with all geometric conflicts retained as energy/evidence",
            "operation":"native whole-chain internal-coordinate realization driven by the retained spatial metric and full atom sterics"}));
        #[cfg(feature="hosted")]
        prepared_transitions().lock().unwrap().insert(transition_key,PreparedTransition{steps:self.steps.clone(),chi:self.chi.clone(),offsets:self.offsets.clone(),contact_frames:self.contact_frames.clone(),surface:self.active_surface.clone(),event:self.events.last().unwrap().clone()});
        Ok(())
    }
    fn assess_points(&self,points:&[v::Point])->crate::peptide_closure::Evidence{
        self.assess_pose(points,&self.steps)
    }
    fn refine_after_admission(&mut self,target:&[[f64;3]],source_step:usize,budget:usize)->serde_json::Value{
        #[cfg(feature="hosted")]
        let trace=std::env::var_os("VOX_FOLD_TRACE").is_some();
        #[cfg(not(feature="hosted"))]
        let _=source_step;
        if (self.active_gram.is_none()&&self.active_surface.is_none())||!self.admitted(&self.steps,&self.chi,target){return serde_json::json!({"executed":false,"reason":"source field absent or coordinate inventory undefined"});}
        let chain=self.chain.iter().map(String::as_str).collect::<Vec<_>>();
        let points_before=crate::protein_relax::independent_pose_points(&chain,&self.steps,&self.chi,&self.offsets);
        let before=self.evaluate(&self.steps,&self.chi,target,true).0;let mut records=Vec::new();let mut accepted=0;
        for iteration in 0..budget{
            let(energy,gradient,points)=self.evaluate(&self.steps,&self.chi,target,true);
            let torque=self.source_torques(&gradient,&points,true);
            let maximum=torque.iter().flatten().fold(0.0_f64,|m,g|m.max(g.abs()));if maximum<1e-10{break;}
            let mut improved=false;let mut trials=Vec::new();let mut blocked=Vec::new();let mut scale=8.0/maximum;
            for _ in 0..14{
                let mut steps=self.steps.clone();let mut chi=self.chi.clone();
                for(i,g)in torque.iter().enumerate(){steps[i].phi-=scale*g[0];steps[i].psi-=scale*g[1];chi[i].phi-=scale*g[2];chi[i].psi-=scale*g[3];}
                let(after,_,trial_points)=self.evaluate(&steps,&chi,target,true);let admitted=self.admitted(&steps,&chi,target);let selected=admitted&&after<energy-1e-14;
                if !admitted{for i in 0..trial_points.len(){for j in i+1..trial_points.len(){
                    if self.geometry.excluded[i][j]{continue;}
                    let(_,overlap)=crate::peptide_closure::nonbonded_measure(trial_points[i],trial_points[j],self.geometry.radii[i]+self.geometry.radii[j]);
                    if overlap>crate::peptide_closure::CLASH_OVERLAP&&!blocked.contains(&(i,j)){blocked.push((i,j));}
                }}}

                let rejected=if !admitted&&trials.is_empty(){
                    let evidence=self.assess_pose(&trial_points,&steps);
                    Some(serde_json::json!({"issues":evidence.issues.iter().map(|i|serde_json::json!({"kind":i.kind,"detail":i.detail})).collect::<Vec<_>>(),"issue_count":evidence.issues.len(),"scope":"actual native predicates on the first offered full source direction"}))
                }else{None};
                trials.push(serde_json::json!({"direction":"full source force","scale":scale,"potential":after,"admitted":admitted,"selected":selected,"native_rejection":rejected}));
                if selected{self.steps=steps;self.chi=chi;improved=true;break;}scale*=0.5;
            }
            // The failed native inequalities supply their own tangent normals.
            // Keep the full source force and its rejected constraints in the return.
            if !improved&&!blocked.is_empty(){
                let chain=self.chain.iter().map(String::as_str).collect::<Vec<_>>();
                let rows=blocked.iter().map(|&(i,j)|{
                    let mut gradient=alloc::vec![(0.0,0.0,0.0);points.len()];
                    let(distance,_)=crate::peptide_closure::nonbonded_measure(points[i],points[j],self.geometry.radii[i]+self.geometry.radii[j]);let d=v::scale(v::sub(points[i],points[j]),1.0/distance);gradient[i]=d;gradient[j]=v::scale(d,-1.0);
                    crate::protein_relax::independent_coordinate_torques(&chain,&self.steps,&self.chi,&self.offsets,&gradient,&points)
                }).collect::<Vec<_>>();
                let radians=core::f64::consts::PI/180.0;
                let clearance=blocked.iter().map(|&(i,j)|{let(_,overlap)=crate::peptide_closure::nonbonded_measure(points[i],points[j],self.geometry.radii[i]+self.geometry.radii[j]);crate::peptide_closure::CLASH_OVERLAP-overlap}).collect::<Vec<_>>();
                let mut projected=torque.iter().map(|row|row.map(|g|-8.0*radians*g/maximum)).collect::<Vec<_>>();let mut contractions=0;
                for _ in 0..torque.len()*4{
                    let mut changed=false;
                    for(row,&slack)in rows.iter().zip(&clearance){
                        let dot=row.iter().flatten().zip(projected.iter().flatten()).map(|(a,b)|a*b).sum::<f64>();
                        let norm=row.iter().flatten().map(|g|g*g).sum::<f64>();
                        if dot< -slack&&norm>0.0{for(r,p)in row.iter().zip(&mut projected){for k in 0..4{p[k]+=r[k]*(-slack-dot)/norm;}}changed=true;contractions+=1;}
                    }
                    if !changed{break;}
                }
                let maximum=projected.iter().flatten().fold(0.0_f64,|m,g|m.max(g.abs()));
                trials.push(serde_json::json!({"direction":"native affine clearance construction","source_atom_pairs":blocked,"distance_jacobian":rows,"source_distance_clearance":clearance,"projected_step_radians":projected,"halfspace_contractions":contractions,"admitted":false,"selected":false,
                    "definition":"current native distance clearance plus its Jacobian applied to the proposed angular step must remain nonnegative"}));
                if maximum>0.0{let mut scale=1.0;
                    for _ in 0..14{
                        let mut steps=self.steps.clone();let mut chi=self.chi.clone();
                        for(i,g)in projected.iter().enumerate(){steps[i].phi+=scale*g[0]/radians;steps[i].psi+=scale*g[1]/radians;chi[i].phi+=scale*g[2]/radians;chi[i].psi+=scale*g[3]/radians;}
                        let mut admitted=self.admitted(&steps,&chi,target);
                        let repair=if !admitted{let result=self.restore_native_clearance(&mut steps,&mut chi,target);admitted=self.admitted(&steps,&chi,target);Some(result)}else{None};
                        let after=self.evaluate(&steps,&chi,target,true).0;let selected=admitted&&after<energy-1e-14;
                        trials.push(serde_json::json!({"direction":if repair.is_some(){"native clearance retraction"}else{"native affine clearance"},"scale":scale,"potential":after,"admitted":admitted,"selected":selected,"native_retraction":repair}));
                        if selected{self.steps=steps;self.chi=chi;improved=true;break;}scale*=0.5;
                    }
                }
            }
            // A rejected joint direction retains its individual generating
            // coordinates as executable continuations of the same source.
            if !improved{
                let mut coordinates=torque.iter().enumerate().flat_map(|(residue,row)|row.iter().enumerate().filter(|(_,g)|**g!=0.0).map(move|(axis,&g)|(residue,axis,g))).collect::<Vec<_>>();
                coordinates.sort_by(|a,b|b.2.abs().total_cmp(&a.2.abs()).then(a.0.cmp(&b.0)).then(a.1.cmp(&b.1)));
                for(residue,axis,g)in coordinates{
                let mut delta=-8.0*g/g.abs();
                for _ in 0..14{
                    let mut steps=self.steps.clone();let mut chi=self.chi.clone();
                    match axis{0=>steps[residue].phi+=delta,1=>steps[residue].psi+=delta,2=>chi[residue].phi+=delta,_=>chi[residue].psi+=delta}
                    let after=self.evaluate(&steps,&chi,target,true).0;let admitted=self.admitted(&steps,&chi,target);let selected=admitted&&after<energy-1e-14;
                    trials.push(serde_json::json!({"direction":"source coordinate","residue":residue,"axis":axis,"delta_degrees":delta,"potential":after,"admitted":admitted,"selected":selected}));
                    if selected{self.steps=steps;self.chi=chi;improved=true;break;}delta*=0.5;
                }
                if improved{break;}
            }}
            records.push(serde_json::json!({"iteration":iteration,"potential_before":energy,"source_torque":torque,"trials":trials}));
            #[cfg(feature="hosted")]
            if trace&&iteration%32==0{std::eprintln!("protein source {source_step}: coupled iteration {iteration}, accepted {accepted}, source potential {energy:.9e}");}
            if !improved{break;}accepted+=1;
        }
        serde_json::json!({"executed":true,"accepted_updates":accepted,"potential_before":before,
            "iteration_budget":budget,"held_continuation":{"source_step":source_step,"operation":"refine_after_admission","iterations_observed":records.len(),"whole_atomic_state_retained":true},
            "potential_after":self.evaluate(&self.steps,&self.chi,target,true).0,
            "atom_positions_before":points_before.iter().map(|p|[p.0,p.1,p.2]).collect::<Vec<_>>(),
            "retained_source_direction_offers":records,
            "operation":"after native geometry admission, apply the same contact and cycle source forces to the retained chain; rejected joint directions retain coordinate continuations"})
    }
    fn restore_representable_clearance(&self,steps:&mut [RamaEntry],chi:&mut [RamaEntry],target:&[[f64;3]])->serde_json::Value{
        let chain=self.chain.iter().map(String::as_str).collect::<Vec<_>>();
        let violations=|points:&[v::Point]|{let mut failed=Vec::new();for i in 0..points.len(){for j in i+1..points.len(){
            if self.geometry.excluded[i][j]{continue;}
            let(_,overlap)=crate::peptide_closure::nonbonded_measure(points[i],points[j],self.geometry.radii[i]+self.geometry.radii[j]);let error=overlap-crate::peptide_closure::CLASH_OVERLAP;
            if error>0.0{failed.push((i,j,error));}
        }}failed};
        let points=crate::protein_relax::independent_pose_points(&chain,steps,chi,&self.offsets);let mut held=violations(&points);let mut attempts=Vec::new();
        for stride in 1..=4*steps.len(){for residue in 0..steps.len(){for axis in 0..4{for positive in [false,true]{
            if held.is_empty(){return serde_json::json!({"admitted":self.admitted(steps,chi,target),"attempts":attempts,"scope":"adjacent representable source angles; no native bound changed"});}
            let mut trial=steps.to_vec();let mut tc=chi.to_vec();let value=match axis{0=>&mut trial[residue].phi,1=>&mut trial[residue].psi,2=>&mut tc[residue].phi,_=>&mut tc[residue].psi};let before=value.to_bits();
            for _ in 0..stride{*value=if positive{value.next_up()}else{value.next_down()};}let after=value.to_bits();
            let points=crate::protein_relax::independent_pose_points(&chain,&trial,&tc,&self.offsets);let failed=violations(&points);
            let preserved=failed.iter().all(|&(i,j,error)|held.iter().any(|&(a,b,old)|a==i&&b==j&&error<=old));
            let improved=failed.len()<held.len()||failed.iter().any(|&(i,j,error)|held.iter().any(|&(a,b,old)|a==i&&b==j&&error<old));let selected=preserved&&improved;
            attempts.push(serde_json::json!({"residue":residue,"axis":axis,"positive":positive,"representable_stride":stride,"source_bits":before,"offered_bits":after,"source_failed_pairs":failed,"selected":selected}));
            if selected{steps.clone_from_slice(&trial);chi.clone_from_slice(&tc);held=failed;}
        }}}}
        serde_json::json!({"admitted":self.admitted(steps,chi,target),"attempts":attempts,"remaining_failed_pairs":held,"scope":"bounded representable-neighbor continuation; native predicates unchanged"})
    }
    fn restore_native_clearance(&self,steps:&mut [RamaEntry],chi:&mut [RamaEntry],target:&[[f64;3]])->serde_json::Value{
        let chain=self.chain.iter().map(String::as_str).collect::<Vec<_>>();let radians=core::f64::consts::PI/180.0;let mut history=Vec::new();
        for iteration in 0..4*steps.len(){
            let points=self.evaluate(steps,chi,target,false).2;let mut failed=Vec::new();
            for i in 0..points.len(){for j in i+1..points.len(){
                if self.geometry.excluded[i][j]{continue;}
                let(distance,overlap)=crate::peptide_closure::nonbonded_measure(points[i],points[j],self.geometry.radii[i]+self.geometry.radii[j]);
                let violation=overlap-crate::peptide_closure::CLASH_OVERLAP;
                if violation>0.0{failed.push((i,j,violation,distance));}
            }}
            if failed.is_empty(){let evidence=self.assess_pose(&points,steps);return serde_json::json!({"admitted":evidence.closed(),"corrections":history,"remaining_native_issues":evidence.issues.iter().map(|i|serde_json::json!({"kind":i.kind,"detail":i.detail})).collect::<Vec<_>>(),"scope":"same native inequalities; no bound relaxed"});}
            let mut correction=alloc::vec![[0.0;4];steps.len()];
            for &(i,j,violation,distance)in &failed{
                if distance==0.0{return serde_json::json!({"admitted":false,"corrections":history,"held":"coincident source atoms have no defined distance normal"});}
                let mut gradient=alloc::vec![(0.0,0.0,0.0);points.len()];let d=v::scale(v::sub(points[i],points[j]),1.0/distance);gradient[i]=d;gradient[j]=v::scale(d,-1.0);
                let row=crate::protein_relax::independent_coordinate_torques(&chain,steps,chi,&self.offsets,&gradient,&points);
                let dot=row.iter().flatten().zip(correction.iter().flatten()).map(|(a,b)|a*b).sum::<f64>();let norm=row.iter().flatten().map(|x|x*x).sum::<f64>();
                if dot<violation&&norm>0.0{for(r,c)in row.iter().zip(&mut correction){for k in 0..4{c[k]+=r[k]*(violation-dot)/norm;}}}
            }
            let maximum=correction.iter().flatten().fold(0.0_f64,|m,x|m.max(x.abs()));
            if !maximum.is_finite()||maximum==0.0{return serde_json::json!({"admitted":false,"corrections":history,"held":"native coordinate normals supplied no finite nonzero correction"});}
            let scale=(8.0*radians/maximum).min(1.0);
            history.push(serde_json::json!({"iteration":iteration,"source_atom_pairs":failed.iter().map(|r|[r.0,r.1]).collect::<Vec<_>>(),"native_overlap_violations":failed.iter().map(|r|r.2).collect::<Vec<_>>(),"correction_radians":correction,"scale":scale}));
            let mut rounded_advances=0;
            let mut advance=|value:&mut f64,delta:f64|{let next=*value+delta;
                if next==*value&&delta!=0.0{*value=if delta>0.0{value.next_up()}else{value.next_down()};rounded_advances+=1;}else{*value=next;}
            };
            for(i,c)in correction.iter().enumerate(){advance(&mut steps[i].phi,scale*c[0]/radians);advance(&mut steps[i].psi,scale*c[1]/radians);advance(&mut chi[i].phi,scale*c[2]/radians);advance(&mut chi[i].psi,scale*c[3]/radians);}
            history.last_mut().unwrap()["representable_coordinate_advances"]=serde_json::json!(rounded_advances);
        }
        #[cfg(feature="hosted")]
        let explore_neighbors=std::env::var_os("VOX_REPRESENTABLE_SEARCH").is_some();
        #[cfg(not(feature="hosted"))]
        let explore_neighbors=false;
        let representable=if explore_neighbors{Some(self.restore_representable_clearance(steps,chi,target))}else{None};
        serde_json::json!({"admitted":self.admitted(steps,chi,target),"corrections":history,"representable_search":representable,"iteration_budget":4*steps.len(),"scope":"bounded native-clearance continuation; original physical bounds retained"})
    }
    fn admitted(&self,steps:&[RamaEntry],chi:&[RamaEntry],target:&[[f64;3]])->bool{
        let points=self.evaluate(steps,chi,target,false).2;
        // Conflicts are carried evidence and stored interaction energy. They do
        // not erase a source conformation or prevent its dynamics from acting.
        // Only an undefined coordinate or broken source inventory cannot be
        // evaluated by this coordinate instrument.
        let evidence=self.assess_pose(&points,steps);
        !evidence.issues.iter().any(|i|matches!(i.kind,"inventory"|"finite"))
    }
    fn source_torques(&self,gradient:&[v::Point],points:&[v::Point],fit:bool)->Vec<[f64;4]>{
        let chain=self.chain.iter().map(String::as_str).collect::<Vec<_>>();let mut base=gradient.to_vec();
        let surface=if fit{self.active_surface.as_ref().and_then(|s|s.potential_gradient(points).ok().map(|(_,g)|(s.residues(),g)))}else{None};
        if let Some((_,g))=&surface{for i in 0..base.len(){base[i]=v::sub(base[i],g[i]);}}
        let mut torque=crate::protein_relax::independent_coordinate_torques(&chain,&self.steps,&self.chi,&self.offsets,&base,points);
        if let Some((ids,g))=surface{
            let mut partial=crate::protein_relax::independent_coordinate_torques(&chain,&self.steps,&self.chi,&self.offsets,&g,points);let lo=ids[0].min(ids[1]);let hi=ids[0].max(ids[1]);
            for i in 0..self.chain.len(){if i<=lo||i>hi{partial[i][0]=0.0;}if i<lo||i>=hi{partial[i][1]=0.0;}if i!=ids[0]&&i!=ids[1]{partial[i][2]=0.0;partial[i][3]=0.0;}for axis in 0..4{torque[i][axis]+=partial[i][axis];}}
        }
        torque
    }
    fn assess_pose(&self,points:&[v::Point],steps:&[RamaEntry])->crate::peptide_closure::Evidence{
        let chain=self.chain.iter().map(String::as_str).collect::<Vec<_>>();let mut backbone=Vec::new();let mut sides=Vec::new();let mut start=0;
        for aa in &chain{backbone.push([points[start],points[start+1],points[start+2],points[start+3]]);let topology=fold3d::sidechain_topology(aa);
            let mut side=topology.iter().enumerate().map(|(j,a)|(a.0,points[start+4+j])).collect::<Vec<_>>();start+=4+topology.len();
            if self.terminal_carboxyl&&sides.len()+1==chain.len(){side.push(("OXT",points[start]));start+=1;}sides.push(side);}
        crate::peptide_closure::assess(&chain,&backbone,&sides,&steps.iter().map(|s|(s.phi,s.psi)).collect::<Vec<_>>())
    }
    pub fn report(&self)->serde_json::Value{
        let chain=self.chain.iter().map(String::as_str).collect::<Vec<_>>();let points=crate::protein_relax::independent_pose_points(&chain,&self.steps,&self.chi,&self.offsets);
        let angles=self.steps.iter().map(|s|(s.phi,s.psi)).collect::<Vec<_>>();let evidence=self.assess_points(&points);
        let mut atoms=Vec::new();let mut start=0;for(i,aa)in chain.iter().enumerate(){let names=["N","CA","C","O"].into_iter().chain(fold3d::sidechain_topology(aa).iter().map(|a|a.0.trim())).collect::<Vec<_>>();for(j,name)in names.iter().enumerate(){let p=points[start+j];atoms.push(serde_json::json!({"residue":i,"name":name,"position":[p.0,p.1,p.2]}));}start+=names.len();}
        if self.terminal_carboxyl{let p=points[start];atoms.push(serde_json::json!({"residue":chain.len()-1,"name":"OXT","position":[p.0,p.1,p.2]}));}
        let mut bridge_conflicts=Vec::new();
        let n=self.fork_masks.len();
        for i in 0..n { for j in 0..n {
            if self.fork_masks[i]==self.reverse_masks[j] {
                let direct=360.0*i as f64/n as f64;
                let reversed=(360.0*j as f64/n as f64+180.0)%360.0;
                if (direct-reversed).abs()>1e-9 {
                    bridge_conflicts.push(serde_json::json!({"native_register":self.fork_masks[i],"direct_angle_degrees":direct,"reverse_angle_degrees":reversed}));
                }
            }
        }}
        let surface_readout=self.active_surface.as_ref().map(|s|match s.potential_gradient(&points){Ok((energy,g))=>serde_json::json!({"potential":energy,"operation":"squared full complex fixed-point residual of the same retained H rho H-adjoint return; no source scale discarded","heavy_atom_gradient":g.iter().map(|p|[p.0,p.1,p.2]).collect::<Vec<_>>()}),Err(reason)=>serde_json::json!({"held":reason})});
        let contact_readout=match (self.contact_frames.as_ref(),self.active_gram){
            (Some(source),Some(gram))=>{
                let response=if self.conditional_contact{crate::protein_contact_frames::conditional_chain_response(source,&points,gram,true)}else{crate::protein_contact_frames::chain_response(source,&points,gram,true)};
                Some(match response{Ok((_,_,report))=>report,Err(reason)=>serde_json::json!({"held":reason})})
            },_=>None,
        };
        serde_json::json!({"kernel":"native-atomic-contact-frame-cycle/v5","terminal_carboxyl_present":self.terminal_carboxyl,"atoms":atoms,"torsions":angles,"sidechain_torsions":self.chi.iter().map(|s|[s.phi,s.psi]).collect::<Vec<_>>(),"events":self.events,"geometry_closed":evidence.closed(),
            "conflicts_retained_as_state":true,"conflict_admission":"finite source inventory; geometric conflicts do not veto the conformation or its dynamics",
            "stored_conflict_energy":{"overlap_potential":self.evaluate(&self.steps,&self.chi,&self.positions(),false).0,"law":"10*sum(max(0,radius_i+radius_j-0.4-distance_ij)^2) over existing nonexcluded source pairs","scope":"existing model energy, retained with its derivative; other conflict observations remain carried without inventing scalar conversion weights"},
            "lattice_justification":{"status":"held","physical_closure_justified":false,
                "angle_assignment_factors_through_native_register":bridge_conflicts.is_empty(),
                "equal_register_unequal_angle_witnesses":bridge_conflicts,
                "required_bridge":"a carried-state lattice action preserving native identities, composition and source observations",
                "unsupported_domain_choices":["fork-slot angles","AREV antipodal angle action","proline pucker selection","steric and metric objective priorities","internal displacement admission","rigid contact classification"]},
            "retained_internal_displacements":self.offsets,
            "directional_contact_frames":self.contact_frames,
            "source_cycle_surface_state":self.active_surface.as_ref().map(|s|s.report()),
            "source_contact_operand_readout":contact_readout,"source_cycle_surface_readout":surface_readout,"active_source_gram":self.active_gram,"source_contact_operand":if self.conditional_contact{"conditional candidate proton"}else{"shared donor proton"},
            "native_geometry_evidence":evidence.report(true),"issue_count":evidence.issues.len(),
            "retained_rigid_contact_evidence":evidence.retained_rigid_contacts,
            "issues":evidence.issues.iter().map(|i|serde_json::json!({"kind":i.kind,"detail":i.detail})).collect::<Vec<_>>()})
    }
}
