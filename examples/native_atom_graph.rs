fn main()->Result<(),Box<dyn std::error::Error>> {
    let charges=std::env::args().skip(1).map(|x|x.parse::<usize>()).collect::<Result<Vec<_>,_>>()?;
    let family=vox::protein_atom_graph::AtomFamily::prepare(&charges).map_err(std::io::Error::other)?;
    let mut record=family.record();
    record["four_character_directions"]=serde_json::json!(
        vox::protein_site_geometry::Frames::new(&[(0.,0.,0.)]).sites[0]);
    println!("{}",record);Ok(())
}
