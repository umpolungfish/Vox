//! Residue relations valued in the native powerset lattice.
//! CLINK is relational composition: union over shared-middle intersections.
//! This is a carried relation, not a Cartesian conformation.
use alloc::{borrow::ToOwned,string::String,vec::Vec};
use num_bigint::BigUint;
use imasm_core::{check::Graph,classic::Token,flow::gate_out,imasm16_3::Reg16_3};
use crate::phase_word::{native_mask_from_runtime as native,runtime_mask_from_native as runtime};

fn reg(n:u8)->Reg16_3{
    let r=runtime(n);Reg16_3{big_t:r&1!=0,big_f:r&2!=0,small_t:r&4!=0,small_f:r&8!=0}
}
fn mask(r:Reg16_3)->u8{native(u8::from(r.big_t)|u8::from(r.big_f)<<1|
    u8::from(r.small_t)<<2|u8::from(r.small_f)<<3)}
type Relation=Vec<Vec<BigUint>>;
fn empty(n:usize)->Relation{alloc::vec![alloc::vec![BigUint::from(0u8);n];n]}
fn compose(a:&[Vec<BigUint>],b:&[Vec<BigUint>])->Relation{
    let n=a.len();let mut result=empty(n);
    for i in 0..n{for k in 0..n{for j in 0..n{result[i][j]|=&a[i][k]& &b[k][j];}}}
    result
}
fn apply(op:Token,input:&BigUint,seed:&BigUint,slot:usize,fan:usize,groups:usize)->BigUint{
    let mut result=BigUint::from(0u8);
    for group in 0..groups{
        let mut x=0;let mut s=0;
        for member in 0..4{let bit=(4*group+member) as u64;
            if input.bit(bit){x|=1<<member;}if seed.bit(bit){s|=1<<member;}
        }
        let out=mask(gate_out(op,reg(x),reg(s),slot,fan));
        for member in 0..4{if out&(1<<member)!=0{result.set_bit((4*group+member) as u64,true);}}
    }result
}
fn record(relation:&Relation)->serde_json::Value{
    serde_json::json!(relation.iter().map(|row|row.iter().map(|v|v.to_str_radix(16)).collect::<Vec<_>>()).collect::<Vec<_>>())
}

