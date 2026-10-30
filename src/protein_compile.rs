//! Shared hosted protein compiler for standalone Vox and G-mOMonadOS.
//! Both entry points execute the same source-bound carrier and coordinate path.
use std::{string::{String, ToString}, vec::Vec, println, eprintln, print};
use crate::{genetic, protein, fold, fold3d};

fn route_branch(glut_path:&str,pdb_path:&str,chain:&[&str])->Result<crate::protein_native::RouteBranch,String>{
    let glut_bytes=std::fs::read(glut_path).map_err(|e|format!("cannot read route carrier: {e}"))?;
    let envelope:serde_json::Value=serde_json::from_slice(&glut_bytes).map_err(|e|format!("invalid route carrier: {e}"))?;
    let protein=&envelope["protein_glut"];
    let residues=protein["residues"].as_array().ok_or("route carrier has no residues")?;
    if residues.len()!=chain.len()||residues.iter().zip(chain).any(|(r,aa)|r["aa"].as_str()!=Some(aa)){
        return Err("route carrier sequence differs from the native source".into());
    }
    let expected=envelope["pdb_sha256"].as_str().filter(|s|!s.is_empty()).ok_or("route carrier does not bind a PDB")?;
    let hash=|path:&str|->Result<String,String>{
        let out=std::process::Command::new("sha256sum").arg("--").arg(path).output()
            .map_err(|e|format!("cannot hash route input: {e}"))?;
        if !out.status.success(){return Err(format!("cannot hash route input {path}"));}
        String::from_utf8(out.stdout).map_err(|e|e.to_string())?.split_whitespace().next()
            .map(|s|s.to_string()).ok_or("sha256sum returned no digest".into())
    };
    let pdb_hash=hash(pdb_path)?;
    if pdb_hash!=expected{return Err("route PDB differs from its carrier binding".into());}
    let pdb=std::fs::read_to_string(pdb_path).map_err(|e|format!("cannot read route PDB: {e}"))?;
    let atoms:Vec<usize>=pdb.lines().filter(|line|line.starts_with("ATOM  ")||line.starts_with("HETATM"))
        .map(|line|line.get(22..26).ok_or("short route atom record")?.trim().parse::<usize>()
            .map_err(|e|format!("invalid route residue address: {e}")))
        .collect::<Result<_,_>>()?;
    if atoms.len()!=protein["observations"].as_array().ok_or("route carrier has no observations")?
        .iter().find(|o|o["name"]=="coordinate-geometry").ok_or("route has no geometry observation")?["values"]["atoms"].as_u64().ok_or("route geometry has no atom count")? as usize{
        return Err("route atom records disagree with the retained geometry boundary".into());
    }
    if atoms.iter().any(|&r|r==0||r>chain.len()){
        return Err("route atom address lies outside the carried sequence".into());
    }
    let interactions=protein["observations"].as_array().unwrap().iter()
        .find(|o|o["name"]=="lattice-interaction-work").ok_or("route has no interaction return")?;
    let pairs=interactions["values"]["pairs"].as_array().ok_or("route has no pair interactions")?;
    let mut joined=std::collections::BTreeMap::<(usize,usize),u8>::new();
    for pair in pairs{
        let ids=pair["atoms"].as_array().ok_or("route pair has no atom addresses")?;
        if ids.len()!=2{return Err("route pair has invalid atom addresses".into());}
        let (a,b)=(ids[0].as_u64().ok_or("invalid route atom address")? as usize,
                   ids[1].as_u64().ok_or("invalid route atom address")? as usize);
        let (ra,rb)=(*atoms.get(a).ok_or("route pair atom exceeds PDB")?,*atoms.get(b).ok_or("route pair atom exceeds PDB")?);
        let key=(ra.min(rb)-1,ra.max(rb)-1);
        for term in pair["terms"].as_array().ok_or("route pair has no source terms")?{
            let bit=match term["member"].as_str(){Some("T")=>1u8<<crate::belnap::B4::T.to_u8(),Some("F")=>1u8<<crate::belnap::B4::F.to_u8(),
                Some("N")=>continue,Some("B")=>1u8<<crate::belnap::B4::B.to_u8(),_=>return Err("unknown route support member".into())};
            *joined.entry(key).or_default()|=bit;
        }
    }
    let residue_pairs=joined.into_iter().filter_map(|((i,j),mask)|
        (i!=j&&mask&( (1u8<<crate::belnap::B4::T.to_u8())|(1u8<<crate::belnap::B4::F.to_u8()) )
            ==( (1u8<<crate::belnap::B4::T.to_u8())|(1u8<<crate::belnap::B4::F.to_u8()) ))
            .then_some((i.min(j),i.max(j),mask))).collect();
    let glut_sha256=hash(glut_path)?;
    Ok(crate::protein_native::RouteBranch{source:glut_path.into(),glut_sha256,pdb_sha256:pdb_hash,residue_pairs})
}

