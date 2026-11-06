//! Source-addressed donor/acceptor frame composition, retaining all path choices.
use alloc::{string::{String,ToString},vec::Vec};
use crate::peptide_geometry::{self as v,Point};

pub(crate) fn execute(molecule:&serde_json::Value,points:&[Point],step:usize,gram:Option<[f64;3]>,previous:Option<&serde_json::Value>)->Result<serde_json::Value,String>{
    let generated:serde_json::Value=serde_json::from_str(include_str!("../../measurements/protein_frame_transport_20261101/native_hydrogen_contact_interface.json")).map_err(|_|String::from("generated contact source binding invalid"))?;
    let word=generated["source_word"].as_str().ok_or("generated contact word missing")?;
    let mut machine=crate::phase_word::Continuation::<usize>::default();
    let mut working=Some(0usize);let mut output=None;let mut trace=Vec::new();
    for (node,mark) in word.chars().enumerate(){
        let before=working;let mut executed=false;
        machine.advance_with(&mark.to_string(),&working.unwrap_or(0),|_,_|{executed=true;Ok(())})?;
        if executed&&mark=='⋈'{output=Some(compose(molecule,points,step)?);working=Some(1);}
        let mut mechanical=false;
        if executed&&mark=='≻'{if let (Some(state),Some(coefficients))=(output.as_mut(),gram){
            transport_gram(state,points,coefficients,previous)?;mechanical=true;
        }}
        if executed&&mark=='≺'{working=None;}
        if executed&&mark=='∋'&&machine.readout().surviving.iter().any(|d|d.observation==1){working=Some(1);}
        trace.push(serde_json::json!({"source_node":node,"opcode":mark.to_string(),"executed":executed,
            "working_object_before":before,"working_object_after":working,"frame_gram_action_executed":mechanical,
            "open_frame_object_refs":machine.frames().iter().map(|f|f.iter().map(|d|d.observation).collect::<Vec<_>>()).collect::<Vec<_>>()}));
    }
    let readout=machine.readout();let expected=&generated["standalone_banked_readout"];
    if expected["restored"]!=readout.restored||expected["weight_lost_in_the_open"]!=readout.exposed{
        return Err("directional frame component differs from generated native banking receipt".into());
    }
    if output.is_none()||working!=Some(1)||readout.exposed!=0||!machine.frames().is_empty(){
        return Err("directional frame composition did not return through its native bank".into());
    }
    let mut output=output.unwrap();
    output["native_component_execution"]=serde_json::json!({"source_word":word,"trace":trace,
        "source_atom_coordinate_object":0,"composed_frame_object":1,"returned_working_object":working,
        "restored_units":readout.restored,"exposed_units":readout.exposed,"inert_steps":readout.inert,
        "surviving_object_refs":readout.surviving.iter().map(|d|d.observation).collect::<Vec<_>>()});
    Ok(output)
}

