// carrier_cli — the carrier: repr tags, judgments, probes are one value.
extern crate alloc;
use ::vox::router_object::{ReprTag, Belnap};
use ::vox::carrier::GValue;

fn main() {
    let reprs = [ReprTag::Symmetric, ReprTag::Residue, ReprTag::Multiplicative];
    let judges = [Belnap::T, Belnap::F, Belnap::B, Belnap::N];
    println!("=== one carrier, three shapes ===");
    for r in &reprs { println!("  repr   {:<14} -> GValue{:?}", r.name(), GValue::single(r.glyph()).marks); }
    for b in &judges { println!("  judge  {:<14} -> GValue{:?}", b.name(), GValue::single(b.glyph()).marks); }
    let shared: Vec<char> = reprs.iter().map(|r| r.glyph())
        .filter(|g| judges.iter().any(|b| b.glyph() == *g)).collect();
    println!("marks shared by the two families: {:?}", shared);
    println!("  -> the undifferentiated corner: N (no distinction) IS any (matches any source); ⊙ is one mark.");
    let probe = GValue::new(&[Belnap::B.glyph(), ReprTag::Symmetric.glyph()]);
    println!("probe (B, Symmetric) -> GValue arity {} {:?}", probe.arity(), probe.marks);

    let mut ok = true;
    for r in &reprs { if GValue::single(r.glyph()).mark0() != Some(r.glyph()) { ok = false; } }
    for b in &judges { if GValue::single(b.glyph()).mark0() != Some(b.glyph()) { ok = false; } }
    println!("lossless single-mark round-trip: {}", ok);
    println!("CARRIER: {}", if ok { "PASS" } else { "OPEN" });
}
