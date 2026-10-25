#![deny(warnings)]
#[path = "../descent_cli.rs"]
mod descent_cli;

fn main() {
    #[cfg(feature = "hosted")]
    ::vox::ordering::cli::intercept();
    let args: Vec<String> = std::env::args().skip(1).collect();
    match descent_cli::execute(&args) {
        Ok(true) => {}
        Ok(false) => std::process::exit(1),
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(2);
        }
    }
}
