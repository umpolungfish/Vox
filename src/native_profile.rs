//! Native instruction sampling for silent prepared ELF executions.
use std::{collections::BTreeMap, ffi::c_void, fs::File, io::{BufWriter, Read, Write},
    os::{fd::FromRawFd, unix::process::CommandExt}, path::Path, process::Command, time::{Duration, Instant}};

unsafe extern "C" {
    fn ptrace(request: u32, pid: i32, addr: *mut c_void, data: *mut c_void) -> i64;
    fn waitpid(pid: i32, status: *mut i32, options: i32) -> i32;
    fn kill(pid: i32, signal: i32) -> i32;
    fn syscall(number: i64, ...) -> i64;
}

// Linux perf_event_attr version 0, as declared in linux/perf_event.h.
// Counters are user-space task counters; kernel and hypervisor work is excluded.
#[repr(C)]
struct PerfAttribute {
    kind: u32, size: u32, config: u64, sample_period: u64,
    sample_type: u64, read_format: u64, flags: u64,
    wakeup_events: u32, breakpoint_type: u32, config1: u64,
}

const _: [(); 64] = [(); std::mem::size_of::<PerfAttribute>()];

fn open_counter(pid: i32, kind: u32, config: u64) -> std::io::Result<File> {
    let attribute = PerfAttribute { kind, size: 64, config, sample_period: 0,
        sample_type: 0, read_format: 3, flags: (1 << 5) | (1 << 6),
        wakeup_events: 0, breakpoint_type: 0, config1: 0 };
    // x86-64 syscall table: perf_event_open = 298. The child is stopped at exec,
    // so an initially enabled task counter cannot count until it is continued.
    let fd = unsafe { syscall(298, &attribute as *const PerfAttribute,
        pid, -1i32, -1i32, 8u64) };
    if fd == -1 { return Err(std::io::Error::last_os_error()); }
    Ok(unsafe { File::from_raw_fd(fd as i32) })
}

fn trace(request: u32, pid: i32, data: usize) -> std::io::Result<()> {
    if unsafe { ptrace(request, pid, std::ptr::null_mut(), data as *mut c_void) } == -1 {
        Err(std::io::Error::last_os_error())
    } else { Ok(()) }
}

fn wait_child(pid: i32) -> std::io::Result<i32> {
    loop {
        let mut status = 0;
        if unsafe { waitpid(pid, &mut status, 0) } == pid { return Ok(status); }
        let error = std::io::Error::last_os_error();
        if error.kind() != std::io::ErrorKind::Interrupted { return Err(error); }
    }
}

