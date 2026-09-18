// carrier_identity_cli — the ⊙ identity: judgment N and wildcard source are one mark.
extern crate alloc;
use ::vox::router_object::{RouterObject, Belnap, ReprTag};
use ::vox::carrier::{GValue, probe, is_probe, is_judgment, is_source, ANY_MARK};

fn main() {
    let r = RouterObject::initial();
    let nclause = r.apply(Belnap::N, ReprTag::Residue).expect("N clause exists");
    let src_glyph = match nclause.source { Some(t) => t.glyph(), None => ANY_MARK };
    println!("the N clause header: judgment {:?}  source {:?}  (both ⊙)", nclause.judgment.glyph(), src_glyph);

    let j = GValue::single(Belnap::N.glyph());     // ⊙ read as a judgment (slot 0)
    let s = GValue::single(ANY_MARK);              // ⊙ read as a wildcard source (slot 1)
    let p = probe(&j, &s);
    println!("probe(N, any) = GValue{:?}   is_probe {}  is_judgment(slot0) {}  is_source(slot1) {}",
        p.marks, is_probe(&p), is_judgment(&j), is_source(&s));

    let mut ok = true;
    for src in [ReprTag::Symmetric, ReprTag::Residue, ReprTag::Multiplicative] {
        let hit = r.apply(Belnap::N, src).is_some();
        println!("  apply(N, {:<14}) -> {}", src.name(), hit);
        if !hit { ok = false; }
    }
    let as_judg = is_judgment(&GValue::single(ANY_MARK));
    let as_src  = is_source(&GValue::single(ANY_MARK));
    println!("⊙ as judgment: {}   ⊙ as source: {}", as_judg, as_src);
    println!("IDENTITY: no distinction == no restriction on distinction — one mark, positional reading, no host special case");
    println!("CARRIER-IDENTITY: {}", if ok && as_judg && as_src { "PASS" } else { "OPEN" });
}
