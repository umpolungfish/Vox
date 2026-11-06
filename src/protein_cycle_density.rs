//! Mixed response of each source cycle core, retaining inter-site coherence.
//! This is a source graph operator; it does not assign molecular bond energies.
use alloc::{string::{String,ToString},vec::Vec};
use crate::{protein_electronic_sector::SectorOperator,sic::frame::{Complex,Operator,Sic}};
#[derive(Clone)]
pub(crate) struct CycleDensity { atoms:Vec<usize>,operator:SectorOperator,rho:Vec<Complex>,scale:f64,events:Vec<serde_json::Value> }
impl CycleDensity {
    pub fn prepare(molecule:&serde_json::Value)->Result<Vec<Self>,String>{
        let atoms=molecule["atoms"].as_array().ok_or("missing cycle atoms")?;
        let bonds=molecule["bonds"].as_array().ok_or("missing cycle edges")?;
        let count=atoms.iter().filter_map(|a|a["residue"].as_u64()).max().map_or(0,|n|n+1);
        let mut output=Vec::new();
        for residue in 0..count{
            let mut core=atoms.iter().enumerate().filter(|(_,a)|a["residue"].as_u64()==Some(residue)).map(|(i,_)|i).collect::<Vec<_>>();
            loop{
                let before=core.len();let old=core.clone();core.retain(|&i|bonds.iter().filter(|b|{
                    let(a,c)=(b[0].as_u64().unwrap() as usize,b[1].as_u64().unwrap() as usize);
                    (a==i&&old.contains(&c))||(c==i&&old.contains(&a))
                }).count()>=2);
                if core.len()==before{break;}
            }
            let n=core.len();if n<3{continue;}
            let mut h=alloc::vec![0.0;n*n];
            for b in bonds{let(a,c)=(b[0].as_u64().unwrap() as usize,b[1].as_u64().unwrap() as usize);
                if let(Some(i),Some(j))=(core.iter().position(|&x|x==a),core.iter().position(|&x|x==c)){h[i*n+j]=1.0;h[j*n+i]=1.0;}}
            let operator=SectorOperator::prepare(n,1,0,&h,&alloc::vec![0.0;n.pow(4)])?;
            let mut rho=alloc::vec![Complex::new(0.0,0.0);n*n];for i in 0..n{rho[i*n+i]=Complex::new(1.0/n as f64,0.0);}
            output.push(Self{atoms:core,operator,rho,scale:1.0,events:Vec::new()});
        }
        Ok(output)
    }
    fn transform(&mut self,rho:&[Complex])->Result<(Vec<Complex>,f64),String>{
        let n=self.atoms.len();let mut left=alloc::vec![Complex::new(0.0,0.0);n*n];
        for j in 0..n{
            let re=self.operator.apply(&(0..n).map(|i|rho[i*n+j].re).collect::<Vec<_>>())?;
            let im=self.operator.apply(&(0..n).map(|i|rho[i*n+j].im).collect::<Vec<_>>())?;
            for i in 0..n{left[i*n+j]=Complex::new(re[i],im[i]);}
        }
        let mut out=left.clone();
        for i in 0..n{
            let re=self.operator.apply(&(0..n).map(|j|left[i*n+j].re).collect::<Vec<_>>())?;
            let im=self.operator.apply(&(0..n).map(|j|left[i*n+j].im).collect::<Vec<_>>())?;
            for j in 0..n{out[i*n+j]=Complex::new(re[j],im[j]);}
        }
        let trace=(0..n).map(|i|out[i*n+i].re).sum::<f64>();
        if trace<=0.0||!trace.is_finite(){return Err("cycle response has no finite retained mass".into());}
        for z in &mut out{z.re/=trace;z.im/=trace;}
        Ok((out,trace))
    }
    pub fn act(&mut self,step:usize)->Result<(),String>{
        let n=self.atoms.len();let before=self.rho.clone();let started=self.operator.applications;
        let (produced,normalization)=self.transform(&before)?;let scale_before=self.scale;self.scale*=normalization;let production_calls=self.operator.applications-started;
        let mut diagonal=before.clone();for i in 0..n{for j in 0..n{if i!=j{diagonal[i*n+j]=Complex::new(0.0,0.0);}}}
        let (control,control_normalization)=self.transform(&diagonal)?;
        let gap=produced.iter().zip(&control).map(|(a,b)|(a.re-b.re)*(a.re-b.re)+(a.im-b.im)*(a.im-b.im)).sum::<f64>();
        // Principal FOUR blocks cover every matrix coordinate. General SIC
        // synthesis preserves a mixed matrix without a rank-one ray assumption.
        let frame=crate::sic::wh::ququart_frame();let mut covers=Vec::new();
        if n<=4{covers.push((0..n).collect::<Vec<_>>());}else{
            for a in 2..n{for b in a+1..n{covers.push(alloc::vec![0,1,a,b]);}}
        }
        let mut returned=alloc::vec![Complex::new(0.0,0.0);n*n];let mut seen=alloc::vec![false;n*n];let mut receipts=Vec::new();
        let tolerance=4096.0*f64::EPSILON;
        for cover in covers{
            let mut block=Operator::zero(4);for (i,&a) in cover.iter().enumerate(){for(j,&b)in cover.iter().enumerate(){block.set(i,j,produced[a*n+b]);}}
            let masses=frame.split(&block).map_err(|e|e.to_string())?;let recovered=frame.fuse(&masses).map_err(|e|e.to_string())?;
            let residual=recovered.distance(&block).map_err(|e|e.to_string())?;
            if residual>tolerance{return Err("mixed source cycle SIC return exceeds its bound".into());}
            for(i,&a)in cover.iter().enumerate(){for(j,&b)in cover.iter().enumerate(){
                let z=recovered.values()[i*4+j];let k=a*n+b;
                if seen[k]&&((returned[k].re-z.re).abs()>tolerance||(returned[k].im-z.im).abs()>tolerance){return Err("mixed cycle principal-frame overlap disagrees".into());}
                returned[k]=z;seen[k]=true;
            }}
            receipts.push(serde_json::json!({"source_sites":cover,"masses":masses.values.iter().map(|z|[z.re,z.im]).collect::<Vec<_>>(),"residual":residual}));
        }
        if seen.iter().any(|&s|!s){return Err("mixed cycle return left a source matrix coordinate uncovered".into());}
        self.rho=returned;
        self.events.push(serde_json::json!({"source_step":step,"source_scale_before":scale_before,"source_scale_after":self.scale,"normalization_retained":normalization,"control_normalization":control_normalization,"operation":"source cycle H rho H^dagger followed by general principal-block SIC return",
            "native_operator_applications":production_calls,"control_native_operator_applications":self.operator.applications-started-production_calls,"population_only_control_squared_gap":gap,
            "input_matrix":before.iter().map(|z|[z.re,z.im]).collect::<Vec<_>>(),
            "produced_matrix":produced.iter().map(|z|[z.re,z.im]).collect::<Vec<_>>(),
            "returned_matrix":self.rho.iter().map(|z|[z.re,z.im]).collect::<Vec<_>>(),"principal_frames":receipts,"tolerance":tolerance}));
        Ok(())
    }
    pub fn report(&self)->serde_json::Value{
        serde_json::json!({"source_atom_refs":self.atoms,"source_scale":self.scale,"operator":self.operator.record(),
            "matrix":self.rho.iter().map(|z|[z.re,z.im]).collect::<Vec<_>>(),"events":self.events,
            "scope":"one-probe mixed source cycle response; no molecular electron count or bond-energy prediction is assigned"})
    }
}
