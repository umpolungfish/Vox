// meta_router_g_cli — the resident trajectory in marks reproduces the enum
// trajectory bit-for-bit: route_g(N) vs meta_router::route(N).
// The judge is judge_g (GValue), re-entry is reenter_mark — no Judg, no Repr,
// on the loop's path.
extern crate alloc;
use ::vox::meta_router::{route, Judg};
use ::vox::judge_g::{route_g, judg_to_mark, J_T, J_B, J_N, J_F};

fn mark_of(j: Judg) -> char { judg_to_mark(j) }

fn main() {
    // a corpus: small composites, primes, semiprimes, powers, one big semiprime
    let mut corpus: Vec<u64> = Vec::new();
    for n in 2u64..=120 { corpus.push(n); }
    corpus.extend_from_slice(&[121, 143, 209, 221, 247, 323, 391, 437, 529,
        10403, 106545994355809, 1000003, 999983, 4, 8, 16, 1024, 7919, 104729]);

    let mut total = 0usize;
    let mut closure_agree = 0usize;
    let mut marks_agree = 0usize;
    let mut witness = String::from("none");
    let mut big_detail = String::new();

    for &n in &corpus {
        let (via_enum, traj_e) = route(n);
        let (via_marks, traj_g) = route_g(n);
        total += 1;
        if via_enum == via_marks { closure_agree += 1; } else if witness == "none" {
            witness = format!("closure n={} enum={:?} marks={:?}", n, via_enum, via_marks);
        }
        // compare the mark sequence to the enum trajectory's mapped judgments
        let a: Vec<char> = traj_e.iter().map(|s| mark_of(s.judg)).collect();
        let b: Vec<char> = traj_g.iter().map(|s| s.judg).collect();
        if a == b { marks_agree += 1; } else if witness == "none" {
            witness = format!("marks n={} enum={:?} marks={:?}", n, a, b);
        }
        if n == 106545994355809 {
            big_detail = format!(
                "  big={} route_g -> {:?}   marks={}   (sym={} mul={})",
                n, via_marks,
                traj_g.iter().map(|s| s.judg).collect::<String>(),
                J_T, J_N);
        }
    }

    println!("corpus size: {}", total);
    println!("TEST1 closure route_g == route : {}/{}", closure_agree, total);
    println!("TEST2 judgment marks route_g == route : {}/{}", marks_agree, total);
    println!("TEST3 first disagreement: {}", witness);
    println!("{}", big_detail);

    // TEST4: a closed big semiprime produces a T trajectory in marks only
    let (_r, tg) = route_g(106545994355809);
    let any_t = tg.iter().any(|s| s.judg == J_T);
    let any_b = tg.iter().any(|s| s.judg == J_B);
    let any_n = tg.iter().any(|s| s.judg == J_N);
    let any_f = tg.iter().any(|s| s.judg == J_F);
    println!("TEST4 big trajectory uses only marks: ⊤={} ⊞={} ⊙={} ⊥={}", any_t, any_b, any_n, any_f);

    let pass = closure_agree == total && marks_agree == total && witness == "none";
    println!("\nROUTE-G {}", if pass { "PASS" } else { "FAIL" });
}
