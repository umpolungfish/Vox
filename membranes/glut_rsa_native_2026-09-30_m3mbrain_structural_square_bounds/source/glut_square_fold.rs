//! Structural square support for a²=N+b² on shared glyph-valued cells.
//! Width frames grow from the source root to the proper-factor midpoint
//! extent. A frame descends by syndrome constraints and bit assignments.
use super::{correlation, mf, trim, ONE};
use alloc::vec::Vec;
pub(super) enum Step {
    Running,
    Closed(Vec<char>, Vec<char>),
    Empty,
}
pub(super) struct SquareFold {
    n: Vec<char>,
    root: Vec<char>,
    upper: Vec<char>,
    width: usize,
    graph: Option<correlation::Correlation>,
    pub advances: usize,
}
impl SquareFold {
    pub fn new(n: &[char], root: &[char]) -> Self {
        // p>=3 implies a=(p+q)/2 < (N+1)/2. This is a source relation,
        // so the terminal width follows N rather than a configured quota.
        let upper = trim(mf::sub(n, &[ONE])[1..].to_vec());
        Self { n: n.to_vec(), root: root.to_vec(), upper,
            width: root.len(), graph: None, advances: 0 }
    }
    pub fn cells(&self) -> usize {
        self.graph.as_ref().map_or(self.n.len()+self.root.len()+self.upper.len(),
            |graph| graph.cell_count()+graph.gate_count())
    }
    pub fn advance(&mut self) -> Step {
        if self.width > self.upper.len() { return Step::Empty; }
        if self.graph.is_none() {
            self.graph = Some(correlation::Correlation::new_midpoint(
                &self.n, self.width, &self.root));
        }
        self.advances += 1;
        match self.graph.as_mut().unwrap().advance() {
            correlation::Step::Running => Step::Running,
            correlation::Step::Empty => {
                self.graph = None;
                self.width += 1;
                Step::Running
            }
            correlation::Step::Closed(p, q) => {
                assert!(crate::trace_algebra::witness_valid(&self.n, &p, &q));
                Step::Closed(p, q)
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn correlated_square_support_matches_small_odd_product_census() {
        for n in (1..=511u64).step_by(2) {
            let tape = mf::tape_u64(n);
            let mut fold = SquareFold::new(&tape, &mf::isqrt(&tape));
            let expected = (3..=n).any(|p| p * p <= n && n % p == 0);
            loop {
                match fold.advance() {
                    Step::Running => {}
                    Step::Empty => {
                        assert!(!expected, "n={n}");
                        break;
                    }
                    Step::Closed(p, q) => {
                        assert!(expected);
                        assert!(crate::trace_algebra::witness_valid(&tape, &p, &q));
                        break;
                    }
                }
            }
        }
    }
}
