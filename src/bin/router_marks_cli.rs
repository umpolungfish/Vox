// router_marks_cli — the enum-backed clause table and the mark-keyed one produce
// identical control: apply_g == apply on every key, run_mark == run on every step.
extern crate alloc;
use ::vox::router_object::{RouterObject, run, partial_router};
use ::vox::router_marks::{RouterG, run_mark, M_N, M_FIX, M_T};

fn main() {
    let routers: [(&str, RouterObject); 2] = [
        ("initial", RouterObject::initial()),
        ("partial", partial_router()),
    ];

    // ---- TEST 1: apply_g == apply over every (judgment, source) key ----
    let jmarks = ['⊤', '⊥', '⊞', '⊙'];
    let smarks = ['⊢', '⊣', '⋈'];
    let mut key_total = 0usize;
    let mut key_agree = 0usize;
    let mut kwit = String::from("none");
    for (rn, rk) in &routers {
        let rg = RouterG::from_enum(rk);
        for &jm in &jmarks {
            for &sm in &smarks {
                let e = rk.apply(
                    ::vox::router_object::Belnap::from_glyph(jm).unwrap(),
                    ::vox::router_object::ReprTag::from_glyph(sm).unwrap());
                let g = rg.apply_g(jm, sm);
                key_total += 1;
                let same = match (e, g) {
                    (None, None) => true,
                    (Some(a), Some(b)) => {
                        a.judgment.glyph() == b.judgment
                            && a.source.map(|t| t.glyph()).unwrap_or('⊙') == b.source
                            && a.transform_word == b.transform_word.iter().collect::<String>()
                            && a.next.map(|t| t.glyph()).unwrap_or('⊡') == b.next
                    }
                    _ => false,
                };
                if same { key_agree += 1; } else if kwit == "none" {
                    kwit = format!("{} jm={} sm={}", rn, jm, sm);
                }
            }
        }
    }

    // ---- TEST 2: run_mark == run, closure + trace step-for-step ----
    let mut corpus: Vec<u64> = Vec::new();
    for n in 2u64..=100 { corpus.push(n); }
    corpus.extend_from_slice(&[121, 143, 209, 221, 247, 323, 391, 437, 106545994355809, 1000003, 8, 1024, 7919]);
    let mut total = 0usize;
    let mut closure_agree = 0usize;
    let mut trace_agree = 0usize;
    let mut twit = String::from("none");
    for (rn, rk) in &routers {
        let rg = RouterG::from_enum(rk);
        for &n in &corpus {
            let (er, etraj) = run(rk, n, 64);
            let (gr, gtraj) = run_mark(&rg, n, 64);
            total += 1;
            if er == gr { closure_agree += 1; } else if twit == "none" {
                twit = format!("closure {} n={} enum={:?} mark={:?}", rn, n, er, gr);
            }
            let mut ok = etraj.len() == gtraj.len();
            if ok {
                for (e, g) in etraj.iter().zip(&gtraj) {
                    let e_recog = if e.recognised { M_T } else { M_N };
                    let e_next = e.next.map(|t| t.glyph()).unwrap_or(M_FIX);
                    if e.repr.glyph() != g.repr
                        || e.judgment.glyph() != g.judgment
                        || e_recog != g.recognised
                        || e_next != g.next
                        || if e.recognised { e.applied_word != g.applied_word.iter().collect::<String>() } else { g.applied_word != alloc::vec![M_N] } { ok = false; break; }
                }
            }
            if ok { trace_agree += 1; } else if twit == "none" {
                twit = format!("trace {} n={} enum_len={} mark_len={}", rn, n, etraj.len(), gtraj.len());
            }
        }
    }

    println!("TEST1 apply_g == apply over keys : {}/{}  (first diff: {})", key_agree, key_total, kwit);
    println!("TEST2 closure run_mark == run : {}/{}", closure_agree, total);
    println!("TEST3 trace run_mark == run (repr,judgment,recognised,next,word) : {}/{}  (first diff: {})", trace_agree, total, twit);

    // ---- TEST 4: the mark router word round-trips through the enum decoder ----
    let rg = RouterG::from_enum(&routers[0].1);
    let word = rg.encode();
    let dec = RouterObject::decode(&word).map(|r| r.transitions.len()).unwrap_or(0);
    println!("TEST4 mark-router word decodes by the enum oracle into {} clauses", dec);

    let pass = key_agree == key_total && closure_agree == total && trace_agree == total;
    println!("\nROUTER-MARKS {}", if pass { "PASS" } else { "FAIL" });
}
