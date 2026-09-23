//! Runtime for one compiled phase-winding and bit-register membrane.
//!
//! Base and modulus arrive as baked IMASM numeral tapes. Dyadic phase
//! observations produce a return relation, which is closed by the two-factor
//! radix-register fold in both product-first and prefix-first order.

#[path = "../phase_partners.rs"] mod phase_partners;

use vox::morphism_factor::parse_numeral;

include!(concat!(env!("OUT_DIR"), "/baked_inputs.rs"));

fn verify_pair_by_lift(
    n: &[char], p: &[char], q: &[char], radix: &[char],
) -> Option<vox::factor_2adic::RadixPrefixMembrane> {
    vox::factor_2adic::radix_prefix_fold(n, p, q, radix)
}

/// Reverse nesting of the same closure: the multiplicative shell closes first,
/// then its factors are carried through the encoded prefix lift. Keeping this
/// order separate from `verify_pair_by_lift` makes disagreement visible rather
/// than silently treating one composition as the other.
fn verify_pair_by_product_outer(n: &[char], p: &[char], q: &[char]) -> bool {
    vox::morphism_factor::mul(p, q) == n
}

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

fn print_frames(label: &str, tape: &[char]) {
    for frame in vox::factor_2adic::frame_sweep(tape) {
        let cells = frame.groups.iter().map(|group| group.iter().collect::<String>())
            .collect::<Vec<_>>().join("|");
        println!("frame {label} window={} tail={} bits={} groups={cells}",
            frame.window, frame.tail_len, tape.len());
        assert_eq!(frame.reconstruct(), tape);
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
    let mut partners = match phase_partners::Partners::new(base.clone(), tape.clone()) {
        Ok(partners) => partners,
        Err(error) => {
            eprintln!("phase membrane could not open: {error}");
            std::process::exit(2);
        }
    };
    let relation = loop {
        match partners.observe() {
            Ok(Some(relation)) => break relation,
            Ok(None) => {
                if partners.squarings % 8 == 0 {
                    eprintln!("phase progress: observations={} registers={}", partners.squarings, partners.stored_residues());
                }
            }
            Err(error) => {
                eprintln!("phase membrane failed: {error}");
                std::process::exit(2);
            }
        }
    };
    let phase_elapsed = phase_started.elapsed();
    let winding = relation.return_exponent.clone();
    let factor_close_started = std::time::Instant::now();
    let factor_pair = vox::shor_braid::factor_close_public(&base, &tape, &winding);
    let factor_close_elapsed = factor_close_started.elapsed();
    let factors = if let Ok((mut p, mut q)) = factor_pair {
        if vox::morphism_factor::cmp(&p, &q) == core::cmp::Ordering::Greater {
            core::mem::swap(&mut p, &mut q);
        }
        let closure_started = std::time::Instant::now();
        let product_outer_started = std::time::Instant::now();
        let product_first_exact = verify_pair_by_product_outer(&tape, &p, &q);
        let product_outer_exact_elapsed = product_outer_started.elapsed();
        let prefix_started = std::time::Instant::now();
        let prefix_state = verify_pair_by_lift(&tape, &p, &q, &radix);
        let prefix_elapsed = prefix_started.elapsed();
        let terminal_started = std::time::Instant::now();
        let prefix_first_terminal = prefix_state.as_ref().is_some_and(|state| state.is_fixed_point());
        let prefix_terminal_elapsed = terminal_started.elapsed();
        let prefix_first = prefix_state.is_some() && prefix_first_terminal;
        let product_first = product_first_exact && prefix_state.is_some();
        let closure_value = fde_closure_value(Some(prefix_first), Some(product_first));
        let closure_elapsed = closure_started.elapsed();
        eprintln!("phase winding: {} ({} dyadic observations, {} phase registers)", vox::morphism_factor::dec_of(&winding), partners.squarings, partners.stored_residues());
        eprintln!("membrane timing: phase-winding={phase_elapsed:?} factor-close={factor_close_elapsed:?} product-outer-exact={product_outer_exact_elapsed:?} shared-prefix-lift={prefix_elapsed:?} prefix-terminal-exact={prefix_terminal_elapsed:?} fde-dual-closure={closure_elapsed:?}");
        println!("FDE closure: {closure_value} (prefix-first={prefix_first}, product-first={product_first})");
        if closure_value == 'T' { Some((p, q)) } else { None }
    } else {
        eprintln!("phase winding: {} ({} dyadic observations, {} phase registers)", vox::morphism_factor::dec_of(&winding), partners.squarings, partners.stored_residues());
        eprintln!("membrane timing: phase-winding={phase_elapsed:?} factor-close={factor_close_elapsed:?} fde-dual-closure=not-run");
        eprintln!("phase factor-close: {}", factor_pair.unwrap_err());
        None
    };

    let output_started = std::time::Instant::now();
    println!("membrane input: baked IMASM base and modulus");
    println!("nesting: phase winding ⊃ factor close ⊃ encoded radix-register fold");
    println!("base = {base_word}");
    println!("lift radix = {radix_word}");
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
        println!("phase winding did not close to a nontrivial factor pair for this baked base");
    }
    let _ = std::io::Write::flush(&mut std::io::stdout());
    eprintln!("membrane timing: output-render-flush={:?} total-in-process={:?}", output_started.elapsed(), run_started.elapsed());
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
