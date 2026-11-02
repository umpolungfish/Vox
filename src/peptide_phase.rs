//! Fourier/SIC return of the source-bound atomic walk coefficient carrier.
//! All source, target, bank, evidence-sector and clock addresses remain carried.
use alloc::{collections::BTreeMap,string::String,vec::Vec};
use crate::sic::{carrier::recover_ray,frame::{Complex,Operator,Sic,SicCoordinates},wh};

fn flat(values:&[Complex])->Vec<f64>{values.iter().flat_map(|v|[v.re,v.im]).collect()}

pub fn prepare(reentry:&serde_json::Value)->Result<serde_json::Value,String>{
    let period=reentry["derived_B_clock_period"].as_u64().ok_or("source clock has no period")? as usize;
    if period==0{return Err("empty source phase clock".into());}
    let mut groups=Vec::new();let mut norm=0.0;
    for bank in reentry["executed_joint_atom_clock_banks"].as_array().ok_or("missing joint source banks")?{
        let tick=bank["tick"].as_u64().ok_or("invalid bank clock")? as usize;
        let mask=if tick==1{15}else{
            reentry["single_pass_degree_masks"][alloc::format!("{tick}")].as_u64().unwrap_or(0) as u8};
        for entry in bank["joint_atom_clock_entries"].as_array().ok_or("missing joint coefficients")?{
            let source=entry[0].as_u64().ok_or("invalid source atom")? as usize;
            let target=entry[1].as_u64().ok_or("invalid target atom")? as usize;
            let phase=entry[2].as_u64().ok_or("invalid source phase")? as usize;
            let count=entry[3].as_str().ok_or("invalid source coefficient")?.parse::<u64>().map_err(|_|"source coefficient exceeds exact integer readout")?;
            if count>1u64<<53{return Err("source phase preparation requires an exact floating integer boundary".into());}
            // Sector 0 retains the complete source basis, including paths with
            // no current native evidence. Sectors 1..4 carry N,T,F,B separately.
            for sector in 0..5{
                if sector!=0&&mask&(1<<(sector-1))==0{continue;}
                groups.push((tick,source,target,sector,phase,count));norm+=(count as f64)*(count as f64);
            }
        }
    }
    if groups.is_empty()||!norm.is_finite()||norm==0.0{return Err("empty source complex coefficient carrier".into());}
    let scale=1.0/libm::sqrt(norm);let fourier_scale=1.0/libm::sqrt(period as f64);
    let mut transformed=Vec::with_capacity(groups.len()*period);
    let mut anchor=0;let mut maximum=0.0;
    for &(tick,source,target,sector,phase,count) in &groups{
        let _=(tick,source,target,sector);
        let mut ray=alloc::vec![Complex::default();period];ray[phase]=Complex::new(count as f64*scale,0.0);
        let out=wh::fft_positive(&ray).map_err(|e|alloc::format!("{e}"))?;
        if count as f64>maximum{maximum=count as f64;anchor=transformed.len()+phase;}
        transformed.extend(out.into_iter().map(|z|z.scale(fourier_scale)));
    }
    let transformed_norm: f64=transformed.iter().map(|z|z.abs2()).sum();
    let norm_error=(transformed_norm-1.0).abs();
    let norm_tolerance=256.0*f64::EPSILON*(transformed.len()+1) as f64;
    if norm_error>norm_tolerance{return Err("source Fourier action lost its normalization".into());}
    let frame=wh::ququart_frame();
    let returned=recover_ray(&frame,&transformed,anchor,None).map_err(|e|alloc::format!("{e}"))?;
    let recovered=returned.recovered.iter().map(|z|*z*returned.gauge.conj()).collect::<Vec<_>>();

    // A phase-only change on an actual returned source coordinate keeps every
    // population and the cover fixed. Fresh masses recover it; old masses fail.
    let changed_index=(0..transformed.len()).rev().find(|&i|i!=anchor&&transformed[i].abs2()>0.0).ok_or("no source coherence control coordinate")?;
    let held_frame=returned.frames.iter().find(|f|f.indices.contains(&changed_index)).ok_or("source coordinate lost its enclosing SIC frame")?;
    let mut control=[Complex::default();4];control[0]=transformed[anchor];
    for (slot,&i) in held_frame.indices.iter().enumerate(){control[slot+1]=transformed[i];}
    let slot=held_frame.indices.iter().position(|&i|i==changed_index).unwrap()+1;
    let population=control[slot].abs2();control[slot]=control[slot]*Complex::new(0.0,1.0);
    let control_gram=Operator::projector(&control).map_err(|e|alloc::format!("{e}"))?;
    let fresh=frame.fuse(&frame.split(&control_gram).map_err(|e|alloc::format!("{e}"))?).map_err(|e|alloc::format!("{e}"))?;
    let stale=frame.fuse(&SicCoordinates{dimension:4,values:held_frame.masses.clone()}).map_err(|e|alloc::format!("{e}"))?;
    let fresh_error=fresh.distance(&control_gram).map_err(|e|alloc::format!("{e}"))?;
    let stale_error=stale.distance(&control_gram).map_err(|e|alloc::format!("{e}"))?;
    if fresh_error>returned.tolerance||stale_error<=returned.tolerance{return Err("source coherence control did not distinguish stale masses".into());}

    // Continue from the SIC-returned values. The source bond operation changes
    // the atom index, while the same step advances the carried clock phase.
    let mut neighbors=BTreeMap::<usize,Vec<usize>>::new();
    for edge in reentry["source_letters"].as_array().ok_or("missing atomic source letters")?{
        neighbors.entry(edge[1].as_u64().unwrap() as usize).or_default().push(edge[2].as_u64().unwrap() as usize);
    }
    let mut next=BTreeMap::<(usize,usize,usize,usize,usize),Complex>::new();
    let mut inverse_error=0.0f64;
    for (group,&(tick,source,target,sector,phase,count)) in groups.iter().enumerate(){
        let ray=&recovered[group*period..(group+1)*period];
        let inverse=wh::fft_positive(&ray.iter().map(|z|z.conj()).collect::<Vec<_>>()).map_err(|e|alloc::format!("{e}"))?;
        for clock in 0..period{
            let actual=inverse[clock].conj().scale(fourier_scale);
            let expected=Complex::new(if clock==phase{count as f64*scale}else{0.0},0.0);
            inverse_error=inverse_error.max(libm::sqrt((actual-expected).abs2()));
        }
        for frequency in 0..period{
            let amplitude=ray[frequency]*wh::turn_phase(frequency as u128,period as u128);
            for &neighbor in neighbors.get(&target).ok_or("source atom has no covalent continuation")?{
                let cell=next.entry((tick,source,neighbor,sector,frequency)).or_default();*cell=*cell+amplitude;
            }
        }
    }
    if inverse_error>libm::sqrt(returned.tolerance){return Err("SIC-returned clock inverse lost its source coefficients".into());}
    let continued=next.iter().map(|(&(bank,source,target,sector,clock),z)|
        serde_json::json!([bank,source,target,sector,clock,z.re,z.im])).collect::<Vec<_>>();
    let continued_norm: f64=next.values().map(|z|z.abs2()).sum();
    Ok(serde_json::json!({"operation":"native source-clock FFT, canonical shared-anchor SIC and coupled atomic continuation",
        "coordinate_groups":groups,"clock_lanes_per_group":period,
        "group_columns":["source_bank_tick","source_atom","target_atom","sector","initial_clock_phase","exact_integer_coefficient"],
        "sectors":["held source basis","N","T","F","B"],"normalization_scale":scale,
        "fourier_engine":"G-mOMonadOS/src/sic/wh.rs::fft_positive (positive sign, normalized here)",
        "source_norm_squared_before_normalization":norm,"fourier_norm_squared":transformed_norm,
        "fourier_norm_residual":norm_error,"source_anchor":anchor,"returned_gauge":[returned.gauge.re,returned.gauge.im],
        "returned_coefficients_re_im":flat(&recovered),
        "sic_frame_projectors":frame.projectors().iter().map(|p|flat(p.values())).collect::<Vec<_>>(),
        "sic_frame_effects":frame.effects().iter().map(|p|flat(p.values())).collect::<Vec<_>>(),
        "sic_frame_duals":frame.duals().iter().map(|p|flat(p.values())).collect::<Vec<_>>(),
        "sic_returns":returned.frames.iter().map(|f|serde_json::json!([f.indices,flat(&f.masses),flat(&f.signed_gram_residuals)])).collect::<Vec<_>>(),
        "sic_return_columns":["group_coordinate_indices","projector_masses_re_im","signed_gram_residuals_re_im"],
        "signed_amplitude_residuals_re_im":flat(&returned.signed_amplitude_residuals),
        "full_gram_bound":returned.full_gram_bound,"source_tolerance":returned.tolerance,
        "inverse_clock_maximum_amplitude_residual":inverse_error,
        "coherence_control":{"coordinate":changed_index,"same_cover":held_frame.indices,
            "population_before":population,"population_after":control[slot].abs2(),
            "fresh_gram_error":fresh_error,"stale_gram_error":stale_error,"stale_masses_rejected":true},
        "continued_from_sic_return":true,"coupled_atom_clock_steps_after_sic":1,
        "continuation_entry_columns":["source_bank_tick","source_atom","target_atom","sector","Fourier_clock_lane","real","imaginary"],
        "continued_entries":continued,"continued_norm_squared":continued_norm,
        "scope":"source operator-coefficient carrier; spatial conformation remains unresolved"}))
}
