//! Probe the current glut's readout, frame transport, and carry verification.
use vox::glut_system::GlutSieve;
use vox::morphism_factor::{dec_of, mul, tape_u64};

fn main() {
    for n in [35, 56, 8051] {
        let tape = tape_u64(n);
        let mut sieve = GlutSieve::new(&tape);
        let mut counts = vec![(sieve.states[0].position, sieve.states.len())];
        while sieve.states.iter().any(|s| !s.is_complete(tape.len())) {
            sieve.frame_superpose(1);
            counts.push((sieve.states[0].position, sieve.states.len()));
        }
        let pairs = sieve.glut_crystal();
        let rendered: Vec<_> = pairs.iter().map(|(p, q)| {
            format!("{}*{}", dec_of(p), dec_of(q))
        }).collect();
        let (p, q) = sieve.readout().unwrap();
        assert_eq!(mul(&p, &q), tape);
        println!("N={n} states={counts:?} crystal={} readout={}*{} toroidal_accepts={}",
            rendered.join(","), dec_of(&p), dec_of(&q), p != tape_u64(1));
        for width in 2..=8 {
            let mut regrouped = GlutSieve::new(&tape);
            while regrouped.states.iter().any(|s| !s.is_complete(tape.len())) {
                regrouped.frame_superpose(width);
            }
            assert_eq!(regrouped.states, sieve.states);
            assert_eq!(regrouped.glut_crystal(), pairs);
        }
        let mut corrupted = sieve.states[0].clone();
        corrupted.carry = vox::morphism_factor::add(&corrupted.carry, &tape_u64(1));
        println!("N={n} widths_2_to_8_same_states=true corrupted_carry_passes_prefix_check={}",
            corrupted.verify_product_bit(&tape));
    }
}
