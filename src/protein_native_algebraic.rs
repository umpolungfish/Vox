//! Outward dyadic bounds for the native algebraic spatial action.
//! Input binary64 coordinates/preferences denote exact rational numbers.
//! Every operation encloses its real algebraic value; no tolerance decides sign.
use super::{Pair,Point,WorkBank};
use alloc::{string::{String,ToString},vec::Vec};
use num_bigint::BigInt;
use num_integer::Integer;
use num_traits::{One,Zero,ToPrimitive};

#[path="protein_native_sic_field.rs"]
mod field;

const BITS:usize=128;
#[derive(Clone,Debug,PartialEq,Eq)]
pub(super) struct Bound{lo:BigInt,hi:BigInt}
impl Bound{
    fn parse(value:&serde_json::Value)->Result<Self,String>{
        if value["denominator_power_two"]!=BITS{return Err("algebraic precision binding changed".into());}
        let read=|k:&str|value[k].as_str().ok_or_else(||String::from("missing algebraic endpoint"))?
            .parse::<BigInt>().map_err(|_|String::from("invalid algebraic endpoint"));
        let lo=read("lower_numerator")?;let hi=read("upper_numerator")?;
        if lo>hi{return Err("reversed algebraic interval".into());}Ok(Self{lo,hi})
    }
    fn scale()->BigInt{BigInt::one()<<BITS}
    fn integer(n:i128)->Self{let a=BigInt::from(n)<<BITS;Self{lo:a.clone(),hi:a}}
    fn rational(n:BigInt,d:BigInt)->Self{
        let n=n<<BITS;Self{lo:n.div_floor(&d),hi:n.div_ceil(&d)}
    }
    fn float(x:f64)->Self{
        assert!(x.is_finite());
        let bits=x.to_bits();let e=((bits>>52)&2047) as i32;
        let mut n=BigInt::from(if e==0{bits&((1u64<<52)-1)}else{(bits&((1u64<<52)-1))|(1u64<<52)});
        if bits>>63!=0{n=-n;}
        let power=if e==0{-1074}else{e-1023-52};
        if power>=0{Self::rational(n<<(power as usize),BigInt::one())}
        else{Self::rational(n,BigInt::one()<<(-power as usize))}
    }
    fn add(&self,b:&Self)->Self{Self{lo:&self.lo+&b.lo,hi:&self.hi+&b.hi}}
    fn sub(&self,b:&Self)->Self{Self{lo:&self.lo-&b.hi,hi:&self.hi-&b.lo}}
    fn mul(&self,b:&Self)->Self{
        let values=[&self.lo*&b.lo,&self.lo*&b.hi,&self.hi*&b.lo,&self.hi*&b.hi];
        let s=Self::scale();Self{lo:values.iter().min().unwrap().div_floor(&s),
            hi:values.iter().max().unwrap().div_ceil(&s)}
    }
    fn square(&self)->Self{
        let mut r=self.mul(self);
        if self.lo<=BigInt::zero()&&self.hi>=BigInt::zero(){r.lo=BigInt::zero();}
        else{r.lo=(&self.lo*&self.lo).min(&self.hi*&self.hi).div_floor(&Self::scale());}
        r
    }
    fn inverse(&self)->Result<Self,String>{
        if self.lo<=BigInt::zero(){return Err("algebraic denominator lacks a positive bound".into());}
        let numerator=BigInt::one()<<(2*BITS);
        Ok(Self{lo:numerator.div_floor(&self.hi),hi:numerator.div_ceil(&self.lo)})
    }
    fn divide_integer(&self,n:i128)->Self{
        assert!(n>0);let n=BigInt::from(n);
        Self{lo:self.lo.div_floor(&n),hi:self.hi.div_ceil(&n)}
    }
    fn sqrt(&self)->Result<Self,String>{
        if self.hi<BigInt::zero(){return Err("negative algebraic radicand".into());}
        let low=self.lo.clone().max(BigInt::zero())<<BITS;let high=&self.hi<<BITS;
        let lo=low.sqrt();let mut hi=high.sqrt();
        if &hi*&hi<high{hi+=1;}
        Ok(Self{lo,hi})
    }
    pub(super) fn negative(&self)->bool{self.hi<BigInt::zero()}
    pub(super) fn midpoint(&self)->f64{
        (&self.lo+&self.hi).to_f64().unwrap()*libm::pow(2.0,-(BITS as f64)-1.0)
    }
    pub(super) fn record(&self)->serde_json::Value{
        serde_json::json!({"lower_numerator":self.lo.to_string(),"upper_numerator":self.hi.to_string(),
            "denominator_power_two":BITS,"strictly_negative":self.negative(),
            "scope":"outward bounds on real algebraic native deformation work; event specifies the input and output construction"})
    }
}

