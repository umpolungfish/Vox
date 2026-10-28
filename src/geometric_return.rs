//! Source-ring return of the SAME four-channel banked geometric operator.
//! All arithmetic remains on definite Belnap numeral rails. The full frame,
//! its inverse, origin and returned amplitudes persist across coordinate changes.
use alloc::{string::{String,ToString},vec::Vec};
use core::cmp::Ordering;
use num_bigint::BigInt;
use num_traits::Signed;
use serde_json::{json,Value};
use crate::{belnap_word::BelnapWord as W,morphism_factor as mf};
type Z=(W,W);
type Matrix=[Z;16];
type Vector=[Z;4];
fn small(n:u64)->W{W::from_parity(&mf::tape_u64(n))}
fn normalize(x:W)->W{W::new(x.0)}
fn reduce(x:&W,n:&W)->W{normalize(x.divide(n).expect("definite return modulus").1)}
fn number(x:&W)->String{mf::dec_of(&x.to_parity().expect("definite return coordinate"))}
fn minus(a:&W,b:&W,n:&W)->W{
    if a.compare(b).unwrap()!=Ordering::Less{normalize(a.subtract(b).0)}
    else{normalize(n.subtract(&normalize(b.subtract(a).0)).0)}
}
fn signed(x:&BigInt,n:&W)->W{
    let tape=mf::decimal_to_tape(&x.abs().to_string()).expect("integer return boundary");
    let value=reduce(&W::from_parity(&tape),n);
    if x.is_negative(){minus(&small(0),&value,n)}else{value}
}
fn record(v:&[Z])->Value{json!(v.iter().map(|(a,b)|[number(a),number(b)]).collect::<Vec<_>>())}
fn equal(a:&[Z],b:&[Z])->bool{a==b}
fn identity()->Matrix{core::array::from_fn(|i|(small(u64::from(i/4==i%4)),small(0)))}
fn product_sum<'a>(terms:impl Iterator<Item=(&'a Z,&'a Z)>,n:&W)->Z{
    let(mut positive,mut negative,mut imaginary)=(small(0),small(0),small(0));
    for ((a,b),(c,d))in terms{
        positive=positive.add(&a.multiply(c));negative=negative.add(&b.multiply(d));
        imaginary=imaginary.add(&a.multiply(d)).add(&b.multiply(c));
    }
    (minus(&reduce(&positive,n),&reduce(&negative,n),n),reduce(&imaginary,n))
}
fn multiply(a:&Matrix,b:&Matrix,n:&W)->Matrix{
    core::array::from_fn(|i|product_sum((0..4).map(|k|(&a[(i/4)*4+k],&b[4*k+i%4])),n))
}
fn apply(a:&Matrix,x:&Vector,n:&W)->Vector{
    core::array::from_fn(|i|product_sum((0..4).map(|k|(&a[4*i+k],&x[k])),n))
}
fn power(a:&Matrix,mut exponent:usize,n:&W)->Matrix{
    let mut result=identity();let mut base=a.clone();
    while exponent!=0{
        if exponent&1!=0{result=multiply(&result,&base,n);}
        exponent>>=1;if exponent!=0{base=multiply(&base,&base,n);}
    }
    result
}
fn gcd(a:&W,b:&W)->W{W::from_parity(&mf::gcd(a.to_parity().unwrap(),b.to_parity().unwrap()))}
fn proper(d:&W,n:&W)->bool{d!=&small(0)&&d!=&small(1)&&d!=n}
fn inverse(x:&W,n:&W)->Result<W,W>{
    mf::mod_inv(&x.to_parity().unwrap(),&n.to_parity().unwrap())
        .map(|v|W::from_parity(&v)).map_err(|v|W::from_parity(&v))
}
fn prime_power_base(m:usize)->Option<usize>{
    let mut d=2;while d<=m/d && m%d!=0{d+=1;}
    if d>m/d{return Some(m);}
    let mut rest=m;while rest%d==0{rest/=d;}
    if rest==1{Some(d)}else{None}
}
pub struct SourceRingReturn{
    source:W,base:Option<Matrix>,current:Matrix,current_inverse:Matrix,
    origin:Vector,exponent:W,bound:usize,ticks:usize,
}
impl SourceRingReturn{
    pub fn prepare(source:&[char])->Self{
        Self{source:W::from_parity(source),base:None,current:identity(),current_inverse:identity(),
            origin:core::array::from_fn(|_|(small(0),small(0))),exponent:small(1),bound:1,ticks:0}
    }
    pub fn learning_fields(&self)->Vec<Value>{
        let mut fields=Vec::new();
        let state=apply(&self.current,&self.origin,&self.source);
        for(name,value)in [("return_exponent",self.exponent.clone()),("return_bound",small(self.bound as u64))]{
            fields.push(json!({"name":name,"word":crate::toroidal_division::word(&value)}));
        }
        for(i,(a,b))in state.iter().enumerate(){
            for(name,value)in [(alloc::format!("return_re_{i}"),a),(alloc::format!("return_im_{i}"),b)]{
                fields.push(json!({"name":name,"word":crate::toroidal_division::word(value)}));
            }
        }
        fields
    }
    /// Reduce the actual rational frame into the source bank, carry the old
    /// return by conjugation, then compose the next prime-power return word.
    pub fn advance(&mut self,matrix:&[(BigInt,BigInt);16],dual:&[(BigInt,BigInt);16],
        normalization:&BigInt,denominator:&BigInt,before:&[(BigInt,BigInt);4])
        ->Result<(Option<W>,Value),String>{
        let n=&self.source;
        let den=signed(denominator,n);let norm=signed(normalization,n);
        let mut inverses=Vec::new();
        for(name,value)in [("operator_denominator",&den),("dual_normalization",&norm)]{
            match inverse(value,n){
                Ok(inv)=>inverses.push(inv),
                Err(d)=>return Ok((proper(&d,n).then_some(d.clone()),json!({
                    "schema":"source-ring-geometric-return-v1","stage":"frame-nonunit",
                    "source":number(n),"coordinate":name,"value":number(value),"gcd":number(&d),
                    "held":!proper(&d,n),"resident_bank_unchanged":true}))),
            }
        }
        let scale=|v:&(BigInt,BigInt),factor:&W|{
            (reduce(&signed(&v.0,n).multiply(factor),n),reduce(&signed(&v.1,n).multiply(factor),n))};
        let frame:Matrix=core::array::from_fn(|i|scale(&matrix[i],&inverses[0]));
        let inv_factor=reduce(&den.multiply(&inverses[1]),n);
        let frame_inverse:Matrix=core::array::from_fn(|i|scale(&dual[i],&inv_factor));
        if !equal(&multiply(&frame_inverse,&frame,n),&identity()){
            return Err("source-ring frame dual failed full-operator inverse".into());
        }
        let initial=self.base.is_none();
        if initial{
            self.origin=core::array::from_fn(|i|(signed(&before[i].0,n),signed(&before[i].1,n)));
            self.base=Some(frame.clone());self.current=frame.clone();self.current_inverse=frame_inverse.clone();
        }else{
            self.base=Some(multiply(&multiply(&frame,self.base.as_ref().unwrap(),n),&frame_inverse,n));
            self.current=multiply(&multiply(&frame,&self.current,n),&frame_inverse,n);
            self.current_inverse=multiply(&multiply(&frame,&self.current_inverse,n),&frame_inverse,n);
            self.origin=apply(&frame,&self.origin,n);
        }
        let origin=self.origin.clone();let prior=self.current.clone();let prior_inverse=self.current_inverse.clone();
        let exponent_before=self.exponent.clone();let bound_before=self.bound;
        let mut crossings=Vec::new();let mut result=None;
        // Eight addressed depths per observation; every prime-power entrance
        // multiplies lcm(1..bound) by its prime, retaining intermediate returns.
        for _ in 0..8{
            self.bound=self.bound.checked_add(1).ok_or("geometric return depth overflow")?;
            let Some(prime)=prime_power_base(self.bound)else{continue};
            self.current=power(&self.current,prime,n);
            self.current_inverse=power(&self.current_inverse,prime,n);
            self.exponent=self.exponent.multiply(&small(prime as u64));
            let state=apply(&self.current,&self.origin,n);
            let defect:Vector=core::array::from_fn(|i|(
                minus(&state[i].0,&self.origin[i].0,n),minus(&state[i].1,&self.origin[i].1,n)));
            let mut common=n.clone();let mut norm=small(0);let mut component_readouts=Vec::new();
            for(a,b)in &defect{
                for value in [a,b]{
                    common=gcd(&common,value);let divisor=gcd(n,value);
                    if result.is_none()&&proper(&divisor,n){result=Some(divisor.clone());}
                    component_readouts.push(number(&divisor));
                }
                norm=reduce(&norm.add(&a.multiply(a)).add(&b.multiply(b)),n);
            }
            let norm_gcd=gcd(n,&norm);
            if result.is_none()&&proper(&norm_gcd,n){result=Some(norm_gcd.clone());}
            crossings.push(json!({"depth":self.bound,"prime":prime,"exponent":number(&self.exponent),
                "state":record(&state),"defect":record(&defect),"joint_return_gcd":number(&common),
                "component_gcds":component_readouts,"defect_norm":number(&norm),"norm_gcd":number(&norm_gcd)}));
            if result.is_some(){break;}
        }
        let state=apply(&self.current,&self.origin,n);
        let recovered=apply(&self.current_inverse,&state,n);
        if recovered!=self.origin||!equal(&multiply(&self.current_inverse,&self.current,n),&identity()){
            return Err("geometric return lost its complete source-bank inverse".into());
        }
        self.ticks+=1;
        Ok((result.clone(),json!({"schema":"source-ring-geometric-return-v1","stage":"return-word",
            "source":number(n),"tick":self.ticks,"initial":initial,"frame":record(&frame),
            "frame_inverse":record(&frame_inverse),"base":record(self.base.as_ref().unwrap()),
            "origin":record(&origin),"prior":record(&prior),"prior_inverse":record(&prior_inverse),
            "exponent_before":number(&exponent_before),"exponent":number(&self.exponent),
            "bound_before":bound_before,"bound":self.bound,"crossings":crossings,
            "current":record(&self.current),"current_inverse":record(&self.current_inverse),
            "state":record(&state),"recovered":record(&recovered),"inverse_verified":true,
            "divisor":result.as_ref().map(number)})))
    }
}
