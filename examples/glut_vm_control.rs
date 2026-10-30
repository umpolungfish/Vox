//! Locate native/lifted differences in glut frame and certificate execution.
#![deny(warnings)]
use vox::glut_system::{GlutSieve, verify_glut_reentry_certificate};
use vox::morphism_factor::{tape_u64, dec_of};
use vox::factor_extract::FactorCarrier;
use vox::reentry_certificate::{certify_reentry, encode_reentry_certificate, decode_reentry_certificate};
fn main() {
    let n = tape_u64(8051);
    println!("copy equality {}", n == n.clone());
    let mut sieve = GlutSieve::new(&n);
    println!("seed {} {}", sieve.states.len(), sieve.states[0].can_close(&n));
    for width in [2, 3, 4, 5] {
        sieve.frame_superpose(width);
        println!("frame {width} states {}", sieve.states.len());
        if let Some((p, q)) = sieve.readout() {
            println!("readout {} {}", dec_of(&p), dec_of(&q));
            let execution = sieve.readout_execution().expect("frame ancestry");
            let trace = execution.trace(&n).unwrap();
            println!("trace {} copy {}", trace.len(), trace == trace.clone());
            let carrier = FactorCarrier::new(n, p, q, trace).unwrap();
            let cert = certify_reentry(&carrier).unwrap();
            println!("source link equality {}", cert.links[0].before == cert.links[0].after);
            let decoded = decode_reentry_certificate(&encode_reentry_certificate(&cert)).unwrap();
            println!("transport link equality {}", decoded.links[0].before == decoded.links[0].after);
            println!("certificate {:?}", verify_glut_reentry_certificate(&decoded).map(|s| s.transforms));
            return;
        }
    }
}
