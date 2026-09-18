// cluster_factor_cli — Stage 22. Stage 17 found only 2 whole-corpus recurring spans. That is a
// property of a heterogeneous corpus, not evidence that carriers are rare. So cluster traces by
// SHAPE first (judgment/recognised/next sequence + applied-word lengths, repr bytes ignored), then
// run FACTOR INSIDE each class. A class carrier at (pos, len) is accepted only if replacing that
// shape-span by its COMPOSED record preserves the chosen relation in EVERY class member.
extern crate alloc;
use ::vox::router_object::RouterObject;
use ::vox::router_marks::{run_word, M_T, GStep};
use ::vox::trace_word::{decode_trace, encode_trace, judge_trace, encode_step};
use ::vox::trace_algebra::{replay_state, factor_of};
use ::vox::tape_delete::{tape_edit, fuse_records};

fn shape(r: &GStep) -> alloc::string::String {
    let mut s = alloc::string::String::new();
    s.push(r.judgment); s.push(r.recognised); s.push(r.next);
    s.push_str(&alloc::format!("{}|", r.applied_word.len()));
    s
}
fn trace_shape(t: &[GStep]) -> alloc::string::String {
    let mut s = alloc::string::String::new();
    for r in t { s.push_str(&shape(r)); }
    s
}
fn span_shape(t: &[GStep], pos: usize, len: usize) -> alloc::string::String {
    let mut s = alloc::string::String::new();
    for r in &t[pos..pos + len] { s.push_str(&shape(r)); }
    s
}
fn compose(t: &[GStep], pos: usize, len: usize) -> alloc::vec::Vec<char> {
    let mut c = t[pos].clone();
    for r in &t[pos + 1..pos + len] { c = fuse_records(&c, r); }
    encode_step(&c)
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

fn main() {
    let reference = RouterObject::initial();
    let word: Vec<char> = reference.encode().chars().collect();
    let mut corpus: Vec<u64> = Vec::new();
    for n in 2u64..=1000 { corpus.push(n); }
    corpus.extend_from_slice(&[1024, 2187, 3125, 2401, 1000003, 104729, 106545994355809, 10000019, 1000000007]);

    // closed traces with their shapes
    let mut traces: alloc::vec::Vec<(u64, alloc::vec::Vec<char>, alloc::vec::Vec<GStep>, alloc::string::String)> = alloc::vec::Vec::new();
    for &n in &corpus {
        let (_r, traj) = run_word(&word, n, 64);
        let tw = encode_trace(&traj);
        if judge_trace(&tw) != M_T { continue; }
        let t = decode_trace(&tw).unwrap();
        let sh = trace_shape(&t);
        traces.push((n, tw, t, sh));
    }
    let closed = traces.len();

    // cluster by shape
    let mut clusters: alloc::vec::Vec<(alloc::string::String, alloc::vec::Vec<usize>)> = alloc::vec::Vec::new();
    for (i, tr) in traces.iter().enumerate() {
        match clusters.iter_mut().find(|(k, _)| *k == tr.3) {
            Some((_, v)) => v.push(i),
            None => clusters.push((tr.3.clone(), alloc::vec![i])),
        }
    }
    let multi: alloc::vec::Vec<usize> = (0..clusters.len()).filter(|&c| clusters[c].1.len() >= 2).collect();
    let largest = clusters.iter().map(|c| c.1.len()).max().unwrap_or(0);
    println!("=== clusters by shape (repr bytes ignored) ===");
    for (_, v) in clusters.iter() {
        if v.len() >= 2 {
            let t = &traces[v[0]].2;
            let mut sk = alloc::string::String::new();
            for r in t.iter().take(6) { sk.push(r.judgment); sk.push(r.recognised); sk.push(r.next); }
            println!("  {} members, nrec={}, skeleton={}…", v.len(), t.len(), sk);
        }
    }

    // FACTOR inside each multi-member class: carriers keyed by span shape at (pos,len), grouped
    for rel in [M_T, '\u{22A5}'] {
        let relname = if rel == M_T { "≡s strict" } else { "≡c relaxed" };
        let mut class_carriers = 0usize;
        let mut accepted = 0usize;
        let mut example = alloc::string::String::new();
        for &ci in &multi {
            let members = &clusters[ci].1;
            // group occurrences by (pos, len, shape)
            let mut keys: alloc::vec::Vec<(usize, usize, alloc::string::String)> = alloc::vec::Vec::new();
            let mut occ: alloc::vec::Vec<alloc::vec::Vec<usize>> = alloc::vec::Vec::new(); // member idx
            for &mi in members {
                let t = &traces[mi].2;
                for len in 2..=3usize {
                    for pos in 0..t.len().saturating_sub(len - 1) {
                        let k = (pos, len, span_shape(t, pos, len));
                        match keys.iter().position(|x| *x == k) {
                            Some(p) => occ[p].push(mi),
                            None => { keys.push(k); occ.push(alloc::vec![mi]); }
                        }
                    }
                }
            }
            for p in 0..keys.len() {
                if occ[p].len() < 2 { continue; }
                class_carriers += 1;
                let (pos, len, _) = keys[p].clone();
                let mut ok = true;
                for &mi in &occ[p] {
                    let (n, ref tw, ref t, _) = traces[mi];
                    let kword = compose(t, pos, len);
                    match tape_edit(tw, pos, len, kword) {
                        Some(c) => if !admissible(tw, &c, n, rel) { ok = false; break; },
                        None => { ok = false; break; }
                    }
                }
                if ok {
                    accepted += 1;
                    if example.is_empty() {
                        example = alloc::format!("    class of {} (pos {}, len {}): carrier accepted under {}",
                            occ[p].len(), pos, len, relname);
                    }
                }
            }
        }
        println!("[{} · inside classes]  class carriers proposed {} · accepted {}", relname, class_carriers, accepted);
        if !example.is_empty() { println!("{}", example); }
    }

    println!("\n=== clustered FACTOR (structural, not literal, recurrence) ===");
    println!("closed traces: {}   distinct shapes: {}   multi-member classes: {}   largest class: {}",
        closed, clusters.len(), multi.len(), largest);
    println!("(a class carrier is a span-shape whose COMPOSED replacement is admissible in every class member)");
}
