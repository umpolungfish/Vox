// tape_factor_cli — FACTOR / cross-input carriers. A carrier K for a span S is accepted ONLY if
// replacing S by K preserves the CHOSEN equivalence at EVERY occurrence of S. First we print the
// record skeletons so we can see what recurs; then we sweep spans of length 2 and 3.
//   state-preserving   carriers under ≡s  (full replay state)
//   closure-preserving carriers under ≡c  (closure + reconstructed factor)
extern crate alloc;
use ::vox::router_object::RouterObject;
use ::vox::router_marks::run_word;
use ::vox::router_marks::{M_T, GStep};
use ::vox::trace_word::{decode_trace, encode_trace, judge_trace, encode_step};
use ::vox::trace_algebra::{replay_state, factor_of};
use ::vox::tape_delete::{fuse_records, tape_edit};

fn key(recs: &[GStep]) -> String {
    let mut v = alloc::vec::Vec::new();
    for r in recs { v.extend_from_slice(&encode_step(r)); }
    v.into_iter().collect()
}
fn compose(recs: &[GStep]) -> GStep {
    let mut c = recs[0].clone();
    for r in &recs[1..] { c = fuse_records(&c, r); }
    c
}
fn equiv(orig: &[char], cand: &[char], n: u64, strict: bool) -> bool {
    let fo = replay_state(orig).as_ref().and_then(|r| factor_of(r, n));
    let fc = replay_state(cand).as_ref().and_then(|r| factor_of(r, n));
    if strict {
        let (ro, rc) = match (replay_state(orig), replay_state(cand)) { (Some(a), Some(b)) => (a, b), _ => return false };
        ro == rc && judge_trace(cand) == M_T && fo == fc
    } else {
        judge_trace(cand) == M_T && fo.is_some() && fo == fc
    }
}
fn skel(t: &[GStep]) -> String {
    let mut s = String::new();
    for g in t.iter().take(10) {
        s.push(g.judgment); s.push(g.recognised); s.push(g.next);
        s.push_str(&alloc::format!("({}) ", g.applied_word.len()));
    }
    s
}

fn main() {
    let reference = RouterObject::initial();
    let word: Vec<char> = reference.encode().chars().collect();
    let mut corpus: Vec<u64> = Vec::new();
    for n in 2u64..=200 { corpus.push(n); }
    corpus.extend_from_slice(&[221, 247, 323, 391, 437, 529, 1000003, 104729, 106545994355809, 10000019]);

    let mut traces: Vec<(u64, Vec<char>, Vec<GStep>)> = Vec::new();
    for &n in &corpus {
        let (_res, traj) = run_word(&word, n, 64);
        let tw = encode_trace(&traj);
        if judge_trace(&tw) != M_T { continue; }
        let t = decode_trace(&tw).unwrap();
        traces.push((n, tw, t));
    }

    println!("=== record skeletons (judgment recognised next (applied_len)) ===");
    for &nn in &[2u64, 3, 4, 5, 6, 7, 8, 9, 10, 12, 15, 106545994355809] {
        if let Some((_, _, t)) = traces.iter().find(|(n, _, _)| *n == nn) {
            println!("  n={:<16} nrec={:<3} {}", nn, t.len(), skel(t));
        }
    }

    // sweep spans of length 2..=3; carrier K = composed record; accept iff ALL occurrences preserve
    let mut total_occ = 0usize;
    let mut multi = 0usize;
    let mut acc_s = 0usize;
    let mut acc_c = 0usize;
    let mut c_only = 0usize;
    let mut examples: Vec<String> = Vec::new();
    for len in 2..=3usize {
        let mut keys: Vec<String> = Vec::new();
        let mut occ: Vec<Vec<(usize, usize, Vec<char>)>> = Vec::new();
        for (ti, (_, _, t)) in traces.iter().enumerate() {
            for i in 0..t.len().saturating_sub(len - 1) {
                let span: Vec<GStep> = t[i..i + len].to_vec();
                let k = key(&span);
                let kw: Vec<char> = encode_step(&compose(&span)).into_iter().collect();
                match keys.iter().position(|x| *x == k) {
                    Some(p) => occ[p].push((ti, i, kw)),
                    None => { keys.push(k); occ.push(alloc::vec![(ti, i, kw)]); }
                }
            }
        }
        for p in 0..keys.len() {
            total_occ += occ[p].len();
            if occ[p].len() < 2 { continue; }
            multi += 1;
            let mut ok_s = true; let mut ok_c = true;
            for &(ti, pos, ref kw) in &occ[p] {
                let (n, ref tw, _) = traces[ti];
                let cand = match tape_edit(tw, pos, len, kw.clone()) { Some(c) => c, None => { ok_s = false; ok_c = false; break; } };
                if decode_trace(&cand).is_none() { ok_s = false; ok_c = false; break; }
                if !equiv(tw, &cand, n, true) { ok_s = false; }
                if !equiv(tw, &cand, n, false) { ok_c = false; }
                if !ok_s && !ok_c { break; }
            }
            if ok_s { acc_s += 1; }
            if ok_c { acc_c += 1; }
            if ok_c && !ok_s {
                c_only += 1;
                if examples.len() < 6 {
                    let (ti, pos, _) = occ[p][0];
                    examples.push(format!("  len {} span, {} occurrences (e.g. n={} @{}): accepted ≡c, rejected ≡s", len, occ[p].len(), traces[ti].0, pos));
                }
            }
        }
    }

    println!("\n=== cross-input carriers (FACTOR) ===");
    println!("closed traces: {}", traces.len());
    println!("total span occurrences (len 2..3): {}", total_occ);
    println!("spans recurring (>=2 occurrences): {}", multi);
    println!("accepted as state-preserving   carriers (≡s): {}", acc_s);
    println!("accepted as closure-preserving carriers (≡c): {}", acc_c);
    println!("closure-only carriers (≡c-but-not-≡s): {}", c_only);
    for l in &examples { println!("{}", l); }

    let pass = traces.len() == 161 && acc_s <= acc_c;
    println!("\nTAPE-FACTOR {}", if pass { "PASS" } else { "FAIL" });
}
