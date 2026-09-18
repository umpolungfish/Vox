// self_repair_word_cli — the resident self-repair loop (1 clause -> 2 -> 3 -> T) with the
// enum-free router: at EVERY generation the word-only control law run_word reproduces the
// enum run() on the corpus, and the clause growth and final trajectories are unchanged.
extern crate alloc;
use ::vox::router_object::{RouterObject, ReprTag, TraceStep, run, judge_router_v2, rewrite,
    rewrite_policy, partial_router, RW_EXPOSE};
use ::vox::router_store::{IStore, execute, OP_PRESERVE, OP_SCAN_APPEND, TraceStore, measure, OP_MEASURE};
use ::vox::router_marks::{RouterG, run_word};

fn closed(res: Option<(u64, u64)>, n: u64) -> bool { matches!(res, Some((p, q)) if p > 1 && q > 1 && p * q == n) }
fn word_probe(tr: &[TraceStep]) -> Option<[char; 2]> {
    let mut v = Vec::new();
    for s in tr { v.push(s.judgment.glyph()); v.push(s.repr.glyph()); v.push(if s.recognised { '\u{22A4}' } else { '\u{22A5}' }); }
    let mut ts = TraceStore::new(v);
    measure(&OP_MEASURE.chars().collect::<Vec<_>>(), &mut ts)
}

fn main() {
    let reference = RouterObject::initial();
    let pol = rewrite_policy();
    let corpus: Vec<u64> = vec![106545994355809];
    let mut rk = partial_router();

    println!("=== self-repair, enum-free router ===   corpus {:?}", corpus);
    let mut growth: Vec<usize> = Vec::new();
    let mut verdicts: Vec<String> = Vec::new();
    let mut word_agree = 0; let mut gens = 0;
    for gen in 0..6usize {
        // enum trajectory
        let results: Vec<(Option<(u64, u64)>, Vec<TraceStep>)> = corpus.iter().map(|&n| run(&rk, n, 64)).collect();
        let traces: Vec<Vec<TraceStep>> = results.iter().map(|(_, t)| t.clone()).collect();
        let cl = results.iter().zip(&corpus).filter(|((res, _), &n)| closed(*res, n)).count();
        let v = judge_router_v2(&traces, cl, corpus.len());
        verdicts.push(v.name().into());
        growth.push(rk.transitions.len());

        // word-only control law at THIS generation, over the corpus
        let rg = RouterG::from_enum(&rk);
        let word: Vec<char> = rg.encode().chars().collect();
        let mut ok = true;
        for &n in &corpus {
            let (er, _) = run(&rk, n, 64);
            let (wr, _) = run_word(&word, n, 64);
            if er != wr { ok = false; }
        }
        if ok { word_agree += 1; }
        gens += 1;
        println!("gen {}: clauses {}  verdict {}  word-run == enum-run {}", gen, rk.transitions.len(), v.name(), ok);

        // resident rewrite (advance the router one level), then continue
        let r_host = rewrite(&rk, v, &traces, &reference);
        let op = pol.apply(v, ReprTag::Symmetric).map(|c| c.transform_word.clone()).unwrap_or_default();
        let (w, probe) = if op == RW_EXPOSE {
            match word_probe(&traces[0]) { Some(p) => (OP_SCAN_APPEND, p), None => (OP_PRESERVE, ['\u{0}', '\u{0}']) }
        } else { (OP_PRESERVE, ['\u{0}', '\u{0}']) };
        let mut st = IStore::new(&rk.word, &reference.encode());
        execute(&w.chars().collect::<Vec<_>>(), &mut st, probe);
        rk = RouterObject::decode(&st.router_word()).expect("decode");
        let _ = r_host;
        if w == OP_PRESERVE { break; }
    }
    println!("growth: {:?}   verdicts: {:?}", growth, verdicts);
    // final-generation closure check on the semiprime
    let rg = RouterG::from_enum(&rk);
    let word: Vec<char> = rg.encode().chars().collect();
    let mut closed_ct = 0;
    for &n in &corpus { if closed(run_word(&word, n, 64).0, n) { closed_ct += 1; } }
    println!("final router: {} clauses, word {} marks, semiprime closed {}/{}", rk.transitions.len(), word.len(), closed_ct, corpus.len());
    let pass = word_agree == gens && closed_ct == corpus.len();
    println!("\nSELF-REPAIR-WORD {}", if pass { "PASS" } else { "FAIL" });
}
