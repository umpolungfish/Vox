// router_self_cli — the self-reconstruction experiment.
//
// R0 = the initial router (an IMASM object). Run a corpus. Judge the ROUTER from
// its own trajectories -> v. Rewrite: R1 = rewrite(R0, v). R1 is still an IMASM
// word. Invariants:
//   (1) round-trip exact:  E(D(R)) == R      and  E(D(W)) == W for the word
//   (2) behavioural closure: R1 closes every N that R0 closed
//   (3) new closures: R1 closes N's that R0 did not
extern crate alloc;
use ::vox::router_object::{RouterObject, Belnap, TraceStep, run, judge_router, rewrite};

fn closed(res: Option<(u64, u64)>, n: u64) -> bool {
    matches!(res, Some((p, q)) if p > 1 && q > 1 && p * q == n)
}

fn reached_t(tr: &[TraceStep]) -> bool { tr.iter().any(|s| s.judgment == Belnap::T) }

fn collect_traces(r: &RouterObject, corpus: &[u64]) -> Vec<Vec<TraceStep>> {
    corpus.iter().map(|&n| run(r, n, 64).1).collect()
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let corpus: Vec<u64> = if args.is_empty() {
        vec![
            1000036000099,       // 1000003 x 1000033   (near root)
            4000064000087,       // 1999993 x 2000023   (near root)
            61304504203,         // 221003 x 277387
            723862509691,        // 723901 x 1000033
            8097694716157,       // 2767907 x 2917103
            106545994355809,     // 9245087 x 11524607
            38531827129376957,   // 55-bit
        ]
    } else {
        args.iter().filter_map(|s| s.parse().ok()).collect()
    };

    // ---- generation 0 ----
    let mut r0 = RouterObject::initial();
    r0.canonicalize();
    let w0 = r0.encode();
    let rt0 = RouterObject::decode(&w0).map(|r| r.encode()) == Some(w0.clone());
    println!("=== router generation 0 ===");
    println!("router word ({} glyphs): {}", w0.chars().count(), w0);
    println!("clauses: {}", r0.transitions.len());
    println!("round-trip E(D(R0))==R0: {}", rt0);

    let mut traces0: Vec<Vec<TraceStep>> = Vec::new();
    let mut closed0 = 0usize;
    let mut reentry0 = 0usize;
    for &n in &corpus {
        let (res, tr) = run(&r0, n, 64);
        if closed(res, n) { closed0 += 1; }
        reentry0 += tr.len();
        traces0.push(tr);
    }
    let v0 = judge_router(&traces0, closed0, corpus.len());
    println!("router verdict v0: {}  (closures {}/{})", v0.name(), closed0, corpus.len());
    println!("re-entries: {}", reentry0);

    // ---- rewrite: R1 = rewrite(R0, v0) -- judged, not matched ----
    let r1 = rewrite(&r0, v0, &traces0, &r0);
    let w1 = r1.encode();
    println!();
    println!("=== router generation 1 ===");
    println!("rewrite word v0 -> R1 ({})", v0.name());
    println!("router word ({} glyphs): {}", w1.chars().count(), w1);
    println!("clauses: {}", r1.transitions.len());
    println!("new clauses vs R0: {}", r1.transitions.len().saturating_sub(r0.transitions.len()));

    // ---- invariant (1): round-trip exact ----
    let rt1 = RouterObject::decode(&w1).map(|r| r.encode()) == Some(w1.clone());
    println!("round-trip E(D(R1))==R1: {}", rt1);

    // ---- invariant (2)+(3): behavioural closure ----
    let mut closed1 = 0usize;
    let mut reentry1 = 0usize;
    let mut recog1 = 0usize;
    let mut new_closed: Vec<u64> = Vec::new();
    let mut preserved = 0usize;
    for (idx, &n) in corpus.iter().enumerate() {
        let (res, tr) = run(&r1, n, 64);
        let c = closed(res, n);
        if c { closed1 += 1; }
        if c && reached_t(&traces0[idx]) { preserved += 1; }
        if c && !reached_t(&traces0[idx]) { new_closed.push(n); }
        if tr.iter().all(|s| s.recognised) { recog1 += 1; }
        reentry1 += tr.len();
    }
    let v1 = judge_router(&collect_traces(&r1, &corpus), closed1, corpus.len());

    println!();
    println!("=== metrics ===");
    println!(" 1 router generation          : 1  (R0 -> R1)");
    println!(" 2 router word R0             : {}", w0);
    println!(" 2 router word R1             : {}", w1);
    println!(" 3 router verdict v0 / v1     : {} / {}", v0.name(), v1.name());
    println!(" 4 word length R0 / R1        : {} / {}", w0.chars().count(), w1.chars().count());
    println!(" 5 re-entry count R0 / R1     : {} / {}", reentry0, reentry1);
    println!(" 6 trajectory classes recog.  : {}/{}", recog1, corpus.len());
    println!(" 7 repr classes R0 / R1       : {} / {}", r0.transitions.len(), r1.transitions.len());
    println!(" 8 old closures preserved     : {}/{}", preserved, closed0);
    println!(" 9 new closures obtained      : {}", new_closed.len());
    for n in &new_closed { println!("     newly closed: {}", n); }
    println!("10 closures R0 -> R1          : {}/{} -> {}/{}", closed0, corpus.len(), closed1, corpus.len());
    println!();
    // ---- growth: reconstruct the ladder from a partial router ----
    println!();
    println!("=== growth: R* reconstructed from a partial router ===");
    let reference = RouterObject::initial();
    let probe: Vec<u64> = vec![106545994355809];
    let mut rk = ::vox::router_object::partial_router();
    let mut rounds = 0usize;
    loop {
        let traces_k: Vec<Vec<TraceStep>> = probe.iter().map(|&n| run(&rk, n, 64).1).collect();
        let cl = probe.iter().filter(|&&n| closed(run(&rk, n, 64).0, n)).count();
        let vk = judge_router(&traces_k, cl, probe.len());
        let rn = ::vox::router_object::rewrite(&rk, vk, &traces_k, &reference);
        println!("  round {}: clauses {} -> {}  verdict {}  word {} glyphs", rounds, rk.transitions.len(), rn.transitions.len(), vk.name(), rn.word.chars().count());
        if rn.encode() == rk.encode() || rounds >= 8 { rk = rn; break; }
        rk = rn; rounds += 1;
    }
    let rt_k = RouterObject::decode(&rk.word).map(|r| r.encode()) == Some(rk.word.clone());
    let cl_final = probe.iter().filter(|&&n| closed(run(&rk, n, 64).0, n)).count();
    println!("  R* clauses: {}  round-trip: {}  probe closed: {}/{}", rk.transitions.len(), rt_k, cl_final, probe.len());

    let pass = rt0 && rt1 && preserved == closed0 && closed1 >= closed0;
    println!("SELF-RECONSTRUCTION: {}", if pass { "PASS" } else { "OPEN" });
}
