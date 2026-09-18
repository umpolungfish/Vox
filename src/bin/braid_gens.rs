extern crate alloc;
use ::vox::morphism_factor::{decimal_to_tape, dec_of, parse_numeral};
fn tape(s:&str)->Vec<char>{ if s.starts_with('⊢'){parse_numeral(s).unwrap()} else {decimal_to_tape(s).unwrap()} }
fn main(){
  let args: Vec<String> = std::env::args().collect();
  if args.len()<3 { eprintln!("usage: braid_gens <a> <N>"); return; }
  let a=tape(&args[1]); let n=tape(&args[2]);
  match ::vox::shor_braid::shor_braid(&a,&n){
    Ok((word, levels)) => {
      let mut gens: Vec<i32> = Vec::new();
      for &c in &word { if c=='≻' {gens.push(1);} else if c=='≺' {gens.push(-1);} }
      println!("braid word: levels={} len={} gens={:?}", levels, word.len(), gens);
      match ::vox::winding_readout::winding_number_tape(&word){
        Ok(r)=>println!("winding r = {}", dec_of(&r)),
        Err(e)=>println!("winding refused: {}", e),
      }
    }
    Err(e)=>println!("no braid word: {}", e),
  }
}
