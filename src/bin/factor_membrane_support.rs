use std::process::Command;

pub fn append_trilattice_reads(report: &str) {
    let values: Vec<(&str, String)> = ["N", "p", "q"]
        .into_iter()
        .filter_map(|name| {
            report.lines().find_map(|line| {
                let value = line.strip_prefix(&format!("{name} = "))?;
                Some((name, value.trim().to_string()))
            })
        })
        .collect();
    if values.is_empty() {
        return;
    }

    let executable = std::env::var_os("VOX_TRILATTICE_FACTOR")
        .unwrap_or_else(|| "g-momonados".into());
    println!("\nLive g-mOMonadOS trilattice readings:");
    let mut readings: [Option<Readout>; 3] = [None, None, None];
    for (index, (label, value)) in values.into_iter().enumerate() {
        println!("[{label}]");
        match Command::new(&executable)
            .arg("trilattice_factor")
            .arg("read")
            .arg(&value)
            .output()
        {
            Ok(output) if output.status.success() => {
                let text = String::from_utf8_lossy(&output.stdout).into_owned();
                print!("{text}");
                readings[index] = parse_readout(&text);
                if !output.stderr.is_empty() {
                    eprint!("{}", String::from_utf8_lossy(&output.stderr));
                }
            }
            Ok(output) => eprintln!(
                "reader exited with {}: {}",
                output.status,
                String::from_utf8_lossy(&output.stderr).trim()
            ),
            Err(error) => eprintln!(
                "could not run {} ({error}); set VOX_TRILATTICE_FACTOR to the g-mOMonadOS executable",
                std::path::Path::new(&executable).display()
            ),
        }
    }
    if let (Some(n), Some(p), Some(q)) = (&readings[0], &readings[1], &readings[2]) {
        println!("Candidate-pair membrane checks:");
        println!("  p and q dialect registers equal N: {}",
            p.dialect.as_str() == n.dialect.as_str()
                && q.dialect.as_str() == n.dialect.as_str());
        println!("  p and q coarse type hashes equal N: {}",
            p.type_hash.as_str() == n.type_hash.as_str()
                && q.type_hash.as_str() == n.type_hash.as_str());
        println!("  period residual N - p - q: {}",
            n.period as i128 - p.period as i128 - q.period as i128);
        println!("  cut residual p + q - N: {}",
            p.cuts as i128 + q.cuts as i128 - n.cuts as i128);
    }
}

struct Readout {
    period: usize,
    cuts: usize,
    dialect: String,
    type_hash: String,
}

fn parse_readout(text: &str) -> Option<Readout> {
    let orbit = text.lines().find(|line| line.contains("winding-commit cut(s) over the orbit"))?;
    let mut numbers = orbit.split(|c: char| !c.is_ascii_digit()).filter(|s| !s.is_empty());
    let period = numbers.next()?.parse().ok()?;
    let cuts = numbers.next()?.parse().ok()?;
    let dialect_line = text.lines().find(|line| line.contains("dialect register"))?;
    let dialect = dialect_line.split(':').nth(1)?.trim().split_whitespace().next()?.to_string();
    let hash_line = text.lines().find(|line| line.contains("type hash"))?;
    let type_hash = hash_line.split(':').nth(1)?.trim().split_whitespace().next()?.to_string();
    Some(Readout { period, cuts, dialect, type_hash })
}
