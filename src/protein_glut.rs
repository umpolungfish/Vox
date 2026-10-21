//! Prepared residue/form words and their retained native executions.
use alloc::{string::{String, ToString}, vec::Vec};
use crate::{fold, fold3d, peptide_closure, phase_word};

struct MarkSpec {
    aa: &'static str, record: &'static str, word: &'static str,
    tuple: &'static str, artifact: &'static str, sha256: &'static str,
}
include!(concat!(env!("OUT_DIR"), "/protein_marks.rs"));

pub struct Form {
    spec: &'static MarkSpec,
    pub execution: phase_word::Readout<&'static str>,
    pub transitions: [u8; 16],
}
pub struct Residue {
    pub index: usize, pub aa: &'static str, pub codon: String, pub forms: Vec<Form>,
}
pub struct ProteinGlut { pub residues: Vec<Residue> }

pub fn quote(value: &str) -> String {
    let mut out = String::from("\"");
    for c in value.chars() { match c {
        '"' => out.push_str("\\\""), '\\' => out.push_str("\\\\"),
        '\n' => out.push_str("\\n"), '\r' => out.push_str("\\r"), '\t' => out.push_str("\\t"),
        c if c < ' ' => out.push_str(&format!("\\u{:04x}", c as u32)), c => out.push(c),
    }}
    out.push('"'); out
}

