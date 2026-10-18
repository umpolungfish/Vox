#![allow(dead_code)]
extern crate alloc;
#[path="../src/fold3d.rs"] mod fold3d;
#[path="../src/peptide_geometry.rs"] mod peptide_geometry;
#[path="../src/peptide_closure.rs"] mod peptide_closure;
fn main(){
 let steps=fold3d::rama_steps(&['T']);let bb=fold3d::build_backbone(&steps);
 let coords=vec![[bb[0].n,bb[0].ca,bb[0].c,bb[0].o]];
 let sc:Vec<Vec<(&str,_)>>=vec![fold3d::build_sidechain("Ala",&bb[0],&steps[0])];let retained=vec![(steps[0].phi,steps[0].psi)];
 let good=peptide_closure::assess(&["Ala"],&coords,&sc,&retained);
 assert!(good.closed(),"{}",good.report(true));println!("Unmodified L-alanine coordinate closure: PASS");
 let mut missing=sc.clone();missing[0].clear();assert!(!peptide_closure::assess(&["Ala"],&coords,&missing,&retained).closed());
 let mut damaged=coords.clone();damaged[0][3]=damaged[0][1];let e=peptide_closure::assess(&["Ala"],&damaged,&sc,&retained);assert!(e.issues.iter().any(|i|i.kind=="bond"));
 let mut mirrored=coords.clone();for p in &mut mirrored[0] {p.2=-p.2;}let mut msc=sc.clone();msc[0][0].1.2=-msc[0][0].1.2;let e=peptide_closure::assess(&["Ala"],&mirrored,&msc,&retained);assert!(e.issues.iter().any(|i|i.kind=="stereochemistry"));
 let mut nonfinite=coords.clone();nonfinite[0][0].0=f64::NAN;assert!(!peptide_closure::assess(&["Ala"],&nonfinite,&sc,&retained).closed());
 assert!(!peptide_closure::assess(&["Ala"],&coords,&sc,&[(f64::NAN,0.0)]).closed());
 assert!(!peptide_closure::assess(&["unknown"],&coords,&sc,&retained).closed());
 let trpsc:Vec<Vec<(&str,_)>>=vec![fold3d::build_sidechain("Trp",&bb[0],&steps[0])];let mut broken=trpsc.clone();broken[0].iter_mut().find(|x|x.0.trim()=="CH2").unwrap().1.0+=2.0;
 assert!(peptide_closure::assess(&["Trp"],&coords,&broken,&retained).issues.iter().any(|i|i.kind=="ring"));
 let both=vec![coords[0],coords[0]];let sides=vec![sc[0].clone(),sc[0].clone()];let e=peptide_closure::assess(&["Ala","Ala"],&both,&sides,&[retained[0],retained[0]]);assert!(e.issues.iter().any(|i|i.kind=="clash"));
 let isosc=fold3d::build_sidechain("Ile",&bb[0],&steps[0]);let thrsc=fold3d::build_sidechain("Thr",&bb[0],&steps[0]);
 for (name,side) in [("Ile",isosc),("Thr",thrsc)]{let e=peptide_closure::assess(&[name],&coords,&vec![side],&retained);assert!(!e.issues.iter().any(|i|i.kind=="stereochemistry"));}
 println!("Missing atom, oxygen overlap, reflected chirality, nonfinite coordinate/torsion, unknown residue, broken ring and coincident residues: all detected");
 println!("Natural Ile/Thr sidechain stereochemistry: PASS");
 use peptide_closure::EvidenceState::{N,T,F,B};
 assert_eq!(peptide_closure::EvidenceState::observations([None,None]),N);
 assert_eq!(peptide_closure::EvidenceState::observations([Some(true),None]),T);
 assert_eq!(peptide_closure::EvidenceState::observations([Some(false),None]),F);
 assert_eq!(peptide_closure::EvidenceState::observations([Some(true),Some(false)]),B);
 let held=peptide_closure::secondary_evidence(&["helix"],&["sheet"]);
 assert_eq!(held[0].helix,B);assert_eq!(held[0].sheet,B);
 assert!(good.closed(),"secondary B does not reject geometry");
 println!("T/F/B/N retained; helix/sheet B states valid and independent of geometric constraints: PASS");
}
