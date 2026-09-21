//! A factoring transformation baked into a membrane, run as pure execution.
//!
//! N is baked as its IMASM numeral before compilation, so the built artifact IS
//! this one transformation and the run takes no input. The core lays candidate
//! factor lanes by dyadic squaring (each nested layer doubles the exponent, the
//! free-lunch nesting), and closes the layer where a rotor aligns: gcd(a^E - 1,
//! N) is a proper divisor. The surviving lane pair is p and q. If no rotor
//! aligns within the stack, it hands to the 31-membrane's own split.
use vox::morphism_factor::{parse_numeral, dec_of, one, cmp, modulo, mul, sub, gcd, divmod, tape_u64};

type Tape = Vec<char>;

/// The nested-squaring split: over several bases, square the rotor per layer,
/// a -> a^2, so layer k carries a^(2^k), and read gcd(a^(2^k) - 1, N) at each.
/// The lane pair (g, N/g) is returned the moment a rotor aligns on one prime.
fn structural_split(n: &[char], layers: usize, bases: u64) -> Option<(Tape, Tape)> {
    let one_t = one();
    for a_val in 2..bases {
        let a = tape_u64(a_val);
        let g0 = gcd(a.clone(), n.to_vec());
        if cmp(&g0, &one_t) != core::cmp::Ordering::Equal && cmp(&g0, n) != core::cmp::Ordering::Equal {
            let (q, _r) = divmod(n, &g0);
            return Some((g0, q));
        }
        let mut x = modulo(&a, n);              // rotor at exponent 1
        for _ in 0..layers {
            x = modulo(&mul(&x, &x), n);         // square: double the exponent (the nest)
            if cmp(&x, &one_t) == core::cmp::Ordering::Equal { break; } // rotor closed on both, no split
            let g = gcd(sub(&x, &one_t), n.to_vec());
            if cmp(&g, &one_t) != core::cmp::Ordering::Equal && cmp(&g, n) != core::cmp::Ordering::Equal {
                let (q, _r) = divmod(n, &g);
                return Some((g, q));
            }
        }
    }
    None
}

fn main() {
    let word: &str = option_env!("FACTOR_N_WORD").unwrap_or("⊢⊙⊡⊣");
    let n = match parse_numeral(word) {
        Ok(v) => v,
        Err(e) => { eprintln!("factor membrane: {e}"); std::process::exit(2); }
    };
    if let Some((p, q)) = structural_split(&n, 4096, 256) {
        println!("N={}\nfactors: {} x {}   [nested-squaring split]", dec_of(&n), dec_of(&p), dec_of(&q));
        return;
    }
    match vox::factorization_31_membrane::factor_n(word) {
        Ok(report) => println!("{report}   [31-membrane split]"),
        Err(e) => { eprintln!("factor membrane: {e}"); std::process::exit(2); }
    }
}
