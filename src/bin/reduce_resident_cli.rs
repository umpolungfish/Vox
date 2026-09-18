// reduce_resident_cli — Stage 20. The SCHEDULE is resident: the machine's cursor/rule/replacement
// live in marks and REDUCE_WORD = ∈∋⊤≻⊡ walks them. A host reference reducer does the same work in
// a plain Rust loop; we require byte equality and identical transform counts across rule sets and
// across BOTH relations (≡s strict, ≡c relaxed), and that the result is a fixed point of the rule set.
extern crate alloc;
use ::vox::router_object::RouterObject;
use ::vox::router_marks::{run_word, M_T, GStep};
use ::vox::trace_word::{decode_trace, encode_trace, judge_trace, encode_step};
use ::vox::trace_algebra::{reduce, replay_state, factor_of};
use ::vox::tape_delete::{tape_edit, fuse_records};
use ::vox::reducer_store::{reduce_resident, Rule, FILL_COMPOSE,
                            rule_delete, rule_fuse, rule_cancel, rule_inline, REDUCE_WORD};

fn equiv(orig: &[char], cand: &[char], n: u64, rel: char) -> bool {
    let fo = replay_state(orig).as_ref().and_then(|r| factor_of(r, n));
    let fc = replay_state(cand).as_ref().and_then(|r| factor_of(r, n));
    if rel == M_T {
        let (ro, rc) = match (replay_state(orig), replay_state(cand)) { (Some(a), Some(b)) => (a, b), _ => return false };
        ro == rc && judge_trace(cand) == M_T && fo == fc
    } else {
        judge_trace(cand) == M_T && fo.is_some() && fo == fc
    }
}

/// host reference: same schedule, expressed as a plain Rust loop (the thing being replaced)
fn reduce_ref(w: &[char], n: u64, rel: char, rules: &[Rule]) -> (Vec<char>, usize) {
    let mut cur: Vec<char> = w.to_vec();
    let mut k = 0usize;
    'o: loop {
        let t = match decode_trace(&cur) { Some(t) => t, None => break };
        let ns = t.len();
        for r in rules {
            if r.groups == 0 { continue; }
            for i in 0..ns {
                let repl: Vec<char> = if r.fill == FILL_COMPOSE {
                    if i + r.groups > ns { continue; }
                    let span: Vec<GStep> = t[i..i + r.groups].to_vec();
                    let mut comp = span[0].clone();
                    for x in &span[1..] { comp = fuse_records(&comp, x); }
                    encode_step(&comp)
                } else { Vec::new() };
                if let Some(cand) = tape_edit(&cur, i, r.groups, repl) {
                    if equiv(&cur, &cand, n, rel) { cur = cand; k += 1; continue 'o; }
                }
            }
        }
        break;
    }
    (cur, k)
}

/// is the result a fixed point of the rule set (no rule x span is admissible)?
fn is_fixed(w: &[char], n: u64, rel: char, rules: &[Rule]) -> bool {
    let t = match decode_trace(w) { Some(t) => t, None => return false };
    for r in rules {
        for i in 0..t.len() {
            let repl: Vec<char> = if r.fill == FILL_COMPOSE {
                if i + r.groups > t.len() { continue; }
                let span: Vec<GStep> = t[i..i + r.groups].to_vec();
                let mut comp = span[0].clone();
                for x in &span[1..] { comp = fuse_records(&comp, x); }
                encode_step(&comp)
            } else { Vec::new() };
            if let Some(c) = tape_edit(w, i, r.groups, repl) { if equiv(w, &c, n, rel) { return false; } }
        }
    }
    true
}

fn recs(w: &[char]) -> usize { decode_trace(w).map(|t| t.len()).unwrap_or(0) }

