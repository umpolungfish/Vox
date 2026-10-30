//! Full fermion annihilation Grams and their one-/two-body observations.
use alloc::{collections::BTreeMap,string::String,vec::Vec};
use crate::protein_electronic::{remove,states};
pub type Complex=[f64;2];
fn add(a:&mut Complex,b:Complex){a[0]+=b[0];a[1]+=b[1];}
fn scale(a:Complex,s:f64)->Complex{[a[0]*s,a[1]*s]}
fn outer(a:Complex,b:Complex)->Complex{[a[0]*b[0]+a[1]*b[1],a[1]*b[0]-a[0]*b[1]]}
pub struct PairDensity {
    pub orbitals:usize,pub electrons:usize,pub norm:f64,pub pair_states:Vec<u64>,
    pub single:Vec<Complex>,pub pair:Vec<Complex>,pub annihilation_actions:u64,
}
fn gram(source:&[u64],amplitudes:&[Complex],basis:&[u64])->(Vec<Complex>,u64) {
    let mut residuals=BTreeMap::<u64,Vec<(usize,Complex)>>::new();let mut actions=0;
    for (&state,&amplitude) in source.iter().zip(amplitudes) {
        if amplitude==[0.,0.] {continue;}
        for (column,&mask) in basis.iter().enumerate() {
            if state&mask!=mask {continue;}
            let mut target=state;let mut sign=1.;let mut bits=mask;
            while bits!=0 {
                let slot=bits.trailing_zeros() as usize;bits&=bits-1;
                let (next,phase)=remove(target,slot).unwrap();target=next;sign*=phase;
            }
            residuals.entry(target).or_default().push((column,scale(amplitude,sign)));actions+=1;
        }
    }
    let mut density=alloc::vec![[0.,0.];basis.len()*basis.len()];
    for row in residuals.values() {for &(i,a) in row {for &(j,b) in row {
        add(&mut density[i*basis.len()+j],outer(a,b));
    }}}
    (density,actions)
}
impl PairDensity {
    pub fn prepare(n:usize,source:&[u64],amplitudes:&[Complex])->Result<Self,String> {
        if n==0||n>32||source.is_empty()||source.len()!=amplitudes.len() {
            return Err("invalid annihilation-Gram source dimensions".into());
        }
        let electrons=source[0].count_ones() as usize;
        if source.windows(2).any(|s|s[0]>=s[1])||source.iter().any(|s|
            s.count_ones() as usize!=electrons||(2*n<64&&s>>(2*n)!=0))||
            amplitudes.iter().flatten().any(|a|!a.is_finite()) {
            return Err("inconsistent indexed fermion source".into());
        }
        let singles=states(n,1)?;let pair_states=states(n,2)?;
        let (single,a)=gram(source,amplitudes,&singles);let (pair,b)=gram(source,amplitudes,&pair_states);
        let norm=amplitudes.iter().map(|a|a[0]*a[0]+a[1]*a[1]).sum();
        Ok(Self{orbitals:n,electrons,norm,pair_states,single,pair,annihilation_actions:a+b})
    }
    pub fn record(&self)->serde_json::Value {
        serde_json::json!({"schema":"native-fermion-annihilation-Gram/v1","orbitals":self.orbitals,
            "electrons":self.electrons,"source_norm":self.norm,"pair_states":self.pair_states,
            "single_density":self.single,"pair_density":self.pair,
            "annihilation_actions":self.annihilation_actions,
            "convention":"rho[i,j]=sum_residual B[residual,i]*conj(B[residual,j]); native ordered removal",
            "retained":"complete pair Gram including all cross-pair coherence; source CI remains carried"})
    }
}
pub fn contract(n:usize,electrons:usize,pair:&[Complex],single:&[Complex],norm:f64)->Result<serde_json::Value,String> {
    if n==0||n>32||electrons>2*n {return Err("invalid density contraction source".into());}
    let pair_states=states(n,2)?;let count=pair_states.len();let slots=2*n;
    if pair.len()!=count*count {return Err("pair Gram dimensions differ from native pair basis".into());}
    let lookup:BTreeMap<_,_>=pair_states.iter().enumerate().map(|(i,&s)|(s,i)).collect();
    let address=|i:usize,j:usize|if i==j {None}else{Some((lookup[&((1<<i)|(1<<j))],if i<j {1.}else{-1.}))};
    let mut spin=alloc::vec![[0.,0.];slots*slots];
    if electrons>=2 {
        for i in 0..slots {for j in 0..slots {for k in 0..slots {
            if let (Some((a,sa)),Some((b,sb)))=(address(i,k),address(j,k)) {
                add(&mut spin[i*slots+j],scale(pair[a*count+b],sa*sb/(electrons-1) as f64));
            }
        }}}
    } else {
        if single.len()!=slots*slots {return Err("single-electron source requires its full single Gram".into());}
        spin.copy_from_slice(single);
    }
    let mut one=alloc::vec![[0.,0.];n*n];let mut two=alloc::vec![[0.,0.];n.pow(4)];
    for p in 0..n {for q in 0..n {for sigma in 0..2 {
        add(&mut one[p*n+q],spin[(2*p+sigma)*slots+2*q+sigma]);
        for r in 0..n {for s in 0..n {for tau in 0..2 {
            if let (Some((a,sa)),Some((b,sb)))=(address(2*p+sigma,2*r+tau),address(2*q+sigma,2*s+tau)) {
                add(&mut two[((p*n+q)*n+r)*n+s],scale(pair[b*count+a],sa*sb));
            }
        }}}
    }}}
    let mut cells=Vec::new();
    for p in 0..n {
        let paired=lookup[&((1<<(2*p))|(1<<(2*p+1)))];let double=pair[paired*count+paired];
        let up=spin[(2*p)*slots+2*p];let down=spin[(2*p+1)*slots+2*p+1];
        cells.push(serde_json::json!({"orbital":p,"empty":[norm-up[0]-down[0]+double[0],-up[1]-down[1]+double[1]],
            "up_only":[up[0]-double[0],up[1]-double[1]],"down_only":[down[0]-double[0],down[1]-double[1]],
            "paired":double,"singleton_native_glut_expectation":double}));
    }
    Ok(serde_json::json!({"schema":"native-pair-Gram-observations/v1","orbitals":n,"electrons":electrons,
        "one_body_density":one,"two_body_density":two,"spin_density":spin,"orbital_cells":cells,
        "one_body_convention":"dm1[p,q]=sum_spin <q_spin^dagger p_spin>",
        "two_body_convention":"dm2[p,q,r,s]=sum_spin,tau <p_spin^dagger r_tau^dagger s_tau q_spin>"}))
}
pub fn from_json(source:&serde_json::Value)->Result<serde_json::Value,String> {
    let n=source["orbitals"].as_u64().ok_or("missing orbital count")? as usize;
    let complex=|key:&str|->Result<Vec<Complex>,String>{source[key].as_array().ok_or_else(||alloc::format!("missing {key}"))?.iter()
        .map(|z|Ok([z[0].as_f64().ok_or("missing real coordinate")?,z[1].as_f64().ok_or("missing imaginary coordinate")?])).collect()};
    if source["operation"]=="contract" {
        return contract(n,source["electrons"].as_u64().ok_or("missing electron count")? as usize,
            &complex("pair_density")?,&complex("single_density")?,source["source_norm"].as_f64().ok_or("missing source norm")?);
    }
    let states=source["states"].as_array().ok_or("missing native states")?.iter()
        .map(|s|s.as_u64().ok_or("invalid native state")).collect::<Result<Vec<_>,_>>()?;
    Ok(PairDensity::prepare(n,&states,&complex("amplitudes")?)?.record())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn complex_pair_grams_recover_the_native_operator_energy_and_every_number_trace() {
        let n:usize=3;
        let h:Vec<_>=(0..n*n).map(|i|((i/n+i%n)+1) as f64/7.).collect();
        let mut eri=alloc::vec![0.;n.pow(4)];
        for p in 0..n {for q in 0..n {for r in 0..n {for s in 0..n {
            eri[((p*n+q)*n+r)*n+s]=((p+q+1)*(r+s+1)) as f64/19.;
        }}}}
        for e in 0..=2*n {
            let operator=crate::protein_electronic::Hamiltonian::prepare(n,e,&h,&eri).unwrap();
            let mut ray:Vec<_>=(0..operator.states.len()).map(|i|[(i+1) as f64,(i%3) as f64-1.]).collect();
            let norm=libm::sqrt(ray.iter().map(|z|z[0]*z[0]+z[1]*z[1]).sum());
            for z in &mut ray {*z=scale(*z,1./norm);}
            let density=PairDensity::prepare(n,&operator.states,&ray).unwrap();
            let out=contract(n,e,&density.pair,&density.single,density.norm).unwrap();
            let one:Vec<_>=out["one_body_density"].as_array().unwrap().iter().map(|v|v[0].as_f64().unwrap()).collect();
            let two:Vec<_>=out["two_body_density"].as_array().unwrap().iter().map(|v|v[0].as_f64().unwrap()).collect();
            let observed=one.iter().zip(&h).map(|(a,b)|a*b).sum::<f64>()+two.iter().zip(&eri).map(|(a,b)|a*b/2.).sum::<f64>();
            let mut expected=0.;for (i,a) in ray.iter().enumerate(){for (j,b) in ray.iter().enumerate(){
                expected+=operator.matrix[i][j]*(a[0]*b[0]+a[1]*b[1]);
            }}
            assert!((observed-expected).abs()<1e-12);
            assert!(((0..n).map(|p|one[p*n+p]).sum::<f64>()-e as f64).abs()<1e-12);
            let pair_count=density.pair_states.len();
            let trace=(0..pair_count).map(|i|density.pair[i*pair_count+i][0]).sum::<f64>();
            assert!((trace-(e*e.saturating_sub(1)/2) as f64).abs()<1e-12);
        }
    }
}
