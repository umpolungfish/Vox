#![deny(warnings)]
#[path = "factor_membrane_support.rs"]
mod support;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|arg| arg == "--dialectic-product") {
        let code = match support::run_dialectic_product(&args) {
            Ok(code) => code,
            Err(error) => { eprintln!("factor-membrane: {error}"); 2 }
        };
        std::process::exit(code);
    }
    let (routed_args, gpu_producer) = match support::route_n_only_to_gpu(&args) {
        Ok(route) => route,
        Err(error) => {
            eprintln!("factor-membrane GPU producer: {error}");
            std::process::exit(2);
        }
    };
    match vox::membrane_factor::command(&routed_args) {
        Ok(report) => {
            if let Some(pos) = report.find("[EMITTED_MODULE_DESTINATION: ") {
                let after = &report[pos + "[EMITTED_MODULE_DESTINATION: ".len()..];
                if let Some(end_bracket) = after.find(']') {
                    let dest_path = &after[..end_bracket];
                    let code = after[end_bracket + 1..].trim_start();
                    if let Err(e) = std::fs::write(dest_path, code) {
                        eprintln!("warning: could not write {dest_path}: {e}");
                    } else {
                        println!("[EMITTED] Module written to {dest_path}");
                    }
                }
            }
            print!("{report}");
            if let Some(producer) = gpu_producer {
                println!("GPU factor producer = {producer}");
            }
            support::append_trilattice_reads(&report);
        }
        Err(error) => {
            eprintln!("factor-membrane: {error}");
            std::process::exit(2);
        }
    }
}
