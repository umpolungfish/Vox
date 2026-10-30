#![deny(warnings)]
use num_bigint::BigUint;
use std::{
    env,
    io::{Read, BufRead, BufReader},
    path::PathBuf,
    process::{Command, Stdio},
    thread,
    time::Duration,
};

/// Execute the source-bound Y/frontier return on the shared product register.
/// An explicit pass count selects a diagnostic; ordinary execution has no cap.
pub fn run_dialectic_product(args: &[String]) -> Result<i32, String> {
    if args.iter().any(|arg| matches!(arg.as_str(), "--help" | "-h")) {
        println!("vox factor-membrane N --dialectic-product [--passes COUNT]\nN-only retained Y descent, midpoint feedback, shared product/carry and phase return.\nOmit --passes to continue to closure; supplying it requests a bounded diagnostic.");
        return Ok(0);
    }
    let mut source_args = Vec::new();
    let mut passes = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--dialectic-product" => i += 1,
            "--passes" => {
                if passes.is_some() { return Err("--passes provided twice".into()); }
                let count = args.get(i+1).ok_or("--passes requires COUNT")?
                    .parse::<usize>().map_err(|_| "invalid --passes count")?;
                if count == 0 { return Err("diagnostic pass count must be positive".into()); }
                passes = Some(count);
                i += 2;
            }
            "--factors" => return Err("dialectic-product production takes N alone".into()),
            _ => { source_args.push(args[i].clone()); i += 1; }
        }
    }
    let source = vox::membrane_factor::source_for_gpu_producer(&source_args)?
        .ok_or("provide one N-only source")?;
    thread::Builder::new().stack_size(64*1024*1024).spawn(move || {
        use vox::{glut_system::PreparedGlutMembrane, morphism_factor as mf};
        let n = mf::decimal_to_tape(&source).ok_or("invalid source")?;
        let start = std::time::Instant::now();
        let mut membrane = PreparedGlutMembrane::prepare_dialectic(&n)?;
        println!("source N={source}\nsource_bits {}\nproducer dialectic-product", n.len());
        println!("nested_word {}", membrane.nested_word().iter().collect::<String>());
        let mut stats = vox::glut_system::FoldStats::default();
        let mut pass = 0usize;
        loop {
            pass += 1;
            let closed = membrane.advance_observed(|observation| stats=observation.clone());
            if closed {
                let readout = start.elapsed().as_secs_f64();
                let result = membrane.execute_retained()?.ok_or("missing proper factor return")?;
                result.verify(&n)?;
                let (q,r) = mf::divmod(&n,&result.p);
                if !mf::zero(&r) || mf::mul(&result.p,&q)!=n || q!=result.q {
                    return Err(String::from("exact source product failed"));
                }
                let bank=membrane.retained_readout().ok_or("missing enclosing return")?;
                println!("factor {}\nfactor {}",mf::dec_of(&result.p),mf::dec_of(&q));
                println!("producing_arm {}",if stats.conditioned_returns>0 {"support-conditioned-square"}else if stats.dialectic_factor_return {"retained-dialectic"}else{"shared-product"});
                println!("conditioned_bindings {}\nconditioned_windows {}\nconditioned_depth {}",stats.conditioned_bindings,stats.conditioned_windows,stats.conditioned_depth);
                println!("passes {pass}\nreadout_seconds {readout:.9}\nverified_seconds {:.9}",start.elapsed().as_secs_f64());
                println!("exact_product true\nzero_remainder true\nbank_deposits {}\nbank_restored {}\nbank_exposed {}",bank.surviving.len(),bank.restored,bank.exposed);
                return Ok(0);
            }
            if passes.is_some_and(|limit|pass>=limit) {
                println!("unresolved passes={pass} descents={} contractions={} excluded_midpoints={} decisions={} conflicts={} gates={} cells={} seconds={:.9}",stats.dialectic_descents,stats.dialectic_contractions,stats.excluded_midpoints,stats.correlation_decisions,stats.correlation_conflicts,stats.correlation_gates,stats.correlation_cells,start.elapsed().as_secs_f64());
                println!("conditioned_bindings {}\nconditioned_windows {}\nconditioned_depth {}",stats.conditioned_bindings,stats.conditioned_windows,stats.conditioned_depth);
                return Ok(3);
            }
        }
    }).map_err(|error| error.to_string())?.join()
        .map_err(|_| String::from("dialectic-product execution panicked"))?
}

/// Replace N-only production with the demonstrated CUDA arithmetic membrane.
/// Vox remains the validator and carry-trace closer; a missing device receipt
/// is an error, never a CPU fallback.
pub fn route_n_only_to_gpu(args: &[String]) -> Result<(Vec<String>, Option<String>), String> {
    let entry=env::var("VOX_GPU_MEMBRANE_ENTRY").unwrap_or_else(|_| "word".into());
    route_n_only_to_gpu_entry(args,&entry)
}

