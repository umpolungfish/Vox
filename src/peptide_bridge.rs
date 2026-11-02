//! Exact transport composition on source-addressed residue signal channels.
//! Physical native-contact stability remains a separate unresolved domain action.
use alloc::{string::String,vec::Vec};
type Matrix=Vec<Vec<i64>>;
fn identity(n:usize)->Matrix{(0..n).map(|i|(0..n).map(|j|i64::from(i==j)).collect()).collect()}
fn transpose(a:&Matrix)->Matrix{(0..a.len()).map(|i|(0..a.len()).map(|j|a[j][i]).collect()).collect()}
fn compose(a:&Matrix,b:&Matrix)->Matrix{
 let n=a.len();(0..n).map(|i|(0..n).map(|j|(0..n).map(|k|a[i][k]*b[k][j]).sum()).collect()).collect()
}
fn transport(n:usize,i:usize,j:usize)->Matrix{
 let mut matrix=identity(n);matrix[i][i]=0;matrix[j][j]=0;matrix[i][j]=1;matrix[j][i]=-1;matrix
}

pub fn prepare(peptide:&serde_json::Value,domain:&serde_json::Value)->Result<serde_json::Value,String>{
 let residues=peptide["residues"].as_array().ok_or("bridge lacks source residues")?;
 let n=residues.len();let unit=identity(n);
 let atoms=domain["molecular_source"]["atoms"].as_array().ok_or("bridge lacks molecular addresses")?;
 let bonds=domain["molecular_source"]["bonds"].as_array().ok_or("bridge lacks source bonds")?;
 let mut source_links=Vec::new();let mut operators=Vec::new();
 for bond in bonds{
  let a=bond[0].as_u64().ok_or("invalid bond source")? as usize;
  let b=bond[1].as_u64().ok_or("invalid bond target")? as usize;
  let i=atoms[a]["residue"].as_u64().ok_or("invalid residue address")? as usize;
  let j=atoms[b]["residue"].as_u64().ok_or("invalid residue address")? as usize;
  if i==j{continue;}
  let matrix=transport(n,i,j);
  if compose(&transpose(&matrix),&matrix)!=unit{return Err("source transport loses signal fidelity".into());}
  source_links.push(serde_json::json!({"atom_source":[a,b],"residue_source":[i,j],"operator":matrix}));
  operators.push((i,j,matrix));
 }
 let mut compositions=Vec::new();
 for (left,a) in operators.iter().enumerate(){for (right,b) in operators.iter().enumerate(){
  if left>=right||!(a.0==b.0||a.0==b.1||a.1==b.0||a.1==b.1){continue;}
  let ab=compose(&a.2,&b.2);let ba=compose(&b.2,&a.2);
  let mut commutator=Vec::new();
  for i in 0..n{for j in 0..n{let value=ab[i][j]-ba[i][j];if value!=0{commutator.push([i as i64,j as i64,value]);}}}
  if compose(&transpose(&ab),&ab)!=unit{return Err("source bridge composition loses its input".into());}
  compositions.push(serde_json::json!({"source_links":[left,right],"ordered_composition":ab,
   "commutator_nonzero_entries":commutator,"adjoint_return_equals_input":true}));
 }}
 Ok(serde_json::json!({"representation":"exact orthogonal transport on one source-addressed residue signal basis",
  "source_links":source_links,"overlapping_compositions":compositions,
  "arithmetic":"signed integer matrix composition; all input basis channels retained",
  "physical_contact_positions_bound":false,"native_contact_stability_proven":false,
  "position_momentum_commutation_relation_bound":false,
  "scope":"source contact-signal composition mechanism prepared for the supplied operator"}))
}
