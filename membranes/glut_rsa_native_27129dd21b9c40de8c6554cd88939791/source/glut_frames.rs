//! Resident evaluation-frame methods drawn from dialectic_reentry,
//! factor_2adic and shor_braid. Each call advances one frame in each lane.
use super::{mf, ONE, ZERO};
use alloc::{collections::BTreeMap, vec, vec::Vec};
use core::cmp::Ordering;
type Tape = Vec<char>;

pub(super) struct Frames {
    n: Tape,
    boundary: Tape,
    multiplier: Tape,
    base: Tape,
    phase: Tape,
    halves: BTreeMap<Tape, Tape>,
    width: usize,
}

impl Frames {
    pub(super) fn new(n: &[char], root: &[char]) -> Self {
        let boundary = if mf::mul(root, root) == n {
            root.to_vec()
        } else {
            mf::add(root, &[ONE])
        };
        Self {
            n: n.to_vec(),
            boundary,
            multiplier: vec![ONE],
            base: vec![ZERO, ONE],
            phase: vec![ZERO, ONE],
            halves: BTreeMap::new(),
            width: n.len().div_ceil(2).max(1),
        }
    }

    fn close(&self, candidate: Tape) -> Option<(Tape, Tape)> {
        let p = mf::gcd(candidate, self.n.clone());
        if mf::cmp(&p, &[ONE]) != Ordering::Greater || p == self.n {
            return None;
        }
        let (q, remainder) = mf::divmod(&self.n, &p);
        assert!(mf::zero(&remainder));
        let radix = {
            let mut r = vec![ZERO; self.width];
            r.push(ONE);
            r
        };
        let fixed = crate::factor_2adic::meet_factor_nestings(&self.n, &p, &q, &radix)
            .expect("both factor frame nestings must return the same fixed point");
        assert!(crate::trace_algebra::witness_valid(
            &self.n, &fixed.p, &fixed.q
        ));
        Some((fixed.p, fixed.q))
    }

    fn exposure(&self, target: &[char], boundary: &[char]) -> Option<(Tape, Tape)> {
        let delta = mf::sub(&mf::mul(boundary, boundary), target);
        let b = mf::isqrt(&delta);
        if mf::mul(&b, &b) != delta {
            return None;
        }
        self.close(mf::sub(boundary, &b))
            .or_else(|| self.close(mf::add(boundary, &b)))
    }

    pub(super) fn advance(&mut self) -> Option<(Tape, Tape)> {
        // The dialectic source boundary, with no fixed frontier span.
        if let Some(pair) = self.exposure(&self.n, &self.boundary) {
            return Some(pair);
        }
        self.boundary = mf::add(&self.boundary, &[ONE]);
        // The multiplier frame grows as an IMASM tape, independently of the
        // source boundary and the support phase register.
        let target = mf::mul(&[ZERO, ZERO, ONE], &mf::mul(&self.multiplier, &self.n));
        let mut a = mf::isqrt(&target);
        if mf::mul(&a, &a) != target {
            a = mf::add(&a, &[ONE]);
        }
        if let Some(pair) = self.exposure(&target, &a) {
            return Some(pair);
        }
        self.multiplier = mf::add(&self.multiplier, &[ONE]);

        let half = self.phase.clone();
        self.phase = mf::mul_mod(&half, &half, &self.n);
        if let Some(previous_half) = self.halves.insert(self.phase.clone(), half.clone()) {
            let difference = if mf::cmp(&half, &previous_half) == Ordering::Less {
                mf::sub(&previous_half, &half)
            } else {
                mf::sub(&half, &previous_half)
            };
            if let Some(pair) = self
                .close(difference)
                .or_else(|| self.close(mf::add(&half, &previous_half)))
            {
                return Some(pair);
            }
            // A completed orbit with trivial half closures opens the next
            // base. Neither orbit length nor base count has a fixed ceiling.
            self.base = mf::add(&self.base, &[ONE]);
            if let Some(pair) = self.close(self.base.clone()) {
                return Some(pair);
            }
            self.phase = mf::modulo(&self.base, &self.n);
            self.halves.clear();
        }
        let residue = mf::eval_binary_support_frame(&self.n, &self.phase, &self.n, self.width);
        self.close(residue)
    }
}