fn transport_gram(state:&mut serde_json::Value,points:&[Point],coefficients:[f64;3],previous:Option<&serde_json::Value>)->Result<(),String>{
    let [pd,pa,c]=coefficients;let lower=pd+pa-2.0*c.abs();
    state["source_gram_coefficients"]=serde_json::json!(coefficients);
    if !lower.is_finite()||lower<=0.0{
        state["frame_gram_status"]=serde_json::json!("held: source Gram lacks a strictly positive shared-proton curvature bound");return Ok(());
    }
    let read_point=|v:&serde_json::Value|->Result<Point,String>{Ok((v[0].as_f64().ok_or("frame point x missing")?,v[1].as_f64().ok_or("frame point y missing")?,v[2].as_f64().ok_or("frame point z missing")?))};
    let mut old=alloc::collections::BTreeMap::new();
    if let Some(prior)=previous{if let (Some(frames),Some(contacts))=(prior["frames"].as_array(),prior["contacts"].as_array()){
        for contact in contacts{for alt in contact["frame_alternatives"].as_array().unwrap(){
            if alt["proton_position"].is_array(){
                let mut key=Vec::new();for slot in 0..2{let i=alt["source_frame_refs"][slot].as_u64().unwrap() as usize;key.extend(frames[i]["source_atoms"].as_array().unwrap().iter().map(|v|v.as_u64().unwrap()));}
                old.insert(key,read_point(&alt["proton_position"])?);
            }
        }}
    }}
    let frames=state["frames"].as_array().unwrap().clone();
    let multiply=|m:&[Point;3],p:Point|->Point{(v::dot(m[0],p),v::dot(m[1],p),v::dot(m[2],p))};
    for contact in state["contacts"].as_array_mut().unwrap(){
        let donor=points[contact["source_atom_pair"][0].as_u64().unwrap() as usize];
        for alt in contact["frame_alternatives"].as_array_mut().unwrap(){
            let r=alt["relative_orientation"].as_array().unwrap().iter().map(|x|x.as_f64().unwrap()).collect::<Vec<_>>();
            let q=core::array::from_fn::<_,3,_>(|i|(r[i],r[3+i],r[6+i]));
            let d=read_point(&alt["donor_frame_displacement"])?;
            let mut m=[(0.0,0.0,0.0);3];
            for i in 0..3{let row=core::array::from_fn::<_,3,_>(|j|if i==j{pd+pa+2.0*c*r[i*3+i]}else{c*(r[i*3+j]+r[j*3+i])});m[i]=(row[0],row[1],row[2]);}
            let b=v::add(v::scale(d,pa),v::scale(multiply(&q,d),c));
            let determinant=v::dot(m[0],v::cross(m[1],m[2]));
            if !determinant.is_finite()||determinant<=0.0{return Err("transported mixed Gram lost positive curvature".into());}
            let h=v::scale(v::add(v::scale(v::cross(m[1],m[2]),b.0),v::add(v::scale(v::cross(m[2],m[0]),b.1),v::scale(v::cross(m[0],m[1]),b.2))),1.0/determinant);
            let i=alt["source_frame_refs"][0].as_u64().unwrap() as usize;
            let basis=frames[i]["basis_columns"].as_array().unwrap().iter().map(read_point).collect::<Result<Vec<_>,_>>()?;
            let position=v::add(donor,v::add(v::scale(basis[0],h.0),v::add(v::scale(basis[1],h.1),v::scale(basis[2],h.2))));
            let mut key=Vec::new();for slot in 0..2{let i=alt["source_frame_refs"][slot].as_u64().unwrap() as usize;key.extend(frames[i]["source_atoms"].as_array().unwrap().iter().map(|v|v.as_u64().unwrap()));}
            let previous=old.get(&key).copied();let prior=previous.unwrap_or(donor);let relative=v::sub(prior,donor);
            let h0=(v::dot(basis[0],relative),v::dot(basis[1],relative),v::dot(basis[2],relative));
            let energy=|x:Point|pd*v::dot(x,x)+pa*v::dot(v::sub(x,d),v::sub(x,d))+2.0*c*v::dot(x,multiply(&q,v::sub(x,d)));
            let residual=v::scale(v::sub(multiply(&m,h),b),2.0);
            alt["proton_position"]=serde_json::json!([position.0,position.1,position.2]);
            alt["proton_in_donor_frame"]=serde_json::json!([h.0,h.1,h.2]);
            alt["previous_proton_position"]=serde_json::json!([prior.0,prior.1,prior.2]);
            alt["continued_previous_proton"]=serde_json::json!(previous.is_some());
            alt["frame_gram_hessian"]=serde_json::json!(m.map(|p|[2.0*p.0,2.0*p.1,2.0*p.2]));
            alt["proton_force_residual"]=serde_json::json!([residual.0,residual.1,residual.2]);
            alt["gram_potential_before"]=serde_json::json!(energy(h0));alt["gram_potential_after"]=serde_json::json!(energy(h));
        }
    }
    state["frame_gram_status"]=serde_json::json!("shared proton fixed-point map executed in every retained frame alternative");
    state["frame_gram_definition"]=serde_json::json!("E=pd*|h|^2+pa*|h-d|^2+2c*h^T*R_DA^T*(h-d); M*h=(pa*I+c*R_DA^T)*d; M=(pd+pa)*I+c*(R_DA+R_DA^T)");
    state["hessian_lower_bound"]=serde_json::json!(2.0*lower);
    let (_,_,coupled)=chain_response(state,points,coefficients,true)?;
    state["coupled_chain_response"]=coupled;
    Ok(())
}

