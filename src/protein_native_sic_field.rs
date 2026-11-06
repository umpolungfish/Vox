//! Exact canonical WH(4) projector field: x²=2, y²=5, u²=2+y.
use alloc::{vec::Vec,string::String};
use num_bigint::BigInt;
use num_integer::Integer;
use num_traits::{One,Zero};
use core::sync::atomic::{AtomicBool,Ordering};

#[derive(Clone)]
pub(super) struct E{pub(super) c:[BigInt;8],pub(super) d:BigInt}
impl E{
    fn float(value:f64)->Self{
        let bits=value.to_bits();let exponent=((bits>>52)&2047) as i32;
        let mut numerator=BigInt::from(if exponent==0{bits&((1u64<<52)-1)}else{(bits&((1u64<<52)-1))|(1u64<<52)});
        if bits>>63!=0{numerator=-numerator;}
        let power=if exponent==0{-1074}else{exponent-1023-52};
        let denominator=if power>=0{numerator<<=power as usize;BigInt::one()}else{BigInt::one()<<(-power as usize)};
        let mut c=core::array::from_fn(|_|BigInt::zero());c[0]=numerator;Self{c,d:denominator}.normalize()
    }
    fn normalize(mut self)->Self{
        let g=self.c.iter().fold(self.d.clone(),|g,c|g.gcd(c));
        if !g.is_one(){for c in &mut self.c{*c/=&g;}self.d/=g;}self
    }
    fn integer(n:i64)->Self{let mut c=core::array::from_fn(|_|BigInt::zero());c[0]=n.into();Self{c,d:1.into()}}
    fn root(bit:usize)->Self{let mut v=Self::integer(0);v.c[1<<bit]=1.into();v}
    fn add(&self,b:&Self)->Self{Self{c:core::array::from_fn(|i|&self.c[i]*&b.d+&b.c[i]*&self.d),d:&self.d*&b.d}.normalize()}
    fn neg(&self)->Self{Self{c:self.c.clone().map(|x|-x),d:self.d.clone()}}
    fn sub(&self,b:&Self)->Self{self.add(&b.neg())}
    fn divide(&self,n:i64)->Self{assert!(n>0);Self{c:self.c.clone(),d:&self.d*n}.normalize()}
    fn mul(&self,b:&Self)->Self{
        let mut out=Self{c:core::array::from_fn(|_|BigInt::zero()),d:&self.d*&b.d};
        for i in 0..8{for j in 0..8{
            if self.c[i].is_zero()||b.c[j].is_zero(){continue;}
            let mut a=&self.c[i]*&b.c[j];let base=i^j;
            if i&j&1!=0{a*=2;}if i&j&2!=0{a*=5;}
            if i&j&4==0{out.c[base]+=&a;}else{
                out.c[base]+=&a*2;
                out.c[base^2]+=&a*if base&2!=0{5}else{1};
            }
        }}out.normalize()
    }
    fn equals_integer(&self,n:i64)->bool{
        self.c[0]==&self.d*n&&self.c[1..].iter().all(Zero::is_zero)
    }
}

