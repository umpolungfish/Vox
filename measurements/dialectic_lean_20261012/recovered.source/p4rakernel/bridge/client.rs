// Shared hosted client. The evaluator and certificates live in Lean.
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

struct BridgeOutput {
    code: i32,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

fn capture_files(
    root: &std::path::Path,
    sequence: &AtomicU64,
    pid: u32,
) -> Result<(PathBuf, PathBuf, File, File), String> {
    loop {
        let number = sequence.fetch_add(1, Ordering::Relaxed);
        let stem = format!(".codex-bridge-{pid}-{number}");
        let stdout_path = root.join("build").join(format!("{stem}.stdout"));
        let stderr_path = root.join("build").join(format!("{stem}.stderr"));
        let stdout = match std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&stdout_path)
        {
            Ok(file) => file,
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error.to_string()),
        };
        match std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&stderr_path)
        {
            Ok(stderr) => return Ok((stdout_path, stderr_path, stdout, stderr)),
            Err(error) => {
                drop(stdout);
                let _ = std::fs::remove_file(stdout_path);
                if error.kind() != std::io::ErrorKind::AlreadyExists {
                    return Err(error.to_string());
                }
            }
        }
    }
}

#[cfg(test)]
mod capture_tests {
    use super::*;
    #[test]
    fn identical_namespace_pids_reserve_distinct_capture_pairs() {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("measurements/modular_residue_20261010")
            .join(format!(
                "capture-reservation-{}-{stamp}",
                std::process::id()
            ));
        std::fs::create_dir_all(root.join("build")).unwrap();
        let occupied_out = root.join("build/.codex-bridge-12-0.stdout");
        let occupied_err = root.join("build/.codex-bridge-12-1.stderr");
        std::fs::write(&occupied_out, b"existing stdout").unwrap();
        std::fs::write(&occupied_err, b"existing stderr").unwrap();
        let first = capture_files(&root, &AtomicU64::new(0), 12).unwrap();
        let second = capture_files(&root, &AtomicU64::new(0), 12).unwrap();
        assert_ne!(first.0, second.0);
        assert_ne!(first.1, second.1);
        assert_eq!(std::fs::read(occupied_out).unwrap(), b"existing stdout");
        assert_eq!(std::fs::read(occupied_err).unwrap(), b"existing stderr");
        assert!(!root.join("build/.codex-bridge-12-1.stdout").exists());
    }
}

fn kernel_root() -> PathBuf {
    std::env::var_os("P4RAKERNEL_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../p4rakernel"))
}

/// Capture through files so a Lake/Lean descendant inheriting a pipe cannot
/// keep the hosted caller waiting after the evaluator itself has exited.
fn run_captured(args: &[String], completion_marker: Option<&str>) -> Result<BridgeOutput, String> {
    static NEXT_CAPTURE: AtomicU64 = AtomicU64::new(0);
    let root = kernel_root();
    // Separate PID namespaces can share this filesystem and the same PID.
    // Exclusive creation reserves the pair across all hosted frontends.
    let (stdout_path, stderr_path, stdout, stderr) =
        capture_files(&root, &NEXT_CAPTURE, std::process::id())?;
    let result = (|| {
        let mut child = evaluator()?
            .args(args)
            .stdin(Stdio::null())
            .stdout(stdout)
            .stderr(stderr)
            .spawn()
            .map_err(|error| error.to_string())?;
        let (status, marker_seen) = loop {
            let stdout = std::fs::read(&stdout_path).map_err(|error| error.to_string())?;
            if completion_marker.is_some_and(|marker| {
                String::from_utf8_lossy(&stdout)
                    .lines()
                    .any(|line| line.contains(marker))
            }) {
                let status = child.wait().map_err(|error| error.to_string())?;
                break (Some(status), true);
            }
            match child.try_wait().map_err(|error| error.to_string())? {
                Some(status) => break (Some(status), false),
                None => std::thread::sleep(Duration::from_millis(10)),
            }
        };
        let stdout = std::fs::read(&stdout_path).map_err(|error| error.to_string())?;
        let stderr = std::fs::read(&stderr_path).map_err(|error| error.to_string())?;
        let code = status.and_then(|result| result.code()).unwrap_or_else(|| {
            if !marker_seen {
                return 2;
            }
            if args.first().is_some_and(|arg| arg == "product")
                && String::from_utf8_lossy(&stdout)
                    .lines()
                    .any(|line| line.starts_with("product-certificate=false "))
            {
                2
            } else {
                0
            }
        });
        Ok(BridgeOutput {
            code,
            stdout,
            stderr,
        })
    })();
    let _ = std::fs::remove_file(stdout_path);
    let _ = std::fs::remove_file(stderr_path);
    result
}

