#![allow(dead_code)]
extern crate alloc;
#[path="../src/fold3d.rs"] mod fold3d;
#[path="../src/peptide_geometry.rs"] mod peptide_geometry;
#[path="../src/peptide_closure.rs"] mod peptide_closure;
#[path="../src/genetic.rs"] mod genetic;
#[path="../src/genetic_table.rs"] mod genetic_table;
#[path="../src/protein.rs"] mod protein;
fn main(){
 let input="ATGGCCTGTGGCAAGCCTGTGGCAAGCCTGTGGCAAGCCTGTGGCAAGCCTGTGGCAAGCCTGTGGCAAGTA";
 let t=protein::translate_full(input,"");
 let b4:Vec<_>=(0..t.protein.len()).map(|k|genetic::nuc_b4(t.mrna.as_bytes()[t.start+k*3] as char).unwrap()).collect();
 let steps=fold3d::rama_steps(&b4);let bb=fold3d::build_backbone(&steps);
 let sidechains:Vec<Vec<(&str,_)>>=t.protein.iter().zip(&bb).zip(&steps).map(|((aa,b),s)|fold3d::build_sidechain(aa,b,s)).collect();
 let coords:Vec<_>=bb.iter().map(|b|[b.n,b.ca,b.c,b.o]).collect();
 let retained:Vec<_>=steps.iter().map(|s|(s.phi,s.psi)).collect();
 let e=peptide_closure::assess(&t.protein,&coords,&sidechains,&retained);
 print!("{}",e.report(true));
 std::fs::write("measurements/geometry_closure_probe.pdb",fold3d::write_pdb_with_steps(&t.protein,&bb,&steps,&[],true,0,"GEOMETRY CLOSURE",'A')).unwrap();
}
