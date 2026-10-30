#![deny(warnings)]
extern crate alloc;
use ::vox::morphism_factor::{decimal_to_tape, dec_of, parse_numeral};
fn tape(s:&str)->Vec<char>{ if s.starts_with('⊢'){parse_numeral(s).unwrap()} else {decimal_to_tape(s).unwrap()} }
fn main(){
    #[cfg(feature = "hosted")]
    ::vox::ordering::cli::intercept();
  let args: Vec<String> = std::env::args().collect();
  if args.len()<3 { eprintln!("usage: braid_gens <a> <N>"); return; }
  let a=tape(&args[1]); let n=tape(&args[2]);
  match ::vox::shor_braid::shor_braid_carrier(&a,&n){
    Ok(braid) => {
      let word=&braid.word; let levels=braid.levels;
      println!("level_order={} inverse_order={} return_verified=true",braid.level_order.label(),braid.level_order.inverse().label());
      println!("physical_word={} level_order_spec={}",word.iter().collect::<String>(),braid.level_order.spec());
      let mut gens: Vec<i32> = Vec::new();
      for &c in word { if c=='≻' {gens.push(1);} else if c=='≺' {gens.push(-1);} }
      println!("braid word: levels={} len={} gens={:?}", levels, word.len(), gens);
      match braid.read_order(){
        Ok(r)=>println!("winding r = {}", dec_of(&r)),
        Err(e)=>println!("winding refused: {}", e),
      }
    }
    Err(e)=>println!("no braid word: {}", e),
  }
}