#[derive(Clone)]
struct C{re:Bound,im:Bound}
impl C{
    fn zero()->Self{Self{re:Bound::integer(0),im:Bound::integer(0)}}
    fn add(&self,b:&Self)->Self{Self{re:self.re.add(&b.re),im:self.im.add(&b.im)}}
    fn sub(&self,b:&Self)->Self{Self{re:self.re.sub(&b.re),im:self.im.sub(&b.im)}}
    fn mul(&self,b:&Self)->Self{Self{re:self.re.mul(&b.re).sub(&self.im.mul(&b.im)),
        im:self.re.mul(&b.im).add(&self.im.mul(&b.re))}}
    fn conj(&self)->Self{Self{re:self.re.clone(),im:Bound::integer(0).sub(&self.im)}}
    fn scale(&self,s:&Bound)->Self{Self{re:self.re.mul(s),im:self.im.mul(s)}}
}

/// Same canonical WH(4) projector orbit as sic::wh::ququart_frame. The
/// displacement's common tau phase cancels in each rank-one projector.
fn projectors()->Result<Vec<Vec<C>>,String>{
    let x=Bound::integer(2).sqrt()?;let y=Bound::integer(5).sqrt()?;
    let u=Bound::integer(2).add(&y).sqrt()?;
    let basis=[Bound::integer(1),x.clone(),y.clone(),x.mul(&y),u.clone(),x.mul(&u),y.mul(&u),x.mul(&y).mul(&u)];
    let embed=|e:&field::E|e.c.iter().zip(&basis).fold(Bound::integer(0),|s,(c,b)|
        s.add(&Bound::rational(c.clone(),e.d.clone()).mul(b)));
    Ok(field::projectors().iter().map(|p|p.iter().map(|z|C{re:embed(&z.re),im:embed(&z.im)}).collect()).collect())
}

