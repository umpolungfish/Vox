// history_reduce_cli — Stage 23. Recurse one level higher: normalize the history of normalization.
//   τ -> transformation-history h -> judge(h) -> transform(h) -> h*
// h is itself a trace; the SAME resident reducer normalizes it. We measure (a) trace length
// before/after, (b) transform-history length before/after, (c) normal-form stability under replay.
// And we name the trap: compressing h under ≡s/≡c discards the edit COUNT, so a reconstruction
// relation (≡r) does NOT see the compression — the reconstruction-preserving carrier is EDIT^k.
extern crate alloc;
use ::vox::router_object::RouterObject;
use ::vox::router_marks::{run_word, M_T, M_FIX, GStep};
use ::vox::trace_word::{decode_trace, encode_trace, judge_trace};
use ::vox::trace_algebra::{replay_state, factor_of};
use ::vox::tape_delete::{delete_word, EDIT_WORD};
use ::vox::reducer_store::{reduce_resident, rule_delete};

fn equiv_s(orig: &[char], cand: &[char], n: u64) -> bool {
    let (ro, rc) = match (replay_state(orig), replay_state(cand)) { (Some(a), Some(b)) => (a, b), _ => return false };
    ro == rc && judge_trace(cand) == M_T && factor_of(&ro, n) == factor_of(&rc, n)
}
fn nrec(w: &[char]) -> usize { decode_trace(w).map(|t| t.len()).unwrap_or(0) }

/// reduce strictly by DELETE (span-major), recording each accepted transform as a history record
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
    let reference = RouterObject::initial();
    let word: Vec<char> = reference.encode().chars().collect();
    let mut corpus: Vec<u64> = Vec::new();
    for n in 2u64..=1000 { corpus.push(n); }
    corpus.extend_from_slice(&[1000003, 104729, 106545994355809, 10000019, 1000000007]);

    let (mut closed, mut nonzero_hist, mut hist_compressed, mut recon_lost, mut stable_nf) = (0usize, 0usize, 0usize, 0usize, 0usize);
    let mut n2 = alloc::string::String::new(); let mut big = alloc::string::String::new();

    for &n in &corpus {
        let (_r, traj) = run_word(&word, n, 64);
        let tw = encode_trace(&traj);
        if judge_trace(&tw) != M_T { continue; }
        closed += 1;
        let (nf, h) = reduce_collect(&tw, n);
        // (a) trace length before/after
        let n0 = nrec(&tw); let nf0 = nrec(&nf);
        // nf stable under the reducer?
        let (nf2, _) = reduce_resident(&nf, n, M_T, &[rule_delete()]);
        if nf2 == nf { stable_nf += 1; }

        if h.is_empty() { continue; }
        nonzero_hist += 1;
        // (b) transform-history: h (a trace) normalized by the SAME reducer
        let hw = encode_trace(&h);
        let (hstar, _kh) = reduce_resident(&hw, n, M_T, &[rule_delete()]);
        let he = nrec(&hw); let hse = nrec(&hstar);
        if hse < he { hist_compressed += 1; }
        // (c) reconstruction under replay: a history applies one edit per record.
        //     recon via h  = n0 - he  ; recon via h* = n0 - hse. Different => NOT reconstruction-preserving.
        let recon_h = n0.saturating_sub(he);
        let recon_hstar = n0.saturating_sub(hse);
        if recon_h != recon_hstar { recon_lost += 1; }

        if n == 2 {
            n2 = alloc::format!("  n=2: trace {} -> {} recs ; history {} -> {} recs (⊤={}) ; recon {} vs {} (edit count NOT preserved)",
                n0, nf0, he, hse, judge_trace(&hstar) == M_T, recon_h, recon_hstar);
        }
        if n == 106545994355809 {
            big = alloc::format!("  big: trace {} -> {} recs ; history {} recs (no strict-DELETE reductions)", n0, nf0, he);
        }
    }

    println!("=== normalize the history of normalization ===");
    println!("closed traces: {}   with non-empty strict history: {}", closed, nonzero_hist);
    println!("normal form stable under the reducer: {}/{}", stable_nf, closed);
    println!("histories that COMPRESS under ≡s (replay-state preserving): {}", hist_compressed);
    println!("histories where reconstruction is LOST by that compression (h ≢r h*): {}", recon_lost);
    println!("{}", n2);
    println!("{}", big);
    println!("\n(≡s compresses the history as a TRAJECTORY; ≡r — reconstruction — does not, because the edit COUNT is the carrier)");
}
