// trace_replay_cli — the trace as an executable proof-object.
extern crate alloc;
use ::vox::router_object::{RouterObject, Belnap, ReprTag, TraceStep, run, judge_router_v2, rewrite_policy, partial_router, RW_EXPOSE};
use ::vox::router_store::{IStore, execute, OP_PRESERVE, OP_SCAN_APPEND};
use ::vox::trace_object::{TraceRecord, encode_trace, decode_trace};

const B_MARK: char = '\u{229E}';  // ⊞
const UNREC: char = '\u{22A5}';   // ⊥
const REC: char = '\u{22A4}';     // ⊤
const NUL: char = '\u{0}';

fn recs(tr: &[TraceStep]) -> Vec<TraceRecord> {
    tr.iter().map(|s| TraceRecord {
        judgment: s.judgment.glyph(), source: s.repr.glyph(),
        flag: if s.recognised { REC } else { UNREC },
        word: s.applied_word.clone(),
    }).collect()
}
fn probe_of(rs: &[TraceRecord]) -> Option<[char; 2]> {
    for r in rs { if r.judgment == B_MARK && r.flag == UNREC { return Some([r.judgment, r.source]); } }
    None
}
fn closed(res: Option<(u64, u64)>, n: u64) -> bool { matches!(res, Some((p, q)) if p > 1 && q > 1 && p * q == n) }

fn main() {
    let reference = RouterObject::initial();
    let pol = rewrite_policy();
    let corpus: Vec<u64> = vec![106545994355809];

    // ---- PHASE A: produce — run the loop, RECORD every generation's trace ----
    let mut rk = partial_router();
    let mut gen_traces: Vec<Vec<TraceRecord>> = Vec::new();
    for _ in 0..6 {
        let results: Vec<(Option<(u64, u64)>, Vec<TraceStep>)> = corpus.iter().map(|&n| run(&rk, n, 64)).collect();
        let traces: Vec<Vec<TraceStep>> = results.iter().map(|(_, t)| t.clone()).collect();
        let cl = results.iter().zip(&corpus).filter(|((res, _), &n)| closed(*res, n)).count();
        let v = judge_router_v2(&traces, cl, corpus.len());
        let rs = recs(&traces[0]);
        gen_traces.push(rs);
        let op = pol.apply(v, ReprTag::Symmetric).map(|c| c.transform_word.clone()).unwrap_or_default();
        let probe = if op == RW_EXPOSE { probe_of(gen_traces.last().unwrap()) } else { None };
        let (word, p) = match probe { Some(p) => (OP_SCAN_APPEND, p), None => (OP_PRESERVE, [NUL, NUL]) };
        let mut st = IStore::new(&rk.word, &reference.encode());
        execute(&word.chars().collect::<Vec<_>>(), &mut st, p);
        rk = RouterObject::decode(&st.router_word()).expect("decode");
        if word == OP_PRESERVE { break; }
    }
    let final_a = rk.word.clone();

    // ---- TEST 1: trace -> encode -> decode -> trace  (exact) ----
    let mut t1 = true;
    for rs in &gen_traces {
        match decode_trace(&encode_trace(rs)) { Some(d) => if d != *rs { t1 = false; }, None => t1 = false }
    }
    println!("TEST1 trace->encode->decode exact: {}", t1);

    // ---- TEST 2: applied_word(trace) == the word actually handed by the router ----
    let mut t2 = true; let mut checked = 0;
    let mut r = partial_router();
    for (g, rs) in gen_traces.iter().enumerate() {
        for rec in rs {
            if rec.flag == REC {
                checked += 1;
                if let (Some(jb), Some(sr)) = (Belnap::from_glyph(rec.judgment), ReprTag::from_glyph(rec.source)) {
                    match r.apply(jb, sr) { Some(c) => if c.transform_word != rec.word { t2 = false; }, None => t2 = false }
                }
            }
        }
        if g + 1 < gen_traces.len() {
            let probe = probe_of(rs);
            let (word, p) = match probe { Some(p) => (OP_SCAN_APPEND, p), None => (OP_PRESERVE, [NUL, NUL]) };
            let mut st = IStore::new(&r.word, &reference.encode());
            execute(&word.chars().collect::<Vec<_>>(), &mut st, p);
            r = RouterObject::decode(&st.router_word()).expect("decode");
        }
    }
    println!("TEST2 applied_word == router word handed: {}  ({} recognised steps checked)", t2, checked);

    // ---- TEST 3: replay(trace) reproduces the router trajectory — no run() calls ----
    let mut rr = partial_router();
    let mut trace_words: Vec<String> = vec![rr.word.clone()];
    for rs in &gen_traces {
        let (word, p) = match probe_of(rs) { Some(p) => (OP_SCAN_APPEND, p), None => (OP_PRESERVE, [NUL, NUL]) };
        let mut st = IStore::new(&rr.word, &reference.encode());
        execute(&word.chars().collect::<Vec<_>>(), &mut st, p);
        rr = RouterObject::decode(&st.router_word()).expect("decode");
        trace_words.push(rr.word.clone());
        if word == OP_PRESERVE { break; }
    }
    let t3 = rr.word == final_a && rr.transitions.len() == 3;
    println!("TEST3 replay reproduces final router: {}  ({} glyphs, {} clauses)", t3, final_a.chars().count(), rr.transitions.len());
    println!("REPLAY: {}", if t1 && t2 && t3 { "PASS" } else { "OPEN" });
}
