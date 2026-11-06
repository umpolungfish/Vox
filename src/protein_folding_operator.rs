//! Execute the supplied Protein_Folding_Operator through its banked word.
//! The source graph and the banked sequential machine retain their own readouts.
use alloc::{string::{String,ToString},vec::Vec};
use imasm_core::check::Graph;
use crate::{phase_word::{Continuation,Readout},sic::frame::{Complex,Operator,Sic}};

#[derive(Clone,Debug,PartialEq,Eq)]
struct SourceObservation { step:usize, object:usize }
fn observations(readout:&Readout<SourceObservation>)->serde_json::Value{
    serde_json::json!({"runtime_register":readout.register,
        "surviving":readout.surviving.iter().map(|d|serde_json::json!({"id":d.id,"runtime_lane":d.lane,
            "source_step":d.observation.step,"object_ref":d.observation.object})).collect::<Vec<_>>(),
        "cleared":readout.cleared,"restored":readout.restored,"exposed":readout.exposed,
        "deposit_events":readout.deposits,"inert_steps":readout.inert,"restore_supports":readout.restore_supports})
}
fn lanes(mask:u8)->[f64;4]{core::array::from_fn(|i|if mask&(1<<i)!=0{1.0}else{0.0})}

#[derive(Clone)]
struct ProteinPayload { paths:crate::peptide_paths::BankedPathObject,spatial:crate::protein_spatial_domain::SpatialFamily,cycles:Vec<crate::protein_cycle_density::CycleDensity>,contact:Vec<crate::protein_contact_gram::ContactGram>,joint:Vec<crate::protein_joint_conformation::JointConformation> }
impl ProteinPayload {
    fn from_source(field:&serde_json::Value,molecule:&serde_json::Value)->Result<Self,String>{
        Ok(Self{paths:crate::peptide_paths::BankedPathObject::from_source(field)?,spatial:crate::protein_spatial_domain::SpatialFamily::source(molecule)?,cycles:crate::protein_cycle_density::CycleDensity::prepare(molecule)?,contact:alloc::vec![crate::protein_contact_gram::ContactGram::prepare(molecule)?],joint:crate::protein_joint_conformation::JointConformation::prepare(molecule)?})
    }
    fn identity_from_source(field:&serde_json::Value)->Result<Self,String>{
        Ok(Self{paths:crate::peptide_paths::BankedPathObject::identity_from_source(field)?,spatial:crate::protein_spatial_domain::SpatialFamily::identity(),cycles:Vec::new(),contact:Vec::new(),joint:Vec::new()})
    }
    fn fuse_sources(sources:&[&Self],step:usize)->Self{
        Self{paths:crate::peptide_paths::BankedPathObject::fuse_sources(&sources.iter().map(|s|&s.paths).collect::<Vec<_>>(),step),
            spatial:crate::protein_spatial_domain::SpatialFamily::fuse(&sources.iter().map(|s|&s.spatial).collect::<Vec<_>>(),step),
            cycles:sources.iter().flat_map(|s|s.cycles.iter().cloned()).collect(),contact:sources.iter().flat_map(|s|s.contact.iter().cloned()).collect(),joint:sources.iter().flat_map(|s|s.joint.iter().cloned()).collect()}
    }
    fn report(&self)->serde_json::Value{let mut report=self.paths.report();report["joint_residue_conformations"]=serde_json::json!(self.joint.iter().map(|c|c.report()).collect::<Vec<_>>());report["directional_contact_grams"]=serde_json::json!(self.contact.iter().map(|c|c.report()).collect::<Vec<_>>());report["spatial_state"]=self.spatial.report();report["cycle_mixed_states"]=serde_json::json!(self.cycles.iter().map(|c|c.report()).collect::<Vec<_>>());report}
}

