// trace_reentry_cli — the trajectory re-enters as the object. The router is judged
// from its trajectory WORDS (judge_router_trace), and that verdict drives the rewrite.
// Cross-checked against the enum judge_router_v2 at every generation.
extern crate alloc;
use ::vox::router_object::{RouterObject, ReprTag, TraceStep, run, judge_router_v2, rewrite,
    rewrite_policy, partial_router, RW_EXPOSE};
use ::vox::router_store::{IStore, execute, OP_PRESERVE, OP_SCAN_APPEND, TraceStore, measure, OP_MEASURE};
use ::vox::router_marks::{RouterG, run_word};
use ::vox::trace_word::{encode_trace, decode_trace, judge_trace, judge_router_trace};

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

    println!("=== trajectory re-enters as the object: judge ∘ trace-word ===   corpus {:?}", corpus);
    let mut growth: Vec<usize> = Vec::new();
    let mut v_enum_seq: Vec<String> = Vec::new();
    let mut v_mark_seq: Vec<char> = Vec::new();
    let mut agree = 0usize; let mut gens = 0usize;
    for gen in 0..6usize {
        // enum trajectory (drives the host rewrite) + mark trajectory words (the judge's input)
        let results: Vec<(Option<(u64, u64)>, Vec<TraceStep>)> = corpus.iter().map(|&n| run(&rk, n, 64)).collect();
        let traces: Vec<Vec<TraceStep>> = results.iter().map(|(_, t)| t.clone()).collect();
        let cl = results.iter().zip(&corpus).filter(|((res, _), &n)| closed(*res, n)).count();
        let v_enum = judge_router_v2(&traces, cl, corpus.len());

        let rg = RouterG::from_enum(&rk);
        let word: Vec<char> = rg.encode().chars().collect();
        let trace_words: Vec<Vec<char>> = corpus.iter().map(|&n| encode_trace(&run_word(&word, n, 64).1)).collect();
        let v_mark = judge_router_trace(&trace_words);          // ← the trajectory judged from the tape

        gens += 1;
        if v_enum.glyph() == v_mark { agree += 1; }
        growth.push(rk.transitions.len());
        v_enum_seq.push(v_enum.name().into());
        v_mark_seq.push(v_mark);
        println!("gen {}: clauses {}  judge(traces-word) = {}  judge(enum) = {}  agree {}",
            gen, rk.transitions.len(), v_mark, v_enum.name(), v_enum.glyph() == v_mark);

        // re-enter: rewrite the router at the level the trajectory verdict exposes
        let r_host = rewrite(&rk, v_enum, &traces, &reference);
        let op = pol.apply(v_enum, ReprTag::Symmetric).map(|c| c.transform_word.clone()).unwrap_or_default();
        let (w, probe) = if op == RW_EXPOSE {
            match word_probe(&traces[0]) { Some(p) => (OP_SCAN_APPEND, p), None => (OP_PRESERVE, ['\u{0}', '\u{0}']) }
        } else { (OP_PRESERVE, ['\u{0}', '\u{0}']) };
        let mut st = IStore::new(&rk.word, &reference.encode());
        execute(&w.chars().collect::<Vec<_>>(), &mut st, probe);
        rk = RouterObject::decode(&st.router_word()).expect("decode");
        let _ = r_host;
        if w == OP_PRESERVE { break; }
    }

    // the final trajectory, judged as a word, must be ⊤ (closed + reconstruction verified)
    let rg = RouterG::from_enum(&rk);
    let word: Vec<char> = rg.encode().chars().collect();
    let final_tw = encode_trace(&run_word(&word, 106545994355809, 64).1);
    let final_j = judge_trace(&final_tw);
    let rt_ok = decode_trace(&final_tw).map(|t| encode_trace(&t) == final_tw).unwrap_or(false);

    println!("growth {:?}  verdicts(enum) {:?}  verdicts(mark) {}", growth, v_enum_seq, v_mark_seq.iter().collect::<String>());
    println!("final trajectory judged {} ; trace round-trip exact {}", final_j, rt_ok);

    let pass = agree == gens && final_j == '⊤' && rt_ok && v_mark_seq.last() == Some(&'⊤');
    println!("\nTRACE-REENTRY {}", if pass { "PASS" } else { "FAIL" });
}
