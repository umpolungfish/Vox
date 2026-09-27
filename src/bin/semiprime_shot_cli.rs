extern crate alloc;
use ::vox::morphism_factor::{cmp, dec_of, decimal_to_tape, mul, trim};

/// Accepts values of arbitrary size: every argument is an ASCII decimal string
/// lifted to a full-width tape via `decimal_to_tape`, and the p*q == N check
/// runs over tape arithmetic, so nothing is capped at u64 or u128.
///
/// The shot's node budget defaults to 500_000_000 and can be set with the
/// SEMIPRIME_BUDGET environment variable — full-width inputs spend real
/// compute per node, so the operator owns the budget.
fn main() {
    let budget: u64 = std::env::var("SEMIPRIME_BUDGET")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(500_000_000);
    let args: Vec<String> = std::env::args().skip(1).collect();
    let defaults = [
        "15", "21", "35", "91", "143", "8051", "16744463", "1000036000099",
        "4000064000087", "10000360000994000064000087",
    ];
    let srcs: Vec<String> = if args.is_empty() {
        defaults.iter().map(|s| s.to_string()).collect()
    } else {
        args
    };
    for s in srcs.iter() {
        let tape = match decimal_to_tape(s) {
            Some(t) => t,
            None => {
                println!("{s} -> rejected: not an ASCII decimal numeral");
                continue;
            }
        };
        let t0 = std::time::Instant::now();
        let (res, st) = ::vox::factor_operator::semiprime_shot(&tape, budget);
        let dt = t0.elapsed().as_millis();
        match res {
            Some((p, q)) => {
                let dp = dec_of(&p);
                let dq = dec_of(&q);
                let ok = cmp(&trim(mul(&p, &q)), &tape) == core::cmp::Ordering::Equal;
                println!("N={} -> {} x {}  ok={}  nodes={} low={} bridge={} high={} depth={} capped={}  {}ms",
                    s, dp, dq, ok, st.nodes, st.low_prunes, st.bridge_prunes, st.high_prunes, st.depth, st.capped, dt);
            }
            None => println!("N={} -> None  nodes={} capped={}  {}ms", s, st.nodes, st.capped, dt),
        }
    }
}
