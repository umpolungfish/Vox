#![deny(warnings)]
#[path = "factor_membrane_support.rs"]
mod support;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match vox::membrane_factor::command(&args) {
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
            support::append_trilattice_reads(&report);
        }
        Err(error) => {
            eprintln!("factor-membrane: {error}");
            std::process::exit(2);
        }
    }
}
