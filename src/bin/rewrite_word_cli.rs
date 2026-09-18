// rewrite_word_cli — WORD-NATIVE TRAJECTORY REWRITE. The loop advances with
// rewrite_word (probe read off the trace tape, SCAN-APPEND, no enum step). Byte-equal to
// the enum oracle rewrite() at every generation of the self-repair sequence 1 → 2 → 3.
extern crate alloc;
use ::vox::router_object::{RouterObject, run, judge_router_v2, rewrite, partial_router};
use ::vox::router_marks::run_word;
use ::vox::trace_word::{encode_trace, judge_router_trace};
use ::vox::router_marks::rewrite_word;

fn closed(res: Option<(u64, u64)>, n: u64) -> bool { matches!(res, Some((p, q)) if p > 1 && q > 1 && p * q == n) }

fn main() {
    let reference = RouterObject::initial();
    let corpus: Vec<u64> = vec![106545994355809];
    let mut rk = partial_router();

    println!("=== word-native trajectory rewrite ===   corpus {:?}", corpus);
    let mut growth: Vec<usize> = Vec::new();
    let mut verdicts_mark: Vec<char> = Vec::new();
    let mut byte_agree = 0usize; let mut gens = 0usize;
    for gen in 0..6usize {
        // enum trajectory + enum verdict (oracle)
        let results: Vec<(Option<(u64, u64)>, Vec<::vox::router_object::TraceStep>)> =
            corpus.iter().map(|&n| run(&rk, n, 64)).collect();
        let traces: Vec<Vec<::vox::router_object::TraceStep>> = results.iter().map(|(_, t)| t.clone()).collect();
        let cl = results.iter().zip(&corpus).filter(|((res, _), &n)| closed(*res, n)).count();
        let v_enum = judge_router_v2(&traces, cl, corpus.len());

        // resident trace words + mark verdict (no enum on this path)
        let word: Vec<char> = rk.word.chars().collect();
        let trace_words: Vec<Vec<char>> = corpus.iter().map(|&n| encode_trace(&run_word(&word, n, 64).1)).collect();
        let v_mark = judge_router_trace(&trace_words);

        // the two rewrites, compared byte-for-byte
        let r_host = rewrite(&rk, v_enum, &traces, &reference);
        let r_word = rewrite_word(&rk.word, &reference.encode(), &trace_words, v_mark);

        gens += 1;
        growth.push(rk.transitions.len());
        verdicts_mark.push(v_mark);
        let eq = r_host.word == r_word;
        if eq { byte_agree += 1; }
        println!("gen {}: clauses {}  verdict {}  rewrite_word == rewrite bytes: {} ({} marks)",
            gen, rk.transitions.len(), v_mark, eq, r_word.chars().count());

        // advance with the WORD-NATIVE router
        rk = RouterObject::decode(&r_word).expect("decode");
        if v_mark == '⊤' { break; }
    }

    // final closure on the semiprime with the word-native final router
    let word: Vec<char> = rk.word.chars().collect();
    let mut closed_ct = 0;
    for &n in &corpus { if closed(run_word(&word, n, 64).0, n) { closed_ct += 1; } }

    println!("growth {:?}  verdicts(mark) {}", growth, verdicts_mark.iter().collect::<String>());
    println!("final router {} clauses, {} marks, semiprime closed {}/{}",
        rk.transitions.len(), word.len(), closed_ct, corpus.len());

    let expected_growth = growth == alloc::vec![1usize, 2, 3];
    let pass = byte_agree == gens && closed_ct == corpus.len() && expected_growth;
    println!("\nREWRITE-WORD {}", if pass { "PASS" } else { "FAIL" });
}
