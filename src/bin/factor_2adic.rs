//! Runtime for one compiled phase-winding and bit-register membrane.
//!
//! Base and modulus arrive as baked IMASM numeral tapes. Dyadic phase
//! observations produce a return relation, which is closed by nesting the
//! product shell around the prefix fold and then reversing that nesting.

#[path = "../phase_partners.rs"] mod phase_partners;
#[path = "../phase_word.rs"] mod phase_word;

use vox::morphism_factor::parse_numeral;

include!(concat!(env!("OUT_DIR"), "/baked_inputs.rs"));

const EXTRACT_WORD: &str = "⊢∈≻⊤⋈⊙≺⊥⊞∋⊡⋈⊙⊣";

/// FDE value for the proposition that the candidate pair closes on N.
/// Prefix-first and product-first are independent support channels: agreement
/// yields T/F, disagreement is retained as B, and an unevaluated pair is N.
fn fde_closure_value(prefix_first: Option<bool>, product_first: Option<bool>) -> char {
    let mut true_support = false;
    let mut false_support = false;
    for evidence in [prefix_first, product_first].into_iter().flatten() {
        if evidence { true_support = true; } else { false_support = true; }
    }
    match (true_support, false_support) {
        (false, false) => 'N',
        (true, false) => 'T',
        (false, true) => 'F',
        (true, true) => 'B',
    }
}

fn append_frames(output: &mut String, label: &str, tape: &[char]) {
    for window in 2..=8 {
        let remainder = tape.len() % window;
        let tail_len = if remainder == 0 { window } else { remainder };
        use core::fmt::Write;
        let token_width = usize::from(window > 4) + 1;
        write!(output, "frame {label} window={} tail={} bits={} groups-hex=",
            window, tail_len, tape.len()).unwrap();
        for group in tape.chunks(window) {
            let mut state = 0u8;
            for (index, &mark) in group.iter().enumerate() {
                if mark == vox::vox::EVALF { state |= 1 << index; }
            }
            write!(output, "{state:0token_width$x}").unwrap();
        }
        output.push('\n');
    }
}