fn component(p:Point,i:usize)->f64{[p.0,p.1,p.2][i]}
fn matrix_apply(m:&[Point;3],p:Point)->Point{(v::dot(m[0],p),v::dot(m[1],p),v::dot(m[2],p))}
fn solve(m:&[Point;3],b:Point)->Result<Point,String>{
    let determinant=v::dot(m[0],v::cross(m[1],m[2]));
    if !determinant.is_finite()||determinant<=0.0{return Err("coupled donor Gram lacks positive determinant".into());}
    Ok(v::scale(v::add(v::scale(v::cross(m[1],m[2]),b.0),v::add(v::scale(v::cross(m[2],m[0]),b.1),v::scale(v::cross(m[0],m[1]),b.2))),1.0/determinant))
}
fn frame_pullback(points:&[Point],path:[usize;3],basis:[Point;3],g:[Point;3],out:&mut[Point]){
    let[a,b,c]=path;let[e,_,n]=basis;let[mut ge,gt,mut gn]=g;
    gn=v::add(gn,v::cross(e,gt));ge=v::add(ge,v::cross(gt,n));
    let normalize=|raw:Point,gradient:Point|{let length=libm::sqrt(v::dot(raw,raw));let unit=v::scale(raw,1.0/length);v::scale(v::sub(gradient,v::scale(unit,v::dot(unit,gradient))),1.0/length)};
    let u=v::sub(points[b],points[a]);let gw=normalize(v::cross(u,e),gn);
    let gu=v::cross(e,gw);ge=v::add(ge,v::cross(gw,u));let gv=normalize(v::sub(points[c],points[b]),ge);
    out[a]=v::sub(out[a],gu);out[b]=v::add(out[b],v::sub(gu,gv));out[c]=v::add(out[c],gv);
}

/// All retained source channels act on one chain and one proton per donor.
/// Eliminated proton forces and covalent-frame derivatives return to that chain.
pub(crate) fn chain_response(source:&serde_json::Value,points:&[Point],coefficients:[f64;3],record:bool)->Result<(f64,Vec<Point>,serde_json::Value),String>{
    source_response(source,points,coefficients,record,false)
}

pub(crate) fn conditional_chain_response(source:&serde_json::Value,points:&[Point],coefficients:[f64;3],record:bool)->Result<(f64,Vec<Point>,serde_json::Value),String>{
    source_response(source,points,coefficients,record,true)
}

