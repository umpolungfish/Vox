use std::time::Instant;
use vox::dialectic_reentry::{Descent, DialecticObject};
use vox::morphism_factor::{decimal_to_tape, dec_of};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let n = decimal_to_tape(&args[2]).expect("decimal N");
    let start = Instant::now();
    if args[1] == "resident" {
        let mut resident = vox::factorization_31_membrane::UnboundedResident::new(n);
        resident.run();
        assert!(resident.boundary_ok && resident.sidearm_round_trip);
        println!("factor {}", dec_of(&resident.remainder));
        for p in resident.factors { println!("factor {}", dec_of(&p)); }
    } else {
        let mut object = DialecticObject::new(n).unwrap();
        let mut steps = 0;
        loop {
            steps += 1;
            match object.descend() {
                Descent::B(next) => object = next,
                Descent::T(closed) => {
                    println!("factor {}", dec_of(&closed.carrier.p));
                    println!("factor {}", dec_of(&closed.carrier.q));
                    println!("descents {steps}");
                    break;
                }
                Descent::N(_) => panic!("N before closure"),
                Descent::F => panic!("F before closure"),
            }
        }
    }
    println!("seconds {}", start.elapsed().as_secs_f64());
}