fn run_baked_membrane() {
    let run_started = std::time::Instant::now();
    let (base_word, word) = match (BAKED_BASE_WORD, BAKED_MODULUS_WORD) {
        (Some(base), Some(modulus)) => (base, modulus),
        _ => {
            eprintln!("This membrane has no baked base/modulus pair. Build it with factor_2adic_membrane.sh <N> [output] [base].");
            std::process::exit(2);
        }
    };

    let base = match parse_numeral(base_word) {
        Ok(base) => base,
        Err(error) => {
            eprintln!("baked IMASM base is invalid: {error}");
            std::process::exit(2);
        }
    };
    let tape = match parse_numeral(word) {
        Ok(tape) => tape,
        Err(error) => {
            eprintln!("baked IMASM modulus is invalid: {error}");
            std::process::exit(2);
        }
    };
    let radix_word = match BAKED_WIDTH_WORD {
        Some(word) => word,
        None => {
            eprintln!("This membrane has no baked lift radix. Rebuild it with a radix argument.");
            std::process::exit(2);
        }
    };
    let radix = match parse_numeral(radix_word) {
        Ok(radix) if vox::morphism_factor::cmp(&radix, &['⊤', '⊥']) == core::cmp::Ordering::Greater => radix,
        _ => {
            eprintln!("baked lift radix must be a numeral greater than one");
            std::process::exit(2);
        }
    };

    // The phase partner relation supplies the winding. No shape scout,
    // divisor walk, rho, sieve, or factor-candidate scan is on this path.
    let phase_started = std::time::Instant::now();
    let partner_open = if BAKED_BASE_IS_UNIT {
        phase_partners::Partners::new_from_validated_bake(base.clone(), tape.clone())
    } else {
        phase_partners::Partners::new(base.clone(), tape.clone())
    };
    let mut partners = match partner_open {
        Ok(partners) => partners,
        Err(error) => {
            eprintln!("phase membrane could not open: {error}");
            std::process::exit(2);
        }
    };
    let phase_init_elapsed = phase_started.elapsed();
    let phase_orbit_started = std::time::Instant::now();
    let relation = loop {
        match partners.observe() {
            Ok(Some(relation)) => break relation,
            // Keep the baked membrane collapsed until the complete winding,
            // factor close, and register fold are ready to emit together.
            Ok(None) => {}
            Err(error) => {
                eprintln!("phase membrane failed: {error}");
                std::process::exit(2);
            }
        }
    };
    let phase_orbit_elapsed = phase_orbit_started.elapsed();
    let phase_elapsed = phase_started.elapsed();
    let extract_started = std::time::Instant::now();
    let transported = match phase_word::execute(EXTRACT_WORD, &relation) {
        Ok(readout) if readout.surviving.len() == 4 && readout.restored == 1 && readout.exposed == 0 => readout,
        Ok(readout) => {
            eprintln!("EXTRACT frame did not restore its complete bank: surviving={} restored={} exposed={}",
                readout.surviving.len(), readout.restored, readout.exposed);
            std::process::exit(2);
        }
        Err(error) => {
            eprintln!("EXTRACT frame failed: {error}");
            std::process::exit(2);
        }
    };
    let relation = transported.surviving[0].observation.clone();
    let extract_elapsed = extract_started.elapsed();
    let winding = relation.return_exponent.clone();
    let factor_close_started = std::time::Instant::now();
    let factor_pair = if let Some((current_half, earlier_half)) = relation.half_residues.as_ref() {
        vox::shor_braid::phase_factor_register_seeds(&tape, current_half, earlier_half)
    } else {
        vox::shor_braid::factor_close_public(&base, &tape, &winding)
    };
    let factor_close_elapsed = factor_close_started.elapsed();
    let (factors, report) = if let Ok((mut p, mut q)) = factor_pair {
        if vox::morphism_factor::cmp(&p, &q) == core::cmp::Ordering::Greater {
            core::mem::swap(&mut p, &mut q);
        }
        let closure_started = std::time::Instant::now();
        let product_outer_started = std::time::Instant::now();
        let product_outer = vox::factor_2adic::nest_product_over_prefix(&tape, &p, &q, &radix);
        let product_outer_exact_elapsed = product_outer_started.elapsed();
        let prefix_started = std::time::Instant::now();
        let prefix_outer = vox::factor_2adic::nest_prefix_over_product(&tape, &p, &q, &radix);
        let prefix_elapsed = prefix_started.elapsed();
        let prefix_first = product_outer.is_some();
        let product_first = prefix_outer.is_some();
        let factors_at_meeting = product_outer
            .zip(prefix_outer)
            .filter(|(product_first, prefix_first)| product_first == prefix_first)
            .map(|(product_first, _)| product_first);
        let closure_value = fde_closure_value(Some(prefix_first), Some(product_first));
        let closure_elapsed = closure_started.elapsed();
        let report = format!("phase winding register: {} bits ({} dyadic observations, {} phase registers)\nmembrane timing: phase-init={phase_init_elapsed:?} phase-orbit={phase_orbit_elapsed:?} phase-winding={phase_elapsed:?} banked-extract={extract_elapsed:?} phase-dual-seed={factor_close_elapsed:?} product-outer-prefix={product_outer_exact_elapsed:?} prefix-outer-product={prefix_elapsed:?} fde-dual-closure={closure_elapsed:?}\nFDE closure: {closure_value} (product-outer-prefix={prefix_first}, prefix-outer-product={product_first})\n", winding.len(), partners.squarings, partners.stored_residues());
        let factors = if closure_value == 'T' {
            factors_at_meeting.and_then(|fixed|
                vox::factor_2adic::terminal_pair_given_semiprime_promise(&tape, fixed))
        } else {
            None
        };
        (factors, report)
    } else {
        let report = format!("phase winding register: {} bits ({} dyadic observations, {} phase registers)\nmembrane timing: phase-init={phase_init_elapsed:?} phase-orbit={phase_orbit_elapsed:?} phase-winding={phase_elapsed:?} banked-extract={extract_elapsed:?} phase-dual-seed={factor_close_elapsed:?} fde-dual-closure=not-run\nphase factor-register seed: {}\n", winding.len(), partners.squarings, partners.stored_residues(), factor_pair.unwrap_err());
        (None, report)
    };

    let output_started = std::time::Instant::now();
    let factor_bits = factors.as_ref().map_or(0, |meeting|
        meeting.p.len().saturating_add(meeting.q.len()));
    let rendered_bits = tape.len().saturating_add(factor_bits);
    let output_capacity = 2048usize.saturating_add(rendered_bits.saturating_mul(20));
    let mut output = String::with_capacity(output_capacity);
    use core::fmt::Write;
    writeln!(output, "membrane input: baked IMASM numeral registers\nnesting: phase winding ⊃ banked EXTRACT ⊃ (product ⊃ prefix) ∩ (prefix ⊃ product) ⊃ factor fixed point\ntermination: exact proper pair under semiprime promise\nbase register bits={}\nlift radix register bits={}\nN register bits={}", base.len(), radix.len(), tape.len()).unwrap();
    append_frames(&mut output, "N", &tape);
    if let Some(meeting) = factors {
        let p_word = vox::morphism_factor::emit_numeral(&meeting.p);
        let q_word = vox::morphism_factor::emit_numeral(&meeting.q);
        writeln!(output, "P = {p_word}\nQ = {q_word}").unwrap();
        append_frames(&mut output, "P", &meeting.p);
        append_frames(&mut output, "Q", &meeting.q);
    } else {
        output.push_str("phase winding did not close to a nontrivial factor pair for this baked base\n");
    }
    let render_elapsed = output_started.elapsed();
    let total_elapsed = run_started.elapsed();
    let output_write_started = std::time::Instant::now();
    let mut stdout = std::io::stdout().lock();
    std::io::Write::write_all(&mut stdout, output.as_bytes()).expect("write complete membrane report");
    std::io::Write::flush(&mut stdout).expect("flush complete membrane report");
    let output_write_elapsed = output_write_started.elapsed();
    let total_to_flush = run_started.elapsed();
    eprintln!("{report}membrane timing: output-render={render_elapsed:?} total-in-process={total_elapsed:?} output-write-flush={output_write_elapsed:?} total-to-flush={total_to_flush:?}");
}

