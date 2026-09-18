// router_rr_cli — R-of-R: the host rewrite vs the resident execution of the rewrite WORD.
// The decisive test is byte-for-byte equality of the resulting serialized router.
extern crate alloc;
use ::vox::router_object::{RouterObject, Belnap, ReprTag, TraceStep, run, judge_router, rewrite, partial_router, no_n_router};
use ::vox::router_store::{IStore, execute, OP_PRESERVE, OP_SCAN_APPEND};

fn closed(res: Option<(u64, u64)>, n: u64) -> bool {
    matches!(res, Some((p, q)) if p > 1 && q > 1 && p * q == n)
}

fn first_exposed(router: &RouterObject, tr: &[TraceStep]) -> Option<(Belnap, ReprTag)> {
    for s in tr {
        if s.judgment == Belnap::B {
            let covered = router.transitions.iter().any(|c| c.judgment == Belnap::B && c.source == Some(s.repr));
            if !covered { return Some((Belnap::B, s.repr)); }
        }
    }
    None
}

fn main() {
    let reference = RouterObject::initial();
    let corpus: Vec<u64> = vec![106545994355809]; // far-apart: needs the whole ladder
    println!("=== R-of-R: host rewrite(R,v)  vs  execute(word, store) ===");
    println!("words: PRESERVE={}  SCAN_APPEND={}", OP_PRESERVE, OP_SCAN_APPEND);

    let mut agree = 0usize;
    let mut total = 0usize;
    let mut rk = partial_router();
    for gen in 0..5usize {
        let results: Vec<(Option<(u64, u64)>, Vec<TraceStep>)> = corpus.iter().map(|&n| run(&rk, n, 64)).collect();
        let traces: Vec<Vec<TraceStep>> = results.iter().map(|(_, t)| t.clone()).collect();
        let cl = results.iter().zip(&corpus).filter(|((res, _), &n)| closed(*res, n)).count();
        let v = judge_router(&traces, cl, corpus.len());

        // --- host oracle ---
        let r_host = rewrite(&rk, v, &traces, &reference);

        // --- resident word ---
        let exposed = first_exposed(&rk, &traces[0]);
        let (word, probe) = match exposed {
            Some((j, r)) => (OP_SCAN_APPEND, [j.glyph(), r.glyph()]),
            None => (OP_PRESERVE, ['\u{0}', '\u{0}']),
        };
        let mut st = IStore::new(&rk.word, &reference.encode());
        execute(&word.chars().collect::<Vec<_>>(), &mut st, probe);
        let r_imasm = RouterObject::decode(&st.router_word()).expect("decode resident result");

        let eq = r_host.word == r_imasm.word;
        total += 1; if eq { agree += 1; }
        println!("gen {}: verdict {}  clauses {} -> host {} / imasm {}  byte-equal: {}",
            gen, v.name(), rk.transitions.len(), r_host.transitions.len(), r_imasm.transitions.len(), eq);

        rk = r_imasm;
        if exposed.is_none() { break; }
    }

    // --- PRESERVE case on the full router (verdict T) ---
    let mut r_full = RouterObject::initial();
    r_full.canonicalize();
    let r_host_full = rewrite(&r_full, Belnap::T, &[], &reference);
    let mut s2 = IStore::new(&r_full.word, &reference.encode());
    execute(&OP_PRESERVE.chars().collect::<Vec<_>>(), &mut s2, ['\u{0}', '\u{0}']);
    let full_eq = s2.router_word() == r_host_full.word && s2.router_word() == r_full.word;
    total += 1; if full_eq { agree += 1; }
    println!("initial+T PRESERVE: byte-equal {}", full_eq);

    // --- DISTINGUISH case: a prime -> verdict N ---
    let prime: u64 = 1000003;
    let rp = no_n_router();
    let pres: Vec<(Option<(u64, u64)>, Vec<TraceStep>)> = vec![run(&rp, prime, 64)];
    let ptr: Vec<Vec<TraceStep>> = pres.iter().map(|(_, t)| t.clone()).collect();
    let pcl = pres.iter().filter(|(res, _)| closed(*res, prime)).count();
    let vp = judge_router(&ptr, pcl, 1);
    let rh = rewrite(&rp, vp, &ptr, &reference);
    let (pw, ppr) = if vp == Belnap::N {
        (OP_SCAN_APPEND, [Belnap::N.glyph(), '\u{2299}'])
    } else if let Some((j, r)) = first_exposed(&rp, &ptr[0]) {
        (OP_SCAN_APPEND, [j.glyph(), r.glyph()])
    } else { (OP_PRESERVE, ['\u{0}', '\u{0}']) };
    let mut ps = IStore::new(&rp.word, &reference.encode());
    execute(&pw.chars().collect::<Vec<_>>(), &mut ps, ppr);
    let pim = RouterObject::decode(&ps.router_word()).expect("decode prime result");
    let peq = rh.word == pim.word;
    total += 1; if peq { agree += 1; }
    println!("prime {}: verdict {}  clauses {} -> host {} / imasm {}  byte-equal: {}",
        prime, vp.name(), rp.transitions.len(), rh.transitions.len(), pim.transitions.len(), peq);

    println!("AGREEMENT: {}/{}", agree, total);
    println!("R-OF-R: {}", if agree == total { "PASS" } else { "OPEN" });
}
