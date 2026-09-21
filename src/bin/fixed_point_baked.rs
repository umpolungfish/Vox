//! No-runtime-input fixed-point factor artifact.
//!
//! `N` is supplied only at build time as a canonical IMASM numeral word. The
//! compiled executable contains that word, reconstructs its numeral tape, runs
//! the fixed-point spectral boundary, and prints only the factors produced by
//! the existing passive extractor. No factor, period, base, or re-entry count is
//! accepted from argv or read from a runtime file.

use vox::factor_extract::extract;
use vox::hadamard_factor_bridge::HadamardDescent;
use vox::hadamard_gate::HadamardCarrier;
use vox::morphism_factor::{dec_of, emit_numeral, parse_numeral};

include!(concat!(env!("OUT_DIR"), "/baked_inputs.rs"));

fn main() -> Result<(), String> {
    if std::env::args_os().nth(1).is_some() {
        return Err(String::from("fixed_point_baked accepts no runtime arguments"));
    }

    let n_word = BAKED_MODULUS_WORD.ok_or("build with one baked IMASM modulus word")?;
    let n = parse_numeral(n_word)?;
    if emit_numeral(&n) != n_word {
        return Err(String::from("baked N is not the canonical IMASM numeral encoding"));
    }

    let spectral = HadamardCarrier::new(&n)?
        .fixed_point_spectral_construction()?;
    let factor_carrier = match spectral.descend_boundary_measurement() {
        HadamardDescent::T(carrier) => carrier,
        HadamardDescent::B(_) => return Err(String::from("baked N produced only a productive fork")),
        HadamardDescent::N(_) => return Err(String::from("baked N did not close at a fixed boundary winding")),
        HadamardDescent::F => return Err(String::from("baked fixed-point descent failed")),
    };

    let readout = extract(&factor_carrier)?;
    let p = readout.p.0;
    let q = readout.q.0;

    println!("n_word {n_word}");
    println!("n {}", dec_of(&n));
    println!("p_word {}", emit_numeral(&p));
    println!("q_word {}", emit_numeral(&q));
    println!("p {}", dec_of(&p));
    println!("q {}", dec_of(&q));
    Ok(())
}
