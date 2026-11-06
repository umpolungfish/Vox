//! Contact-driven torsion work on one retained peptide backbone.
//! Numerical contact restraints are an experimental geometry model, separate
//! from the Lean translation and word-transport laws. No reference coordinates
//! enter production. Every trial is rebuilt with the canonical NeRF bonds.
use alloc::{string::String, vec, vec::Vec};
use crate::{fold3d::{self, RamaEntry, BackboneAtom}, peptide_geometry as v, phase_word};
use crate::{belnap::B4, protein_lattice::{Interaction, PairReadout, Response}};

pub struct Contact { pub i: usize, pub j: usize }
pub struct Return {
    pub steps: Vec<RamaEntry>, pub backbone: Vec<BackboneAtom>,
    pub initial: f64, pub final_score: f64, pub accepted: usize,
    pub word_calls: usize, pub payload_moves: usize,
    pub interactions: Vec<PairReadout>,
}
fn local(bb:&[BackboneAtom], i:usize, step:&RamaEntry, aa:&str)->Vec<v::Point> {
    let b=&bb[i];let deg=core::f64::consts::PI/180.0;
    let c=if i==0 {b.c} else {v::nerf(bb[i-1].c,b.n,b.ca,1.525,111.0*deg,step.phi*deg)};
    let o=v::nerf(b.n,b.ca,c,1.231,120.8*deg,(step.psi+180.0)*deg);
    let frame=BackboneAtom{n:b.n,ca:b.ca,c,o};
    let mut points=vec![b.n,b.ca,c,o];
    points.extend(fold3d::build_sidechain(aa,&frame,step).into_iter().map(|a|a.1));points
}
struct Model {
    excluded:Vec<alloc::collections::BTreeSet<usize>>, radii:Vec<f64>,
    starts:Vec<usize>, degrees:Vec<usize>,
}
impl Model {
    fn new(chain:&[&str], contacts:&[Contact])->Self {
        let mut radii=Vec::new();let mut starts=Vec::new();let mut degrees=vec![0;chain.len()];
        for aa in chain {
            starts.push(radii.len());
            radii.extend(["N","CA","C","O"].iter().map(|a|crate::peptide_closure::radius(a)));
            radii.extend(fold3d::sidechain_topology(aa).iter().map(|a|crate::peptide_closure::radius(a.0.trim())));
        }
        for c in contacts {degrees[c.i]+=1;degrees[c.j]+=1;}
        Self{excluded:crate::peptide_closure::steric_exclusions(chain),radii,starts,degrees}
    }
}
fn score(chain:&[&str], steps:&[RamaEntry], bb:&[BackboneAtom], contacts:&[Contact], model:&Model)->(f64,Vec<v::Point>,Vec<v::Point>) {
    evaluate(chain,steps,bb,contacts,model,None)
}
fn evaluate(chain:&[&str], steps:&[RamaEntry], bb:&[BackboneAtom], contacts:&[Contact], model:&Model,
            mut retained:Option<&mut Vec<PairReadout>>)->(f64,Vec<v::Point>,Vec<v::Point>) {
    let points:Vec<_>=(0..chain.len()).flat_map(|i|local(bb,i,&steps[i],chain[i])).collect();
    let mut gradient=vec![(0.0,0.0,0.0);points.len()];let mut value=0.0;
    let mut interactions=alloc::collections::BTreeMap::<(usize,usize),Interaction>::new();
    for (source,c) in contacts.iter().enumerate() {
        let (left,right)=(model.starts[c.i]+1,model.starts[c.j]+1);
        let (i,j)=(left.min(right),left.max(right));
        let delta=v::sub(points[i],points[j]);
        let distance=fold3d::sqrt_f64(v::dot(delta,delta)).max(1e-9);
        let k=1.0/model.degrees[c.i].max(model.degrees[c.j]).max(1) as f64;
        let error=(distance-8.0).max(0.0);
        interactions.entry((i,j)).or_default().insert(source,B4::T,
            Response{energy:k*error*error,radial_derivative:2.0*k*error});
    }
    // Cell list avoids scanning all heavy-atom pairs on long proteins.
    let cell=|p:v::Point| {
        let floor=|x:f64|{let k=x as i64;if x<(k as f64){k-1}else{k}};
        (floor(p.0/4.0),floor(p.1/4.0),floor(p.2/4.0))
    };
    let mut cells=alloc::collections::BTreeMap::<(i64,i64,i64),Vec<usize>>::new();
    for (i,&point) in points.iter().enumerate() {cells.entry(cell(point)).or_default().push(i);}
    for (i,&point) in points.iter().enumerate() {
        let (x,y,z)=cell(point);
        for dx in -1..=1 {for dy in -1..=1 {for dz in -1..=1 {
            if let Some(indices)=cells.get(&(x+dx,y+dy,z+dz)) {for &j in indices {
                if j<=i || model.excluded[i].contains(&j) {continue;}
                let delta=v::sub(point,points[j]);
                let distance=fold3d::sqrt_f64(v::dot(delta,delta)).max(1e-9);
                let error=(model.radii[i]+model.radii[j]-crate::peptide_closure::CLASH_OVERLAP-distance).max(0.0);
                if error>0.0 {
                    interactions.entry((i,j)).or_default().insert(contacts.len(),B4::F,
                        Response{energy:10.0*error*error,radial_derivative:-20.0*error});
                }
            }}
        }}}
    }
    for ((i,j),interaction) in interactions {
        let distance=fold3d::sqrt_f64(v::dot(v::sub(points[i],points[j]),v::sub(points[i],points[j]))).max(1e-9);
        let total=interaction.evaluate();
        value+=total.energy;
        let g=v::scale(v::sub(points[i],points[j]),total.radial_derivative/distance);
        gradient[i]=v::add(gradient[i],g);gradient[j]=v::sub(gradient[j],g);
        if let Some(rows)=retained.as_deref_mut() {
            let (state,family)=interaction.family();
            rows.push(PairReadout{atoms:(i,j),distance,state,family,terms:interaction.terms});
        }
    }
    (value,gradient,points)
}
fn derivatives(chain:&[&str], steps:&[RamaEntry], bb:&[BackboneAtom], gradient:&[v::Point], points:&[v::Point], model:&Model)->Vec<(f64,f64)> {
    let mut out=vec![(0.0,0.0);bb.len()];
    let mut force=(0.0,0.0,0.0);let mut moment=(0.0,0.0,0.0);
    let epsilon=0.001;let radians=epsilon*core::f64::consts::PI/180.0;
    for i in (0..bb.len()).rev() {
        let b=&bb[i];
        let torque=|origin,axis|v::dot(v::unit(axis),v::sub(moment,v::cross(origin,force)));
        out[i]=(if i==0 {0.0}else{torque(b.n,v::sub(b.ca,b.n))},torque(b.ca,v::sub(b.c,b.ca)));
        let start=model.starts[i];let end=model.starts.get(i+1).copied().unwrap_or(points.len());
        // Local oxygen and sidechain coordinates also depend on retained phi/psi.
        // Differentiate those through the same geometry implementation.
        for phi in [true,false] {
            let mut high=steps[i];let mut low=steps[i];
            if phi {high.phi+=epsilon;low.phi-=epsilon;}else{high.psi+=epsilon;low.psi-=epsilon;}
            let high=local(bb,i,&high,chain[i]);let low=local(bb,i,&low,chain[i]);
            let local_gradient:f64=high.iter().zip(&low).zip(&gradient[start..end])
                .map(|((&a,&b),&g)|v::dot(g,v::scale(v::sub(a,b),0.5/radians))).sum();
            if phi {out[i].0+=local_gradient;}else{out[i].1+=local_gradient;}
        }
        for j in start..end {force=v::add(force,gradient[j]);moment=v::add(moment,v::cross(points[j],gradient[j]));}
    }
    out
}
fn wrap(x:f64)->f64 {let mut y=x;while y>180.0 {y-=360.0;}while y< -180.0 {y+=360.0;}y}

