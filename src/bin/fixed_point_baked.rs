//! No-runtime-input fixed-point membrane artifact.
//!
//! `N` is supplied only at build time as a canonical IMASM numeral word. The
//! executable membrane is affine: execution consumes both the baked value and
//! the compiled membrane object and returns only the factor pair. No `N`,
//! membrane state, winding, period, base, phase sample, or re-entry count is
//! returned from a successful invocation.
//!
//! This entrypoint deliberately does not walk TANCH re-entry states. A compiled
//! nested tower must collapse to the membrane relation before execution; runtime
//! iteration is not a substitute for that compilation step.

use vox::factor_extract::extract;
use vox::hadamard_factor_bridge::HadamardDescent;
use vox::hadamard_gate::{HadamardCarrier, Tape};
use vox::morphism_factor::{dec_of, emit_numeral, parse_numeral};

include!(concat!(env!("OUT_DIR"), "/baked_inputs.rs"));

struct FactorPair {
    p: Tape,
    q: Tape,
}

/// The compiled fixed-point membrane. It is intentionally zero-sized: the
/// executable code is the membrane. `consume` takes ownership of both itself and
/// the value, making the successful boundary `(N, membrane) -> (p, q)`.
struct CompiledFixedPointMembrane;

impl CompiledFixedPointMembrane {
    const fn compiled() -> Self {
        Self
    }

    fn consume(self, n: Tape) -> Result<FactorPair, String> {
        let spectral = HadamardCarrier::new(&n)
            .map_err(String::from)?
            .fixed_point_spectral_construction()
            .map_err(String::from)?;

        let factor_carrier = match spectral.descend_boundary_measurement() {
            HadamardDescent::T(carrier) => carrier,
            HadamardDescent::B(_) => {
                return Err(String::from("compiled membrane produced only a productive fork"));
            }
            HadamardDescent::N(_) => {
                return Err(String::from(
                    "compiled membrane exposed no terminal factor relation; runtime re-entry is forbidden",
                ));
            }
            HadamardDescent::F => {
                return Err(String::from("compiled fixed-point membrane failed"));
            }
        };

        let readout = extract(&factor_carrier)?;
        Ok(FactorPair {
            p: readout.p.0,
            q: readout.q.0,
        })
    }
}

fn main() -> Result<(), String> {
    if std::env::args_os().nth(1).is_some() {
        return Err(String::from("fixed_point_baked accepts no runtime arguments"));
    }

    // The shared bake file declares optional fields used by other artifacts.
    // This membrane consumes only N.
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
