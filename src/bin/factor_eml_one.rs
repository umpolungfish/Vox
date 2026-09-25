//! Run the nested phase/EML factoring carrier over one baked IMASM numeral.

include!(concat!(env!("OUT_DIR"), "/baked_inputs.rs"));

const EML_NINE: &str = "⊢∈≻⊤⊥≻≺∈⊤⊥⊞≺⊙∋⊡∋∈⊤≺⊥∋∈⊤⊞⊥∋∈≻⊤≺⊥⊞⋈∋∈⊤≺⊞⊥∋∈⊙⊞⋈∋∈⊙≺⋈∋∈≻⋈⊤⊥∋∈⊙≻⋈∋∈⊙≻⊤≺⊥⋈∋⊙⊡⊣";

fn main() {
    let (Some(source), Some(phase_base)) = (BAKED_MODULUS_WORD, BAKED_BASE_WORD) else {
        eprintln!("build this membrane with baked IMASM modulus and phase-base words");
        std::process::exit(2);
    };
    match vox::morphism_factor::factor_with_phase_base(EML_NINE, source, phase_base) {
        Ok(factor) => println!("{factor}"),
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(2);
        }
    }
}
