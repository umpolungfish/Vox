//! Fast factorization using the Hensel-lifted 2-adic method (O(log N) steps)

use vox::factor_2adic::factor_2adic_hensel;
use vox::morphism_factor::decimal_to_tape;
use num_bigint::BigUint;
use std::env;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let Some(input) = args.first() else {
        eprintln!("Usage: hensel_factor <decimal-number>");
        std::process::exit(1);
    };

    let n = BigUint::parse_bytes(input.as_bytes(), 10)
        .ok_or_else(|| format!("Invalid decimal: {}", input))
        .unwrap();

    if n.bit(0) == false {
        eprintln!("N must be odd");
        std::process::exit(1);
    }

    println!("N = {}", n);
    println!("Bit length = {}", n.bits());

    // Convert to IMASM tape (LSB-first, ⊥=1, ⊤=0)
    let tape = decimal_to_tape(input).unwrap();
    println!("IMASM tape length = {}", tape.len());

    // Execute the Hensel-lifted 2-adic factorization
    let start = std::time::Instant::now();
    let factors = factor_2adic_hensel(&tape, Some(1));
    let elapsed = start.elapsed();

    match factors.first() {
        Some((p_tape, q_tape)) => {
            let p = tape_to_biguint(p_tape);
            let q = tape_to_biguint(q_tape);

            println!("\n=== FACTORIZATION COMPLETE in {:.3}s ===", elapsed.as_secs_f64());
            println!("p = {}", p);
            println!("q = {}", q);
            println!("p * q = {}", &p * &q);
            println!("Verified: {}", &p * &q == n);
        }
        None => {
            eprintln!("Factorization failed");
            std::process::exit(1);
        }
    }
}

fn tape_to_biguint(tape: &[char]) -> BigUint {
    let mut bytes = vec![0u8; (tape.len() + 7) / 8];
    for (index, mark) in tape.iter().enumerate() {
        if *mark == '⊥' {
            bytes[index / 8] |= 1 << (index % 8);
        }
    }
    BigUint::from_bytes_le(&bytes)
}