extern crate alloc;
use ::vox::morphism_factor::{dec_of, tape_u64};
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let cases: Vec<u64> = if args.is_empty() {
        vec![15, 21, 35, 91, 143, 8051, 16744463, 1000036000099, 4000064000087]
    } else {
        args.iter().filter_map(|s| s.parse().ok()).collect()
    };
    for &n in cases.iter() {
        let tape = tape_u64(n);
        let t0 = std::time::Instant::now();
        let (res, st) = ::vox::factor_operator::semiprime_shot(&tape, 500_000_000);
        let dt = t0.elapsed().as_millis();
        match res {
            Some((p, q)) => {
                let dp: u128 = dec_of(&p).parse().unwrap();
                let dq: u128 = dec_of(&q).parse().unwrap();
                println!("N={n:>14} -> {dp} x {dq}  ok={}  nodes={} low={} bridge={} high={} depth={} capped={}  {}ms",
                    dp * dq == n as u128, st.nodes, st.low_prunes, st.bridge_prunes, st.high_prunes, st.depth, st.capped, dt);
            }
            None => println!("N={n:>14} -> None  nodes={} capped={}  {}ms", st.nodes, st.capped, dt),
        }
    }
}
