//! Sequential dyadic partner observations. No register enumeration or supplied
//! order. A collision proves a return exponent, which may be a multiple of order.
use std::collections::BTreeMap;
use vox::morphism_factor::{add, cmp, gcd, modulo, mul, one, sub};
type Tape = Vec<char>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReturnRelation {
    pub earlier: Tape,
    pub later: Tape,
    pub residue: Tape,
    pub return_exponent: Tape,
}

fn power(a: &[char], exponent: &[char], n: &[char]) -> Tape {
    let mut result = one();
    let mut base = modulo(a, n);
    for &bit in exponent {
        if bit == vox::vox::EVALF { result = modulo(&mul(&result, &base), n); }
        base = modulo(&mul(&base, &base), n);
    }
    result
}

impl ReturnRelation {
    pub fn verify(&self, a: &[char], n: &[char]) -> bool {
        if cmp(n, &one()) != std::cmp::Ordering::Greater
            || gcd(a.to_vec(), n.to_vec()) != one()
            || cmp(&self.later, &self.earlier) != std::cmp::Ordering::Greater {
            return false;
        }
        self.return_exponent == sub(&self.later, &self.earlier)
            && power(a, &self.earlier, n) == self.residue
            && power(a, &self.later, n) == self.residue
            && power(a, &self.return_exponent, n) == one()
    }
}

pub struct Partners {
    a: Tape,
    n: Tape,
    exponent: Tape,
    residue: Tape,
    seen: BTreeMap<Tape, Tape>,
    pub squarings: usize,
}

impl Partners {
    pub fn new(a: Tape, n: Tape) -> Result<Self, String> {
        if cmp(&n, &one()) != std::cmp::Ordering::Greater || gcd(a.clone(), n.clone()) != one() {
            return Err("partners require N > 1 and a coprime base".into());
        }
        let mut seen = BTreeMap::new();
        seen.insert(one(), vox::morphism_factor::tape_u64(0));
        let residue = modulo(&a, &n);
        Ok(Self { a, n, exponent: one(), residue, seen, squarings: 0 })
    }

    /// One observation and one squaring. Caller chooses when to observe again;
    /// no fixed search ceiling is imposed on this resident state.
    pub fn observe(&mut self) -> Result<Option<ReturnRelation>, String> {
        let relation = self.seen.get(&self.residue).map(|earlier| ReturnRelation {
            earlier: earlier.clone(), later: self.exponent.clone(), residue: self.residue.clone(),
            return_exponent: sub(&self.exponent, earlier),
        });
        if let Some(ref r) = relation {
            if !r.verify(&self.a, &self.n) { return Err("partner relation failed replay".into()); }
        }
        self.seen.entry(self.residue.clone()).or_insert_with(|| self.exponent.clone());
        self.residue = modulo(&mul(&self.residue, &self.residue), &self.n);
        self.exponent = add(&self.exponent, &self.exponent);
        self.squarings += 1;
        Ok(relation)
    }

    pub fn stored_residues(&self) -> usize { self.seen.len() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vox::morphism_factor::tape_u64 as t;
    #[test]
    fn relations_are_replayed_and_mutations_rejected() {
        for (n, expected) in [(15,4), (21,6), (35,12)] {
            let mut p = Partners::new(t(2),t(n)).unwrap();
            let relation = loop { if let Some(r) = p.observe().unwrap() { break r; } };
            assert_eq!(relation.return_exponent, t(expected));
            assert!(relation.verify(&t(2), &t(n)));
            let mut bad = relation.clone();
            bad.residue = t(0);
            assert!(!bad.verify(&t(2), &t(n)));
            let mut bad = relation;
            bad.return_exponent = t(1);
            assert!(!bad.verify(&t(2), &t(n)));
        }
        assert!(Partners::new(t(3),t(15)).is_err());
    }
}
