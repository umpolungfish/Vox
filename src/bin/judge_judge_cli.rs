// judge_judge_cli — judge the judge: does the current judge conflate "no distinction"
// (unmatched but well-formed) with "failed recognition" (malformed)?
extern crate alloc;
use ::vox::router_object::{RouterObject, Belnap, TraceStep, run, judge_router, judge_router_v2, partial_router, no_n_router};

fn unmatched(tr: &[TraceStep]) -> usize { tr.iter().filter(|s| !s.recognised).count() }

fn main() {
    let mut init = RouterObject::initial(); init.canonicalize();
    let routers: Vec<(&str, RouterObject)> = vec![
        ("initial", init), ("partial", partial_router()), ("no_n", no_n_router()),
    ];
    let ns: Vec<u64> = vec![1000036000099, 61304504203, 723862509691, 106545994355809,
                            1000003, 1000000007];
    println!("=== judge the judge: v1 (unrecognized->F)  vs  v2 (unmatched->N) ===");
    let mut total = 0; let mut conflated = 0;
    for (rn, r) in &routers {
        for &n in &ns {
            let (res, tr) = run(r, n, 64);
            let cl = if matches!(res, Some((p, q)) if p > 1 && q > 1 && p * q == n) { 1 } else { 0 };
            let v1 = judge_router(&[tr.clone()], cl, 1);
            let v2 = judge_router_v2(&[tr.clone()], cl, 1);
            let u = unmatched(&tr);
            total += 1;
            let conf = v1 == Belnap::F && v2 == Belnap::N;
            if conf { conflated += 1; }
            println!("{:<8} N={:<18} steps {:>2} unmat {:>2}  v1 {} v2 {}{}",
                rn, n, tr.len(), u, v1.name(), v2.name(), if conf { "   [v1 F is really N]" } else { "" });
        }
    }
    println!("total {} ; v1-F-that-are-really-N: {}", total, conflated);

    let (_, tr) = run(&partial_router(), 106545994355809, 64);
    println!("witness trace (partial router, far semiprime) — every step a legal word:");
    for (k, s) in tr.iter().enumerate() {
        println!("  step {}: repr {:<14} judgment {}  recognised {:<5}  word {}",
            k, s.repr.name(), s.judgment.name(), s.recognised, s.applied_word);
    }
    // a genuinely malformed contrast would need an illegal composition; none occurs here.
    println!("JUDGE-THE-JUDGE: {}", if conflated > 0 { "CONFLATION EXPOSED — no separate malformed signal" } else { "none" });
}
