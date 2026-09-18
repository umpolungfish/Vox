extern crate alloc;
use ::vox::morphism_factor::{decimal_to_tape, parse_numeral, modulo, mul, one, cmp};
fn tape(s:&str)->Vec<char>{ if s.starts_with('⊢'){parse_numeral(s).unwrap()} else {decimal_to_tape(s).unwrap()} }
fn main(){
  let args: Vec<String> = std::env::args().collect();
  if args.len()<3 { eprintln!("usage: orbit_uncapped <a> <N>"); return; }
  let a=tape(&args[1]); let n=tape(&args[2]);
  let mut v=one();
  let mut k: u128 = 0;
  let t0=std::time::Instant::now();
  loop {
    v = modulo(&mul(&v,&a), &n);
    k += 1;
    if cmp(&v,&one())==core::cmp::Ordering::Equal { println!("ORBIT CLOSED: r = {k}  [{:?}]", t0.elapsed()); return; }
    if k % 2_000_000 == 0 { println!("  uncapped orbit step {k}  [{:?}]  (torus modulus Z/N, no cap)", t0.elapsed()); }
  }
}
