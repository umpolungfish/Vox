//! Each build embeds its case as IMASM numerals. There is no runtime input API.
extern crate alloc;
#[path = "../membrane_complex.rs"] mod membrane_complex;
#[path = "../fde_shor_membrane.rs"] mod fde_shor_membrane;
#[path = "../shor_qft.rs"] mod shor_qft;
#[path = "../phase_word.rs"] mod phase_word;

use vox::phase_partners;
use vox::morphism_factor::{dec_of, parse_numeral};

include!(concat!(env!("OUT_DIR"), "/baked_inputs.rs"));
const BASE: Option<&str> = BAKED_BASE_WORD;
const MODULUS: Option<&str> = BAKED_MODULUS_WORD;
const WIDTH: Option<&str> = BAKED_WIDTH_WORD;
const EXECUTION: Option<&str> = option_env!("VOX_PHASE_EXECUTION_WORD");

fn main() -> Result<(), String> {
    let a = parse_numeral(BASE.ok_or("build with an embedded base word")?)?;
    let n = parse_numeral(MODULUS.ok_or("build with an embedded modulus word")?)?;
    let width = parse_numeral(WIDTH.ok_or("build with an embedded width word")?)?;
    let qubits = dec_of(&width).parse::<usize>().map_err(|e| e.to_string())?;
    if option_env!("VOX_PHASE_ACQUISITION") == Some("hsoa") {
        let register = shor_qft::ObservedPhaseRegister::from_modulus(a.clone(), n.clone(), qubits)?;
        let transported = phase_word::execute(EXECUTION.ok_or("bake execution word")?, &register.probabilities)?;
        let Some(deposit) = transported.surviving.first() else {
            println!("unresolved: no surviving spectrum");
            return Ok(());
        };
        assert_eq!(deposit.observation, register.probabilities);
        println!("base {}\nmodulus {}\npopulation {}", dec_of(&a), dec_of(&n), register.branch_population);
        for (k, p) in deposit.observation.iter().enumerate() { println!("probability {k} {p:.17}"); }
        if let Some(r) = register.extract_order() {
            println!("winding {}", dec_of(&r));
            // Close from the recovered readout, with no reference order input.
            if let Ok((p,q)) = vox::shor_braid::factor_close_public(&a, &n, &r) {
                println!("factor {}\nfactor {}", dec_of(&p), dec_of(&q));
            }
        } else { println!("unresolved: spectrum supplied no certified return"); }
        return Ok(());
    }
    if option_env!("VOX_PHASE_ACQUISITION") == Some("partners") {
        let mut partners = phase_partners::Partners::new(a.clone(), n.clone())?;
        println!("base {}\nmodulus {}\nobservations {qubits}", dec_of(&a), dec_of(&n));
        for _ in 0..qubits {
            if let Some(relation) = partners.observe()? {
                let readout = phase_word::execute(EXECUTION.ok_or("bake execution word")?, &relation)?;
                for d in &readout.surviving { assert!(d.observation.verify(&a, &n)); }
                if let Some(d) = readout.surviving.first() {
                    println!("return_exponent {}", dec_of(&d.observation.return_exponent));
                    if let Ok((p,q)) = vox::shor_braid::factor_close_public(&a, &n, &d.observation.return_exponent) {
                        println!("factor {}\nfactor {}", dec_of(&p), dec_of(&q));
                    }
                    println!("squarings {} stored_residues {}", partners.squarings, partners.stored_residues());
                    return Ok(());
                }
            }
        }
        println!("unresolved: no retained return relation");
        println!("squarings {} stored_residues {}", partners.squarings, partners.stored_residues());
        return Ok(());
    }
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