/// Stable no-argument entry for direct execution from a lifted IMASM module.
#[no_mangle]
pub extern "C" fn imasm_entry() {
    run_baked_membrane();
}

fn main() {
    if std::env::args_os().len() != 1 {
        eprintln!("This membrane already contains its base and modulus and accepts no runtime arguments.");
        std::process::exit(2);
    }
    imasm_entry();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repaired_extract_frame_restores_the_phase_relation_before_closure() {
        let relation = phase_partners::ReturnRelation {
            earlier: vec!['⊤'],
            later: vec!['⊥'],
            residue: vec!['⊤', '⊥'],
            return_exponent: vec!['⊥'],
            half_residues: None,
        };
        let readout = phase_word::execute(EXTRACT_WORD, &relation).unwrap();
        assert_eq!(readout.surviving.len(), 4);
        assert_eq!(readout.restored, 1);
        assert_eq!(readout.exposed, 0);
        assert!(readout.surviving.iter().all(|deposit| deposit.observation == relation));
    }

    #[test]
    fn compact_frame_alphabets_reconstruct_the_complete_source_tape() {
        let source = vec!['⊥', '⊤', '⊥', '⊥', '⊤', '⊥', '⊤', '⊤', '⊥', '⊥', '⊥'];
        let mut output = String::new();
        append_frames(&mut output, "N", &source);
        for line in output.lines() {
            let mut fields = line.split_whitespace();
            assert_eq!(fields.next(), Some("frame"));
            assert_eq!(fields.next(), Some("N"));
            let window = fields.next().unwrap().strip_prefix("window=").unwrap().parse::<usize>().unwrap();
            let tail = fields.next().unwrap().strip_prefix("tail=").unwrap().parse::<usize>().unwrap();
            let bits = fields.next().unwrap().strip_prefix("bits=").unwrap().parse::<usize>().unwrap();
            let encoded = fields.next().unwrap().strip_prefix("groups-hex=").unwrap();
            let token_width = usize::from(window > 4) + 1;
            let mut reconstructed = Vec::with_capacity(bits);
            for (group_index, token) in encoded.as_bytes().chunks(token_width).enumerate() {
                let token = core::str::from_utf8(token).unwrap();
                let state = u8::from_str_radix(token, 16).unwrap();
                let group_bits = if group_index + 1 == (bits + window - 1) / window {
                    tail
                } else {
                    window
                };
                for index in 0..group_bits {
                    reconstructed.push(if (state >> index) & 1 == 1 { '⊥' } else { '⊤' });
                }
            }
            assert_eq!(reconstructed, source);
        }
    }
}