struct PayloadRun {
    carried:Continuation<SourceObservation>, trace:Vec<serde_json::Value>,
    arena:Vec<ProteinPayload>, parents:Vec<Vec<usize>>, active:Vec<usize>,
    clears:Vec<serde_json::Value>, inert:Vec<usize>, fixed_at:Option<usize>,
}
fn ancestor(parents:&[Vec<usize>],a:usize,b:usize)->bool{
    a==b||parents[b].iter().any(|&p|ancestor(parents,a,p))
}
fn run_payload(source:&serde_json::Value,field:&serde_json::Value,molecule:&serde_json::Value,skip:&[usize])->Result<PayloadRun,String>{
    let mut run=PayloadRun{carried:Continuation::default(),trace:Vec::new(),
        arena:alloc::vec![ProteinPayload::from_source(field,molecule)?],parents:alloc::vec![Vec::new()],
        active:alloc::vec![0],clears:Vec::new(),inert:Vec::new(),fixed_at:None};
    let source_paths=run.arena[0].paths.clone();
    let mut frozen=alloc::vec![false];
    for (step,mark) in source["glyph_word"].as_str().ok_or("missing word")?.chars().enumerate(){
        if skip.contains(&step){continue;}
        let before=run.carried.readout().clone();let active_before=run.active.clone();
        let was_fixed=run.carried.is_fixed();
        if !was_fixed&&matches!(mark,'⊤'|'⊥'|'⊞')&&run.active.is_empty(){
            run.active.push(run.arena.len());run.arena.push(ProteinPayload::identity_from_source(field)?);
            run.parents.push(Vec::new());frozen.push(false);
        }
        let observation=SourceObservation{step,object:run.active.first().copied().unwrap_or(usize::MAX)};
        let mut executed=false;
        run.carried.advance_with(&mark.to_string(),&observation,|_,_|{executed=true;Ok(())})?;
        if executed&&matches!(mark,'⋈'|'≻'){
            for id in &mut run.active{
                if frozen[*id]{let old=*id;*id=run.arena.len();run.arena.push(run.arena[old].clone());
                    run.parents.push(alloc::vec![old]);frozen.push(false);}
                if mark=='⋈'{run.arena[*id].paths.compose_at(step);}else{run.arena[*id].paths.propagate_at(step,&source_paths)?;}
                run.arena[*id].spatial.act(step,mark)?;
                for contact in &mut run.arena[*id].contact{contact.prepare_from_field(step,before.register)?;}
                if mark=='⋈'{for cycle in &mut run.arena[*id].cycles{cycle.act(step)?;}}
                let cycles=run.arena[*id].cycles.iter().map(|c|c.report()).collect::<Vec<_>>();
                let gram=run.arena[*id].contact.first().and_then(|c|c.spatial_gram());
                for joint in &mut run.arena[*id].joint{joint.act(step,mark,molecule,&cycles,gram)?;}
            }
        }
        if executed&&mark=='⊤'{for &id in &run.active{run.arena[id].spatial.assert_source_angles(step);}}
        if executed&&mark=='≺'{
            for &id in &run.active{frozen[id]=true;}
            run.clears.push(serde_json::json!({"source_step":step,"cleared_ids":before.surviving.iter().map(|d|d.id).collect::<Vec<_>>(),
                "held_object_refs":run.active,"held_object_versions":run.active.iter().map(|&id|run.arena[id].paths.version()).collect::<Vec<_>>(),
                "banked_before_clear":before.surviving.iter().all(|d|run.carried.frames().iter().flatten().any(|h|h.id==d.id))}));
            run.active.clear();
        }
        if executed&&mark=='∋'{
            let mut roots=run.active.clone();
            for d in &run.carried.readout().surviving{if !roots.contains(&d.observation.object){roots.push(d.observation.object);}}
            let all=roots.clone();roots.retain(|&a|!all.iter().any(|&b|a!=b&&ancestor(&run.parents,a,b)));
            if roots.len()>1{
                let joined=ProteinPayload::fuse_sources(&roots.iter().map(|&id|&run.arena[id]).collect::<Vec<_>>(),step);
                let id=run.arena.len();run.arena.push(joined);run.parents.push(roots);frozen.push(false);run.active=alloc::vec![id];
            }else{run.active=roots;}
        }
        if executed&&matches!(mark,'⊤'|'⊥'|'⊞'|'∋'){
            let mut counts=[0.0;4];
            for deposit in &run.carried.readout().surviving{for(i,value)in lanes(deposit.lane).iter().enumerate(){counts[i]+=value;}}
            let total=counts.iter().sum::<f64>();
            if total>0.0{
                let mut density=Operator::zero(4);for i in 0..4{density.set(i,i,Complex::new(counts[i]/total,0.0));}
                for &id in &run.active{for contact in &mut run.arena[id].contact{contact.update(step,&density,total)?;}}
            }
        }
        if !executed{run.inert.push(step);}
        if run.carried.is_fixed()&&!was_fixed{run.fixed_at=Some(step);}
        run.trace.push(serde_json::json!({"source_step":step,"opcode":mark.to_string(),"payload_executed":executed,
            "domain_action":source["phases"]["phase_4"]["steps"][step]["domain_action"],
            "payload_scope":"ordered source paths and exact spatial distance domains in the native banks; native contact selection unresolved",
            "active_payload_before":active_before,"active_payload_after":run.active,"before":observations(&before),
            "after":observations(run.carried.readout()),"open_frames":run.carried.frames().iter().map(|frame|
                frame.iter().map(|d|serde_json::json!({"id":d.id,"runtime_lane":d.lane,"object_ref":d.observation.object})).collect::<Vec<_>>()).collect::<Vec<_>>()}));
    }
    Ok(run)
}
fn payload_report(run:&PayloadRun)->serde_json::Value{
    serde_json::json!({"active_roots":run.active,"active_path_count":run.active.iter().map(|&id|run.arena[id].paths.path_count()).sum::<usize>(),
        "arena":run.arena.iter().enumerate().map(|(id,o)|serde_json::json!({"object_id":id,"parents":run.parents[id],"payload":o.report(),
            "reachable_from_active":run.active.iter().any(|&root|ancestor(&run.parents,id,root))})).collect::<Vec<_>>()})
}

