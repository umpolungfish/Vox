// router_rr_cli — R-of-R with the probe DERIVED BY THE MEASURE WORD.
// Host probe derivation is gone from production; the probe comes from IMASM.
extern crate alloc;
use ::vox::router_object::{RouterObject, Belnap, TraceStep, run, judge_router, rewrite, partial_router};
use ::vox::router_store::{IStore, execute, OP_PRESERVE, OP_SCAN_APPEND, TraceStore, measure, OP_MEASURE};

fn closed(res: Option<(u64, u64)>, n: u64) -> bool {
    matches!(res, Some((p, q)) if p > 1 && q > 1 && p * q == n)
}

fn serialize_trace(tr: &[TraceStep]) -> Vec<char> {
    let mut v = Vec::new();
    for s in tr { v.push(s.judgment.glyph()); v.push(s.repr.glyph()); v.push(if s.recognised { '\u{22A4}' } else { '\u{22A5}' }); }
    v
}

/// The probe, derived entirely by the MEASURE word from the serialized trace.
fn word_probe(tr: &[TraceStep]) -> Option<[char; 2]> {
    let mut ts = TraceStore::new(serialize_trace(tr));
    measure(&OP_MEASURE.chars().collect::<Vec<_>>(), &mut ts)
}

fn main() {
    let reference = RouterObject::initial();
    let corpus: Vec<u64> = vec![106545994355809];
    println!("=== R-of-R, probe derived by MEASURE word {} ===", OP_MEASURE);
    println!("words: PRESERVE={}  SCAN_APPEND={}  MEASURE={}", OP_PRESERVE, OP_SCAN_APPEND, OP_MEASURE);

    let mut agree = 0; let mut total = 0;
    let mut rk = partial_router();
    for gen in 0..5usize {
        let results: Vec<(Option<(u64, u64)>, Vec<TraceStep>)> = corpus.iter().map(|&n| run(&rk, n, 64)).collect();
        let traces: Vec<Vec<TraceStep>> = results.iter().map(|(_, t)| t.clone()).collect();
        let cl = results.iter().zip(&corpus).filter(|((res, _), &n)| closed(*res, n)).count();
        let v = judge_router(&traces, cl, corpus.len());

        let r_host = rewrite(&rk, v, &traces, &reference);

        let probe_opt = word_probe(&traces[0]);           // <-- IMASM, not host
        let (word, probe) = match probe_opt {
            Some(p) => (OP_SCAN_APPEND, p),
            None => (OP_PRESERVE, ['\u{0}', '\u{0}']),
        };
        let mut st = IStore::new(&rk.word, &reference.encode());
        execute(&word.chars().collect::<Vec<_>>(), &mut st, probe);
        let r_imasm = RouterObject::decode(&st.router_word()).expect("decode resident result");

        let eq = r_host.word == r_imasm.word;
        total += 1; if eq { agree += 1; }
        println!("gen {}: verdict {}  clauses {} -> host {} / imasm {}  eq {}",
            gen, v.name(), rk.transitions.len(), r_host.transitions.len(), r_imasm.transitions.len(), eq);

        rk = r_imasm;
        if probe_opt.is_none() { break; }
    }

    let mut r_full = RouterObject::initial();
    r_full.canonicalize();
    let r_host_full = rewrite(&r_full, Belnap::T, &[], &reference);
    let mut s2 = IStore::new(&r_full.word, &reference.encode());
    execute(&OP_PRESERVE.chars().collect::<Vec<_>>(), &mut s2, ['\u{0}', '\u{0}']);
    let full_eq = s2.router_word() == r_host_full.word && s2.router_word() == r_full.word;
    total += 1; if full_eq { agree += 1; }
    println!("initial+T PRESERVE: byte-equal {}", full_eq);

    println!("AGREEMENT: {}/{}", agree, total);
    println!("R-OF-R: {}", if agree == total { "PASS" } else { "OPEN" });
}
