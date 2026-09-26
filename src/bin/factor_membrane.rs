#[path = "factor_membrane_support.rs"]
mod support;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match vox::membrane_factor::command(&args) {
        Ok(report) => {
            print!("{report}");
            support::append_trilattice_reads(&report);
        }
        Err(error) => {
            eprintln!("factor-membrane: {error}");
            std::process::exit(2);
        }
    }
}
