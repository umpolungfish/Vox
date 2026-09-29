//! Census concrete prefix storage before closure, with a single baked sparse source.
#![deny(warnings)]
use std::collections::BTreeMap;
use vox::glut_system::GlutSieve;
use vox::morphism_factor::decimal_to_tape;
fn main() {
    let n = decimal_to_tape("115792089237316195423570985008687907853269984665640564039457584007913129639936").unwrap();
    let mut sieve = GlutSieve::new(&n);
    for _ in 1..14 { sieve.frame_superpose(1); }
    let mut carries = BTreeMap::new();
    let mut valuations = BTreeMap::new();
    let mut zero_p = 0;
    let mut zero_q = 0;
    let mut mirrored = 0;
    let mut stored_cells = 0;
    for s in &sieve.states {
        *carries.entry(s.carry.clone()).or_insert(0usize) += 1;
        let vp = s.p_prefix.iter().position(|&c| c == '⊥').unwrap_or(s.position);
        let vq = s.q_prefix.iter().position(|&c| c == '⊥').unwrap_or(s.position);
        *valuations.entry((vp, vq)).or_insert(0usize) += 1;
        zero_p += usize::from(vp == s.position);
        zero_q += usize::from(vq == s.position);
        mirrored += usize::from(s.p_prefix > s.q_prefix);
        stored_cells += s.p_prefix.len() + s.q_prefix.len() + s.carry.len();
    }
    println!("source_bits {} position {} states {} stored_value_cells {}", n.len(), sieve.states[0].position, sieve.states.len(), stored_cells);
    println!("distinct_carries {} distinct_valuation_pairs {} zero_p {} zero_q {} mirrored {}", carries.len(), valuations.len(), zero_p, zero_q, mirrored);
    assert!(sieve.states.iter().all(|s| s.carry == vec!['⊤'] && s.can_close(&n)));
    println!("all_zero_carry_and_reverse_valid true");
}
