use vox::dialectic_certificate::{certify_dialectic, verify_dialectic_certificate};
use vox::dialectic_reentry::DialecticObject;
use vox::morphism_factor::{mul, tape_u64};

#[test]
fn deleting_or_duplicating_any_checkpoint_breaks_exact_continuity() {
    let start = DialecticObject::new(mul(&tape_u64(1_000_003), &tape_u64(10_000_019))).unwrap();
    let certificate = certify_dialectic(&start).unwrap();
    verify_dialectic_certificate(&certificate).unwrap();
    // The first checkpoint defines this certificate's start. A suffix is a
    // legitimate restart, so deletion controls begin after that checkpoint.
    for index in 1..certificate.objects.len() {
        let mut deleted = certificate.clone();
        deleted.objects.remove(index);
        assert!(verify_dialectic_certificate(&deleted).is_err(), "deleted {index}");
    }
    for index in 0..certificate.objects.len() {
        let mut duplicated = certificate.clone();
        duplicated.objects.insert(index, duplicated.objects[index].clone());
        assert!(verify_dialectic_certificate(&duplicated).is_err(), "duplicated {index}");
        let mut suffix = certificate.clone();
        suffix.objects = suffix.objects[index..].to_vec();
        let verified = verify_dialectic_certificate(&suffix).unwrap();
        assert_eq!(verified.terminal_carrier.encode(), certificate.terminal_carrier);
        assert_eq!(verified.descents, certificate.objects.len()-index);
    }
}
