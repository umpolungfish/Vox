//! Correlated source execution paths under the supplied enclosing graph.
//! Composition joins the same carried-register boundary, retaining each path.
use alloc::{collections::BTreeMap,string::String,vec::Vec};
use imasm_core::{check::Graph,classic::Token,flow::gate_out,imasm16_3::Reg16_3};
use crate::phase_word::{native_mask_from_runtime as native,runtime_mask_from_native as runtime};

type Boundary=(usize,u8);
#[derive(Clone,Debug,PartialEq,Eq,PartialOrd,Ord)]
struct Path{start:Boundary,end:Boundary,steps:Vec<(usize,bool)>}
type Family=BTreeMap<Path,u8>;
fn reg(n:u8)->Reg16_3{let r=runtime(n);Reg16_3{big_t:r&1!=0,big_f:r&2!=0,small_t:r&4!=0,small_f:r&8!=0}}
fn mask(r:Reg16_3)->u8{native(u8::from(r.big_t)|u8::from(r.big_f)<<1|u8::from(r.small_t)<<2|u8::from(r.small_f)<<3)}
fn add(f:&mut Family,path:Path,value:u8){*f.entry(path).or_default()|=value;}
fn record(f:&Family)->serde_json::Value{
    serde_json::json!(f.iter().map(|(p,&value)|serde_json::json!({"start":p.start,"end":p.end,
        "source_steps":p.steps,"native_register":value})).collect::<Vec<_>>())
}
fn compose(f:&Family)->(Family,Vec<[usize;3]>){
    let rows=f.iter().collect::<Vec<_>>();
    let mut starts=BTreeMap::<Boundary,Vec<usize>>::new();
    for (i,(path,_)) in rows.iter().enumerate(){starts.entry(path.start).or_default().push(i);}
    let mut out=Family::new();let mut pending=Vec::new();
    for (left,(a,ma)) in rows.iter().enumerate(){
        if let Some(rights)=starts.get(&a.end){for &right in rights{
            let (b,mb)=rows[right];let mut steps=a.steps.clone();steps.extend_from_slice(&b.steps);
            let path=Path{start:a.start,end:b.end,steps};
            // A held path with zero register evidence remains a path. It is
            // distinct from an absent successor and can be engaged later.
            add(&mut out,path.clone(),**ma & *mb);pending.push((left,right,path));
        }}
    }
    let ids=out.keys().cloned().enumerate().map(|(i,p)|(p,i)).collect::<BTreeMap<_,_>>();
    let witnesses=pending.into_iter().map(|(a,b,p)|[a,b,ids[&p]]).collect();
    (out,witnesses)
}

#[derive(Clone)]
pub(crate) struct BankedPathObject { current:Family, versions:Vec<serde_json::Value> }
impl BankedPathObject {
    pub fn from_source(field:&serde_json::Value)->Result<Self,String>{
        let mut current=Family::new();
        let rows=field["ordered_source_carrier"]["steps"].as_array().ok_or("missing source path template")?;
        for (id,row) in rows.iter().enumerate(){if row["reachable_from_source_seed"]==true{
            let parse=|v:&serde_json::Value|->Result<Boundary,String>{Ok((v[0].as_u64().ok_or("invalid source boundary")? as usize,
                v[1].as_u64().filter(|v|*v<16).ok_or("invalid source register")? as u8))};
            let start=parse(&row["input_node"])?;let end=parse(&row["output_node"])?;
            add(&mut current,Path{start,end,steps:alloc::vec![(id,false)]},start.1);
        }}
        let initial=serde_json::json!({"version":0,"operation":"source preparation","paths":record(&current)});
        Ok(Self{current,versions:alloc::vec![initial]})
    }
    pub fn compose_at(&mut self,source_step:usize){
        let (next,witnesses)=compose(&self.current);self.current=next;
        self.versions.push(serde_json::json!({"version":self.versions.len(),"source_step":source_step,
            "operation":"CLINK composition on the same source path object",
            "paths":record(&self.current),"shared_boundary_witnesses":witnesses}));
    }
    pub fn propagate_at(&mut self,source_step:usize,source:&Self)->Result<(),String>{
        let mut rounds=0;let mut witness_counts=Vec::new();
        // Source boundaries increase by residue. This is finite ordered source
        // propagation, retaining the exact path word and all zero-evidence paths.
        if source.current.keys().any(|p|p.end.0<=p.start.0){return Err("source propagation requires its ordered residue DAG".into());}
        loop{
            let mut next=self.current.clone();let mut witnesses=0;
            for (a,&ma) in &self.current{for (b,&mb) in &source.current{
                if a.end==b.start{
                    let mut steps=a.steps.clone();steps.extend_from_slice(&b.steps);
                    add(&mut next,Path{start:a.start,end:b.end,steps},ma&mb);witnesses+=1;
                }
            }}
            if next==self.current{break;}
            self.current=next;rounds+=1;witness_counts.push(witnesses);
        }
        self.versions.push(serde_json::json!({"version":self.versions.len(),"source_step":source_step,
            "operation":"AFWD exact ordered source-path propagation to finite closure","rounds":rounds,
            "extension_witness_counts":witness_counts,"paths":record(&self.current)}));
        Ok(())
    }
    pub fn version(&self)->usize{self.versions.len()-1}
    pub fn identity_from_source(field:&serde_json::Value)->Result<Self,String>{
        let root=&field["ordered_source_carrier"]["source_root"];
        let boundary=(root[0].as_u64().ok_or("missing source unit boundary")? as usize,
            root[1].as_u64().ok_or("missing source unit register")? as u8);
        let mut current=Family::new();add(&mut current,Path{start:boundary,end:boundary,steps:Vec::new()},15);
        Ok(Self{versions:alloc::vec![serde_json::json!({"version":0,"operation":"identity continuation on the retained source boundary",
            "paths":record(&current)})],current})
    }
    pub fn fuse_sources(sources:&[&Self],source_step:usize)->Self{
        let mut current=Family::new();
        for source in sources{for (path,&value) in &source.current{add(&mut current,path.clone(),value);}}
        Self{versions:alloc::vec![serde_json::json!({"version":0,"operation":"source FFUSE union with parent references retained",
            "source_step":source_step,"paths":record(&current)})],current}
    }
    pub fn complete_chain_witnesses(&self,root:(usize,u8),residues:usize)->serde_json::Value{
        let mut rows=Vec::new();
        for (path,&value) in &self.current{
            if path.start!=root||path.end.0!=residues||path.steps.len()!=residues{continue;}
            let cuts=(1..path.steps.len()).flat_map(|a|((a+1)..path.steps.len()).map(move|b|[a,b])).collect::<Vec<_>>();
            rows.push(serde_json::json!({"start":path.start,"end":path.end,"source_steps":path.steps,
                "native_register":value,"three_unit_path_cuts":cuts,
                "scope":"complete ordered source-word witness; physical residue contacts remain unresolved"}));
        }
        serde_json::json!(rows)
    }
    pub fn path_count(&self)->usize{self.current.len()}
    pub fn report(&self)->serde_json::Value{serde_json::json!({"versions":self.versions,
        "current_version":self.version(),"current_paths":self.current.len(),
        "source_role":"whole source path/contact-map hypothesis object; spatial conformation remains unresolved"})}
}