impl ProteinGlut {
    pub fn words(&self) -> Vec<Vec<&str>> {
        self.residues.iter().map(|r|r.forms.iter().map(|f|f.spec.word).collect()).collect()
    }
    pub fn prepare(chain: &[&'static str], mrna: &str, start: usize) -> Result<Self, String> {
        let mut residues = Vec::with_capacity(chain.len());
        for (index, &aa) in chain.iter().enumerate() {
            let mut forms = Vec::new();
            for spec in MARKS.iter().filter(|m| m.aa == aa) {
                forms.push(Form { spec, execution: phase_word::execute(spec.word, &spec.tuple)?,
                    transitions: phase_word::native_transition_table(spec.word)? });
            }
            if forms.is_empty() { return Err(format!("no self-mark for residue {} {}", index, aa)); }
            let codon = mrna.get(start+3*index..start+3*index+3)
                .ok_or_else(|| format!("missing codon for residue {}", index))?.into();
            residues.push(Residue { index, aa, codon, forms });
        }
        Ok(Self { residues })
    }

    pub fn pdb_remarks(&self) -> String {
        let mut out = String::from("REMARK 901 NATIVE PROTEIN STATE B; COORDINATES ARE A BOUNDARY READOUT\n");
        for residue in &self.residues { for form in &residue.forms {
            out.push_str(&format!("REMARK 901 RESIDUE {} {} SELF-MARK {}\nREMARK 901 SOURCE SHA256 {}\n",
                residue.index+1, residue.aa, form.spec.artifact, form.spec.sha256));
        }}
        out
    }

    pub fn json(&self, b4: &[char], secondary: &[peptide_closure::SecondaryEvidence<'_>],
                geometry: &peptide_closure::Evidence, contacts: &[fold::TertiaryContact]) -> String {
        let mut alternatives = Vec::new();
        for from in ['N','T','F','B'] { for to in ['N','T','F','B'] {
            let r = fold3d::ramachandran(from,to);
            alternatives.push(format!("{{\"transition\":[{},{}],\"phi\":{},\"psi\":{},\"structure\":{}}}",
                quote(&from.to_string()), quote(&to.to_string()), r.phi, r.psi, quote(r.ss)));
        }}
        let mut rows = Vec::new();
        for residue in &self.residues {
            let mut forms = Vec::new();
            for form in &residue.forms {
                let f = &form.execution;
                let surviving = f.surviving.iter().map(|d| format!("{{\"id\":{},\"lane\":{},\"tuple\":{}}}",
                    d.id,d.lane,quote(d.observation))).collect::<Vec<_>>().join(",");
                forms.push(format!("{},\"execution\":{{\"register\":{},\"surviving\":[{}],\"cleared\":{},\"restored\":{},\"exposed\":{},\"deposits\":{},\"inert\":{},\"seeded\":{},\"restore_supports\":{:?},\"native_transition_table\":{:?},\"source_binding\":{}}}}}",
                    form.spec.record.strip_suffix('}').unwrap(), f.register,surviving,f.cleared,f.restored,
                    f.exposed,f.deposits,f.inert,f.seeded,f.restore_supports,form.transitions,SOURCE_BINDING));
            }
            let i = residue.index;
            rows.push(format!("{{\"index\":{},\"aa\":{},\"codon\":{},\"native_state\":\"B\",\"self_marks\":[{}],\"torsion_articulations\":[{}],\"coordinate_readout_transition\":[{},{}]}}",
                i,quote(residue.aa),quote(&residue.codon),forms.join(","),alternatives.join(","),
                quote(&if i==0 {'N'} else {b4[i-1]}.to_string()), quote(&b4[i].to_string())));
        }
        let observations = secondary.iter().map(|s| format!("{{\"residue\":{},\"sequence\":{},\"backbone\":{},\"helix\":{},\"sheet\":{},\"other\":{}}}",
            s.residue,quote(s.sequence.unwrap_or("")),quote(s.backbone.unwrap_or("")),
            quote(&s.helix.mark().to_string()),quote(&s.sheet.mark().to_string()),quote(&s.other.mark().to_string()))).collect::<Vec<_>>().join(",");
        let edges = contacts.iter().map(|c| {
            let mut forms=Vec::new();
            for a in &self.residues[c.i].forms {for b in &self.residues[c.j].forms {
                forms.push(format!("{{\"left\":{},\"right\":{}}}",quote(a.spec.artifact),quote(b.spec.artifact)));
            }}
            format!("{{\"left\":{},\"right\":{},\"kind\":{},\"forms\":[{}]}}",c.i,c.j,quote(&format!("{:?}",c.kind)),forms.join(","))
        }).collect::<Vec<_>>().join(",");
        let issues = geometry.issues.iter().map(|i|format!("{{\"kind\":{},\"detail\":{}}}",quote(i.kind),quote(&i.detail))).collect::<Vec<_>>().join(",");
        let readout = b4.iter().enumerate().map(|(i,c)|format!("[{},{}]",
            quote(&if i==0 {'N'}else{b4[i-1]}.to_string()),quote(&c.to_string()))).collect::<Vec<_>>().join(",");
        format!("{{\"native_state\":\"B\",\"semantics\":\"native-glut\",\"weights\":\"none\",\"coordinate_role\":\"boundary-readout\",\"residues\":[{}],\"observations\":[{{\"name\":\"secondary-structure\",\"values\":[{}]}},{{\"name\":\"contact-relations\",\"values\":[{}]}},{{\"name\":\"coordinate-geometry\",\"values\":{{\"constraints_satisfied\":{},\"atoms\":{},\"bonds\":{},\"angles\":{},\"rings\":{},\"stereocenters\":{},\"backbone_torsions\":{},\"nonbonded_pairs\":{},\"max_bond_error\":{},\"max_overlap\":{},\"issues\":[{}],\"report\":{}}}}}],\"readouts\":[{{\"name\":\"codon-aligned-torsions\",\"values\":[{}]}}],\"lean_source_sha256\":{}}}",
            rows.join(","),observations,edges,geometry.closed(),geometry.atoms,geometry.bonds,geometry.angles,
            geometry.rings,geometry.stereocenters,geometry.torsions,geometry.nonbonded_pairs,geometry.max_bond_error,
            geometry.max_overlap,issues,quote(&geometry.report(true)),readout,quote(crate::genetic_table::LEAN_SOURCE_SHA256))
    }
}
