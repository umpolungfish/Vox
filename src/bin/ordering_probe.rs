//! N-only ordering integration: exact carrier return and optional bounded GFE execution.
fn main() -> Result<(), String> {
    vox::ordering::cli::intercept();
    let file=std::env::args().nth(1).ok_or("expected N-only decimal or IMASM source file")?;
    let raw=std::fs::read_to_string(file).map_err(|e|e.to_string())?;
    let n=if raw.trim().starts_with('⊢') {vox::combined_factor::decode_numeral_word(raw.trim())?}
        else {num_bigint::BigUint::parse_bytes(raw.trim().as_bytes(),10).ok_or("invalid numeral")?};
    if n.bits()<200 {return Err("carrier checks require a source of at least 200 bits".into());}
    let word=vox::fixed_point_imasm::ImasmNumeralWord::checked(&vox::combined_factor::native_word(&n)).map_err(str::to_string)?;
    let tape=vox::morphism_factor::parse_numeral(word.as_str())?;
    let args:Vec<_>=std::env::args().collect();
    if let Some(position)=args.iter().position(|arg|arg=="--factor-seconds") {
        let seconds=args.get(position+1).ok_or("factor bound needs seconds")?.parse::<u64>().map_err(|e|e.to_string())?;
        if seconds==0 || seconds>60 {return Err("factor execution bound must be 1..60 seconds".into());}
        let preparation=std::time::Instant::now();
        let mut membrane=vox::geometer_factor_extractor::GeometerFactorExtractor::new(&tape)?;
        println!("factor_preparation_seconds={:.9}",preparation.elapsed().as_secs_f64());
        let trigger=std::time::Instant::now();
        while trigger.elapsed().as_secs()<seconds {
            if let Some(readout)=membrane.advance()? {
                let (quotient,remainder)=vox::morphism_factor::divmod(&tape,&readout.p);
                if !readout.product_closed || vox::morphism_factor::mul(&readout.p,&readout.q)!=tape
                    || quotient!=readout.q || !vox::morphism_factor::zero(&remainder)
                    || readout.p.len()==1 || readout.q.len()==1 {return Err("factor return failed exact proper product closure".into());}
                println!("factor_status=closed producer={} p={} q={} product_verified=true trigger_to_factor_seconds={:.9}",
                    readout.producer,vox::morphism_factor::dec_of(&readout.p),vox::morphism_factor::dec_of(&readout.q),trigger.elapsed().as_secs_f64());
                return Ok(());
            }
        }
        println!("factor_status=held trigger_seconds={:.9} {}",trigger.elapsed().as_secs_f64(),membrane.progress_report());
        return Ok(());
    }
    let braid=vox::pwoat_torus::BitBraid::torus_twist(n.bits() as usize);
    let transformed=braid.apply_word(&word,n.bits() as usize)?;
    let recovered=braid.return_word(&transformed,n.bits() as usize)?;
    if recovered!=word {return Err("selected bit braid failed exact return".into());}
    let circuit=vox::geometer_factor_extractor::prepare_source_circuit_braid(&tape,braid.axis_permutation())?;
    let ports=vox::glut_system::GlutComponentProgram::compile()?.component_bindings();
    println!("source_bits={} bit_return=true bit_crossings={} inverse_residual={:.17e} axis_order={:?} ports={:?} operator={:?}",
        n.bits(),braid.generators.len(),circuit.inverse_recovery_residual,circuit.axis_order,ports,circuit.operator);
    Ok(())
}
