//! Shared internal-coordinate peptide geometry. All distances are Angstroms.
use alloc::vec::Vec;
pub type Point = (f64, f64, f64);
use super::fold3d::{sin_f64 as sin, cos_f64 as cos, sqrt_f64 as sqrt};
const PI: f64 = core::f64::consts::PI;
const DEG: f64 = PI / 180.0;
pub fn add(a: Point,b: Point)->Point {(a.0+b.0,a.1+b.1,a.2+b.2)}
pub fn sub(a: Point,b: Point)->Point {(a.0-b.0,a.1-b.1,a.2-b.2)}
pub fn scale(a: Point,s:f64)->Point {(a.0*s,a.1*s,a.2*s)}
pub fn dot(a: Point,b: Point)->f64 {a.0*b.0+a.1*b.1+a.2*b.2}
pub fn cross(a: Point,b: Point)->Point {(a.1*b.2-a.2*b.1,a.2*b.0-a.0*b.2,a.0*b.1-a.1*b.0)}
pub fn unit(a: Point)->Point {let l=sqrt(dot(a,a)); assert!(l>1e-12,"degenerate peptide frame");scale(a,1.0/l)}
/// Place D from the actual A-B-C plane, with B-C-D angle and A-B-C-D torsion.
pub fn nerf(a:Point,b:Point,c:Point,length:f64,angle:f64,torsion:f64)->Point {
 let e=unit(sub(c,b)); let normal=unit(cross(sub(b,a),e));let tangent=cross(normal,e);
 add(c,scale(add(scale(e,-cos(angle)),scale(add(scale(tangent,cos(torsion)),scale(normal,sin(torsion))),sin(angle))),length))
}
fn ring(origin:Point,axis:Point,tangent:Point,n:usize,length:f64)->Vec<Point> {
 let radius=length/(2.0*sin(PI/n as f64));
 (0..n).map(|k| {let a=2.0*PI*k as f64/n as f64;
 add(origin,scale(add(scale(axis,1.0-cos(a)),scale(tangent,sin(a))),radius))}).collect()
}
/// Bond lengths distinguish carbonyl, amide, aromatic and aliphatic chemistry.
pub fn bond_length(residue:&str,name:&str,parent:&str)->f64 {
 let name=name.trim();
 if name.starts_with('S') || parent.starts_with('S') {return 1.81;}
 if name.starts_with('O') {return if matches!(residue,"Asp"|"ASP"|"Glu"|"GLU"|"Asn"|"ASN"|"Gln"|"GLN") {1.25} else if matches!(residue,"Tyr"|"TYR") {1.36} else {1.43};}
 if matches!(residue,"Arg"|"ARG") && (parent=="CZ" || name=="CZ") {return 1.33;}
 if name.starts_with('N') {return if matches!(residue,"Asn"|"ASN"|"Gln"|"GLN") {1.33} else {1.47};}
 1.525
}
/// Complete sidechain geometry; B4 supplies torsions, topology supplies connectivity.
pub fn sidechain(residue:&str,n:Point,ca:Point,c:Point,phi:f64,psi:f64)->Vec<(&'static str,Point)> {
 // Solve residue geometry in one canonical local frame. Optimizing a proline
 // ring directly in world coordinates let floating-point ties choose different
 // ring conformers under a rigid rotation of the same backbone.
 let x=unit(sub(n,ca));let toward_c=sub(c,ca);
 let y=unit(sub(toward_c,scale(x,dot(toward_c,x))));let z=cross(x,y);
 let local_n=(1.458,0.0,0.0);
 let local_c=(1.525*cos(111.0*DEG),1.525*sin(111.0*DEG),0.0);
 sidechain_in_frame(residue,local_n,(0.0,0.0,0.0),local_c,phi,psi).into_iter()
  .map(|(name,p)|(name,add(ca,add(scale(x,p.0),add(scale(y,p.1),scale(z,p.2)))))).collect()
}
fn sidechain_in_frame(residue:&str,n:Point,ca:Point,c:Point,phi:f64,psi:f64)->Vec<(&'static str,Point)> {
 let topology=super::fold3d::sidechain_topology(residue);
 let mut atoms:Vec<(&'static str,Point)>=Vec::with_capacity(topology.len());
 if topology.is_empty(){return atoms;}
 // L-alpha stereochemistry is fixed by the backbone, rather than a free chi angle.
 let cb=nerf(c,n,ca,1.53,110.5*DEG,-122.5*DEG);
 atoms.push((" CB ",cb));
 let get=|name:&str,atoms:&Vec<(&'static str,Point)>| ->Point {
  match name {"N"=>n,"CA"=>ca,"C"=>c,_=>atoms.iter().find(|x|x.0.trim()==name).expect("sidechain ancestor").1}
 };
 for &(name,parent,offset) in topology.iter().skip(1) {
  let parent_entry=topology.iter().find(|x|x.0.trim()==parent).expect("sidechain parent");
  let grand=parent_entry.1;
  let great=if grand=="CA" {"N"} else {topology.iter().find(|x|x.0.trim()==grand).expect("sidechain grandparent").1};
  let angle=if matches!(residue,"Asn"|"ASN"|"Asp"|"ASP") && parent=="CG"
   || matches!(residue,"Gln"|"GLN"|"Glu"|"GLU") && parent=="CD"
   || matches!(residue,"Arg"|"ARG") && matches!(parent,"NE"|"CZ") {120.0} else {109.5};
  let chi=if parent=="CB" {phi} else {psi};
  let offset=if name.trim()=="CG2" && matches!(residue,"Ile"|"ILE"|"Thr"|"THR") {-offset}else{offset};
  atoms.push((name,nerf(get(great,&atoms),get(grand,&atoms),get(parent,&atoms),bond_length(residue,name,parent),angle*DEG,(chi+offset)*DEG)));
 }
 if matches!(residue,"Pro"|"PRO") {
  // Close both ring bonds while fitting the remaining pyrrolidine angles.
  let candidate=|torsion:f64,rotation:f64| -> Option<(f64,Point,Point)> {
   let cg=nerf(n,ca,cb,1.50,104.5*DEG,torsion);
   let e=unit(sub(n,cg));let distance=sqrt(dot(sub(n,cg),sub(n,cg)));
   if distance>=2.97 || distance<=0.03 {return None;}
   let along=(1.50*1.50-1.47*1.47+distance*distance)/(2.0*distance);
   let center=add(cg,scale(e,along));
   let radial=sub(ca,center);let u=unit(sub(radial,scale(e,dot(radial,e))));let v=cross(e,u);
   let cd=add(center,scale(add(scale(u,cos(rotation)),scale(v,sin(rotation))),sqrt(1.50*1.50-along*along)));
   let angle_cos=|a:Point,b:Point,c:Point|dot(unit(sub(a,b)),unit(sub(c,b)));
   let errors=[angle_cos(cb,cg,cd)-cos(104.5*DEG),angle_cos(cg,cd,n)-cos(104.5*DEG),angle_cos(cd,n,ca)-cos(110.0*DEG)];
   Some((errors.iter().map(|x|x*x).sum(),cg,cd))
  };
  let mut best=(f64::INFINITY,0.0,0.0,cb,n);
  for i in 0..36 {for j in 0..36 {
   let t=i as f64*10.0*DEG;let r=j as f64*10.0*DEG;
   if let Some((score,cg,cd))=candidate(t,r) {if score<best.0 {best=(score,t,r,cg,cd);}}
  }}
  let mut delta=5.0*DEG;
  for _ in 0..12 {
   let base=best;
   for i in -1..=1 {for j in -1..=1 {
    let t=base.1+i as f64*delta;let r=base.2+j as f64*delta;
    if let Some((score,cg,cd))=candidate(t,r) {if score<best.0 {best=(score,t,r,cg,cd);}}
   }}
   delta*=0.5;
  }
  assert!(best.0.is_finite(),"proline ring closure");
  atoms[1].1=best.3;atoms[2].1=best.4;
 }
 if matches!(residue,"Phe"|"PHE"|"Tyr"|"TYR"|"His"|"HIS"|"Trp"|"TRP") {
  let cg=get("CG",&atoms);let axis=unit(sub(cg,cb));
  let normal=unit(cross(sub(cb,ca),axis));let tangent=cross(normal,axis);
  let six=matches!(residue,"Phe"|"PHE"|"Tyr"|"TYR");
  let vertices=ring(cg,axis,tangent,if six {6}else{5},1.39);
  let names:&[&str]=if six {&["CG","CD1","CE1","CZ","CE2","CD2"]}
   else if matches!(residue,"His"|"HIS") {&["CG","ND1","CE1","NE2","CD2"]}
   else {&["CG","CD1","NE1","CE2","CD2"]};
  for (name,point) in names.iter().zip(vertices.iter()) {atoms.iter_mut().find(|x|x.0.trim()==*name).unwrap().1=*point;}
  if matches!(residue,"Tyr"|"TYR") {let cz=get("CZ",&atoms);atoms.iter_mut().find(|x|x.0.trim()=="OH").unwrap().1=add(cz,scale(axis,1.36));}
  if matches!(residue,"Trp"|"TRP") {
   let a=get("CD2",&atoms);let b=get("CE2",&atoms);let edge=unit(sub(b,a));
   let middle=scale(add(a,b),0.5);let pent_center=add(cg,scale(axis,1.39/(2.0*sin(PI/5.0))));
   let mut outward=unit(cross(normal,edge));if dot(outward,sub(pent_center,middle))>0.0 {outward=scale(outward,-1.0);}
   let hex_center=add(middle,scale(outward,1.39*sqrt(3.0)/2.0));let hex_axis=unit(sub(hex_center,a));
   let hex_tangent=unit(sub(scale(sub(b,a),1.0/1.39),scale(hex_axis,0.5)));
   let fused=ring(a,hex_axis,hex_tangent,6,1.39);
   for (name,point) in ["CD2","CE2","CZ2","CH2","CZ3","CE3"].iter().zip(fused.iter()) {atoms.iter_mut().find(|x|x.0.trim()==*name).unwrap().1=*point;}
  }
 }
 atoms
}
