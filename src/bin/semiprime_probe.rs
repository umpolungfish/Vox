#![deny(warnings)]
extern crate alloc;
#[path = "../membrane_complex.rs"] mod membrane_complex;
#[path = "../fde_shor_membrane.rs"] mod fde_shor_membrane;
#[path = "../shor_qft.rs"] mod shor_qft;
use std::time::Instant;
use vox::dialectic_reentry::{Descent, DialecticObject};
use vox::morphism_factor::{parse_numeral, dec_of};
include!(concat!(env!("OUT_DIR"), "/baked_inputs.rs"));

fn main() {
    #[cfg(feature = "hosted")]
    ::vox::ordering::cli::intercept();
    let mode = option_env!("VOX_PROBE_MODE").expect("bake the producer at build time");
    let n = parse_numeral(BAKED_MODULUS_WORD.expect("bake the IMASM modulus at build time")).unwrap();
    let a = if mode == "synthesis" || mode == "dialectic-lean" { vox::morphism_factor::two() }
        else { parse_numeral(BAKED_BASE_WORD.expect("bake the IMASM base at build time")).unwrap() };
    let start = Instant::now();
    if mode == "synthesis" {
        let mut resident = vox::factor_synthesis::FactorSynthesis::new(&n).unwrap();
        let mut last_report = Instant::now();
        loop {
            if let Some(result) = resident.advance_observed(|stats| {
                if last_report.elapsed().as_secs() >= 5 {
                    eprintln!("synthesis seconds={:.3} stage={} phase={} square={} rows={} columns={} frontier={} decisions={}",
                        start.elapsed().as_secs_f64(), stats.diagnostic_stage, stats.phase_stage,
                        stats.square_advances, stats.phase_rows, stats.phase_columns,
                        stats.frontier, stats.correlation_decisions);
                    last_report = Instant::now();
                }
            }).unwrap() {
                println!("factor {}\nfactor {}", dec_of(&result.p), dec_of(&result.q));
                println!("producer {} turns {}", result.producer, result.turns);
                break;
            }
        }
    } else if mode == "braid" {
        eprintln!("stage braid order acquisition, base=2");
        let braid = vox::shor_braid::shor_braid_carrier(&a, &n).unwrap();
        let levels = braid.levels;
        eprintln!("stage winding readout; levels={levels}");
        let r = braid.read_order().unwrap();
        println!("order {}", dec_of(&r));
        let (p,q) = vox::shor_braid::factor_close_public(&a, &n, &r).unwrap();
        println!("factor {}\nfactor {}", dec_of(&p), dec_of(&q));
    } else if mode == "sparse" {
        let qubits: usize = dec_of(&parse_numeral(BAKED_WIDTH_WORD.expect("bake register width")).unwrap()).parse().unwrap();
        let branch = shor_qft::SparsePhaseBranch::from_modulus(&a, &n, qubits).unwrap();
        println!("positions {} register_size {} modular_steps {} stored_position_bytes {}",
            branch.positions.len(), branch.register_size, branch.modular_steps,
            branch.positions.capacity()*std::mem::size_of::<usize>());
        for k in [0,1,branch.register_size/2,branch.register_size-1] {
            println!("probability {k} {}",branch.probability(k).unwrap());
        }
    } else if mode == "observed" {
        let qubits: usize = dec_of(&parse_numeral(BAKED_WIDTH_WORD.expect("bake register width")).unwrap()).parse().unwrap();
        let reg = shor_qft::ObservedPhaseRegister::from_modulus(a, n.clone(), qubits).unwrap();
        println!("branch_population {}", reg.branch_population);
        let Some(order) = reg.extract_order() else {
            println!("unresolved phase register");
            return;
        };
        println!("order {}", dec_of(&order));
        let (p,q) = shor_qft::factor_close_public(&reg.a, &n, &order).unwrap();
        println!("factor {}\nfactor {}", dec_of(&p), dec_of(&q));
    } else if mode == "symbolic" {
        eprintln!("stage modular evolution and observed QFT, base=2");
        let order = shor_qft::observe_order(a.clone(), n.clone()).unwrap();
        println!("order {}", dec_of(&order));
        let (p,q) = shor_qft::factor_close_public(&a, &n, &order).unwrap();
        println!("factor {}\nfactor {}", dec_of(&p), dec_of(&q));
    } else if mode == "phase" {
        eprintln!("stage nested phase factor execution");
        let word = vox::morphism_factor::emit_numeral(&n);
        let result = vox::morphism_factor::factor(&word).expect("phase factor");
        let p = vox::morphism_factor::parse_numeral(&result).unwrap();
        let (q, rem) = vox::morphism_factor::divmod(&n, &p);
        assert!(vox::morphism_factor::zero(&rem));
        println!("factor {}\nfactor {}", dec_of(&p), dec_of(&q));
    } else if mode == "smart" {
        eprintln!("stage general unbounded tape factor route");
        let (factors, route) = vox::morphism_factor::smart_factor(&n);
        if factors.len() < 2 { panic!("general route returned no composite factor"); }
        let product = factors.iter().fold(vox::morphism_factor::one(), |acc, f| vox::morphism_factor::mul(&acc, f));
        assert_eq!(product, n, "general route product does not reconstruct N");
        for factor in factors { println!("factor {}", dec_of(&factor)); }
        eprintln!("route {route}");
    } else if mode == "resident" {
        let mut resident = vox::factorization_31_membrane::UnboundedResident::new(n);
        resident.run();
        assert!(resident.boundary_ok && resident.sidearm_round_trip);
        println!("factor {}", dec_of(&resident.remainder));
        for p in resident.factors { println!("factor {}", dec_of(&p)); }
    } else if mode == "dialectic-lean" {
        let preparation_start = Instant::now();
        let mut resident = vox::dialectic_lean::ResidentDialectic::new(n.clone()).unwrap();
        let preparation = preparation_start.elapsed().as_secs_f64();
        eprintln!("preparation_seconds {preparation:.9}");
        let trigger = Instant::now();
        let mut last_report = Instant::now();
        while !resident.advance().unwrap_or_else(|error| {
            if let Some(frame) = &resident.scan.last_frame {
                eprintln!("failed_frame source={} multiplier={} a={} residual={} cell={} lattice={:?}",
                    dec_of(&frame.source), dec_of(&frame.multiplier), dec_of(&frame.a),
                    dec_of(&frame.residual), dec_of(&frame.lattice_cell), frame.lattice);
            }
            panic!("{error}")
        }) {
            if last_report.elapsed().as_secs() >= 5 {
                let object = resident.object().unwrap();
                eprintln!("dialectic-lean seconds={:.6} descents={} contractions={} temporal_ticks={} information_joins={} reversed_descents={} phase={} evidence={} boundary={} cells={} square_tests={} rejected={} residual_advances={}",
                    trigger.elapsed().as_secs_f64(), resident.operations.descents,
                    resident.operations.contractions, resident.operations.temporal_ticks,
                    resident.operations.information_joins, resident.operations.reversed_descents,
                    resident.phase, resident.evidence,
                    dec_of(object.boundary()), resident.scan.cells,
                    resident.scan.square_tests, resident.scan.rejected_cells,
                    resident.scan.residual_advances);
                last_report = Instant::now();
            }
        }
        let readout = trigger.elapsed().as_secs_f64();
        let closure = resident.closure().unwrap();
        let p = &closure.carrier.p;
        let (q, remainder) = vox::morphism_factor::divmod(&n, p);
        assert!(vox::morphism_factor::zero(&remainder));
        assert_eq!(vox::morphism_factor::mul(p, &q), n);
        assert_eq!(q, closure.carrier.q);
        let verified = trigger.elapsed().as_secs_f64();
        println!("factor {}\nfactor {}", dec_of(p), dec_of(&q));
        println!("producer dialectic-resident lattice {:?}", vox::dialectic_reentry::decode_imasm_execution(closure.word()).unwrap().lattice());
        println!("preparation_seconds {preparation:.9}\ntrigger_to_readout_seconds {readout:.9}\ntrigger_to_verified_factor_seconds {verified:.9}");
        println!("descents {} contractions {} temporal_ticks {} information_joins {} reversed_descents {} cells {} square_tests {} rejected {} residual_advances {} phase {} evidence {}",
            resident.operations.descents, resident.operations.contractions,
            resident.operations.temporal_ticks, resident.operations.information_joins,
            resident.operations.reversed_descents, resident.scan.cells, resident.scan.square_tests,
            resident.scan.rejected_cells, resident.scan.residual_advances, resident.phase, resident.evidence);
        let frame = resident.scan.last_frame.as_ref().unwrap();
        println!("residual_frame source={} multiplier={} a={} residual={} cell={} lattice={:?}",
            dec_of(&frame.source), dec_of(&frame.multiplier), dec_of(&frame.a),
            dec_of(&frame.residual), dec_of(&frame.lattice_cell), frame.lattice);
        println!("executed_word {}", closure.word().iter().collect::<String>());
    } else if mode == "dialectic" {
        let mut object = DialecticObject::new(n).unwrap();
        let mut steps = 0;
        loop {
            steps += 1;
            if option_env!("VOX_PROBE_TRACE").is_some() {
                let judgment = object.judge_current_imscription();
                eprintln!("diagnostic: validation={:?} judgment={}",object.validate(),judgment.four());
                let delta = vox::morphism_factor::sub(&vox::morphism_factor::mul(object.boundary(),object.boundary()),&object.n);
                let root = vox::morphism_factor::isqrt(&delta);
                eprintln!("diagnostic: boundary={} delta={} sqrt_delta={}",dec_of(object.boundary()),dec_of(&delta),dec_of(&root));
                let mut closed = object.word().to_vec();
                closed.insert(closed.len()-2,vox::vox::FFUSE);
                eprintln!("diagnostic: closed_decode={:?}",vox::dialectic_reentry::decode_imasm_execution(&closed));
                if let Some(witness) = judgment.witness() {
                    eprintln!("diagnostic: p={} q={}",dec_of(&witness.p),dec_of(&witness.q));
                }
            }
            match object.descend() {
                Descent::B(next) => object = next,
                Descent::T(closed) => {
                    println!("factor {}", dec_of(&closed.carrier.p));
                    println!("factor {}", dec_of(&closed.carrier.q));
                    println!("descents {steps}");
                    break;
                }
                Descent::N(_) => panic!("N before closure"),
                Descent::F => panic!("F before closure"),
            }
        }
    } else { panic!("unknown baked producer"); }
    println!("seconds {}", start.elapsed().as_secs_f64());
}