fn graph_domain_run(source:&serde_json::Value,field:&serde_json::Value,molecule:&serde_json::Value,
    graph:&Graph,node_masks:&[u8],edge_masks:&[u8])->Result<serde_json::Value,String>{
    // The native graph carries IFIX and applies AREV's involution. Its domain
    // binding therefore retains immutable records and a separate working cursor.
    // This does not change the sequential banked interpreter or its receipts.
    if graph.edges.iter().any(|&(a,b)|a>=b){return Err("graph domain continuation needs its cyclic scheduler".into());}
    let word=source["glyph_word"].as_str().unwrap().chars().collect::<Vec<_>>();
    let mut arena=alloc::vec![ProteinPayload::from_source(field,molecule)?];
    let mut parents:Vec<Vec<usize>>=alloc::vec![Vec::new()];let mut kinds=alloc::vec!["prepared source"];
    let original_paths=arena[0].paths.clone();let mut edges=alloc::vec![Vec::<usize>::new();graph.edges.len()];
    let mut trace=Vec::new();let mut records:Vec<(usize,Vec<usize>,Vec<String>)>=Vec::new();let mut node_roots=Vec::new();
    let mut inverse_pairs:Vec<(usize,usize,usize)>=Vec::new();
    let mut fuse_information=Vec::new();
    fn undo(id:usize,parents:&[Vec<usize>],kinds:&[&str],pairs:&[(usize,usize,usize)])->Vec<usize>{
        if kinds[id]=="AREV"{if let Some(&(a,b,_))=pairs.iter().find(|&&(a,b,_)|a==id||b==id){return alloc::vec![if a==id{b}else{a}];}}
        if kinds[id]=="AFWD"||kinds[id]=="CLINK"{return parents[id].clone();}
        if parents[id].is_empty(){return alloc::vec![id];}
        let mut result=Vec::new();for &p in &parents[id]{for prior in undo(p,parents,kinds,pairs){if !result.contains(&prior){result.push(prior);}}}result
    }
    for (node,&mark) in word.iter().enumerate(){
        let mut roots=Vec::new();
        if node==0{roots.push(0);}else{for(e,&(_,b))in graph.edges.iter().enumerate(){if b==node{for &id in &edges[e]{if !roots.contains(&id){roots.push(id);}}}}}
        let input_roots=roots.clone();let input_mask=node_masks[node];
        let output_mask=graph.edges.iter().enumerate().filter(|(_,edge)|edge.0==node).fold(0,|m,(e,_)|m|edge_masks[e]);
        let mut executed=false;let mut memory_sources=Vec::new();
        if mark=='≺'&&input_mask!=0{
            let mut restored=Vec::new();for &id in &roots{
                if let Some(&(a,b,_))=inverse_pairs.iter().find(|&&(a,b,_)|a==id||b==id){let p=if a==id{b}else{a};if !restored.contains(&p){restored.push(p);}continue;}
                let predecessors=undo(id,&parents,&kinds,&inverse_pairs);
                if predecessors.len()!=1{return Err("graph reverse retains multiple correlated checkpoints requiring a joint continuation".into());}
                let previous=predecessors[0];let target=arena.len();arena.push(arena[previous].clone());parents.push(alloc::vec![id]);kinds.push("AREV");
                inverse_pairs.push((id,target,node));restored.push(target);
            }
            roots=restored;executed=true;
        }else{
            if mark=='⋈'&&input_mask!=0{
                if let Some((record_node,record_roots,_))=records.last(){
                    memory_sources.push(*record_node);
                    for &id in record_roots{if !roots.contains(&id){roots.push(id);}}
                }
            }
            // A graph fuse unions complete source-addressed alternatives.
            // Their cycle matrices and coordinate arrays remain paired inside
            // each payload; no flat cross-product or coordinate average is made.
            let movement=input_mask!=0&&matches!(mark,'≻'|'⋈');
            let affirmative=input_mask!=0&&mark=='⊤';
            let observation=output_mask!=0&&matches!(mark,'⊤'|'⊥'|'⊞'|'∋');
            if movement||affirmative||observation{
                let mut transformed=Vec::new();
                for &old in &roots{
                    let mut state=arena[old].clone();
                    if movement{
                        if mark=='⋈'{state.paths.compose_at(node);}else{state.paths.propagate_at(node,&original_paths)?;}
                        state.spatial.act(node,mark)?;
                        for contact in &mut state.contact{contact.prepare_from_field(node,input_mask)?;}
                        if mark=='⋈'{for cycle in &mut state.cycles{cycle.act(node)?;}}
                        let cycles=state.cycles.iter().map(|c|c.report()).collect::<Vec<_>>();
                        let gram=state.contact.first().and_then(|c|c.spatial_gram());
                        for joint in &mut state.joint{joint.act(node,mark,molecule,&cycles,gram)?;}
                    }
                    if affirmative{state.spatial.assert_source_angles(node);}
                    if observation{
                        let counts=lanes(output_mask);let scale=counts.iter().sum::<f64>();let mut density=Operator::zero(4);
                        for i in 0..4{density.set(i,i,Complex::new(counts[i]/scale,0.0));}
                        for contact in &mut state.contact{contact.update(node,&density,scale)?;}
                    }
                    let id=arena.len();arena.push(state);parents.push(alloc::vec![old]);kinds.push(if movement{if mark=='⋈'{"CLINK"}else{"AFWD"}}else{"source observation"});transformed.push(id);
                }roots=transformed;executed=true;
            }
        }
        if mark=='⊡'&&input_mask!=0{
            let frozen=roots.iter().map(|&id|arena[id].report().to_string()).collect::<Vec<_>>();
            records.push((node,roots.clone(),frozen));executed=true;
        }
        if mark=='∋'{
            let reg=|m:u8|imasm_core::imasm16_3::Reg16_3{big_t:m&1!=0,big_f:m&2!=0,small_t:m&4!=0,small_f:m&8!=0};
            let joined=reg(input_mask);
            let incoming=graph.edges.iter().enumerate().filter(|(_,e)|e.1==node).map(|(e,_)|edge_masks[e]).collect::<Vec<_>>();
            let upper_bound=incoming.iter().all(|&m|imasm_core::imasm16_3::leq_i(reg(m),joined));
            let retained=input_roots.iter().all(|&id|roots.iter().any(|&out|ancestor(&parents,id,out)));
            if !upper_bound||!retained{return Err("native fuse lost incoming information or payload ancestry".into());}
            fuse_information.push(serde_json::json!({"source_step":node,"incoming_native_masks":incoming,
                "native_join_mask":input_mask,"join_above_every_input":upper_bound,
                "input_object_refs":input_roots,"output_object_refs":roots,"all_input_payload_ancestry_retained":retained,
                "operation":"native information join with retained whole-source alternatives"}));
        }
        for(e,&(a,_))in graph.edges.iter().enumerate(){if a==node{edges[e]=roots.clone();}}
        trace.push(serde_json::json!({"source_step":node,"opcode":mark.to_string(),"native_input_mask":input_mask,"native_output_mask":output_mask,
            "input_object_refs":input_roots,"output_object_refs":roots,"physical_action_executed":executed,
            "immutable_memory_sources":memory_sources,"domain_action":source["phases"]["phase_4"]["steps"][node]["domain_action"],
            "scope":"native explicit-edge graph interpretation; zero-evidence source alternatives remain carried"}));
        node_roots.push(roots);
    }
    for(_,refs,frozen)in &records{for(&id,text)in refs.iter().zip(frozen){if arena[id].report().to_string()!=*text{return Err("graph continuation mutated an immutable source record".into());}}}
    let mut terminal=Vec::new();for(node,roots)in node_roots.iter().enumerate(){if graph.out_degree(node)==0{for &id in roots{if !terminal.contains(&id){terminal.push(id);}}}}
    Ok(serde_json::json!({"interpretation":"native graph-flow IFIX carry, AREV involution; full payload checkpoints supply reversible domain continuation",
        "native_node_inputs":node_masks,"native_edge_values":edge_masks,"trace":trace,"terminal_object_refs":terminal,
        "fuse_information_order":fuse_information,
        "immutable_records":records.iter().map(|(node,refs,_)|serde_json::json!({"source_step":node,"object_refs":refs,"unchanged_after_continuation":true})).collect::<Vec<_>>(),
        "reverse_involution_pairs":inverse_pairs.iter().map(|&(a,b,node)|serde_json::json!({"objects":[a,b],"created_at_source_step":node,"second_application_returns_input":true})).collect::<Vec<_>>(),
        "arena":arena.iter().enumerate().map(|(id,state)|serde_json::json!({"object_id":id,"parents":parents[id],"operation":kinds[id],"payload":state.report()})).collect::<Vec<_>>(),
        "folding_equilibrium_closed":false,"scope":"bounded source-graph mechanical realization; complete atom-level equilibrium remains open"}))
}

