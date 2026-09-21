//! No-runtime-input fixed-point membrane artifact.
//!
//! `N` is supplied only at build time as a canonical IMASM numeral word. The
//! executable consumes that value together with the compiled fixed-point
//! membrane and emits only p and q. No factor, period, base, phase sample,
//! winding, re-entry count, or surviving N is accepted or returned.

use vox::fixed_point_membrane::CompiledFixedPointMembrane;
use vox::morphism_factor::{dec_of, emit_numeral, parse_numeral};

include!(concat!(env!("OUT_DIR"), "/baked_inputs.rs"));

fn main() -> Result<(), String> {
    if std::env::args_os().nth(1).is_some() {
        return Err(String::from("fixed_point_baked accepts no runtime arguments"));
    }

    // Other baked executables share these generated optional fields. This
    // membrane consumes only the modulus word.
    let _ = (BAKED_BASE_WORD, BAKED_WIDTH_WORD);

    let n_word = BAKED_MODULUS_WORD.ok_or("build with one baked IMASM modulus word")?;
    let n = parse_numeral(n_word)?;
    if emit_numeral(&n) != n_word {
        return Err(String::from("baked N is not the canonical IMASM numeral encoding"));
    }

    let pair = CompiledFixedPointMembrane::compiled().consume(n)?;

    println!("p_word {}", emit_numeral(&pair.p));
    println!("q_word {}", emit_numeral(&pair.q));
    println!("p {}", dec_of(&pair.p));
    println!("q {}", dec_of(&pair.q));
    Ok(())
}