/// Execute the canonical exact split and full dual synthesis on this input.
/// A cache reuses only equal numerical operators; callers retain source ancestry.
pub(super) fn return_operator(input:&[(f64,f64)])->Result<Vec<(f64,f64)>,String>{
    if input.len()!=16||input.iter().any(|(r,i)|!r.is_finite()||!i.is_finite()){return Err("invalid exact SIC source operator".into());}
    #[cfg(feature="hosted")]
    let key:Vec<_>=input.iter().flat_map(|(r,i)|[r.to_bits(),i.to_bits()]).collect();
    #[cfg(feature="hosted")]
    static CACHE:std::sync::OnceLock<std::sync::Mutex<alloc::collections::BTreeSet<Vec<u64>>>>=std::sync::OnceLock::new();
    #[cfg(feature="hosted")]
    if CACHE.get_or_init(Default::default).lock().unwrap().contains(&key){return Ok(input.to_vec());}
    let source:Vec<_>=input.iter().map(|&(r,i)|Z{re:E::float(r),im:E::float(i)}).collect();
    let zero=|z:&Z|z.re.equals_integer(0)&&z.im.equals_integer(0);
    let projectors=projectors();let mut returned=alloc::vec![Z::zero();16];
    for projector in projectors{
        let mut mass=Z::zero();
        for i in 0..4{for j in 0..4{let z=&source[i*4+j];if !zero(z){mass=mass.add(&projector[j*4+i].mul(z));}}}
        mass=mass.divide(4);
        for output in 0..16{
            let mut dual=projector[output].scale(5);if output/4==output%4{dual.re=dual.re.sub(&E::integer(1));}
            returned[output]=returned[output].add(&mass.mul(&dual));
        }
    }
    for (a,b) in returned.iter().zip(&source){
        if !zero(&a.sub(b)){return Err("exact canonical SIC dual did not return the full source matrix".into());}
    }
    #[cfg(feature="hosted")]
    CACHE.get().unwrap().lock().unwrap().insert(key);
    // Exact equality establishes that these binary64 values serialize the
    // returned field elements without a second numerical approximation.
    Ok(input.to_vec())
}
#[derive(Clone)]
pub(super) struct Z{pub(super) re:E,pub(super) im:E}
impl Z{
    fn zero()->Self{Self{re:E::integer(0),im:E::integer(0)}}
    fn add(&self,b:&Self)->Self{Self{re:self.re.add(&b.re),im:self.im.add(&b.im)}}
    fn sub(&self,b:&Self)->Self{Self{re:self.re.sub(&b.re),im:self.im.sub(&b.im)}}
    fn mul(&self,b:&Self)->Self{Self{re:self.re.mul(&b.re).sub(&self.im.mul(&b.im)),im:self.re.mul(&b.im).add(&self.im.mul(&b.re))}}
    fn conj(&self)->Self{Self{re:self.re.clone(),im:self.im.neg()}}
    fn scale(&self,n:i64)->Self{Self{re:self.re.mul(&E::integer(n)),im:self.im.mul(&E::integer(n))}}
    fn divide(&self,n:i64)->Self{Self{re:self.re.divide(n),im:self.im.divide(n)}}
}
pub(super) fn projectors()->Vec<Vec<Z>>{
    let x=E::root(0);let y=E::root(1);let u=E::root(2);let one=E::integer(1);
    // Every fiducial component has common a*c. Its squared scale is in
    // Q(x,y); sin(pi/8)=cos(pi/8)*(x-1) on the positive branch.
    let v=x.sub(&one);let weight=E::integer(5).sub(&y).mul(&E::integer(2).add(&x)).divide(160);
    let psi=[Z{re:E::integer(2),im:E::integer(0)},
        Z{re:one.sub(&u).mul(&v),im:one.add(&u)},Z{re:E::integer(0),im:E::integer(2).mul(&v)},
        Z{re:one.add(&u).mul(&v),im:one.sub(&u)}];
    let mut out=Vec::new();
    for p in 0..4{for q in 0..4{
        let ray=core::array::from_fn::<_,4,_>(|n|{let k=(n+4-p)%4;let z=&psi[k];
            match q*k%4{0=>z.clone(),1=>Z{re:z.im.neg(),im:z.re.clone()},2=>Z::zero().sub(z),_=>Z{re:z.im.clone(),im:z.re.neg()}}});
        out.push((0..16).map(|i|{let z=ray[i/4].mul(&ray[i%4].conj());
            Z{re:z.re.mul(&weight),im:z.im.mul(&weight)}}).collect());
    }}out
}
static CHECKED:AtomicBool=AtomicBool::new(false);
pub(super) fn certify()->Result<(),String>{
    if CHECKED.load(Ordering::Acquire){return Ok(());}
    let p=projectors();
    for input in 0..16{for output in 0..16{
        let mut sum=Z::zero();
        for pi in &p{
            let mass=pi[4*(input%4)+input/4].divide(4);
            let mut dual=pi[output].scale(5);
            if output/4==output%4{dual.re=dual.re.sub(&E::integer(1));}
            sum=sum.add(&mass.mul(&dual));
        }
        if !sum.re.equals_integer(i64::from(input==output))||!sum.im.equals_integer(0){
            return Err("canonical radical SIC dual map is not identity".into());
        }
    }}CHECKED.store(true,Ordering::Release);Ok(())
}