pub(crate) fn prepare(source:&serde_json::Value,peptide:serde_json::Value,graph:&Graph,seed:u8)
    ->Result<serde_json::Value,String>{
    let word=source["glyph_word"].as_str().ok_or("missing supplied protein word")?;
    let tuple=source["grounded_tuple"].as_str().ok_or("missing supplied protein tuple")?;
    let rounds=source["reentry"]["rounds"].as_array().ok_or("protein source lacks its typed re-entry binding")?;
    let typed=rounds.last().ok_or("protein source has no typed re-entry round")?;
    let logic=&typed["logical_expressions"]["navigators"]["CL9NK"];
    if logic["tuple"]["⋈"]!="𐑞"{return Err("banked protein entry requires its supplied mixed-state Fidelity.they binding".into());}
    let input=imasm_core::imasm16_3::Reg16_3{big_t:seed&1!=0,big_f:seed&2!=0,small_t:seed&4!=0,small_f:seed&8!=0};
    let (graph_nodes,graph_edges)=imasm_core::flow::flow_values(graph,input);
    let runtime_mask=|r:&imasm_core::imasm16_3::Reg16_3|u8::from(r.big_t)|u8::from(r.big_f)<<1|u8::from(r.small_t)<<2|u8::from(r.small_f)<<3;
    let node_values=graph_nodes.iter().map(runtime_mask).collect::<Vec<_>>();
    let edge_values=graph_edges.iter().map(runtime_mask).collect::<Vec<_>>();
    if serde_json::json!(node_values)!=source["wiring_check"]["node_inputs"]
        ||serde_json::json!(edge_values)!=source["wiring_check"]["edge_values"]{
        return Err("supplied protein graph readout differs from its source receipt".into());
    }
    let chain=peptide["residues"].as_array().ok_or("missing source residues")?.iter()
        .map(|r|r["aa"].as_str().ok_or("missing spatial source identity")).collect::<Result<Vec<_>,_>>()?;
    let mut molecule=crate::peptide_closure::molecular_constraint_source(&chain);
    molecule["residue_identities"]=serde_json::json!(chain);
    molecule=crate::peptide_closure::complete_open_carboxyl(&molecule)?;
    molecule["source_codons"]=serde_json::json!(peptide["residues"].as_array().unwrap().iter().map(|r|r["codon"].clone()).collect::<Vec<_>>());
    let fork=graph.nodes.iter().position(|t|t.is_brancher()).ok_or("protein graph has no native conformational fork")?;
    let stitch3=logic["formula_decomposition"]["promoted_atoms"].as_array().is_some_and(|a|a.iter().any(|v|v=="STITCH_3"));
    let arity=if stitch3{3}else{graph.out_degree(fork)};
    let fork_masks=(0..arity).map(|slot|runtime_mask(&imasm_core::flow::gate_out(graph.nodes[fork],input,input,slot,arity))).collect::<Vec<_>>();
    molecule["native_rotamer_fork_receipt"]=serde_json::json!({"instrument":"imasm_core::flow::gate_out","source_input_mask":seed,"arity":arity,"promoted_stitch_3":stitch3,"output_masks":fork_masks,"scope":"internal conformational decomposition; supplied external graph unchanged"});
    molecule["native_rotamer_fork_masks"]=serde_json::json!(fork_masks);
    molecule["native_rotamer_reverse_masks"]=serde_json::json!(fork_masks.iter().map(|&m|{
        let r=imasm_core::imasm16_3::Reg16_3{big_t:m&1!=0,big_f:m&2!=0,small_t:m&4!=0,small_f:m&8!=0};runtime_mask(&r.invol())
    }).collect::<Vec<_>>());
    // Character transport on the actual native T,F,t,f carrier. Retaining
    // cardinality makes this a lossless address, rather than a spatial average.
    // No identification with FOUR's N,T,F,B basis or molecular angles is made.
    fn affine(r:&imasm_core::imasm16_3::Reg16_3)->[i32;4]{
        let [a,b,c,d]=[r.big_t,r.big_f,r.small_t,r.small_f].map(i32::from);
        [a+b+c+d,a-b+c-d,a+b-c-d,a-b-c+d]
    }
    let mut frame_receipt=Vec::new();
    for mask in 0u8..16{
        let r=imasm_core::imasm16_3::Reg16_3{big_t:mask&1!=0,big_f:mask&2!=0,small_t:mask&4!=0,small_f:mask&8!=0};
        let image=affine(&r);let reverse=affine(&r.invol());
        if reverse!=[image[0],-image[1],image[2],-image[3]]{
            return Err("native affine AREV transport failed".into());
        }
        let [k,x,y,z]=image;
        let recovered=[k+x+y+z,k-x+y-z,k+x-y-z,k-x-y+z];
        if recovered!=[r.big_t,r.big_f,r.small_t,r.small_f].map(|b|4*i32::from(b)){
            return Err("native affine lane recovery failed".into());
        }
        frame_receipt.push(serde_json::json!({"native_mask":mask,"affine":image,"reverse_native_mask":runtime_mask(&r.invol()),"reverse_affine":reverse}));
    }
    molecule["native_affine_lattice_transport"]=serde_json::json!({
        "basis":["T","F","t","f"],"coordinates":["cardinality","T-F+t-f","T+F-t-f","T-F-t+f"],
        "rows":frame_receipt,"all_lanes_recovered":true,"actual_AREV_commutes":true,
        "proof":"Imscribing.Biochem.FoldingDistinctions.NativeReverseBridge",
        "scope":"lossless native register coordinates; molecular domain bridge remains held"});
    let mut domain=serde_json::json!({"molecular_source":molecule,
        "operation":"spatial distance domains execute inside the banked payload"});
    let field=crate::peptide_operator::lattice_field(graph,&peptide,seed)?;
    let graph_domain=graph_domain_run(source,&field,&domain["molecular_source"],graph,&node_values,&edge_values)?;
    domain["graph_domain_execution"]=graph_domain;
    let run=run_payload(source,&field,&domain["molecular_source"],&[])?;
    let carried=&run.carried;
    let trace=&run.trace;let mut clears=run.clears.clone();let inert=&run.inert;let fixed_at=run.fixed_at;
    let root_json=&field["ordered_source_carrier"]["source_root"];
    let root=(root_json[0].as_u64().unwrap() as usize,root_json[1].as_u64().unwrap() as u8);
    let residue_count=peptide["residues"].as_array().ok_or("missing source residues")?.len();
    let complete=run.active.iter().map(|&id|serde_json::json!({"object_ref":id,
        "witnesses":run.arena[id].paths.complete_chain_witnesses(root,residue_count)})).collect::<Vec<_>>();
    let final_readout=carried.readout();
    let expected=&source["banked_count_check"];
    for (actual,key) in [(final_readout.restored,"restored"),(final_readout.exposed,"weight_lost_in_the_open"),
        (final_readout.deposits,"deposits"),(final_readout.inert,"inert")]{
        if expected[key].as_u64()!=Some(actual as u64){return Err(alloc::format!("native protein execution disagrees with source {key}"));}
    }
    if carried.frames().len()!=0{return Err("protein source returns an open holding frame".into());}
    for clear in &mut clears{
        let ids=clear["cleared_ids"].as_array().unwrap();
        let restored=ids.iter().all(|id|final_readout.surviving.iter().any(|d|id.as_u64()==Some(d.id as u64)&&run.active.iter().any(|&root|ancestor(&run.parents,d.observation.object,root))));
        if !restored{return Err("protein outer frame did not restore its whole-object observations".into());}
        clear["restored_at_return"]=serde_json::json!(restored);
    }
    // A mixture of actual retained observation units. Recovered units keep
    // their IDs and are counted once, so restoration does not create mass.
    let mut weights=[0.0;4];
    for deposit in &final_readout.surviving{
        let lane=lanes(deposit.lane);for i in 0..4{weights[i]+=lane[i];}
    }
    let mass: f64=weights.iter().sum();if mass==0.0{return Err("protein source returned no retained evidence units".into());}
    let unit_counts=weights;
    for w in &mut weights{*w/=mass;}
    let mut density=Operator::zero(4);for i in 0..4{density.set(i,i,Complex::new(weights[i],0.0));}
    let purity: f64=weights.iter().map(|w|w*w).sum();
    let frame=crate::sic::wh::ququart_frame();let masses=frame.split(&density).map_err(|e|e.to_string())?;
    let recovered=frame.fuse(&masses).map_err(|e|e.to_string())?;
    let residual=recovered.distance(&density).map_err(|e|e.to_string())?;
    let tolerance=1024.0*f64::EPSILON;
    if purity>=1.0||residual>tolerance{return Err("protein source mixed-density return failed".into());}
    let recovered_purity=recovered.trace_product(&recovered).map_err(|e|e.to_string())?.re;
    let recovered_trace=recovered.trace();
    if (recovered_trace.re-1.0).abs()>tolerance||recovered_trace.im.abs()>tolerance||recovered_purity>=1.0{
        return Err("returned protein observation does not satisfy its mixed-state fragment".into());
    }
    // Keep the object label as a direct-sum block, rather than discard it
    // when taking the four-lane marginal above. Bank restoration preserves IDs.
    let mut blocks=alloc::collections::BTreeMap::<usize,[f64;4]>::new();
    for deposit in &final_readout.surviving{
        let block=blocks.entry(deposit.observation.object).or_insert([0.0;4]);
        for (i,value) in lanes(deposit.lane).iter().enumerate(){block[i]+=value/mass;}
    }
    let mut object_blocks=Vec::new();let mut joint_purity=0.0;
    for (object,weights) in blocks{
        let mut block=Operator::zero(4);for i in 0..4{block.set(i,i,Complex::new(weights[i],0.0));}
        let measured=frame.split(&block).map_err(|e|e.to_string())?;
        let returned=frame.fuse(&measured).map_err(|e|e.to_string())?;
        let error=returned.distance(&block).map_err(|e|e.to_string())?;
        if error>tolerance{return Err("object-addressed mixed-state SIC return failed".into());}
        joint_purity+=returned.trace_product(&returned).map_err(|e|e.to_string())?.re;
        object_blocks.push(serde_json::json!({"object_ref":object,"weights":weights,
            "retained_unit_ids":final_readout.surviving.iter().filter(|d|d.observation.object==object).map(|d|d.id).collect::<Vec<_>>(),
            "sic_masses":measured.values.iter().map(|z|[z.re,z.im]).collect::<Vec<_>>(),
            "returned_matrix":returned.values().iter().map(|z|[z.re,z.im]).collect::<Vec<_>>(),
            "residual":error}));
    }
    let pairs=source["wiring_check"]["pairs"].as_array().ok_or("protein source lacks its declared pairs")?;
    let outer=pairs.iter().find(|p|pairs.iter().any(|q|
        p[0].as_u64()<q[0].as_u64()&&q[1].as_u64()<p[1].as_u64())).ok_or("protein source lacks its nested outer holding frame")?;
    let (fork,fuse)=(outer[0].as_u64().unwrap() as usize,outer[1].as_u64().unwrap() as usize);
    let control_word=word.chars().enumerate().filter_map(|(i,c)|if i==fork||i==fuse{None}else{Some(c)}).collect::<String>();
    let control_run=run_payload(source,&field,&domain["molecular_source"],&[fork,fuse])?;
    let control=control_run.carried.readout().clone();
    if control_run.active.iter().any(|&root|ancestor(&control_run.parents,0,root)){
        return Err("unbanked control retained access to the cleared source payload".into());
    }
    if control.exposed==0||control.restored>=final_readout.restored{
        return Err("removing the outer holding frame did not expose the native clear".into());
    }
    let values=|op:&Operator|op.values().iter().map(|z|[z.re,z.im]).collect::<Vec<_>>();
    domain["native_lattice_field"]=field;
    domain["banked_contact_object"]=payload_report(&run);
    let density_report=serde_json::json!({"basis":"runtime T,F,t,f observation lanes",
            "object_addressed_blocks":object_blocks,"object_lane_joint_purity":joint_purity,
            "object_state_scope":"classical direct sum of retained payload labels with ququart lane blocks; source paths remain in the referenced payload arena",
            "weights":weights,"lane_unit_counts":unit_counts,"retained_source_units":mass,"purity":purity,"matrix":values(&density),
            "scope":"mixture of retained whole-object observations, not independent residue marginals",
            "weighting_rule":"one retained native evidence unit contributes one observation unit; restored IDs are counted once",
            "sic_effects":frame.effects().iter().map(values).collect::<Vec<_>>(),
            "sic_duals":frame.duals().iter().map(values).collect::<Vec<_>>(),
            "sic_masses":masses.values.iter().map(|z|[z.re,z.im]).collect::<Vec<_>>(),
            "sic_returned_matrix":values(&recovered),"sic_returned_purity":recovered_purity,
            "sic_returned_trace":[recovered_trace.re,recovered_trace.im],"returned_mixed_state_assertion":true,
            "signed_matrix_residuals":recovered.values().iter().zip(density.values()).map(|(a,b)|[a.re-b.re,a.im-b.im]).collect::<Vec<_>>(),
            "residual":residual,"tolerance":tolerance});
    Ok(serde_json::json!({"schema":"supplied-protein-folding-operator/v1","operator_artifact_id":source["artifact_id"],
        "operator_tuple":tuple,"operator_word":word,"operator_boundary":source["phases"]["phase_0"]["boundary_condition"],
        "source_fidelity":logic["tuple"]["⋈"],"source_reentry_round":typed["round"],
        "source_logical_fragments":logic["formula_decomposition"]["per_primitive_fragments"],
        "source_graph_readout":{"node_inputs":node_values,"edge_values":edge_values,
            "scope":"native explicit-graph flow; sequential banked return is retained separately"},
        "source_peptide":peptide,"source_wires":source["wiring_check"]["wires"],"source_seed":seed,
        "domain_carrier":domain,"complete_source_chain_paths":complete,"native_banked_trace":trace,"clear_restore_witnesses":clears,
        "native_return":observations(final_readout),"native_fixed_at":fixed_at,"native_inert_source_steps":inert,
        "outer_frame_control":{"removed_source_pair":[fork,fuse],"control_word":control_word,
            "complete_source_chain_paths":control_run.active.iter().map(|&id|control_run.arena[id].paths.complete_chain_witnesses(root,residue_count)).collect::<Vec<_>>(),
            "native_return":observations(&control),"payload":payload_report(&control_run),"trace":control_run.trace,"scope":"counterfactual only; supplied source word is preserved"},
        "mixed_observation_density":density_report,
        "status":"nested source holding frames and mixed contact-observation return executed; spatial folding remains open",
        "physical_fixation_permitted":false,"full_chain_lattice_fold_closed":false}))
}

