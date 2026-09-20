//! Each build embeds its case as IMASM numerals. There is no runtime input API.
extern crate alloc;
#[path = "../membrane_complex.rs"] mod membrane_complex;
#[path = "../fde_shor_membrane.rs"] mod fde_shor_membrane;
#[path = "../shor_qft.rs"] mod shor_qft;
#[path = "../phase_word.rs"] mod phase_word;

use vox::morphism_factor::{dec_of, parse_numeral};

const BASE: Option<&str> = option_env!("VOX_PHASE_BASE_WORD");
const MODULUS: Option<&str> = option_env!("VOX_PHASE_MODULUS_WORD");
const WIDTH: Option<&str> = option_env!("VOX_PHASE_WIDTH_WORD");
const EXECUTION: Option<&str> = option_env!("VOX_PHASE_EXECUTION_WORD");

fn main() -> Result<(), String> {
    let a = parse_numeral(BASE.ok_or("build with an embedded base word")?)?;
    let n = parse_numeral(MODULUS.ok_or("build with an embedded modulus word")?)?;
    let width = parse_numeral(WIDTH.ok_or("build with an embedded width word")?)?;
    let qubits = dec_of(&width).parse::<usize>().map_err(|e| e.to_string())?;
    let branch = shor_qft::SparsePhaseBranch::from_modulus(&a, &n, qubits)?;
    let steps = branch.modular_steps;
    let size = branch.register_size;
    let readout = phase_word::execute(EXECUTION.ok_or("bake the blueprint execution word")?, &branch)?;
    println!("register {} surviving {} cleared {} restored {} exposed {}",
        readout.register, readout.surviving.len(), readout.cleared, readout.restored, readout.exposed);
    println!("base {}\nmodulus {}\nqubits {qubits}", dec_of(&a), dec_of(&n));
    let Some(deposit) = readout.surviving.first() else {
        println!("unresolved: no surviving phase observation");
        return Ok(());
    };
    let restored = &deposit.observation;
    println!("modular_steps {steps}\nregister_size {size}\npopulation {}", restored.positions.len());
    println!("banked_restored {}", readout.restored > 0 && readout.exposed == 0);
    for k in [0, 1, size / 2, size - 1] {
        println!("probability {k} {:.17}", restored.probability(k)?);
    }
    Ok(())
}