fn main() {
    let reference = RouterObject::initial();
    let word: Vec<char> = reference.encode().chars().collect();
    let mut corpus: Vec<u64> = Vec::new();
    for n in 2u64..=200 { corpus.push(n); }
    corpus.extend_from_slice(&[221, 247, 323, 391, 437, 529, 1000003, 104729, 106545994355809, 10000019]);

    let set_del = alloc::vec![rule_delete()];
    let set_df  = alloc::vec![rule_delete(), rule_fuse()];
    let set_all = alloc::vec![rule_delete(), rule_fuse(), rule_cancel(3), rule_inline(3)];
    let sets: alloc::vec::Vec<(&str, &alloc::vec::Vec<Rule>)> =
        alloc::vec![("delete", &set_del), ("delete+fuse", &set_df), ("all", &set_all)];

    let mut closed = 0usize;
    let mut mism = 0usize;
    let mut fixed_ok = 0usize;
    let mut fixed_tot = 0usize;
    let mut oracle_mism = 0usize;
    let mut s_del = 0usize; let mut s_df = 0usize; let mut s_all = 0usize;
    let mut c_del = 0usize; let mut c_all = 0usize;
    let mut n2 = String::new(); let mut big = String::new();

    for &n in &corpus {
        let (_res, traj) = run_word(&word, n, 64);
        let tw = encode_trace(&traj);
        if judge_trace(&tw) != M_T { continue; }
        closed += 1;

        for rel in [M_T, '\u{22A5}'] {
            for (nm, rules) in &sets {
                let (r_res, k_res) = reduce_resident(&tw, n, rel, rules);
                let (r_ref, k_ref) = reduce_ref(&tw, n, rel, rules);
                if r_res != r_ref || k_res != k_ref { mism += 1; }
                fixed_tot += 1;
                if is_fixed(&r_res, n, rel, rules) { fixed_ok += 1; }
                if rel == M_T {
                    if *nm == "delete" { if recs(&r_res) < recs(&tw) { s_del += 1; } }
                    if *nm == "delete+fuse" { if recs(&r_res) < recs(&tw) { s_df += 1; } }
                    if *nm == "all" { if recs(&r_res) < recs(&tw) { s_all += 1; } }
                } else {
                    if *nm == "delete" { if recs(&r_res) < recs(&tw) { c_del += 1; } }
                    if *nm == "all" { if recs(&r_res) < recs(&tw) { c_all += 1; } }
                }
                if n == 2 && rel == M_T && *nm == "delete" { n2 = alloc::format!("  n=2: resident {} recs -> {} ({} transforms)", recs(&tw), recs(&r_res), k_res); }
                if n == 106545994355809 && rel == M_T && *nm == "all" { big = alloc::format!("  big: resident strict-all {} recs -> {}", recs(&tw), recs(&r_res)); }
            }
        }
        // cross-check the resident delete-strict against the canonical trace_algebra::reduce
        let (r_canon, _) = reduce(&tw, n);
        let (r_res, _) = reduce_resident(&tw, n, M_T, &set_del);
        if r_canon != r_res { oracle_mism += 1; }
    }

    println!("=== resident reducer: REDUCE_WORD = \"{}\" ({} marks) ===", REDUCE_WORD, REDUCE_WORD.chars().count());
    println!("machine state in marks: cursor, rule, replacement; the word is the SCHEDULE");
    println!("closed traces: {}", closed);
    println!("resident vs host-reference disagreements (bytes or count): {}", mism);
    println!("resident delete-strict vs trace_algebra::reduce disagreements: {}", oracle_mism);
    println!("fixed point of the rule set after reduction: {}/{}", fixed_ok, fixed_tot);
    println!("strict compressible:  delete {} · delete+fuse {} · all {}", s_del, s_df, s_all);
    println!("relaxed compressible: delete {} · all {}", c_del, c_all);
    println!("{}", n2);
    println!("{}", big);

    let pass = closed == 161 && mism == 0 && oracle_mism == 0 && fixed_ok == fixed_tot
        && s_del == 1 && s_df == 2 && c_del == 2;
    println!("\nREDUCE-RESIDENT {}", if pass { "PASS" } else { "FAIL" });
}