/// Resume the retained physical payload without replaying the source word or its banks.
#[cfg(feature="hosted")]
pub(crate) fn resume_supplied(value:&mut serde_json::Value,budget:usize)->Result<bool,String>{
    if value["schema"]=="supplied-protein-folding-operator/v1"{
        if budget==0{return Ok(true);}
        let mut molecule=if value["domain_carrier"]["active_molecular_source"].is_object(){value["domain_carrier"]["active_molecular_source"].clone()}else{value["domain_carrier"]["molecular_source"].clone()};
        if std::env::var_os("VOX_COMPLETE_OPEN_CHAIN").is_some()&&!molecule["terminal_carboxyl_boundary"].is_object(){
            molecule=crate::peptide_closure::complete_open_carboxyl(&molecule)?;
            value["domain_carrier"]["active_molecular_source"]=molecule.clone();
        }

        let graph=&mut value["domain_carrier"]["graph_domain_execution"];
        let roots:Vec<usize>=serde_json::from_value(graph["terminal_object_refs"].clone()).map_err(|e|e.to_string())?;
        let arena=graph["arena"].as_array_mut().ok_or("retained graph arena missing")?;
        let mut next=Vec::new();let mut resumed=0;
        for old in roots{
            let mut object=arena.get(old).ok_or("retained terminal reference invalid")?.clone();
            let joints=object["payload"]["joint_residue_conformations"].as_array_mut().ok_or("retained joint states missing")?;
            for joint in joints{
                if joint["atomic_spatial_state"].is_null(){continue;}
                let state=crate::protein_atomic_conformation::AtomicConformation::resume_saved(&joint["atomic_spatial_state"],&molecule,budget)?;
                let before=joint["positions"].clone();
                joint["positions"]=serde_json::json!(state.positions());joint["atomic_spatial_state"]=state.report();
                if let Some(events)=joint["events"].as_array_mut(){events.push(serde_json::json!({"operation":"resume retained atomic refinement","positions_before":before,"positions_after":state.positions(),"source_operator_advanced":false}));}
                resumed+=1;
            }
            let id=arena.len();object["object_id"]=serde_json::json!(id);object["parents"]=serde_json::json!([old]);object["operation"]=serde_json::json!("retained atomic refinement continuation");
            arena.push(object);next.push(id);
        }
        if resumed==0{return Err("retained graph has no atomic state to resume".into());}
        graph["terminal_object_refs"]=serde_json::json!(next);
        let entry=serde_json::json!({"iteration_budget":budget,"resumed_atomic_states":resumed,"source_operator_advanced":false,"prior_arena_objects_preserved":true,"source_banks_preserved":true});
        if !graph["physical_continuations"].is_array(){graph["physical_continuations"]=serde_json::json!([]);}
        graph["physical_continuations"].as_array_mut().unwrap().push(entry);
        return Ok(true);
    }
    if let Some(object)=value.as_object_mut(){for child in object.values_mut(){if child.is_object()&&resume_supplied(child,budget)?{return Ok(true);}}}
    Ok(false)
}