fn source_response(source:&serde_json::Value,points:&[Point],coefficients:[f64;3],record:bool,conditional:bool)->Result<(f64,Vec<Point>,serde_json::Value),String>{
    let[pd,pa,c]=coefficients;
    if pd+pa<=2.0*c.abs(){return Err("coupled source Gram lacks positive shared-proton curvature".into());}
    if c==0.0{return diagonal_chain_response(source,points,pd,pa,record,conditional);}
    let mut paths=Vec::new();let mut frames=Vec::new();
    for frame in source["frames"].as_array().ok_or("coupled source frames missing")?{
        let path=core::array::from_fn::<_,3,_>(|i|frame["source_atoms"][i].as_u64().unwrap() as usize);
        let axis=v::sub(points[path[2]],points[path[1]]);let length=libm::sqrt(v::dot(axis,axis));
        let normal=v::cross(v::sub(points[path[1]],points[path[0]]),axis);let size=libm::sqrt(v::dot(normal,normal));
        if length==0.0||size==0.0{return Err("coupled source frame became singular".into());}
        let e=v::scale(axis,1.0/length);let n=v::scale(normal,1.0/size);paths.push(path);frames.push([e,v::cross(n,e),n]);
    }
    let mut bindings=Vec::new();
    for contact in source["contacts"].as_array().ok_or("coupled source contact guards missing")?{
        let d=contact["source_atom_pair"][0].as_u64().unwrap() as usize;let a=contact["source_atom_pair"][1].as_u64().unwrap() as usize;
        for alt in contact["frame_alternatives"].as_array().unwrap(){bindings.push((d,a,alt["source_frame_refs"][0].as_u64().unwrap() as usize,alt["source_frame_refs"][1].as_u64().unwrap() as usize));}
    }
    let groups=if conditional{bindings.len()}else{points.len()};
    let mut matrices=alloc::vec![[(0.0,0.0,0.0);3];groups];let mut first=alloc::vec![(0.0,0.0,0.0);groups];let mut counts=alloc::vec![0usize;groups];
    for(index,&(d,a,i,j))in bindings.iter().enumerate(){
        let group=if conditional{index}else{d};
        let k=core::array::from_fn::<_,3,_>(|row|(0..3).fold((0.0,0.0,0.0),|sum,t|v::add(sum,v::scale(frames[j][t],component(frames[i][t],row)))));
        let kt=core::array::from_fn::<_,3,_>(|row|(component(k[0],row),component(k[1],row),component(k[2],row)));
        for row in 0..3{
            let diagonal=core::array::from_fn::<_,3,_>(|col|if row==col{pd+pa}else{0.0});
            matrices[group][row]=v::add(matrices[group][row],v::add((diagonal[0],diagonal[1],diagonal[2]),v::scale(v::add(k[row],kt[row]),c)));
        }
        first[group]=v::add(first[group],v::add(v::add(v::scale(points[d],pd),v::scale(points[a],pa)),v::scale(v::add(matrix_apply(&k,points[a]),matrix_apply(&kt,points[d])),c)));counts[group]+=1;
    }
    let mut protons=alloc::vec![(0.0,0.0,0.0);groups];let mut roots=Vec::new();
    for group in 0..groups{if counts[group]!=0{
        let d=if conditional{bindings[group].0}else{group};
        protons[group]=solve(&matrices[group],first[group])?;let residual=v::scale(v::sub(matrix_apply(&matrices[group],protons[group]),first[group]),2.0);
        if record{roots.push(serde_json::json!({"donor_atom":d,"position":[protons[group].0,protons[group].1,protons[group].2],"source_channel_count":counts[group],"source_alternative":if conditional{Some([bindings[group].1,bindings[group].2,bindings[group].3])}else{None},"force_residual":[residual.0,residual.1,residual.2],"hessian_lower_bound":2.0*counts[group] as f64*(pd+pa-2.0*c.abs())}));}
    }}
    let mut energy=0.0;let mut gradient=alloc::vec![(0.0,0.0,0.0);points.len()];
    for(index,&(d,a,i,j))in bindings.iter().enumerate(){
        let group=if conditional{index}else{d};
        let hd=v::sub(protons[group],points[d]);let ha=v::sub(protons[group],points[a]);
        let u=core::array::from_fn::<_,3,_>(|t|v::dot(frames[i][t],hd));let w=core::array::from_fn::<_,3,_>(|t|v::dot(frames[j][t],ha));
        let mut gd=[(0.0,0.0,0.0);3];let mut ga=gd;
        for t in 0..3{
            energy+=pd*u[t]*u[t]+pa*w[t]*w[t]+2.0*c*u[t]*w[t];
            let left=2.0*(pd*u[t]+c*w[t]);let right=2.0*(pa*w[t]+c*u[t]);
            gradient[d]=v::sub(gradient[d],v::scale(frames[i][t],left));gradient[a]=v::sub(gradient[a],v::scale(frames[j][t],right));
            gd[t]=v::scale(hd,left);ga[t]=v::scale(ha,right);
        }
        frame_pullback(points,paths[i],frames[i],gd,&mut gradient);frame_pullback(points,paths[j],frames[j],ga,&mut gradient);
    }
    let report=if record{serde_json::json!({"source_coefficients":coefficients,"shared_donor_protons":roots,
        "retained_source_channel_count":bindings.len(),"potential":energy,
        "heavy_atom_gradient":gradient.iter().map(|p|[p.0,p.1,p.2]).collect::<Vec<_>>(),
        "operation":if conditional{"source candidate labels retained with a conditional proton for each pair/frame operand"}else{"source contact Grams summed with one shared proton per donor; envelope force and every source-frame derivative returned to the same chain"},
        "scope":"coupled held contact-candidate model; chemical admission remains retained"})}else{serde_json::Value::Null};
    Ok((energy,gradient,report))
}