pub fn execute(graph:&Graph,field:&serde_json::Value)->Result<serde_json::Value,String>{
    let dag=&field["ordered_source_carrier"];
    let steps=dag["steps"].as_array().ok_or("missing correlated source carrier")?;
    let mut seed=Family::new();
    for (id,step) in steps.iter().enumerate(){if step["reachable_from_source_seed"]==true{
        let boundary=|v:&serde_json::Value|->Result<Boundary,String>{
            Ok((v[0].as_u64().ok_or("invalid path boundary")? as usize,
                v[1].as_u64().filter(|r|*r<16).ok_or("invalid path register")? as u8))
        };
        let start=boundary(&step["input_node"])?;let end=boundary(&step["output_node"])?;
        add(&mut seed,Path{start,end,steps:alloc::vec![(id,false)]},start.1);
    }}
    let mut nodes=alloc::vec![Family::new();graph.nodes.len()];
    let mut edges=alloc::vec![Family::new();graph.edges.len()];
    let mut compositions=Vec::new();let mut reversals=Vec::new();
    for node in 0..graph.nodes.len(){
        let mut incoming=Family::new();
        for (edge,&(from,to)) in graph.edges.iter().enumerate(){if to==node{
            if from>=node{return Err("source path graph is not topologically ordered".into());}
            for (path,&value) in &edges[edge]{add(&mut incoming,path.clone(),value);}
        }}
        nodes[node]=incoming.clone();let op=graph.nodes[node];
        let transformed=if op==Token::Clink{
            let (out,witnesses)=compose(&incoming);
            compositions.push(serde_json::json!({"source_node":node,"input_paths":incoming.len(),
                "output_paths":out.len(),"output_edge":graph.edges.iter().position(|&(from,_)|from==node),
                "shared_boundary_witnesses":witnesses}));out
        }else if op==Token::Arev{
            let mut out=Family::new();
            for (path,&value) in &incoming{
                let start=(path.end.0,mask(reg(path.end.1).invol()));
                let end=(path.start.0,mask(reg(path.start.1).invol()));
                let steps=path.steps.iter().rev().map(|&(id,reverse)|(id,!reverse)).collect();
                add(&mut out,Path{start,end,steps},mask(reg(value).invol()));
            }
            reversals.push(serde_json::json!({"source_node":node,"paths":out.len()}));out
        }else{incoming};
        let mut slot=0;let fan=graph.out_degree(node);
        for (edge,&(from,_)) in graph.edges.iter().enumerate(){if from==node{
            if op==Token::Vinit{edges[edge]=seed.clone();}else{
                for (path,&value) in &transformed{
                    let returned=if op==Token::Clink||op==Token::Arev{value}
                        else{mask(gate_out(op,reg(value),reg(0),slot,fan))};
                    add(&mut edges[edge],path.clone(),returned);
                }
            }slot+=1;
        }}
    }
    let mut pool=Vec::new();let mut pool_ids=BTreeMap::new();
    let mut intern=|f:&Family|{
        let key=f.iter().map(|(p,&v)|(p.clone(),v)).collect::<Vec<_>>();
        if let Some(&id)=pool_ids.get(&key){id}else{let id=pool.len();pool.push(record(f));pool_ids.insert(key,id);id}
    };
    let node_refs=nodes.iter().map(&mut intern).collect::<Vec<_>>();
    let edge_refs=edges.iter().map(&mut intern).collect::<Vec<_>>();
    Ok(serde_json::json!({"algebra":"source-path composition over the native lattice; matching carried boundary required",
        "source":"native_lattice_field.ordered_source_carrier.steps",
        "seed_paths":record(&seed),"path_pool":pool,"node_paths":node_refs,"edge_paths":edge_refs,
        "compositions":compositions,"reversals":reversals,
        "zero_evidence_paths_retained":true,"spatial_conformation_resolved":false}))
}
