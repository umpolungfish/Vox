extern crate alloc;
#[path = "../membrane_complex.rs"] mod membrane_complex;
#[path = "../fde_shor_membrane.rs"] mod fde_shor_membrane;
#[path = "../shor_qft.rs"] mod shor_qft;
use std::time::Instant;
use vox::dialectic_reentry::{Descent, DialecticObject};
use vox::morphism_factor::{decimal_to_tape, dec_of};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let n = decimal_to_tape(&args[2]).expect("decimal N");
    let start = Instant::now();
    if args[1] == "braid" {
        eprintln!("stage braid order acquisition, base=2");
        let (word, levels) = vox::shor_braid::shor_braid(&vox::morphism_factor::tape_u64(2), &n).unwrap();
        eprintln!("stage winding readout; levels={levels}");
        let r = vox::winding_readout::winding_number_tape(&word).unwrap();
        println!("order {}", dec_of(&r));
        let (p,q) = vox::shor_braid::factor_close_public(&vox::morphism_factor::tape_u64(2), &n, &r).unwrap();
        println!("factor {}\nfactor {}", dec_of(&p), dec_of(&q));
    } else if args[1] == "observed" {
        let qubits: usize = args.get(3).expect("observed requires register qubits").parse().unwrap();
        let reg = shor_qft::ObservedPhaseRegister::from_modulus(vox::morphism_factor::tape_u64(2), n.clone(), qubits).unwrap();
        println!("branch_population {}", reg.branch_population);
        let Some(order) = reg.extract_order() else {
            println!("unresolved phase register");
            return;
        };
        println!("order {}", dec_of(&order));
        let (p,q) = shor_qft::factor_close_public(&reg.a, &n, &order).unwrap();
        println!("factor {}\nfactor {}", dec_of(&p), dec_of(&q));
    } else if args[1] == "symbolic" {
        eprintln!("stage modular evolution and observed QFT, base=2");
        let a = vox::morphism_factor::tape_u64(2);
        let order = shor_qft::observe_order(a.clone(), n.clone()).unwrap();
        println!("order {}", dec_of(&order));
        let (p,q) = shor_qft::factor_close_public(&a, &n, &order).unwrap();
        println!("factor {}\nfactor {}", dec_of(&p), dec_of(&q));
    } else if args[1] == "phase" {
        eprintln!("stage nested phase factor execution");
        let word = vox::morphism_factor::emit_numeral(&n);
        let result = vox::morphism_factor::factor(&word).expect("phase factor");
        let p = vox::morphism_factor::parse_numeral(&result).unwrap();
        let (q, rem) = vox::morphism_factor::divmod(&n, &p);
        assert!(vox::morphism_factor::zero(&rem));
        println!("factor {}\nfactor {}", dec_of(&p), dec_of(&q));
    } else if args[1] == "resident" {
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
