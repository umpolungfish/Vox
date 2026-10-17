use vox::shor_membrane::{ShorCarrier, shor_factor};
use vox::morphism_factor::tape_u64;

fn main() {
    let n = tape_u64(91);
    let mut carrier = ShorCarrier::new(n.clone()).unwrap();
    match carrier.run(Some(tape_u64(2))) {
        Ok((p, q)) => println!("SUCCESS: {} = {} x {}", vox::morphism_factor::dec_of(&n), vox::morphism_factor::dec_of(&p), vox::morphism_factor::dec_of(&q)),
        Err(e) => println!("ERROR: {}", e),
    }
}
