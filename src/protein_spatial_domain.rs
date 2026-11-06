//! Exact distance-domain actions on the payload retained by the supplied word.
use alloc::{string::{String,ToString},vec::Vec};
const OPEN:u64=u64::MAX/4;
fn bond_tolerance()->u64{(crate::peptide_closure::BOND_TOLERANCE*1000.0+0.5) as u64}
#[derive(Clone,PartialEq,Eq)]
struct DistanceDomain { lower:Vec<Vec<u64>>,upper:Vec<Vec<u64>>,bonds:Vec<(usize,usize,u64)> }
#[derive(Clone)]
pub(crate) struct SpatialFamily { domains:Vec<DistanceDomain>,identity:bool,events:Vec<serde_json::Value> }
impl SpatialFamily {
    pub fn source(molecule:&serde_json::Value)->Result<Self,String>{
        let n=molecule["atoms"].as_array().ok_or("missing spatial atom inventory")?.len();
        let mut upper=alloc::vec![alloc::vec![OPEN;n];n];let mut lower=alloc::vec![alloc::vec![0;n];n];
        for i in 0..n{upper[i][i]=0;}
        let mut bonds=Vec::new();
        for row in molecule["bonds"].as_array().ok_or("missing spatial source bonds")?{
            let i=row[0].as_u64().ok_or("invalid spatial bond")? as usize;
            let j=row[1].as_u64().ok_or("invalid spatial bond")? as usize;
            let d=row[2].as_u64().ok_or("invalid spatial source length")?;
            if i>=n||j>=n||i==j||d>=OPEN{return Err("invalid spatial bond boundary".into());}
            upper[i][j]=d+bond_tolerance();upper[j][i]=d+bond_tolerance();lower[i][j]=d.saturating_sub(bond_tolerance());lower[j][i]=d.saturating_sub(bond_tolerance());bonds.push((i,j,d));
        }
        Ok(Self{domains:alloc::vec![DistanceDomain{lower,upper,bonds}],identity:false,events:Vec::new()})
    }
    pub fn identity()->Self{Self{domains:Vec::new(),identity:true,events:Vec::new()}}
    pub fn fuse(sources:&[&Self],step:usize)->Self{
        let mut domains=Vec::new();
        for source in sources{for domain in &source.domains{if !domains.contains(domain){domains.push(domain.clone());}}}
        Self{domains,identity:sources.iter().any(|s|s.identity),events:alloc::vec![serde_json::json!({
            "source_step":step,"operation":"FFUSE retains whole distance-domain alternatives and the identity continuation"})]}
    }
    pub fn assert_source_angles(&mut self,step:usize){
        // The native heavy-atom closure predicate checks every covalent angle
        // in 90..135 degrees. Convert its exact three-atom relation to outward
        // integer distance bounds without introducing a torsion table.
        fn floor_sqrt(n:u128)->u128{
            let(mut lo,mut hi)=(0,n.min(u64::MAX as u128)+1);
            while hi-lo>1{let mid=(lo+hi)/2;if mid<=n/mid{lo=mid;}else{hi=mid;}}
            lo
        }
        fn ceil_sqrt(n:u128)->u128{let r=floor_sqrt(n);r+u128::from(r*r<n)}
        for (alternative,domain) in self.domains.iter_mut().enumerate(){
            let mut neighbors=alloc::vec![Vec::new();domain.upper.len()];
            for &(a,b,d) in &domain.bonds{neighbors[a].push((b,d));neighbors[b].push((a,d));}
            let mut witnesses=Vec::new();let mut changes=0;
            for (b,adjacent) in neighbors.iter().enumerate(){for i in 0..adjacent.len(){for &(c,y) in &adjacent[i+1..]{
                let(a,x)=adjacent[i];
                let source_lengths=[x,y];
                let(lx,ux,ly,uy)=(domain.lower[a][b] as u128,domain.upper[a][b] as u128,
                    domain.lower[b][c] as u128,domain.upper[b][c] as u128);
                let lower=floor_sqrt(lx*lx+ly*ly) as u64;
                let upper=ceil_sqrt(ux*ux+uy*uy+ceil_sqrt(2*ux*ux*uy*uy)) as u64;
                if lower>domain.lower[a][c]{domain.lower[a][c]=lower;domain.lower[c][a]=lower;changes+=1;}
                if upper<domain.upper[a][c]{domain.upper[a][c]=upper;domain.upper[c][a]=upper;changes+=1;}
                witnesses.push(serde_json::json!({"atoms":[a,b,c],"source_bond_lengths":source_lengths,"source_bond_intervals":[[lx as u64,ux as u64],[ly as u64,uy as u64]],
                    "outward_endpoint_distance_bounds":[lower,upper]}));
            }}}
            self.events.push(serde_json::json!({"source_step":step,"alternative":alternative,"opcode":"⊤",
                "operation":"native covalent three-atom angle predicate on the retained spatial domain",
                "predicate_source":"peptide_closure::assess all covalent angles 90..135 degrees",
                "scope":"necessary geometric part of the supplied affirmative action; native long-range contact selection remains unresolved",
                "bound_updates":changes,"angle_witnesses":witnesses}));
        }
    }
    pub fn act(&mut self,step:usize,mark:char)->Result<(),String>{
        for (alternative,domain) in self.domains.iter_mut().enumerate(){
            let n=domain.upper.len();let mut upper_updates=0;let mut lower_updates=0;let mut rounds=0;
            // CLINK is min-plus composition at one shared atom. AFWD closes
            // the same exact integer bounds through all intermediate atoms.
            loop{
                let before_upper=domain.upper.clone();let before_lower=domain.lower.clone();
                for k in 0..n{for i in 0..n{for j in (i+1)..n{
                    let composed=before_upper[i][k].saturating_add(before_upper[k][j]).min(OPEN);
                    if composed<domain.upper[i][j]{domain.upper[i][j]=composed;domain.upper[j][i]=composed;upper_updates+=1;}
                    let lower=before_lower[i][k].saturating_sub(before_upper[k][j])
                        .max(before_lower[k][j].saturating_sub(before_upper[i][k]));
                    if lower>domain.lower[i][j]{domain.lower[i][j]=lower;domain.lower[j][i]=lower;lower_updates+=1;}
                }}}
                rounds+=1;
                if domain.upper==before_upper&&domain.lower==before_lower{break;}
                if mark=='⋈'{break;}
            }
            let contradictions=(0..n).flat_map(|i|((i+1)..n).map(move|j|(i,j)))
                .filter(|&(i,j)|domain.lower[i][j]>domain.upper[i][j]).map(|(i,j)|[i,j]).collect::<Vec<_>>();
            self.events.push(serde_json::json!({"source_step":step,"opcode":mark.to_string(),"alternative":alternative,
                "contradictory_bound_pairs_retained":contradictions,"upper_updates":upper_updates,"lower_updates":lower_updates,"rounds":rounds,
                "operation":"triangle and reverse-triangle actions on the retained distance domain"}));
        }
        Ok(())
    }
    pub fn report(&self)->serde_json::Value{
        let domains=self.domains.iter().map(|d|{
            let upper=d.upper.iter().map(|row|row.iter().map(|&v|if v==OPEN{None}else{Some(v)}).collect::<Vec<_>>()).collect::<Vec<_>>();
            serde_json::json!({"distance_lower_bounds":d.lower,"distance_upper_bounds":upper,"source_bonds":d.bonds,
                "atoms":d.upper.len(),"finite_pairs":d.upper.iter().enumerate().map(|(i,r)|r.iter().skip(i+1).filter(|&&v|v<OPEN).count()).sum::<usize>()})
        }).collect::<Vec<_>>();
        serde_json::json!({"representation":"retained alternatives of exact molecular distance domains",
            "distance_unit":"1/1000 angstrom","native_bond_tolerance_milliangstrom":bond_tolerance(),"identity_continuation":self.identity,"domains":domains,"events":self.events,
            "cartesian_conformation_resolved":false})
    }
}
