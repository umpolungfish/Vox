use vox::morphism_factor::{mul, tape_u64};
use vox::shor_braid::{shor_braid, shor_factor_via_braid, shor_factor_via_braid_scanned};
use vox::winding_readout::winding_number_tape;

#[test]
fn orbit_closes_past_the_removed_execution_cap() {
    // Regression for the former 2^18 step cutoff, independent of campaign width.
    let n = tape_u64(1009 * 1151);
    let a = tape_u64(2);
    let (word, _) = shor_braid(&a, &n).unwrap();
    assert_eq!(winding_number_tape(&word).unwrap(), tape_u64(289800));
    let (p, q) = shor_factor_via_braid(&a, &n).unwrap();
    assert_eq!(mul(&p, &q), n);
    assert!(p == tape_u64(1009) || p == tape_u64(1151));
}

#[test]
fn invalid_inputs_fail_and_base_scan_closes_without_a_budget() {
    for n in [0, 1] {
        assert!(shor_braid(&tape_u64(2), &tape_u64(n)).is_err());
    }
    assert!(shor_braid(&tape_u64(3), &tape_u64(15)).is_err());
    let (p, q, _) = shor_factor_via_braid_scanned(&tape_u64(15)).unwrap();
    assert_eq!(mul(&p, &q), tape_u64(15));
    assert!(shor_factor_via_braid_scanned(&tape_u64(7)).is_err());
}
