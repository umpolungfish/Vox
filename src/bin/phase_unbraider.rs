//! phase_unbraider — IMASM-resident phase readout, the Vox membrane.
//!
//! Quantum phase estimation on baked IMASM numeral tapes.
//! Unbounded BigUint register with QFT continued-fraction readout.

extern crate alloc;
#[path = "../phase_unbraid.rs"] mod phase_unbraid;

use num_bigint::BigUint;

fn main() {
    let result = (|| -> Result<String, String> {
        let raw = option_env!("MEMBRANE_WORDS")
            .ok_or("Build with MEMBRANE_WORDS=\"<a-word> <N-word>\"")?;
        let mut words: Vec<Vec<char>> = Vec::new();
        for w in raw.split_whitespace() {
            words.push(::vox::morphism_factor::parse_numeral(w)
                .map_err(|e| format!("numeral parse: {e}"))?);
        }
        if words.len() != 2 { return Err("phase_unbraider needs a and N (two words)".into()); }
        let a_tape = &words[0];
        let n_tape = &words[1];

        let n_dec = ::vox::morphism_factor::dec_of(n_tape);
        let a_dec = ::vox::morphism_factor::dec_of(a_tape);
        let n_val: BigUint = n_dec.parse().map_err(|_| "N decimal parse failure")?;
        let a_val: u64 = a_dec.parse().unwrap_or(2);

        let mut o = String::new();
        o.push_str("phase_unbraider — factors from a quantum phase readout, no search\n");
        o.push_str("surface A: QFT-register phase readout (phase_unbraid)\n");
        o.push_str(&alloc::format!("N = {}  ({} bits)\n", n_dec, n_val.bits()));
        o.push_str(&alloc::format!("a = {}\n", a_val));

        let report = phase_unbraid::phase_unbraid_report_big(&n_dec, a_val, 64, 1 << 22, None)?;
        o.push_str(&report);
        Ok(o)
    })();
    match result {
        Ok(report) => println!("{report}"),
        Err(error) => { eprintln!("{error}"); std::process::exit(2); }
    }
}