//! Exact transport of the carried horn/spindle frame into the source ring.
//! The two source-bound circle coordinates, normal, miniature and outer point
//! remain together. Coordinate nonunits retain their gcd and source witness.
use crate::{belnap::B4,belnap_word::BelnapWord as W,morphism_factor as mf};
use alloc::{vec,vec::Vec,string::{String,ToString}};
use serde_json::{json,Value};
use core::cmp::Ordering;
type Circle=(W,W);
fn small(v:u64)->W{W::from_parity(&mf::tape_u64(v))}
fn numeral(x:&W)->String{mf::dec_of(&x.to_parity().expect("definite frame coordinate"))}
fn field(context:&Value,name:&str)->Result<W,String>{
    let f=context["fields"].as_array().ok_or("frame fields absent")?.iter()
        .find(|f|f["name"]==name).ok_or("frame input absent")?;
    let cells=f["word"]["cells"].as_array().ok_or("frame word absent")?;
    let w=W::new(cells.iter().map(|c|match c.as_str(){
        Some("T")=>Ok(B4::T),Some("F")=>Ok(B4::F),Some("N")=>Ok(B4::N),
        Some("B")=>Ok(B4::B),_=>Err("unknown frame cell".to_string())}).collect::<Result<Vec<_>,_>>()?);
    w.to_parity().map_err(str::to_string)?;Ok(w)
}
fn reduce(v:W,n:&W)->W{W::new(v.divide(n).expect("definite frame modulus").1.0)}
fn add(a:&W,b:&W,n:&W)->W{reduce(a.add(b),n)}
fn mul(a:&W,b:&W,n:&W)->W{reduce(a.multiply(b),n)}
fn sub(a:&W,b:&W,n:&W)->W{
    if a.compare(b).unwrap()==Ordering::Less{W::new(n.subtract(&W::new(b.subtract(a).0.0)).0.0)}
    else{W::new(a.subtract(b).0.0)}
}
fn negative_word(a:&W,n:&W)->W{sub(&small(0),a,n)}
fn gcd(a:&W,n:&W)->W{W::from_parity(&mf::gcd(a.to_parity().unwrap(),n.to_parity().unwrap()))}
fn proper(d:&W,n:&W)->bool{d!=&small(0)&&d!=&small(1)&&d!=n}
fn inv(a:&W,n:&W)->Result<W,W>{mf::mod_inv(&a.to_parity().unwrap(),&n.to_parity().unwrap())
    .map(|v|W::from_parity(&v)).map_err(|d|W::from_parity(&d))}