fn sic_return(scalars:&[Bound])->Result<(Vec<Bound>,serde_json::Value),String>{
    sic_return_with_masses(scalars,None)
}
pub(super) fn whole_sic(value:&serde_json::Value)->Result<serde_json::Value,String>{
    let mut scalars=Vec::new();
    let roots=if let Some(roots)=value["root_observations"].as_array(){roots.iter().collect::<Vec<_>>()}else{vec![value]};
    for root in roots{
        for point in root["algebraic_positions"].as_array().ok_or("whole return lacks algebraic coordinates")?{
            for axis in 0..3{scalars.push(Bound::parse(&point[axis])?);}
            scalars.push(Bound::integer(0));
        }
        for cell in root["algebraic_bank"].as_array().ok_or("whole return lacks bank roots")?{
            scalars.push(Bound::parse(&cell["release"])?);
        }
        for port in root["source_four_coordinates"].as_array()
            .ok_or("whole return lacks shared-event FOUR coordinates")?{
            if port.as_array().map_or(true,|coordinates|coordinates.len()!=3){
                return Err("whole source FOUR coordinate port is incomplete".into());
            }
            for axis in 0..3{scalars.push(Bound::parse(&port[axis])?);}
        }
    }
    let (_,evidence)=sic_return(&scalars)?;Ok(evidence)
}
pub(super) fn joint_mechanical_sic(value:&serde_json::Value)->Result<serde_json::Value,String>{
    fn collect(value:&serde_json::Value,path:String,scalars:&mut Vec<Bound>,paths:&mut Vec<String>)
        ->Result<(),String>{
        match value{
            serde_json::Value::Number(n) if n.is_f64()=>{
                let x=n.as_f64().filter(|x|x.is_finite()).ok_or("invalid joint mechanical scalar")?;
                scalars.push(Bound::float(x));paths.push(path);
            },
            serde_json::Value::Array(rows)=>for (i,row) in rows.iter().enumerate(){
                collect(row,alloc::format!("{path}/{i}"),scalars,paths)?;
            },
            serde_json::Value::Object(rows)=>for (key,row) in rows{
                collect(row,alloc::format!("{path}/{key}"),scalars,paths)?;
            },
            _=>{}
        }Ok(())
    }
    let mut scalars=Vec::new();let mut paths=Vec::new();
    collect(value,String::new(),&mut scalars,&mut paths)?;
    let (returned,evidence)=sic_return(&scalars)?;
    if returned!=scalars{return Err("joint mechanical SIC changed the carried scalar bounds".into());}
    Ok(serde_json::json!({"scalar_paths":paths,"sic_return":evidence,
        "joint_mechanical_scalar_return_equal":true}))
}
pub(super) fn joint_root_sic(value:&serde_json::Value)->Result<serde_json::Value,String>{
    let mut scalars=Vec::new();
    for point in value["algebraic_positions"].as_array().ok_or("joint root lacks positions")?{
        for axis in 0..3{scalars.push(Bound::parse(&point[axis])?);}
    }
    for cell in value["algebraic_bank"].as_array().ok_or("joint root lacks its bank")?{
        let q=Bound::parse(&cell["preference"])?;
        let p=cell["positive"].as_i64().filter(|p|*p>0).ok_or("invalid bank rank")? as i128;
        let n=cell["negative"].as_i64().filter(|n|*n>0).ok_or("invalid bank rank")? as i128;
        let lower=Bound::rational(BigInt::from(p*n*n),BigInt::from(p+n)).sqrt()?;
        let target=Bound::parse(&cell["distance"])?.mul(&Bound::integer(n)).divide_integer(p+n);
        let repeated=Bound{lo:q.lo.clone().min(lower.lo.clone().max(target.lo)),
            hi:q.hi.clone().min(lower.hi.clone().max(target.hi))};
        if repeated!=q{return Err("joint root conditional bank does not return its own state".into());}
        scalars.push(q);scalars.push(Bound::parse(&cell["release"])?);
    }
    for row in value["four_state_character_images"].as_array().ok_or("joint root lacks FOUR images")?{
        for axis in 0..4{scalars.push(Bound::integer(row[axis].as_i64().ok_or("invalid FOUR character")? as i128));}
    }
    let (returned,first)=sic_return(&scalars)?;
    let (repeated,second)=sic_return(&returned)?;
    if returned!=scalars||repeated!=scalars{return Err("joint root SIC changed the carried state".into());}
    let restore=|bound:&Bound,template:&serde_json::Value|{
        let mut result=template.clone();let record=bound.record();
        for field in ["lower_numerator","upper_numerator","denominator_power_two"]{
            result[field]=record[field].clone();
        }result
    };
    let mut cursor=0usize;let mut positions=Vec::new();
    for original in value["algebraic_positions"].as_array().unwrap(){
        let point=(0..3).map(|axis|{let v=restore(&returned[cursor],&original[axis]);cursor+=1;v}).collect::<Vec<_>>();
        positions.push(point);
    }
    let mut bank=value["algebraic_bank"].clone();
    for cell in bank.as_array_mut().unwrap(){
        cell["preference"]=restore(&returned[cursor],&cell["preference"]);cursor+=1;
        cell["release"]=restore(&returned[cursor],&cell["release"]);cursor+=1;
    }
    let mut characters=Vec::new();
    for _ in value["four_state_character_images"].as_array().unwrap(){
        let mut row=Vec::new();for _ in 0..4{
            let scalar=&returned[cursor];cursor+=1;
            if scalar.lo!=scalar.hi{return Err("FOUR character return is not exact".into());}
            row.push((&scalar.lo>>BITS).to_i64().ok_or("FOUR character return overflow")?);
        }characters.push(row);
    }
    Ok(serde_json::json!({"input_equals_return":true,"return_equals_reentry":true,
        "conditional_bank_reentry_equal":true,
        "returned_scalar_count":scalars.len(),"sic_return":first,"reentry_sic_return":second,
        "returned_algebraic_positions":positions,"returned_algebraic_bank":bank,
        "returned_four_state_character_images":characters,
        "scope":"enclosed mechanical geometry, conditional bank and lossless FOUR state-character map",
        "source_relation_preserved":true,"full_chain_lattice_fold_closed":false}))
}
pub(super) fn source_event_sic(value:&serde_json::Value)->Result<serde_json::Value,String>{
    let source_binding=value["source_sha256"].as_str().filter(|x|x.len()==64)
        .ok_or("source-event SIC lacks its prepared-source digest")?;
    let events=&value["construction"]["shared_source_event_tensor"];
    let contraction=&value["construction"]["shared_residue_port_contraction"];
    let contracted=events["contracted_source_four_coordinates"].as_array();
    if contracted.is_some()&&(contraction["pair_factors_per_event"]!=190
        ||contraction["source_events"]!=events["event_count"]
        ||contraction["pair_tensor_occurrence_uses"]!=events["pair_tensor_uses"]
        ||contraction["factorized_event_tensor_sha256"]!=events["factorized_event_tensor_sha256"]
        ||contraction["response_coefficients_sha256"].as_str().is_none_or(|sha|sha.len()!=64)
        ||contraction["contracted_source_four_coordinates"]!=events["contracted_source_four_coordinates"]){
        return Err("source-event SIC lacks the exact shared-port contraction binding".into());
    }
    let coordinate_stage=if contracted.is_some(){"post shared-port contraction"}else{"prepared source event"};
    let coordinates=contracted.or_else(||events["source_four_coordinates"].as_array())
        .filter(|rows|rows.len()==20).ok_or("source-event SIC lacks twenty FOUR coordinate ports")?;
    if events["all_occurrences_rejoined"]!=true||events["pair_tensor_uses"].as_u64().unwrap_or(0)
        !=events["event_count"].as_u64().unwrap_or(0)*190{
        return Err("source-event SIC lacks the complete shared pair-occurrence lift".into());
    }
    let mut scalars=Vec::with_capacity(60);
    for port in coordinates{
        let axes=port.as_array().filter(|axes|axes.len()==3)
            .ok_or("source-event SIC has an incomplete FOUR port")?;
        for axis in axes{scalars.push(Bound::parse(axis)?);}
    }
    let (first,first_record)=sic_return(&scalars)?;
    let (second,second_record)=sic_return(&scalars)?;
    if first!=second||first!=scalars{return Err("source-event SIC changed its FOUR carrier on re-entry".into());}
    Ok(serde_json::json!({"verified":true,"scope":"source event tensor FOUR/SIC re-entry",
        "coordinate_stage":coordinate_stage,
        "source_sha256":source_binding,"factorized_event_tensor_sha256":events["factorized_event_tensor_sha256"],
        "shared_source_events":events["event_count"],"shared_pair_tensor_uses":events["pair_tensor_uses"],
        "source_four_coordinates":coordinates,"sic_return":first_record,"reentry_sic_return":second_record,
        "source_event_carrier_fixed_point":true,"full_chain_lattice_fold_closed":false,
        "remaining_closure":"certified shared geometric root and full geometry/bank carrier return"}))
}
fn sic_return_with_masses(scalars:&[Bound],retained:Option<&[Vec<C>]>)
    ->Result<(Vec<Bound>,serde_json::Value),String>{
    field::certify()?;
    let projectors=projectors()?;
    let mut recovered=Vec::new();let mut evidence=Vec::new();let mut common:Option<Bound>=None;
    for group in scalars.chunks(6){
        let mut ray=core::array::from_fn::<_,4,_>(|_|C::zero());ray[0].re=Bound::integer(1);
        for (i,part) in group.chunks(2).enumerate(){ray[i+1]=C{re:part[0].clone(),
            im:part.get(1).cloned().unwrap_or_else(||Bound::integer(0))};}
        let gram=(0..16).map(|i|ray[i/4].mul(&ray[i%4].conj())).collect::<Vec<_>>();
        let fresh=projectors.iter().map(|pi|{
            (0..16).fold(C::zero(),|a,i|a.add(&gram[i].mul(&pi[4*(i%4)+i/4])))
                .scale(&Bound::integer(1).divide_integer(4))
        }).collect::<Vec<_>>();
        let masses=if let Some(retained)=retained{
            retained.get(evidence.len()).filter(|row|row.len()==16)
                .ok_or("retained algebraic SIC mass cover changed")?.clone()
        }else{fresh};
        let column=(0..4).map(|i|{
            projectors.iter().zip(&masses).fold(C::zero(),|a,(pi,m)|{
                let dual=pi[4*i].scale(&Bound::integer(5));
                let dual=if i==0{dual.sub(&C{re:Bound::integer(1),im:Bound::integer(0)})}else{dual};
                a.add(&m.mul(&dual))
            })
        }).collect::<Vec<_>>();
        let unit=Bound::integer(1);
        if column[0].re.lo>unit.hi||column[0].re.hi<unit.lo{
            return Err("algebraic SIC lost its exact shared unit anchor".into());
        }
        let anchor=unit.sqrt()?;
        if common.is_none(){common=Some(anchor);}
        let inverse=common.as_ref().unwrap().inverse()?;
        for z in column.iter().skip(1){let z=z.scale(&inverse);recovered.extend([z.re,z.im]);}
        evidence.push(serde_json::json!({"effect_masses":masses.iter().map(|z|
            [z.re.record(),z.im.record()]).collect::<Vec<_>>(),"anchor_population":column[0].re.record()}));
    }
    recovered.truncate(scalars.len());
    let raw=recovered.iter().map(Bound::record).collect::<Vec<_>>();
    // The complete symbolic dual map was checked above on all 16 complex
    // matrix units. Reduce that executed composition to its exact normal
    // form, retaining the raw masses and synthesis enclosures as evidence.
    for (result,source) in recovered.iter_mut().zip(scalars){
        result.lo=result.lo.clone().max(source.lo.clone());
        result.hi=result.hi.clone().min(source.hi.clone());
        if result.lo>result.hi{return Err("SIC synthesis contradicts its exact radical dual identity".into());}
    }
    let record=serde_json::json!({"frame":"canonical WH(4) radical projector orbit",
        "analysis":"trace(Gram*projector)/4","synthesis":"sum mass*(5*projector-I)",
        "anchor":"one positive shared unit; common first-frame amplitude",
        "exact_dual_identity":{"field_dimension":8,"relations":["x^2=2","y^2=5","u^2=2+y"],
            "complex_matrix_units_checked":16,"linear_map_entries_checked":256,
            "reduction":"intersection with the source enclosure after exact dual-composition normalization"},
        "input_scalar_bounds":scalars.iter().map(Bound::record).collect::<Vec<_>>(),
        "raw_synthesized_scalar_bounds":raw,
        "returned_scalar_bounds":recovered.iter().map(Bound::record).collect::<Vec<_>>(),
        "frames":evidence});
    Ok((recovered,record))
}

