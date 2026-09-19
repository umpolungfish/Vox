use core::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

use vox::factor_extract::{extract, FactorCarrier};
use vox::factorization_31_membrane::{UnboundedResident, WORD};
use vox::morphism_factor::{
    cmp, dec_of, divmod, miller_rabin, mul, tape_u64, zero,
};
use vox::provenance_envelope::{restored_support_ladder, suffix_fibre_size};
use vox::reentry_certificate::{
    certify_reentry, decode_reentry_certificate, encode_reentry_certificate,
    verify_reentry_certificate,
};
use vox::router_marks::{GStep, M_B, M_FIX, M_N, M_T};
use vox::trace_algebra::witness_valid;
use vox::trace_word::encode_trace;
use vox::vox::{EVALF, EVALT};

fn all_sequences(k: u32, d: usize) -> Vec<Vec<u32>> {
    let base = 1u64 << k;
    let total = 1u64 << (k as usize * d);
    (0..total)
        .map(|mut n| {
            let mut q = Vec::with_capacity(d);
            for _ in 0..d {
                q.push((n % base) as u32);
                n /= base;
            }
            q
        })
        .collect()
}

fn terminal_step() -> GStep {
    GStep {
        repr: '⋈',
        judgment: M_T,
        recognised: M_T,
        next: M_FIX,
        applied_word: "⊢⊙⊡⊣".chars().collect(),
    }
}

fn trace_for_deposits(q: &[u32], lane_bits: u32) -> Vec<char> {
    let mut steps = Vec::with_capacity(q.len() + 1);
    for (i, &mask) in q.iter().enumerate() {
        let mut payload = vec!['⊢', '∈'];
        for bit in 0..lane_bits {
            payload.push(if (mask >> bit) & 1 == 1 { EVALF } else { EVALT });
        }
        payload.extend(['∋', '⊙', if i % 2 == 0 { '≻' } else { '≺' }, '⊣']);
        steps.push(GStep {
            repr: match i % 3 {
                0 => '⊢',
                1 => '⊣',
                _ => '⋈',
            },
            judgment: if mask == 0 { M_N } else { M_B },
            recognised: M_T,
            next: '⋈',
            applied_word: payload,
        });
    }
    steps.push(terminal_step());
    encode_trace(&steps)
}

#[test]
fn native_suffix_fibres_embed_as_exact_relaxed_representative_multiplicities() {
    const K: u32 = 2;
    const D: usize = 3;

    let mut fibres: BTreeMap<Vec<u32>, Vec<Vec<u32>>> = BTreeMap::new();
    for q in all_sequences(K, D) {
        fibres.entry(restored_support_ladder(&q)).or_default().push(q);
    }
    assert_eq!(fibres.len(), (D + 1).pow(K));

    let n = tape_u64(8051);
    let p = tape_u64(83);
    let q_factor = tape_u64(97);
    let terminal = encode_trace(&[terminal_step()]);
    let mut representatives = 0usize;

    for (ladder, deposits) in fibres {
        let expected = suffix_fibre_size(&ladder).expect("native ladder was not descending") as usize;
        assert_eq!(deposits.len(), expected, "fibre multiplicity changed for {ladder:?}");

        let mut encoded_representatives = BTreeSet::new();
        for deposit_schedule in deposits {
            let trace = trace_for_deposits(&deposit_schedule, K);
            let carrier = FactorCarrier::new(
                n.clone(),
                p.clone(),
                q_factor.clone(),
                trace,
            )
            .unwrap();
            assert!(encoded_representatives.insert(carrier.encode()));

            let readout = extract(&carrier).unwrap();
            assert_eq!(readout.transforms, D);
            assert_eq!(readout.generations.len(), D + 1);
            assert_eq!(readout.normal_form, terminal);

            let cert = certify_reentry(&carrier).unwrap();
            let summary = verify_reentry_certificate(&cert).unwrap();
            assert_eq!(summary.transforms, D);
            assert_eq!(summary.normal_form, terminal);
            representatives += 1;
        }
        assert_eq!(encoded_representatives.len(), expected);
    }

    assert_eq!(representatives, 1usize << (K as usize * D));
    println!(
        "syzygy fibre: {} native deposit representatives across {} restored-support fibres all collapse to one certified factor-bearing fixed object",
        representatives,
        (D + 1).pow(K),
    );
}

fn resident_program_trace() -> Vec<char> {
    let glyphs: Vec<char> = WORD.chars().collect();
    assert_eq!(glyphs.len(), 31);
    let steps: Vec<GStep> = glyphs
        .iter()
        .enumerate()
        .map(|(i, &glyph)| {
            let last = i + 1 == glyphs.len();
            GStep {
                repr: '⋈',
                judgment: if last { M_T } else { M_B },
                recognised: M_T,
                next: if last { M_FIX } else { '⋈' },
                applied_word: vec![glyph],
            }
        })
        .collect();
    encode_trace(&steps)
}

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

#[test]
fn controlled_gap_semiprimes_enter_resident_then_collapse_through_certificates() {
    let p_value = next_prime(1_000_000);
    let requested_gaps = [2u64, 1_000, 100_000, 1_000_000, 4_000_000];
    let p_expected = tape_u64(p_value);

    for requested_gap in requested_gaps {
        let q_value = next_prime(p_value + requested_gap);
        let q_expected = tape_u64(q_value);
        let n = mul(&p_expected, &q_expected);

        // Only N enters the factor producer. The expected primes remain outside
        // the resident and are used after production as the experiment oracle.
        let mut resident = UnboundedResident::new(n.clone());
        resident.run();
        assert!(resident.boundary_ok, "resident boundary failed for requested gap {requested_gap}");
        assert!(resident.sidearm_round_trip, "resident sidearm failed for requested gap {requested_gap}");

        let one = tape_u64(1);
        let factor = resident
            .factors
            .iter()
            .find(|f| cmp(f, &one) == Ordering::Greater && cmp(f, &n) == Ordering::Less)
            .expect("resident produced no proper factor")
            .clone();
        let (cofactor, remainder) = divmod(&n, &factor);
        assert!(zero(&remainder));
        assert!(witness_valid(&n, &factor, &cofactor));
        assert!(
            same_pair(&factor, &cofactor, &p_expected, &q_expected),
            "resident returned the wrong pair for requested gap {requested_gap}"
        );

        let carrier = FactorCarrier::new(
            n.clone(),
            factor.clone(),
            cofactor.clone(),
            resident_program_trace(),
        )
        .unwrap();
        let readout = extract(&carrier).unwrap();
        assert_eq!(cmp(&mul(&readout.p, &readout.q), &n), Ordering::Equal);

        let cert = certify_reentry(&carrier).unwrap();
        let wire = encode_reentry_certificate(&cert);
        let reconstructed = decode_reentry_certificate(&wire).unwrap();
        let summary = verify_reentry_certificate(&reconstructed).unwrap();
        assert_eq!(summary.transforms, 30);
        assert_eq!(summary.generations, 31);
        assert_eq!(summary.normal_form, readout.normal_form);

        println!(
            "semiprime gap: requested={} actual={} N_digits={} p={} q={} route={} cert_marks={}",
            requested_gap,
            q_value - p_value,
            dec_of(&n).len(),
            p_value,
            q_value,
            resident.shape_route,
            wire.len(),
        );
    }
}