pub fn execute(graph:&Graph,peptide:&serde_json::Value,molecule:&serde_json::Value,
               field:&serde_json::Value)->Result<serde_json::Value,String>{
    let n=peptide["residues"].as_array().ok_or("missing residue relation source")?.len();
    let sites=field["sites"].as_array().ok_or("missing addressed lattice field")?;
    let values=field["initial_field"].as_array().ok_or("missing initial lattice field")?;
    if sites.len()!=values.len(){return Err("residue relation field lost its source addresses".into());}
    // Source program words, primitive-component slots and edge ordinals define
    // the feature universe. The four memberships remain separate at every one.
    let programs=field["source_programs"].as_array().ok_or("missing source program bindings")?;
    let mut keys=alloc::collections::BTreeSet::new();
    for site in sites{
        let p=site["source_program"].as_u64().ok_or("invalid source program")? as usize;
        keys.insert((site["component"].as_u64().ok_or("invalid source component")?,
            programs[p]["word"].as_str().ok_or("missing source program word")?.to_owned(),
            serde_json::to_string(&programs[p]["source_pairs"]).map_err(|e|alloc::format!("{e}"))?,
            site["source_edge"].as_u64().ok_or("invalid source edge")?));
    }
    let features=keys.into_iter().collect::<Vec<_>>();
    let ids=features.iter().cloned().enumerate().map(|(i,key)|(key,i)).collect::<alloc::collections::BTreeMap<_,_>>();
    let mut incidence=alloc::vec![Vec::new();n];let mut support=alloc::vec![BigUint::from(0u8);n];
    for (id,(site,value)) in sites.iter().zip(values).enumerate(){
        let r=site["residue"].as_u64().ok_or("invalid relation residue")? as usize;
        let v=value.as_u64().filter(|v|*v<16).ok_or("invalid relation lattice mask")? as u8;
        if r>=n{return Err("relation site outside source chain".into());}
        let p=site["source_program"].as_u64().unwrap() as usize;
        let key=(site["component"].as_u64().unwrap(),programs[p]["word"].as_str().unwrap().to_owned(),
            serde_json::to_string(&programs[p]["source_pairs"]).map_err(|e|alloc::format!("{e}"))?,
            site["source_edge"].as_u64().unwrap());
        let feature=ids[&key];
        incidence[r].push(id);
        for member in 0..4{if v&(1<<member)!=0{support[r].set_bit((4*feature+member) as u64,true);}}
    }
    let atoms=molecule["atoms"].as_array().ok_or("missing relation atom source")?;
    let bonds=molecule["bonds"].as_array().ok_or("missing relation bond source")?;
    let mut seed=empty(n);let mut seed_sources=Vec::new();
    for (index,bond) in bonds.iter().enumerate(){
        let a=bond[0].as_u64().ok_or("invalid relation bond")? as usize;
        let b=bond[1].as_u64().ok_or("invalid relation bond")? as usize;
        let left=atoms[a]["residue"].as_u64().ok_or("missing source residue")? as usize;
        let right=atoms[b]["residue"].as_u64().ok_or("missing target residue")? as usize;
        if left!=right{
            // Directed source incidence retains N-to-C order, without adding
            // noncovalent contacts or inferring positions from sequence distance.
            seed[left][right]|=&support[left]& &support[right];
            seed_sources.push(serde_json::json!({"source_bond":index,"residues":[left,right],
                "source_atoms":[a,b],"left_field_sites":incidence[left],"right_field_sites":incidence[right]}));
        }
    }
    let empty=empty(n);
    let mut node_relations=alloc::vec![empty.clone();graph.nodes.len()];
    let mut edge_relations=alloc::vec![empty.clone();graph.edges.len()];
    let mut compositions=Vec::new();let mut operations=Vec::new();
    for node in 0..graph.nodes.len(){
        let mut incoming=empty.clone();
        for (edge,&(from,to)) in graph.edges.iter().enumerate(){if to==node{
            if from>=node{return Err("relation execution requires topologically ordered source nodes".into());}
            for i in 0..n{for j in 0..n{incoming[i][j]|=&edge_relations[edge][i][j];}}
        }}
        node_relations[node]=incoming.clone();
        let op=graph.nodes[node];
        let transformed=if op==Token::Clink{
            let result=compose(&incoming,&incoming);
            let mut witnesses=Vec::new();
            for i in 0..n{for k in 0..n{for j in 0..n{
                let common=&incoming[i][k]& &incoming[k][j];
                if common!=BigUint::from(0u8){witnesses.push(serde_json::json!([i,k,j]));}
            }}}
            compositions.push(serde_json::json!({"source_node":node,
                "shared_middle_witnesses":witnesses}));
            result
        }else if op==Token::Arev{
            // Dagger reverses both the relation direction and native lanes.
            let mut result=empty.clone();
            for i in 0..n{for j in 0..n{result[i][j]=apply(op,&incoming[j][i],&seed[i][j],0,1,features.len());}}
            result
        }else{incoming.clone()};
        let mut slot=0;let fan=graph.out_degree(node);
        for (edge,&(from,_)) in graph.edges.iter().enumerate(){if from==node{
            for i in 0..n{for j in 0..n{
                edge_relations[edge][i][j]=if op==Token::Clink||op==Token::Arev{
                    transformed[i][j].clone()
                }else{apply(op,&transformed[i][j],&seed[i][j],slot,fan,features.len())};
            }}slot+=1;
        }}
        operations.push(serde_json::json!({"source_node":node,"opcode":op.code(),
            "incoming_relation_retained":true}));
    }
    let top=(BigUint::from(1u8)<<(4*features.len()))-BigUint::from(1u8);
    let mut identity=empty.clone();for i in 0..n{identity[i][i]=top.clone();}
    let identity_left=compose(&identity,&seed)==seed;
    let identity_right=compose(&seed,&identity)==seed;
    // Three successive source transformations share their actual intermediate
    // residue addresses. No product of residue marginals is constructed.
    let s2=compose(&seed,&seed);
    let associative=compose(&s2,&seed)==compose(&seed,&s2);
    if !identity_left||!identity_right||!associative{return Err("native relation composition laws failed".into());}
    // Share identical relation values without merging their source operation
    // addresses. Every node/edge keeps its own reference in the return.
    let mut pool=Vec::new();let mut pool_ids=alloc::collections::BTreeMap::new();
    let mut intern=|relation:&Relation|{
        if let Some(&id)=pool_ids.get(relation){id}else{
            let id=pool.len();pool.push(record(relation));pool_ids.insert(relation.clone(),id);id
        }
    };
    let node_refs=node_relations.iter().map(&mut intern).collect::<Vec<_>>();
    let edge_refs=edge_relations.iter().map(&mut intern).collect::<Vec<_>>();
    Ok(serde_json::json!({"algebra":"powerset of addressed primitive-edge memberships; union/intersection relation composition",
        "features":features,"feature_bit":"4 * feature_index + native FOUR member index",
        "residue_source_sites":incidence,"source_feature_profiles":support.iter().map(|v|v.to_str_radix(16)).collect::<Vec<_>>(),
        "source_field_role":"primitive slot, source program, source edge and membership retained",
        "seed_relation":record(&seed),"seed_source_bonds":seed_sources,
        "relation_pool":pool,"node_relations":node_refs,"edge_relations":edge_refs,
        "compositions":compositions,"operations":operations,
        "laws":{"left_identity":identity_left,"right_identity":identity_right,"associativity_on_source":associative},
        "spatial_conformation_resolved":false}))
}
