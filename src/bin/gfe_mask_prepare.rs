//! Host-side compiler stage for the finite small-prime GFE orbit mask.
#![deny(warnings)]

fn main() {
    #[cfg(feature = "hosted")]
    ::vox::ordering::cli::intercept();
    match vox::geometer_factor_extractor::SmallPrimeOrbitMask::prepare() {
        Ok(mask) => println!("{}", mask.word().as_str()),
        Err(error) => {
            eprintln!("GFE orbit-mask preparation failed: {error}");
            std::process::exit(2);
        }
    }
}
