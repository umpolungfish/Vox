// trace_measure_cli — the probe derivation: host vs a resident MEASURE word.
extern crate alloc;
use ::vox::router_object::{RouterObject, Belnap, ReprTag, TraceStep, run, partial_router, no_n_router};
use ::vox::router_store::{TraceStore, measure, OP_MEASURE};

fn serialize_trace(tr: &[TraceStep]) -> Vec<char> {
    let mut v = Vec::new();
    for s in tr { v.push(s.judgment.glyph()); v.push(s.repr.glyph()); v.push(if s.recognised { '\u{22A4}' } else { '\u{22A5}' }); }
    v
}
fn host_probe(tr: &[TraceStep]) -> Option<(Belnap, ReprTag)> {
    for s in tr { if s.judgment == Belnap::B && !s.recognised { return Some((Belnap::B, s.repr)); } }
    None
}
fn imasm_probe(tr: &[TraceStep]) -> Option<[char; 2]> {
    let mut st = TraceStore::new(serialize_trace(tr));
    measure(&OP_MEASURE.chars().collect::<Vec<_>>(), &mut st)
}
fn show(p: Option<[char; 2]>) -> String { match p { Some(a) => a.iter().collect(), None => "-".into() } }

fn main() {
    let mut init = RouterObject::initial(); init.canonicalize();
    let routers: Vec<(&str, RouterObject)> = vec![
        ("initial", init), ("partial", partial_router()), ("no_n", no_n_router()),
    ];
    let ns: Vec<u64> = vec![1000036000099, 61304504203, 723862509691, 8097694716157,
                            106545994355809, 38531827129376957, 1000003, 1000000007];
    println!("=== probe derivation: host vs MEASURE word {} ===", OP_MEASURE);
    let mut agree = 0; let mut total = 0;
    for (rn, r) in &routers {
        for &n in &ns {
            let (_, tr) = run(r, n, 64);
            let hp = host_probe(&tr);
            let hg = hp.map(|(j, s)| [j.glyph(), s.glyph()]);
            let ip = imasm_probe(&tr);
            let eq = hg == ip;
            total += 1; if eq { agree += 1; }
            println!("{:<8} N={:<18} steps {:>2}  host {} / imasm {}  eq {}",
                rn, n, tr.len(), show(hg), show(ip), eq);
        }
    }
    println!("PROBE AGREEMENT: {}/{}", agree, total);
    println!("PROBE-DERIVATION: {}", if agree == total { "PASS" } else { "OPEN" });
}
