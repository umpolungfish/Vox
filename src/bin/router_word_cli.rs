// router_word_cli — the clause is only a word with a reader: run_word (tape-matched)
// == run_mark (mark-keyed clauses) == run (enum oracle), on closure and on every step.
extern crate alloc;
use ::vox::router_object::{RouterObject, run, partial_router};
use ::vox::router_marks::{RouterG, run_mark, run_word, match_word, M_N};

fn main() {
    let routers: [(&str, RouterObject); 2] = [
        ("initial", RouterObject::initial()),
        ("partial", partial_router()),
    ];

    // ---- TEST 1: match_word on the tape == apply_g on the clause vector, all keys ----
    let jmarks = ['⊤', '⊥', '⊞', '⊙'];
    let smarks = ['⊢', '⊣', '⋈'];
    let mut key_total = 0usize; let mut key_agree = 0usize;
    for (_rn, rk) in &routers {
        let rg = RouterG::from_enum(rk);
        let word: Vec<char> = rg.encode().chars().collect();
        for &jm in &jmarks {
            for &sm in &smarks {
                let a = rg.apply_g(jm, sm).map(|c| (c.transform_word.clone(), c.next));
                let b = match_word(&word, jm, sm);
                key_total += 1;
                if a == b { key_agree += 1; }
            }
        }
    }

    // ---- TEST 2/3: closure and trace equality across the corpus ----
    let mut corpus: Vec<u64> = Vec::new();
    for n in 2u64..=100 { corpus.push(n); }
    corpus.extend_from_slice(&[121, 143, 209, 221, 247, 323, 391, 437, 106545994355809, 1000003, 8, 1024, 7919]);
    let mut total = 0usize; let mut clo = 0usize; let mut tr = 0usize;
    let mut wit = String::from("none");
    for (rn, rk) in &routers {
        let rg = RouterG::from_enum(rk);
        let word: Vec<char> = rg.encode().chars().collect();
        for &n in &corpus {
            let (er, etraj) = run(rk, n, 64);
            let (mr, mtraj) = run_mark(&rg, n, 64);
            let (wr, wtraj) = run_word(&word, n, 64);
            total += 1;
            if er == mr && er == wr { clo += 1; } else if wit == "none" {
                wit = format!("closure {} n={} enum={:?} mark={:?} word={:?}", rn, n, er, mr, wr);
            }
            let mk_eq = etraj.iter().zip(&mtraj).all(|(e, g)| {
                e.repr.glyph() == g.repr && e.judgment.glyph() == g.judgment
                    && (if e.recognised { '⊤' } else { M_N }) == g.recognised
                    && e.next.map(|t| t.glyph()).unwrap_or('⊡') == g.next
                    && if e.recognised { e.applied_word == g.applied_word.iter().collect::<String>() } else { g.applied_word == alloc::vec![M_N] }
            }) && etraj.len() == mtraj.len();
            // run_word must reproduce run_mark exactly (it is the same law with tape storage)
            let w_eq = mtraj.iter().zip(&wtraj).all(|(a, b)| {
                a.repr == b.repr && a.judgment == b.judgment && a.recognised == b.recognised
                    && a.next == b.next && a.applied_word == b.applied_word
            }) && mtraj.len() == wtraj.len();
            if mk_eq && w_eq { tr += 1; } else if wit == "none" {
                wit = format!("trace {} n={} lens enum={} mark={} word={}", rn, n, etraj.len(), mtraj.len(), wtraj.len());
            }
        }
    }

    println!("TEST1 match_word(tape) == apply_g(clauses) : {}/{}", key_agree, key_total);
    println!("TEST2 closure enum == mark == word : {}/{}", clo, total);
    println!("TEST3 trace enum == mark == word   : {}/{}   (first diff: {})", tr, total, wit);

    // ---- TEST 4: the word is the only storage; show it for the canonical router ----
    let rg = RouterG::from_enum(&routers[0].1);
    let word = rg.encode();
    println!("TEST4 canonical router as a word: {} marks, {} clauses on the tape", word.chars().count(), rg.clauses.len());

    let pass = key_agree == key_total && clo == total && tr == total;
    println!("\nROUTER-WORD {}", if pass { "PASS" } else { "FAIL" });
}
