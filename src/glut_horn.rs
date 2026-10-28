//! Nonlinear horn contraction on the existing u,v,p,q graph cells.
//! The zero-defect fibre is u²=N+v², p=u-v, q=u+v. No proposal
//! supplies a factor value: intervals and masks carry every surviving arm.
use super::*;
use core::cmp::Ordering;
type Domain=Vec<Option<char>>;
type Range=(Vec<char>,Vec<char>);
fn ceil_root(x:&[char])->Vec<char>{
    let r=mf::isqrt(x);if mf::mul(&r,&r)==x {r}else{mf::add(&r,&[ONE])}
}
fn half(x:&[char],up:bool)->Vec<char>{
    let h=trim(x.get(1..).unwrap_or(&[]).to_vec());
    if up&&bit(x,0)==1 {mf::add(&h,&[ONE])}else{h}
}
fn positive_sub(a:&[char],b:&[char])->Vec<char>{
    if mf::cmp(a,b)==Ordering::Less{vec![ZERO]}else{mf::sub(a,b)}
}
fn tighten(d:&mut[Domain;4],r:&mut[Range;4],i:usize,lo:Vec<char>,hi:Vec<char>)->Option<bool>{
    let lo=super::super::max(lo,r[i].0.clone());let hi=super::super::min(hi,r[i].1.clone());
    let interval=super::super::extrema(&d[i],&lo,&hi)?;
    let changed=super::super::constrain(&mut d[i],&interval.0,&interval.1)?;
    r[i]=interval;Some(changed)
}
fn residue_basis(n:&[char])->Vec<[usize;4]>{
    let bits=n.len().min(12);let modulus=1usize<<bits;let mask=modulus-1;
    let source=(0..bits).map(|i|(bit(n,i) as usize)<<i).sum::<usize>();
    let mut squares=vec![Vec::new();modulus];
    for v in 0..modulus{squares[v*v&mask].push(v);}
    let mut rows=Vec::new();
    for u in 0..modulus{
        let target=(u*u+modulus-source)&mask;
        for &v in &squares[target]{rows.push([u,v,(u+modulus-v)&mask,(u+v)&mask]);}
    }
    rows
}
fn residue_contract(d:&mut[Domain;4],basis:&[[usize;4]],bits:usize)->Option<bool>{
    let mut supports=vec![vec![0u8;bits];4];let mut count=0;
    for row in basis {
        let admitted=(0..4).all(|lane|(0..bits).all(|i|match d[lane].get(i).copied().unwrap_or(Some(ZERO)){
            Some(mark)=>usize::from(mark==ONE)==(row[lane]>>i)&1,None=>true}));
        if !admitted{continue;}count+=1;
        for lane in 0..4{for i in 0..bits{supports[lane][i]|=1<<((row[lane]>>i)&1);}}
    }
    if count==0{return None;}let mut changed=false;
    for lane in 0..4{for i in 0..bits.min(d[lane].len()){
        if d[lane][i].is_none(){
            let mark=match supports[lane][i]{1=>Some(ZERO),2=>Some(ONE),_=>None};
            if mark.is_some(){d[lane][i]=mark;changed=true;}
        }
    }}
    Some(changed)
}
fn contract(n:&[char],d:&mut[Domain;4],basis:&[[usize;4]])->Option<()> {
    let mut r: [Range;4]=core::array::from_fn(|i|super::super::bounds(&d[i]));
    loop {
        let mut changed=residue_contract(d,basis,n.len().min(12))?;
        let low=ceil_root(&mf::add(n,&mf::mul(&r[1].0,&r[1].0)));
        let high=mf::isqrt(&mf::add(n,&mf::mul(&r[1].1,&r[1].1)));
        changed|=tighten(d,&mut r,0,low,high)?;
        let square_hi=mf::mul(&r[0].1,&r[0].1);
        if mf::cmp(&square_hi,n)==Ordering::Less{return None;}
        let low=ceil_root(&positive_sub(&mf::mul(&r[0].0,&r[0].0),n));
        let high=mf::isqrt(&mf::sub(&square_hi,n));
        changed|=tighten(d,&mut r,1,low,high)?;
        let low=half(&mf::add(&r[2].0,&r[3].0),true);
        let high=half(&mf::add(&r[2].1,&r[3].1),false);
        changed|=tighten(d,&mut r,0,low,high)?;
        if mf::cmp(&r[3].1,&r[2].0)==Ordering::Less{return None;}
        let low=half(&positive_sub(&r[3].0,&r[2].1),true);
        let high=half(&mf::sub(&r[3].1,&r[2].0),false);
        changed|=tighten(d,&mut r,1,low,high)?;
        if mf::cmp(&r[0].1,&r[1].0)==Ordering::Less{return None;}
        let low=super::super::max(vec![ONE,ONE],positive_sub(&r[0].0,&r[1].1));
        let high=mf::sub(&r[0].1,&r[1].0);
        changed|=tighten(d,&mut r,2,low,high)?;
        let low=mf::add(&r[0].0,&r[1].0);let high=mf::add(&r[0].1,&r[1].1);
        changed|=tighten(d,&mut r,3,low,high)?;
        if !changed{return Some(());}
    }
}
fn record(d:&[Domain;4])->serde_json::Value {
    serde_json::json!(d.iter().map(|v|v.iter().map(|x|match x{
        Some(c) if *c==ONE=>1,Some(_)=>0,None=>-1}).collect::<Vec<_>>()).collect::<Vec<_>>())
}
impl Correlation {
    pub(crate) fn horn_snapshot(&self)->serde_json::Value{
        serde_json::json!({"schema":"resident-zero-horn-contraction-v2",
            "joint_residue_bits":self.source.len().min(12),"joint_residue_rows":self.horn_basis.len(),
            "source":mf::dec_of(&self.source),"coordinate_order":["u","v","p","q"],
            "calls":self.horn_receipts.len(),"receipts":self.horn_receipts})
    }
    pub(super) fn contract_horn(&mut self)->Result<(),Vec<Literal>>{
        if self.source.is_empty()||self.midpoint.is_empty()||self.gap.is_empty(){return Ok(());}
        if self.horn_basis.is_empty(){self.horn_basis=residue_basis(&self.source);}
        let cells=[self.midpoint.clone(),self.gap.clone(),self.p.clone(),self.q.clone()];
        let mut domains:[Domain;4]=core::array::from_fn(|i|cells[i].iter().map(|&c|self.values[c].parity()).collect());
        let before=record(&domains);
        let mut premise=Vec::new();
        for &cell in cells.iter().flatten(){
            if self.levels[cell]>0 {
                if let Some(mark)=self.values[cell].parity(){
                    let lit=Literal{cell,mark}.opposite();if !premise.contains(&lit){premise.push(lit);}
                }
            }
        }
        let mut rejected=contract(&self.source,&mut domains,&self.horn_basis).is_none();
        let mut probe=serde_json::Value::Null;
        if !rejected {
            // Rotate the tested coordinate through the same held domain. Both
            // values are contracted; only an unsupported alternative is cut.
            let free:Vec<_>=domains.iter().enumerate().flat_map(|(i,d)|
                d.iter().enumerate().rev().filter_map(move |(j,v)|v.is_none().then_some((i,j)))).collect();
            if let Some(&(i,j))=free.get(self.horn_receipts.len()%free.len().max(1)) {
                let mut admitted=[false;2];
                for (k,mark) in [ZERO,ONE].into_iter().enumerate(){
                    let mut trial=domains.clone();trial[i][j]=Some(mark);
                    admitted[k]=contract(&self.source,&mut trial,&self.horn_basis).is_some();
                }
                probe=serde_json::json!({"coordinate":i,"position":j,"admitted":admitted});
                match admitted {
                    [false,false]=>rejected=true,
                    [true,false]=>domains[i][j]=Some(ZERO),
                    [false,true]=>domains[i][j]=Some(ONE),_=>(),
                }
                if !rejected{rejected=contract(&self.source,&mut domains,&self.horn_basis).is_none();}
            }
        }
        let after=record(&domains);
        self.horn_receipts.push(serde_json::json!({"before":before,"after":after,
            "probe":probe,"rejected":rejected,"depth":self.boundaries.len(),
            "source_relation":"u*u=N+v*v; p+v=u; u+v=q","branch_preserving":true}));
        self.work+=domains.iter().map(Vec::len).sum::<usize>();
        if rejected{return Err(premise);}
        for (lane,values) in cells.iter().zip(&domains){
            for (&cell,&value) in lane.iter().zip(values){
                if self.values[cell].parity().is_none(){
                    if let Some(mark)=value {
                        let lit=Literal{cell,mark};let mut reason=premise.clone();reason.push(lit);
                        if !self.assign(lit,Some(reason.clone())){return Err(reason);}
                    }
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    extern crate std;
    #[test]
    fn source_200_horn_preserves_reference_only_witnesses(){
        let path=std::env::var("HORN_CONTROL_REFERENCE").expect("separate 200-bit verifier reference");
        let reference:serde_json::Value=serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
        let read=|name:&str|mf::decimal_to_tape(reference[name].as_str().unwrap()).unwrap();
        let(n,p,q)=(read("N"),read("p"),read("q"));
        assert!(n.len()>=200);assert_eq!(mf::mul(&p,&q),n);
        let u=half(&mf::add(&p,&q),false);let v=mf::sub(&q,&u);
        let witness=[u,v,p,q];let basis=residue_basis(&n);
        for seed in 0..128 {
            let mut domains:[Domain;4]=core::array::from_fn(|lane|{
                (0..witness[lane].len()).map(|i|
                    ((i*17+lane*11+seed*7)%13<6).then_some(witness[lane][i])).collect()
            });
            assert!(contract(&n,&mut domains,&basis).is_some(),"lost source witness at mask {seed}");
            for (lane,d) in domains.iter().enumerate(){
                for (i,mark) in d.iter().enumerate(){if let Some(mark)=mark{assert_eq!(*mark,witness[lane][i]);}}
            }
        }
        for lane in 0..4 {
            let mut domains:[Domain;4]=core::array::from_fn(|i|witness[i].iter().copied().map(Some).collect());
            domains[lane][3]=Some(if witness[lane][3]==ONE{ZERO}else{ONE});
            assert!(contract(&n,&mut domains,&basis).is_none(),"admitted changed complete coordinate {lane}");
        }
    }
}
