//! Retained spin-sector electronic operator with exact single/double connections.
//! No magnitude threshold removes an evaluated integral or a resulting edge.
use alloc::{collections::BTreeMap, string::String, vec::Vec};
use crate::protein_electronic::{create, remove};

fn combinations(n: usize, left: usize, start: usize, bits: u64, out: &mut Vec<u64>) {
    if left == 0 { out.push(bits); return; }
    if n-start < left { return; }
    for p in start..=n-left { combinations(n,left-1,p+1,bits|(1<<p),out); }
}
fn action(mut state: u64, ops: &[(bool,usize)]) -> Option<(u64,f64)> {
    let mut phase=1.0;
    for &(addition,slot) in ops {
        let (next,sign)=if addition { create(state,slot)? } else { remove(state,slot)? };
        state=next; phase*=sign;
    }
    Some((state,phase))
}
fn choose(n:usize,k:usize)->u128 {
    if k>n { return 0; }
    let mut result=1;
    for i in 0..k.min(n-k) { result=result*(n-i) as u128/(i+1) as u128; }
    result
}
pub struct SectorOperator {
    pub orbitals:usize, pub up:usize, pub down:usize, pub states:Vec<u64>,
    pub diagonal:Vec<f64>, pub edges:Vec<(usize,usize,f64)>, pub applications:u64,
}
impl SectorOperator {
    pub fn prepare(n:usize,up:usize,down:usize,h:&[f64],eri:&[f64]) -> Result<Self,String> {
        if n>32||up>n||down>n { return Err("invalid native spin-sector source".into()); }
        if h.len()!=n*n||eri.len()!=n.pow(4)||h.iter().chain(eri).any(|v|!v.is_finite()) {
            return Err("invalid evaluated integral bank".into());
        }
        let count=choose(n,up)*choose(n,down);
        if count>1_000_000 { return Err(alloc::format!("retained spin sector has {count} states; explicit edge bank boundary is 1000000")); }
        let mut alpha=Vec::new(); let mut beta=Vec::new();
        combinations(n,up,0,0,&mut alpha); combinations(n,down,0,0,&mut beta);
        let mut states=Vec::with_capacity(count as usize);
        for a in alpha { for &b in &beta {
            let mut state=0;
            for p in 0..n { state|=((a>>p)&1)<<(2*p); state|=((b>>p)&1)<<(2*p+1); }
            states.push(state);
        }}
        states.sort_unstable();
        let lookup:BTreeMap<_,_>=states.iter().enumerate().map(|(i,&s)|(s,i)).collect();
        let g=|p:usize,q:usize,r:usize,s:usize|eri[((p*n+q)*n+r)*n+s];
        // Slater connections use the symmetries of real Coulomb integrals.
        // Reject a source outside that domain rather than silently symmetrize it.
        let scale=h.iter().chain(eri).fold(1.0f64,|a,b|a.max(b.abs()));
        let tolerance=f64::EPSILON*(n.pow(4)+n*n) as f64*scale;
        for p in 0..n { for q in 0..n {
            if (h[p*n+q]-h[q*n+p]).abs()>tolerance { return Err("one-body bank is not real symmetric".into()); }
            for r in 0..n { for s in 0..n {
                let value=g(p,q,r,s);
                if [(value-g(q,p,r,s)).abs(),(value-g(p,q,s,r)).abs(),(value-g(r,s,p,q)).abs()].iter().any(|v|*v>tolerance) {
                    return Err("two-body bank is outside real Coulomb symmetry".into());
                }
            }}
        }}
        let mut diagonal=Vec::with_capacity(states.len()); let mut edges=Vec::new();
        for (column,&source) in states.iter().enumerate() {
            let occupied:Vec<_>=(0..2*n).filter(|p|source&(1<<p)!=0).collect();
            let vacant:Vec<_>=(0..2*n).filter(|p|source&(1<<p)==0).collect();
            let mut value=0.0;
            for &i in &occupied {
                value+=h[(i/2)*n+i/2];
                for &j in &occupied {
                    value+=0.5*(g(i/2,i/2,j/2,j/2)-if i%2==j%2 {g(i/2,j/2,j/2,i/2)} else {0.0});
                }
            }
            diagonal.push(value);
            for &q in &occupied { for &p in &vacant {
                if p%2!=q%2 {continue;}
                let (target,phase)=action(source,&[(false,q),(true,p)]).unwrap();
                let row=lookup[&target]; if row<=column {continue;}
                let mut coefficient=h[(p/2)*n+q/2];
                for &i in &occupied { if i!=q {
                    coefficient+=g(p/2,q/2,i/2,i/2)-if i%2==p%2 {g(p/2,i/2,i/2,q/2)} else {0.0};
                }}
                if coefficient!=0.0 {edges.push((row,column,phase*coefficient));}
            }}
            for (qi,&q) in occupied.iter().enumerate() { for &s in &occupied[qi+1..] {
                for (pi,&p) in vacant.iter().enumerate() { for &r in &vacant[pi+1..] {
                    let direct=p%2==q%2&&r%2==s%2;
                    let exchange=p%2==s%2&&r%2==q%2;
                    if !direct&&!exchange {continue;}
                    let coefficient=(if direct {g(p/2,q/2,r/2,s/2)} else {0.0})
                        -(if exchange {g(p/2,s/2,r/2,q/2)} else {0.0});
                    if coefficient==0.0 {continue;}
                    let (target,phase)=action(source,&[(false,q),(false,s),(true,r),(true,p)]).unwrap();
                    let row=lookup[&target]; if row>column {edges.push((row,column,phase*coefficient));}
                }}
            }}
            if edges.len()>20_000_000 {return Err("retained electronic edge bank exceeds 20000000 edges".into());}
        }
        Ok(Self{orbitals:n,up,down,states,diagonal,edges,applications:0})
    }
    pub fn apply(&mut self,vector:&[f64])->Result<Vec<f64>,String> {
        if vector.len()!=self.states.len()||vector.iter().any(|x|!x.is_finite()) {
            return Err("invalid retained electronic vector".into());
        }
        let mut out:Vec<_>=self.diagonal.iter().zip(vector).map(|(d,v)|d*v).collect();
        for &(row,column,value) in &self.edges {out[row]+=value*vector[column];out[column]+=value*vector[row];}
        self.applications+=1;Ok(out)
    }
    pub fn record(&self)->serde_json::Value {
        serde_json::json!({"schema":"native-electronic-spin-sector/v1","orbitals":self.orbitals,
            "up":self.up,"down":self.down,"states":self.states,"diagonal":self.diagonal,
            "undirected_edges":self.edges.len(),"applications":self.applications,
            "retained":"complete fixed-up/down family and every nonzero single/double connection",
            "other_spin_sectors":"invariant direct-sum siblings; not enumerated by this sector return"})
    }
    pub fn from_json(source:&serde_json::Value)->Result<Self,String> {
        let integer=|key:&str|source[key].as_u64().map(|x|x as usize).ok_or_else(||alloc::format!("missing {key}"));
        let values=|key:&str|->Result<Vec<f64>,String> {source[key].as_array().ok_or_else(||alloc::format!("missing {key}"))?.iter()
            .map(|x|x.as_f64().ok_or_else(||alloc::format!("invalid {key}"))).collect()};
        Self::prepare(integer("orbitals")?,integer("up")?,integer("down")?,&values("one_body")?,&values("two_body")?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sparse_connections_equal_ordered_creation_on_every_spin_sector() {
        let n=4;let mut h=alloc::vec![0.0;n*n];let mut eri=alloc::vec![0.0;n.pow(4)];
        for p in 0..n {for q in 0..n {
            h[p*n+q]=(p+q+1) as f64/13.0;
            for r in 0..n {for s in 0..n {
                eri[((p*n+q)*n+r)*n+s]=((p+q+1)*(r+s+1)) as f64/37.0;
            }}
        }}
        for electrons in 0..=2*n {
            let dense=crate::protein_electronic::Hamiltonian::prepare(n,electrons,&h,&eri).unwrap();
            let lookup:BTreeMap<_,_>=dense.states.iter().enumerate().map(|(i,&s)|(s,i)).collect();
            for up in 0..=n {let down=electrons.saturating_sub(up);if up>electrons||down>n {continue;}
                let mut sparse=SectorOperator::prepare(n,up,down,&h,&eri).unwrap();
                for column in 0..sparse.states.len() {
                    let mut vector=alloc::vec![0.0;sparse.states.len()];vector[column]=1.0;
                    let out=sparse.apply(&vector).unwrap();
                    for (row,&value) in out.iter().enumerate() {
                        assert!((value-dense.matrix[lookup[&sparse.states[row]]][lookup[&sparse.states[column]]]).abs()<1e-12);
                    }
                }
            }
        }
    }
}