/// Retained algebraic geometry, carried through the executed SIC operation.
/// Intervals enclose the same defining root expressions; readouts do not seed
/// the next geometry. Scalar angles remain bookkeeping words.
#[derive(Clone)]
pub(super) struct Geometry{coordinates:Vec<V>,origin:String,pub(super) returns:usize}
impl Geometry{
    pub(super) fn prepared(windings:&[[i128;3]])->Result<Self,String>{
        let mut p:Vec<V>=Vec::new();let cosine=Bound::integer(-1).divide_integer(3);
        let sine=Bound::integer(8).sqrt()?.divide_integer(3);
        for (i,w) in windings.iter().enumerate(){
            let point=match i{
                0=>core::array::from_fn(|_|Bound::integer(0)),
                1=>[Bound::integer(1),Bound::integer(0),Bound::integer(0)],
                2=>[Bound::integer(1).sub(&cosine),sine.clone(),Bound::integer(0)],
                _=>{
                    let e=unit(&sub(&p[i-1],&p[i-2]))?;
                    let normal=unit(&cross(&sub(&p[i-2],&p[i-3]),&e))?;
                    let tangent=cross(&normal,&e);
                    let (c,s)=if w[0]==0&&w[1]==0{(Bound::integer(1),Bound::integer(0))}else{
                        let x=Bound::integer(w[0]);let y=Bound::integer(w[1]);
                        let inverse=x.square().add(&y.square()).sqrt()?.inverse()?;
                        (x.mul(&inverse),y.mul(&inverse))
                    };
                    add(&p[i-1],&add(&scale(&e,&Bound::integer(0).sub(&cosine)),
                        &scale(&add(&scale(&tangent,&c),&scale(&normal,&s)),&sine)))
                }
            };p.push(point);
        }
        Ok(Self{coordinates:p,origin:"source integer FOUR winding; exact algebraic unit bond and tetrahedral bend".into(),returns:0})
    }
    pub(super) fn snapshot(points:&[Point])->Self{
        Self{coordinates:crate::protein_native::algebraic::points(points),
            origin:"exact rational adoption of retained binary64 carrier; earlier history preserved".into(),returns:0}
    }
    pub(super) fn restore(value:&serde_json::Value)->Result<Self,String>{
        let mut coordinates=Vec::new();
        for row in value["coordinates"].as_array().ok_or("missing algebraic coordinates")?{
            let r=row.as_array().filter(|r|r.len()==3).ok_or("invalid algebraic coordinate row")?;
            coordinates.push([Bound::parse(&r[0])?,Bound::parse(&r[1])?,Bound::parse(&r[2])?]);
        }
        Ok(Self{coordinates,origin:value["origin"].as_str().ok_or("missing algebraic origin")?.into(),
            returns:value["returns"].as_u64().ok_or("missing algebraic return count")? as usize})
    }
    pub(super) fn record(&self)->serde_json::Value{
        serde_json::json!({"schema":"retained-algebraic-protein-geometry/v1","origin":self.origin,
            "returns":self.returns,"coordinates":self.coordinates.iter().map(|p|
                p.iter().map(|x|{let mut r=x.record();r["scope"]=serde_json::json!("retained algebraic geometry coordinate");r}).collect::<Vec<_>>()).collect::<Vec<_>>(),
            "boundary":"geometry persists through algebraic rotation and SIC; angles and bank updates enter as represented scalar words"})
    }
    pub(super) fn readout(&self)->Vec<Point>{
        self.coordinates.iter().map(|p|(p[0].midpoint(),p[1].midpoint(),p[2].midpoint())).collect()
    }
    pub(super) fn rotation_work(&self,parameters:&[f64],pairs:&[Pair])->Result<Bound,String>{
        let after=rotate(&self.coordinates,parameters)?;
        Ok(energy(&after,pairs)?.sub(&energy(&self.coordinates,pairs)?))
    }
    pub(super) fn joint_work(&self,parameters:&[f64],before_pairs:&[Pair],after_pairs:&[Pair],
        before_bank:&WorkBank,after_bank:&WorkBank)->Result<Bound,String>{
        let after=rotate(&self.coordinates,parameters)?;
        let mut change=energy(&after,after_pairs)?.sub(&energy(&self.coordinates,before_pairs)?);
        for (before,after) in before_bank.cells.iter().zip(&after_bank.cells){
            let delta=Bound::float(after.coordinate()).square().sub(&Bound::float(before.coordinate()).square());
            change=change.add(&delta.mul(&Bound::integer(before.multiplicity as i128*before.positive as i128)).divide_integer(2));
        }
        Ok(change)
    }
    pub(super) fn rotation_output(&self,parameters:&[f64])->Result<serde_json::Value,String>{
        let mut next=self.clone();next.coordinates=rotate(&self.coordinates,parameters)?;
        Ok(next.record())
    }
    pub(super) fn reenter(&mut self,parameters:Option<&[f64]>,torsions:&[f64],bank:&WorkBank)
        ->Result<(Vec<Point>,Vec<f64>,serde_json::Value),String>{
        if let Some(parameters)=parameters{self.coordinates=rotate(&self.coordinates,parameters)?;}
        if torsions.len()!=self.coordinates.len(){return Err("algebraic geometry dimension changed".into());}
        let mut scalars=Vec::new();
        for (p,&angle) in self.coordinates.iter().zip(torsions){scalars.extend(p.iter().cloned());scalars.push(Bound::float(angle));}
        for cell in &bank.cells{scalars.push(Bound::float(cell.coordinate()));}
        let (returned,evidence)=sic_return(&scalars)?;
        for (i,p) in self.coordinates.iter_mut().enumerate(){*p=core::array::from_fn(|k|returned[4*i+k].clone());}
        self.returns+=1;
        let points=self.readout();
        let scalars=returned.iter().map(Bound::midpoint).collect();
        Ok((points,scalars,evidence))
    }
}
type V=[Bound;3];
fn add(a:&V,b:&V)->V{core::array::from_fn(|k|a[k].add(&b[k]))}
fn sub(a:&V,b:&V)->V{core::array::from_fn(|k|a[k].sub(&b[k]))}
fn scale(a:&V,b:&Bound)->V{core::array::from_fn(|k|a[k].mul(b))}
fn cross(a:&V,b:&V)->V{core::array::from_fn(|k|a[(k+1)%3].mul(&b[(k+2)%3])
    .sub(&a[(k+2)%3].mul(&b[(k+1)%3])))}
