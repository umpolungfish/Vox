//! Source-bound Geometer Factor Extractor membrane.
//!
//! `gfe_factor_one.sh` places the canonical N-only IMASM numeral into the
//! Cargo image. This executable accepts no number at runtime: its first live
//! operation is the prepared GFE resident, and its terminal is the exact
//! factor/product readout.
#![deny(warnings)]

include!(concat!(env!("OUT_DIR"), "/baked_inputs.rs"));

const EXPECTED_GFE_MONOMERS: [&str; 8] = [
    "torus_encoding",
    "microtubule",
    "grammar_enclosing_repair",
    "zeta_sic_dual_frame",
    "dual_d12_inverse",
    "exact_product_terminal",
    "sic_d12_gram_matrix",
    "gabor_torus_dual_frame",
];

fn run() -> Result<(), String> {
    let preparation_started = std::time::Instant::now();
    let word = FACTOR_N_WORD.ok_or("GFE source numeral was not baked into this image")?;
    let polymer = GFE_POLYMER_SEQUENCE
        .ok_or("catalog-derived GFE polymer was not baked into this image")?;
    let monomer_rows: Vec<_> = polymer.lines().collect();
    if monomer_rows.len() != EXPECTED_GFE_MONOMERS.len()
        || monomer_rows.iter().zip(EXPECTED_GFE_MONOMERS).any(|(row, expected)| {
            row.split_once('\t').map(|(name, _)| name) != Some(expected)
        })
    {
        return Err("baked GFE polymer sequence does not match the selected construction".into());
    }
    let mask_word = GFE_SMALL_PRIME_MASK
        .ok_or("compiled GFE small-prime orbit mask is absent")?;
    let small_prime_orbit_mask =
        vox::geometer_factor_extractor::SmallPrimeOrbitMask::from_imasm_word(mask_word)?;
    let mask_word_cells = small_prime_orbit_mask.word().as_str().chars().count();
    let mask_marked_cells = small_prime_orbit_mask.marked_cells();
    let source = vox::morphism_factor::parse_numeral(word)?;
    let mut membrane = vox::geometer_factor_extractor::GeometerFactorExtractor::new_with_orbit_mask(
        &source, small_prime_orbit_mask,
    )?;
    let circuit = membrane.circuit_braid_state()
        .ok_or("GFE preparation did not retain its circuit state")?;
    if !circuit.priority_order_changed && vox::ordering::setting("IG_ORDER_SITE").is_none() {
        return Err("GFE circuit braid did not transform the prepared resident priority order".into());
    }
    let preparation_seconds = preparation_started.elapsed().as_secs_f64();
    println!("preparation_boundary=runtime preparation_seconds={preparation_seconds:.9}");
    println!(
        "prepared=gfe source_bits={} source_embedded=true carrier=telechelic-open polymer_units={} polymer_sequence={} small_prime_orbit_mask_imasm_bits={} small_prime_orbit_mask_marked_cells={} circuit={} circuit_braid_word={:?} circuit_source_phase_bin={} circuit_input_label={} circuit_output_weights={:?} circuit_unitary_residual={:.3e} circuit_norm_residual={:.3e} torus_axis_order={:?} circuit_axis_order={:?} circuit_priority_order_changed={} circuit_applied_to_resident=true",
        source.len(),
        monomer_rows.len(),
        monomer_rows.iter().filter_map(|row| row.split_once('\t').map(|(name, _)| name))
            .collect::<Vec<_>>().join("⋈"),
        mask_word_cells,
        mask_marked_cells,
        circuit.circuit,
        circuit.braid_word,
        circuit.source_phase_bin,
        circuit.input_label,
        circuit.output_weights,
        circuit.unitary_residual,
        circuit.norm_residual,
        circuit.torus_axis_order,
        circuit.axis_order,
        circuit.priority_order_changed,
    );

    let started = std::time::Instant::now();
    let timeout_seconds = std::env::var("GFE_TIMEOUT_SECONDS")
        .ok()
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(300.0);
    let mut turns = 0u64;
    let mut last_resident_observation = String::new();
    let mut last_observation_report = started;
    loop {
        turns += 1;
        let advance = membrane.advance_observed(|stats| {
            last_resident_observation = format!("{stats:?}");
            if last_observation_report.elapsed().as_secs_f64() >= 1.0 {
                eprintln!("gfe resident trigger_seconds={:.6} {last_resident_observation}",
                    started.elapsed().as_secs_f64());
                last_observation_report = std::time::Instant::now();
            }
        });
        let advance = advance.map_err(|error| format!(
            "{error}; preparation_boundary=runtime preparation_seconds={preparation_seconds:.9} trigger_seconds={:.9} last_resident={last_resident_observation} architecture={}",
            started.elapsed().as_secs_f64(), membrane.progress_report(),
        ))?;
        if let Some(readout) = advance {
            let trigger_to_readout_seconds = started.elapsed().as_secs_f64();
            let p = vox::morphism_factor::dec_of(&readout.p);
            let q = vox::morphism_factor::dec_of(&readout.q);
            let product = vox::morphism_factor::mul(&readout.p, &readout.q);
            let (derived_quotient, remainder) =
                vox::morphism_factor::divmod(&source, &readout.p);
            if !readout.final_readout_fixed
                || !readout.product_closed
                || product != source
                || !vox::morphism_factor::zero(&remainder)
                || derived_quotient != readout.q
                || p == "1"
                || q == "1"
                || readout.circuit_braid.as_ref().is_none_or(|state|
                    state.circuit != "QFT1+HTSX"
                    || state.braid_word.len() != 59
                    || state.source_phase_bin != vox::geometer_factor_extractor::source_phase_bin(&source)
                    || state.norm_residual > 1e-12
                    || (!state.priority_order_changed && vox::ordering::setting("IG_ORDER_SITE").is_none())
                    || state.axis_order.iter().copied().collect::<std::collections::BTreeSet<_>>().len() != 12)
            {
                return Err("GFE terminal failed circuit-state or exact source/product closure".into());
            }
            let factor_word = vox::morphism_factor::emit_numeral(&readout.p);
            let cofactor_word = vox::morphism_factor::emit_numeral(&readout.q);
            let product_word = vox::morphism_factor::emit_numeral(&product);
            let source_word = vox::morphism_factor::emit_numeral(&source);
            let trigger_to_verified_factor_seconds = started.elapsed().as_secs_f64();
            println!("preparation_boundary=runtime preparation_seconds={preparation_seconds:.9} trigger_to_readout_seconds={trigger_to_readout_seconds:.9} trigger_to_verified_factor_seconds={trigger_to_verified_factor_seconds:.9} zero_remainder=true derived_quotient_matches=true");
            println!(
                "producer={} factor_word={} cofactor_word={} product_word={} source_word={} exact_imasm_product=true product_closed={} final_readout_fixed={} turns={} torus_monad_laws={} resident_monad_laws={} orbit_verified={} circuit={} circuit_braid_word={:?} circuit_source_phase_bin={} circuit_input_label={} circuit_output_weights={:?} circuit_unitary_residual={:.3e} circuit_norm_residual={:.3e} circuit_axis_order={:?} elapsed_seconds={:.6}",
                readout.producer,
                factor_word,
                cofactor_word,
                product_word,
                source_word,
                readout.product_closed,
                readout.final_readout_fixed,
                turns,
                readout.torus_powerset_monad_laws_verified,
                readout.gfe_powerset_monad_laws_verified,
                readout.torus_full_orbit_verified,
                readout.circuit_braid.as_ref().unwrap().circuit,
                readout.circuit_braid.as_ref().unwrap().braid_word,
                readout.circuit_braid.as_ref().unwrap().source_phase_bin,
                readout.circuit_braid.as_ref().unwrap().input_label,
                readout.circuit_braid.as_ref().unwrap().output_weights,
                readout.circuit_braid.as_ref().unwrap().unitary_residual,
                readout.circuit_braid.as_ref().unwrap().norm_residual,
                readout.circuit_braid.as_ref().unwrap().axis_order,
                started.elapsed().as_secs_f64(),
            );
            return Ok(());
        }
        let elapsed = started.elapsed().as_secs_f64();
        if elapsed > timeout_seconds {
            return Err(format!(
                "GFE execution limit reached ({timeout_seconds:.1}s), last resident: {last_resident_observation}"
            ));
        }
        if turns % 10 == 0 {
            eprintln!(
                "gfe progress turns={} elapsed_seconds={:.3} resident={} architecture={}",
                turns,
                elapsed,
                last_resident_observation,
                membrane.progress_report(),
            );
        }
    }
}

fn main() {
    #[cfg(feature = "hosted")]
    ::vox::ordering::cli::intercept();
    if std::env::args_os().len() != 1 {
        eprintln!("this GFE membrane has a compile-time N and accepts no arguments");
        std::process::exit(2);
    }
    if let Err(error) = run() {
        eprintln!("gfe-factor: {error}");
        std::process::exit(2);
    }
}
