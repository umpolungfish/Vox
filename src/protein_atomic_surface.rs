//! Atomic-frame Gram bridge on the retained native cycle response.
//! All covalent frame choices are retained by an isometric source lift.
use alloc::{string::{String,ToString},vec::Vec};
use crate::{peptide_geometry as v,protein_electronic_sector::SectorOperator,sic::frame::{Complex,Operator,Sic}};
type Point=v::Point;

#[derive(Clone)]
pub(crate) struct AtomicSurface{
    paths:Vec<[usize;3]>,weights:Vec<f64>,split:usize,residues:[usize;2],
    a:SectorOperator,h:Vec<f64>,rho:Vec<Complex>,length:f64,
    lift_residuals:Vec<f64>,events:Vec<serde_json::Value>,
}

#[cfg(test)]
mod tests{
    use super::*;
    #[test]
    fn atomic_frame_force_is_conservative_and_observes_ring_orientation(){
        let chain=["Tyr","Ala","Trp"];
        let mut molecule=crate::peptide_closure::molecular_constraint_source(&chain);
        molecule["residue_identities"]=serde_json::json!(chain);
        let steps=alloc::vec![crate::fold3d::RamaEntry{phi:-65.0,psi:-40.0,ss:"other",conf:0.0};chain.len()];
        let offsets=alloc::vec![(0.0,0.0,0.0);molecule["atoms"].as_array().unwrap().len()];
        let mut points=crate::protein_relax::independent_pose_points(&chain,&steps,&steps,&offsets);
        let cycles=crate::protein_cycle_density::CycleDensity::prepare(&molecule).unwrap().iter().map(|c|c.report()).collect::<Vec<_>>();
        let mut field=AtomicSurface::prepare(&molecule,&cycles,&points).unwrap().unwrap();field.advance(3,&points).unwrap();
        assert!(field.lift_residuals.iter().all(|r|r.abs()<64.0*f64::EPSILON));
        assert_eq!(field.events[0]["native_operator_applications"][0].as_u64(),Some(4*field.paths.len() as u64));
        let(energy,gradient)=field.potential_gradient(&points).unwrap();let delta=libm::pow(2.0,-18.0);
        assert!(energy>=0.0);
        let mut native_return=field.clone();native_return.advance(4,&points).unwrap();
        let native_residual=field.rho.iter().zip(&native_return.rho).map(|(a,b)|{
            let re=b.re-a.re;let im=b.im-a.im;re*re+im*im
        }).sum::<f64>();
        assert!((native_residual-energy).abs()<1e-10*(1.0+energy),"mechanical residual {energy} differs from actual native/SIC return {native_residual}");
        for atom in 0..points.len(){for axis in 0..3{
            let original=points[atom];let mut p=[original.0,original.1,original.2];p[axis]+=delta;points[atom]=(p[0],p[1],p[2]);let plus=field.potential_gradient(&points).unwrap().0;
            p[axis]-=2.0*delta;points[atom]=(p[0],p[1],p[2]);let minus=field.potential_gradient(&points).unwrap().0;points[atom]=original;
            let actual=[gradient[atom].0,gradient[atom].1,gradient[atom].2][axis];let measured=(plus-minus)/(2.0*delta);
            assert!((actual-measured).abs()<1e-5*(1.0+actual.abs()),"atom {atom}, axis {axis}: {actual} vs {measured}");
        }}
        let rotated=points.iter().map(|p|(-p.0,p.1,-p.2)).collect::<Vec<_>>();let(rotated_energy,rotated_gradient)=field.potential_gradient(&rotated).unwrap();
        assert!((rotated_energy-energy).abs()<1e-10*(1.0+energy.abs()));
        for (a,b) in gradient.iter().zip(rotated_gradient){assert!((a.0+b.0).abs()+(a.1-b.1).abs()+(a.2+b.2).abs()<1e-9*(1.0+v::dot(*a,*a)));}
        let atoms=molecule["atoms"].as_array().unwrap();let find=|name:&str|atoms.iter().position(|a|a["residue"]==2&&a["name"].as_str().unwrap().trim()==name).unwrap();
        let origin=points[find("CG")];let axis=v::unit(v::sub(origin,points[find("CB")]));let mut changed=points.clone();
        for atom in cycles.iter().find(|c|c["source_atom_refs"].as_array().unwrap().iter().any(|a|a.as_u64()==Some(find("NE1") as u64))).unwrap()["source_atom_refs"].as_array().unwrap(){
            let i=atom.as_u64().unwrap() as usize;let d=v::sub(points[i],origin);changed[i]=v::add(origin,v::add(v::scale(axis,v::dot(axis,d)),v::cross(axis,d)));
        }
        assert!((field.potential_gradient(&changed).unwrap().0-energy).abs()>1e-8*(1.0+energy.abs()));
        let n=field.paths.len();let mut phase=field.clone();for i in 0..n{for j in 0..n{let index=i*n+j;let mut z=phase.rho[index];if i==0{z=Complex::new(-z.im,z.re);}if j==0{z=Complex::new(z.im,-z.re);}phase.rho[index]=z;}}
        assert!((phase.potential_gradient(&points).unwrap().0-energy).abs()>1e-8*(1.0+energy.abs()));
        #[cfg(feature="hosted")]{
            let saved:serde_json::Value=serde_json::from_str(&serde_json::to_string(&field.report()).unwrap()).unwrap();
            let mut restored=AtomicSurface::from_saved(&saved,points.len()).unwrap();
            assert_eq!(restored.report(),field.report());
            assert_eq!(restored.potential_gradient(&points).unwrap().0.to_bits(),energy.to_bits());
            field.advance(4,&points).unwrap();restored.advance(4,&points).unwrap();
            assert_eq!(restored.report(),field.report());
        }

    }
}
impl AtomicSurface{
    pub fn prepare(molecule:&serde_json::Value,cycles:&[serde_json::Value],points:&[Point])->Result<Option<Self>,String>{
        let atoms=molecule["atoms"].as_array().ok_or("atomic surface atom source missing")?;
        let find=|name:&str|cycles.iter().find(|c|{
            let i=c["source_atom_refs"][0].as_u64().unwrap() as usize;
            molecule["residue_identities"][atoms[i]["residue"].as_u64().unwrap() as usize]==name
        });
        let(Some(left),Some(right))=(find("Tyr"),find("Trp"))else{return Ok(None)};
        let residues=[left,right].map(|c|{let i=c["source_atom_refs"][0].as_u64().unwrap() as usize;atoms[i]["residue"].as_u64().unwrap() as usize});
        let bonds=molecule["bonds"].as_array().ok_or("atomic surface bond source missing")?;
        let mut paths=Vec::new();let mut weights=Vec::new();let mut site_ids=Vec::new();let mut parts=Vec::new();
        let mut source_refs=Vec::new();let mut residuals=Vec::new();let mut split=0;let mut lengths=Vec::new();
        for(part,cycle)in [left,right].iter().enumerate(){
            let refs=cycle["source_atom_refs"].as_array().unwrap().iter().map(|v|v.as_u64().unwrap() as usize).collect::<Vec<_>>();
            let mut reference=None;
            for(site,&center)in refs.iter().enumerate(){
                let mut neighbors=Vec::new();for bond in bonds{
                    let(a,b)=(bond[0].as_u64().unwrap() as usize,bond[1].as_u64().unwrap() as usize);
                    if a==center&&refs.contains(&b){neighbors.push(b);}if b==center&&refs.contains(&a){neighbors.push(a);}
                }neighbors.sort_unstable();neighbors.dedup();
                let count=neighbors.len()*(neighbors.len().saturating_sub(1))/2;
                if count==0{return Err("cycle source has no covalent plane frame".into());}
                let magnitude=1.0/libm::sqrt(count as f64);let mut norm=0.0;
                for i in 0..neighbors.len(){for j in i+1..neighbors.len(){
                    let path=[neighbors[i],center,neighbors[j]];let normal=Self::normal(path,points)?.0;
                    let axis=*reference.get_or_insert(normal);let sign=if v::dot(axis,normal)>=0.0{1.0}else{-1.0};
                    paths.push(path);weights.push(sign*magnitude);site_ids.push(site);parts.push(part);norm+=magnitude*magnitude;
                }}residuals.push(norm-1.0);
            }
            for bond in bonds{let(a,b)=(bond[0].as_u64().unwrap() as usize,bond[1].as_u64().unwrap() as usize);
                if refs.contains(&a)&&refs.contains(&b){lengths.push(bond[2].as_f64().unwrap()/1000.0);}
            }
            source_refs.push(refs);if part==0{split=paths.len();}
        }
        let n=paths.len();let mut h=alloc::vec![0.0;n*n];let mut rho=alloc::vec![Complex::new(0.0,0.0);n*n];
        for i in 0..n{for j in 0..n{if parts[i]!=parts[j]{continue;}
            let cycle=if parts[i]==0{left}else{right};let count=source_refs[parts[i]].len();
            let z=&cycle["matrix"][site_ids[i]*count+site_ids[j]];let scale=cycle["source_scale"].as_f64().unwrap()*weights[i]*weights[j];
            rho[i*n+j]=Complex::new(scale*z[0].as_f64().unwrap(),scale*z[1].as_f64().unwrap());
            if bonds.iter().any(|b|{let(a,c)=(b[0].as_u64().unwrap() as usize,b[1].as_u64().unwrap() as usize);(a==paths[i][1]&&c==paths[j][1])||(c==paths[i][1]&&a==paths[j][1])}){
                h[i*n+j]=weights[i]*weights[j];
            }
        }}
        let a=SectorOperator::prepare(n,1,0,&h,&alloc::vec![0.0;n.pow(4)])?;
        Ok(Some(Self{paths,weights,split,residues,a,h,rho,length:lengths.iter().sum::<f64>()/lengths.len() as f64,lift_residuals:residuals,events:Vec::new()}))
    }
    fn normal(path:[usize;3],points:&[Point])->Result<(Point,f64),String>{
        let w=v::cross(v::sub(points[path[0]],points[path[1]]),v::sub(points[path[2]],points[path[1]]));
        let length=libm::sqrt(v::dot(w,w));if !length.is_finite()||length==0.0{return Err("atomic cycle plane is singular".into());}
        Ok((v::scale(w,1.0/length),length))
    }
    fn bridge(&self,points:&[Point])->Result<(Vec<f64>,Vec<(Point,f64)>),String>{
        let normals=self.paths.iter().map(|&p|Self::normal(p,points)).collect::<Result<Vec<_>,_>>()?;
        let n=self.paths.len();let mut b=alloc::vec![0.0;n*n];
        for i in 0..self.split{for j in self.split..n{
            let d=v::sub(points[self.paths[i][1]],points[self.paths[j][1]]);let r=libm::sqrt(v::dot(d,d));
            if !r.is_finite()||r==0.0{return Err("atomic cycle contact is singular".into());}
            let value=self.length/r*self.weights[i].abs()*self.weights[j].abs()*v::dot(normals[i].0,normals[j].0);
            b[i*n+j]=value;b[j*n+i]=value;
        }}Ok((b,normals))
    }
    pub fn potential_gradient(&self,points:&[Point])->Result<(f64,Vec<Point>),String>{
        let(b,normals)=self.bridge(points)?;let n=self.paths.len();let h=self.h.iter().zip(&b).map(|(a,b)|a+b).collect::<Vec<_>>();
        // The actual retained return is H rho H†. Its full complex difference
        // from rho supplies one fixed-point/mechanical objective. A larger
        // returned signal alone does not establish closure.
        let mut hr=alloc::vec![Complex::new(0.0,0.0);n*n];let mut rh=hr.clone();
        for i in 0..n{for j in 0..n{for k in 0..n{
            hr[i*n+j].re+=h[i*n+k]*self.rho[k*n+j].re;hr[i*n+j].im+=h[i*n+k]*self.rho[k*n+j].im;
            rh[i*n+j].re+=self.rho[i*n+k].re*h[k*n+j];rh[i*n+j].im+=self.rho[i*n+k].im*h[k*n+j];
        }}}
        let mut residual=alloc::vec![Complex::new(0.0,0.0);n*n];let mut energy=0.0;
        for i in 0..n{for j in 0..n{
            let mut z=Complex::new(-self.rho[i*n+j].re,-self.rho[i*n+j].im);
            for k in 0..n{z.re+=hr[i*n+k].re*h[k*n+j];z.im+=hr[i*n+k].im*h[k*n+j];}
            energy+=z.re*z.re+z.im*z.im;residual[i*n+j]=z;
        }}
        let mut operator_gradient=alloc::vec![0.0;n*n];
        for a in 0..n{for c in 0..n{for j in 0..n{
            let r=residual[a*n+j];let z=rh[c*n+j];operator_gradient[a*n+c]+=2.0*(r.re*z.re+r.im*z.im);
            let r=residual[j*n+c];let z=hr[j*n+a];operator_gradient[a*n+c]+=2.0*(r.re*z.re+r.im*z.im);
        }}}
        let mut gradient=alloc::vec![(0.0,0.0,0.0);points.len()];let mut normal_gradient=alloc::vec![(0.0,0.0,0.0);n];
        for i in 0..self.split{for j in self.split..n{
            let a=self.paths[i][1];let c=self.paths[j][1];let d=v::sub(points[a],points[c]);let square=v::dot(d,d);let r=libm::sqrt(square);
            let g=operator_gradient[i*n+j]+operator_gradient[j*n+i];let coefficient=self.length/r*self.weights[i].abs()*self.weights[j].abs();let cosine=v::dot(normals[i].0,normals[j].0);
            let radial=v::scale(d,-g*coefficient*cosine/square);gradient[a]=v::add(gradient[a],radial);gradient[c]=v::sub(gradient[c],radial);
            normal_gradient[i]=v::add(normal_gradient[i],v::scale(normals[j].0,g*coefficient));normal_gradient[j]=v::add(normal_gradient[j],v::scale(normals[i].0,g*coefficient));
        }}
        for i in 0..n{
            let[a,c,b]=self.paths[i];let(normal,length)=normals[i];let g=normal_gradient[i];
            let cross_gradient=v::scale(v::sub(g,v::scale(normal,v::dot(normal,g))),1.0/length);
            let u=v::sub(points[a],points[c]);let w=v::sub(points[b],points[c]);let gu=v::cross(w,cross_gradient);let gw=v::cross(cross_gradient,u);
            gradient[a]=v::add(gradient[a],gu);gradient[b]=v::add(gradient[b],gw);gradient[c]=v::sub(gradient[c],v::add(gu,gw));
        }
        Ok((energy,gradient))
    }
    pub fn advance(&mut self,step:usize,points:&[Point])->Result<(),String>{
        let(b,_)=self.bridge(points)?;let n=self.paths.len();let mut bridge=SectorOperator::prepare(n,1,0,&b,&alloc::vec![0.0;n.pow(4)])?;
        let before=self.rho.clone();let started=self.a.applications;
        let mut apply=|values:&[f64]|->Result<Vec<f64>,String>{let a=self.a.apply(values)?;let b=bridge.apply(values)?;Ok(a.iter().zip(b).map(|(a,b)|a+b).collect())};
        let mut left=alloc::vec![Complex::new(0.0,0.0);n*n];let mut produced=left.clone();
        for j in 0..n{let re=apply(&(0..n).map(|i|before[i*n+j].re).collect::<Vec<_>>())?;let im=apply(&(0..n).map(|i|before[i*n+j].im).collect::<Vec<_>>())?;for i in 0..n{left[i*n+j]=Complex::new(re[i],im[i]);}}
        for i in 0..n{let re=apply(&(0..n).map(|j|left[i*n+j].re).collect::<Vec<_>>())?;let im=apply(&(0..n).map(|j|left[i*n+j].im).collect::<Vec<_>>())?;for j in 0..n{produced[i*n+j]=Complex::new(re[j],im[j]);}}
        let scale=(0..n).map(|i|produced[i*n+i].re).sum::<f64>();if !scale.is_finite()||scale<=0.0{return Err("atomic surface lost positive source scale".into());}
        let frame=crate::sic::wh::ququart_frame();let tolerance=4096.0*f64::EPSILON;let mut returned=alloc::vec![Complex::new(0.0,0.0);n*n];let mut seen=alloc::vec![false;n*n];let mut receipts=Vec::new();
        for a in 2..n{for b in a+1..n{
            let cover=[0,1,a,b];let mut block=Operator::zero(4);for(i,&x)in cover.iter().enumerate(){for(j,&y)in cover.iter().enumerate(){let z=produced[x*n+y];block.set(i,j,Complex::new(z.re/scale,z.im/scale));}}
            let masses=frame.split(&block).map_err(|e|e.to_string())?;let recovered=frame.fuse(&masses).map_err(|e|e.to_string())?;let residual=recovered.distance(&block).map_err(|e|e.to_string())?;
            if residual>tolerance{return Err("atomic surface SIC return exceeds its source bound".into());}
            for(i,&x)in cover.iter().enumerate(){for(j,&y)in cover.iter().enumerate(){let z=recovered.get(i,j);let k=x*n+y;
                if seen[k]&&((returned[k].re/scale-z.re).abs()>tolerance||(returned[k].im/scale-z.im).abs()>tolerance){return Err("atomic surface principal frames disagree".into());}
                returned[k]=Complex::new(z.re*scale,z.im*scale);seen[k]=true;
            }}receipts.push(serde_json::json!({"source_frames":cover,"masses":masses.values.iter().map(|z|[z.re,z.im]).collect::<Vec<_>>(),"residual":residual}));
        }}if seen.iter().any(|v|!*v){return Err("atomic surface has uncovered source coordinates".into());}
        self.rho=returned;
        self.events.push(serde_json::json!({"source_step":step,"source_scale":scale,"input_matrix":before.iter().map(|z|[z.re,z.im]).collect::<Vec<_>>(),"produced_matrix":produced.iter().map(|z|[z.re,z.im]).collect::<Vec<_>>(),"returned_matrix":self.rho.iter().map(|z|[z.re,z.im]).collect::<Vec<_>>(),"geometric_bridge":b,"principal_frames":receipts,"tolerance":tolerance,"native_operator_applications":[self.a.applications-started,bridge.applications]}));
        Ok(())
    }
    pub fn report(&self)->serde_json::Value{
        serde_json::json!({"source_residues":self.residues,"source_frame_paths":self.paths,"source_embedding_weights":self.weights,"left_frame_count":self.split,"source_mean_bond_length":self.length,
            "lift_isometry_signed_residuals":self.lift_residuals,"native_internal_operator":self.h,"retained_matrix":self.rho.iter().map(|z|[z.re,z.im]).collect::<Vec<_>>(),"events":self.events,
            "bridge_definition":"B_ij=L/|x_i-x_j| * |V_i V_j| * dot(n_i,n_j), on different source cycle cores",
            "scope":"atomic-frame one-probe source response with full complex SIC return; molecular electron counts and calibrated binding energies are not assigned"})
    }
    pub fn residues(&self)->[usize;2]{self.residues}
    #[cfg(feature="hosted")]
    pub(crate) fn from_saved(value:&serde_json::Value,atom_count:usize)->Result<Self,String>{
        let paths:Vec<[usize;3]>=serde_json::from_value(value["source_frame_paths"].clone()).map_err(|e|e.to_string())?;
        let weights:Vec<f64>=serde_json::from_value(value["source_embedding_weights"].clone()).map_err(|e|e.to_string())?;
        let split=value["left_frame_count"].as_u64().ok_or("saved surface partition missing")? as usize;
        let residues:[usize;2]=serde_json::from_value(value["source_residues"].clone()).map_err(|e|e.to_string())?;
        let h:Vec<f64>=serde_json::from_value(value["native_internal_operator"].clone()).map_err(|e|e.to_string())?;
        let rows:Vec<[f64;2]>=serde_json::from_value(value["retained_matrix"].clone()).map_err(|e|e.to_string())?;
        let n=paths.len();if n<4||split==0||split>=n||weights.len()!=n||h.len()!=n*n||rows.len()!=n*n||paths.iter().flatten().any(|&i|i>=atom_count){return Err("saved atomic surface dimensions differ from its source".into());}
        if h.iter().chain(&weights).any(|x|!x.is_finite())||rows.iter().flatten().any(|x|!x.is_finite()){return Err("saved atomic surface has nonfinite entries".into());}
        let events=value["events"].as_array().ok_or("saved atomic surface history missing")?.clone();
        let mut a=SectorOperator::prepare(n,1,0,&h,&alloc::vec![0.0;n.pow(4)])?;
        a.applications=events.iter().filter_map(|e|e["native_operator_applications"][0].as_u64()).sum();
        let length=value["source_mean_bond_length"].as_f64().filter(|x|x.is_finite()&&*x>0.0).ok_or("saved surface unit missing")?;
        let lift_residuals=serde_json::from_value(value["lift_isometry_signed_residuals"].clone()).map_err(|e|e.to_string())?;
        Ok(Self{paths,weights,split,residues,a,h,rho:rows.into_iter().map(|z|Complex::new(z[0],z[1])).collect(),length,lift_residuals,events})
    }
}
