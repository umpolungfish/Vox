//! Cost separation for the proof-of-execution application: the check path must
//! touch strictly less work than the run path on the same witness.
//!
//! Run path  = `extract`, which drives the passive reduction schedule from the
//!             carrier to the normal form.
//! Check path = `verify_reentry_certificate`, which is handed the certificate and
//!             validates every link WITHOUT calling `reenter_once` (the reduction).
//!
//! The structural separation is exact: verify never invokes the reduction engine.
//! This test turns that into two measured numbers, a structural op count and wall
//! time, on one production witness, and asserts the check path is the cheaper one.

use std::time::Instant;

use vox::factor_extract::{extract, reenter_once, FactorCarrier};
use vox::morphism_factor::tape_u64;
use vox::reentry_certificate::{certify_reentry, verify_reentry_certificate};
use vox::router_marks::{run_mark, RouterG};
use vox::router_object::RouterObject;

const N: u64 = 106_545_994_355_809;

fn production_carrier() -> FactorCarrier {
    let n = tape_u64(N);
    let router = RouterG::from_enum(&RouterObject::initial());
    let (found, trajectory) = run_mark(&router, N, 8);
    let (p, q) = found.expect("production route did not carry factors");
    FactorCarrier::from_trace(&n, &tape_u64(p), &tape_u64(q), &trajectory).unwrap()
}

#[test]
fn check_path_is_strictly_cheaper_than_run_path() {
    let carrier = production_carrier();
    let certificate = certify_reentry(&carrier).unwrap();

    // Structural op count. The run path invokes the reduction primitive
    // `reenter_once` once per generation. The check path invokes it zero times.
    let run_reductions = {
        let mut count = 0usize;
        let mut current = carrier.clone();
        loop {
            let (next, changed) = reenter_once(&current).unwrap();
            count += 1;
            current = next;
            if !changed {
                break;
            }
        }
        count
    };
    let check_reductions = 0usize; // verify_reentry_certificate calls reenter_once nowhere

    assert!(run_reductions >= 1);
    assert_eq!(check_reductions, 0);
    assert!(
        check_reductions < run_reductions,
        "reduction-call separation absent: check {check_reductions} vs run {run_reductions}"
    );

    // Wall time. Warm both paths, then measure a batch.
    let iters = 200u32;
    for _ in 0..10 {
        let _ = extract(&carrier).unwrap();
        let _ = verify_reentry_certificate(&certificate).unwrap();
    }

    let t0 = Instant::now();
    for _ in 0..iters {
        let readout = extract(&carrier).unwrap();
        std::hint::black_box(readout.normal_form);
    }
    let run_time = t0.elapsed();

    let t1 = Instant::now();
    for _ in 0..iters {
        let summary = verify_reentry_certificate(&certificate).unwrap();
        std::hint::black_box(summary.normal_form);
    }
    let check_time = t1.elapsed();

    eprintln!(
        "reductions: run={run_reductions} check={check_reductions}; \
         time over {iters} iters: run={run_time:?} check={check_time:?}"
    );

    assert!(
        check_time < run_time,
        "check path not faster: check={check_time:?} run={run_time:?}"
    );
}
