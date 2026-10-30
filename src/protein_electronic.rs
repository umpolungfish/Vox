//! Number-conserving electronic operators on the retained native Fock family.
//! The integral engine supplies evaluated matrix elements. All occupation,
//! exclusion, ordered creation and Hamiltonian actions execute here.
use alloc::{collections::BTreeMap,string::String,vec::Vec};

pub fn create(state:u64,slot:usize)->Option<(u64,f64)>{
    if slot>=64||state&(1u64<<slot)!=0{return None;}
    let phase=if (state&((1u64<<slot)-1)).count_ones()%2==0{1.0}else{-1.0};
    Some((state|(1u64<<slot),phase))
}
pub fn remove(state:u64,slot:usize)->Option<(u64,f64)>{
    if slot>=64||state&(1u64<<slot)==0{return None;}
    let phase=if (state&((1u64<<slot)-1)).count_ones()%2==0{1.0}else{-1.0};
    Some((state&!(1u64<<slot),phase))
}
fn apply(state:u64,ops:&[(bool,usize)])->Option<(u64,f64)>{
    let mut result=(state,1.0);
    for &(addition,slot) in ops{
        let (next,sign)=if addition{create(result.0,slot)?}else{remove(result.0,slot)?};
        result=(next,result.1*sign);
    }Some(result)
}
fn choose_slots(start:usize,slots:usize,left:usize,state:u64,states:&mut Vec<u64>){
    if left==0{states.push(state);return;}
    if slots-start<left{return;}
    for slot in start..=slots-left{choose_slots(slot+1,slots,left-1,state|(1u64<<slot),states);}
}
pub fn states(orbitals:usize,electrons:usize)->Result<Vec<u64>,String>{
    let slots=2*orbitals;
    if slots>64||electrons>slots{return Err("electron source exceeds the indexed native spin slots".into());}
    let mut count=1u128;let rank=electrons.min(slots-electrons);
    for i in 0..rank{count=count*(slots-i) as u128/(i+1) as u128;}
    if count>4096{return Err("dense electronic readout exceeds 4096 states; retain the operator family for contraction".into());}
    let mut states=Vec::new();choose_slots(0,slots,electrons,0,&mut states);
    states.sort_unstable();Ok(states)
}
pub struct Hamiltonian {
    pub states:Vec<u64>,pub matrix:Vec<Vec<f64>>,
    pub one_body_actions:u64,pub two_body_actions:u64,
}
impl Hamiltonian {
    pub fn prepare(orbitals:usize,electrons:usize,h:&[f64],eri:&[f64])->Result<Self,String>{
        if h.len()!=orbitals*orbitals||eri.len()!=orbitals.pow(4){return Err("electronic integral dimensions differ from native orbital source".into());}
        if h.iter().chain(eri).any(|x|!x.is_finite()){return Err("nonfinite electronic matrix element".into());}
        let states=states(orbitals,electrons)?;
        let index:BTreeMap<_,_>=states.iter().enumerate().map(|(i,&s)|(s,i)).collect();
        let mut matrix=alloc::vec![alloc::vec![0.0;states.len()];states.len()];
        let mut one_body_actions=0;let mut two_body_actions=0;
        for (column,&source) in states.iter().enumerate(){
            for p in 0..orbitals{for q in 0..orbitals{
                let coefficient=h[p*orbitals+q];if coefficient==0.0{continue;}
                for spin in 0..2{
                    if let Some((target,sign))=apply(source,&[(false,2*q+spin),(true,2*p+spin)]){
                        matrix[index[&target]][column]+=coefficient*sign;one_body_actions+=1;
                    }
                }
            }}
            for p in 0..orbitals{for q in 0..orbitals{for r in 0..orbitals{for s in 0..orbitals{
                let coefficient=eri[((p*orbitals+q)*orbitals+r)*orbitals+s]/2.0;
                if coefficient==0.0{continue;}
                for spin in 0..2{for other in 0..2{
                    if let Some((target,sign))=apply(source,&[(false,2*q+spin),
                        (false,2*s+other),(true,2*r+other),(true,2*p+spin)]){
                        matrix[index[&target]][column]+=coefficient*sign;two_body_actions+=1;
                    }
                }}
            }}}}
        }
        Ok(Self{states,matrix,one_body_actions,two_body_actions})
    }
    pub fn record(&self)->serde_json::Value{
        let mut residual=0.0f64;
        for i in 0..self.matrix.len(){for j in 0..i{
            residual=residual.max((self.matrix[i][j]-self.matrix[j][i]).abs());
        }}
        serde_json::json!({"schema":"native-electronic-hamiltonian/v1",
            "basis":"ordered exterior products of indexed orbital up/down slots",
            "states":self.states,"hamiltonian":self.matrix,
            "one_body_actions":self.one_body_actions,"two_body_actions":self.two_body_actions,
            "hermitian_residual":residual,
            "retained":"complete number-conserving joint family, ordered phases and evaluated operator"})
    }
}
pub fn from_json(source:&serde_json::Value)->Result<serde_json::Value,String>{
    let n=source["orbitals"].as_u64().ok_or("missing orbital count")? as usize;
    let e=source["electrons"].as_u64().ok_or("missing electron count")? as usize;
    let values=|name:&str|->Result<Vec<f64>,String>{
        source[name].as_array().ok_or_else(||alloc::format!("missing {name}"))?.iter()
            .map(|v|v.as_f64().ok_or_else(||alloc::format!("invalid {name} value"))).collect()
    };
    let mut result=Hamiltonian::prepare(n,e,&values("one_body")?,&values("two_body")?)?.record();
    result["source"]=source.clone();Ok(result)
}

#[cfg(test)]
mod tests{
    use super::*;
    #[test]
    fn creation_annihilation_obey_the_complete_canonical_anticommutator(){
        for state in 0..256{for i in 0..8{for j in 0..8{
            let mut result=BTreeMap::<u64,f64>::new();
            for ops in [&[(true,j),(false,i)][..],&[(false,i),(true,j)][..]]{
                if let Some((target,sign))=apply(state,ops){*result.entry(target).or_default()+=sign;}
            }
            for (target,coefficient) in result{
                assert_eq!(coefficient,if i==j&&target==state{1.0}else{0.0});
            }
        }}}
    }
    #[test]
    fn one_orbital_pair_pays_one_interaction_and_same_spin_has_no_extra_slot(){
        let pair=Hamiltonian::prepare(1,2,&[0.0],&[1.0]).unwrap();
        assert_eq!(pair.states,alloc::vec![3]);assert_eq!(pair.matrix,alloc::vec![alloc::vec![1.0]]);
        assert_eq!(Hamiltonian::prepare(1,1,&[0.0],&[1.0]).unwrap().matrix,
            alloc::vec![alloc::vec![0.0,0.0],alloc::vec![0.0,0.0]]);
        assert!(states(1,3).is_err());
    }
}
