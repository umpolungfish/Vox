extern crate alloc;
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let cases: Vec<u64> = if args.is_empty() {
        vec![1000036000099, 4000064000087, 61304504203, 723862509691, 8097694716157, 106545994355809, 38531827129376957]
    } else { args.iter().filter_map(|s| s.parse().ok()).collect() };
    println!("router operators: {}", ::vox::meta_router::words());
    for &n in cases.iter() {
        let t0 = std::time::Instant::now();
        let (res, traj) = ::vox::meta_router::route(n);
        let us = t0.elapsed().as_micros();
        let found = match res { Some((p, q)) => format!("{} x {} ok={}", p, q, p * q == n), None => String::from("None") };
        println!("N={n:>18} -> {found}  steps={} {us}us", traj.len());
        println!("   {}", ::vox::meta_router::trajectory_word(&traj));
        for s in traj.iter() {
            println!("     {:>14}  judge {}  {}  [{}]", s.repr.name(), s.judg.name(), s.word, s.note);
        }
    }
}
