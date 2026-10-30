//! Export the prepared native electronic graph without a dense matrix.
use std::io::{Read,Write};
fn main()->Result<(),Box<dyn std::error::Error>> {
    let mut text=String::new();std::io::stdin().read_to_string(&mut text)?;
    let source:serde_json::Value=serde_json::from_str(&text)?;
    let operator=vox::protein_electronic_sector::SectorOperator::from_json(&source)
        .map_err(std::io::Error::other)?;
    let mut header=operator.record();
    header["edge_format"]=serde_json::json!("little-endian u32 row, u32 column, f64 value; undirected");
    let mut output=std::io::BufWriter::new(std::io::stdout().lock());
    serde_json::to_writer(&mut output,&header)?;output.write_all(b"\n")?;
    for &(row,column,value) in &operator.edges {
        output.write_all(&(row as u32).to_le_bytes())?;
        output.write_all(&(column as u32).to_le_bytes())?;
        output.write_all(&value.to_le_bytes())?;
    }
    Ok(())
}
