// tower_words_cli — dump the tower walk words (Router, Trace, History) as glyphs.
extern crate alloc;
use ::vox::router_object::RouterObject;
use ::vox::router_marks::{RouterG, run_word, GStep, M_T, M_FIX};
use ::vox::trace_word::{decode_trace, encode_trace, judge_trace};
use ::vox::trace_algebra::{replay_state, factor_of};
use ::vox::tape_delete::{delete_word, EDIT_WORD};

fn equiv_s(orig: &[char], cand: &[char], n: u64) -> bool {
    let (ro, rc) = match (replay_state(orig), replay_state(cand)) { (Some(a), Some(b)) => (a, b), _ => return false };
    ro == rc && judge_trace(cand) == M_T && factor_of(&ro, n) == factor_of(&rc, n)
}

fn reduce_collect(w: &[char], n: u64) -> (Vec<char>, alloc::vec::Vec<GStep>) {
    let mut cur: Vec<char> = w.to_vec();
    let mut h: alloc::vec::Vec<GStep> = alloc::vec::Vec::new();
    'o: loop {
        let t = match decode_trace(&cur) { Some(t) => t, None => break };
        for i in 0..t.len() {
            if let Some(c) = delete_word(&cur, i) {
                if equiv_s(&cur, &c, n) {
                    h.push(GStep { repr: t[i].repr, judgment: M_T, recognised: M_T, next: M_FIX,
                                   applied_word: EDIT_WORD.chars().collect() });
                    cur = c; continue 'o;
                }
            }
        }
        break;
    }
    (cur, h)
}

fn main() {
    let mut r0 = RouterObject::initial();
    r0.canonicalize();
    let w0 = r0.encode();
    println!("ROUTER\t{}\t{}", w0.chars().count(), w0);

    let rg = RouterG::from_enum(&RouterObject::initial());
    let word: Vec<char> = rg.encode().chars().collect();

    let (_r, traj) = run_word(&word, 106545994355809, 64);
    let tw: Vec<char> = encode_trace(&traj);
    let tws: String = tw.iter().collect();
    println!("TRACE\t{}\t{}", tw.len(), tws);

    let (_r2, traj2) = run_word(&word, 2, 64);
    let tw2 = encode_trace(&traj2);
    let (_nf, h) = reduce_collect(&tw2, 2);
    let hw: Vec<char> = encode_trace(&h);
    let hws: String = hw.iter().collect();
    println!("HISTORY\t{}\t{}", hw.len(), hws);
}
