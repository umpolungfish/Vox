//! Coordinate evidence for the complete heavy-atom peptide, retained at readout.
//! Thresholds are explicit model checks, not a claim of experimental structure.
#![allow(dead_code)]
use alloc::{string::String,vec::Vec,format};
use crate::peptide_geometry::{Point,sub,dot,cross};
use crate::fold3d::{sin_f64 as sin,cos_f64 as cos,sqrt_f64 as sqrt};
const DEG:f64=core::f64::consts::PI/180.0;
pub const BOND_TOLERANCE:f64=0.02;
pub const ANGLE_TOLERANCE:f64=8.0;
pub const CLASH_OVERLAP:f64=0.4;
#[derive(Debug)]
pub struct Issue {pub kind:&'static str,pub detail:String}
#[derive(Debug,Default)]
pub struct Evidence {
 pub atoms:usize,pub bonds:usize,pub angles:usize,pub rings:usize,
 pub stereocenters:usize,pub torsions:usize,pub nonbonded_pairs:usize,
 pub max_bond_error:f64,pub max_overlap:f64,pub overlap_potential:f64,pub issues:Vec<Issue>,
 pub bond_edges:Vec<(usize,usize)>,
 pub retained_rigid_contacts:Vec<String>,
}
impl Evidence {
 pub fn closed(&self)->bool {self.atoms>0 && self.issues.is_empty()}
 fn issue(&mut self,kind:&'static str,detail:String){self.issues.push(Issue{kind,detail});}
 pub fn report(&self,details:bool)->String {
  let mut s=format!("Geometry closure (heavy-atom model): {}\n  checked: {} atoms, {} bonds, {} angles, {} ring closures, {} stereocenters, {} backbone torsions, {} nonbonded pairs\n  tolerances: bond {:.3} A, backbone angle {:.1} deg, other bonded angles 90..135 deg, clash overlap {:.2} A\n  measured: max bond residual {:.6} A, max nonbonded overlap {:.6} A\n",if self.closed(){"SATISFIED"}else{"OPEN (coordinate conflicts retained)"},self.atoms,self.bonds,self.angles,self.rings,self.stereocenters,self.torsions,self.nonbonded_pairs,BOND_TOLERANCE,ANGLE_TOLERANCE,CLASH_OVERLAP,self.max_bond_error,self.max_overlap);
  for kind in ["inventory","finite","bond","angle","ring","planarity","stereochemistry","torsion","clash"] {
   let count=self.issues.iter().filter(|x|x.kind==kind).count();
   if count>0 {s.push_str(&format!("  {}: {} conflict(s)\n",kind,count));}
  }
  if details {for issue in &self.issues {s.push_str(&format!("  {}: {}\n",issue.kind,issue.detail));}}
  s.push_str(&format!("  retained covalent-frame contact evidence: {} pair(s)\n",self.retained_rigid_contacts.len()));
  s
 }
}
#[derive(Clone,Copy)]
struct Atom<'a>{residue:usize,name:&'a str,xyz:Point}
fn distance(a:Point,b:Point)->f64 {let v=sub(a,b);sqrt(dot(v,v))}
/// One numerical distance/overlap operation for native checks and their callers.
pub(crate) fn nonbonded_measure(a:Point,b:Point,radius_sum:f64)->(f64,f64){
 let measured=distance(a,b);(measured,radius_sum-measured)
}

fn normalized(v:Point)->Option<Point>{let n=distance(v,(0.0,0.0,0.0));if !n.is_finite()||n<1e-10 {None}else{Some((v.0/n,v.1/n,v.2/n))}}
fn angle(a:Point,b:Point,c:Point)->Option<f64>{
 let u=normalized(sub(a,b))?;let v=normalized(sub(c,b))?;let target=dot(u,v).clamp(-1.0,1.0);
 let(mut lo,mut hi)=(0.0,core::f64::consts::PI);
 for _ in 0..48 {let mid=(lo+hi)*0.5;if cos(mid)>target {lo=mid}else{hi=mid}}
 Some((lo+hi)*0.5/DEG)
}
fn check_angle(e:&mut Evidence,atoms:&[Atom<'_>],ids:[usize;3],expected:f64){
 e.angles+=1;let [a,b,c]=ids;
 let measured=angle(atoms[a].xyz,atoms[b].xyz,atoms[c].xyz);
 if measured.is_none()|| (measured.unwrap()-expected).abs()>ANGLE_TOLERANCE {
  e.issue("angle",format!("{}:{}-{}-{} = {:?} deg, expected {:.1}",atoms[b].residue+1,atoms[a].name,atoms[b].name,atoms[c].name,measured,expected));
 }
}
fn check_torsion(e:&mut Evidence,atoms:&[Atom<'_>],ids:[usize;4],expected:f64){
 e.torsions+=1;let [a,b,c,d]=ids;
 let axis=normalized(sub(atoms[c].xyz,atoms[b].xyz));
 let u=normalized(cross(sub(atoms[b].xyz,atoms[a].xyz),sub(atoms[c].xyz,atoms[b].xyz)));
 let v=normalized(cross(sub(atoms[c].xyz,atoms[b].xyz),sub(atoms[d].xyz,atoms[c].xyz)));
 let agreement=match(axis,u,v){(Some(axis),Some(u),Some(v))=>dot(u,v)*cos(expected*DEG)+dot(cross(u,v),axis)*sin(expected*DEG),_=>-1.0};
 if agreement<cos(1.0*DEG){e.issue("torsion",format!("{}:{}-{}-{}-{} disagrees with retained {:.3} deg",atoms[c].residue+1,atoms[a].name,atoms[b].name,atoms[c].name,atoms[d].name,expected));}
}
pub(crate) fn radius(name:&str)->f64 {match name.as_bytes().first(){Some(b'N')=>1.55,Some(b'O')=>1.52,Some(b'S')=>1.80,_=>1.70}}
/// C-alpha projection of the existing full molecular nonbonded check.
/// The exclusions follow molecular connectivity rather than residue spacing.
pub(crate) fn ca_exclusion_relations(chain:&[&str])->Vec<(usize,usize,f64)>{
 let excluded=steric_exclusions(chain);let mut ca=Vec::new();let mut offset=0;
 for aa in chain{ca.push(offset+1);offset+=4+crate::fold3d::sidechain_topology(aa).len();}
 let mut result=Vec::new();
 for i in 0..ca.len(){for j in i+1..ca.len(){
  if !excluded[ca[i]].contains(&ca[j]){
   result.push((i,j,2.0*radius("CA")-CLASH_OVERLAP));
  }
 }}result
}
/// Source molecular constraints for propagation, before any contact selection.
pub(crate) fn molecular_constraint_source(chain:&[&str])->serde_json::Value{
 let mut atoms=Vec::new();let mut starts=Vec::new();let mut names=Vec::new();
 for (residue,aa) in chain.iter().enumerate(){
  starts.push(names.len());
  for name in ["N","CA","C","O"].into_iter().chain(crate::fold3d::sidechain_topology(aa).iter().map(|x|x.0.trim())){
   atoms.push(serde_json::json!({"residue":residue,"name":name}));names.push(name);
  }
 }
 let mut edges=Vec::new();
 for (i,&aa) in chain.iter().enumerate(){
  let start=starts[i];let end=starts.get(i+1).copied().unwrap_or(names.len());
  let find=|name:&str|start+names[start..end].iter().position(|&n|n==name).expect("source atom inventory");
  for (a,b,length) in [("N","CA",1.458),("CA","C",1.525),("C","O",1.231)]{
   edges.push(serde_json::json!([find(a),find(b),(length*1000.0+0.5) as u64]));
  }
  if i>0{edges.push(serde_json::json!([starts[i-1]+2,find("N"),1329]));}
  for &(name,parent,_) in crate::fold3d::sidechain_topology(aa){
   let length=source_sidechain_length(aa,name.trim(),parent);
   edges.push(serde_json::json!([find(name.trim()),find(parent),(length*1000.0+0.5) as u64]));
  }
  for &(a,b,length) in ring_edges(aa){edges.push(serde_json::json!([find(a),find(b),(length*1000.0+0.5) as u64]));}
 }
 serde_json::json!({"atoms":atoms,"bonds":edges,"distance_unit":"1/1000 angstrom",
  "source":"native peptide molecular inventory and bond constraints"})
}
/// Retain the unfused terminal carboxyl port of an open peptide chain.
/// The metric is the existing native carboxyl template, not a reference fit.
pub(crate) fn complete_open_carboxyl(source:&serde_json::Value)->Result<serde_json::Value,String>{
 if source["terminal_carboxyl_boundary"].is_object(){return Ok(source.clone());}
 let residues=source["residue_identities"].as_array().ok_or("open chain has no residue source")?;
 if residues.is_empty(){return Err("open carboxyl boundary requires a nonempty chain".into());}
 let last=residues.len()-1;let mut out=source.clone();
 let atoms=out["atoms"].as_array_mut().ok_or("open chain has no atom source")?;
 let c=atoms.iter().position(|a|a["residue"].as_u64()==Some(last as u64)&&a["name"]=="C").ok_or("terminal carbonyl carbon absent")?;
 let index=atoms.len();atoms.push(serde_json::json!({"residue":last,"name":"OXT","source_role":"unfused terminal carboxyl oxygen"}));
 out["bonds"].as_array_mut().ok_or("open chain has no covalent source")?.push(serde_json::json!([c,index,(crate::peptide_geometry::terminal_carboxyl_length()*1000.0+0.5) as u64]));
 out["terminal_carboxyl_boundary"]=serde_json::json!({"source_role":"unfused terminal carboxyl port","residue":last,"source_atom":index,"peptide_joins":last,"free_monomer_carboxyl_oxygen_count":2*residues.len(),"required_backbone_carboxyl_oxygen_count":residues.len()+1,
  "topology_justification":"two carboxyl oxygens per free source monomer; each of n-1 peptide joins consumes one; the terminal port remains",
  "source_component":"serine_as_peptide_component VINIT free amino/carboxyl groups, TANCH termini and CLINK peptide joining",
  "metric_reference":"native peptide_geometry::bond_length(Asp,OD2,CG); declared model parameter, not a fitted coordinate",
  "position_rule":"axis-preserving half-turn of the retained terminal carbonyl direction about its C-to-CA axis, scaled to the existing carboxyl length"});
 Ok(out)
}
fn aromatic(residue:&str)->bool {matches!(residue,"Phe"|"PHE"|"Tyr"|"TYR"|"His"|"HIS"|"Trp"|"TRP")}
fn source_sidechain_length(residue:&str,name:&str,parent:&str)->f64{
 if name=="CB"{1.53}else if aromatic(residue)&&parent!="CB"{
  if name=="OH"{1.36}else{1.39}
 }else if matches!(residue,"Pro"|"PRO"){1.50}
 else{crate::peptide_geometry::bond_length(residue,name,parent)}
}
fn ring_edges(residue:&str)->&'static [(&'static str,&'static str,f64)] {
 match residue {
  "Pro"|"PRO"=>&[("CD","N",1.47)],
  "Phe"|"PHE"|"Tyr"|"TYR"=>&[("CE2","CZ",1.39)],
  "His"|"HIS"=>&[("CE1","NE2",1.39)],
  "Trp"|"TRP"=>&[("NE1","CE2",1.39),("CZ3","CH2",1.39)],
  _=>&[],
 }
}
/// Atom order and connectivity shared by the torsion producer and observation.
pub(crate) fn steric_exclusions(chain:&[&str])->Vec<alloc::collections::BTreeSet<usize>> {
 steric_exclusions_with_terminal(chain,false)
}
pub(crate) fn steric_exclusions_with_terminal(chain:&[&str],terminal:bool)->Vec<alloc::collections::BTreeSet<usize>> {
 let mut names=Vec::new();let mut starts=Vec::new();
 for aa in chain {
  starts.push(names.len());names.extend(["N","CA","C","O"]);
  names.extend(crate::fold3d::sidechain_topology(aa).iter().map(|a|a.0.trim()));
 }
 if terminal&&!chain.is_empty(){names.push("OXT");}
 let mut adjacent=alloc::vec![Vec::new();names.len()];
 let mut connect=|a:usize,b:usize|{adjacent[a].push(b);adjacent[b].push(a);};
 for (i,aa) in chain.iter().enumerate() {
  let start=starts[i];let end=starts.get(i+1).copied().unwrap_or(names.len());
  let find=|name:&str|start+names[start..end].iter().position(|n|*n==name).unwrap();
  for (a,b) in [("N","CA"),("CA","C"),("C","O")] {connect(find(a),find(b));}
  if i>0 {connect(starts[i-1]+2,start);}
  for &(name,parent,_) in crate::fold3d::sidechain_topology(aa) {connect(find(name.trim()),find(parent));}
  for &(a,b,_) in ring_edges(aa) {connect(find(a),find(b));}
 }
 if terminal&&!chain.is_empty(){connect(starts[chain.len()-1]+2,names.len()-1);}
 let mut excluded=adjacent.iter().enumerate().map(|(i,neighbors)|{
  let mut excluded=alloc::collections::BTreeSet::new();excluded.insert(i);
  for &j in neighbors {excluded.insert(j);for &k in &adjacent[j] {excluded.insert(k);}}
  excluded
 }).collect::<Vec<_>>();
 // O(i)-C(i)-N(i+1)-CA(i+1) is one rigid trans-amide frame.
 // Its endpoint distance is already fixed by the checked covalent angles
 // and omega. It has no independent nonbonded separation degree of freedom.
 for i in 0..chain.len().saturating_sub(1){let frame=[starts[i]+1,starts[i]+2,starts[i]+3,starts[i+1],starts[i+1]+1];for &a in &frame{for &b in &frame{excluded[a].insert(b);}}}
 for(i,aa)in chain.iter().enumerate(){if !aromatic(aa){continue;}
  let mut core=(starts[i]..starts.get(i+1).copied().unwrap_or(names.len())).collect::<Vec<_>>();
  loop{let before=core.clone();core.retain(|a|adjacent[*a].iter().filter(|b|before.contains(b)).count()>=2);if core==before{break;}}
  for &a in &core{for &b in &core{excluded[a].insert(b);}}
 }
 excluded
}
/// Assess exactly the coordinates that are printed and exported. Keep every conflict.
pub fn assess(chain:&[&str],backbone:&[[Point;4]],sidechains:&[Vec<(&str,Point)>],steps:&[(f64,f64)])->Evidence {
 let mut e=Evidence::default();let mut atoms=Vec::new();
 let terminal=sidechains.last().is_some_and(|s|s.iter().any(|a|a.0.trim()=="OXT"));
 if chain.is_empty()||chain.len()!=backbone.len()||chain.len()!=sidechains.len()||chain.len()!=steps.len(){
  e.issue("inventory",format!("chain/backbone/sidechain/step lengths {}/{}/{}/{}",chain.len(),backbone.len(),sidechains.len(),steps.len()));return e;
 }
 for (i,bb) in backbone.iter().enumerate(){
  if !["Ala","Arg","Asn","Asp","Cys","Gln","Glu","Gly","His","Ile","Leu","Lys","Met","Phe","Pro","Ser","Thr","Trp","Tyr","Val"].iter().any(|aa|aa.eq_ignore_ascii_case(chain[i])) {
   e.issue("inventory",format!("{}: unknown residue {}",i+1,chain[i]));
  }
  if !steps[i].0.is_finite() || !steps[i].1.is_finite() {e.issue("finite",format!("{}: nonfinite retained torsion",i+1));}
  for (name,xyz) in ["N","CA","C","O"].iter().zip(bb){atoms.push(Atom{residue:i,name,xyz:*xyz});}
  let expected=crate::fold3d::sidechain_topology(chain[i]);
  for &(name,xyz) in &sidechains[i] {atoms.push(Atom{residue:i,name:name.trim(),xyz});}
  for &(name,_,_) in expected {
   if sidechains[i].iter().filter(|a|a.0.trim()==name.trim()).count()!=1 {e.issue("inventory",format!("{}:{} missing or duplicated",i+1,name.trim()));}
  }
  for &(name,_) in &sidechains[i] {
   if !expected.iter().any(|x|x.0.trim()==name.trim())&&!(terminal&&i+1==chain.len()&&name.trim()=="OXT"){e.issue("inventory",format!("{}:{} unexpected atom",i+1,name));}
  }
 }
 if terminal&&sidechains.last().unwrap().iter().filter(|a|a.0.trim()=="OXT").count()!=1{e.issue("inventory","terminal OXT is duplicated".into());}
 e.atoms=atoms.len();
 for a in &atoms {if !a.xyz.0.is_finite()||!a.xyz.1.is_finite()||!a.xyz.2.is_finite(){e.issue("finite",format!("{}:{} nonfinite coordinate",a.residue+1,a.name));}}
 // Invalid inventories cannot participate in a complete molecular closure.
 if !e.issues.is_empty(){return e;}
 let find=|residue:usize,name:&str|atoms.iter().position(|a|a.residue==residue && a.name==name).expect("validated atom inventory");
 let mut edges:Vec<(usize,usize,f64,bool)>=Vec::new();
 for (i,&residue) in chain.iter().enumerate(){
  for (a,b,length) in [("N","CA",1.458),("CA","C",1.525),("C","O",1.231)]{edges.push((find(i,a),find(i,b),length,false));}
  if i>0 {edges.push((find(i-1,"C"),find(i,"N"),1.329,false));}
  for &(name,parent,_) in crate::fold3d::sidechain_topology(residue){
   let name=name.trim();
   let length=source_sidechain_length(residue,name,parent);
   edges.push((find(i,parent),find(i,name),length,false));
  }
  for &(a,b,length) in ring_edges(residue){edges.push((find(i,a),find(i,b),length,true));}
  if terminal&&i+1==chain.len(){edges.push((find(i,"C"),find(i,"OXT"),crate::peptide_geometry::terminal_carboxyl_length(),false));check_angle(&mut e,&atoms,[find(i,"CA"),find(i,"C"),find(i,"OXT")],120.8);}
  check_angle(&mut e,&atoms,[find(i,"N"),find(i,"CA"),find(i,"C")],111.0);
  check_angle(&mut e,&atoms,[find(i,"CA"),find(i,"C"),find(i,"O")],120.8);
  if i>0 {
   check_angle(&mut e,&atoms,[find(i-1,"CA"),find(i-1,"C"),find(i,"N")],116.2);
   check_angle(&mut e,&atoms,[find(i-1,"C"),find(i,"N"),find(i,"CA")],121.7);
   check_torsion(&mut e,&atoms,[find(i-1,"C"),find(i,"N"),find(i,"CA"),find(i,"C")],steps[i].0);
   check_torsion(&mut e,&atoms,[find(i-1,"N"),find(i-1,"CA"),find(i-1,"C"),find(i,"N")],steps[i-1].1);
   check_torsion(&mut e,&atoms,[find(i-1,"CA"),find(i-1,"C"),find(i,"N"),find(i,"CA")],180.0);
  }
  // O stays in the peptide plane, with its retained psi-dependent orientation.
  check_torsion(&mut e,&atoms,[find(i,"N"),find(i,"CA"),find(i,"C"),find(i,"O")],steps[i].1+180.0);
  if !matches!(residue,"Gly"|"GLY") {
   e.stereocenters+=1;let ca=atoms[find(i,"CA")].xyz;
   let volume=dot(sub(atoms[find(i,"N")].xyz,ca),cross(sub(atoms[find(i,"C")].xyz,ca),sub(atoms[find(i,"CB")].xyz,ca)));
   if volume<=0.1 {e.issue("stereochemistry",format!("{}:CA L-alpha signed volume {:.6} A^3 (expected positive)",i+1,volume));}
  }
  let side_center=match residue {"Ile"|"ILE"=>Some((["CA","CG1","CG2"],1.0)),"Thr"|"THR"=>Some((["OG1","CA","CG2"],-1.0)),_=>None};
  if let Some((names,sign))=side_center{
   e.stereocenters+=1;let cb=atoms[find(i,"CB")].xyz;
   let v=names.map(|name|sub(atoms[find(i,name)].xyz,cb));let volume=dot(v[0],cross(v[1],v[2]));
   if volume*sign<=0.1 {e.issue("stereochemistry",format!("{}:CB natural sidechain signed volume {:.6} A^3",i+1,volume));}
  }
  if aromatic(residue){
   let names:Vec<_>=crate::fold3d::sidechain_topology(residue).iter().filter(|x|x.0.trim()!="CB"&&x.0.trim()!="OH").map(|x|x.0.trim()).collect();
   let origin=atoms[find(i,names[0])].xyz;
   let normal=normalized(cross(sub(atoms[find(i,names[1])].xyz,origin),sub(atoms[find(i,names[2])].xyz,origin)));
   for name in names {let residual=normal.map(|n|dot(sub(atoms[find(i,name)].xyz,origin),n).abs()).unwrap_or(f64::INFINITY);
    if residual>0.01 {e.issue("planarity",format!("{}:{} aromatic plane residual {:.6} A",i+1,name,residual));}}
  }
 }
 let mut neighbors=alloc::vec![Vec::new();atoms.len()];
 e.bond_edges=edges.iter().map(|&(a,b,_,_)|(a,b)).collect();
 for &(a,b,length,ring) in &edges {
  neighbors[a].push(b);neighbors[b].push(a);e.bonds+=1;if ring{e.rings+=1;}
  let measured=distance(atoms[a].xyz,atoms[b].xyz);let residual=(measured-length).abs();e.max_bond_error=e.max_bond_error.max(residual);
  if residual>BOND_TOLERANCE {e.issue(if ring{"ring"}else{"bond"},format!("{}:{}--{}:{} {:.6} A, expected {:.3}",atoms[a].residue+1,atoms[a].name,atoms[b].residue+1,atoms[b].name,measured,length));}
 }
 // Check every angle, including branches and closing edges, for collapse.
 for (b,adjacent) in neighbors.iter().enumerate(){for (j,&a) in adjacent.iter().enumerate(){for &c in &adjacent[j+1..]{
  e.angles+=1;
  let measured=angle(atoms[a].xyz,atoms[b].xyz,atoms[c].xyz).unwrap_or(0.0);
  if !(90.0..=135.0).contains(&measured){e.issue("angle",format!("{}:{}-{}-{} collapsed angle {:.3} deg",atoms[b].residue+1,atoms[a].name,atoms[b].name,atoms[c].name,measured));}
 }}}
 // The same covalent-frame ownership used by the producing coordinate model.
 let excluded=steric_exclusions_with_terminal(chain,terminal);let mut canonical=alloc::vec![0;atoms.len()];let mut index=0;
 for(i,aa)in chain.iter().enumerate(){for name in ["N","CA","C","O"].into_iter().chain(crate::fold3d::sidechain_topology(aa).iter().map(|a|a.0.trim())){canonical[find(i,name)]=index;index+=1;}}
 if terminal{canonical[find(chain.len()-1,"OXT")]=index;}
 for a in 0..atoms.len(){for b in a+1..atoms.len(){
  if neighbors[a].contains(&b)||neighbors[a].iter().any(|&mid|neighbors[mid].contains(&b)){continue;}
  let(measured,overlap)=nonbonded_measure(atoms[a].xyz,atoms[b].xyz,radius(atoms[a].name)+radius(atoms[b].name));
  if excluded[canonical[a]].contains(&canonical[b]){
   if overlap>CLASH_OVERLAP{e.retained_rigid_contacts.push(format!("B: {}:{}--{}:{} fixed covalent-frame distance {:.6} A; generic radius overlap {:.6} A",atoms[a].residue+1,atoms[a].name,atoms[b].residue+1,atoms[b].name,measured,overlap));}
   continue;
  }
  let stored=(overlap-CLASH_OVERLAP).max(0.0);e.overlap_potential+=10.0*stored*stored;
  e.nonbonded_pairs+=1;e.max_overlap=e.max_overlap.max(overlap);
  if overlap>CLASH_OVERLAP {e.issue("clash",format!("{}:{}--{}:{} distance {:.6} A, overlap {:.6} A",atoms[a].residue+1,atoms[a].name,atoms[b].residue+1,atoms[b].name,measured,overlap));}
 }}
 e
}

/// Four-valued evidence about one proposition. B retains affirmation and denial.
/// B is a valid state; it is never a geometric error or a reason to discard a residue.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum EvidenceState {N,T,F,B}
impl EvidenceState {
 pub fn mark(self)->char {match self{Self::N=>'N',Self::T=>'T',Self::F=>'F',Self::B=>'B'}}
 pub fn observations(values:[Option<bool>;2])->Self {
  let positive=values.contains(&Some(true));let negative=values.contains(&Some(false));
  match(positive,negative){(false,false)=>Self::N,(true,false)=>Self::T,(false,true)=>Self::F,(true,true)=>Self::B}
 }
}
#[derive(Clone,Copy,Debug)]
pub struct SecondaryEvidence<'a> {
 pub residue:usize,pub sequence:Option<&'a str>,pub backbone:Option<&'a str>,
 pub helix:EvidenceState,pub sheet:EvidenceState,pub other:EvidenceState,
}
/// Keep both observations on each same secondary-structure proposition.
/// Geometry constraints are assessed separately and cannot reject a B state.
pub fn secondary_evidence<'a>(sequence:&[&'a str],backbone:&[&'a str])->Vec<SecondaryEvidence<'a>> {
 let classify=|label:&str|match label{"helix"|"helix_l"=>0,"sheet"=>1,_=>2};
 (0..sequence.len().max(backbone.len())).map(|i|{
  let sequence=sequence.get(i).copied();let backbone=backbone.get(i).copied();
  let state=|kind|EvidenceState::observations([sequence.map(|x|classify(x)==kind),backbone.map(|x|classify(x)==kind)]);
  SecondaryEvidence{residue:i+1,sequence,backbone,helix:state(0),sheet:state(1),other:state(2)}
 }).collect()
}
pub fn secondary_report(evidence:&[SecondaryEvidence<'_>],details:bool)->String {
 let mut out=String::from("Secondary-structure evidence: B is valid (affirmation and denial retained)\n");
 for (name,kind) in [("helix",0),("sheet",1),("other",2)] {
  let states:Vec<_>=evidence.iter().map(|e|match kind{0=>e.helix,1=>e.sheet,_=>e.other}).collect();
  out.push_str(&format!("  {}: T {}  F {}  B {}  N {}\n",name,states.iter().filter(|&&s|s==EvidenceState::T).count(),states.iter().filter(|&&s|s==EvidenceState::F).count(),states.iter().filter(|&&s|s==EvidenceState::B).count(),states.iter().filter(|&&s|s==EvidenceState::N).count()));
 }
 if details {for e in evidence{out.push_str(&format!("  residue {}: sequence {:?}, B4 backbone {:?}; helix {}, sheet {}, other {}\n",e.residue,e.sequence,e.backbone,e.helix.mark(),e.sheet.mark(),e.other.mark()));}}
 out
}

#[cfg(test)]
mod native_distance_boundary_tests{
 use super::*;
 #[test]
 fn native_distance_boundary_is_shared_without_relaxation(){
  let d=libm::sqrt(4.5);let a=(0.0,0.0,0.0);let b=(d,d,0.0);
  let(measured,overlap)=nonbonded_measure(a,b,3.4);
  let alternate=3.4-libm::sqrt(dot(b,b));
  assert_eq!(measured.to_bits(),3.0f64.to_bits());
  assert!(overlap<=CLASH_OVERLAP);
  assert!(alternate>CLASH_OVERLAP);
 }
}