/// Observe current physical roots without replaying or changing the membrane.
#[cfg(feature="hosted")]
pub(crate) fn supplied_observations(value:&serde_json::Value)->Option<serde_json::Value>{
    if value["schema"]!="supplied-protein-folding-operator/v1"{
        return value.as_object()?.values().filter(|v|v.is_object()).find_map(supplied_observations);
    }
    let graph=&value["domain_carrier"]["graph_domain_execution"];let arena=graph["arena"].as_array()?;let mut roots=Vec::new();
    for address in graph["terminal_object_refs"].as_array()?{
        let index=address.as_u64()? as usize;let object=arena.get(index)?;
        let parent=object["parents"][0].as_u64().and_then(|p|arena.get(p as usize));
        for (joint_index,joint)in object["payload"]["joint_residue_conformations"].as_array()?.iter().enumerate(){
            let state=&joint["atomic_spatial_state"];if state.is_null(){continue;}
            let old=parent.map(|p|&p["payload"]["joint_residue_conformations"][joint_index]);
            let previous=old.map(|p|&p["atomic_spatial_state"]);
            let events=state["events"].as_array()?;let last=events.last()?;
            let refinement=if last["coupled_refinement"].is_object(){&last["coupled_refinement"]}else{&last["post_admission_coupled_refinement"]};
            let offers=refinement["retained_source_direction_offers"].as_array().map(|offers|offers.iter().map(|offer|{
                let trials=offer["trials"].as_array().map(|trials|trials.iter().map(|trial|{
                    let repair=&trial["native_retraction"];let corrections=repair["corrections"].as_array();let last=corrections.and_then(|c|c.last());let search=&repair["representable_search"];
                    serde_json::json!({"direction":trial["direction"],"selected":trial["selected"],"admitted":trial["admitted"],"potential":trial["potential"],"scale":trial["scale"],"residue":trial["residue"],"axis":trial["axis"],"delta_degrees":trial["delta_degrees"],
                        "native_rejection":trial["native_rejection"],"projected_step_radians":trial["projected_step_radians"],"source_squared_distance_clearance":trial["source_squared_distance_clearance"],"source_distance_clearance":trial["source_distance_clearance"],
                        "native_retraction":if repair.is_object(){Some(serde_json::json!({"admitted":repair["admitted"],"iterations":corrections.map(Vec::len),
                            "representable_coordinate_advances":corrections.map(|c|c.iter().filter_map(|r|r["representable_coordinate_advances"].as_u64()).sum::<u64>()),
                            "last_failed_source_pairs":last.map(|r|r["source_atom_pairs"].clone()),"last_overlap_violations":last.map(|r|r["native_overlap_violations"].clone()),
                            "neighbor_attempts":search["attempts"].as_array().map(Vec::len),"neighbor_selected":search["attempts"].as_array().map(|a|a.iter().filter(|r|r["selected"]==true).count()),"neighbor_admitted":search["admitted"],"remaining_failed_pairs":search["remaining_failed_pairs"],"remaining_native_issues":repair["remaining_native_issues"]}))}else{None}})
                }).collect::<Vec<_>>()).unwrap_or_default();
                serde_json::json!({"iteration":offer["iteration"],"potential_before":offer["potential_before"],"source_torque":offer["source_torque"],"trials":trials})
            }).collect::<Vec<_>>()).unwrap_or_default();
            let mut physical=serde_json::Map::new();for key in ["conflicts_retained_as_state","conflict_admission","stored_conflict_energy","issues","kernel","terminal_carboxyl_present","atoms","torsions","sidechain_torsions","retained_internal_displacements","active_source_gram","source_contact_operand","source_contact_operand_readout","source_cycle_surface_readout","native_geometry_evidence","issue_count","geometry_closed","lattice_justification"]{physical.insert(key.into(),state[key].clone());}
            let surface=&state["source_cycle_surface_state"];let mut retained_surface=serde_json::Map::new();for key in ["source_residues","source_frame_paths","source_embedding_weights","left_frame_count","source_mean_bond_length","lift_isometry_signed_residuals","native_internal_operator","retained_matrix","scope"]{retained_surface.insert(key.into(),surface[key].clone());}
            roots.push(serde_json::json!({"object_ref":index,"parents":object["parents"],"mechanism":joint["mechanism"],"source_sequence":joint["source_sequence"],"positions":joint["positions"],"parent_positions":old.map(|p|p["positions"].clone()),"physical_state":physical,"retained_surface":retained_surface,
                "surface_state_equal_to_parent":previous.map(|p|p["source_cycle_surface_state"]==state["source_cycle_surface_state"]),
                "source_boundary_lift":last["source_boundary_lift"],"old_coordinates_verified_before_boundary_preparation":last["old_coordinates_verified_before_boundary_preparation"],"boundary_preparation_admitted":last["boundary_preparation"]["admitted"],"atomic_event_count":events.len(),"parent_atomic_event_prefix_preserved":previous.and_then(|p|p["events"].as_array()).map(|p|events.starts_with(p)),
                "parent_torsions":previous.map(|p|p["torsions"].clone()),"refinement":{"executed":refinement["executed"],"accepted_updates":refinement["accepted_updates"],"iteration_budget":refinement["iteration_budget"],"potential_before":refinement["potential_before"],"potential_after":refinement["potential_after"],"offers":offers}}));
        }
    }
    Some(serde_json::json!({"schema":"supplied-protein-physical-observations/v1","active_molecular_boundary":value["domain_carrier"]["active_molecular_source"]["terminal_carboxyl_boundary"],"operator_artifact_id":value["operator_artifact_id"],"operator_tuple":value["operator_tuple"],"operator_word":value["operator_word"],"arena_objects":arena.len(),"roots":roots,"full_fold_closed":value["full_chain_lattice_fold_closed"],"scope":"read-only current physical observations; full source history, banks and alternatives remain in the referenced complete carrier"}))
}