fn circle(t:&W,n:&W)->Result<Circle,W>{
    let square=mul(t,t,n);let denominator=add(&small(1),&square,n);
    let inverse=inv(&denominator,n)?;
    Ok((mul(&sub(&small(1),&square,n),&inverse,n),mul(&mul(&small(2),t,n),&inverse,n)))
}
fn cmul(x:&Circle,y:&Circle,n:&W)->Circle{
    (sub(&mul(&x.0,&y.0,n),&mul(&x.1,&y.1,n),n),
     add(&mul(&x.0,&y.1,n),&mul(&x.1,&y.0,n),n))
}
fn cpow(x:&Circle,k:i32,n:&W)->Circle{
    let mut base=if k<0{(x.0.clone(),negative_word(&x.1,n))}else{x.clone()};
    let mut exponent=k.unsigned_abs();let mut out=(small(1),small(0));
    while exponent!=0{if exponent&1!=0{out=cmul(&out,&base,n);}exponent>>=1;
        if exponent!=0{base=cmul(&base,&base,n);}}
    out
}
fn wind(z:&[Circle;2],m:[[i32;2];2],n:&W)->[Circle;2]{
    core::array::from_fn(|i|cmul(&cpow(&z[0],m[i][0],n),&cpow(&z[1],m[i][1],n),n))
}
fn record(v:&[W])->Value{json!(v.iter().map(numeral).collect::<Vec<_>>())}
fn circles(z:&[Circle;2])->Value{json!(z.iter().map(|v|vec![numeral(&v.0),numeral(&v.1)]).collect::<Vec<_>>())}
fn point(z:&[Circle;2],major:u64,minor:u64,n:&W)->([W;3],[W;3]){
    let normal=[mul(&z[0].0,&z[1].0,n),mul(&z[0].1,&z[1].0,n),z[1].1.clone()];
    let centre=[mul(&small(major),&z[0].0,n),mul(&small(major),&z[0].1,n),small(0)];
    (core::array::from_fn(|i|add(&centre[i],&mul(&small(minor),&normal[i],n),n)),normal)
}
/// The complete existing source context is retained by the caller. These are
/// arithmetic circle coordinates, with the same polynomial frame identities;
/// their source parameters and winding word remain resident beside readout.
pub fn observe(context:&Value,tick:usize)->Result<(W,W,Option<W>,Value),String>{
    let n=field(context,"source")?;
    let raw_p=field(context,"p")?;let raw_q=field(context,"q")?;
    let height=field(context,"height")?;let negative=field(context,"negative")?;
    if negative!=small(0)&&negative!=small(1){return Err("height sign remains unresolved".into());}
    let product=raw_p.multiply(&raw_q);
    let (delta,borrow)=if negative==small(1){n.subtract(&product)}else{product.subtract(&n)};
    let mut carried=vec![B4::F;n.0.len()];carried.extend(height.0.iter().copied());
    if borrow!=B4::F||W::new(delta.0)!=W::new(carried){return Err("frame lost exact source/product/carry relation".into());}
    let p=reduce(raw_p.clone(),&n);
    let unsigned=reduce(height.clone(),&n);
    let h=if negative==small(1){negative_word(&unsigned,&n)}else{unsigned};
    let nonunit=|which:&str,d:W|{
        let candidate=proper(&d,&n).then_some(d.clone());
        (p.clone(),h.clone(),candidate,json!({"schema":"carried-source-height-torus-frame-v2",
            "stage":"coordinate-nonunit","tick":tick,"coordinate":which,"gcd":numeral(&d),
            "source":numeral(&n),"parameters":[numeral(&p),numeral(&h)],
            "input_pair":[numeral(&raw_p),numeral(&raw_q)],"height":numeral(&height),"height_negative":negative==small(1),"source_relation_verified":true,
            "source_bank_retained":true,"held":!proper(&d,&n)}))};
    let a=match circle(&p,&n){Ok(v)=>v,Err(d)=>return Ok(nonunit("azimuth-parameter",d))};
    let b=match circle(&h,&n){Ok(v)=>(negative_word(&v.0,&n),v.1),Err(d)=>return Ok(nonunit("height-parameter",d))};
    let before=[a,b];
    let m=if tick%2==0{[[-47,256],[-9,49]]}else{[[-38,169],[-9,40]]};
    let mi=[[m[1][1],-m[0][1]],[-m[1][0],m[0][0]]];
    let after=wind(&before,m,&n);let recovered=wind(&after,mi,&n);
    if recovered!=before{return Err("source-ring winding lost its angular carrier".into());}
    for v in &after{if add(&mul(&v.0,&v.0,&n),&mul(&v.1,&v.1,&n),&n)!=small(1){
        return Err("source-ring angular carrier lost unit-circle identity".into());}}
    // Horn, spindle and open actions share precisely these same directions.
    let mut frames=Vec::new();let mut divisors=Vec::new();let mut normal_readout=None;
    for (name,major,minor) in [("horn",1u64,1u64),("spindle",1,2),("open",2,1)]{
        let(outer,normal)=point(&after,major,minor,&n);let(inner,_)=point(&after,major,major,&n);
        let centre:[W;3]=core::array::from_fn(|i|sub(&outer[i],&mul(&small(minor),&normal[i],&n),&n));
        let expected=[mul(&small(major),&after[0].0,&n),mul(&small(major),&after[0].1,&n),small(0)];
        if centre!=expected{return Err("normal subtraction lost the same source centreline".into());}
        for i in 0..3{
            // outer-inner=(minor-major)*normal, using signed ring subtraction.
            let offset=mul(&sub(&small(minor),&small(major),&n),&normal[i],&n);
            if sub(&outer[i],&inner[i],&n)!=offset{return Err("miniature source correspondence failed".into());}
        }
        if name=="spindle" {
            let azimuth=(centre[0].clone(),centre[1].clone());
            let tube=(add(&mul(&normal[0],&azimuth.0,&n),&mul(&normal[1],&azimuth.1,&n),&n),normal[2].clone());
            normal_readout=Some([azimuth,tube]);
        }
        let signed_radius=add(&small(major),&mul(&small(minor),&after[1].0,&n),&n);
        let d=gcd(&signed_radius,&n);if proper(&d,&n){divisors.push(d.clone());}
        frames.push(json!({"name":name,"R":major,"r":minor,"outer":record(&outer),
            "miniature":record(&inner),"normal":record(&normal),"centreline":record(&centre),
            "signed_radius":numeral(&signed_radius),"radial_gcd":numeral(&d),"normal_return_verified":true}));
    }
    // Read the same winding through its norm-one coordinates. Nonunit
    // inverse-chart seams stay observable and can expose a proper source split.
    let readout=normal_readout.ok_or("source spindle readout absent")?;
    if readout!=after{return Err("normal readout changed addressed directions".into());}
    let mut parameters=Vec::new();let mut seams=Vec::new();
    for (index,v) in readout.iter().enumerate(){
        let den=if index==0{add(&small(1),&v.0,&n)}else{sub(&small(1),&v.0,&n)};
        match inv(&den,&n){Ok(inverse)=>{parameters.push(mul(&v.1,&inverse,&n));
            seams.push(json!({"coordinate":index,"denominator":numeral(&den),"gcd":"1","held":false}));},
            Err(d)=>{if proper(&d,&n){divisors.push(d.clone());}
                // Keep the original addressed parameter as continuation at an
                // unresolved chart seam, with the full wound circle retained.
                parameters.push(if index==0{p.clone()}else{h.clone()});
                seams.push(json!({"coordinate":index,"denominator":numeral(&den),"gcd":numeral(&d),"held":true}));}}
    }
    let result=divisors.first().cloned();
    Ok((parameters[0].clone(),parameters[1].clone(),result.clone(),json!({
        "schema":"carried-source-height-torus-frame-v2","stage":"frame-return","tick":tick,
        "source":numeral(&n),"parameters":[numeral(&p),numeral(&h)],
        "input_pair":[numeral(&raw_p),numeral(&raw_q)],"height":numeral(&height),"height_negative":negative==small(1),
        "source_relation_verified":true,"exact_height_zero":height==small(0),
        "source_horn":record(&point(&before,1,1,&n).0),"before":circles(&before),
        "matrix":m,"inverse_matrix":mi,"after":circles(&after),"recovered":circles(&recovered),
        "frames":frames,"normal_readout":circles(&readout),"chart_seams":seams,"readout_parameters":record(&parameters),
        "divisor":result.as_ref().map(numeral),"source_bank_retained":true,"inverse_verified":true})))
}
