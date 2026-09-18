// confluence_cli — Stage 21. Stage 16 showed a "fixed point" depends on the rule set. Now: once
// several admissible rewrites exist, do different ORDERS converge to the same normal form?
// Two measurements, per relation (≡s strict, ≡c relaxed — never collapsed):
//   (1) ONE-STEP: from every admissible one-step edit, reduce to a normal form; do branches rejoin?
//   (2) ALL-ORDERINGS: full-tree DFS over every maximal reduction (bounded) — do ALL terminal
//       traces agree under the relation? (the strongest invariant asked for)
extern crate alloc;
use ::vox::router_object::RouterObject;
use ::vox::router_marks::{run_word, M_T, GStep};
use ::vox::trace_word::{decode_trace, encode_trace, judge_trace, encode_step};
use ::vox::trace_algebra::{replay_state, factor_of};
use ::vox::tape_delete::{tape_edit, fuse_records};
use ::vox::reducer_store::{reduce_resident, Rule, rule_delete, FILL_COMPOSE};

fn nf_equiv(f1: &[char], f2: &[char], n: u64, rel: char) -> bool {
    let fa = replay_state(f1).as_ref().and_then(|r| factor_of(r, n));
    let fb = replay_state(f2).as_ref().and_then(|r| factor_of(r, n));
    if rel == M_T {
        replay_state(f1) == replay_state(f2) && judge_trace(f2) == M_T && fa == fb
    } else {
        judge_trace(f1) == M_T && judge_trace(f2) == M_T && fa.is_some() && fa == fb
    }
}
fn admissible(orig: &[char], cand: &[char], n: u64, rel: char) -> bool {
    let fo = replay_state(orig).as_ref().and_then(|r| factor_of(r, n));
    let fc = replay_state(cand).as_ref().and_then(|r| factor_of(r, n));
    if rel == M_T {
        let (ro, rc) = match (replay_state(orig), replay_state(cand)) { (Some(a), Some(b)) => (a, b), _ => return false };
        ro == rc && judge_trace(cand) == M_T && fo == fc
    } else {
        judge_trace(cand) == M_T && fo.is_some() && fo == fc
    }
}
fn one_step(t: &[char], n: u64, rel: char, rules: &[Rule]) -> alloc::vec::Vec<alloc::vec::Vec<char>> {
    let dec = match decode_trace(t) { Some(d) => d, None => return alloc::vec::Vec::new() };
    let mut out = alloc::vec::Vec::new();
    for r in rules {
        for i in 0..dec.len() {
            let repl: alloc::vec::Vec<char> = if r.fill == FILL_COMPOSE {
                if i + r.groups > dec.len() { continue; }
                let span: alloc::vec::Vec<GStep> = dec[i..i + r.groups].to_vec();
                let mut comp = span[0].clone();
                for x in &span[1..] { comp = fuse_records(&comp, x); }
                encode_step(&comp)
            } else { alloc::vec::Vec::new() };
            if let Some(c) = tape_edit(t, i, r.groups, repl) { if admissible(t, &c, n, rel) { out.push(c); } }
        }
    }
    out
}
/// full-tree DFS over every maximal reduction; collect terminal traces. Bounded by `budget`.
fn all_nfs(t: &[char], n: u64, rel: char, rules: &[Rule],
           out: &mut alloc::vec::Vec<alloc::vec::Vec<char>>, budget: &mut i64) {
    if *budget <= 0 { return; }
    *budget -= 1;
    let succs = one_step(t, n, rel, rules);
    if succs.is_empty() { out.push(t.to_vec()); return; }
    for c in succs {
        if out.len() > 4096 { return; }
        all_nfs(&c, n, rel, rules, out, budget);
    }
}

fn main() {
    let reference = RouterObject::initial();
    let word: Vec<char> = reference.encode().chars().collect();
    let mut corpus: Vec<u64> = Vec::new();
    for n in 2u64..=1000 { corpus.push(n); }
    corpus.extend_from_slice(&[1024, 2187, 3125, 2401, 1000003, 104729, 106545994355809, 10000019, 1000000007]);

    let mut closed = 0usize;
    let mut multi = 0usize; // traces with >1 record
    for &n in &corpus {
        let (_r, traj) = run_word(&word, n, 64);
        let tw = encode_trace(&traj);
        if judge_trace(&tw) != M_T { continue; }
        closed += 1;
        if decode_trace(&tw).map(|t| t.len()).unwrap_or(0) > 1 { multi += 1; }
    }
    println!("=== confluence: order-independence of the normal form ===");
    println!("closed traces: {}   (multi-record: {})", closed, multi);

    for rel in [M_T, '\u{22A5}'] {
        let relname = if rel == M_T { "≡s strict" } else { "≡c relaxed" };
        let rules = alloc::vec![rule_delete()];
        let (mut branch, mut conf, mut noncon) = (0usize, 0usize, 0usize);
        let (mut ft_checked, mut ft_confluent, mut ft_capped, mut ft_bad) = (0usize, 0usize, 0usize, 0usize);
        for &n in &corpus {
            let (_r, traj) = run_word(&word, n, 64);
            let tw = encode_trace(&traj);
            if judge_trace(&tw) != M_T { continue; }
            let succs = one_step(&tw, n, rel, &rules);
            if succs.len() >= 2 {
                branch += 1;
                let mut nfs: alloc::vec::Vec<alloc::vec::Vec<char>> = alloc::vec::Vec::new();
                nfs.push(reduce_resident(&tw, n, rel, &rules).0);
                for c in &succs { nfs.push(reduce_resident(c, n, rel, &rules).0); }
                let mut ok = true;
                'ck: for a in 0..nfs.len() { for b in a + 1..nfs.len() {
                    if !nf_equiv(&nfs[a], &nfs[b], n, rel) { ok = false; break 'ck; } } }
                if ok { conf += 1; } else { noncon += 1; }
                // full-tree (all orderings) on traces small enough to enumerate
                let nrec = decode_trace(&tw).map(|t| t.len()).unwrap_or(0);
                if nrec <= 12 && succs.len() <= 16 {
                    let mut out: alloc::vec::Vec<alloc::vec::Vec<char>> = alloc::vec::Vec::new();
                    let mut budget = 200_000i64;
                    all_nfs(&tw, n, rel, &rules, &mut out, &mut budget);
                    if budget <= 0 { ft_capped += 1; }
                    else {
                        ft_checked += 1;
                        let mut allok = true;
                        'f: for a in 0..out.len() { for b in a + 1..out.len() {
                            if !nf_equiv(&out[a], &out[b], n, rel) { allok = false; break 'f; } } }
                        if allok { ft_confluent += 1; } else { ft_bad += 1; }
                    }
                }
            }
        }
        println!("  [{} · delete rule set]", relname);
        println!("    one-step rejoining:  branch points {} · confluent {} · NOT confluent {}", branch, conf, noncon);
        println!("    all-orderings (full tree): checked {} · confluent {} · NOT {} · capped {}",
                 ft_checked, ft_confluent, ft_bad, ft_capped);
    }
    println!("\n(confluence on this corpus = every branch point's normal form is order-independent under the relation)");
}
