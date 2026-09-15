//! Native batch membrane with all operands resident before execution.
//!
//! The decimal operands are compile-time constants.  Each report is retained
//! until the complete batch has finished, so the process emits no payload
//! output during factoring.

const BAKED: &[&str] = &[
    "10000000000000000016800000000000000005031",
    "1000000000000000000000000000370000000000000000000000000026829",
    "100000000000000000039000000009900000000000000003861",
];

fn main() {
    let tower = ::vox::complete_membrane::CompleteMembrane::new(8).unwrap();
    let mut reports = Vec::with_capacity(BAKED.len());
    for decimal in BAKED {
        let started = std::time::Instant::now();
        let carrier = decimal.as_bytes();
        assert!(tower.forward_pairs().iter().all(|&(from, to)|
            tower.preserves(from, to, carrier)));
        let word = ::vox::morphism_factor::emit_numeral(
            &::vox::morphism_factor::decimal_to_tape(decimal)
                .unwrap_or_else(|| panic!("invalid baked decimal: {decimal}")));
        let report = ::vox::factorization_31_membrane::dispatch_report_word(&word)
            .unwrap_or_else(|e| panic!("baked membrane failed for {decimal}: {e}"));
        reports.push((report, started.elapsed()));
    }
    for (report, elapsed) in reports {
        println!("{report}\nnative_item_seconds={:.6}", elapsed.as_secs_f64());
    }
}