fn diagonal_chain_response(source:&serde_json::Value,points:&[Point],pd:f64,pa:f64,record:bool,conditional:bool)->Result<(f64,Vec<Point>,serde_json::Value),String>{
    // Orthogonal frame norms cancel exactly when the source cross-entry is
    // zero. Retain channel multiplicities; this is not a coordinate average.
    let mut pairs=Vec::new();let mut counts=alloc::vec![0usize;points.len()];let mut first=alloc::vec![(0.0,0.0,0.0);points.len()];
    for contact in source["contacts"].as_array().ok_or("diagonal source contacts missing")?{
        let d=contact["source_atom_pair"][0].as_u64().unwrap() as usize;let a=contact["source_atom_pair"][1].as_u64().unwrap() as usize;
        let count=contact["frame_alternatives"].as_array().unwrap().len();if count==0{continue;}
        pairs.push((d,a,count));counts[d]+=count;
        first[d]=v::add(first[d],v::scale(v::add(v::scale(points[d],pd),v::scale(points[a],pa)),count as f64));
    }
    if conditional{
        let coefficient=pd*pa/(pd+pa);let mut energy=0.0;let mut gradient=alloc::vec![(0.0,0.0,0.0);points.len()];let mut roots=Vec::new();
        for(d,a,count)in pairs{
            let delta=v::sub(points[d],points[a]);let weight=count as f64;
            energy+=weight*coefficient*v::dot(delta,delta);let g=v::scale(delta,2.0*weight*coefficient);gradient[d]=v::add(gradient[d],g);gradient[a]=v::sub(gradient[a],g);
            if record{let h=v::scale(v::add(v::scale(points[d],pd),v::scale(points[a],pa)),1.0/(pd+pa));roots.push(serde_json::json!({"source_atom_pair":[d,a],"position":[h.0,h.1,h.2],"source_channel_count":count}));}
        }
        let report=if record{serde_json::json!({"source_coefficients":[pd,pa,0.0],"conditional_protons":roots,"potential":energy,"heavy_atom_gradient":gradient.iter().map(|p|[p.0,p.1,p.2]).collect::<Vec<_>>(),"operation":"retain the source pair label during proton fixed-point closure; exact diagonal-source reduction","scope":"alternative operand realization; contact-map interpretation remains carried"})}else{serde_json::Value::Null};
        return Ok((energy,gradient,report));
    }
    let mut protons=alloc::vec![(0.0,0.0,0.0);points.len()];let mut roots=Vec::new();
    for d in 0..points.len(){if counts[d]!=0{
        let stiffness=(pd+pa)*counts[d] as f64;protons[d]=v::scale(first[d],1.0/stiffness);
        if record{let residual=v::scale(v::sub(v::scale(protons[d],stiffness),first[d]),2.0);
            roots.push(serde_json::json!({"donor_atom":d,"position":[protons[d].0,protons[d].1,protons[d].2],"source_channel_count":counts[d],"force_residual":[residual.0,residual.1,residual.2],"hessian_lower_bound":2.0*stiffness}));}
    }}
    let mut energy=0.0;let mut gradient=alloc::vec![(0.0,0.0,0.0);points.len()];
    for(d,a,count)in pairs{
        let m=count as f64;let u=v::sub(protons[d],points[d]);let w=v::sub(protons[d],points[a]);
        energy+=m*(pd*v::dot(u,u)+pa*v::dot(w,w));
        gradient[d]=v::sub(gradient[d],v::scale(u,2.0*m*pd));gradient[a]=v::sub(gradient[a],v::scale(w,2.0*m*pa));
    }
    let report=if record{serde_json::json!({"source_coefficients":[pd,pa,0.0],"shared_donor_protons":roots,
        "retained_source_channel_count":counts.iter().sum::<usize>(),"potential":energy,
        "heavy_atom_gradient":gradient.iter().map(|p|[p.0,p.1,p.2]).collect::<Vec<_>>(),
        "operation":"exact diagonal-source reduction of the same shared-proton frame Gram; all source channel multiplicities retained",
        "scope":"coupled held contact-candidate model; chemical admission remains retained"})}else{serde_json::Value::Null};
    Ok((energy,gradient,report))
}

