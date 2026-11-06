//! Joint-coordinate mechanism recovered from the user's unified compiler.
//! Its original coarse model and the measured cutoff control remain separate.
use alloc::{string::String,vec::Vec};
#[derive(Clone)]
struct Residue { aa:String,r:f64,kd:f64,q:f64 }
#[derive(Clone)]
struct Contact { i:usize,j:usize,a:Option<f64>,b:f64,k1:f64,k2:f64,value:&'static str,kind:&'static str }
#[derive(Clone)]
pub(crate) struct JointConformation {
    residues:Vec<Residue>,contacts:Vec<Contact>,positions:Vec<[f64;3]>,
    memberships:Vec<u8>,unclamped:bool,events:Vec<serde_json::Value>,
    coupled:bool,surface:Option<crate::protein_surface_response::SurfaceResponse>,
    guards:Vec<(usize,usize,f64)>,
    atomic:Option<crate::protein_atomic_conformation::AtomicConformation>,
}
fn source()->serde_json::Value{serde_json::from_str(include_str!("../../measurements/protein_frame_transport_20261101/unified_model_source.json")).expect("exported native model source")}
fn random(state:&mut u32)->f64{
    *state=state.wrapping_add(0x6D2B79F5);
    let mut t=(*state^(*state>>15)).wrapping_mul(1|*state);
    t=t.wrapping_add((t^(t>>7)).wrapping_mul(61|t))^t;
    ((t^(t>>14)) as f64)/4294967296.0
}
fn gaussian(state:&mut u32)->f64{
    let mut u=0.0;while u==0.0{u=random(state);}let v=random(state);
    libm::sqrt(-2.0*libm::log(u))*libm::cos(2.0*core::f64::consts::PI*v)
}
impl JointConformation {
    pub fn prepare(molecule:&serde_json::Value)->Result<Vec<Self>,String>{
        let exported=source();let model=&exported["model"];let mut residues=Vec::new();
        let table=model["AA"].as_object().ok_or("missing unified source residue table")?;
        for name in molecule["residue_identities"].as_array().ok_or("missing joint coordinate source sequence")?{
            let (aa,row)=table.iter().find(|(_,row)|row["n"]==*name).ok_or("unbound unified source residue")?;
            residues.push(Residue{aa:aa.clone(),r:row["r"].as_f64().unwrap(),kd:row["kd"].as_f64().unwrap(),q:row["q"].as_f64().unwrap()});
        }
        let n=residues.len();let mut contacts=Vec::new();let mut memberships=alloc::vec![0;n];
        let hb=model["HB"].as_array().unwrap();
        for i in 0..n{for j in i+3..n{
            let(x,y)=(&residues[i],&residues[j]);let s=x.r+y.r;
            let mut contact=if x.kd>=1.8&&y.kd>=1.8{
                Some(Contact{i,j,a:Some(0.8*s),b:s,k1:(0.3*(x.kd+y.kd)).clamp(0.8,3.0),k2:6.0,value:"",kind:"hydrophobic"})
            }else if x.q*y.q<0.0{
                Some(Contact{i,j,a:Some(0.8*s),b:s,k1:2.5,k2:6.0,value:"",kind:"salt-bridge"})
            }else if hb.iter().any(|v|v==&x.aa)&&hb.iter().any(|v|v==&y.aa){
                Some(Contact{i,j,a:Some(1.05*s),b:s,k1:1.0,k2:6.0,value:"",kind:"h-bond"})
            }else if x.q*y.q>0.0{
                Some(Contact{i,j,a:None,b:model["R_LIKE"].as_f64().unwrap(),k1:0.0,k2:2.0,value:"",kind:"like-charge"})
            }else{None};
            if let Some(c)=&mut contact{
                let (value,mask)=match c.a{None=>("F",4),Some(a) if c.b-a>1e-9=>("B",8),_=>("T",2)};
                c.value=value;memberships[i]|=mask;memberships[j]|=mask;
            }
            if let Some(c)=contact{contacts.push(c);}
        }}
        for i in 0..n{if memberships[i]==0&&(0..n).any(|j|i.abs_diff(j)>=3){memberships[i]=1;}}
        let mut positions=Vec::new();let mut rng=1;
        for i in 0..n{positions.push([(i as f64-n as f64/2.0)*3.6,if i%2==1{0.8}else{-0.8}+0.3*gaussian(&mut rng),0.3*gaussian(&mut rng)]);}
        let original=Self{residues,contacts,positions,memberships,unclamped:false,events:Vec::new(),coupled:false,surface:None,guards:Vec::new(),atomic:None};
        let mut control=original.clone();control.unclamped=true;
        let mut coupled=control.clone();coupled.coupled=true;
        let chain=molecule["residue_identities"].as_array().unwrap().iter().map(|v|v.as_str().unwrap()).collect::<Vec<_>>();
        coupled.guards=crate::peptide_closure::ca_exclusion_relations(&chain);
        let mut atomic=coupled.clone();atomic.atomic=Some(crate::protein_atomic_conformation::AtomicConformation::prepare(molecule)?);
        atomic.positions=atomic.atomic.as_ref().unwrap().positions();
        Ok(alloc::vec![original,control,coupled,atomic])
    }
    fn energy_gradient(&self,model:&serde_json::Value)->(f64,Vec<[f64;3]>){
        let n=self.positions.len();let mut gradient=alloc::vec![[0.0;3];n];let mut energy=0.0;
        for i in 0..n{for j in i+1..n{
            let dx=core::array::from_fn::<_,3,_>(|k|self.positions[i][k]-self.positions[j][k]);
            let r=libm::sqrt(dx.iter().map(|x|x*x).sum::<f64>()).max(1e-6);
            let(mut u,mut g)=(0.0,0.0);
            if j-i<=2{
                let(k,b)=if j-i==1{(model["K_BOND"].as_f64().unwrap(),model["R_BOND"].as_f64().unwrap())}
                    else{(model["K_13"].as_f64().unwrap(),model["R_13"].as_f64().unwrap())};
                u=0.5*k*(r-b)*(r-b);g=k*(r-b);
            }else if let Some(c)=self.contacts.iter().find(|c|c.i==i&&c.j==j){
                if let Some(a)=c.a{if r>a{
                    let cutoff=model["RCUT"].as_f64().unwrap();let rr=if self.unclamped{r}else{r.min(cutoff)};
                    u+=0.5*c.k1*(rr-a)*(rr-a);if self.unclamped||r<cutoff{g+=c.k1*(r-a);}
                }}
                if r<c.b{u+=0.5*c.k2*(r-c.b)*(r-c.b);g+=c.k2*(r-c.b);}
            }else{
                let b=self.residues[i].r+self.residues[j].r;let k=model["K_EX"].as_f64().unwrap();
                if r<b{u+=0.5*k*(r-b)*(r-b);g+=k*(r-b);}
            }
            for axis in 0..3{let f=g*dx[axis]/r;gradient[i][axis]+=f;gradient[j][axis]-=f;}
            energy+=u;
        }}
        if let Some(surface)=&self.surface{
            let(value,ids,g)=surface.potential_gradient(&self.positions);energy+=value;
            for k in 0..3{gradient[ids[0]][k]+=g[k];gradient[ids[1]][k]-=g[k];}
        }
        (energy,gradient)
    }
    pub fn act(&mut self,step:usize,mark:char,molecule:&serde_json::Value,cycles:&[serde_json::Value],gram:Option<[f64;3]>)->Result<(),String>{
        if let Some(atomic)=&mut self.atomic{
            let before=self.positions.clone();atomic.act(step,mark,&before,molecule,gram,cycles)?;
            self.positions=atomic.positions();
            self.events.push(serde_json::json!({"source_step":step,"positions_before":before,"positions_after":self.positions,
                "operation":"native contact and atomic-frame cycle source responses on the same atom coordinates",
                "surface_kernel":"source atomic covalent-frame Gram","coarse_model_executed":false}));
            return Ok(());
        }
        if self.coupled&&mark=='⋈'{
            if self.surface.is_none(){self.surface=crate::protein_surface_response::SurfaceResponse::prepare(molecule,cycles)?;}
            if let Some(surface)=&mut self.surface{surface.advance(step,&self.positions)?;}
        }
        let exported=source();let model=&exported["model"];let before=self.positions.clone();
        let initial=self.energy_gradient(model).0;
        let mut held_steps=0;let mut backtracks=0;let mut maximum_increase=0.0_f64;
        for _ in 0..2000{
            let(energy_before,gradient)=self.energy_gradient(model);
            if self.coupled{
                let previous=self.positions.clone();let mut accepted=false;let mut eta=0.005;
                for trial in 0..20{
                    self.positions=previous.iter().zip(&gradient).map(|(p,g)|core::array::from_fn(|k|p[k]-eta*g[k])).collect();
                    for _ in 0..8{for &(i,j,minimum) in &self.guards{
                        let dx=core::array::from_fn::<_,3,_>(|k|self.positions[i][k]-self.positions[j][k]);
                        let r=libm::sqrt(dx.iter().map(|x|x*x).sum::<f64>());
                        if r>1e-12&&r<minimum{for k in 0..3{let shift=0.5*(minimum-r)*dx[k]/r;self.positions[i][k]+=shift;self.positions[j][k]-=shift;}}
                    }}
                    let admissible=self.guards.iter().all(|&(i,j,m)|{
                        let distance=libm::sqrt((0..3).map(|k|{let d=self.positions[i][k]-self.positions[j][k];d*d}).sum::<f64>());
                        distance>=m-1e-9
                    });
                    let energy_after=self.energy_gradient(model).0;
                    if admissible&&energy_after.is_finite()&&energy_after<=energy_before+1e-10{
                        maximum_increase=maximum_increase.max(energy_after-energy_before);accepted=true;backtracks+=trial;break;
                    }eta*=0.5;
                }
                if !accepted{self.positions=previous;held_steps+=1;}
            }else{
                for(i,p)in self.positions.iter_mut().enumerate(){for k in 0..3{p[k]-=0.005*gradient[i][k];}}
            }
            let center=core::array::from_fn::<_,3,_>(|k|self.positions.iter().map(|p|p[k]).sum::<f64>()/self.positions.len() as f64);
            for p in &mut self.positions{for k in 0..3{p[k]-=center[k];}}
        }
        let(energy,gradient)=self.energy_gradient(model);
        let rms=libm::sqrt(gradient.iter().flat_map(|p|p.iter()).map(|x|x*x).sum::<f64>()/self.positions.len() as f64);
        self.events.push(serde_json::json!({"source_step":step,"steps":2000,"temperature":0,"eta":0.005,
            "positions_before":before,"positions_after":self.positions,"energy_before":initial,"energy_after":energy,
            "gradient_after":gradient,"rms_gradient_after":rms,"full_source_memberships":self.memberships,
            "surface_response":self.surface.as_ref().map(|s|s.report()),
            "coordinate_step":if self.coupled{"native CA guard projection and bounded energy backtracking"}else{"original page fixed step"},
            "source_ca_guards":self.guards,"held_steps":held_steps,"backtracks":backtracks,"maximum_accepted_energy_increase":maximum_increase,
            "operation":"all source contact forces act on the same retained residue coordinates"}));
        Ok(())
    }
    pub fn report(&self)->serde_json::Value{
        serde_json::json!({"source":source(),"mechanism":if self.atomic.is_some(){"atomic_surface_response_control"}else if self.coupled{"cycle_surface_response_control"}else if self.unclamped{"unclamped_attraction_control"}else{"original"},
            "source_sequence":self.residues.iter().map(|r|r.aa.as_str()).collect::<String>(),"positions":self.positions,
            "full_source_memberships":self.memberships,"events":self.events,
            "atomic_spatial_state":self.atomic.as_ref().map(|a|a.report()),
            "contacts":self.contacts.iter().map(|c|serde_json::json!({"i":c.i,"j":c.j,"a":c.a,"b":c.b,"k1":c.k1,"k2":c.k2,"val":c.value,"kind":c.kind})).collect::<Vec<_>>(),
            "scope":if self.atomic.is_some(){"source-bound atomic coordinates with native geometry evidence, retained contact and cycle responses; lattice bridge and full native-fold closure remain held"}else{"recovered source-bound coarse residue model; all-atom lifting, handedness, specific Trp contacts and native folding remain open"}})
    }
}
