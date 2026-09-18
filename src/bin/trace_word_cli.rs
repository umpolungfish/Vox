// trace_word_cli — Stage 13: the trace is only a word. GStep is a reader; the
// production representation is the record word. Round-trip exact, judge structural,
// replay positional (applied_word = ⊙ means no operation was committed).
extern crate alloc;
use ::vox::router_object::RouterObject;
use ::vox::router_marks::{RouterG, run_word, GStep, M_T, M_B, M_N, M_F, M_FIX};
use ::vox::trace_word::{encode_trace, decode_trace, judge_trace, replay};

fn steps_eq(a: &[GStep], b: &[GStep]) -> bool {
    a.len() == b.len() && a.iter().zip(b).all(|(x, y)|
        x.repr == y.repr && x.judgment == y.judgment && x.recognised == y.recognised
            && x.next == y.next && x.applied_word == y.applied_word)
}

fn main() {
    let rk = RouterObject::initial();
    let rg = RouterG::from_enum(&rk);
    let word: Vec<char> = rg.encode().chars().collect();

    let mut corpus: Vec<u64> = Vec::new();
    for n in 2u64..=100 { corpus.push(n); }
    corpus.extend_from_slice(&[121, 143, 209, 221, 247, 323, 106545994355809, 1000003, 8, 1024, 7919]);

    // ---- TEST 1: trace round-trip exact (struct -> word -> struct, and word stable) ----
    let mut rt = 0usize; let mut total = 0usize;
    let mut judge_ok = 0usize; let mut judge_total = 0usize;
    let mut has_dash = false;
    for &n in &corpus {
        let (_, traj) = run_word(&word, n, 64);
        let tw = encode_trace(&traj);
        if tw.contains(&'\u{2014}') { has_dash = true; }         // no dash sentinel anywhere
        let back = decode_trace(&tw).expect("decode");
        total += 1;
        if steps_eq(&traj, &back) && encode_trace(&back) == tw { rt += 1; }
        // the structural judge must agree with what the trajectory IS
        let jm = judge_trace(&tw);
        let expected = if traj.last().map(|s| s.judgment == M_T).unwrap_or(false)
                          && traj.iter().all(|s| s.recognised == M_T) { M_T }
                       else if traj.iter().any(|s| s.judgment == M_B) { M_B } else { M_N };
        judge_total += 1;
        if jm == expected { judge_ok += 1; }
    }

    // ---- TEST 2: replay is positional — ⊙ committed nothing ----
    let (_r, traj_big) = run_word(&word, 106545994355809, 64);
    let tw_big = encode_trace(&traj_big);
    let mut committed = 0usize;
    replay(&tw_big, |w| { committed += 1; let _ = w; });
    let committed_ops = traj_big.iter().filter(|s| !(s.applied_word.len() == 1 && s.applied_word[0] == M_N) && !s.applied_word.is_empty()).count();
    let replay_ok = committed == committed_ops;

    // ---- TEST 3: a malformed trace word judges F ----
    let mut bad = tw_big.clone(); bad.pop();                    // drop a closing ∋
    let bad_mark = judge_trace(&bad);
    let malformed_ok = bad_mark == M_F;

    // ---- TEST 4: the big semiprime trace is closed => ⊤ ----
    let big_mark = judge_trace(&tw_big);

    println!("TEST1 trace round-trip exact : {}/{}", rt, total);
    println!("TEST2 judge_trace == trajectory form : {}/{}", judge_ok, judge_total);
    println!("TEST3 replay positional: {}/{} committed ops  (ok {})", committed, committed_ops, replay_ok);
    println!("TEST4 malformed word -> ⊥ : {}   big semiprime trace -> {}", malformed_ok, big_mark);
    println!("       no dash sentinel in any trace word: {}", !has_dash);
    println!("       big trace = {} marks, records = {}", tw_big.len(), traj_big.len());

    let pass = rt == total && judge_ok == judge_total && replay_ok && malformed_ok
        && big_mark == M_T && !has_dash;
    println!("\nTRACE-WORD {}", if pass { "PASS" } else { "FAIL" });
    let _ = (M_N, M_FIX);
}
