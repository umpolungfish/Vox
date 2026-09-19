use core::cmp::Ordering;
use std::collections::BTreeSet;

use vox::dialectic_reentry::{
    DialecticObject, EXTENDED_FERMAT_WORD, LEHMAN_WORD, SHORT_FRONTIER_WORD,
};
use vox::imscription_cycle::{certify_imscription_cycle, verify_imscription_cycle};
use vox::morphism_factor::{cmp, dec_of, miller_rabin, mul, tape_u64};
use vox::trace_algebra::witness_valid;

fn next_prime(mut value: u64) -> u64 {
    if value <= 2 {
        return 2;
    }
    if value & 1 == 0 {
        value += 1;
    }
    loop {
        if miller_rabin(&tape_u64(value)) {
            return value;
        }
        value = value.checked_add(2).expect("prime scan overflow");
    }
}

fn same_pair(a: &[char], b: &[char], x: &[char], y: &[char]) -> bool {
    (cmp(a, x) == Ordering::Equal && cmp(b, y) == Ordering::Equal)
        || (cmp(a, y) == Ordering::Equal && cmp(b, x) == Ordering::Equal)
}

fn topology(certificate: &vox::imscription_cycle::ImscriptionCycleCertificate) -> (usize, usize, usize) {
    let mut short = 0usize;
    let mut extended = 0usize;
    let mut lehman = 0usize;
    for wire in &certificate.dialectic.objects {
        let object = DialecticObject::decode(wire).unwrap();
        if object.word == SHORT_FRONTIER_WORD.chars().collect::<Vec<_>>() {
            short += 1;
        } else if object.word == EXTENDED_FERMAT_WORD.chars().collect::<Vec<_>>() {
            extended += 1;
        } else if object.word == LEHMAN_WORD.chars().collect::<Vec<_>>() {
            lehman += 1;
        } else {
            panic!("scale certificate contained an unknown imscription word");
        }
    }
    (short, extended, lehman)
}

#[test]
fn controlled_gap_family_changes_fermat_ring_without_changing_cycle_identity() {
    let p_value = next_prime(1_000_000);
    let p = tape_u64(p_value);
    let requested_gaps = [
        1_000u64, 2_000, 4_000, 8_000, 16_000, 32_000,
        48_000, 64_000, 96_000, 128_000, 160_000,
    ];

    let mut supports = BTreeSet::new();
    let mut observed = Vec::new();

    for requested_gap in requested_gaps {
        let q_value = next_prime(p_value + requested_gap);
        let q = tape_u64(q_value);
        let n = mul(&p, &q);
        let start = DialecticObject::new(n).unwrap();
        let certificate = certify_imscription_cycle(&start).unwrap();
        let summary = verify_imscription_cycle(&certificate).unwrap();

        assert_eq!(summary.descents, summary.quotient_transforms);
        assert_eq!(summary.quotient_generations, summary.descents + 1);
        assert!(witness_valid(
            &summary.fixed_carrier.n,
            &summary.fixed_carrier.p,
            &summary.fixed_carrier.q,
        ));
        assert!(same_pair(
            &summary.fixed_carrier.p,
            &summary.fixed_carrier.q,
            &p,
            &q,
        ));
        assert_eq!(summary.fixed_carrier.encode(), certificate.dialectic.terminal_carrier);

        let (short, extended, lehman) = topology(&certificate);
        assert_eq!(short, 1);
        assert_eq!(lehman, 0, "controlled gap family unexpectedly changed lattice");
        match summary.terminal_support {
            39 => {
                assert_eq!(summary.descents, 1);
                assert_eq!(extended, 0);
                assert_eq!(cmp(&certificate.dialectic.lattice_cell, &tape_u64(64)), Ordering::Less);
                assert!(certificate.dialectic.lehman_multiplier.is_none());
            }
            47 => {
                assert_eq!(summary.descents, 2);
                assert_eq!(extended, 1);
                assert_ne!(cmp(&certificate.dialectic.lattice_cell, &tape_u64(64)), Ordering::Less);
                assert_eq!(cmp(&certificate.dialectic.lattice_cell, &tape_u64(4096)), Ordering::Less);
                assert!(certificate.dialectic.lehman_multiplier.is_none());
            }
            support => panic!("gap family closed in unexpected support {support}"),
        }

        supports.insert(summary.terminal_support);
        observed.push((
            q_value - p_value,
            summary.descents,
            summary.terminal_support,
            dec_of(&certificate.dialectic.lattice_cell),
        ));
    }

    assert_eq!(supports, BTreeSet::from([39u32, 47u32]));
    let transitions: Vec<_> = observed
        .windows(2)
        .filter(|w| w[0].2 != w[1].2)
        .map(|w| (w[0].clone(), w[1].clone()))
        .collect();
    assert!(!transitions.is_empty(), "gap family exposed no imscription ring transition");

    for (left, right) in transitions {
        println!(
            "imscription ring transition: gap={} depth={} support={} cell={} -> gap={} depth={} support={} cell={}",
            left.0, left.1, left.2, left.3,
            right.0, right.1, right.2, right.3,
        );
    }
    println!("imscription gap scale: {:?}", observed);
}

#[test]
fn multiplier_family_scales_lattice_change_to_sixty_six_exact_descents() {
    let p_value = next_prime(1_000_000);
    let p = tape_u64(p_value);
    let multipliers = [2u64, 3, 4, 5, 8, 10, 16, 24, 32, 48, 64];
    let mut observed = Vec::new();

    for k in multipliers {
        let q_value = next_prime(
            p_value
                .checked_mul(k)
                .expect("multiplier semiprime fixture overflow"),
        );
        let q = tape_u64(q_value);
        let n = mul(&p, &q);
        let start = DialecticObject::new(n).unwrap();
        let certificate = certify_imscription_cycle(&start).unwrap();
        let summary = verify_imscription_cycle(&certificate).unwrap();

        assert_eq!(summary.terminal_support, 63);
        assert_eq!(summary.descents, k as usize + 2);
        assert_eq!(summary.quotient_transforms, summary.descents);
        assert_eq!(summary.quotient_generations, summary.descents + 1);
        assert_eq!(
            certificate.dialectic.lehman_multiplier.as_deref(),
            Some(tape_u64(k).as_slice()),
            "designed multiplier family did not close on its imscribed k",
        );
        assert_eq!(cmp(&certificate.dialectic.lattice_cell, &tape_u64(64)), Ordering::Less);

        let (short, extended, lehman) = topology(&certificate);
        assert_eq!(short, 1);
        assert_eq!(extended, 1);
        assert_eq!(lehman, k as usize);

        assert!(witness_valid(
            &summary.fixed_carrier.n,
            &summary.fixed_carrier.p,
            &summary.fixed_carrier.q,
        ));
        assert!(same_pair(
            &summary.fixed_carrier.p,
            &summary.fixed_carrier.q,
            &p,
            &q,
        ));
        assert_eq!(summary.fixed_carrier.encode(), certificate.dialectic.terminal_carrier);

        observed.push((
            k,
            q_value - p_value,
            summary.descents,
            summary.quotient_transforms,
            dec_of(&certificate.dialectic.lattice_cell),
        ));
    }

    assert_eq!(observed.first().unwrap().2, 4);
    assert_eq!(observed.last().unwrap().2, 66);
    println!("imscription multiplier scale: {:?}", observed);
}
