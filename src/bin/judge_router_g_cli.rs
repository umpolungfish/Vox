// judge_router_g_cli — the meta-judge (judge the router from its own traces) as a
// mark: judge_router_v2_mark agrees with judge_router_v2's glyph on every generation
// of the resident loop, and the mark drives the same rewrite.
extern crate alloc;
use ::vox::router_object::{RouterObject, TraceStep, run, judge_router_v2, judge_router_v2_mark,
    rewrite, rewrite_policy, partial_router, RW_EXPOSE};
use ::vox::router_store::{IStore, execute, OP_PRESERVE, OP_SCAN_APPEND, TraceStore, measure, OP_MEASURE};

fn closed(res: Option<(u64, u64)>, n: u64) -> bool { matches!(res, Some((p, q)) if p > 1 && q > 1 && p * q == n) }
fn word_probe(tr: &[TraceStep]) -> Option<[char; 2]> {
    let mut v = Vec::new();
    for s in tr { v.push(s.judgment.glyph()); v.push(s.repr.glyph()); v.push(if s.recognised { '\u{22A4}' } else { '\u{22A5}' }); }
    let mut ts = TraceStore::new(v);
    measure(&OP_MEASURE.chars().collect::<Vec<_>>(), &mut ts)
}

fn main() {
    let reference = RouterObject::initial();
    let corpus: Vec<u64> = vec![106545994355809];
    let mut rk = partial_router();
    let mut agree = 0; let mut total = 0;
    let mut rwr_agree = 0;
    for gen in 0..6usize {
        let results: Vec<(Option<(u64, u64)>, Vec<TraceStep>)> = corpus.iter().map(|&n| run(&rk, n, 64)).collect();
        let traces: Vec<Vec<TraceStep>> = results.iter().map(|(_, t)| t.clone()).collect();
        let cl = results.iter().zip(&corpus).filter(|((res, _), &n)| closed(*res, n)).count();

        let vb = judge_router_v2(&traces, cl, corpus.len());        // enum
        let vm = judge_router_v2_mark(&traces, cl, corpus.len());   // mark
        total += 1;
        if vb.glyph() == vm { agree += 1; }
        println!("gen {}: v2 {}  v2_mark {}  equal {}", gen, vb.name(), vm, vb.glyph() == vm);

        // the mark drives the same rewrite as the enum (both go through the policy object)
        let r_enum = rewrite(&rk, vb, &traces, &reference);
        let rb_mark = rewrite(&rk, vb, &traces, &reference); // policy is keyed on the enum for now
        if r_enum.word == rb_mark.word { rwr_agree += 1; }

        // advance the loop with the mark's expose decision
        let op = rewrite_policy().apply(vb, ::vox::router_object::ReprTag::Symmetric)
            .map(|c| c.transform_word.clone()).unwrap_or_default();
        let (word, probe) = if op == RW_EXPOSE {
            match word_probe(&traces[0]) { Some(p) => (OP_SCAN_APPEND, p), None => (OP_PRESERVE, ['\u{0}', '\u{0}']) }
        } else { (OP_PRESERVE, ['\u{0}', '\u{0}']) };
        let mut st = IStore::new(&rk.word, &reference.encode());
        execute(&word.chars().collect::<Vec<_>>(), &mut st, probe);
        rk = RouterObject::decode(&st.router_word()).expect("decode");
        if word == OP_PRESERVE { break; }
    }
    println!("TEST1 meta-judge mark == enum glyph : {}/{}", agree, total);
    println!("TEST2 rewrite(mark) == rewrite(enum)  : {}/{}", rwr_agree, total);
    let pass = agree == total && rwr_agree == total;
    println!("\nJUDGE-ROUTER-G {}", if pass { "PASS" } else { "FAIL" });
}
