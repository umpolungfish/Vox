//! Banked source-word re-entry on covalent walk languages.
//! The finite clock retains a language of complete source walks, not a quotient
//! that identifies their histories. Integer coefficients count distinct walks.
use alloc::{collections::BTreeMap,string::{String,ToString},vec::Vec};
use num_bigint::BigUint;
use imasm_core::{check::Graph,classic::Token,flow::gate_out,imasm16_3::Reg16_3};
use crate::phase_word::{native_mask_from_runtime as native,runtime_mask_from_native as runtime};
type Degrees=BTreeMap<usize,u8>;
fn reg(n:u8)->Reg16_3{let r=runtime(n);Reg16_3{big_t:r&1!=0,big_f:r&2!=0,small_t:r&4!=0,small_f:r&8!=0}}
fn mask(r:Reg16_3)->u8{native(u8::from(r.big_t)|u8::from(r.big_f)<<1|u8::from(r.small_t)<<2|u8::from(r.small_f)<<3)}
fn union(a:&mut Degrees,b:&Degrees){for (&d,&v) in b{*a.entry(d).or_default()|=v;}}
fn word(graph:&Graph,seed:&Degrees,limit:usize)->Result<Degrees,String>{
    let mut edges=alloc::vec![Degrees::new();graph.edges.len()];let mut result=Degrees::new();
    for node in 0..graph.nodes.len(){
        let mut input=Degrees::new();
        for (e,&(from,to)) in graph.edges.iter().enumerate(){if to==node{
            if from>=node{return Err("walk-language graph is not topologically ordered".into());}
            union(&mut input,&edges[e]);
        }}
        let op=graph.nodes[node];
        let transformed=if op==Token::Clink{
            let mut out=Degrees::new();
            for (&a,&ma) in &input{for (&b,&mb) in &input{if a+b<=limit{
                *out.entry(a+b).or_default()|=ma&mb;
            }}}out
        }else if op==Token::Arev{
            input.iter().map(|(&d,&v)|(d,mask(reg(v).invol()))).collect()
        }else{input};
        let mut slot=0;let fan=graph.out_degree(node);
        for (e,&(from,_)) in graph.edges.iter().enumerate(){if from==node{
            edges[e]=if op==Token::Vinit{seed.clone()}else{
                transformed.iter().map(|(&d,&v)|(d,if op==Token::Clink||op==Token::Arev{v}
                    else{mask(gate_out(op,reg(v),reg(0),slot,fan))})).collect()
            };slot+=1;
        }}
        result=transformed;
    }Ok(result)
}

pub fn prepare(graph:&Graph,molecule:&serde_json::Value)->Result<serde_json::Value,String>{
    let atoms=molecule["atoms"].as_array().ok_or("missing atomic re-entry source")?;
    let bonds=molecule["bonds"].as_array().ok_or("missing atomic re-entry bonds")?;
    let n=atoms.len();if n==0{return Err("empty atomic re-entry source".into());}
    let limit=64usize;let seed=Degrees::from([(1,15u8)]);
    let first=word(graph,&seed,limit)?;
    let b_degrees=first.iter().filter_map(|(&d,&v)|if v&8!=0{Some(d)}else{None}).collect::<Vec<_>>();
    if b_degrees.len()!=1||b_degrees[0]<=1{return Err("supplied source has no unique nonlinear B-sector degree".into());}
    let degree=b_degrees[0];let period=degree-1;
    let first_true=*first.iter().filter(|(_,v)|**v&2!=0).map(|(d,_)|d).min().ok_or("source lacks constructive walk sector")?;
    let mut held=seed.clone();let mut iterations=Vec::new();let mut stable=false;
    for iteration in 0..=limit*5{
        let returned=word(graph,&held,limit)?;let mut next=held.clone();union(&mut next,&returned);
        iterations.push(serde_json::json!({"iteration":iteration,"input":held,"returned":returned,"banked_return":next}));
        if next==held{stable=true;break;}held=next;
    }
    if !stable{return Err("walk-language observed prefix did not stabilize".into());}
    for d in 1..=limit{
        let expected=if d==1{15}else{(if d>=first_true{2}else{0})|(if d%period==1{8}else{0})};
        if held.get(&d).copied()!=Some(expected){return Err("regular-language sector formula disagrees with executed source prefix".into());}
    }
    // Prepare one joint atom/clock coefficient register from all source basis
    // atoms. Every bond step changes both the atom coordinate and clock phase.
    let mut directed=Vec::new();
    for (id,bond) in bonds.iter().enumerate(){
        let a=bond[0].as_u64().ok_or("invalid re-entry source bond")? as usize;
        let b=bond[1].as_u64().ok_or("invalid re-entry source bond")? as usize;
        if a>=n||b>=n{return Err("re-entry bond leaves the molecule".into());}
        directed.push((id,a,b,false));directed.push((id,b,a,true));
    }
    let zero=BigUint::from(0u8);let one=BigUint::from(1u8);
    let width=n*period;
    let mut carried=alloc::vec![alloc::vec![zero.clone();width];n];
    for source in 0..n{carried[source][source*period]=one.clone();}
    let mut banks=Vec::new();
    for tick in 1..=degree{
        let mut next=alloc::vec![alloc::vec![zero.clone();width];n];
        for source in 0..n{for &(_,from,to,_) in &directed{for phase in 0..period{
            next[source][to*period+(phase+1)%period]+=&carried[source][from*period+phase];
        }}}
        carried=next;let mut entries=Vec::new();let mut total=zero.clone();
        for source in 0..n{for atom in 0..n{for phase in 0..period{
            let value=&carried[source][atom*period+phase];
            if value!=&zero{
                entries.push(serde_json::json!([source,atom,phase,value.to_string()]));total+=value;
            }
        }}}
        banks.push(serde_json::json!({"tick":tick,"joint_atom_clock_entries":entries,
            "total_source_walk_coefficients":total.to_string()}));
    }
    Ok(serde_json::json!({"operation":"banked native source-word iteration with a full covalent walk language",
        "source_graph":"molecular_source.bonds; each bond has both oriented source letters",
        "single_pass_degree_masks":first,"observed_degree_limit":limit,"observed_prefix_fixed":stable,
        "iterations":iterations,"banked_degree_masks":held,
        "B_sector_source_equation":alloc::format!("B = source union B^{degree}"),
        "derived_B_clock_period":period,"first_constructive_degree":first_true,
        "full_language_tail":"retained by source-labelled covalent automaton and sector predicates; not truncated to the observed prefix",
        "sector_predicates":{"basis":"positive-length source walks",
            "N":"one source bond","F":"one source bond",
            "T":alloc::format!("length 1 or length at least {first_true}"),
            "B":alloc::format!("length congruent to 1 modulo {period}")},
        "source_letters":directed,"joint_register_shape":[n,n,period],
        "executed_joint_atom_clock_banks":banks,
        "coefficient_scope":"formal source walk counts; distinct source histories remain words in the automaton",
        "spatial_fixation_permitted":false}))
}
