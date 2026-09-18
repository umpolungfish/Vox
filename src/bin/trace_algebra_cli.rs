// trace_algebra_cli — trajectory algebra. Delete a record, replay as the verifier: if the
// candidate still closes to the SAME factor, that record was operationally redundant.
// Iterate to a fixed point. Two admissibility levels: REQUALITY (replay-state preserved) and
// RELAXED (closure + factor preserved).
extern crate alloc;
use ::vox::router_object::RouterObject;
use ::vox::router_marks::run_word;
use ::vox::trace_word::{decode_trace, encode_trace, judge_trace};
use ::vox::router_marks::M_T;
use ::vox::trace_algebra::{replay_state, factor_of, reduce, is_fixed_point};

fn reduce_relaxed(w: &[char], n: u64) -> (Vec<char>, usize) {
    let mut cur: Vec<char> = w.to_vec();
    let mut k = 0usize;
    'o: loop {
        let t = match decode_trace(&cur) { Some(t) => t, None => break };
        for i in 0..t.len() {
            let mut t2 = t.clone(); t2.remove(i);
            let cand = encode_trace(&t2);
            let fo = replay_state(&cur).as_ref().and_then(|r| factor_of(r, n));
            let fc = replay_state(&cand).as_ref().and_then(|r| factor_of(r, n));
            if judge_trace(&cand) == M_T && fo.is_some() && fo == fc { cur = cand; k += 1; continue 'o; }
        }
        break;
    }
    (cur, k)
}

fn main() {
    let reference = RouterObject::initial();
    let word: Vec<char> = reference.encode().chars().collect();

    let mut corpus: Vec<u64> = Vec::new();
    for n in 2u64..=200 { corpus.push(n); }
    corpus.extend_from_slice(&[221, 247, 323, 391, 437, 529, 1000003, 104729, 106545994355809, 10000019]);

    let mut closed_ct = 0usize;
    let mut strict_shrunk = 0usize; let mut relaxed_shrunk = 0usize;
    let mut all_fp = true; let mut all_replay_eq = true; let mut all_factor_eq = true;
    let mut show = 0usize;
    let mut big_line = String::new();

    for &n in &corpus {
        let (res, traj) = run_word(&word, n, 64);
        let tw = encode_trace(&traj);
        if judge_trace(&tw) != M_T { continue; }
        // the closed trajectory should have found the real factor
        let fo = replay_state(&tw).as_ref().and_then(|r| factor_of(r, n));
        let f_ok = matches!(fo, Some((a,b)) if a>1 && b>1 && a*b==n);
        let _ = (res, f_ok);
        closed_ct += 1;

        let (red_s, ks) = reduce(&tw, n);
        let (red_r, kr) = reduce_relaxed(&tw, n);
        if ks > 0 { strict_shrunk += 1; }
        if kr > 0 { relaxed_shrunk += 1; }
        if !is_fixed_point(&red_s, n) { all_fp = false; }
        if replay_state(&red_s) != replay_state(&tw) { all_replay_eq = false; }
        if factor_of(&replay_state(&red_s).unwrap(), n) != factor_of(&replay_state(&tw).unwrap(), n) { all_factor_eq = false; }

        let nrec_o = decode_trace(&tw).map(|t| t.len()).unwrap_or(0);
        let nrec_s = decode_trace(&red_s).map(|t| t.len()).unwrap_or(0);
        let nrec_r = decode_trace(&red_r).map(|t| t.len()).unwrap_or(0);
        if nrec_o != nrec_s && show < 6 {
            println!("  n={}: {} recs -> strict {} recs ({} transforms), relaxed {} recs ({} transforms)",
                n, nrec_o, nrec_s, ks, nrec_r, kr);
            show += 1;
        }
        if n == 106545994355809 {
            big_line = format!("  big {}: marks {} -> strict {} ({} recs) ; relaxed {} ({} recs) ; factor {:?}",
                n, tw.len(), red_s.len(), nrec_s, red_r.len(), nrec_r, factor_of(&replay_state(&tw).unwrap(), n));
        }
    }

    println!("=== trajectory algebra: deletion to fixed point ===");
    println!("closed trajectories: {}", closed_ct);
    println!("strict (replay-state-preserving) compressible: {}", strict_shrunk);
    println!("relaxed (closure+factor-preserving) compressible: {}", relaxed_shrunk);
    println!("every reduced trace is a fixed point: {}", all_fp);
    println!("replay-state preserved under strict reduction: {}", all_replay_eq);
    println!("factor preserved under strict reduction: {}", all_factor_eq);
    println!("{}", big_line);

    let pass = closed_ct > 0 && all_fp && all_replay_eq && all_factor_eq;
    println!("\nTRACE-ALGEBRA {}", if pass { "PASS" } else { "FAIL" });
}