/// `vox compile <seq> [--code standard|mitochondrial] [--carrier <path>]`
///
/// One entry point, both directions of the same pipeline. RNA/DNA in:
/// `protein::translate_full` (every residue, not just the twelve promoted
/// ones `genetic::lift_rna` keeps), then native-unit Glut folding. Protein
/// in: reverse-translate with the Frobenius-preferred codon per residue
/// (`protein::preferred_codon_for_aa`) and run the SAME fold on the input,
/// so the "vice versa" direction carries fold info too. Direction
/// auto-detects from the input alphabet: pure A/C/G/T/U reads as nucleic
/// acid, anything else as protein codes.
pub fn run(args: &[String]) -> i32 {
    let mut dialect = String::new();
    let mut pdb_path: Option<String> = None;
    let mut carrier_path: Option<String> = None;
    let mut input_kind = String::from("auto");
    let mut coords_mode = false;
    let mut native_lattice = true;
    let mut observation_steps = None;
    let mut resume_path = None;
    let mut route_glut_path:Option<String>=None;
    let mut route_pdb_path:Option<String>=None;
    let mut seq_parts: Vec<String> = Vec::new();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--help" | "-h" => {
                println!("vox compile <seq> | --fasta <file> [--input auto|rna|dna|protein] [--code standard|mitochondrial] [--carrier <path>] [--steps <count>] [--resume <carrier>] [--route-branch <glut.json> --route-branch-pdb <pdb>] [--coords] [--pdb <path>] [--configured]");
                println!("Execute native Glut parameters and FOUR-derived geometry in membership units by default. --carrier exports the source-bound residue-centre structure and full lattice interactions; --coords prints native positions. --input protein resolves nucleotide-alphabet ambiguity. --fasta reads one record.");
                println!("--lattice explicitly selects the default native path. --configured selects the earlier model with supplied physical parameters and enables PDB export.");
                println!("--steps bounds the native observation interval and returns the complete held carrier.");
                return 0;
            }
            "--resume" if i+1<args.len() => {resume_path=Some(args[i+1].clone());i+=2;}
            "--route-branch" if i+1<args.len() => {route_glut_path=Some(args[i+1].clone());i+=2;}
            "--route-branch-pdb" if i+1<args.len() => {route_pdb_path=Some(args[i+1].clone());i+=2;}
            "--coords" => { coords_mode = true; i += 1; }
            "--lattice" => { native_lattice = true; i += 1; }
            "--configured" => { native_lattice = false; i += 1; }
            "--steps" if i + 1 < args.len() => {
                match args[i+1].parse::<usize>() {
                    Ok(count) if count>0 => observation_steps=Some(count),
                    _ => {eprintln!("--steps expects a positive integer");return 1;}
                }
                i += 2;
            }
            "--code" if i + 1 < args.len() => {
                if !["standard","mitochondrial","mito"].contains(&args[i+1].as_str()) {
                    eprintln!("unknown genetic code {}",args[i+1]);return 1;
                }
                dialect = if args[i + 1] == "mitochondrial" || args[i + 1] == "mito" {
                    String::from("mitochondrial")
                } else { String::new() };
                i += 2;
            }
            "--pdb" if i + 1 < args.len() => { pdb_path = Some(args[i + 1].clone()); i += 2; }
            "--carrier" if i + 1 < args.len() => { carrier_path = Some(args[i + 1].clone()); i += 2; }
            "--input" if i + 1 < args.len() => {
                input_kind = args[i+1].clone();
                if !["auto","rna","dna","protein"].contains(&input_kind.as_str()) {
                    eprintln!("--input expects auto, rna, dna or protein"); return 1;
                }
                i += 2;
            }
            "--fasta" if i + 1 < args.len() => {
                let text = match std::fs::read_to_string(&args[i+1]) {
                    Ok(text) => text, Err(e) => {eprintln!("cannot read FASTA: {}",e);return 1;}
                };
                if text.lines().filter(|line|line.starts_with('>')).count() != 1 {
                    eprintln!("--fasta expects exactly one record");return 1;
                }
                seq_parts.push(genetic::protein_from_fasta(&text)); i += 2;
            }
            other if other.starts_with("--") => {eprintln!("unknown or incomplete compile option: {}",other);return 1;}
            other => { seq_parts.push(other.to_string()); i += 1; }
        }
    }
    let seq = seq_parts.join(" ");
    if observation_steps.is_some() && !native_lattice {
        eprintln!("--steps requires native folding");return 1;
    }
    if route_glut_path.is_some()!=route_pdb_path.is_some(){
        eprintln!("--route-branch and --route-branch-pdb must be supplied together");return 1;
    }
    if resume_path.is_some()&&route_glut_path.is_some(){
        eprintln!("route branches enter native preparation; they cannot be added to a held return");return 1;
    }
    if let Some(path)=resume_path {
        if !native_lattice||!seq_parts.is_empty(){
            eprintln!("--resume uses the native source already retained in the carrier");return 1;
        }
        let operation=(||->Result<(),String>{
            use std::io::Write;
            let bytes=std::fs::read(&path).map_err(|e|format!("cannot read held carrier: {e}"))?;
            let mut held:serde_json::Value=serde_json::from_slice(&bytes).map_err(|e|e.to_string())?;
            drop(bytes);
            crate::protein_records::compact_held(&mut held);
            let mut result=crate::protein_native::resume_saved(&held,observation_steps)?;
            result["source"]=held["source"].clone();
            let before=held["continuation"]["next_clock"].clone();
            let after=result["continuation"]["next_clock"].clone();
            let mut segments=held["source"]["execution_segments"].as_array().cloned().unwrap_or_else(||vec![
                serde_json::json!({"start_clock":0,"end_clock":before,
                    "source_binding":held["source"]["source_binding"]})]);
            segments.push(serde_json::json!({"start_clock":before,"end_clock":after,
                "source_binding":crate::protein_glut::ProteinGlut::source_binding()}));
            result["source"]["execution_segments"]=serde_json::json!(segments);
            if let Some(path)=pdb_path {
                let residues=result["source"]["residues"].as_array().ok_or("held source has no residues")?;
                let positions=result["positions"].as_array().ok_or("held carrier has no positions")?;
                if residues.len()!=positions.len(){return Err("held residue and coordinate counts differ".into());}
                let torsions:Vec<f64>=serde_json::from_value(result["torsions"].clone()).map_err(|e|e.to_string())?;
                if torsions.len()!=positions.len(){return Err("held torsion and coordinate counts differ".into());}
                let anchors:Vec<[f64;3]>=positions.iter().map(|point|Ok([
                    point[0].as_f64().ok_or("invalid native coordinate")?,
                    point[1].as_f64().ok_or("invalid native coordinate")?,
                    point[2].as_f64().ok_or("invalid native coordinate")?]))
                    .collect::<Result<_,String>>()?;
                let add=|a:[f64;3],b:[f64;3]|[a[0]+b[0],a[1]+b[1],a[2]+b[2]];
                let sub=|a:[f64;3],b:[f64;3]|[a[0]-b[0],a[1]-b[1],a[2]-b[2]];
                let scale=|a:[f64;3],s:f64|[a[0]*s,a[1]*s,a[2]*s];
                let dot=|a:[f64;3],b:[f64;3]|a[0]*b[0]+a[1]*b[1]+a[2]*b[2];
                let cross=|a:[f64;3],b:[f64;3]|[a[1]*b[2]-a[2]*b[1],a[2]*b[0]-a[0]*b[2],a[0]*b[1]-a[1]*b[0]];
                let unit=|a:[f64;3]|{let length=dot(a,a).sqrt();if length>1e-12{scale(a,1.0/length)}else{[1.0,0.0,0.0]}};
                let mut pdb=String::from("HEADER    NATIVE GLUT ENSEMBLE BACKBONE SCAFFOLD\nREMARK 900 CA anchors are carried native coordinates in membership units, not Angstroms.\nREMARK 900 N,C,O,CB atoms are a derived local scaffold rotated by retained virtual torsions.\n");
                let mut serials=Vec::with_capacity(positions.len());
                let mut serial=0u32;
                for (index,(residue,ca)) in residues.iter().zip(&anchors).enumerate(){
                    let aa=residue["aa"].as_str().ok_or("held residue has no amino-acid code")?;
                    let res3=match aa {"A"|"Ala"=>"ALA","R"|"Arg"=>"ARG","N"|"Asn"=>"ASN","D"|"Asp"=>"ASP","C"|"Cys"=>"CYS","Q"|"Gln"=>"GLN","E"|"Glu"=>"GLU","G"|"Gly"=>"GLY","H"|"His"=>"HIS","I"|"Ile"=>"ILE","L"|"Leu"=>"LEU","K"|"Lys"=>"LYS","M"|"Met"=>"MET","F"|"Phe"=>"PHE","P"|"Pro"=>"PRO","S"|"Ser"=>"SER","T"|"Thr"=>"THR","W"|"Trp"=>"TRP","Y"|"Tyr"=>"TYR","V"|"Val"=>"VAL",_=>"UNK"};
                    let previous=if index>0{sub(*ca,anchors[index-1])}else{sub(anchors[index+1],*ca)};
                    let next=if index+1<anchors.len(){sub(anchors[index+1],*ca)}else{sub(*ca,anchors[index-1])};
                    let tangent=unit(add(previous,next));
                    let mut normal=cross(previous,next);
                    if dot(normal,normal)<1e-20{normal=cross(tangent,[0.0,0.0,1.0]);if dot(normal,normal)<1e-20{normal=cross(tangent,[0.0,1.0,0.0]);}}
                    normal=unit(normal);
                    normal=unit(sub(normal,scale(tangent,dot(normal,tangent))));
                    let angle=torsions[index];
                    normal=unit(add(scale(normal,libm::cos(angle)),scale(cross(tangent,normal),libm::sin(angle))));
                    let binormal=unit(cross(tangent,normal));
                    let left=if index>0{dot(sub(*ca,anchors[index-1]),sub(*ca,anchors[index-1])).sqrt()}else{1.0};
                    let right=if index+1<anchors.len(){dot(sub(anchors[index+1],*ca),sub(anchors[index+1],*ca)).sqrt()}else{left};
                    let unit_scale=(left+right)/2.0;
                    let n=add(*ca,add(scale(tangent,-0.36*unit_scale),scale(normal,0.15*unit_scale)));
                    let c=add(*ca,add(scale(tangent,0.36*unit_scale),scale(normal,0.15*unit_scale)));
                    let o=add(c,add(scale(tangent,0.14*unit_scale),scale(normal,0.24*unit_scale)));
                    let cb=add(*ca,add(scale(normal,-0.38*unit_scale),scale(binormal,0.12*unit_scale)));
                    let atoms=[(" N  "," N",n),(" CA "," C",*ca),(" C  "," C",c),(" O  "," O",o)];
                    let mut ids=[0u32;5];
                    for (slot,(name,element,xyz)) in atoms.into_iter().enumerate(){
                        serial+=1;ids[slot]=serial;
                        pdb.push_str(&format!("ATOM  {:>5} {} {} A{:>4}    {:>8.3}{:>8.3}{:>8.3}{:>6.2}{:>6.2}          {:>2}\n",serial,name,res3,index+1,xyz[0],xyz[1],xyz[2],1.0,0.0,element));
                    }
                    if aa!="G"&&aa!="Gly"{
                        serial+=1;ids[4]=serial;
                        pdb.push_str(&format!("ATOM  {:>5}  CB  {} A{:>4}    {:>8.3}{:>8.3}{:>8.3}{:>6.2}{:>6.2}           C\n",serial,res3,index+1,cb[0],cb[1],cb[2],1.0,0.0));
                    }
                    serials.push(ids);
                }
                pdb.push_str("TER\n");
                for (index,ids) in serials.iter().enumerate(){
                    for (a,b) in [(0,1),(1,2),(2,3),(1,4)]{if ids[b]!=0{pdb.push_str(&format!("CONECT{:>5}{:>5}\n",ids[a],ids[b]));}}
                    if index+1<serials.len(){pdb.push_str(&format!("CONECT{:>5}{:>5}\n",ids[2],serials[index+1][0]));}
                }
                pdb.push_str("END\n");
                std::fs::write(&path,pdb.as_bytes()).map_err(|e|format!("cannot write {}: {}",path,e))?;
                println!("membership-unit backbone scaffold PDB written to {}",path);
            }
            println!("resumed native ensemble: clock {} -> {}; {}",before,after,result["continuation"]["reason"]);
            println!("deformation {}; retained strain {}; {} accepted transformations",
                result["final_deformation"],result["retained_strain"],result["accepted_steps"].as_array().unwrap().len());
            if coords_mode{for (i,p) in result["positions"].as_array().unwrap().iter().enumerate(){
                println!("{} {} {} {}",i+1,p[0],p[1],p[2]);
            }}
            if let Some(output)=carrier_path {
                crate::protein_records::compact_held(&mut result);
                let file=std::fs::File::create(&output).map_err(|e|e.to_string())?;
                let mut writer=std::io::BufWriter::new(file);
                serde_json::to_writer_pretty(&mut writer,&result).map_err(|e|e.to_string())?;
                writer.flush().map_err(|e|e.to_string())?;
                println!("carrier written to {}",output);
            }
            Ok(())
        })();
        return match operation{Ok(())=>0,Err(e)=>{eprintln!("{e}");1}};
    }
    let dialect_name = if dialect.is_empty() { "standard" } else { "mitochondrial" };

    let compact: String = seq.chars().filter(|c| !c.is_whitespace() && *c != '-' && *c != ',').collect();
    let nucleic_alphabet = !compact.is_empty()
        && compact.chars().all(|c| matches!(c.to_ascii_uppercase(), 'A' | 'C' | 'G' | 'T' | 'U'));
    if matches!(input_kind.as_str(), "rna"|"dna") && !nucleic_alphabet {
        eprintln!("nucleic input contains a non-nucleotide");return 1;
    }
    let is_nucleic = matches!(input_kind.as_str(),"rna"|"dna") || input_kind=="auto" && nucleic_alphabet;

    fn report_fold(chain: &[&'static str], b4_path: &[char], mrna: &str, start: usize,
                   pdb_path: Option<&str>, carrier_path: Option<&str>, coords_mode: bool,
                   native_lattice: bool, observation_steps: Option<usize>,
                   route_paths:Option<(&str,&str)>) -> Result<(), String> {
        let carrier = crate::protein_glut::ProteinGlut::prepare(chain, mrna, start)?;
        println!("native protein B: {} residues, {} chemical forms; full self-marks executed",
            carrier.residues.len(), carrier.residues.iter().map(|r|r.forms.len()).sum::<usize>());
        if native_lattice {
            if pdb_path.is_some(){return Err("native folding exports membership units through --carrier; PDB requires a derived unit conversion".into());}
            let route=route_paths.map(|(glut,pdb)|route_branch(glut,pdb,chain)).transpose()?;
            let mut result=crate::protein_native::fold_programs_observed_with_branch(&carrier.native_programs(),observation_steps,route.as_ref())?;
            result["source"]=carrier.source_record();
            println!("information-order closure: {} maximal joint observations, {} retained information gains",
                result["maximal_joint_observations"],result["information_transition_count"]);
            println!("mechanical readout: deformation {} -> {}, retained strain {}, {} accepted coordinate steps",
                result["initial_deformation"],result["final_deformation"],result["retained_strain"],
                result["accepted_steps"].as_array().unwrap().len());
            println!("geometry: native membership units; {} retained residue centres",chain.len());
            if coords_mode {
                for (i,p) in result["positions"].as_array().unwrap().iter().enumerate(){
                    println!("{} {} {} {} {}",i+1,chain[i],p[0],p[1],p[2]);
                }
            }
            if let Some(path)=carrier_path {
                use std::io::Write;
                if route.is_some(){crate::protein_records::compact_held(&mut result);}
                let file=std::fs::File::create(path).map_err(|e|format!("cannot write native folding return: {e}"))?;
                let mut writer=std::io::BufWriter::new(file);
                serde_json::to_writer_pretty(&mut writer,&result).map_err(|e|e.to_string())?;
                writer.flush().map_err(|e|format!("cannot finish native folding return: {e}"))?;
                println!("carrier written to {}",path);
            }
            return Ok(());
        }
        if route_paths.is_some(){return Err("route branch input applies only to native folding".into());}
        let f = fold::fold_sequence(chain);
        let n_h = f.residues.iter().filter(|r| r.secondary == fold::SecondaryLabel::Helix).count();
        let n_s = f.residues.iter().filter(|r| r.secondary == fold::SecondaryLabel::Sheet).count();
        let n_c = f.residues.len() - n_h - n_s;
        println!();
        println!("sequence fold: helix {}  sheet {}  coil {}   ({} contacts, SerpentRod contact invariant {})",
            n_h, n_s, n_c, f.contacts.len(), if f.frobenius_ok { "PASS" } else { "FAIL" });
        let initial_steps = fold3d::rama_steps(b4_path);
        let contacts: Vec<_> = f.contacts.iter().map(|c|crate::protein_relax::Contact{i:c.i,j:c.j}).collect();
        let relaxed = crate::protein_relax::run(chain,&initial_steps,&contacts,&carrier.words(),160)?;
        println!("contact-driven torsion work: {} accepted, score {} -> {}, {} full form words, {} payload moves",
            relaxed.accepted,relaxed.initial,relaxed.final_score,relaxed.word_calls,relaxed.payload_moves);
        let interaction_return=serde_json::json!({
            "schema":"protein-lattice-interactions/v1",
            "native_membership_basis":["N","T","F","B"],
            "realization":{"T":"configured contact potential","F":"configured excluded-volume potential",
                "contact_cutoff":8.0,"steric_coefficient":10.0,
                "clash_overlap":crate::peptide_closure::CLASH_OVERLAP},
            "initial_potential":relaxed.initial,"final_potential":relaxed.final_score,
            "potential_drop":relaxed.initial-relaxed.final_score,
            "pairs":relaxed.interactions.iter().map(|r|r.json()).collect::<Vec<_>>()});
        let steps = relaxed.steps;
        let backbone = relaxed.backbone;
        println!("backbone {} residues placed (contact-relaxed NeRF)", backbone.len());
        let sidechain_count: usize = chain.iter()
            .map(|residue| fold3d::sidechain_topology(residue).len()).sum();
        println!("sidechain {} heavy atoms (complete residue inventory)", sidechain_count);
        let coordinates: Vec<_> = backbone.iter().map(|bb| [bb.n, bb.ca, bb.c, bb.o]).collect();
        let sidechains: Vec<Vec<(&str, fold3d::Vec3)>> = chain.iter().zip(&backbone).zip(&steps)
            .map(|((residue, bb), step)| fold3d::build_sidechain(residue, bb, step)).collect();
        if coords_mode {
            println!("== Per-residue coordinates (A) ==");
            println!("{:>4} {:<4} {:<5} {:>9} {:>9} {:>9}", "i", "AA", "atom", "x", "y", "z");
            for (i, ((aa, bb), sc)) in chain.iter().zip(&backbone).zip(&sidechains).enumerate() {
                for (name, xyz) in [("N",bb.n),("CA",bb.ca),("C",bb.c),("O",bb.o)].iter().chain(sc.iter()) {
                    println!("{:>4} {:<4} {:<5} {:>9.3} {:>9.3} {:>9.3}", i+1, aa, name, xyz.0, xyz.1, xyz.2);
                }
            }
        }
        let retained_steps: Vec<_> = steps.iter().map(|s| (s.phi, s.psi)).collect();
        let geometry = crate::peptide_closure::assess(chain, &coordinates, &sidechains, &retained_steps);
        print!("{}", geometry.report(false));
        let b_h = steps.iter().filter(|s| s.ss == "helix" || s.ss == "helix_l").count();
        let b_s = steps.iter().filter(|s| s.ss == "sheet").count();
        println!("retained B4 backbone labels: helix {}  sheet {}  other {}", b_h, b_s, steps.len()-b_h-b_s);
        let sequence_labels: Vec<&str> = f.residues.iter().map(|r| match r.secondary {
            fold::SecondaryLabel::Helix => "helix", fold::SecondaryLabel::Sheet => "sheet", fold::SecondaryLabel::Coil => "other",
        }).collect();
        let retained_labels: Vec<&str> = steps.iter().map(|s| s.ss).collect();
        let secondary = crate::peptide_closure::secondary_evidence(&sequence_labels, &retained_labels);
        print!("{}", crate::peptide_closure::secondary_report(&secondary, false));
        let mut pdb_hash = String::from("null");
        if let Some(path) = pdb_path {
            let elements = fold3d::group_ss_elements(&steps);
            let winding = f.residues.iter().map(|r| r.winding_number).max().unwrap_or(0);
            let mut pdb = fold3d::write_pdb_with_steps(chain, &backbone, &steps, &elements, f.frobenius_ok, winding, "COMPILED THROUGH VOX", 'A');
            let evidence_remarks: String = crate::peptide_closure::secondary_report(&secondary, true).lines()
                .map(|line| format!("REMARK 900 {}\n", line)).collect();
            pdb = pdb.replacen("END\n", &(evidence_remarks + &carrier.pdb_remarks() + "END\n"), 1);
            std::fs::write(path, pdb.as_bytes()).map_err(|e|format!("cannot write {}: {}",path,e))?;
            let hash = std::process::Command::new("sha256sum").arg("--").arg(path)
                .output().map_err(|e|format!("cannot bind PDB hash: {}",e))?;
            if !hash.status.success() {return Err(String::from("PDB SHA-256 failed"));}
            let text = String::from_utf8(hash.stdout).map_err(|e|e.to_string())?;
            pdb_hash = crate::protein_glut::quote(text.split_whitespace().next().ok_or("missing PDB hash")?);
            println!("pdb      written to {}",path);
        }
        let default_carrier = pdb_path.map(|p| format!("{}.glut.json",p));
        if let Some(path) = carrier_path.or(default_carrier.as_deref()) {
            let mut returned:serde_json::Value=serde_json::from_str(&carrier.json(b4_path,&secondary,&geometry,&f.contacts))
                .map_err(|e|format!("cannot retain lattice interaction return: {}",e))?;
            returned["observations"].as_array_mut().ok_or("missing protein observations")?.push(
                serde_json::json!({"name":"lattice-interaction-work","values":interaction_return}));
            let json = returned.to_string();
            std::fs::write(path,format!("{{\"pdb_sha256\":{},\"protein_glut\":{}}}\n",pdb_hash,json))
                .map_err(|e|format!("cannot write carrier {}: {}",path,e))?;
            println!("carrier  written to {}",path);
        }
        Ok(())
    }

    if is_nucleic {
        let t = protein::translate_full(&compact, &dialect);
        if t.protein.is_empty() {
            eprintln!("no protein translated from '{}'; needs an ATG/AUG start codon", seq);
            return 1;
        }
        let bytes: Vec<char> = t.mrna.chars().collect();
        let b4_path: Vec<char> = (0..t.protein.len()).map(|k| {
            let p = t.start + k * 3;
            bytes.get(p).and_then(|&c| genetic::nuc_b4(c)).unwrap_or('N')
        }).collect();

        println!("== vox compile: RNA/DNA -> protein ({}) ==", dialect_name);
        println!("input    {}", seq);
        println!("mrna     {}", t.mrna);
        println!("protein  {}", t.protein.join("-"));
        match t.stopped {
            Some(s) => println!("stop     {}", s),
            None => println!("stop     (none: the sequence ran out before a stop)"),
        }
        match report_fold(&t.protein, &b4_path, &t.mrna, t.start, pdb_path.as_deref(), carrier_path.as_deref(), coords_mode, native_lattice, observation_steps,
            route_glut_path.as_deref().zip(route_pdb_path.as_deref())) {
            Ok(())=>0, Err(e)=>{eprintln!("{}",e);1}
        }
    } else {
        let chain = match protein::parse_chain(&seq) {
            Some(c) if !c.is_empty() => c,
            _ => {
                eprintln!("could not parse '{}' as protein or nucleic acid", seq);
                eprintln!("use 3-letter (Met-Ala) or 1-letter (MA) amino acid codes, or A/C/G/T/U");
                return 1;
            }
        };
        let reverse = match protein::reverse_translate_full(&chain, &dialect) {
            Some(r) => r,
            None => { eprintln!("no codon exists for some residue under the {} table", dialect_name); return 1; }
        };
        let dna = protein::reverse_transcribe(&reverse.mrna);
        let bytes: Vec<char> = reverse.mrna.chars().collect();
        let b4_path: Vec<char> = (0..chain.len()).map(|k| {
            genetic::nuc_b4(bytes[k * 3]).unwrap_or('N')
        }).collect();

        println!("== vox compile: protein -> RNA/DNA ({}) ==", dialect_name);
        println!("input    {}", chain.join("-"));
        println!("mrna     {}  (Frobenius-preferred codon per residue)", reverse.mrna);
        println!("dna      {}", dna);
        println!("degeneracy  {} total possible mRNA sequences (product of per-residue codon counts)",
            reverse.total_combinations);
        match report_fold(&chain, &b4_path, &reverse.mrna, 0, pdb_path.as_deref(), carrier_path.as_deref(), coords_mode, native_lattice, observation_steps,
            route_glut_path.as_deref().zip(route_pdb_path.as_deref())) {
            Ok(())=>0, Err(e)=>{eprintln!("{}",e);1}
        }
    }
}

