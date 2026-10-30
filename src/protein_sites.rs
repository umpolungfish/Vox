//! Source-addressed protocol edges executed by the shared native graph kernel.
use alloc::{collections::BTreeMap, string::{String, ToString}, vec::Vec};
use imasm_core::{check::{self, ClosureState}, classic::Token,
    flow::{flow_values, gate_out}, imasm16_3::Reg16_3};
use crate::phase_word;

#[derive(Clone, Copy)]
pub struct ProgramSpec<'a> {
    pub word: &'a str,
    /// Primitive programs carry their Lean-exported fork/fuse pairs.
    /// Self-mark words use the native protocol builder's pairing operation.
    pub pairs: Option<&'a [(usize, usize)]>,
}

fn register(mask:u8)->Reg16_3 {
    let r=phase_word::runtime_mask_from_native(mask);
    Reg16_3{big_t:r&1!=0,big_f:r&2!=0,small_t:r&4!=0,small_f:r&8!=0}
}
fn native(r:Reg16_3)->u8 {
    phase_word::native_mask_from_runtime(u8::from(r.big_t)|u8::from(r.big_f)<<1|
        u8::from(r.small_t)<<2|u8::from(r.small_f)<<3)
}

pub struct SiteProgram {
    graph:check::Graph,
    word:String,
    pairs:Vec<(usize,usize)>,
    node_states:Vec<Vec<u8>>,
    edge_states:Vec<Vec<u8>>,
    histograms:Vec<[u64;16]>,
}
impl SiteProgram {
    pub fn prepare(spec:ProgramSpec<'_>)->Result<Self,String> {
        if spec.word.is_empty(){return Err("empty native site program".into());}
        let ops:Vec<_>=spec.word.chars().map(|c|Token::parse(&c.to_string())
            .ok_or_else(||format!("unknown native site opcode {c}"))).collect::<Result<_,_>>()?;
        let derived=check::match_pairs(&ops);
        let pairs=spec.pairs.unwrap_or(&derived).to_vec();
        // Do not silently repair a stale or malformed exported source graph.
        let mut declared=pairs.clone();declared.sort_unstable();
        let mut expected=derived;expected.sort_unstable();
        if declared!=expected{return Err("native site source pairings differ from protocol ancestry".into());}
        let graph=check::from_sequence(&ops,&pairs);
        let errors=graph.validate();
        if !errors.is_empty(){return Err(format!("invalid native site graph: {errors:?}"));}
        let mut node_states=Vec::new();let mut edge_states=Vec::new();let mut histograms=Vec::new();
        for seed in 0..16 {
            let (nodes,edges)=flow_values(&graph,register(seed));
            // The kernel returns a bounded iteration. Check the actual returned
            // edge equation before using that state as a fixed-point reading.
            for i in 0..graph.nodes.len(){
                let mut input=Reg16_3::default();
                for (e,&(_,b)) in graph.edges.iter().enumerate(){if b==i{input=input.union(edges[e]);}}
                if input!=nodes[i]{return Err("native site node flow did not converge".into());}
                let mut slot=0;
                for (e,&(a,_)) in graph.edges.iter().enumerate(){if a==i{
                    if edges[e]!=gate_out(graph.nodes[i],input,register(seed),slot,graph.out_degree(i)){
                        return Err("native site edge flow did not converge".into());
                    }slot+=1;
                }}
            }
            let values:Vec<_>=edges.into_iter().map(native).collect();
            let mut histogram=[0;16];for &state in &values{histogram[state as usize]+=1;}
            histograms.push(histogram);node_states.push(nodes.into_iter().map(native).collect());
            edge_states.push(values);
        }
        Ok(Self{graph,word:spec.word.into(),pairs,node_states,edge_states,histograms})
    }
    pub fn histogram(&self,seed:u8)->&[u64;16]{&self.histograms[seed as usize]}
    pub fn edge_count(&self)->usize{self.graph.edges.len()}
    pub fn record(&self)->serde_json::Value {
        let (pairs,fully)=self.graph.frobenius_closures();
        let closure=match self.graph.closure_state(){ClosureState::Closed(_)=>"transformed-reconnected",
            ClosureState::Identity=>"identity-reconnected",ClosureState::Open=>"open",ClosureState::None=>"no-dyad"};
        let nodes:Vec<_>=self.graph.nodes.iter().enumerate().map(|(i,t)|{
            let arity=t.arity();serde_json::json!({"id":i,"opcode":t.code(),
                "arity":[arity.0,arity.1],"in_degree":self.graph.in_degree(i),
                "out_degree":self.graph.out_degree(i)})
        }).collect();
        let mut out_slots=alloc::vec![0;nodes.len()];let mut in_slots=out_slots.clone();
        let edges:Vec<_>=self.graph.edges.iter().enumerate().map(|(id,&(a,b))|{
            let row=serde_json::json!({"id":id,"source":a,"target":b,
                "out_port":out_slots[a],"in_port":in_slots[b]});
            out_slots[a]+=1;in_slots[b]+=1;row
        }).collect();
        serde_json::json!({"word":self.word,"nodes":nodes,"edges":edges,
            "source_pairs":self.pairs,"ancestry_pairs":pairs,"fully_reconnected":fully,
            "closure":closure,"seed_node_states":self.node_states,
            "seed_edge_states":self.edge_states,"seed_histograms":self.histograms,
            "flow_equations_verified":true})
    }
}

