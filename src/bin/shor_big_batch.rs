//! Batch arbitrary-width Shor membrane. All operands are compile-time baked.

extern crate alloc;
#[allow(dead_code)]
#[path = "../shor_qft.rs"] mod shor_qft;
#[allow(dead_code)]
#[path = "../membrane_complex.rs"] mod membrane_complex;
#[allow(dead_code)]
#[path = "../baked_membrane.rs"] mod baked_membrane;

fn tape_to_usize(tape: &[char]) -> Result<usize, String> {
    tape.iter().enumerate().try_fold(0usize, |value, (bit, &mark)| {
        if mark == vox::vox::EVALF { value.checked_add(1usize.checked_shl(bit as u32)?) } else { Some(value) }
    }).ok_or("resident QFT depth cannot be represented by this host index type".into())
}

fn main() {
    let result = (|| -> Result<Vec<String>, String> {
        let baked = include_str!("/home/mrnob0dy666/imsgct/bvalsd.txt");
        let a = vox::morphism_factor::tape_u64(2);
        let qbits = vox::morphism_factor::tape_u64(12);
        let mut words = Vec::new();
        for line in baked.lines().filter(|line| !line.trim().is_empty()) {
            words.push(a.clone());
            words.push(vox::morphism_factor::decimal_to_tape(line.trim())
                .ok_or_else(|| format!("invalid decimal in baked input: {}", line.trim()))?);
            words.push(qbits.clone());
        }
        if words.len() % 3 != 0 { return Err("baked batch must contain a, N, qubits triples".into()); }
        let mut reports = Vec::with_capacity(words.len() / 3);
        for group in words.chunks_exact(3) {
            let started = std::time::Instant::now();
            let n = group[1].clone();
            let qbits = tape_to_usize(&group[2])?;
            let outcome = match shor_qft::run_shor_big_report(group[0].clone(), n.clone(), qbits) {
                Ok(report) => report,
                Err(error) => format!(
                    "shor: N={} status=error reason={}",
                    vox::morphism_factor::dec_of(&n), error
                ),
            };
            reports.push(format!("{} elapsed_ms={}", outcome, started.elapsed().as_millis()));
        }
        Ok(reports)
    })();
    match result {
        Ok(reports) => for report in reports { println!("{report}"); },
        Err(error) => { eprintln!("{error}"); std::process::exit(2); }
    }
}
