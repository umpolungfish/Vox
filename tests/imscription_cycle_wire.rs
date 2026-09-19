use vox::dialectic_certificate::{
    certify_dialectic, decode_dialectic_certificate, encode_dialectic_certificate,
    verify_dialectic_certificate,
};
use vox::dialectic_reentry::DialecticObject;
use vox::imscription_cycle::{
    certify_imscription_cycle, decode_imscription_cycle, encode_imscription_cycle,
    verify_imscription_cycle,
};
use vox::morphism_factor::{miller_rabin, mul, tape_u64};
use vox::vox::{EVALF, EVALT};

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

fn deep_fixture() -> (Vec<char>, Vec<char>, Vec<char>) {
    let p_value = next_prime(1_000_000);
    let q_value = next_prime(p_value.checked_mul(64).unwrap());
    let p = tape_u64(p_value);
    let q = tape_u64(q_value);
    let n = mul(&p, &q);
    (n, p, q)
}

fn read_len(word: &[char], cursor: &mut usize) -> usize {
    assert_eq!(word[*cursor], '∈');
    *cursor += 1;
    let mut value = 0usize;
    let mut bit = 0usize;
    while word[*cursor] != '∋' {
        match word[*cursor] {
            EVALT => {}
            EVALF => value |= 1usize << bit,
            other => panic!("non-numeral length mark {other}"),
        }
        bit += 1;
        *cursor += 1;
    }
    *cursor += 1;
    value
}

#[test]
fn dialectic_certificate_itself_is_a_restartable_marks_only_object() {
    let cases = [
        (1_000_003u64, 1_001_003u64, 1usize, 39u32),
        (1_000_003u64, 1_032_007u64, 2usize, 47u32),
        (1_000_003u64, 10_000_019u64, 12usize, 63u32),
    ];

    for (p_u64, q_u64, expected_descents, expected_support) in cases {
        let n = mul(&tape_u64(p_u64), &tape_u64(q_u64));
        let start = DialecticObject::new(n).unwrap();
        let certificate = certify_dialectic(&start).unwrap();
        let wire = encode_dialectic_certificate(&certificate);

        // Kill every typed object; the wire is now the only continuation state.
        let restored = decode_dialectic_certificate(&wire).unwrap();
        assert_eq!(restored, certificate);
        let summary = verify_dialectic_certificate(&restored).unwrap();
        assert_eq!(summary.descents, expected_descents);
        assert_eq!(summary.terminal_support, expected_support);

        println!(
            "dialectic wire restart: pair={}x{} descents={} support={} marks={}",
            p_u64,
            q_u64,
            summary.descents,
            summary.terminal_support,
            wire.len(),
        );
    }
}

#[test]
fn complete_deep_cycle_round_trips_as_one_wire_object_and_rejects_inner_forgery() {
    let (n, _p, _q) = deep_fixture();
    let start = DialecticObject::new(n).unwrap();
    let certificate = certify_imscription_cycle(&start).unwrap();
    let before = verify_imscription_cycle(&certificate).unwrap();
    assert_eq!(before.descents, 66);
    assert_eq!(before.quotient_transforms, 66);
    assert_eq!(before.terminal_support, 63);

    let wire = encode_imscription_cycle(&certificate);
    let restored = decode_imscription_cycle(&wire).unwrap();
    assert_eq!(restored, certificate);
    let after = verify_imscription_cycle(&restored).unwrap();
    assert_eq!(after.descents, 66);
    assert_eq!(after.quotient_transforms, 66);
    assert_eq!(after.quotient_generations, 67);
    assert_eq!(after.fixed_carrier.encode(), before.fixed_carrier.encode());

    // Keep every outer length field intact but corrupt the exact lifted bridge
    // inside the second blob. Structural decoding may still succeed; semantic
    // replay must reject the forged complete cycle.
    let mut forged = wire.clone();
    let mut cursor = 2usize;
    let dialectic_len = read_len(&forged, &mut cursor);
    cursor += dialectic_len;
    let lifted_len = read_len(&forged, &mut cursor);
    let lifted_start = cursor;
    let lifted_end = lifted_start + lifted_len;
    let flip_at = (lifted_start..lifted_end)
        .find(|&i| forged[i] == EVALF || forged[i] == EVALT)
        .expect("lifted carrier contained no numeral mark");
    forged[flip_at] = if forged[flip_at] == EVALF { EVALT } else { EVALF };
    let forged_cycle = decode_imscription_cycle(&forged).unwrap();
    assert!(verify_imscription_cycle(&forged_cycle).is_err());

    // Length corruption is a framing failure rather than a semantic proof failure.
    let mut bad_length = wire.clone();
    let length_mark = bad_length[3];
    assert!(length_mark == EVALF || length_mark == EVALT);
    bad_length[3] = if length_mark == EVALF { EVALT } else { EVALF };
    assert!(decode_imscription_cycle(&bad_length).is_err());

    println!(
        "imscription outer wire: 66 descents + 66 quotient transforms survive one marks-only restart; inner bridge and length forgeries rejected; marks={}",
        wire.len(),
    );
}