fn offset_frames(aa:&str,points:&[v::Point])->Vec<[v::Point;3]>{
    let topology=fold3d::sidechain_topology(aa);let mut frames=vec![[(1.0,0.0,0.0),(0.0,1.0,0.0),(0.0,0.0,1.0)];points.len()];
    let find=|name:&str|match name{"N"=>0,"CA"=>1,"C"=>2,"O"=>3,_=>4+topology.iter().position(|a|a.0.trim()==name).unwrap()};
    for(i,&(_,parent,_))in topology.iter().enumerate(){
        let grand=if parent=="CA"{"N"}else{topology.iter().find(|a|a.0.trim()==parent).unwrap().1};
        let p=points[find(parent)];let x=v::unit(v::sub(points[4+i],p));let raw=v::sub(points[find(grand)],p);
        let y=v::unit(v::sub(raw,v::scale(x,v::dot(raw,x))));frames[4+i]=[x,y,v::cross(x,y)];
    }frames
}
fn independent_local(bb:&[BackboneAtom],i:usize,step:&RamaEntry,chi:&RamaEntry,aa:&str,offsets:&[v::Point])->Vec<v::Point>{
    let b=&bb[i];let deg=core::f64::consts::PI/180.0;
    let c=if i==0{b.c}else{v::nerf(bb[i-1].c,b.n,b.ca,1.525,111.0*deg,step.phi*deg)};
    let o=v::nerf(b.n,b.ca,c,1.231,120.8*deg,(step.psi+180.0)*deg);
    let frame=BackboneAtom{n:b.n,ca:b.ca,c,o};let mut points=vec![b.n,b.ca,c,o];
    points.extend(fold3d::build_sidechain(aa,&frame,chi).into_iter().map(|a|a.1));
    if offsets.len()==points.len()+1{points.push(v::terminal_carboxyl_position(frame.ca,frame.c,frame.o));}
    let frames=offset_frames(aa,&points);
    for j in 4..points.len(){let d=offsets[j];points[j]=v::add(points[j],v::add(v::scale(frames[j][0],d.0),v::add(v::scale(frames[j][1],d.1),v::scale(frames[j][2],d.2))));}
    points
}
pub(crate) fn independent_pose_points(chain:&[&str],steps:&[RamaEntry],chi:&[RamaEntry],offsets:&[v::Point])->Vec<v::Point>{
    let bb=fold3d::build_backbone(steps);let mut start=0;let mut out=Vec::new();
    for i in 0..chain.len(){let end=if i+1==chain.len(){offsets.len()}else{start+4+fold3d::sidechain_topology(chain[i]).len()};out.extend(independent_local(&bb,i,&steps[i],&chi[i],chain[i],&offsets[start..end]));start=end;}out
}
pub(crate) fn independent_displacement_frames(chain:&[&str],steps:&[RamaEntry],chi:&[RamaEntry])->Vec<[v::Point;3]>{
    let bb=fold3d::build_backbone(steps);let mut out=Vec::new();
    for i in 0..chain.len(){let zeros=vec![(0.0,0.0,0.0);4+fold3d::sidechain_topology(chain[i]).len()];let points=independent_local(&bb,i,&steps[i],&chi[i],chain[i],&zeros);out.extend(offset_frames(chain[i],&points));}out
}
pub(crate) fn independent_coordinate_torques(chain:&[&str],steps:&[RamaEntry],chi:&[RamaEntry],offsets:&[v::Point],gradient:&[v::Point],points:&[v::Point])->Vec<[f64;4]>{
    let bb=fold3d::build_backbone(steps);let mut starts=Vec::new();let mut atom_count=0;
    for aa in chain{starts.push(atom_count);atom_count+=4+fold3d::sidechain_topology(aa).len();}
    let mut out=vec![[0.0;4];bb.len()];
    let(mut force,mut moment)=((0.0,0.0,0.0),(0.0,0.0,0.0));let epsilon=0.001;let radians=epsilon*core::f64::consts::PI/180.0;
    for i in (0..bb.len()).rev(){let b=&bb[i];let torque=|origin,axis|v::dot(v::unit(axis),v::sub(moment,v::cross(origin,force)));
        out[i][0]=if i==0{0.0}else{torque(b.n,v::sub(b.ca,b.n))};out[i][1]=torque(b.ca,v::sub(b.c,b.ca));
        let start=starts[i];let end=starts.get(i+1).copied().unwrap_or(points.len());
        for coordinate in 0..4{let(mut high,mut low,mut hc,mut lc)=(steps[i],steps[i],chi[i],chi[i]);
            match coordinate{0=>{high.phi+=epsilon;low.phi-=epsilon;},1=>{high.psi+=epsilon;low.psi-=epsilon;},2=>{hc.phi+=epsilon;lc.phi-=epsilon;},_=>{hc.psi+=epsilon;lc.psi-=epsilon;}}
            let high=independent_local(&bb,i,&high,&hc,chain[i],&offsets[start..end]);let low=independent_local(&bb,i,&low,&lc,chain[i],&offsets[start..end]);
            out[i][coordinate]+=high.iter().zip(&low).zip(&gradient[start..end]).map(|((&a,&b),&g)|v::dot(g,v::scale(v::sub(a,b),0.5/radians))).sum::<f64>();
        }
        for j in start..end{force=v::add(force,gradient[j]);moment=v::add(moment,v::cross(points[j],gradient[j]));}
    }out
}

