//! multi_quantum_membrane — three FDE quantum computers nested into ONE
//! membrane, baked and one-shot. a, N and the register widths are IMASM
//! numerals compiled in (include_str! of the pre-encoded words file); there is
//! no runtime input and no runtime encoding. One prepared state, one measurement.
extern crate alloc;
#[allow(dead_code)]
#[path = "../shor_qft.rs"] mod shor_qft;
#[allow(dead_code)]
#[path = "../membrane_complex.rs"] mod membrane_complex;
#[allow(dead_code)]
#[path = "../baked_membrane.rs"] mod baked_membrane;
#[path = "../fde_shor_membrane.rs"] mod fde_shor_membrane;

use std::io::Write;

const BAKED: &str = include_str!("../../membrane_words.imasm");

fn tape_to_usize(tape: &[char]) -> Result<usize, String> {
    tape.iter().enumerate().try_fold(0usize, |v, (b, &m)| {
        if m == vox::vox::EVALF { v.checked_add(1usize.checked_shl(b as u32)?) } else { Some(v) }
    }).ok_or("register width out of host range".into())
}

fn factors_in(text: &str) -> Option<(String, String)> {
    for line in text.lines() {
        let k = if let Some(i) = line.find("factors=") { Some(i + 8) }
            else if line.contains('\u{d7}') { line.find("= ").map(|i| i + 2) }
            else { None };
        if let Some(k) = k {
            let mut it = line[k..].split(|c| c == 'x' || c == '\u{d7}');
            if let (Some(p), Some(q)) = (it.next(), it.next()) {
                let p: String = p.trim().chars().take_while(|c| c.is_ascii_digit()).collect();
                let q: String = q.trim().chars().take_while(|c| c.is_ascii_digit()).collect();
                if !p.is_empty() && !q.is_empty() { return Some((p, q)); }
            }
        }
    }
    None
}

fn main() {
    let result = (|| -> Result<(), String> {
        let words = baked_membrane::numeral_words(Some(BAKED))?;
        if words.len() < 2 { return Err("membrane requires a and N as baked numerals".into()); }
        let a = words[0].clone();
        let n = words[1].clone();
        let q0 = match words.get(2) { Some(t) => tape_to_usize(&t[..])?, None => 13 };
        println!("FDE QC membrane - one-shot, IMASM-baked; no runtime input, no decoded input");
        let _ = std::io::stdout().flush();

        let mut acc: Option<(String, String)> = None;
        let mut disagree = false;
        let depths = [q0, q0 + 4, (2 * n.len()).max(16)];
        for (i, q) in depths.iter().enumerate() {
            println!("\n[arm{} resident FDE QC — register depth {}]", i + 1, q);
            let _ = std::io::stdout().flush();
            match shor_qft::run_shor_big_report(a.clone(), n.clone(), *q) {
                Ok(s) => {
                    println!("{s}");
                    if let Some(v) = factors_in(&s) {
                        match &acc {
                            None => acc = Some(v),
                            Some(a0) if *a0 != v => disagree = true,
                            _ => {}
                        }
                    }
                }
                Err(e) => println!("arm{} Err: {e}", i + 1),
            }
            let _ = std::io::stdout().flush();
        }

        println!("\n-- membrane verdict --");
        match (&acc, disagree) {
            (Some((p, q)), false) => println!("Verdict: T - arms agree: factors = {p} x {q}"),
            (Some((p, q)), true) => println!("Verdict: B - arms disagree; held (e.g. {p} x {q})"),
            (None, _) => println!("Verdict: N - no arm closed a factor"),
        }
        Ok(())
    })();
    if let Err(e) = result { eprintln!("{e}"); std::process::exit(2); }
}
