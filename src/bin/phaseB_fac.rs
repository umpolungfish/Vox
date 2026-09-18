extern crate alloc;
fn main() {
    let raw = option_env!("MEMBRANE_WORDS").expect("build with MEMBRANE_WORDS=\"<a> <N>\"");
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