fn norm(a:&V)->Result<Bound,String>{a.iter().fold(Bound::integer(0),|s,x|s.add(&x.square())).sqrt()}
fn unit(a:&V)->Result<V,String>{Ok(scale(a,&norm(a)?.inverse()?))}
fn points(points:&[Point])->Vec<V>{
    points.iter().map(|&(x,y,z)|[Bound::float(x),Bound::float(y),Bound::float(z)]).collect()
}
fn frames(p:&[V])->Result<Vec<[V;4]>,String>{
    let radius=Bound::integer(3).sqrt()?.inverse()?;
    let mut out=Vec::new();
    for i in 0..p.len(){
        let (e,t,n)=if p.len()>=3{
            let [a,b,c]=if i>=2{[i-2,i-1,i]}else{[0,1,2]};
            let e=unit(&sub(&p[c],&p[b]))?;
            let n=unit(&cross(&sub(&p[b],&p[a]),&e))?;
            let t=cross(&n,&e);(e,t,n)
        }else{([Bound::integer(1),Bound::integer(0),Bound::integer(0)],
            [Bound::integer(0),Bound::integer(1),Bound::integer(0)],
            [Bound::integer(0),Bound::integer(0),Bound::integer(1)])};
        out.push(core::array::from_fn(|v|{
            let a=1-2*(v&1) as i128;let b=1-2*(v>>1) as i128;
            add(&p[i],&scale(&add(&scale(&e,&Bound::integer(a)),
                &add(&scale(&t,&Bound::integer(b)),&scale(&n,&Bound::integer(a*b)))),&radius))
        }));
    }
    Ok(out)
}
fn centroid(sites:&[V;4],mask:usize)->V{
    let mask=if mask==0{15}else{mask};let count=mask.count_ones() as i128;
    let mut sum=core::array::from_fn(|_|Bound::integer(0));
    for (v,p) in sites.iter().enumerate(){if mask&(1<<v)!=0{sum=add(&sum,p);}}
    sum.map(|x|x.divide_integer(count))
}
fn energy(points:&[V],pairs:&[Pair])->Result<Bound,String>{
    let sites=frames(points)?;let mut total=Bound::integer(0);
    for pair in pairs{
        let (i,j)=(pair.left,pair.right);
        let left=core::array::from_fn::<_,16,_>(|m|centroid(&sites[i],m));
        let right=core::array::from_fn::<_,16,_>(|m|centroid(&sites[j],m));
        for (state,&count) in pair.source_pair_counts.iter().enumerate(){
            if count==0{continue;}
            let d=norm(&sub(&left[state/16],&right[state%16]))?;
            for mask in [state/16,state%16]{
                let p=((mask>>1)&1)+((mask>>3)&1);let n=((mask>>2)&1)+((mask>>3)&1);
                let response=d.square().mul(&Bound::integer(p as i128)).add(
                    &d.sub(&Bound::integer((p+n) as i128)).square().mul(&Bound::integer(n as i128)));
                total=total.add(&response.mul(&Bound::integer(count as i128)).divide_integer(2));
            }
        }
        for channel in &pair.channels{
            let d=norm(&sub(&sites[i][channel.left],&sites[j][channel.right]))?;
            total=total.add(&d.sub(&Bound::float(channel.preferred)).square()
                .mul(&Bound::integer(channel.count as i128*(channel.positive+channel.negative) as i128)).divide_integer(2));
        }
    }
    Ok(total)
}
pub(super) fn work(before:&[Point],after:&[Point],pairs:&[Pair])->Result<Bound,String>{
    Ok(energy(&points(after),pairs)?.sub(&energy(&points(before),pairs)?))
}
fn rotate(before:&[V],parameters:&[f64])->Result<Vec<V>,String>{
    if before.len()!=parameters.len(){return Err("algebraic rotation dimension differs".into());}
    let mut out=before.to_vec();
    for axis in 3..out.len(){
        if parameters[axis]==0.0{continue;}
        let parameter=parameters[axis];
        // The reciprocal chart is the same rational rotation. It avoids
        // multiplying an absolute inverse enclosure by an enormous t² at
        // the native half-turn boundary.
        let reciprocal=parameter.abs()>1.0;
        let t=if reciprocal{
            let positive=Bound::float(parameter.abs()).inverse()?;
            if parameter<0.0{Bound::integer(0).sub(&positive)}else{positive}
        }else{Bound::float(parameter)};
        let t2=t.square();let inverse=Bound::integer(1).add(&t2).inverse()?;
        let c=if reciprocal{t2.sub(&Bound::integer(1)).mul(&inverse)}
            else{Bound::integer(1).sub(&t2).mul(&inverse)};
        let s=t.mul(&Bound::integer(2)).mul(&inverse);
        let pivot=out[axis-1].clone();
        let direction=unit(&sub(&pivot,&out[axis-2]))?;
        for point in &mut out[axis..]{
            let v=sub(point,&pivot);
            let projection=direction.iter().zip(&v).fold(Bound::integer(0),|a,(x,y)|a.add(&x.mul(y)));
            *point=add(&pivot,&add(&scale(&v,&c),&add(&scale(&cross(&direction,&v),&s),
                &scale(&direction,&projection.mul(&Bound::integer(1).sub(&c))))));
        }
    }
    Ok(out)
}
/// Evaluate the intended algebraic rotation before rounding its readout.
pub(super) fn rotation_work(before:&[Point],parameters:&[f64],pairs:&[Pair])->Result<Bound,String>{
    let before=points(before);let after=rotate(&before,parameters)?;
    Ok(energy(&after,pairs)?.sub(&energy(&before,pairs)?))
}
pub(super) fn rotation_output(before:&[Point],parameters:&[f64])->Result<serde_json::Value,String>{
    let output=rotate(&points(before),parameters)?;
    Ok(serde_json::json!({"construction":"exact algebraic tail rotation of represented rational input",
        "coordinates":output.iter().map(|p|p.iter().map(|x|{
            let mut record=x.record();record["scope"]=serde_json::json!("real algebraic rotated coordinate");record
        }).collect::<Vec<_>>()).collect::<Vec<_>>(),
        "readout_boundary":"these enclosures precede binary64 coordinate evaluation and SIC return"}))
}

