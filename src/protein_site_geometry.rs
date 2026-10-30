//! FOUR character sites in carried backbone frames, including frame derivatives.
use alloc::{vec,vec::Vec};
use crate::{belnap::B4, protein_native::vertex};

pub type Point=(f64,f64,f64);
fn add(a:Point,b:Point)->Point{(a.0+b.0,a.1+b.1,a.2+b.2)}
fn sub(a:Point,b:Point)->Point{(a.0-b.0,a.1-b.1,a.2-b.2)}
fn scale(a:Point,s:f64)->Point{(a.0*s,a.1*s,a.2*s)}
fn dot(a:Point,b:Point)->f64{a.0*b.0+a.1*b.1+a.2*b.2}
fn cross(a:Point,b:Point)->Point{(a.1*b.2-a.2*b.1,a.2*b.0-a.0*b.2,a.0*b.1-a.1*b.0)}
fn norm(a:Point)->f64{libm::sqrt(dot(a,a))}
fn unit(a:Point)->Point{scale(a,1.0/norm(a))}
fn normalize_pullback(raw:Point,gradient:Point)->Point{
    let n=unit(raw);scale(sub(gradient,scale(n,dot(n,gradient))),1.0/norm(raw))
}
fn coordinates(value:usize)->Point{
    // Unit radius is the native singleton-B equilibrium. FOUR supplies the
    // three character coordinates and their Gram normalization.
    let v=vertex(B4::from_u8(value as u8));scale(v,1.0/norm(v))
}

pub struct Frames {
    pub sites:Vec<[Point;4]>,
    bases:Vec<(Point,Point,Point)>,
    triples:Vec<Option<[usize;3]>>,
}
impl Frames {
    pub fn new(points:&[Point])->Self{
        let mut sites=Vec::new();let mut bases=Vec::new();let mut triples=Vec::new();
        for i in 0..points.len(){
            let (e,t,n,triple)=if points.len()>=3{
                let [a,b,c]=if i>=2{[i-2,i-1,i]}else{[0,1,2]};
                let e=unit(sub(points[c],points[b]));
                let n=unit(cross(sub(points[b],points[a]),e));
                (e,cross(n,e),n,Some([a,b,c]))
            }else{((1.0,0.0,0.0),(0.0,1.0,0.0),(0.0,0.0,1.0),None)};
            bases.push((e,t,n));triples.push(triple);
            sites.push(core::array::from_fn(|v|{
                let q=coordinates(v);add(points[i],add(scale(e,q.0),add(scale(t,q.1),scale(n,q.2))))
            }));
        }
        Self{sites,bases,triples}
    }

    /// Return forces through the same carried frame construction. Treating
    /// frame orientation as a constant would drop part of the torsion force.
    pub fn pullback(&self,points:&[Point],site_gradient:&[[Point;4]])->Vec<Point>{
        let mut out=vec![(0.0,0.0,0.0);points.len()];
        for i in 0..points.len(){
            let (e,_,n)=self.bases[i];
            let (mut ge,mut gt,mut gn)=((0.0,0.0,0.0),(0.0,0.0,0.0),(0.0,0.0,0.0));
            for (value,&g) in site_gradient[i].iter().enumerate(){
                out[i]=add(out[i],g);let q=coordinates(value);
                ge=add(ge,scale(g,q.0));gt=add(gt,scale(g,q.1));gn=add(gn,scale(g,q.2));
            }
            if let Some([a,b,c])=self.triples[i]{
                // tangent = normal × axis
                gn=add(gn,cross(e,gt));ge=add(ge,cross(gt,n));
                let u=sub(points[b],points[a]);
                let gw=normalize_pullback(cross(u,e),gn);
                let gu=cross(e,gw);ge=add(ge,cross(gw,u));
                let gv=normalize_pullback(sub(points[c],points[b]),ge);
                out[a]=sub(out[a],gu);out[b]=add(out[b],sub(gu,gv));out[c]=add(out[c],gv);
            }
        }
        out
    }
}

#[cfg(test)]
mod tests{
    use super::*;
    #[test]
    fn moving_character_frames_preserve_gram_and_return_every_derivative(){
        let mut points=vec![(0.0,0.0,0.0),(1.0,0.0,0.0),(1.3,0.8,0.1),
            (1.8,1.2,0.9),(2.4,0.5,1.4)];
        let frames=Frames::new(&points);
        for (i,sites) in frames.sites.iter().enumerate(){for a in 0..4{for b in 0..4{
            let gram=dot(sub(sites[a],points[i]),sub(sites[b],points[i]));
            let expected=if a==b{1.0}else{-1.0/3.0};
            assert!((gram-expected).abs()<32.0*f64::EPSILON);
        }}}
        let gradient:Vec<[Point;4]>=(0..points.len()).map(|i|core::array::from_fn(|v|
            (i as f64+0.3,v as f64-0.7,(i*v) as f64+0.1))).collect();
        let analytic=frames.pullback(&points,&gradient);
        let functional=|p:&[Point]|{
            let f=Frames::new(p);let mut s=0.0;
            for i in 0..p.len(){for v in 0..4{s+=dot(f.sites[i][v],gradient[i][v]);}}s
        };
        let delta=libm::cbrt(f64::EPSILON);
        for i in 0..points.len(){for axis in 0..3{
            let original=points[i];
            match axis{0=>points[i].0+=delta,1=>points[i].1+=delta,_=>points[i].2+=delta};let high=functional(&points);
            points[i]=original;
            match axis{0=>points[i].0-=delta,1=>points[i].1-=delta,_=>points[i].2-=delta};let low=functional(&points);
            points[i]=original;
            let measured=(high-low)/(2.0*delta);
            let expected=match axis{0=>analytic[i].0,1=>analytic[i].1,_=>analytic[i].2};
            assert!((expected-measured).abs()<libm::sqrt(delta)*(1.0+measured.abs()));
        }}
    }
}
