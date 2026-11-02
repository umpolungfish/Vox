//! Execute the supplied enclosing peptide operator with its declared wiring.
use alloc::{string::{String,ToString},vec::Vec};
use imasm_core::{check::Graph,classic::Token,flow::{flow_values,gate_out},imasm16_3::Reg16_3};

/// Lift the declared graph to the whole addressed residue field. Each field
/// coordinate retains its source graph edge; no marginal distributions are made.
fn lattice_field(graph:&Graph,peptide:&serde_json::Value,seed:u8)->Result<serde_json::Value,String>{
    use crate::{phase_word,protein_sites::{SiteProgram,ProgramSpec}};
    let residues=peptide["residues"].as_array().ok_or("missing lattice residues")?;
    let native_seed=phase_word::native_mask_from_runtime(seed);
    let mut programs=Vec::new();let mut ids=alloc::collections::BTreeMap::new();
    let mut bindings=Vec::new();
    let mut sites=Vec::new();let mut initial=Vec::new();
    for (residue,r) in residues.iter().enumerate(){
        let mut form_bindings=Vec::new();
        for (form,f) in r["forms"].as_array().ok_or("missing lattice forms")?.iter().enumerate(){
            let mut component_bindings=Vec::new();
            for (component,c) in f["native_components"].as_array().ok_or("missing lattice components")?.iter().enumerate(){
                let word=c["word"].as_str().ok_or("missing lattice source word")?;
                let pairs=c["pairs"].as_array().map(|rows|rows.iter().map(|row|{
                    Ok((row[0].as_u64().ok_or("invalid source fork")? as usize,
                        row[1].as_u64().ok_or("invalid source fuse")? as usize))
                }).collect::<Result<Vec<_>,String>>()).transpose()?;
                let key=alloc::format!("{}:{:?}",word,pairs);
                let program=if let Some(&id)=ids.get(&key){id}else{
                    let p=SiteProgram::prepare(ProgramSpec{word,pairs:pairs.as_deref()})?;
                    let id=programs.len();programs.push(p);ids.insert(key,id);id
                };
                component_bindings.push(program);
                for (edge,&state) in programs[program].edge_states(native_seed).iter().enumerate(){
                    sites.push(serde_json::json!({"residue":residue,"form":form,
                        "component":component,"source_program":program,"source_edge":edge}));
                    initial.push(state);
                }
            }
            form_bindings.push(component_bindings);
        }
        bindings.push(form_bindings);
    }
    // Carry the source register through ordered residues and primitive
    // components. A DAG retains every form alternative and the shared prefix;
    // no independent residue probabilities or product rows are introduced.
    let mut source_steps=Vec::new();let mut reachable=alloc::vec![false;16];
    reachable[native_seed as usize]=true;
    for (residue,forms) in bindings.iter().enumerate(){
        let mut next_reachable=alloc::vec![false;16];
        for input in 0..16u8{for (form,components) in forms.iter().enumerate(){
            let mut carried=input;let mut actions=Vec::new();
            for (component,&program) in components.iter().enumerate(){
                let before=carried;carried=programs[program].returned_state(carried);
                actions.push(serde_json::json!({"component":component,"program":program,
                    "input_native_register":before,"returned_native_register":carried,
                    "port_state_reference":[program,before]}));
            }
            let active=reachable[input as usize];if active{next_reachable[carried as usize]=true;}
            source_steps.push(serde_json::json!({"residue":residue,"form":form,
                "input_node":[residue,input],"output_node":[residue+1,carried],
                "reachable_from_source_seed":active,"actions":actions}));
        }}
        reachable=next_reachable;
    }
    let mut node_fields=alloc::vec![Vec::with_capacity(sites.len());graph.nodes.len()];
    let mut edge_fields=alloc::vec![Vec::with_capacity(sites.len());graph.edges.len()];
    // All sixteen transfer actions are computed once, then applied to every
    // addressed coordinate of this same field without merging equal sources.
    let mut transfers=Vec::new();
    for state in 0..16 {
        let r=phase_word::runtime_mask_from_native(state);
        let input=Reg16_3{big_t:r&1!=0,big_f:r&2!=0,small_t:r&4!=0,small_f:r&8!=0};
        let (nodes,edges)=flow_values(graph,input);
        for i in 0..graph.nodes.len(){
            let mut incoming=Reg16_3::default();
            for (e,&(_,to)) in graph.edges.iter().enumerate(){if to==i{incoming=incoming.union(edges[e]);}}
            if incoming!=nodes[i]{return Err("lifted lattice node equation failed".into());}
            let mut slot=0;
            for (e,&(from,_)) in graph.edges.iter().enumerate(){if from==i{
                if edges[e]!=gate_out(graph.nodes[i],incoming,input,slot,graph.out_degree(i)){
                    return Err("lifted lattice edge equation failed".into());
                }slot+=1;
            }}
        }
        transfers.push((nodes,edges));
    }
    for &state in &initial{
        let (nodes,edges)=&transfers[state as usize];
        for (field,&value) in node_fields.iter_mut().zip(nodes){field.push(phase_word::native_mask_from_runtime(mask(value)));}
        for (field,&value) in edge_fields.iter_mut().zip(edges){field.push(phase_word::native_mask_from_runtime(mask(value)));}
    }
    let changed_edges=edge_fields.iter().filter(|field|**field!=initial).count();
    Ok(serde_json::json!({"operation":"pointwise lift of the supplied explicit graph to one source-addressed lattice field",
        "basis":"native N,T,F,B membership masks","source_native_seed":native_seed,
        "source_programs":programs.iter().map(SiteProgram::record).collect::<Vec<_>>(),
        "ordered_source_carrier":{"representation":"all form alternatives on retained source-register prefix DAG",
            "source_root":[0,native_seed],"steps":source_steps,
            "reachable_terminal_registers":(0..16).filter(|&r|reachable[r]).collect::<Vec<_>>(),
            "history_role":"incoming alternatives and full component port references retained"},
        "sites":sites,"initial_field":initial,"node_fields":node_fields,"edge_fields":edge_fields,
        "changed_edge_fields":changed_edges,"flow_equations_verified_for_all_inputs":true,
        "source_addresses_retained":true,"cartesian_readout_resolved":false}))
}

