//! Shared hosted protein compiler for standalone Vox and G-mOMonadOS.
//! Both entry points execute the same source-bound carrier and coordinate path.
use std::{string::{String, ToString}, vec::Vec, println, eprintln, print};
use crate::{genetic, protein, fold, fold3d};

/// `vox compile <seq> [--code standard|mitochondrial] [--pdb <path>]`
///
/// One entry point, both directions of the same pipeline. RNA/DNA in:
/// `protein::translate_full` (every residue, not just the twelve promoted
/// ones `genetic::lift_rna` keeps), then fold (`fold::fold_sequence` for
/// secondary/tertiary structure, `fold3d` for a real 3D backbone). Protein
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
    let mut seq_parts: Vec<String> = Vec::new();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--help" | "-h" => {
                println!("vox compile <seq> | --fasta <file> [--input auto|rna|dna|protein] [--code standard|mitochondrial] [--pdb <path>] [--carrier <path>] [--coords]");
                println!("Execute full residue/form self-marks and retain native protein glut. PDB export also writes <path>.glut.json bound to the PDB SHA-256. --carrier selects another carrier path. --input protein resolves nucleotide-alphabet ambiguity. --fasta reads one record.");
                return 0;
            }
            "--coords" => { coords_mode = true; i += 1; }
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
    let dialect_name = if dialect.is_empty() { "standard" } else { "mitochondrial" };

    let compact: String = seq.chars().filter(|c| !c.is_whitespace() && *c != '-' && *c != ',').collect();
    let nucleic_alphabet = !compact.is_empty()
        && compact.chars().all(|c| matches!(c.to_ascii_uppercase(), 'A' | 'C' | 'G' | 'T' | 'U'));
    if matches!(input_kind.as_str(), "rna"|"dna") && !nucleic_alphabet {
        eprintln!("nucleic input contains a non-nucleotide");return 1;
    }
    let is_nucleic = matches!(input_kind.as_str(),"rna"|"dna") || input_kind=="auto" && nucleic_alphabet;

    fn report_fold(chain: &[&'static str], b4_path: &[char], mrna: &str, start: usize,
                   pdb_path: Option<&str>, carrier_path: Option<&str>, coords_mode: bool) -> Result<(), String> {
        let carrier = crate::protein_glut::ProteinGlut::prepare(chain, mrna, start)?;
        println!("native protein B: {} residues, {} chemical forms; full self-marks executed",
            carrier.residues.len(), carrier.residues.iter().map(|r|r.forms.len()).sum::<usize>());
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
            let json = carrier.json(b4_path,&secondary,&geometry,&f.contacts);
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
        match report_fold(&t.protein, &b4_path, &t.mrna, t.start, pdb_path.as_deref(), carrier_path.as_deref(), coords_mode) {
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
        match report_fold(&chain, &b4_path, &reverse.mrna, 0, pdb_path.as_deref(), carrier_path.as_deref(), coords_mode) {
            Ok(())=>0, Err(e)=>{eprintln!("{}",e);1}
        }
    }
}