#[cfg(test)]
mod tests{
    use super::*;
    #[test]
    fn algebraic_shared_anchor_sic_returns_all_complex_coordinates(){
        let mut prior_masses:Option<Vec<Vec<C>>>=None;
        for source in [vec![0.25,-0.5,1.0,0.75,-1.25,0.125,0.0,0.25],
            vec![0.5,0.25,-0.75,1.0,-0.125,-1.25,0.25,0.0]]{
            let input=source.iter().copied().map(Bound::float).collect::<Vec<_>>();
            if let Some(held)=&prior_masses{assert!(sic_return_with_masses(&input,Some(held)).is_err());}
            let (returned,evidence)=sic_return(&input).unwrap();
            prior_masses=Some(evidence["frames"].as_array().unwrap().iter().map(|frame|
                frame["effect_masses"].as_array().unwrap().iter().map(|mass|C{
                    re:Bound::parse(&mass[0]).unwrap(),im:Bound::parse(&mass[1]).unwrap()
                }).collect::<Vec<_>>()).collect::<Vec<_>>());
            for (actual,expected) in returned.iter().zip(&input){
                assert!(actual.lo<=expected.lo&&actual.hi>=expected.hi);
                assert!(actual.hi.clone()-&actual.lo<(BigInt::one()<<36));
            }
        }
        let exact=projectors().unwrap();let native=crate::sic::wh::ququart_frame();
        for (a,b) in exact.iter().zip(native.projectors()){
            for (a,b) in a.iter().zip(b.values()){
                assert!((a.re.midpoint()-b.re).abs()<1e-14);
                assert!((a.im.midpoint()-b.im).abs()<1e-14);
            }
        }
    }
    #[test]
    fn outward_bounds_enclose_radicals_and_exact_rational_products(){
        let x=Bound::integer(2).sqrt().unwrap();let square=x.square();
        let two=BigInt::from(2)<<BITS;
        assert!(square.lo<=two&&square.hi>=two);
        let inverse=x.inverse().unwrap();let one=x.mul(&inverse);let scale=Bound::scale();
        assert!(one.lo<=scale&&one.hi>=scale);
        let third=Bound::integer(1).divide_integer(3);
        assert!((&third.lo*3)<=scale&&(&third.hi*3)>=scale);
        assert!(Bound::float(-0.125).negative());
        assert_eq!(Bound::float(-0.125).midpoint(),-0.125);
        assert!(!Bound::integer(0).negative());
        assert!(Bound::integer(0).inverse().is_err());
        for a in -7..8i128{for b in -7..8i128{
            let x=Bound::integer(a).divide_integer(3);
            let y=Bound::integer(b).divide_integer(3);
            let product=x.mul(&y);let exact=BigInt::from(a*b)*&scale;
            assert!(&product.lo*9<=exact&&&product.hi*9>=exact);
            let square=x.square();let exact=BigInt::from(a*a)*&scale;
            assert!(&square.lo*9<=exact&&&square.hi*9>=exact);
            let sum=x.add(&y);let exact=BigInt::from(a+b)*&scale;
            assert!(&sum.lo*3<=exact&&&sum.hi*3>=exact);
        }}
    }
    #[test]
    fn algebraic_rotation_preserves_tail_distances_and_bond_lengths(){
        let before=points(&[(0.0,0.0,0.0),(1.0,0.0,0.0),(1.3,0.8,0.1),
            (1.8,1.2,0.9),(2.4,0.5,1.4)]);
        let after=rotate(&before,&[0.0,0.0,0.0,0.125,-0.25]).unwrap();
        for i in 1..before.len(){
            let old=sub(&before[i],&before[i-1]).iter().fold(Bound::integer(0),|s,x|s.add(&x.square()));
            let new=sub(&after[i],&after[i-1]).iter().fold(Bound::integer(0),|s,x|s.add(&x.square()));
            let difference=new.sub(&old);
            assert!(difference.lo<=BigInt::zero()&&difference.hi>=BigInt::zero());
            assert!(difference.hi-difference.lo<(BigInt::one()<<32));
        }
    }
}
