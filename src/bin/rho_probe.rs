fn main() {
    let args: Vec<String> = std::env::args().collect();
    let n = ::vox::morphism_factor::decimal_to_tape(&args[1]).unwrap();
    let budget: u64 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(1u64<<32);
    let t0 = std::time::Instant::now();
    match ::vox::morphism_factor::pollard_rho_brent(&n, budget) {
        Some(d) => println!("factor {}  [{:?}]", ::vox::morphism_factor::dec_of(&d), t0.elapsed()),
        None => println!("none  [{:?}]", t0.elapsed()),
    }
}
