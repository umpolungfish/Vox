// transform_verdict_cli — the re-entry: JUDGE THE TRANSFORMATION WORD.
//   trace -> transform-word -> candidate -> judge -> replay -> verdict on the transform-word
// The history of rewrites is recorded as a TRACE whose records' applied words ARE the resident
// edit word. That meta-trace is read by the SAME judge_trace as any object trace. So the system
// judges the transformations that judge and rewrite its histories.
extern crate alloc;
use ::vox::router_object::RouterObject;
use ::vox::router_marks::{run_word, GStep, M_T, M_FIX};
use ::vox::trace_word::{decode_trace, encode_trace, judge_trace};
use ::vox::trace_algebra::{replay_state, factor_of};
use ::vox::tape_delete::{delete_word, EDIT_WORD};

fn equiv_s(orig: &[char], cand: &[char], n: u64) -> bool {
    let (ro, rc) = match (replay_state(orig), replay_state(cand)) { (Some(a), Some(b)) => (a, b), _ => return false };
    ro == rc && judge_trace(cand) == M_T && factor_of(&ro, n) == factor_of(&rc, n)
}

/// Reduce strictly, recording each accepted transform as an edit record (whose applied word
/// is the resident EDIT word — the program that rewrote the history).
fn reduce_with_edits(w: &[char], n: u64) -> (Vec<char>, Vec<GStep>) {
    let mut cur: Vec<char> = w.to_vec();
    let mut edits: Vec<GStep> = Vec::new();
    'o: loop {
        let t = match decode_trace(&cur) { Some(t) => t, None => break };
        for i in 0..t.len() {
            if let Some(cand) = delete_word(&cur, i) {
                if equiv_s(&cur, &cand, n) {
                    edits.push(GStep { repr: t[i].repr, judgment: M_T, recognised: M_T,
                                       next: M_FIX, applied_word: EDIT_WORD.chars().collect() });
                    cur = cand; continue 'o;
                }
            }
        }
        break;
    }
    (cur, edits)
}

fn main() {
    let reference = RouterObject::initial();
    let word: Vec<char> = reference.encode().chars().collect();
    let mut corpus: Vec<u64> = Vec::new();
    for n in 2u64..=200 { corpus.push(n); }
    corpus.extend_from_slice(&[221, 247, 323, 391, 437, 529, 1000003, 104729, 106545994355809, 10000019]);

    let mut closed = 0usize;
    let mut total_edits = 0usize;
    let mut meta_closed = 0usize;   // meta-traces that judge ⊤ (a closed transformation trajectory)
    let mut identity = 0usize;      // traces needing no transform (meta-trace judged ⊙)
    let mut meta_words: Vec<alloc::vec::Vec<char>> = Vec::new();
    let mut n2 = String::new(); let mut big = String::new();

    for &n in &corpus {
        let (_res, traj) = run_word(&word, n, 64);
        let tw = encode_trace(&traj);
        if judge_trace(&tw) != M_T { continue; }
        closed += 1;

        let (red, edits) = reduce_with_edits(&tw, n);
        total_edits += edits.len();

        // the transformation history, as a trace of the same kind
        let meta_word = encode_trace(&edits);
        let verdict = judge_trace(&meta_word);
        meta_words.push(meta_word.clone());
        match verdict {
            v if v == M_T => meta_closed += 1,
            v if v == '\u{2299}' => identity += 1,
            _ => {}
        }
        if n == 2 {
            n2 = format!("  n=2: {} edits -> meta-trace {} recs, verdict {} ; reduced {} -> {} recs",
                edits.len(), edits.len(), verdict, decode_trace(&tw).unwrap().len(), decode_trace(&red).unwrap().len());
        }
        if n == 106545994355809 {
            big = format!("  big {}: {} edits -> meta-trace verdict {} ; reduced {} -> {} recs",
                n, edits.len(), verdict, decode_trace(&tw).unwrap().len(), decode_trace(&red).unwrap().len());
        }
    }

    // the verdict on the whole TRANSFORMATION FAMILY — judge_router_trace over all meta-traces
    let mut any_unmatched = false; let mut n_closed = 0usize;
    for mw in &meta_words {
        let t = decode_trace(mw).unwrap();
        if t.iter().any(|s| s.recognised == '\u{2299}') { any_unmatched = true; }
        if t.last().map(|s| s.judgment == M_T).unwrap_or(false) { n_closed += 1; }
    }
    let family = if any_unmatched { '\u{2299}' } else if n_closed == meta_words.len() { M_T } else if n_closed == 0 { '\u{2299}' } else { '\u{229E}' };

    println!("=== judge the transformation word (re-entry) ===");
    println!("closed object-traces: {}", closed);
    println!("total accepted transforms (resident EDIT word applies): {}", total_edits);
    println!("meta-traces judged ⊤ (closed transformation trajectory): {}", meta_closed);
    println!("meta-traces judged ⊙ (no transform needed — identity): {}", identity);
    println!("{}", n2);
    println!("{}", big);
    println!("verdict on the TRANSFORMATION FAMILY: {}", family);

    let pass = closed == 161 && meta_closed + identity == 161 && total_edits >= 63;
    println!("\nTRANSFORM-VERDICT {}", if pass { "PASS" } else { "FAIL" });
}