fn compose(molecule:&serde_json::Value,points:&[Point],step:usize)->Result<serde_json::Value,String>{
    let atoms=molecule["atoms"].as_array().ok_or("contact frame atom source missing")?;
    if points.len()!=atoms.len(){return Err("contact frame coordinate source differs from atom inventory".into());}
    let mut adjacent=alloc::vec![Vec::new();points.len()];
    for bond in molecule["bonds"].as_array().ok_or("contact frame bonds missing")?{
        let(a,b)=(bond[0].as_u64().unwrap() as usize,bond[1].as_u64().unwrap() as usize);
        adjacent[a].push(b);adjacent[b].push(a);
    }
    let candidate_source=crate::protein_contact_gram::ContactGram::prepare(molecule)?.report();
    let candidates=candidate_source["candidate_source_atom_pairs"].as_array().unwrap();
    let mut needed=alloc::vec![false;points.len()];
    for pair in candidates{for i in 0..2{needed[pair[i].as_u64().unwrap() as usize]=true;}}
    let mut frames:Vec<(usize,[Point;3])>=Vec::new();let mut records=Vec::new();let mut held=Vec::new();
    for c in 0..points.len(){if !needed[c]{continue;}
        for &b in &adjacent[c]{for &a in &adjacent[b]{if a==c{continue;}
            let axis=v::sub(points[c],points[b]);let normal=v::cross(v::sub(points[b],points[a]),axis);
            let axis2=v::dot(axis,axis);let normal2=v::dot(normal,normal);
            if axis2==0.0||normal2==0.0{
                held.push(serde_json::json!({"source_atoms":[a,b,c],"reason":"singular covalent frame"}));continue;
            }
            let e=v::scale(axis,1.0/libm::sqrt(axis2));let n=v::scale(normal,1.0/libm::sqrt(normal2));
            let basis=[e,v::cross(n,e),n];
            records.push(serde_json::json!({"frame_ref":frames.len(),"source_atoms":[a,b,c],"basis_columns":basis.map(|p|[p.0,p.1,p.2])}));
            frames.push((c,basis));
        }}
    }
    let mut compositions=Vec::new();
    for pair in candidates{
        let(d,a)=(pair[0].as_u64().unwrap() as usize,pair[1].as_u64().unwrap() as usize);
        let displacement=v::sub(points[a],points[d]);let mut alternatives=Vec::new();
        for(i,(_,df))in frames.iter().enumerate().filter(|(_,f)|f.0==d){
            for(j,(_,af))in frames.iter().enumerate().filter(|(_,f)|f.0==a){
                let relative:Vec<_>=(0..3).flat_map(|x|(0..3).map(move|y|v::dot(df[x],af[y]))).collect();
                let donor=core::array::from_fn::<_,3,_>(|k|v::dot(df[k],displacement));
                let acceptor=core::array::from_fn::<_,3,_>(|k|v::dot(af[k],displacement));
                alternatives.push(serde_json::json!({"source_frame_refs":[i,j],"relative_orientation":relative,
                    "donor_frame_displacement":donor,"acceptor_frame_displacement":acceptor}));
            }
        }
        compositions.push(serde_json::json!({"source_atom_pair":[d,a],"distance_squared":v::dot(displacement,displacement),"frame_alternatives":alternatives}));
    }
    let generated:serde_json::Value=serde_json::from_str(include_str!("../../measurements/protein_frame_transport_20261101/native_hydrogen_contact_interface.json")).map_err(|_|String::from("generated contact source binding invalid"))?;
    Ok(serde_json::json!({"source_step":step,"source_artifact":generated["artifact_id"],"source_artifact_sha256":generated["artifact_sha256"],
        "domain_action":"compose local donor and acceptor frames into their relative orientation matrix",
        "frames":records,"held_singular_frames":held,"contacts":compositions,
        "definition":"R_DA = F_D^T F_A; u_D = F_D^T(A-D); u_A = F_A^T(A-D)",
        "retention":"all ordered source covalent two-edge frame paths retained; no frame or contact selected",
        "scope":"executed covariant frame composition on held contact candidates; directional potential and chemical admission remain held"}))
}

