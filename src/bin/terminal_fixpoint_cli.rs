// terminal_fixpoint_cli — the n=2 post-closure spin, resolved two ways.
//   (a) the COMPRESSOR removes the spin: under run_word a closed state re-enters identically to
//       max_steps (n=2: 64 records), and DELETE reduces 64 -> 1.
//   (b) the CONTROL GRAMMAR makes a ⊤ closure a terminal fixed point: run_word_terminal halts on
//       the ⊡ (M_FIX) terminal marker instead of re-entering. Word-native, same factor for every n.
extern crate alloc;
use ::vox::router_object::RouterObject;
use ::vox::router_marks::{run_word, run_word_terminal};
use ::vox::router_marks::M_T;
use ::vox::trace_word::{encode_trace, judge_trace};
use ::vox::trace_algebra::{replay_state, factor_of};
use ::vox::tape_delete::delete_word;
use ::vox::trace_word::decode_trace;

fn reduce_word(w: &[char], n: u64) -> usize {
    let mut cur: Vec<char> = w.to_vec();
    let mut k = 0usize;
    'o: loop {
        let t = match decode_trace(&cur) { Some(t) => t, None => break };
        for i in 0..t.len() {
            if let Some(cand) = delete_word(&cur, i) {
                let (ro, rc) = match (replay_state(&cur), replay_state(&cand)) { (Some(a), Some(b)) => (a, b), _ => continue };
                if ro == rc && judge_trace(&cand) == M_T && factor_of(&ro, n) == factor_of(&rc, n) { cur = cand; k += 1; continue 'o; }
            }
        }
        break;
    }
    k
}

fn main() {
    let reference = RouterObject::initial();
    let word: Vec<char> = reference.encode().chars().collect();
    let mut corpus: Vec<u64> = Vec::new();
    for n in 2u64..=200 { corpus.push(n); }
    corpus.extend_from_slice(&[221, 247, 323, 391, 437, 529, 1000003, 104729, 106545994355809, 10000019]);

    let (mut closed_a, mut closed_b) = (0usize, 0usize);
    let mut factor_mismatch = 0usize;
    let mut shortened = 0usize;
    let mut max_a = 0usize; let mut max_b = 0usize;
    let mut grew = 0usize;
    let mut n2 = String::new(); let mut big = String::new();

    for &n in &corpus {
        let (res_a, traj_a) = run_word(&word, n, 64);
        let (res_b, traj_b) = run_word_terminal(&word, n, 64);
        let twa = encode_trace(&traj_a);
        let twb = encode_trace(&traj_b);
        if judge_trace(&twa) == M_T { closed_a += 1; }
        if judge_trace(&twb) == M_T { closed_b += 1; }
        if res_a != res_b { factor_mismatch += 1; }
        if traj_b.len() < traj_a.len() { shortened += 1; }
        if traj_b.len() > traj_a.len() { grew += 1; }
        if traj_a.len() > max_a { max_a = traj_a.len(); }
        if traj_b.len() > max_b { max_b = traj_b.len(); }
        if n == 2 {
            let spin = traj_a.len();
            let ks = reduce_word(&twa, n);
            n2 = format!("  n=2: spin-law {} recs ({} del) ; terminal-law {} recs ; both judged ⊤={}",
                spin, ks, traj_b.len(), judge_trace(&twb) == M_T);
        }
        if n == 106545994355809 {
            big = format!("  big {}: spin-law {} recs factor {:?} ; terminal-law {} recs factor {:?}",
                n, traj_a.len(), res_a, traj_b.len(), res_b);
        }
    }

    println!("=== terminal fixed point vs post-closure spin ===");
    println!("closed (judged ⊤)  spin-law {}   terminal-law {}", closed_a, closed_b);
    println!("factor disagreement across the two laws: {}", factor_mismatch);
    println!("traces the terminal law SHORTENS: {}   lengthens: {}", shortened, grew);
    println!("max trajectory length:  spin-law {}   terminal-law {}", max_a, max_b);
    println!("{}", n2);
    println!("{}", big);

    let pass = closed_a == 161 && closed_b == 161 && factor_mismatch == 0 && grew == 0 && max_b < max_a;
    println!("\nTERMINAL-FIXPOINT {}", if pass { "PASS" } else { "FAIL" });
}
