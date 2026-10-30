use vox::dialectic_reentry::DialecticObject;
use vox::imscription_cycle::{
    certify_imscription_cycle, cycle_checkpoints, decode_cycle_checkpoint,
    encode_cycle_checkpoint, resume_cycle_checkpoint, verify_imscription_cycle,
    CyclePhase,
};
use vox::morphism_factor::{miller_rabin, mul, tape_u64};

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

fn deep_start() -> DialecticObject {
    let p_value = next_prime(1_000_000);
    let q_value = next_prime(p_value.checked_mul(64).unwrap());
    let n = mul(&tape_u64(p_value), &tape_u64(q_value));
    DialecticObject::new(n).unwrap()
}

#[test]
fn every_complete_cycle_cut_restarts_from_its_own_marks_and_reaches_the_same_point() {
    let start = deep_start();
    let certificate = certify_imscription_cycle(&start).unwrap();
    let full = verify_imscription_cycle(&certificate).unwrap();
    assert_eq!(full.descents, 66);
    assert_eq!(full.quotient_transforms, 66);

    let fixed = full.fixed_carrier.encode();
    let checkpoints = cycle_checkpoints(&certificate).unwrap();
    assert_eq!(checkpoints.len(), 2 * full.descents + 1);
    assert_eq!(checkpoints.len(), 133);

    let mut imscription_cuts = 0usize;
    let mut quotient_cuts = 0usize;
    let mut total_checkpoint_marks = 0usize;

    for (index, checkpoint) in checkpoints.iter().enumerate() {
        let wire = encode_cycle_checkpoint(checkpoint);
        total_checkpoint_marks += wire.len();

        // The persisted checkpoint is the only continuation state.
        let restored = decode_cycle_checkpoint(&wire).unwrap();
        assert_eq!(&restored, checkpoint);
        let resumed = resume_cycle_checkpoint(&restored).unwrap();
        assert_eq!(resumed.fixed_carrier.encode(), fixed);

        if index < full.descents {
            let expected_remaining = full.descents - index;
            assert_eq!(restored.phase, CyclePhase::Imscription);
            assert_eq!(resumed.phase, CyclePhase::Imscription);
            assert_eq!(resumed.remaining_descents, expected_remaining);
            assert_eq!(resumed.remaining_quotient_transforms, expected_remaining);
            imscription_cuts += 1;
        } else {
            let quotient_index = index - full.descents;
            let expected_remaining = full.quotient_transforms - quotient_index;
            assert_eq!(restored.phase, CyclePhase::Quotient);
            assert_eq!(resumed.phase, CyclePhase::Quotient);
            assert_eq!(resumed.remaining_descents, 0);
            assert_eq!(resumed.remaining_quotient_transforms, expected_remaining);
            quotient_cuts += 1;
        }
    }

    assert_eq!(imscription_cuts, 66);
    assert_eq!(quotient_cuts, 67);
    println!(
        "imscription every-cut restart: {} imscription + {} quotient = {} persisted cuts all converge to one exact fixed carrier; checkpoint_marks_total={}",
        imscription_cuts,
        quotient_cuts,
        checkpoints.len(),
        total_checkpoint_marks,
    );
}

#[test]
fn checkpoint_phase_is_load_bearing_and_cannot_be_retyped() {
    let start = deep_start();
    let certificate = certify_imscription_cycle(&start).unwrap();
    let checkpoints = cycle_checkpoints(&certificate).unwrap();

    let first_imscription = checkpoints
        .iter()
        .find(|checkpoint| checkpoint.phase == CyclePhase::Imscription)
        .unwrap();
    let first_quotient = checkpoints
        .iter()
        .find(|checkpoint| checkpoint.phase == CyclePhase::Quotient)
        .unwrap();

    let mut imscription_wire = encode_cycle_checkpoint(first_imscription);
    let mut quotient_wire = encode_cycle_checkpoint(first_quotient);

    // Retagging preserves outer framing and the inner bytes, but changes how the
    // payload is interpreted. The phase/state type mismatch must fail structurally.
    imscription_wire[1] = '≺';
    quotient_wire[1] = '≻';
    assert!(decode_cycle_checkpoint(&imscription_wire).is_err());
    assert!(decode_cycle_checkpoint(&quotient_wire).is_err());

    println!("imscription checkpoint phase forgery: both cross-phase retypings rejected");
}