#[cfg(test)]
mod tests{
    use super::*;
    #[test]
    fn joint_proton_envelope_returns_full_frame_force_to_same_chain(){
        let source=serde_json::json!({"frames":[{"source_atoms":[0,1,2]},{"source_atoms":[3,4,5]},{"source_atoms":[6,7,8]}],
            "contacts":[{"source_atom_pair":[2,5],"frame_alternatives":[{"source_frame_refs":[0,1]}]},
                        {"source_atom_pair":[2,8],"frame_alternatives":[{"source_frame_refs":[0,2]}]}]});
        let mut points=alloc::vec![(0.0,0.0,0.0),(1.0,0.1,0.0),(1.4,1.0,0.2),
            (2.0,-0.3,0.9),(2.4,0.7,0.3),(3.0,1.0,1.0),
            (-1.0,0.2,0.6),(-0.4,0.9,1.2),(0.2,1.3,0.8)];
        for conditional in [false,true]{for gram in [[0.6,0.4,0.15],[1.0,1.0,0.0]]{
            let(_,gradient,report)=source_response(&source,&points,gram,true,conditional).unwrap();
            if !conditional{assert_eq!(report["shared_donor_protons"].as_array().unwrap().len(),1);}
            let delta=libm::pow(2.0,-18.0);
            for i in 0..points.len(){for axis in 0..3{
                let original=points[i];let mut p=[original.0,original.1,original.2];p[axis]+=delta;points[i]=(p[0],p[1],p[2]);
                let plus=source_response(&source,&points,gram,false,conditional).unwrap().0;
                p[axis]-=2.0*delta;points[i]=(p[0],p[1],p[2]);
                let minus=source_response(&source,&points,gram,false,conditional).unwrap().0;points[i]=original;
                let measured=(plus-minus)/(2.0*delta);let actual=component(gradient[i],axis);
                assert!((actual-measured).abs()<1e-7*(1.0+actual.abs()),"atom {i}, axis {axis}: {actual} vs {measured}");
            }}
        }}
    }
}
