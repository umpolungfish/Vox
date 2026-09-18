// router_g_cli — the router's control law in marks (run_g, judge_g) reproduces
// the enum control law (run) step-for-step: same closure, same judgment marks,
// same recognition flags, same next tags, same applied words.
extern crate alloc;
use ::vox::router_object::{RouterObject, run, run_g, partial_router};

fn main() {
    let routers: [(&str, RouterObject); 2] = [
        ("initial", RouterObject::initial()),
        ("partial", partial_router()),
    ];
    let mut corpus: Vec<u64> = Vec::new();
    for n in 2u64..=100 { corpus.push(n); }
    corpus.extend_from_slice(&[121, 143, 209, 221, 247, 323, 106545994355809, 1000003, 8, 1024]);

    let mut total = 0usize;
    let mut closure_agree = 0usize;
    let mut trace_agree = 0usize;
    let mut witness = String::from("none");

    for (rname, rk) in &routers {
        for &n in &corpus {
            let (er, etraj) = run(rk, n, 64);
            let (gr, gtraj) = run_g(rk, n, 64);
            total += 1;
            if er == gr { closure_agree += 1; } else if witness == "none" {
                witness = format!("closure {} n={} enum={:?} marks={:?}", rname, n, er, gr);
            }
            let mut ok = etraj.len() == gtraj.len();
            if ok {
                for (e, g) in etraj.iter().zip(&gtraj) {
                    if e.repr.glyph() != g.tag
                        || e.judgment.glyph() != g.judgment
                        || e.recognised != g.recognised
                        || e.next.map(|t| t.glyph()) != g.next
                        || e.applied_word != g.applied_word { ok = false; break; }
                }
            }
            if ok { trace_agree += 1; } else if witness == "none" {
                witness = format!("trace {} n={} enum_len={} mark_len={}", rname, n, etraj.len(), gtraj.len());
            }
        }
    }

    println!("routers x corpus: {}", total);
    println!("TEST1 closure run_g == run            : {}/{}", closure_agree, total);
    println!("TEST2 trace run_g == run (step-by-step): {}/{}", trace_agree, total);
    println!("TEST3 first disagreement: {}", witness);

    // TEST4: with the partial router (no advance clause) the loop must report the
    // unrecognised branch as a MARK step, no enum.
    let (_r, tg) = run_g(&routers[1].1, 106545994355809, 64);
    let marks: String = tg.iter().map(|s| s.judgment).collect();
    let unrecognised = tg.iter().any(|s| !s.recognised);
    println!("TEST4 partial router big-N mark trace = {}  (any unrecognised = {})", marks, unrecognised);

    let pass = closure_agree == total && trace_agree == total && witness == "none";
    println!("\nROUTER-G {}", if pass { "PASS" } else { "FAIL" });
}
