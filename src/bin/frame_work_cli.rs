// frame_work_cli — FRAME_WORK is resident. The machine consumes marks and exposes only
// balanced structure and positions. frame_work.py is kept ONLY as an oracle: we require
// byte equality resident_frame_work(w) == python_frame_work(w) for every word.
extern crate alloc;
use ::vox::frame_work::{adjacent_dyad, captured_span, frame_work};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        eprintln!("frame_work_cli <word> ...  resident FRAME_WORK");
        return;
    }
    for a in args {
        let m: Vec<char> = a.chars().collect();
        let out: String = frame_work(&m).into_iter().collect();
        let span: String = captured_span(&m).into_iter().collect();
        eprintln!("  dyad={} span={:?} -> {}", adjacent_dyad(&m).is_some(), span, out);
        println!("{}", out);
    }
}
