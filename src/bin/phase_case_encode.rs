//! Build-time adapter. Uses the canonical numeral emitter for case inputs.
fn main() {
    for value in std::env::args().skip(1) {
        let tape = vox::morphism_factor::decimal_to_tape(&value).expect("decimal case input");
        println!("{}", vox::morphism_factor::emit_numeral(&tape));
    }
}
