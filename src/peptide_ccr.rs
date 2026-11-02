//! Source-addressed polynomial coordinate operators. No finite matrix truncation.
//! Momentum is -i hbar d/dq, with hbar retained as a formal symbol.
use alloc::{collections::BTreeMap, string::{String,ToString},vec::Vec};
use num_bigint::BigInt;

#[derive(Clone,Debug,PartialEq,Eq,PartialOrd,Ord)]
struct Monomial { q:BTreeMap<usize,u32>, hbar:u32 }
#[derive(Clone,Debug,PartialEq,Eq)]
struct Polynomial(BTreeMap<Monomial,(BigInt,BigInt)>);
impl Polynomial {
    fn add(&mut self,m:Monomial,re:BigInt,im:BigInt){
        let entry=self.0.entry(m.clone()).or_insert_with(||(0.into(),0.into()));
        entry.0+=re;entry.1+=im;
        if entry.0==BigInt::from(0)&&entry.1==BigInt::from(0){self.0.remove(&m);}
    }
    fn position(&self,axis:usize)->Self{
        let mut out=Self(BTreeMap::new());
        for (m,(re,im)) in &self.0 {
            let mut m=m.clone();*m.q.entry(axis).or_default()+=1;
            out.add(m,re.clone(),im.clone());
        }out
    }
    fn momentum(&self,axis:usize)->Self{
        let mut out=Self(BTreeMap::new());
        for (m,(re,im)) in &self.0 {
            if let Some(&degree)=m.q.get(&axis){
                let mut m=m.clone();
                if degree==1{m.q.remove(&axis);}else{m.q.insert(axis,degree-1);}
                m.hbar+=1;
                out.add(m,im*degree,-re*degree);
            }
        }out
    }
    fn subtract(&self,rhs:&Self)->Self{
        let mut out=self.clone();
        for (m,(re,im)) in &rhs.0{out.add(m.clone(),-re,-im);}out
    }
    fn plus(&self,rhs:&Self)->Self{
        let mut out=self.clone();for (m,(re,im)) in &rhs.0{out.add(m.clone(),re.clone(),im.clone());}out
    }
    fn scaled(&self,scale:i64)->Self{
        let mut out=Self(BTreeMap::new());
        for (m,(re,im)) in &self.0{out.add(m.clone(),re*scale,im*scale);}out
    }
    fn distance_squared(&self,a:usize,b:usize)->Self{
        let mut out=Self(BTreeMap::new());
        for k in 0..3{
            out=out.plus(&self.position(3*a+k).position(3*a+k))
                .plus(&self.position(3*b+k).position(3*b+k))
                .subtract(&self.position(3*a+k).position(3*b+k).scaled(2));
        }out
    }
    fn dilation(&self,b:usize,c:usize)->Self{
        let mut out=Self(BTreeMap::new());
        for k in 0..3{
            let p=self.momentum(3*b+k);
            out=out.plus(&p.position(3*b+k)).subtract(&p.position(3*c+k));
        }out
    }
    fn i_hbar(&self)->Self{
        let mut out=Self(BTreeMap::new());
        for (m,(re,im)) in &self.0{
            let mut m=m.clone();m.hbar+=1;out.add(m,-im,re.clone());
        }out
    }
    fn record(&self)->serde_json::Value{
        serde_json::Value::Array(self.0.iter().map(|(m,(re,im))|
            serde_json::json!({"coordinate_powers":m.q,"hbar_power":m.hbar,
                "real_integer":re.to_string(),"imaginary_integer":im.to_string()})).collect())
    }
}

