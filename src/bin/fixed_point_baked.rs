//! No-runtime-input fixed-point factor artifact.
//!
//! `N` is supplied only at build time as a canonical IMASM numeral word. The
//! compiled executable contains that word, reconstructs its numeral tape, runs
//! the fixed-point spectral construction, and prints only the factors produced
//! by the existing passive extractor. No factor, period, base, or re-entry count
//! is accepted from argv or read from a runtime file.
//!
//! The repaired boundary keeps its instant non-walking read unchanged. If that
//! boundary has no closing IFIX deposit, this artifact follows the source TANCH
//! scheduler explicitly: one six-op re-entry cycle at a time, with each cycle
//! consuming one marker and producing exactly one next winding. The executable
//! therefore needs only baked N; no host-side re-entry count is supplied.

use vox::factor_extract::extract;
use vox::hadamard_factor_bridge::HadamardDescent;
use vox::hadamard_gate::HadamardCarrier;
use vox::morphism_factor::{dec_of, emit_numeral, parse_numeral};

include!(concat!(env!("OUT_DIR"), "/baked_inputs.rs"));

fn main() -> Result<(), String> {
    if std::env::args_os().nth(1).is_some() {
        return Err(String::from("fixed_point_baked accepts no runtime arguments"));
    }

    // The shared bake file also declares the optional phase-binary fields.
    // This artifact deliberately consumes none of them; touching the generated
    // constants only keeps the compiler audit clean while N remains the sole
    // baked input required by this executable.
    let _ = (BAKED_BASE_WORD, BAKED_WIDTH_WORD);

    let n_word = BAKED_MODULUS_WORD.ok_or("build with one baked IMASM modulus word")?;
    let n = parse_numeral(n_word)?;
    if emit_numeral(&n) != n_word {
        return Err(String::from("baked N is not the canonical IMASM numeral encoding"));
    }

    let spectral = HadamardCarrier::new(&n)?
        .fixed_point_spectral_construction()?;

    let factor_carrier = if spectral.instant_non_walking_read()?.is_some() {
        match spectral.descend_boundary_measurement() {
            HadamardDescent::T(carrier) => carrier,
            HadamardDescent::B(_) => {
                return Err(String::from("baked N produced only a productive boundary fork"));
            }
            HadamardDescent::N(_) => {
                return Err(String::from("baked N closed at the boundary without a nontrivial split"));
            }
            HadamardDescent::F => {
                return Err(String::from("baked fixed-point boundary descent failed"));
            }
        }
    } else {
        // TANCH is the cyclic fixed-point anchor. Follow its scheduler exactly
        // one structural cycle per iteration. This does not alter or call the
        // instant non-walking read again, and no period/re-entry count is baked.
        let mut state = spectral.reentry_anchor()?;
        loop {
            state = spectral.run_reentry_cycle(&state)?.next;
            if !state.closes_modular_phase() {
                continue;
            }

            break match spectral.descend_reentry_state(&state) {
                HadamardDescent::T(carrier) => carrier,
                HadamardDescent::B(_) => {
                    return Err(String::from("baked N produced only a productive re-entry fork"));
                }
                HadamardDescent::N(_) => {
                    return Err(String::from("first closing re-entry did not yield a nontrivial split"));
                }
                HadamardDescent::F => {
                    return Err(String::from("baked fixed-point re-entry descent failed"));
                }
            };
        }
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
