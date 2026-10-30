use std::{env, fs, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-env-changed=IG_ORDER_SITE");
    println!("cargo:rerun-if-env-changed=IG_ORDER_SPEC");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=tools/generate_protein_marks.py");
    println!("cargo:rerun-if-env-changed=VOX_PRIMITIVE_PROGRAMS_FILE");
    let protein_table = PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("protein_marks.rs");
    let generated = std::process::Command::new("python3")
        .arg("tools/generate_protein_marks.py").arg(&protein_table)
        .output().expect("run source-bound protein self-mark generator");
    assert!(generated.status.success(), "protein self-mark preparation failed: {}",
        String::from_utf8_lossy(&generated.stderr));
    print!("{}", String::from_utf8_lossy(&generated.stdout));
    println!("cargo:rerun-if-env-changed=VOX_BAKED_INPUT_FILE");
    println!("cargo:rerun-if-env-changed=VOX_FACTOR_N_FILE");
    println!("cargo:rerun-if-env-changed=EXCRIBE_NUMERAL_WORD");
    println!("cargo:rerun-if-env-changed=EXCRIBE_NUMERAL_SOURCE");
    let keys = ["VOX_PHASE_MODULUS_WORD", "VOX_PHASE_BASE_WORD", "VOX_PHASE_WIDTH_WORD"];
    let names = ["BAKED_MODULUS_WORD", "BAKED_BASE_WORD", "BAKED_WIDTH_WORD"];
    let file = env::var("VOX_BAKED_INPUT_FILE").ok().map(|path| {
        println!("cargo:rerun-if-changed={path}");
        fs::read_to_string(path).expect("read build-time IMASM input file")
    });
    let lines: Vec<_> = file.as_deref().map(|s| s.lines().collect()).unwrap_or_default();
    if file.is_some() { assert!((1..=4).contains(&lines.len()), "expected modulus, optional base, optional width, optional validated-unit marker"); }
    let mut source = String::new();
    for (i, (key, name)) in keys.iter().zip(names).enumerate() {
        println!("cargo:rerun-if-env-changed={key}");
        let value = if file.is_some() { lines.get(i).map(|s| s.to_string()) } else { env::var(key).ok() };
        source.push_str(&format!("#[allow(dead_code)]\nconst {name}: Option<&str> = {:?};\n", value.as_deref()));
    }
    let base_is_unit = lines.get(3).is_some_and(|marker| *marker == "unit");
    source.push_str(&format!("#[allow(dead_code)]\nconst BAKED_BASE_IS_UNIT: bool = {base_is_unit};\n"));
    println!("cargo:rerun-if-env-changed=FACTOR_N_WORD");
    let factor_word = env::var("VOX_FACTOR_N_FILE").ok().map(|path| {
        println!("cargo:rerun-if-changed={path}");
        fs::read_to_string(path).expect("read baked IMASM factor numeral")
    }).or_else(|| env::var("FACTOR_N_WORD").ok());

    println!("cargo:rerun-if-env-changed=GFE_POLYMER_SEQUENCE_FILE");
    println!("cargo:rerun-if-env-changed=GFE_POLYMER_SEQUENCE");
    const CANONICAL_GFE_POLYMER: &str = "\
torus_encoding\t⟨𐑦𐑸𐑾𐑿𐑐𐑧𐑲𐑜⊙𐑫𐑳𐑭⟩\n\
microtubule\t⟨𐑛𐑡𐑽𐑯𐑐𐑤𐑔𐑵𐑻𐑒𐑳𐑟⟩\n\
grammar_enclosing_repair\t⟨𐑦𐑶𐑑𐑿𐑐𐑧𐑚𐑜⊙𐑫𐑳𐑴⟩\n\
zeta_sic_dual_frame\t⟨𐑦𐑸𐑑𐑹𐑐𐑧𐑚𐑜⊙𐑖𐑳𐑴⟩\n\
dual_d12_inverse\t⟨𐑦𐑸𐑽𐑹𐑐𐑧𐑔𐑵⊙𐑖𐑙𐑴⟩\n\
exact_product_terminal\t⟨𐑦𐑸𐑑𐑹𐑐𐑧𐑔𐑜⊙𐑫𐑳𐑭⟩\n\
sic_d12_gram_matrix\t⟨𐑦𐑸𐑑𐑹𐑐𐑧𐑔𐑵⊙𐑖𐑙𐑴⟩\n\
gabor_torus_dual_frame\t⟨𐑦𐑥𐑽𐑹𐑐𐑧𐑔𐑵⊙𐑖𐑳𐑭⟩\n";

    let gfe_polymer = env::var("GFE_POLYMER_SEQUENCE_FILE").ok().map(|path| {
        println!("cargo:rerun-if-changed={path}");
        fs::read_to_string(path).expect("read catalog-derived GFE monomer sequence")
    }).or_else(|| env::var("GFE_POLYMER_SEQUENCE").ok())
    .or_else(|| Some(CANONICAL_GFE_POLYMER.to_string()));

    source.push_str(&format!(
        "#[allow(dead_code)]\nconst FACTOR_N_WORD: Option<&str> = {:?};\n",
        factor_word.as_deref().map(str::trim_end)
    ));
    source.push_str(&format!(
        "#[allow(dead_code)]\nconst GFE_POLYMER_SEQUENCE: Option<&str> = {:?};\n",
        gfe_polymer.as_deref()
    ));

    println!("cargo:rerun-if-env-changed=GFE_SMALL_PRIME_MASK_FILE");
    println!("cargo:rerun-if-env-changed=GFE_SMALL_PRIME_MASK");
    let gfe_small_prime_mask = env::var("GFE_SMALL_PRIME_MASK_FILE").ok().map(|path| {
        println!("cargo:rerun-if-changed={path}");
        fs::read_to_string(path).expect("read host-prepared GFE IMASM orbit mask")
    }).or_else(|| env::var("GFE_SMALL_PRIME_MASK").ok())
    .or_else(|| {
        let cached = PathBuf::from("membranes/small_prime_orbit_mask.imasm");
        if cached.exists() {
            println!("cargo:rerun-if-changed=membranes/small_prime_orbit_mask.imasm");
            fs::read_to_string(cached).ok()
        } else {
            None
        }
    });

    source.push_str(&format!(
        "#[allow(dead_code)]\nconst GFE_SMALL_PRIME_MASK: Option<&str> = {:?};\n",
        gfe_small_prime_mask.as_deref().map(str::trim_end)
    ));
    fs::write(PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("baked_inputs.rs"), source).unwrap();
}
