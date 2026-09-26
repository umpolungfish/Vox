fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match vox::membrane_factor::command(&args) {
        Ok(report) => print!("{report}"),
        Err(error) => {
            eprintln!("factor-membrane: {error}");
            std::process::exit(2);
        }
    }
}