pub fn run(chain:&[&str], initial:&[RamaEntry], contacts:&[Contact],
           words:&[Vec<&str>], iterations:usize)->Result<Return,String> {
    if chain.len()!=initial.len() || words.len()!=chain.len() || words.iter().any(|w|w.is_empty()) {
        return Err("folding state lost residue/form boundaries".into());
    }
    if contacts.iter().any(|c|c.i>=chain.len()||c.j>=chain.len()||c.i==c.j) {
        return Err("folding contact outside carried chain".into());
    }
    let mut steps=initial.to_vec();let mut bb=fold3d::build_backbone(&steps);
    let model=Model::new(chain,contacts);
    let initial_score=score(chain,&steps,&bb,contacts,&model).0;
    let mut current=initial_score;let mut accepted=0;let mut calls=0;let mut moves=0;
    for _ in 0..iterations {
        let (_,gradient,points)=score(chain,&steps,&bb,contacts,&model);
        let torque=derivatives(chain,&steps,&bb,&gradient,&points,&model);
        let maximum=torque.iter().fold(0.0f64,|m,(a,b)|m.max(a.abs()).max(b.abs()));
        if maximum<1e-9 {break;}
        let mut scale=8.0/maximum;
        let mut improved=false;
        for _ in 0..14 {
            let mut candidate=steps.clone();
            for i in 0..chain.len() {
                for word in &words[i] {
                    let mut moved=false;
                    phase_word::execute_with(word,&i,|mark,&index| {
                        // The first forward operation acts on this resident
                        // residue while its complete form word executes.
                        if mark=='≻' && !moved {
                            let fraction=scale / words[index].len() as f64;
                            candidate[index].phi=wrap(candidate[index].phi-fraction*torque[index].0);
                            candidate[index].psi=wrap(candidate[index].psi-fraction*torque[index].1);
                            moved=true;moves+=1;
                        }
                        Ok(())
                    })?;
                    calls+=1;
                }
            }
            let trial=fold3d::build_backbone(&candidate);let trial_score=score(chain,&candidate,&trial,contacts,&model).0;
            if trial_score+1e-10<current {
                steps=candidate;bb=trial;current=trial_score;accepted+=1;improved=true;break;
            }
            scale*=0.5;
        }
        if !improved {break;}
    }
    let mut interactions=Vec::new();
    evaluate(chain,&steps,&bb,contacts,&model,Some(&mut interactions));
    Ok(Return{steps,backbone:bb,initial:initial_score,final_score:current,accepted,
        word_calls:calls,payload_moves:moves,interactions})
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn open_carboxyl_lift_preserves_core_pose_and_uses_native_terminal_metric(){
        let chain=["Asn","Trp","Pro","Ser"];let steps=vec![RamaEntry{phi:-65.0,psi:-40.0,ss:"other",conf:0.0};chain.len()];
        let count=chain.iter().map(|aa|4+fold3d::sidechain_topology(aa).len()).sum::<usize>();let mut offsets=vec![(0.0,0.0,0.0);count];
        let old=independent_pose_points(&chain,&steps,&steps,&offsets);offsets.push((0.0,0.0,0.0));let new=independent_pose_points(&chain,&steps,&steps,&offsets);
        assert_eq!(new.len(),old.len()+1);for(a,b)in old.iter().zip(&new){assert_eq!([a.0.to_bits(),a.1.to_bits(),a.2.to_bits()],[b.0.to_bits(),b.1.to_bits(),b.2.to_bits()]);}
        let mut source=crate::peptide_closure::molecular_constraint_source(&chain);source["residue_identities"]=serde_json::json!(chain);let completed=crate::peptide_closure::complete_open_carboxyl(&source).unwrap();
        assert_eq!(completed["atoms"].as_array().unwrap().len(),new.len());assert_eq!(completed["atoms"].as_array().unwrap().last().unwrap()["name"],"OXT");
        let oxygen_count=completed["atoms"].as_array().unwrap().iter().filter(|a|a["name"]=="O"||a["name"]=="OXT").count();assert_eq!(oxygen_count,chain.len()+1);
        let start=count-4-fold3d::sidechain_topology("Ser").len();let c=new[start+2];let xt=*new.last().unwrap();let d=v::sub(xt,c);assert!((libm::sqrt(v::dot(d,d))-v::terminal_carboxyl_length()).abs()<1e-12);
        let axis=v::unit(v::sub(new[start+1],c));let first=v::unit(v::sub(new[start+3],c));let second=v::unit(d);assert!((v::dot(axis,first)-v::dot(axis,second)).abs()<1e-12);
        let exclusions=crate::peptide_closure::steric_exclusions_with_terminal(&chain,true);assert_eq!(exclusions.len(),new.len());assert!(exclusions[count].contains(&(start+2)));
    }
    #[test]
    fn independent_torsions_return_derivatives_with_retained_displacements(){
        for terminal in [false,true]{
        let chain=["Asn","Trp","Pro","Ser"];
        let steps=vec![RamaEntry{phi:-65.0,psi:-40.0,ss:"other",conf:0.0};chain.len()];
        let chi=vec![RamaEntry{phi:-33.25,psi:47.2,ss:"other",conf:0.0};chain.len()];
        let count=chain.iter().map(|aa|4+fold3d::sidechain_topology(aa).len()).sum::<usize>()+usize::from(terminal);
        let mut offsets=vec![(0.0,0.0,0.0);count];let mut start=0;
        for aa in chain{let end=start+4+fold3d::sidechain_topology(aa).len();for i in start+4..end{offsets[i]=(0.001*(i%3) as f64,-0.002,0.001);}start=end;}
        let points=independent_pose_points(&chain,&steps,&chi,&offsets);
        let gradient=(0..count).map(|i|(0.01*i as f64-0.2,0.4-0.02*i as f64,0.03*i as f64)).collect::<Vec<_>>();
        let analytic=independent_coordinate_torques(&chain,&steps,&chi,&offsets,&gradient,&points);
        let value=|steps:&[RamaEntry],chi:&[RamaEntry]|independent_pose_points(&chain,steps,chi,&offsets).iter().zip(&gradient).map(|(&p,&g)|v::dot(p,g)).sum::<f64>();
        for i in 0..chain.len(){for axis in 0..4{
            let mut high=steps.clone();let mut low=steps.clone();let mut hc=chi.clone();let mut lc=chi.clone();let delta=0.001;
            match axis{0=>{high[i].phi+=delta;low[i].phi-=delta;},1=>{high[i].psi+=delta;low[i].psi-=delta;},2=>{hc[i].phi+=delta;lc[i].phi-=delta;},_=>{hc[i].psi+=delta;lc[i].psi-=delta;}}
            let measured=(value(&high,&hc)-value(&low,&lc))/(2.0*delta)*180.0/core::f64::consts::PI;
            assert!((measured-analytic[i][axis]).abs()<1e-4*(1.0+measured.abs()),"residue {i} axis {axis}: {measured} vs {}",analytic[i][axis]);
        }}
        }
    }
    #[test]
    fn torsion_gradient_matches_coordinate_perturbation() {
        let mut steps=vec![RamaEntry{phi:-65.0,psi:-40.0,ss:"other",conf:0.0};8];
        let contacts=[Contact{i:0,j:7}];
        let chain=["Ala";8];let model=Model::new(&chain,&contacts);
        let bb=fold3d::build_backbone(&steps);
        let (_,gradient,points)=score(&chain,&steps,&bb,&contacts,&model);
        let analytic=derivatives(&chain,&steps,&bb,&gradient,&points,&model);
        let epsilon=0.0001;let degrees_per_radian=180.0/core::f64::consts::PI;
        for i in 1..7 {for phi in [true,false] {
            if phi {steps[i].phi+=epsilon;}else{steps[i].psi+=epsilon;}
            let high=score(&chain,&steps,&fold3d::build_backbone(&steps),&contacts,&model).0;
            if phi {steps[i].phi-=2.0*epsilon;}else{steps[i].psi-=2.0*epsilon;}
            let low=score(&chain,&steps,&fold3d::build_backbone(&steps),&contacts,&model).0;
            if phi {steps[i].phi+=epsilon;}else{steps[i].psi+=epsilon;}
            let measured=(high-low)/(2.0*epsilon)*degrees_per_radian;
            let expected=if phi {analytic[i].0}else{analytic[i].1};
            assert!((measured-expected).abs()<1e-4,"residue {i} phi={phi}: {measured} vs {expected}");
        }}
    }
    #[test]
    fn mixed_residue_gradient_matches_coordinate_perturbation() {
        let chain=["Met","Ala","Asp","Trp","Asn","Cys","Lys","Lys","Ile","Val","Pro","Lys","Tyr","Tyr","Gly","Arg"];
        let mut steps=vec![RamaEntry{phi:-65.0,psi:-40.0,ss:"other",conf:0.0};chain.len()];
        let contacts=[Contact{i:0,j:15}];let model=Model::new(&chain,&contacts);
        let bb=fold3d::build_backbone(&steps);
        let (_,gradient,points)=score(&chain,&steps,&bb,&contacts,&model);
        let analytic=derivatives(&chain,&steps,&bb,&gradient,&points,&model);
        let epsilon=0.001;let degrees_per_radian=180.0/core::f64::consts::PI;
        for i in 0..chain.len() {for phi in [true,false] {
            if phi {steps[i].phi+=epsilon;}else{steps[i].psi+=epsilon;}
            let high=score(&chain,&steps,&fold3d::build_backbone(&steps),&contacts,&model).0;
            if phi {steps[i].phi-=2.0*epsilon;}else{steps[i].psi-=2.0*epsilon;}
            let low=score(&chain,&steps,&fold3d::build_backbone(&steps),&contacts,&model).0;
            if phi {steps[i].phi+=epsilon;}else{steps[i].psi+=epsilon;}
            let measured=(high-low)/(2.0*epsilon)*degrees_per_radian;
            let expected=if phi {analytic[i].0}else{analytic[i].1};
            assert!((measured-expected).abs()<0.02,"residue {i} phi={phi}: {measured} vs {expected}");
        }}
    }
}