pub fn run(args: &[String]) -> i32 {
    run_to(args, |bytes| {
        let _ = std::io::stdout().lock().write_all(bytes);
    })
}

pub fn run_to(args: &[String], emit: impl FnOnce(&[u8])) -> i32 {
    if args.is_empty() || matches!(args[0].as_str(), "help" | "--help" | "-h") {
        emit(b"reentry [orbit SEED TICK] identity|negation|hold|seeded MASK|constant MASK|Y MASK|OP MASK\nreentry [orbit SEED TICK] program TOKEN ... (@ applies the two preceding terms)\nreentry word GLYPHS [SEED TICK]\nreentry table SEED TICK OUTPUT0 ... OUTPUT15\nreentry product SOURCE LEFT RIGHT (full-width arithmetic certificate)\nOP: join_i meet_i join_t meet_t join_f meet_f; native mask bits: N=1 T=2 F=4 B=8\n");
        return 0;
    }
    let completion_marker = if args.first().is_some_and(|arg| arg == "product") {
        Some("evidence-certified=true")
    } else {
        None
    };
    match run_captured(args, completion_marker) {
        Ok(output) => {
            emit(&output.stdout);
            if output.code != 0 {
                let _ = std::io::stderr().lock().write_all(&output.stderr);
            }
            output.code
        }
        Err(error) => {
            eprintln!("Lean kernel bridge: {error}");
            2
        }
    }
}

fn evaluator() -> Result<Command, String> {
    let root = kernel_root();
    let compiled = root.join("p4ramill/.lake/build/bin/kernel_bridge");
    if compiled.is_file() {
        return Ok(Command::new(compiled));
    }
    let compiler_dir = root.join("build/stage1/bin");
    let mut paths = vec![compiler_dir.clone()];
    if let Some(path) = std::env::var_os("PATH") {
        paths.extend(std::env::split_paths(&path));
    }
    let path = std::env::join_paths(paths).map_err(|error| error.to_string())?;
    let mut command = Command::new(compiler_dir.join("lake"));
    command
        .current_dir(root.join("p4ramill"))
        .env("PATH", path)
        .args(["env", "lean", "--run", "KernelBridge.lean"]);
    Ok(command)
}

/// Hosted admission requires the actual Lean carrier and certified Y reducer.
pub fn ensure_default() -> Result<u8, String> {
    static READY: std::sync::OnceLock<Result<u8, String>> = std::sync::OnceLock::new();
    READY
        .get_or_init(|| {
            let output = run_captured(&["Y".into(), "6".into()], Some("cycle-certified=true"))?;
            let report = String::from_utf8(output.stdout).map_err(|e| e.to_string())?;
            if output.code != 0 {
                return Err(String::from_utf8_lossy(&output.stderr).into_owned());
            }
            if !report
                .lines()
                .any(|line| line == "backend=Lean-kernel SIXTEEN_3=active mask-order=N,T,F,B")
                || !report.lines().any(|line| line == "kernel-reentry=6")
                || !report
                    .lines()
                    .any(|line| line == "least=6 greatest=6 certified=true")
            {
                return Err(
                    "Lean did not certify kernel re-entry and the default recursive carrier".into(),
                );
            }
            Ok(6)
        })
        .clone()
}

pub fn run_word(args: &[String], table: impl FnOnce(&str) -> Result<[u8; 16], String>) -> i32 {
    run_word_to(args, table, |bytes| {
        let _ = std::io::stdout().lock().write_all(bytes);
    })
}

pub fn run_word_to(
    args: &[String],
    table: impl FnOnce(&str) -> Result<[u8; 16], String>,
    emit: impl FnOnce(&[u8]),
) -> i32 {
    if args.len() != 2 && args.len() != 4 {
        eprintln!("reentry word GLYPHS [SEED TICK]");
        return 2;
    }
    let map = match table(&args[1]) {
        Ok(map) => map,
        Err(error) => {
            eprintln!("phase word: {error}");
            return 2;
        }
    };
    let mut request = vec![
        "table".to_owned(),
        args.get(2).cloned().unwrap_or_else(|| "0".into()),
        args.get(3).cloned().unwrap_or_else(|| "4".into()),
    ];
    request.extend(map.into_iter().map(|value| value.to_string()));
    run_to(&request, emit)
}
