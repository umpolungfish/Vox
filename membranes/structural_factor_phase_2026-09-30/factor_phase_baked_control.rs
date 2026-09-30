//! Baked basis controls for the structural oracle, not a factor search.
//! Expected marked, unmarked and trivial branches are supplied test controls.
#![deny(warnings)]
use vox::fixed_point_quantum_membrane::FixedPointQuantumMembrane;
use vox::fixed_point_quantum_phase::factor_oracle::{Control, GateSink};
use vox::morphism_factor::{emit_numeral, parse_numeral};
use vox::vox::{EVALF, EVALT};
struct Basis {
    cells: Vec<char>,
    phase: char,
}
impl GateSink for Basis {
    fn toggle(&mut self, controls: &[Control], target: usize) -> Result<(), &'static str> {
        if controls.iter().any(|c| c.cell == target) { return Err("gate aliases its target"); }
        if controls.iter().all(|c| self.cells[c.cell] == c.value) {
            self.cells[target] = if self.cells[target] == EVALT { EVALF } else { EVALT };
        }
        Ok(())
    }
    fn phase_flip(&mut self, controls: &[Control]) -> Result<(), &'static str> {
        if controls.iter().all(|c| self.cells[c.cell] == c.value) {
            self.phase = if self.phase == EVALT { EVALF } else { EVALT };
        }
        Ok(())
    }
}
fn main() {
    // All represented values are IMASM numerals. These are basis controls.
    let n = parse_numeral("⊢⊥⊥⊥⊥⊣").unwrap();
    let program = FixedPointQuantumMembrane::from_n(&n).unwrap()
        .prepare_structural_execution().unwrap();
    let oracle = program.factor_phase_oracle().unwrap();
    let layout = oracle.layout();
    let mut output = emit_numeral(&n);
    output.push('\n');
    for (pword,qword,expected) in [
        ("⊢⊥⊥⊣", "⊢⊥⊤⊥⊣", EVALF),
        ("⊢⊥⊥⊣", "⊢⊤⊤⊥⊣", EVALT),
        ("⊢⊥⊣", "⊢⊥⊥⊥⊥⊣", EVALT),
    ] {
        let p = parse_numeral(pword).unwrap();
        let q = parse_numeral(qword).unwrap();
        let mut basis = Basis {cells:vec![EVALT;layout.cells],phase:EVALT};
        for (range,value) in [(layout.p.clone(),p),(layout.q.clone(),q)] {
            for (i,cell) in range.enumerate() { basis.cells[cell] = value.get(i).copied().unwrap_or(EVALT); }
        }
        let before = basis.cells.clone();
        oracle.lower(&mut basis).unwrap();
        assert_eq!(basis.cells,before,"oracle failed to restore workspace");
        assert_eq!(basis.phase,expected,"factor-specific phase mismatch");
        output.push_str(&emit_numeral(&[basis.phase]));
        output.push('\n');
        oracle.lower(&mut basis).unwrap();
        assert_eq!(basis.cells,before,"oracle involution changed workspace");
        assert_eq!(basis.phase,EVALT,"oracle involution did not restore phase");
    }
    use std::io::{self,Write};
    io::stdout().lock().write_all(output.as_bytes()).unwrap();
}