fn mask(r:Reg16_3)->u8{u8::from(r.big_t)|u8::from(r.big_f)<<1|
    u8::from(r.small_t)<<2|u8::from(r.small_f)<<3}

fn propagate_source(peptide:&serde_json::Value)->Result<serde_json::Value,String>{
    let residues=peptide["residues"].as_array().ok_or("unfolded chain lacks residues")?;
    let chain=residues.iter().map(|r|r["aa"].as_str().ok_or("residue lacks identity"))
        .collect::<Result<Vec<_>,_>>()?;
    let molecule=crate::peptide_closure::molecular_constraint_source(&chain);
    let n=molecule["atoms"].as_array().unwrap().len();
    let mut upper=alloc::vec![alloc::vec![u64::MAX/4;n];n];
    for i in 0..n{upper[i][i]=0;}
    for bond in molecule["bonds"].as_array().unwrap(){
        let (i,j,d)=(bond[0].as_u64().unwrap() as usize,bond[1].as_u64().unwrap() as usize,bond[2].as_u64().unwrap());
        upper[i][j]=upper[i][j].min(d);upper[j][i]=upper[j][i].min(d);
    }
    let mut changes=0usize;
    for k in 0..n{for i in 0..n{for j in 0..n{
        let bound=upper[i][k]+upper[k][j];
        if bound<upper[i][j]{upper[i][j]=bound;changes+=1;}
    }}}
    if upper.iter().any(|row|row.iter().any(|&d|d==u64::MAX/4)){
        return Err("source propagation leaves a disconnected molecular component".into());
    }
    Ok(serde_json::json!({"molecular_source":molecule,"distance_upper_bounds":upper,
        "propagated_bound_updates":changes,"operation":"directed molecular path-bound propagation",
        "noncovalent_contacts":[],"source_atoms_retained":n,
        "cartesian_conformation_resolved":false}))
}

