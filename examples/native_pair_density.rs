use std::io::{BufRead,Write};
fn main()->Result<(),Box<dyn std::error::Error>> {
    let mut output=std::io::BufWriter::new(std::io::stdout().lock());
    for line in std::io::stdin().lock().lines() {
        let source:serde_json::Value=serde_json::from_str(&line?)?;
        let returned=vox::protein_pair_density::from_json(&source).map_err(std::io::Error::other)?;
        serde_json::to_writer(&mut output,&returned)?;
        output.write_all(b"\n")?;output.flush()?;
    }
    Ok(())
}