/// Graphs are shared by exact source word, while every residue/form/component
/// address remains in the binding tree. Equal edge values never erase ports.
pub struct SiteCarrier {
    pub programs:Vec<SiteProgram>,
    pub bindings:Vec<Vec<Vec<usize>>>,
}
impl SiteCarrier {
    pub fn prepare(sources:&[Vec<Vec<ProgramSpec<'_>>>])->Result<Self,String>{
        let mut programs=Vec::<SiteProgram>::new();let mut ids=BTreeMap::<String,usize>::new();
        let mut bindings=Vec::new();
        for residue in sources{let mut forms=Vec::new();for form in residue{
            let mut parts=Vec::new();for &spec in form{
                let id=if let Some(&id)=ids.get(spec.word){
                    if let Some(pairs)=spec.pairs{
                        let mut actual=pairs.to_vec();actual.sort_unstable();
                        let mut held=programs[id].pairs.clone();held.sort_unstable();
                        if actual!=held{return Err("conflicting graph source pairings for one word".into());}
                    }id
                }else{
                    let program=SiteProgram::prepare(spec)?;let id=programs.len();
                    ids.insert(spec.word.into(),id);programs.push(program);id
                };parts.push(id);
            }forms.push(parts);
        }bindings.push(forms);}
        Ok(Self{programs,bindings})
    }
    pub fn program(&self,residue:usize,form:usize,part:usize)->&SiteProgram{
        &self.programs[self.bindings[residue][form][part]]
    }
    pub fn record(&self)->serde_json::Value{
        serde_json::json!({"schema":"native-protein-port-graphs/v1",
            "engine":"imasm_core::check::from_sequence + flow::flow_values",
            "site":"addressed protocol edge with typed endpoint ports",
            "seed":"banked source-prefix register in native N,T,F,B basis",
            "bindings":self.bindings,"programs":self.programs.iter().map(SiteProgram::record).collect::<Vec<_>>()})
    }
}

#[cfg(test)]
mod tests{
    use super::*;
    #[test]
    fn exported_native_graphs_execute_every_seed_and_preserve_all_ports(){
        let fixture:serde_json::Value=serde_json::from_str(include_str!(
            "../tests/fixtures/native_graph_programs_20261028.json")).unwrap();
        for p in fixture["programs"].as_array().unwrap(){
            let pairs:Vec<_>=p["pairs"].as_array().unwrap().iter().map(|x|
                (x[0].as_u64().unwrap() as usize,x[1].as_u64().unwrap() as usize)).collect();
            let program=SiteProgram::prepare(ProgramSpec{word:p["word"].as_str().unwrap(),pairs:Some(&pairs)}).unwrap();
            for seed in 0..16u8{
                assert_eq!(native(register(seed)),seed);
                assert_eq!(program.histogram(seed).iter().sum::<u64>(),program.edge_count() as u64);
            }
            assert!(program.graph.validate().is_empty());
            assert_eq!(program.graph.frobenius_closures().0.len(),pairs.len());
        }
    }
    #[test]
    fn bypass_arm_changes_the_return_and_duplicate_ports_remain_distinct(){
        let word="⊢∈⊤∋⊣";
        let p=SiteProgram::prepare(ProgramSpec{word,pairs:Some(&[(1,3)])}).unwrap();
        let seed=15;let (_,edges)=flow_values(&p.graph,register(seed));
        let (fuse,_) = flow_values(&p.graph,register(seed));
        assert_eq!(native(fuse[3]),15);
        assert!(p.graph.edges.iter().zip(edges.iter()).any(|(&(a,b),&v)|a==1&&b==3&&native(v)!=0));
        assert_eq!(p.graph.edges.iter().filter(|&&(a,b)|a==1&&b==3).count(),2);
        let mut chain=p.graph.clone();chain.edges.truncate(chain.nodes.len()-1);
        let (unreconnected,_)=flow_values(&chain,register(seed));
        assert_ne!(native(unreconnected[3]),native(fuse[3]));
        assert!(SiteProgram::prepare(ProgramSpec{word,pairs:Some(&[])}).is_err());
    }
}
