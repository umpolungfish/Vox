extern crate alloc;

// Item position: the generated file declares `const` items, which parse only at
// module scope, not inside a function body.
include!(concat!(env!("OUT_DIR"), "/baked_inputs.rs"));

fn main() {
    let _ = BAKED_WIDTH_WORD;  // phaseB takes no width; keep the baked const live
    let raw = match (BAKED_BASE_WORD, BAKED_MODULUS_WORD) {
        (Some(a), Some(n)) => [a, n].join(" "),
        _ => option_env!("MEMBRANE_WORDS").expect("bake two IMASM words or build with MEMBRANE_WORDS=\"<a> <N>\"").to_string(),
    };
    let mut ws: Vec<Vec<char>> = Vec::new();
    for w in raw.split_whitespace() {
        ws.push(::vox::morphism_factor::parse_numeral(w).expect("numeral parse"));
    }
    let (a, n) = (&ws[0], &ws[1]);
    let t0 = std::time::Instant::now();
    match ::vox::shor_braid::shor_factor_via_braid(a, n) {
        Ok((p, q)) => println!("factors {} x {}   [{:?}]",
            ::vox::morphism_factor::dec_of(&p),
            ::vox::morphism_factor::dec_of(&q),
            t0.elapsed()),
        Err(e) => println!("refused: {}   [{:?}]", e, t0.elapsed()),
    }
}