pub fn route_n_only_to_gpu_entry(args:&[String],entry:&str)->Result<(Vec<String>,Option<String>),String> {
    let Some(n_text) = vox::membrane_factor::source_for_gpu_producer(args)? else {
        return Ok((args.to_vec(), None));
    };
    let n = n_text.parse::<BigUint>().map_err(|_| "invalid decoded GPU source".to_string())?;
    let executable = env::var_os("VOX_GPU_FACTORIZER").map(PathBuf::from).unwrap_or_else(|| {
        let sibling = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../G-mOMonadOS/target/release/g-momonados");
        if sibling.is_file() { sibling } else { PathBuf::from("g-momonados") }
    });
    let word=env::var("VOX_GPU_MEMBRANE_WORD")
        .unwrap_or_else(|_| "⊢⊙∈≻⊤≺⊥⊞⋈∋⊡⊣".into());
    let mut producer_command=Command::new(&executable);
    let program=match entry {
        "glut"=>{
            producer_command.args(["gpu_glut",&n_text]);
            String::from("GLUT shared factor/product/carry register")
        }
        "word"=>{
            producer_command.args(["membrane_family","run",&word,&n_text]);
            word.clone()
        }
        "make"=>{
            let template=env::var("VOX_GPU_MEMBRANE_TEMPLATE").unwrap_or_else(|_| "carry_fuse_closure".into());
            producer_command.args(["make",&n_text,&template]);
            template
        }
        "hyperstack"=>{
            let types=env::var("VOX_GPU_HYPERSTACK_TYPES").unwrap_or_else(|_| "phase shor fib".into());
            producer_command.args(["hyperstack",&n_text]).args(types.split_whitespace());
            types
        }
        _=>return Err(format!("unknown CUDA membrane entry {entry}")),
    };
    let run_seconds=env::var("MEMBRANE_RUN_SECONDS").ok()
        .and_then(|s|s.parse::<u64>().ok()).unwrap_or(300);
    let mut child = producer_command
        .stdout(Stdio::piped())
        .env("MEMBRANE_RUN_SECONDS", run_seconds.to_string())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("could not start GPU factor producer {}: {error}", executable.display()))?;
    let stdout = child.stdout.take().ok_or("GPU producer stdout unavailable")?;
    let stderr = child.stderr.take().ok_or("GPU producer stderr unavailable")?;
    let output_reader = thread::spawn(move || {
        let mut text = String::new();
        let mut pipe = stdout;
        pipe.read_to_string(&mut text).map(|_| text)
    });
    let error_reader = thread::spawn(move || {
        let mut pipe = BufReader::new(stderr);
        let mut line = String::new();
        loop {
            line.clear();
            if pipe.read_line(&mut line)? == 0 { return Ok::<(), std::io::Error>(()); }
            eprint!("{line}");
        }
    });
    let status = loop {
        if let Some(status) = child.try_wait().map_err(|error| format!("GPU producer wait failed: {error}"))? {
            break status;
        }
        thread::sleep(Duration::from_millis(100));
    };
    let output = output_reader.join().map_err(|_| "GPU output reader panicked")?
        .map_err(|error| format!("could not read GPU factor receipt: {error}"))?;
    error_reader.join().map_err(|_| "GPU diagnostics reader panicked")?
        .map_err(|error| format!("could not read GPU diagnostics: {error}"))?;
    if !status.success() {
        return Err(format!("GPU factor producer exited with {status}: {}", output.trim()));
    }
    let device=output.lines().find_map(|line| {
        let field=line.trim().strip_prefix("CUDA device=")?;
        field.split(';').next()?.trim().parse::<usize>().ok()
    });
    if !matches!(device,Some(0)|Some(1)) || !output.contains("GPU product closure verified") {
        return Err(format!("GPU factor receipt has no completed CUDA product closure: {}", output.trim()));
    }
    let product_row = output.lines().find(|line| line.contains(" = ") && line.contains(" x "))
        .ok_or_else(|| "CUDA membrane receipt has no factor row".to_string())?;
    let pair = product_row.split_once(" = ").unwrap().1
        .split(" (GPU product closure verified)").next().unwrap_or("");
    let (p_text, q_text) = pair.split_once(" x ")
        .ok_or_else(|| "CUDA membrane receipt has malformed factor row".to_string())?;
    let p = p_text.trim().parse::<BigUint>().map_err(|_| "CUDA receipt has invalid first factor".to_string())?;
    let q = q_text.trim().parse::<BigUint>().map_err(|_| "CUDA receipt has invalid second factor".to_string())?;
    if p <= BigUint::from(1u8) || q <= BigUint::from(1u8) || &p * &q != n {
        return Err("GPU factor receipt failed exact product closure".into());
    }
    let mut routed = args.to_vec();
    routed.extend(["--factors".into(), p.to_string(), q.to_string()]);
    let producer = format!("{} entry={entry} program={program} CUDA device={}", executable.display(),device.unwrap());
    Ok((routed, Some(producer)))
}

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
