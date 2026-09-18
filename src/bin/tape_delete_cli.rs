// tape_delete_cli — DELETE is resident. A WORD walks the trace tape, identifies one
// record span and commits its complement. The Rust delete_record is kept ONLY as an
// oracle: we require byte equality delete_word(trace,i) == delete_record(trace,i) for
// every trace and every index, then drive the whole reduction through the word.
extern crate alloc;
use ::vox::router_object::RouterObject;
use ::vox::router_marks::run_word;
use ::vox::router_marks::M_T;
use ::vox::trace_word::{decode_trace, encode_trace, judge_trace};
use ::vox::trace_algebra::{delete_record, replay_state, factor_of};
use ::vox::tape_delete::{delete_word, DELETE_WORD};

/// Strict reduction driven ENTIRELY by the resident word (no host delete_record).
fn reduce_word(w: &[char], n: u64) -> (Vec<char>, usize) {
    let mut cur: Vec<char> = w.to_vec();
    let mut k = 0usize;
    'o: loop {
        let t = match decode_trace(&cur) { Some(t) => t, None => break };
        for i in 0..t.len() {
            if let Some(cand) = delete_word(&cur, i) {
                let (ro, rc) = match (replay_state(&cur), replay_state(&cand)) { (Some(a), Some(b)) => (a, b), _ => continue };
                if ro == rc && judge_trace(&cand) == M_T && factor_of(&ro, n) == factor_of(&rc, n) {
                    cur = cand; k += 1; continue 'o;
                }
            }
        }
        break;
    }
    (cur, k)
}

/// Relaxed (closure + factor) reduction, also driven by the word.
fn reduce_relaxed_word(w: &[char], n: u64) -> (Vec<char>, usize) {
    let mut cur: Vec<char> = w.to_vec();
    let mut k = 0usize;
    'o: loop {
        let t = match decode_trace(&cur) { Some(t) => t, None => break };
        let fo = replay_state(&cur).as_ref().and_then(|r| factor_of(r, n));
        for i in 0..t.len() {
            if let Some(cand) = delete_word(&cur, i) {
                let fc = replay_state(&cand).as_ref().and_then(|r| factor_of(r, n));
                if judge_trace(&cand) == M_T && fo.is_some() && fo == fc {
                    cur = cand; k += 1; continue 'o;
                }
            }
        }
        break;
    }
    (cur, k)
}

fn main() {
    let reference = RouterObject::initial();
    let word: Vec<char> = reference.encode().chars().collect();

    let mut corpus: Vec<u64> = Vec::new();
    for n in 2u64..=200 { corpus.push(n); }
    corpus.extend_from_slice(&[221, 247, 323, 391, 437, 529, 1000003, 104729, 106545994355809, 10000019]);

    // ---- 1. BYTE EQUALITY: word vs oracle, every trace, every index (and out-of-range) ----
    let mut eq_checks = 0usize;
    let mut eq_fail = 0usize;
    let mut none_checks = 0usize;
    let mut none_fail = 0usize;
    for &n in &corpus {
        let (_res, traj) = run_word(&word, n, 64);
        let tw = encode_trace(&traj);
        let len = match decode_trace(&tw) { Some(t) => t.len(), None => continue };
        for i in 0..len {
            let a = delete_word(&tw, i);
            let b = delete_record(&tw, i);
            eq_checks += 1;
            if a != b { eq_fail += 1; }
        }
        // out-of-range must both return None
        for i in len..len + 2 {
            let a = delete_word(&tw, i);
            let b = delete_record(&tw, i);
            none_checks += 1;
            if a != b { none_fail += 1; }
        }
    }

    // ---- 2. Drive the SAME reduction through the word ----
    let mut closed_ct = 0usize;
    let mut strict_shrunk = 0usize;
    let mut relaxed_shrunk = 0usize;
    let mut n2_line = String::new();
    let mut big_line = String::new();
    for &n in &corpus {
        let (_res, traj) = run_word(&word, n, 64);
        let tw = encode_trace(&traj);
        if judge_trace(&tw) != M_T { continue; }
        closed_ct += 1;
        let (red_s, ks) = reduce_word(&tw, n);
        let (red_r, kr) = reduce_relaxed_word(&tw, n);
        if ks > 0 { strict_shrunk += 1; }
        if kr > 0 { relaxed_shrunk += 1; }
        if n == 2 {
            let o = decode_trace(&tw).map(|t| t.len()).unwrap_or(0);
            let s = decode_trace(&red_s).map(|t| t.len()).unwrap_or(0);
            n2_line = format!("  n=2: {} recs -> strict {} recs ({} transforms)", o, s, ks);
        }
        if n == 106545994355809 {
            let _os = decode_trace(&tw).map(|t| t.len()).unwrap_or(0);
            let ss = decode_trace(&red_s).map(|t| t.len()).unwrap_or(0);
            let rs = decode_trace(&red_r).map(|t| t.len()).unwrap_or(0);
            big_line = format!("  big {}: marks {} -> strict {} ({} recs) ; relaxed {} ({} recs) ; factor {:?}",
                n, tw.len(), red_s.len(), ss, red_r.len(), rs, factor_of(&replay_state(&tw).unwrap(), n));
        }
    }

    println!("=== resident DELETE: a word walks the tape, commits the complement ===");
    println!("DELETE_WORD = \"{}\"  ({} marks)", DELETE_WORD, DELETE_WORD.chars().count());
    println!("byte-equality vs delete_record oracle: {}/{}", eq_checks - eq_fail, eq_checks);
    println!("out-of-range (None) agreement:         {}/{}", none_checks - none_fail, none_checks);
    println!("reduction driven by the word:");
    println!("closed trajectories: {}", closed_ct);
    println!("strict (replay-state-preserving) compressible: {}", strict_shrunk);
    println!("relaxed (closure+factor-preserving) compressible: {}", relaxed_shrunk);
    println!("{}", n2_line);
    println!("{}", big_line);

    let pass = eq_fail == 0 && none_fail == 0
        && closed_ct == 161 && strict_shrunk == 1 && relaxed_shrunk == 2;
    println!("\nTAPE-DELETE {}", if pass { "PASS" } else { "FAIL" });
}
