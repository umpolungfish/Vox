// tape_algebra_words_cli — DELETE, CANCEL, FUSE, INLINE are ONE resident word under four
// data settings (groups × replacement). replay is the only semantics. This measures how far
// each setting reaches and, crucially, how many DELETE-fixed-points the WIDER set still
// reduces — the gap between local and global irreducibility.
//   ≡s strict : replay_state equal + judge ⊤ + factor equal
//   ≡c relaxed: judge ⊤ + closure & factor equal
extern crate alloc;
use ::vox::router_object::RouterObject;
use ::vox::router_marks::run_word;
use ::vox::router_marks::M_T;
use ::vox::trace_word::{decode_trace, encode_trace, judge_trace};
use ::vox::trace_algebra::{replay_state, factor_of};
use ::vox::tape_delete::{delete_word, fuse_word, cancel_word, inline_word};

#[derive(Clone, Copy, PartialEq)]
enum S { S, C } // strict / relaxed

fn ok(orig: &[char], cand: &[char], n: u64, rel: S) -> bool {
    let fo = replay_state(orig).as_ref().and_then(|r| factor_of(r, n));
    let fc = replay_state(cand).as_ref().and_then(|r| factor_of(r, n));
    if rel == S::S {
        let (ro, rc) = match (replay_state(orig), replay_state(cand)) { (Some(a), Some(b)) => (a, b), _ => return false };
        ro == rc && judge_trace(cand) == M_T && fo == fc
    } else {
        judge_trace(cand) == M_T && fo.is_some() && fo == fc
    }
}

/// reduce with an allowed transform set: bit0 delete, bit1 fuse, bit2 cancel(k<=3), bit3 inline(k<=3)
fn reduce(w: &[char], n: u64, rel: S, set: u8) -> (Vec<char>, usize) {
    let mut cur: Vec<char> = w.to_vec();
    let mut k = 0usize;
    'o: loop {
        let t = match decode_trace(&cur) { Some(t) => t, None => break };
        let len = t.len();
        if set & 1 != 0 { for i in 0..len { if let Some(c) = delete_word(&cur, i) { if ok(&cur, &c, n, rel) { cur = c; k += 1; continue 'o; } } } }
        if set & 2 != 0 { for i in 0..len.saturating_sub(1) { if let Some(c) = fuse_word(&cur, i) { if ok(&cur, &c, n, rel) { cur = c; k += 1; continue 'o; } } } }
        if set & 4 != 0 { for kk in 2..=3usize { for i in 0..len.saturating_sub(kk - 1) { if let Some(c) = cancel_word(&cur, i, kk) { if ok(&cur, &c, n, rel) { cur = c; k += 1; continue 'o; } } } } }
        if set & 8 != 0 { for kk in 2..=3usize { for i in 0..len.saturating_sub(kk - 1) { if let Some(c) = inline_word(&cur, i, kk) { if ok(&cur, &c, n, rel) { cur = c; k += 1; continue 'o; } } } } }
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

    let (mut closed, mut s_del, mut s_fuse, mut s_full, mut c_del, mut c_full) = (0, 0, 0, 0, 0, 0);
    let mut local_trap = 0usize;          // DELETE-strict fixed points still reduced by the wider set
    let mut trap_list: Vec<String> = Vec::new();
    let mut big = String::new(); let mut n2 = String::new();

    for &n in &corpus {
        let (_res, traj) = run_word(&word, n, 64);
        let tw = encode_trace(&traj);
        if judge_trace(&tw) != M_T { continue; }
        closed += 1;
        let (rd, _) = reduce(&tw, n, S::S, 1);
        let (rf, _) = reduce(&tw, n, S::S, 3);
        let (rfull, _) = reduce(&tw, n, S::S, 15);
        let (rcd, _) = reduce(&tw, n, S::C, 1);
        let (rcf, _) = reduce(&tw, n, S::C, 15);
        if recs(&rd) < recs(&tw) { s_del += 1; }
        if recs(&rf) < recs(&tw) { s_fuse += 1; }
        if recs(&rfull) < recs(&tw) { s_full += 1; }
        if recs(&rcd) < recs(&tw) { c_del += 1; }
        if recs(&rcf) < recs(&tw) { c_full += 1; }
        // local irreducibility trap: strict-delete fixed point, but the wider set still reduces
        if recs(&rd) == recs(&tw) && recs(&rfull) < recs(&tw) {
            local_trap += 1;
            if trap_list.len() < 6 { trap_list.push(format!("  n={}: {}=delete-fixed -> full {} recs", n, recs(&tw), recs(&rfull))); }
        }
        if n == 2 { n2 = format!("  n=2: {} recs -> strict-delete {} | strict-full {} | relaxed-full {}", recs(&tw), recs(&rd), recs(&rfull), recs(&rcf)); }
        if n == 106545994355809 {
            big = format!("  big {}: {} recs -> strict-delete {} | strict-fuse {} | strict-full {} | relaxed-delete {} | relaxed-full {}",
                n, recs(&tw), recs(&rd), recs(&rf), recs(&rfull), recs(&rcd), recs(&rcf));
        }
    }

    println!("=== one resident word, four data settings ===");
    println!("  DELETE = (groups 1, ∅)   CANCEL = (k>=2, ∅)   FUSE = (2, merge)   INLINE = (k>=2, compose)");
    println!("closed trajectories: {}", closed);
    println!("strict  quite-compressible:  delete {} · +fuse {} · +cancel+inline {}", s_del, s_fuse, s_full);
    println!("relaxed compressible:        delete {} · full {}", c_del, c_full);
    println!("{}", n2);
    println!("{}", big);
    println!("LOCAL-vs-GLOBAL: strict DELETE-fixed-points still reduced by the wider set: {}", local_trap);
    for l in &trap_list { println!("{}", l); }

    let pass = closed == 161 && s_del == 1 && c_del == 2 && local_trap >= 1;
    println!("\nTAPE-ALGEBRA-WORDS {}", if pass { "PASS" } else { "FAIL" });
}