pub fn prepare(operator:&serde_json::Value,peptide:serde_json::Value)->Result<serde_json::Value,String>{
    let word=operator["glyph_word"].as_str().ok_or("operator lacks its source word")?;
    let steps=operator["phases"]["phase_4"]["steps"].as_array().ok_or("operator lacks domain steps")?;
    let mut graph=Graph::new();let glyphs=word.chars().collect::<Vec<_>>();
    if steps.len()!=glyphs.len(){return Err("operator domain steps do not cover its word".into());}
    for (index,&glyph) in glyphs.iter().enumerate(){
        if steps[index]["opcode"].as_str()!=Some(glyph.to_string().as_str()){
            return Err("operator domain action differs from its source opcode".into());
        }
        graph.add(Token::parse(&glyph.to_string()).ok_or("invalid operator opcode")?);
    }
    let wires=operator["wiring_check"]["wires"].as_array().ok_or("operator lacks explicit source wires")?;
    for wire in wires{
        let from=wire[0].as_u64().ok_or("invalid source wire")? as usize;
        let to=wire[2].as_u64().ok_or("invalid target wire")? as usize;
        if from>=glyphs.len()||to>=glyphs.len(){return Err("operator wire leaves its source graph".into());}
        graph.connect(from,to);
    }
    let errors=graph.validate();if !errors.is_empty(){return Err(alloc::format!("operator wiring invalid: {errors:?}"));}
    let seed=operator["wiring_check"]["seed"].as_u64().filter(|x|*x<16).ok_or("operator lacks its seed")? as u8;
    let register=Reg16_3{big_t:seed&1!=0,big_f:seed&2!=0,small_t:seed&4!=0,small_f:seed&8!=0};
    let (nodes,edges)=flow_values(&graph,register);
    for i in 0..graph.nodes.len(){
        let mut input=Reg16_3::default();
        for (e,&(_,to)) in graph.edges.iter().enumerate(){if to==i{input=input.union(edges[e]);}}
        if input!=nodes[i]{return Err("operator node flow did not converge".into());}
        let mut slot=0;
        for (e,&(from,_)) in graph.edges.iter().enumerate(){if from==i{
            if edges[e]!=gate_out(graph.nodes[i],input,register,slot,graph.out_degree(i)){
                return Err("operator edge flow did not converge".into());
            }slot+=1;
        }}
    }
    let (pairs,reconnected)=graph.frobenius_closures();
    let forward=steps.iter().position(|s|s["opcode"]=="≻").ok_or("operator lacks forward propagation")?;
    let mut propagated=propagate_source(&peptide)?;
    propagated["native_lattice_field"]=lattice_field(&graph,&peptide,seed)?;
    propagated["correlated_source_paths"]=crate::peptide_paths::execute(&graph,
        &propagated["native_lattice_field"])?;
    propagated["contact_coordinate_bridges"]=crate::peptide_ccr::contact_compositions(
        &propagated["molecular_source"],&propagated["correlated_source_paths"])?;
    propagated["atomic_path_reentry"]=crate::peptide_reentry::prepare(&graph,&propagated["molecular_source"])?;
    propagated["source_phase_return"]=crate::peptide_phase::prepare(&propagated["atomic_path_reentry"])?;
    propagated["prepared_canonical_bridge"]=crate::peptide_ccr::prepare(&propagated["molecular_source"])?;
    let mut split_record=serde_json::Value::Null;
    let mut next_actions=Vec::new();
    if glyphs.get(forward+1)==Some(&'∈'){
        let fork=forward+1;let mut arms=Vec::new();let mut recovered=0u8;
        for (edge,&(from,to)) in graph.edges.iter().enumerate(){if from==fork{
            let register=mask(edges[edge]);recovered|=register;
            arms.push(serde_json::json!({"source_edge":edge,"source_port":wires[edge][1],
                "runtime_register":register,"entry_node":to,
                "carrier_reference":"domain_carrier","domain_verdict":"pending"}));
            next_actions.push(steps[to].clone());
        }}
        if recovered!=mask(nodes[fork]){return Err("source ensemble split lost its input register".into());}
        split_record=serde_json::json!({"source_node":fork,"arms":arms,
            "input_runtime_register":mask(nodes[fork]),"partition_recovers_input":true,
            "shared_source_carrier":true});
    }
    Ok(serde_json::json!({"schema":"supplied-peptide-folding-operator/v1",
        "operator_artifact_id":operator["artifact_id"],"operator_tuple":operator["grounded_tuple"],
        "operator_word":word,"operator_boundary":operator["phases"]["phase_0"]["boundary_condition"],
        "source_peptide":peptide,"domain_steps":steps,"source_wires":wires,
        "source_seed":seed,"node_registers":nodes.into_iter().map(mask).collect::<Vec<_>>(),
        "edge_registers":edges.into_iter().map(mask).collect::<Vec<_>>(),
        "ancestry_pairs":pairs,"fully_reconnected":reconnected,"native_flow_equations_verified":true,
        "domain_carrier":propagated,"executed_forward_node":forward,
        "ensemble_split":split_record,"next_domain_actions":next_actions,
        "status":"source lattice, coordinate bridges and coherent re-entry executed; spatial closure pending",
        "next_domain_action":steps.get(forward+2),
        "physical_fixation_permitted":false,"full_chain_lattice_fold_closed":false}))
}
