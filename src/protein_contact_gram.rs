//! One shared proton in a mixed donor/acceptor contact Gram field.
//! This realizes a normalized quadratic mechanical model on the same state;
//! the caller retains chemical, steric and whole-chain admission obligations.
use alloc::{string::{String,ToString},vec::Vec};
use crate::sic::frame::{Operator,Sic};
use crate::protein_sic_exact as exact_sic;
#[derive(Clone)]
pub(crate) struct ContactGram {
    candidates:Vec<[usize;2]>,root:Option<[f64;2]>,events:Vec<serde_json::Value>,
}

#[cfg(test)]
mod tests{
    use super::*;
    #[test]
    fn exact_sic_retains_diagonal_zero_and_complex_contact_coherence(){
        let molecule=serde_json::json!({"atoms":[{"name":"N","residue":0},{"name":"O","residue":1}],"bonds":[],"residue_identities":["Asn","Asp"]});
        let mut contact=ContactGram::prepare(&molecule).unwrap();
        let mut matrix=Operator::zero(4);for i in 0..3{matrix.set(i,i,crate::sic::frame::Complex::new(1.0/3.0,0.0));}
        contact.update(0,&matrix,3.0).unwrap();assert_eq!(contact.spatial_gram(),Some([2.0,1.0,0.0]));
        let mut matrix=Operator::zero(4);matrix.set(0,0,crate::sic::frame::Complex::new(0.5,0.0));matrix.set(1,1,crate::sic::frame::Complex::new(0.5,0.0));
        matrix.set(0,1,crate::sic::frame::Complex::new(0.125,0.0625));matrix.set(1,0,crate::sic::frame::Complex::new(0.125,-0.0625));
        contact.update(1,&matrix,1.0).unwrap();assert_eq!(contact.spatial_gram(),Some([0.5,0.5,0.125]));
        assert_eq!(contact.events.last().unwrap()["local_density"][1],serde_json::json!([0.125,0.0625]));
        assert_eq!(contact.events.last().unwrap()["exact_dual_recovery"],true);
    }
}
impl ContactGram {
    pub fn prepare_from_field(&mut self,step:usize,mask:u8)->Result<(),String>{
        if !self.events.is_empty()||mask==0{return Ok(());}
        let scale=mask.count_ones() as f64;let mut input=Operator::zero(4);
        for i in 0..4{if mask&(1<<i)!=0{input.set(i,i,crate::sic::frame::Complex::new(1.0/scale,0.0));}}
        self.update(step,&input,scale)?;
        if let Some(event)=self.events.last_mut(){event["source_density_origin"]=serde_json::json!("native incoming working-field membership; not a banked observation");event["native_input_mask"]=serde_json::json!(mask);}
        Ok(())
    }
    pub fn spatial_gram(&self)->Option<[f64;3]>{
        let event=self.events.last()?;
        if event["local_mixed"].as_bool()!=Some(true){return None;}
        let density=&event["local_density"];
        Some([density[0][0].as_f64()?,density[3][0].as_f64()?,density[1][0].as_f64()?])
    }
    pub fn prepare(molecule:&serde_json::Value)->Result<Self,String>{
        let atoms=molecule["atoms"].as_array().ok_or("missing contact source atoms")?;
        let mut candidates=Vec::new();
        let mut adjacent=alloc::vec![Vec::new();atoms.len()];
        for b in molecule["bonds"].as_array().ok_or("missing contact source graph")?{
            let(a,b)=(b[0].as_u64().unwrap() as usize,b[1].as_u64().unwrap() as usize);adjacent[a].push(b);adjacent[b].push(a);
        }
        for (d,donor) in atoms.iter().enumerate(){
            let name=donor["name"].as_str().ok_or("missing contact atom name")?;
            if !name.starts_with('N'){continue;}
            let residue=donor["residue"].as_u64().unwrap() as usize;
            if name=="N"&&molecule["residue_identities"][residue]=="Pro"&&residue!=0{continue;}
            for(a,acceptor)in atoms.iter().enumerate(){
                if !acceptor["name"].as_str().unwrap_or("").starts_with('O')||d==a||adjacent[d].contains(&a)
                    ||adjacent[d].iter().any(|&b|adjacent[b].contains(&a)){continue;}
                candidates.push([d,a]);
            }
        }
        Ok(Self{candidates,root:None,events:Vec::new()})
    }
    pub fn update(&mut self,step:usize,input:&Operator,source_scale:f64)->Result<(),String>{
        if self.candidates.is_empty(){return Ok(());}
        if !source_scale.is_finite()||source_scale<=0.0{return Err("invalid retained contact source scale".into());}
        let frame=crate::sic::wh::ququart_frame();let masses=frame.split(input).map_err(|e|e.to_string())?;
        let returned=frame.fuse(&masses).map_err(|e|e.to_string())?;
        let residual=returned.distance(input).map_err(|e|e.to_string())?;let tolerance=4096.0*f64::EPSILON;
        if residual>tolerance{return Err("contact Gram state did not return through its general SIC frame".into());}
        let exact=exact_sic::return_operator(&input.values().iter().map(|z|(z.re,z.im)).collect::<Vec<_>>())?;
        // Runtime T,F,t,f = constructivity x donor/acceptor arm. Both
        // constructivity lanes contribute; the whole FOUR matrix stays carried.
        // SIC transports the normalized matrix together with its source mass.
        // Restore that mass before computing energy, force and curvature.
        let pd=source_scale*(exact[0].0+exact[10].0);
        let pa=source_scale*(exact[5].0+exact[15].0);
        let c=source_scale*(exact[1].0+exact[11].0);
        let im=source_scale*(exact[1].1+exact[11].1);
        let determinant=pd*pa-c*c-im*im;let stiffness=pd+pa+2.0*c;
        let before=self.root;
        let mixed=determinant>tolerance&&pd>0.0&&pa>0.0&&stiffness>tolerance;
        let mut gradient=None;let mut decrease=None;let mut retained_pair=None;
        if mixed{
            let root=[(pd+c)/stiffness,(pa+c)/stiffness];
            if let Some(previous)=before{
                gradient=Some([2.0*(stiffness*previous[0]-pd-c),2.0*(stiffness*previous[1]-pa-c)]);
                let delta=[previous[0]-root[0],previous[1]-root[1]];
                decrease=Some(core::array::from_fn::<_,4,_>(|i|stiffness*delta[i/2]*delta[i%2]));
            }
            self.root=Some(root);
            // Exact elimination keeps the coupling on the original D/A atoms.
            // It is not a new independent producer or a coordinate displacement.
            let coupling=(pd*pa-c*c)/stiffness;
            retained_pair=Some(serde_json::json!({"coefficient":coupling,
                "donor_force_coefficients":[-2.0*coupling,2.0*coupling],
                "acceptor_force_coefficients":[2.0*coupling,-2.0*coupling],
                "potential":"coefficient*|D-A|^2",
                "scope":"guarded contact alternative on existing source coordinates; covalent and steric constraints remain in the containing state"}));
        }
        self.events.push(serde_json::json!({"source_step":step,"source_basis":"T,F,t,f; both constructivity lanes retained",
            "input_four_matrix":input.values().iter().map(|z|[z.re,z.im]).collect::<Vec<_>>(),
            "sic_masses":masses.values.iter().map(|z|[z.re,z.im]).collect::<Vec<_>>(),
            "returned_four_matrix":exact.iter().map(|&(r,i)|[r,i]).collect::<Vec<_>>(),
            "floating_returned_four_matrix":returned.values().iter().map(|z|[z.re,z.im]).collect::<Vec<_>>(),
            "exact_dual_recovery":true,
            "exact_dual_scope":"canonical radical-field split and complete dual synthesis on the binary64 source matrix interpreted as exact dyadic entries; floating reconstruction and residual retained separately",
            "source_scale":source_scale,"local_density":[[pd,0.0],[c,im],[c,-im],[pa,0.0]],"local_mixed":mixed,
            "local_determinant":determinant,"quadratic_stiffness":stiffness,"hessian_diagonal":2.0*stiffness,
            "proton_affine_coefficients_before":before,"proton_affine_coefficients_after":self.root,
            "force_coefficients_before_update":gradient,"decrease_in_gram_potential_matrix":decrease,
            "retained_donor_acceptor_coupling":retained_pair,
            "sic_residual":residual,"tolerance":tolerance,
            "status":if mixed{"shared proton affine state transformed by the returned mixed Gram"}else{"contact Gram held with its unresolved source density"}}));
        Ok(())
    }
    pub fn report(&self)->serde_json::Value{
        serde_json::json!({"candidate_source_atom_pairs":self.candidates,"proton_affine_coefficients":self.root,"events":self.events,
            "generated_interface":serde_json::from_str::<serde_json::Value>(include_str!("../../measurements/protein_frame_transport_20261101/native_hydrogen_contact_interface.json")).expect("source-bound native contact interface"),
            "source_role":"guarded donor/acceptor alternatives over one global source conformation; one shared proton variable per donor",
            "candidate_admission":"source nitrogen/oxygen candidates with covalent exclusions; chemical, protonation and steric guards remain carried obligations",
            "model_gram_potential":"pd*|H-D|^2+pa*|H-A|^2+2*Re(c)*dot(H-D,H-A)",
            "executing_map":"H=((pd+Re(c))*D+(pa+Re(c))*A)/(pd+pa+2*Re(c))",
            "coordinate_binding":"D and A refer to existing source atom coordinates; each candidate assigns the same donor-proton variable under its contact guard",
            "realization_scope":"normalized mixed-contact Gram mechanical realization; whole-chain contact admission and folded coordinates remain open"})
    }
}