/// The bridge on composed contacts is a differential operator on the same
/// source coordinates. [C_ab, D_bc] = 2 i hbar (q_b-q_c).(q_b-q_a).
/// Its action is evaluated on the second contact observable C_bc.
pub fn contact_compositions(molecule:&serde_json::Value,paths:&serde_json::Value)
    ->Result<serde_json::Value,String>{
    use alloc::collections::{BTreeMap,BTreeSet};
    let atoms=molecule["atoms"].as_array().ok_or("missing contact coordinate source")?;
    let n=atoms.iter().filter_map(|a|a["residue"].as_u64()).max().ok_or("empty contact coordinate source")? as usize+1;
    let mut boundary_atoms=Vec::new();
    for residue in 0..=n{
        let (r,name)=if residue==n{(n-1,"C")}else{(residue,"N")};
        boundary_atoms.push(atoms.iter().position(|a|a["residue"].as_u64()==Some(r as u64)&&a["name"]==name)
            .ok_or("source boundary lacks its backbone atom")?);
    }
    let pool=paths["path_pool"].as_array().ok_or("missing source path families")?;
    let mut triples=BTreeSet::new();let mut bindings=Vec::new();
    for composition in paths["compositions"].as_array().ok_or("missing contact compositions")?{
        let node=composition["source_node"].as_u64().ok_or("invalid contact node")? as usize;
        let family=paths["node_paths"][node].as_u64().ok_or("missing contact input family")? as usize;
        let rows=pool[family].as_array().ok_or("invalid contact family")?;
        for witness in composition["shared_boundary_witnesses"].as_array().ok_or("missing contact witness")?{
            let left=witness[0].as_u64().ok_or("invalid contact witness")? as usize;
            let right=witness[1].as_u64().ok_or("invalid contact witness")? as usize;
            if rows[left]["end"]!=rows[right]["start"]{return Err("contact bridge lost its shared carried boundary".into());}
            let a=rows[left]["start"][0].as_u64().ok_or("invalid contact boundary")? as usize;
            let b=rows[left]["end"][0].as_u64().ok_or("invalid contact boundary")? as usize;
            let c=rows[right]["end"][0].as_u64().ok_or("invalid contact boundary")? as usize;
            if a>n||b>n||c>n{return Err("contact path leaves its molecular boundary".into());}
            let triple=(boundary_atoms[a],boundary_atoms[b],boundary_atoms[c]);triples.insert(triple);
            bindings.push((node,left,right,witness[2].clone(),triple));
        }
    }
    let mut ids=BTreeMap::new();let mut actions=Vec::new();
    let mut one=Polynomial(BTreeMap::new());
    one.add(Monomial{q:BTreeMap::new(),hbar:0},1.into(),0.into());
    for (id,&(a,b,c)) in triples.iter().enumerate(){
        ids.insert((a,b,c),id);
        let operand=one.distance_squared(b,c);
        let cd=operand.dilation(b,c).distance_squared(a,b);
        let dc=operand.distance_squared(a,b).dilation(b,c);
        let commutator=cd.subtract(&dc);
        let mut angle=Polynomial(BTreeMap::new());
        for k in 0..3{
            let (a,b,c)=(3*a+k,3*b+k,3*c+k);
            angle=angle.plus(&one.position(b).position(b))
                .subtract(&one.position(b).position(a))
                .subtract(&one.position(c).position(b))
                .plus(&one.position(c).position(a));
        }
        let angle=angle.scaled(2);
        let expected=angle.distance_squared(b,c).i_hbar();
        if commutator!=expected{return Err("source contact commutator action failed".into());}
        actions.push(serde_json::json!({"source_atoms":[a,b,c],"operand":operand.record(),
            "contact_after_dilation":cd.record(),"dilation_after_contact":dc.record(),
            "commutator":commutator.record(),"commutator_coefficient":angle.record(),
            "signed_residual":commutator.subtract(&expected).record()}));
    }
    Ok(serde_json::json!({"boundary_atom_map":boundary_atoms,
        "boundary_map":"N of each residue; terminal boundary is C of the final residue",
        "contact_operator":"C_ab = sum_k (q_a,k - q_b,k)^2",
        "dilation_operator":"D_bc = sum_k (q_b,k - q_c,k) P_b,k",
        "relation":"[C_ab,D_bc] = 2 i hbar (q_b-q_c) dot (q_b-q_a)",
        "actions":actions,"source_path_bindings":bindings.into_iter().map(|(node,left,right,result,triple)|
            serde_json::json!({"source_node":node,"input_paths":[left,right],"output_path":result,
                "coordinate_action":ids[&triple]})).collect::<Vec<_>>(),
        "scope":"exact coordinate-operator actions on source contact observables; conformation remains carried symbolically"}))
}

pub fn prepare(molecule:&serde_json::Value)->Result<serde_json::Value,String>{
    let atoms=molecule["atoms"].as_array().ok_or("CCR source lacks molecular atom addresses")?;
    let mut coordinates=Vec::new();
    for (atom,source) in atoms.iter().enumerate(){for axis in ["x","y","z"]{
        coordinates.push(serde_json::json!({"atom":atom,"residue":source["residue"],
            "name":source["name"],"axis":axis}));
    }}
    if coordinates.len()<2{return Err("CCR requires source coordinate addresses".into());}
    let mut actions=Vec::new();
    let bonds=molecule["bonds"].as_array().ok_or("CCR source lacks molecular bonds")?;
    for (bond_index,bond) in bonds.iter().enumerate(){
        let a=bond[0].as_u64().ok_or("invalid CCR bond atom")? as usize;
        let b=bond[1].as_u64().ok_or("invalid CCR bond atom")? as usize;
        let length=BigInt::from(bond[2].as_u64().ok_or("invalid CCR bond length")?);
        if a>=atoms.len()||b>=atoms.len()||a==b{return Err("invalid CCR bond endpoints".into());}
        // Coordinates are in angstroms; source lengths are integer milliangstroms.
        // The exact source constraint is 10^6 |q_a-q_b|^2 - length^2.
        let mut probe=Polynomial(BTreeMap::new());
        probe.add(Monomial{q:BTreeMap::new(),hbar:0},-&length*&length,0.into());
        for k in 0..3{
            for atom in [a,b]{
                let mut q=BTreeMap::new();q.insert(3*atom+k,2);
                probe.add(Monomial{q,hbar:0},1_000_000.into(),0.into());
            }
            let mut q=BTreeMap::new();q.insert(3*a+k,1);q.insert(3*b+k,1);
            probe.add(Monomial{q,hbar:0},(-2_000_000).into(),0.into());
        }
        for axis in [3*a,3*a+1,3*a+2,3*b,3*b+1,3*b+2]{
        let other=(axis+1)%coordinates.len();
        let xp=probe.momentum(axis).position(axis);
        let px=probe.position(axis).momentum(axis);
        let commutator=xp.subtract(&px);
        let cross=probe.momentum(other).position(axis)
            .subtract(&probe.position(axis).momentum(other));
        if commutator!=probe.i_hbar()||!cross.0.is_empty(){
            return Err("source coordinate CCR action failed".into());
        }
        actions.push(serde_json::json!({"source_bond":bond_index,"coordinate":axis,"cross_coordinate":other,
            "input":probe.record(),"XP":xp.record(),"PX":px.record(),
            "commutator":commutator.record(),"cross_commutator":cross.record()}));
    }}
    Ok(serde_json::json!({"representation":"polynomials over Gaussian integers with formal hbar",
        "position_action":"q_i multiplication","momentum_action":"-i hbar derivative_i",
        "source_coordinates":coordinates,"executed_actions":actions,
        "operand":"source covalent squared-distance constraints in angstrom coordinates",
        "relation":"[X_i,P_j] = i hbar delta_ij",
        "scope":"ambient molecular coordinate operators; not a solved molecular state",
        "native_contact_verdict":"pending","domain_node_executed":false}))
}
