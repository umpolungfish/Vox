//! Post-production reference control. This is not linked into the folding producer.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input=std::env::args().nth(1).ok_or("expected observation input")?;
    let v:serde_json::Value=serde_json::from_slice(&std::fs::read(input)?)?;
    if let Some(ensembles)=v["ensembles"].as_array() {
        use imasm_core::imasm16_3::Reg16_3;
        let mut result=Vec::new();
        for ensemble in ensembles {
            let mut predicates=Vec::new();let mut both=0;
            for predicate in ensemble["predicates"].as_array().ok_or("predicates missing")? {
                let mut state=Reg16_3::default();
                for member in ensemble["members"].as_array().ok_or("member identities missing")? {
                    let t=predicate["support_ids"].as_array().ok_or("support identities missing")?.contains(member);
                    let f=predicate["refute_ids"].as_array().ok_or("refutation identities missing")?.contains(member);
                    state=state.union(Reg16_3{big_t:t,big_f:f,small_t:false,small_f:false});
                }
                if state.big_t&&state.big_f{both+=1;}
                let mut returned=predicate.clone();returned["native_return"]=serde_json::json!({"register":state.name(),"glyph":state.glyph().to_string(),"T":state.big_t,"F":state.big_f,"t":state.small_t,"f":state.small_f});predicates.push(returned);
            }
            result.push(serde_json::json!({"name":ensemble["name"],"members":ensemble["members"],"predicates":predicates,"both_count":both}));
        }
        println!("{}",serde_json::to_string_pretty(&serde_json::json!({"scope":"native Reg16_3 information union; whole member identities and all witnesses retained","ensembles":result}))?);
        return Ok(());
    }
    let chain:Vec<String>=serde_json::from_value(v["chain"].clone())?;
    let chain=chain.iter().map(String::as_str).collect::<Vec<_>>();
    let mut observations=Vec::new();
    for model in v["models"].as_array().ok_or("models missing")? {
        let bb:Vec<[(f64,f64,f64);4]>=serde_json::from_value(model["backbone"].clone())?;
        let sides:Vec<Vec<(String,(f64,f64,f64))>>=serde_json::from_value(model["sidechains"].clone())?;
        let sides=sides.iter().map(|r|r.iter().map(|(n,p)|(n.as_str(),*p)).collect::<Vec<_>>()).collect::<Vec<_>>();
        let steps:Vec<(f64,f64)>=serde_json::from_value(model["steps"].clone())?;
        let e=vox::peptide_closure::assess(&chain,&bb,&sides,&steps);
        observations.push(serde_json::json!({"closed":e.closed(),"atoms":e.atoms,
            "max_bond_error":e.max_bond_error,"max_overlap":e.max_overlap,"stored_overlap_potential":e.overlap_potential,
            "issues":e.issues.iter().map(|i|serde_json::json!({"kind":i.kind,"detail":i.detail})).collect::<Vec<_>>(),
            "native_report":e.report(true)}));
    }
    println!("{}",serde_json::to_string_pretty(&serde_json::json!({"scope":"post-production reference geometry control only","models":observations}))?);
    Ok(())
}