/*
    let word = match FACTOR_N_WORD {
        Some(word) => word,
        None => {
            eprintln!("This membrane has no baked numeral. Build it with factor_2adic_membrane.sh <N> [output].");
            std::process::exit(2);
        }
    };

    let tape = match vox::morphism_factor::parse_numeral(word) {
        Ok(tape) => tape,
        Err(error) => {
            eprintln!("baked IMASM numeral is invalid: {error}");
            std::process::exit(2);
        }
    };

    // Route semiprimes through the fast shape scout first. Its closure pair is
    // already the two-register result, so recursively proving each factor
    // prime would add a second primality workload to the baked semiprime path.
    let factor_started = std::time::Instant::now();
    let (direct_pair, scout_route) = vox::morphism_factor::scout_semiprime(&tape);
    let (pair, route) = if let Some((p, q, _shape)) = direct_pair {
        (Some((p, q)), scout_route)
    } else if scout_route.contains("shape: prime") {
        (None, scout_route)
    } else {
        let (mut prime_parts, route) = vox::morphism_factor::smart_factor(&tape);
        prime_parts.sort_by(|left, right| vox::morphism_factor::cmp(left, right));
        if prime_parts.len() >= 2 {
            let p = prime_parts.remove(0);
            let q = prime_parts.into_iter().fold(vec![EVALF], |product, factor| {
                vox::morphism_factor::mul(&product, &factor)
            });
            (Some((p, q)), route)
        } else {
            (None, route)
        }
    };
    let factor_elapsed = factor_started.elapsed();
    let factors = if let Some((mut p, mut q)) = pair {
        if vox::morphism_factor::cmp(&p, &q) == core::cmp::Ordering::Greater {
            core::mem::swap(&mut p, &mut q);
        }
        let lift_started = std::time::Instant::now();
        let prefix_first = verify_pair_by_lift(&tape, &p, &q);
        let product_first = verify_pair_by_product_outer(&tape, &p, &q);
        let closure_value = fde_closure_value(Some(prefix_first), Some(product_first));
        let lift_elapsed = lift_started.elapsed();
        eprintln!("factor route: {}", route.trim().replace('\n', "; "));
        eprintln!("membrane timing: tape-factorization={factor_elapsed:?} fde-dual-closure={lift_elapsed:?}");
        println!("FDE closure: {closure_value} (prefix-first={prefix_first}, product-first={product_first})");
        if closure_value == 'T' { Some((p, q)) } else { None }
    } else {
        eprintln!("factor route: {}", route.trim().replace('\n', "; "));
        eprintln!("membrane timing: tape-factorization={factor_elapsed:?} fde-dual-closure=not-run");
        None
    };

    println!("membrane input: baked IMASM numeral");
    println!("nesting: Vox tape factor membrane ⊃ encoded prefix lift ⊃ product closure");
    println!("N = {word}");
    print_frames("N", &tape);
    if let Some((p, q)) = factors {
        let p_word = vox::morphism_factor::emit_numeral(&p);
        let q_word = vox::morphism_factor::emit_numeral(&q);
        println!("P = {p_word}");
        println!("Q = {q_word}");
        print_frames("P", &p);
        print_frames("Q", &q);
    } else {
        println!("no nontrivial odd factor pair");
    }
}

/// Stable no-argument entry for direct execution from a lifted IMASM module.
#[no_mangle]
pub extern "C" fn imasm_entry() {
    run_baked_membrane();
}

fn main() {
    if std::env::args_os().len() != 1 {
        eprintln!("This membrane already contains its value and accepts no runtime arguments.");
        std::process::exit(2);
    }
    imasm_entry();
}
*/