pub fn run(file: &str, prefix: &str, register_samples: bool) -> Result<(), Box<dyn std::error::Error>> {
    let path = std::fs::canonicalize(file)?;
    let raw = std::fs::read(&path)?;
    let loaded = vox::loader::load(&raw);
    let mut symbols: Vec<_> = loaded.symbols.iter().map(|(n,a)| (*a,n.clone())).collect();
    symbols.sort();
    let mut samples = BufWriter::new(File::create(format!("{prefix}.samples.tsv"))?);
    if register_samples {
        writeln!(samples, "seconds\taddress\trax\trbx\trcx\trdx\trsi\trdi\trbp\trsp\tr8\tr9\tr10\tr11\tr12\tr13\tr14\tr15")?;
    } else { writeln!(samples, "seconds\taddress")?; }
    let mut command = Command::new(&path);
    command.stdout(File::create(format!("{prefix}.stdout"))?)
        .stderr(File::create(format!("{prefix}.stderr"))?);
    unsafe { command.pre_exec(|| trace(0, 0, 0)); }
    let mut child = command.spawn()?;
    let pid = child.id() as i32;
    let mut status = wait_child(pid)?;
    if status & 0x7f != 0x7f { return Err("target exited before its exec trap".into()); }
    // A tracer exit also closes its child instead of leaving an orphan run.
    trace(0x4200, pid, 1 << 20)?;
    let maps = std::fs::read_to_string(format!("/proc/{pid}/maps"))?;
    let mapping = maps.lines().find(|line| line.ends_with(path.to_str().unwrap()) && line.contains("r--p"))
        .ok_or("ELF load mapping absent")?;
    let fields: Vec<_> = mapping.split_whitespace().collect();
    let base = u64::from_str_radix(fields[0].split('-').next().unwrap(),16)?
        - u64::from_str_radix(fields[2],16)?;
    let mut counter_report = File::create(format!("{prefix}.counters.tsv"))?;
    writeln!(counter_report, "event\tcount\tenabled_ns\trunning_ns\terror")?;
    let mut counters = Vec::new();
    for (name, kind, config) in [("cycles", 0, 0), ("instructions", 0, 1),
        ("cache_references", 0, 2), ("cache_misses", 0, 3),
        ("branch_misses", 0, 5), ("l1d_read_misses", 3, 1 << 16)] {
        match open_counter(pid, kind, config) {
            Ok(counter) => counters.push((name, counter)),
            Err(error) => writeln!(counter_report, "{name}\t\t\t\t{error}")?,
        }
    }
    let sieve_object = vox::loader::elf_object_symbol(&raw, "VOX_SIEVE_COUNTERS")
        .filter(|&(_, size)| size >= 9 * 8);
    let sieve_address = sieve_object.map(|(address, _)| base + address);
    let sieve_fields = sieve_object.map(|(_, size)| (size / 8).min(15)).unwrap_or(0);
    let mut sieve_samples = if sieve_address.is_some() {
        let mut output = BufWriter::new(File::create(format!("{prefix}.sieve.tsv"))?);
        let fields = ["bits", "base_width", "relation_target", "polynomials", "scanned_positions",
            "candidates", "relations", "core_rows", "core_columns", "cofactor_le_base2",
            "cofactor_le_base4", "cofactor_larger", "a_bits", "target_a_bits", "candidate_bits"];
        writeln!(output, "seconds\t{}", fields[..sieve_fields as usize].join("\t"))?;
        Some(output)
    } else { None };
    let ququart_address = vox::loader::elf_object_symbol(&raw, "VOX_QUQUART_COUNTERS")
        .filter(|&(_, size)| size >= 8 * 8).map(|(address, _)| base + address);
    let mut ququart_samples = if ququart_address.is_some() {
        let mut output = BufWriter::new(File::create(format!("{prefix}.ququart.tsv"))?);
        writeln!(output, "seconds\tbits\tphase_digits\tmeasured_digits\tnested_operations\tretained_nodes\tpeak_nodes\tshots_started\tshots_finished")?;
        Some(output)
    } else { None };
    let started = Instant::now();
    let modular_address = vox::loader::elf_object_symbol(&raw, "VOX_MODULAR_COUNTERS")
        .filter(|&(_, size)| size >= 4 * 8).map(|(address, _)| base + address);
    let mut modular_samples = if modular_address.is_some() {
        let mut output = BufWriter::new(File::create(format!("{prefix}.modular.tsv"))?);
        writeln!(output, "seconds\tcalls\tstage\tentry_nodes\texit_nodes")?;
        Some(output)
    } else { None };
    let mut counts = BTreeMap::<String, u64>::new();
    let mut total = 0u64;
    let mut random = pid as u64 | 1;
    trace(7, pid, 0)?;
    loop {
        random ^= random << 13; random ^= random >> 7; random ^= random << 17;
        std::thread::sleep(Duration::from_micros(5_000 + random % 10_000));
        unsafe { kill(pid, 19); }
        status = wait_child(pid)?;
        if status & 0x7f != 0x7f { break; }
        let mut regs = [0u64; 27];
        trace(12, pid, regs.as_mut_ptr() as usize)?;
        let address = regs[16].wrapping_sub(base);
        let index = symbols.partition_point(|(a,_)| *a <= address);
        let name = if regs[16] < base || index == 0 || address > loaded.code.iter()
            .map(|(a,b)| *a + b.len() as u64).max().unwrap_or(0) {
            "[external]".to_string()
        } else { symbols[index-1].1.clone() };
        *counts.entry(name).or_default() += 1;
        total += 1;
        write!(samples, "{:.6}\t{:x}", started.elapsed().as_secs_f64(), address)?;
        if register_samples {
            for index in [10, 5, 11, 12, 13, 14, 4, 19, 9, 8, 7, 6, 3, 2, 1, 0] {
                write!(samples, "\t{:x}", regs[index])?;
            }
        }
        writeln!(samples)?;
        if let (Some(address), Some(output)) = (sieve_address, sieve_samples.as_mut()) {
            write!(output, "{:.6}", started.elapsed().as_secs_f64())?;
            for index in 0..sieve_fields {
                let value = unsafe { ptrace(2, pid, (address + index * 8) as *mut c_void,
                    std::ptr::null_mut()) };
                if value == -1 { return Err(std::io::Error::last_os_error().into()); }
                write!(output, "\t{}", value as u64)?;
            }
            writeln!(output)?;
            if total % 64 == 0 { output.flush()?; }
        }
        if let (Some(address), Some(output)) = (ququart_address, ququart_samples.as_mut()) {
            write!(output, "{:.6}", started.elapsed().as_secs_f64())?;
            for index in 0..8 {
                let value = unsafe { ptrace(2, pid, (address + index * 8) as *mut c_void,
                    std::ptr::null_mut()) };
                if value == -1 { return Err(std::io::Error::last_os_error().into()); }
                write!(output, "\t{}", value as u64)?;
            }
            writeln!(output)?;
            // Keep a bounded interrupted profile useful without changing the
            // target's terminal-only output contract.
            if total % 64 == 0 { output.flush()?; }
        }
        // The signal used to sample is swallowed; unrelated signals keep their semantics.
        if let (Some(address), Some(output)) = (modular_address, modular_samples.as_mut()) {
            write!(output, "{:.6}", started.elapsed().as_secs_f64())?;
            for index in 0..4 {
                let value = unsafe { ptrace(2, pid, (address + index * 8) as *mut c_void,
                    std::ptr::null_mut()) };
                if value == -1 { return Err(std::io::Error::last_os_error().into()); }
                write!(output, "\t{}", value as u64)?;
            }
            writeln!(output)?;
            if total % 64 == 0 { output.flush()?; }
        }
        let signal = (status >> 8) & 0xff;
        trace(7, pid, if signal == 19 || signal == 5 { 0 } else { signal as usize })?;
    }
    let elapsed = started.elapsed().as_secs_f64();
    let _ = child.wait();
    samples.flush()?;
    if let Some(output) = sieve_samples.as_mut() { output.flush()?; }
    if let Some(output) = ququart_samples.as_mut() { output.flush()?; }
    if let Some(output) = modular_samples.as_mut() { output.flush()?; }
    for (name, mut counter) in counters {
        let mut raw = [0u8; 24];
        match counter.read_exact(&mut raw) {
            Ok(()) => {
                let value = u64::from_ne_bytes(raw[0..8].try_into().unwrap());
                let enabled = u64::from_ne_bytes(raw[8..16].try_into().unwrap());
                let running = u64::from_ne_bytes(raw[16..24].try_into().unwrap());
                writeln!(counter_report, "{name}\t{value}\t{enabled}\t{running}\t")?;
            }
            Err(error) => writeln!(counter_report, "{name}\t\t\t\t{error}")?,
        }
    }
    let mut ranking: Vec<_> = counts.into_iter().collect();
    ranking.sort_by_key(|(_,n)| std::cmp::Reverse(*n));
    let mut report = File::create(format!("{prefix}.profile.tsv"))?;
    writeln!(report, "samples\tpercent\tsymbol")?;
    for (name,n) in ranking {
        writeln!(report, "{n}\t{:.3}\t{name}", n as f64 * 100.0 / total.max(1) as f64)?;
    }
    std::fs::write(format!("{prefix}.status"), format!("elapsed_seconds={elapsed:.6}\nsamples={total}\nwait_status={status}\nbinary={}\n", Path::new(file).display()))?;
    if status != 0 { return Err(format!("target terminated with wait status {status:#x}").into()); }
    println!("native profile closed in {elapsed:.3}s; {total} samples; {prefix}.profile.tsv");
    Ok(())
}
