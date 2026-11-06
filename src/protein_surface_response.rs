//! Bounded source-cycle coupling experiment, in source response units.
//! This does not assign molecular electron counts or calibrated bond energies.
use alloc::{string::{String,ToString},vec::Vec};
use crate::{protein_electronic_sector::SectorOperator,sic::frame::{Complex,Operator,Sic}};
#[derive(Clone)]
pub(crate) struct SurfaceResponse {
    residues:[usize;2],refs:Vec<Vec<usize>>,a:SectorOperator,b:SectorOperator,
    rho:Vec<Complex>,length:f64,coefficients:[f64;3],events:Vec<serde_json::Value>,
}
impl SurfaceResponse {
    pub fn prepare(molecule:&serde_json::Value,cycles:&[serde_json::Value])->Result<Option<Self>,String>{
        let atoms=molecule["atoms"].as_array().ok_or("surface source atoms missing")?;
        let find=|name:&str|cycles.iter().find(|c|{
            let index=c["source_atom_refs"][0].as_u64().unwrap() as usize;
            molecule["residue_identities"][atoms[index]["residue"].as_u64().unwrap() as usize]==name
        });
        let(Some(left),Some(right))=(find("Tyr"),find("Trp"))else{return Ok(None)};
        let refs=[left,right].iter().map(|c|c["source_atom_refs"].as_array().unwrap().iter().map(|v|v.as_u64().unwrap() as usize).collect::<Vec<_>>()).collect::<Vec<_>>();
        let n=refs.iter().map(Vec::len).sum::<usize>();let mut h=alloc::vec![0.0;n*n];let mut bridge=h.clone();let mut rho=alloc::vec![Complex::new(0.0,0.0);n*n];
        let mut offset=0;let mut lengths=Vec::new();let mut residues=[0;2];
        for (part,cycle) in [left,right].iter().enumerate(){
            let size=refs[part].len();let scale=cycle["source_scale"].as_f64().unwrap();residues[part]=atoms[refs[part][0]]["residue"].as_u64().unwrap() as usize;
            for i in 0..size{for j in 0..size{let value=&cycle["matrix"][i*size+j];rho[(offset+i)*n+offset+j]=Complex::new(scale*value[0].as_f64().unwrap(),scale*value[1].as_f64().unwrap());}}
            for bond in molecule["bonds"].as_array().unwrap(){
                let(i,j)=(bond[0].as_u64().unwrap() as usize,bond[1].as_u64().unwrap() as usize);
                if let(Some(a),Some(b))=(refs[part].iter().position(|&x|x==i),refs[part].iter().position(|&x|x==j)){
                    h[(offset+a)*n+offset+b]=1.0;h[(offset+b)*n+offset+a]=1.0;lengths.push(bond[2].as_f64().unwrap()/1000.0);
                }
            }offset+=size;
        }
        for i in 0..refs[0].len(){for j in refs[0].len()..n{bridge[i*n+j]=1.0;bridge[j*n+i]=1.0;}}
        let a=SectorOperator::prepare(n,1,0,&h,&alloc::vec![0.0;n.pow(4)])?;
        let b=SectorOperator::prepare(n,1,0,&bridge,&alloc::vec![0.0;n.pow(4)])?;
        Ok(Some(Self{residues,refs,a,b,rho,length:lengths.iter().sum::<f64>()/lengths.len() as f64,coefficients:[0.0;3],events:Vec::new()}))
    }
    fn apply(&mut self,values:&[f64],t:f64)->Result<Vec<f64>,String>{
        let a=self.a.apply(values)?;let b=self.b.apply(values)?;
        Ok(a.iter().zip(b).map(|(x,y)|x+t*y).collect())
    }
    fn transform(&mut self,rho:&[Complex],t:f64)->Result<Vec<Complex>,String>{
        let n=self.a.record()["orbitals"].as_u64().unwrap() as usize;
        let mut left=alloc::vec![Complex::new(0.0,0.0);n*n];let mut out=left.clone();
        for j in 0..n{
            let re=self.apply(&(0..n).map(|i|rho[i*n+j].re).collect::<Vec<_>>(),t)?;
            let im=self.apply(&(0..n).map(|i|rho[i*n+j].im).collect::<Vec<_>>(),t)?;
            for i in 0..n{left[i*n+j]=Complex::new(re[i],im[i]);}
        }
        for i in 0..n{
            let re=self.apply(&(0..n).map(|j|left[i*n+j].re).collect::<Vec<_>>(),t)?;
            let im=self.apply(&(0..n).map(|j|left[i*n+j].im).collect::<Vec<_>>(),t)?;
            for j in 0..n{out[i*n+j]=Complex::new(re[j],im[j]);}
        }Ok(out)
    }
    pub fn advance(&mut self,step:usize,positions:&[[f64;3]])->Result<(),String>{
        let n=self.refs.iter().map(Vec::len).sum::<usize>();
        let distance=libm::sqrt((0..3).map(|k|{let d=positions[self.residues[0]][k]-positions[self.residues[1]][k];d*d}).sum::<f64>()).max(1e-6);
        let before=self.rho.clone();let counts=[self.a.applications,self.b.applications];
        let produced=self.transform(&before,self.length/distance)?;
        let scale=(0..n).map(|i|produced[i*n+i].re).sum::<f64>();
        if !scale.is_finite()||scale<=0.0{return Err("surface response has no finite positive source scale".into());}
        let frame=crate::sic::wh::ququart_frame();let tolerance=4096.0*f64::EPSILON;let mut returned=alloc::vec![Complex::new(0.0,0.0);n*n];let mut seen=alloc::vec![false;n*n];let mut frames=Vec::new();
        for a in 2..n{for b in a+1..n{
            let cover=[0,1,a,b];let mut block=Operator::zero(4);
            for(i,&x)in cover.iter().enumerate(){for(j,&y)in cover.iter().enumerate(){let z=produced[x*n+y];block.set(i,j,Complex::new(z.re/scale,z.im/scale));}}
            let masses=frame.split(&block).map_err(|e|e.to_string())?;let recovered=frame.fuse(&masses).map_err(|e|e.to_string())?;
            let residual=recovered.distance(&block).map_err(|e|e.to_string())?;if residual>tolerance{return Err("joint surface SIC return exceeds tolerance".into());}
            for(i,&x)in cover.iter().enumerate(){for(j,&y)in cover.iter().enumerate(){let z=recovered.get(i,j);let k=x*n+y;
                if seen[k]&&((returned[k].re/scale-z.re).abs()>tolerance||(returned[k].im/scale-z.im).abs()>tolerance){return Err("surface principal frames disagree".into());}
                returned[k]=Complex::new(z.re*scale,z.im*scale);seen[k]=true;
            }}
            frames.push(serde_json::json!({"source_sites":cover,"masses":masses.values.iter().map(|z|[z.re,z.im]).collect::<Vec<_>>(),"residual":residual}));
        }}
        if seen.iter().any(|v|!*v){return Err("surface return left uncovered coordinates".into());}
        self.rho=returned;
        let retained=self.rho.clone();let mut z=[0.0;3];
        for(i,t)in [0.0,1.0,-1.0].iter().enumerate(){let out=self.transform(&retained,*t)?;z[i]=(0..n).map(|j|out[j*n+j].re).sum();}
        self.coefficients=[z[0],(z[1]-z[2])/2.0,(z[1]+z[2])/2.0-z[0]];
        self.events.push(serde_json::json!({"source_step":step,"distance_before":distance,"source_scale":scale,
            "input_matrix":before.iter().map(|z|[z.re,z.im]).collect::<Vec<_>>(),"produced_matrix":produced.iter().map(|z|[z.re,z.im]).collect::<Vec<_>>(),
            "returned_matrix":self.rho.iter().map(|z|[z.re,z.im]).collect::<Vec<_>>(),"principal_frames":frames,"tolerance":tolerance,
            "response_coefficients":self.coefficients,"native_operator_applications":[self.a.applications-counts[0],self.b.applications-counts[1]]}));
        Ok(())
    }
    pub fn potential_gradient(&self,positions:&[[f64;3]])->(f64,[usize;2],[f64;3]){
        let dx=core::array::from_fn::<_,3,_>(|k|positions[self.residues[0]][k]-positions[self.residues[1]][k]);
        let r=libm::sqrt(dx.iter().map(|v|v*v).sum::<f64>()).max(1e-6);let t=self.length/r;
        let[a,b,c]=self.coefficients;let g=(b+2.0*c*t)*self.length/(r*r);
        (-(a+b*t+c*t*t),self.residues,dx.map(|x|g*x/r))
    }
    pub fn report(&self)->serde_json::Value{
        serde_json::json!({"source_residues":self.residues,"source_atom_refs":self.refs,"source_mean_bond_length":self.length,
            "response_coefficients":self.coefficients,"events":self.events,
            "scope":"inverse-distance one-probe cycle-response control in source response units; not calibrated molecular binding energy"})
    }
}
