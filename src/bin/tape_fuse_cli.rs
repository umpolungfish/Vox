// tape_fuse_cli — FUSE is resident too. The same word, a two-group span, the merged record
// spliced in. DELETE is the empty-replacement case (still byte-identical). Replay is the
// semantics: a fusion is admissible iff it preserves the CHOSEN equivalence.
//   ≡s (strict):  replay_state equal + judge ⊤ + factor equal
//   ≡c (relaxed): judge ⊤ + closure & factor equal
extern crate alloc;
use ::vox::router_object::RouterObject;
use ::vox::router_marks::run_word;
use ::vox::router_marks::M_T;
use ::vox::trace_word::{decode_trace, encode_trace, judge_trace};
use ::vox::trace_algebra::{delete_record, replay_state, factor_of};
use ::vox::tape_delete::{delete_word, fuse_word, EDIT_WORD};

fn equiv_s(orig: &[char], cand: &[char], n: u64) -> bool {
    let (ro, rc) = match (replay_state(orig), replay_state(cand)) { (Some(a), Some(b)) => (a, b), _ => return false };
    ro == rc && judge_trace(cand) == M_T && factor_of(&ro, n) == factor_of(&rc, n)
}
fn equiv_c(orig: &[char], cand: &[char], n: u64) -> bool {
    let fo = replay_state(orig).as_ref().and_then(|r| factor_of(r, n));
    let fc = replay_state(cand).as_ref().and_then(|r| factor_of(r, n));
    judge_trace(cand) == M_T && fo.is_some() && fo == fc
}

fn reduce(w: &[char], n: u64, use_fuse: bool, strict: bool) -> (Vec<char>, usize) {
    let test = if strict { equiv_s } else { equiv_c };
    let mut cur: Vec<char> = w.to_vec();
    let mut k = 0usize;
    'o: loop {
        let t = match decode_trace(&cur) { Some(t) => t, None => break };
        for i in 0..t.len() {
            if let Some(cand) = delete_word(&cur, i) {
                if test(&cur, &cand, n) { cur = cand; k += 1; continue 'o; }
            }
        }
        if use_fuse {
            for i in 0..t.len().saturating_sub(1) {
                if let Some(cand) = fuse_word(&cur, i) {
                    if test(&cur, &cand, n) { cur = cand; k += 1; continue 'o; }
                }
            }
        }
        break;
    }
    (cur, k)
}

fn recs(w: &[char]) -> usize { decode_trace(w).map(|t| t.len()).unwrap_or(0) }

fn main() {
    let reference = RouterObject::initial();
    let word: Vec<char> = reference.encode().chars().collect();

    let mut corpus: Vec<u64> = Vec::new();
    for n in 2u64..=200 { corpus.push(n); }
    corpus.extend_from_slice(&[221, 247, 323, 391, 437, 529, 1000003, 104729, 106545994355809, 10000019]);

    // ---- regression: DELETE byte-equality vs oracle ----
    let mut eq_checks = 0usize; let mut eq_fail = 0usize;
    let mut fuse_wellformed = 0usize; let mut fuse_bad = 0usize;

    let mut closed_ct = 0usize;
    let mut s_do = 0usize;  // strict compressible, delete only
    let mut s_df = 0usize;  // strict compressible, delete + fuse
    let mut c_do = 0usize;  // relaxed compressible, delete only
    let mut c_df = 0usize;  // relaxed compressible, delete + fuse
    let mut fuse_extra = Vec::new();   // traces where FUSE strictly beats DELETE
    let mut big = String::new();

    for &n in &corpus {
        let (_res, traj) = run_word(&word, n, 64);
        let tw = encode_trace(&traj);
        let l = match decode_trace(&tw) { Some(t) => t.len(), None => continue };
        for i in 0..l {
            eq_checks += 1;
            if delete_word(&tw, i) != delete_record(&tw, i) { eq_fail += 1; }
        }
        for i in 0..l.saturating_sub(1) {
            match fuse_word(&tw, i) {
                Some(c) => { if decode_trace(&c).is_some() { fuse_wellformed += 1; } else { fuse_bad += 1; } }
                None => { fuse_bad += 1; }
            }
        }
        if judge_trace(&tw) != M_T { continue; }
        closed_ct += 1;

        let (rs_do, _) = reduce(&tw, n, false, true);
        let (rs_df, _) = reduce(&tw, n, true, true);
        let (rc_do, _) = reduce(&tw, n, false, false);
        let (rc_df, _) = reduce(&tw, n, true, false);
        if recs(&rs_do) < recs(&tw) { s_do += 1; }
        if recs(&rs_df) < recs(&tw) { s_df += 1; }
        if recs(&rc_do) < recs(&tw) { c_do += 1; }
        if recs(&rc_df) < recs(&tw) { c_df += 1; }
        if recs(&rs_df) < recs(&rs_do) && fuse_extra.len() < 8 {
            fuse_extra.push(format!("  n={}: strict delete-only {} recs -> delete+fuse {} recs", n, recs(&rs_do), recs(&rs_df)));
        }
        if n == 106545994355809 {
            big = format!("  big {}: {} recs | strict do {} df {} | relaxed do {} df {}",
                n, recs(&tw), recs(&rs_do), recs(&rs_df), recs(&rc_do), recs(&rc_df));
        }
    }

    println!("=== resident FUSE: same word, two-group span, merged record spliced ===");
    println!("EDIT_WORD = \"{}\"  ({} marks)   DELETE = 1 group/empty   FUSE = 2 groups/merged",
        EDIT_WORD, EDIT_WORD.chars().count());
    println!("DELETE byte-equality vs oracle (regression): {}/{}", eq_checks - eq_fail, eq_checks);
    println!("FUSE candidates well-formed: {}/{}", fuse_wellformed, fuse_wellformed + fuse_bad);
    println!("closed trajectories: {}", closed_ct);
    println!("strict compressible   delete-only: {}   delete+fuse: {}", s_do, s_df);
    println!("relaxed compressible  delete-only: {}   delete+fuse: {}", c_do, c_df);
    println!("{}", big);
    println!("traces where FUSE strictly beats DELETE:");
    if fuse_extra.is_empty() { println!("  (none)"); } else { for l in &fuse_extra { println!("{}", l); } }

    let pass = eq_fail == 0 && fuse_bad == 0 && closed_ct == 161 && s_do == 1 && c_do == 2;
    println!("\nTAPE-FUSE {}", if pass { "PASS" } else { "FAIL" });
}
