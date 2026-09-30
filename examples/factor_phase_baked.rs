//! Exact CPU coherent execution with baked IMASM input and quantile.
#![deny(warnings)]
extern crate alloc;
extern crate vox as vox_core;
#[path = "../../G-mOMonadOS/src/factor_phase.rs"]
mod factor_phase;
fn main() {
    let output = factor_phase::execute_baked().expect("prepared coherent factor measurement");
    use std::io::{self, Write};
    io::stdout().lock().write_all(output.as_bytes()).unwrap();
}
