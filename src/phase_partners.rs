//! Sequential dyadic partner observations. No register enumeration or supplied
//! order. A collision proves a return exponent, which may be a multiple of order.
use crate::morphism_factor::{cmp, gcd, modulo, mul, one, sub};
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use core::cmp::Ordering;
use num_bigint::BigUint;
use num_traits::Zero;
type Tape = Vec<char>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReturnRelation {
    pub earlier: Tape,
    pub later: Tape,
    pub residue: Tape,
    pub return_exponent: Tape,
    /// Adjacent dyadic residues whose quotient is a^(return_exponent / 2).
    /// Absent when the return exponent is odd or has no positive half-step.
    pub half_residues: Option<(Tape, Tape)>,
}

/// The one coefficient object behind the codec, frame sweep, and modular
/// support reads. `bits_le[i]` is the coefficient of X^i, so evaluating this
/// polynomial at 2 reconstructs the baked integer exactly.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SupportPolynomial {
    bits_le: Vec<char>,
}

/// A factor target is a zero of the same support polynomial modulo a proper
/// divisor of its value. The phase register is retained as the witness point.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SupportClosure {
    pub phase_index: usize,
    pub phase_register: Tape,
    pub polynomial_residue: Tape,
    pub p: Tape,
    pub q: Tape,
}

/// Production returns one divisor and its source-bound modular evidence.
/// No quotient register is prepared or searched here.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DivisorCycle {
    pub base: Tape,
    pub residue: Tape,
    pub phase_index: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DivisorWitness {
    source: Tape,
    pub divisor: Tape,
    pub dividend: Tape,
    pub phase_index: usize,
    pub arm: &'static str,
    pub base: Tape,
    pub preceding_cycles: Vec<DivisorCycle>,
}

impl DivisorWitness {
    /// The quotient is derived only after production has returned a divisor.
    pub fn derive_quotient(&self, source: &[char]) -> Result<Tape, String> {
        if source != self.source || cmp(&self.divisor, &one()) != Ordering::Greater
            || cmp(&self.divisor, source) != Ordering::Less {
            return Err("divisor return is not proper or belongs to another source".into());
        }
        if gcd(self.dividend.clone(), source.to_vec()) != self.divisor {
            return Err("divisor return lost its modular evidence".into());
        }
        let (quotient, remainder) = crate::morphism_factor::divmod(source, &self.divisor);
        if !crate::morphism_factor::zero(&remainder)
            || mul(&self.divisor, &quotient) != source {
            return Err("derived quotient failed exact source closure".into());
        }
        Ok(quotient)
    }
}

impl SupportPolynomial {
    fn from_tape(n: &[char]) -> Self {
        Self {
            bits_le: n.to_vec(),
        }
    }

    /// Horner-evaluate the one support polynomial through a width-bit frame.
    /// Every width is an equivalent regrouping of the same coefficient stream.
    fn evaluate_frame(&self, x: &BigUint, modulus: &BigUint, width: usize) -> BigUint {
        debug_assert!((2..=8).contains(&width));
        let mut powers = Vec::with_capacity(width + 1);
        powers.push(BigUint::from(1u8));
        for index in 1..=width {
            powers.push((&powers[index - 1] * x) % modulus);
        }
        let frame_base = &powers[width];
        let mut value = BigUint::from(0u8);
        let group_count = self.bits_le.len().div_ceil(width);
        let mut group_index = group_count;
        while group_index > 0 {
            let end = group_index * width;
            let start = end.saturating_sub(width);
            let group = &self.bits_le[start..end.min(self.bits_le.len())];
            if group.iter().all(|mark| *mark == crate::vox::EVALT) {
                let run_end = group_index;
                while group_index > 0 {
                    let end = group_index * width;
                    let start = end.saturating_sub(width);
                    let previous = &self.bits_le[start..end.min(self.bits_le.len())];
                    if previous.iter().any(|mark| *mark == crate::vox::EVALF) {
                        break;
                    }
                    group_index -= 1;
                }
                let zero_run = BigUint::from(run_end - group_index);
                let scale = frame_base.modpow(&zero_run, modulus);
                value = (value * scale) % modulus;
                continue;
            }
            let mut symbol = BigUint::from(0u8);
            for (position, mark) in group.iter().enumerate() {
                if *mark == crate::vox::EVALF {
                    symbol += &powers[position];
                }
            }
            value = (value * frame_base + symbol) % modulus;
            group_index -= 1;
        }
        value
    }

    fn target(
        &self,
        x: &BigUint,
        n: &BigUint,
        width: usize,
    ) -> Option<(BigUint, BigUint, BigUint)> {
        let (residue, factor) = self.divisor_target(x, n, width)?;
        let cofactor = n / &factor;
        if &cofactor * &factor != *n {
            return None;
        }
        Some((residue, factor, cofactor))
    }

    fn divisor_target(&self, x: &BigUint, n: &BigUint, width: usize)
        -> Option<(BigUint, BigUint)> {
        let residue = self.evaluate_frame(x, n, width);
        let factor = biguint_gcd(residue.clone(), n.clone());
        (factor > BigUint::from(1u8) && factor < *n).then_some((residue, factor))
    }
}

#[allow(dead_code)] // Retained for explicit, non-hot-path relation replay.
fn power(a: &[char], exponent: &[char], n: &[char]) -> Tape {
    let mut result = one();
    let mut base = modulo(a, n);
    for (i, &bit) in exponent.iter().enumerate() {
        if bit == crate::vox::EVALF {
            result = modulo(&mul(&result, &base), n);
        }
        if i + 1 < exponent.len() {
            base = modulo(&mul(&base, &base), n);
        }
    }
    result
}

impl ReturnRelation {
    #[allow(dead_code)] // Independent verifier; observe() relies on its recurrence invariant.
    pub fn verify(&self, a: &[char], n: &[char]) -> bool {
        if cmp(n, &one()) != Ordering::Greater
            || gcd(a.to_vec(), n.to_vec()) != one()
            || cmp(&self.later, &self.earlier) != Ordering::Greater
        {
            return false;
        }
        self.return_exponent == sub(&self.later, &self.earlier)
            && power(a, &self.earlier, n) == self.residue
            && power(a, &self.later, n) == self.residue
            && power(a, &self.return_exponent, n) == one()
    }
}

pub struct Partners {
    n: BigUint,
    initial_residue: BigUint,
    residue: BigUint,
    support: SupportPolynomial,
    // Brent's orbit winding retains a fixed number of dynamic registers. The
    // prior BTreeMap copied a modulus-width residue at every phase observation.
    tortoise: BigUint,
    orbit_power: usize,
    orbit_length: usize,
    pub squarings: usize,
    divisor_base: BigUint,
    divisor_cycles: Vec<DivisorCycle>,

}

fn tape_to_biguint(tape: &[char]) -> BigUint {
    let mut packed = vec![0u8; (tape.len() + 7) / 8];
    for (index, &mark) in tape.iter().enumerate() {
        if mark == crate::vox::EVALF {
            packed[index / 8] |= 1u8 << (index % 8);
        }
    }
    BigUint::from_bytes_le(&packed)
}

fn biguint_gcd(mut left: BigUint, mut right: BigUint) -> BigUint {
    while !right.is_zero() {
        let remainder = &left % &right;
        left = right;
        right = remainder;
    }
    left
}

fn biguint_to_tape(value: &BigUint) -> Tape {
    if value.is_zero() {
        return vec![crate::vox::EVALT];
    }
    let packed = value.to_bytes_le();
    let mut tape = Vec::with_capacity(packed.len() * 8);
    for byte in packed {
        for bit in 0..8 {
            tape.push(if (byte >> bit) & 1 == 1 {
                crate::vox::EVALF
            } else {
                crate::vox::EVALT
            });
        }
    }
    while tape.last() == Some(&crate::vox::EVALT) {
        tape.pop();
    }
    tape
}

fn power_of_two_exponent(index: usize) -> Tape {
    let mut exponent = vec![crate::vox::EVALT; index + 1];
    exponent[index] = crate::vox::EVALF;
    exponent
}

impl Partners {
    /// A nonunit base is already a useful one-divisor observation.
    pub fn new_divisor(a: Tape, n: Tape) -> Result<Self, String> {
        Self::new_inner(a, n, false)
    }
    pub fn new(a: Tape, n: Tape) -> Result<Self, String> {
        Self::new_inner(a, n, true)
    }

    /// Runtime entry for a phase base whose unit property was checked while
    /// its IMASM numeral was baked into the contained binary.
    pub fn new_from_validated_bake(a: Tape, n: Tape) -> Result<Self, String> {
        Self::new_inner(a, n, false)
    }

    fn new_inner(a: Tape, n: Tape, check_unit: bool) -> Result<Self, String> {
        if cmp(&n, &one()) != Ordering::Greater {
            return Err("partners require N > 1 and a coprime base".into());
        }
        let n_big = tape_to_biguint(&n);
        let a_big = tape_to_biguint(&a);
        if check_unit && biguint_gcd(a_big.clone(), n_big.clone()) != BigUint::from(1u8) {
            return Err("partners require N > 1 and a coprime base".into());
        }
        let residue = &a_big % &n_big;
        let initial_residue = residue.clone();
        let support = SupportPolynomial::from_tape(&n);
        Ok(Self {
            n: n_big,
            initial_residue: initial_residue.clone(),
            residue,
            support,
            tortoise: initial_residue,
            orbit_power: 1,
            orbit_length: 0,
            squarings: 0,
            divisor_base: a_big,
            divisor_cycles: Vec::new(),
        })
    }

    /// Test the current phase register against the shared support object.
    /// A proper gcd is an exact factor witness; the semiprime membrane still
    /// requires both nested arithmetic closures before emitting it.
    pub fn support_target(&self) -> Option<SupportClosure> {
        // Probe the complete initial frame sweep, then one register per doubled
        // phase scale. The full-return lane remains active between probes.
        let index = self.squarings;
        if index == 0 || (index > 8 && !index.is_power_of_two()) {
            return None;
        }
        let (polynomial_residue, p, q) = self.support.target(&self.residue, &self.n, 8)?;
        // Widths 2..7 are the same polynomial re-chunked, not independent
        // predicates. `all_frame_widths_are_faces_of_one_support_polynomial`
        // checks the identity; repeating six full wide evaluations here only
        // taxes the closure path without adding a new constraint.
        Some(SupportClosure {
            phase_index: self.squarings,
            phase_register: biguint_to_tape(&self.residue),
            polynomial_residue: biguint_to_tape(&polynomial_residue),
            p: biguint_to_tape(&p),
            q: biguint_to_tape(&q),
        })
    }

    /// Whether the current resident register is one of the support apertures
    /// that is actually read by `support_target`.
    pub fn support_probe(&self) -> bool {
        let index = self.squarings;
        index != 0 && (index <= 8 || index.is_power_of_two())
    }

    /// Preserve a negative support read as dynamic tape data. The phase index
    /// and resident register are the certificate; no host-width residue is
    /// retained.
    pub fn negative_support_snapshot(&self) -> (Tape, Tape) {
        (
            biguint_to_tape(&BigUint::from(self.squarings)),
            biguint_to_tape(&self.residue),
        )
    }

    /// One observation and one squaring. Caller chooses when to observe again;
    /// no fixed search ceiling is imposed on this resident state.
    pub fn observe(&mut self) -> Result<Option<ReturnRelation>, String> {
        let relation = if self.squarings > 0 && self.residue == self.tortoise {
            Some(self.resolve_cycle(self.orbit_length))
        } else {
            None
        };
        self.advance_register();
        Ok(relation)
    }

    fn advance_register(&mut self) {
        if self.orbit_length >= self.orbit_power {
            self.tortoise = self.residue.clone();
            self.orbit_power = self.orbit_power.saturating_mul(2);
            self.orbit_length = 0;
        }
        self.residue = (&self.residue * &self.residue) % &self.n;
        self.squarings += 1;
        self.orbit_length += 1;
    }

    fn divisor_witness(&self, dividend: BigUint, arm: &'static str) -> Option<DivisorWitness> {
        let divisor = biguint_gcd(dividend.clone(), self.n.clone());
        if divisor <= BigUint::from(1u8) || divisor >= self.n { return None; }
        Some(DivisorWitness { source: biguint_to_tape(&self.n),
            divisor: biguint_to_tape(&divisor), dividend: biguint_to_tape(&dividend),
            phase_index: self.squarings, arm, base: biguint_to_tape(&self.divisor_base),
            preceding_cycles: self.divisor_cycles.clone() })
    }

    /// One retained step, terminating on a proper divisor of N. Partial
    /// collisions modulo either factor are consumed before any full-N return.
    /// No exponent recovery or independent quotient producer participates.
    pub fn advance_divisor(&mut self) -> Option<DivisorWitness> {
        if self.squarings == 0 {
            if let Some(hit) = self.divisor_witness(self.residue.clone(), "base-nonunit") { return Some(hit); }
        }
        let unit = BigUint::from(1u8);
        // Fuse the three modular predicates before one gcd read. When the
        // joint read contains both factors, recover the separate predicates.
        let minus = if self.residue >= unit { &self.residue - &unit }
            else { &self.n - &unit };
        let plus = &self.residue + &unit;
        let difference = if self.residue >= self.tortoise { &self.residue - &self.tortoise }
            else { &self.tortoise - &self.residue };
        let predicates = [(minus, "phase-minus-unit"), (plus, "phase-plus-unit"),
            (difference, "partial-phase-collision")];
        let mut product = unit.clone();
        for (value, _) in &predicates {
            // A zero predicate gives gcd N and obscures the other arms.
            if !value.is_zero() { product = (product * value) % &self.n; }
        }
        let joint = biguint_gcd(product.clone(), self.n.clone());
        if joint > unit {
            for (value, arm) in &predicates {
                if let Some(hit) = self.divisor_witness(value.clone(), arm) { return Some(hit); }
            }
        }
        if self.support_probe() {
            if let Some((residue, divisor)) = self.support.divisor_target(&self.residue, &self.n, 8) {
                return Some(DivisorWitness { source: biguint_to_tape(&self.n),
                    divisor: biguint_to_tape(&divisor), dividend: biguint_to_tape(&residue),
                    phase_index: self.squarings, arm: "support-polynomial",
                    base: biguint_to_tape(&self.divisor_base), preceding_cycles: self.divisor_cycles.clone() });
            }
        }
        if self.squarings > 0 && self.residue == self.tortoise {
            // A whole-source cycle with no divisor is an open return, not
            // completion. Keep its evidence and re-enter on the next base
            // in this same resident object and the same caller's Y state.
            self.divisor_cycles.push(DivisorCycle { base: biguint_to_tape(&self.divisor_base),
                residue: biguint_to_tape(&self.residue), phase_index: self.squarings });
            self.divisor_base += &unit;
            self.residue = &self.divisor_base % &self.n;
            self.initial_residue = self.residue.clone();
            self.tortoise = self.residue.clone();
            self.orbit_power = 1;
            self.orbit_length = 0;
            self.squarings = 0;
            return None;
        }
        self.advance_register();
        None
    }

    fn resolve_cycle(&self, period: usize) -> ReturnRelation {
        let mut slow = self.initial_residue.clone();
        let mut fast = self.initial_residue.clone();
        let mut fast_previous = None;
        for _ in 0..period {
            fast_previous = Some(fast.clone());
            fast = (&fast * &fast) % &self.n;
        }
        let mut slow_previous = None;
        let mut entry = 0usize;
        while slow != fast {
            slow_previous = Some(slow.clone());
            slow = (&slow * &slow) % &self.n;
            fast_previous = Some(fast.clone());
            fast = (&fast * &fast) % &self.n;
            entry += 1;
        }
        let earlier_index = entry;
        let later_index = entry + period;
        let earlier = power_of_two_exponent(earlier_index);
        let later = power_of_two_exponent(later_index);
        let half_residues = if entry == 0 {
            None
        } else {
            slow_previous
                .zip(fast_previous)
                .map(|(earlier_half, current_half)| {
                    (
                        biguint_to_tape(&current_half),
                        biguint_to_tape(&earlier_half),
                    )
                })
        };
        ReturnRelation {
            earlier: earlier.clone(),
            later: later.clone(),
            residue: biguint_to_tape(&slow),
            return_exponent: sub(&later, &earlier),
            half_residues,
        }
    }

    pub fn stored_residues(&self) -> usize {
        3 + 2 * self.divisor_cycles.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::morphism_factor::tape_u64 as t;

    fn single_divisor_fixture() -> (Tape, Tape) {
        // An operation-level collision fixture on a hard balanced 208-bit
        // source. The expected divisor is not a production constructor input.
        (crate::morphism_factor::decimal_to_tape("266175155279792434973506259190185380993589458872977057421959357").unwrap(),
         crate::morphism_factor::decimal_to_tape("13419294329574030763782528217097").unwrap())
    }

    #[test]
    fn single_divisor_partial_collision_returns_before_full_phase_collision() {
        let (n, p) = single_divisor_fixture();
        let mut state = Partners::new_divisor(t(2), n.clone()).unwrap();
        state.residue = tape_to_biguint(&crate::morphism_factor::add(&p, &t(2)));
        state.squarings = 1;
        assert_ne!(state.residue, state.tortoise);
        let hit = state.advance_divisor().expect("proper partial collision must return immediately");
        assert_eq!(hit.arm, "partial-phase-collision");
        assert_eq!(hit.divisor, p);
        assert_eq!(state.squarings, 1, "a divisor return must not advance past its witness");
        let q = hit.derive_quotient(&n).unwrap();
        assert_eq!(mul(&hit.divisor, &q), n);
        assert!(hit.derive_quotient(&crate::morphism_factor::add(&n, &t(2))).is_err());
    }

    #[test]
    fn single_divisor_rejects_trivial_gcd_and_retains_y_continuation() {
        let (n, _) = single_divisor_fixture();
        let mut state = Partners::new_divisor(t(2), n.clone()).unwrap();
        assert!(state.divisor_witness(BigUint::from(0u8), "test").is_none());
        assert!(state.divisor_witness(BigUint::from(1u8), "test").is_none());
        assert!(state.divisor_witness(tape_to_biguint(&n), "test").is_none());
        let mut driver = crate::resident_y::ResidentY::from_certified_program(
            crate::resident_y::ResidentY::certified_program()).unwrap();
        for _ in 0..8 {
            assert!(!driver.advance(|| Ok(state.advance_divisor().is_some())).unwrap());
        }
        assert_eq!(driver.calls, 8);
        assert_eq!(state.squarings, 8);
        assert_eq!(state.negative_support_snapshot().1,
            power(&t(2), &power_of_two_exponent(8), &n));
    }

    #[test]
    fn single_divisor_full_cycle_keeps_evidence_and_reenters_same_source() {
        let (n, _) = single_divisor_fixture();
        let mut state = Partners::new_divisor(t(1), n.clone()).unwrap();
        assert!(state.advance_divisor().is_none());
        assert!(state.advance_divisor().is_none());
        assert_eq!(state.n, tape_to_biguint(&n));
        assert_eq!(state.divisor_base, BigUint::from(2u8));
        assert_eq!(state.divisor_cycles.len(), 1);
        assert_eq!(state.divisor_cycles[0].base, t(1));
        assert_eq!(state.divisor_cycles[0].residue, t(1));
        assert_eq!(state.squarings, 0);
    }

    #[test]
    fn dynamic_integer_phase_register_round_trips_tapes() {
        for word in ["0", "1", "255", "256", "123456789012345678901234567890"] {
            let tape = crate::morphism_factor::decimal_to_tape(word).unwrap();
            assert_eq!(biguint_to_tape(&tape_to_biguint(&tape)), tape);
        }
    }

    #[test]
    fn all_frame_widths_are_faces_of_one_support_polynomial() {
        for (n, x) in [(15u64, 4u64), (21, 4), (35, 16), (8051, 37)] {
            let n_big = BigUint::from(n);
            let x_big = BigUint::from(x);
            let support = SupportPolynomial::from_tape(&t(n));
            let direct =
                support
                    .bits_le
                    .iter()
                    .enumerate()
                    .fold(BigUint::from(0u8), |sum, (index, bit)| {
                        if *bit == crate::vox::EVALF {
                            (sum + x_big.modpow(&BigUint::from(index), &n_big)) % &n_big
                        } else {
                            sum
                        }
                    });
            for width in 2..=8 {
                assert_eq!(
                    support.evaluate_frame(&x_big, &n_big, width),
                    direct,
                    "N={n}, x={x}, frame width={width}"
                );
            }
        }
    }

    #[test]
    fn support_polynomial_zero_targets_a_proper_factor_without_a_full_collision() {
        let mut partners = Partners::new(t(2), t(15)).unwrap();
        assert!(partners.support_target().is_none()); // F_15(2)=15, trivial gcd
        partners.observe().unwrap();
        let target = partners.support_target().expect("F_15(4) shares factor 5");
        assert_eq!(target.phase_index, 1);
        assert_eq!(crate::morphism_factor::dec_of(&target.p), "5");
        assert_eq!(crate::morphism_factor::dec_of(&target.q), "3");
        assert_eq!(crate::morphism_factor::mul(&target.p, &target.q), t(15));
    }

    #[test]
    fn relations_are_replayed_and_mutations_rejected() {
        for (n, expected) in [(15, 4), (21, 6), (35, 12)] {
            let mut p = Partners::new(t(2), t(n)).unwrap();
            let relation = loop {
                if let Some(r) = p.observe().unwrap() {
                    break r;
                }
            };
            assert_eq!(relation.return_exponent, t(expected));
            assert!(relation.verify(&t(2), &t(n)));
            let mut bad = relation.clone();
            bad.residue = t(0);
            assert!(!bad.verify(&t(2), &t(n)));
            let mut bad = relation;
            bad.return_exponent = t(1);
            assert!(!bad.verify(&t(2), &t(n)));
        }
        assert!(Partners::new(t(3), t(15)).is_err());
    }

    #[test]
    fn collision_carries_the_two_half_step_residues() {
        let mut partners = Partners::new(t(2), t(15)).unwrap();
        let relation = loop {
            if let Some(relation) = partners.observe().unwrap() {
                break relation;
            }
        };
        assert_eq!(relation.return_exponent, t(4));
        let (current_half, earlier_half) = relation.half_residues.unwrap();
        assert_eq!(crate::morphism_factor::dec_of(&current_half), "1");
        assert_eq!(crate::morphism_factor::dec_of(&earlier_half), "4");
        let (p, q) =
            crate::shor_braid::phase_factor_register_seeds(&t(15), &current_half, &earlier_half)
                .unwrap();
        assert_eq!(crate::morphism_factor::dec_of(&p), "3");
        assert_eq!(crate::morphism_factor::dec_of(&q), "5");
    }

    #[test]
    fn wide_partner_brent_winding_keeps_constant_registers_and_both_closures() {
        let shift = BigUint::from(1u8) << 3300usize;
        let expected_p = BigUint::from(7161u16) * &shift + BigUint::from(1u8);
        let expected_q = BigUint::from(9135u16) * &shift + BigUint::from(1u8);
        let n = biguint_to_tape(&(&expected_p * &expected_q));
        let base = t(2);
        let mut partners = Partners::new(base.clone(), n.clone()).unwrap();
        let relation = (0..5000).find_map(|_| partners.observe().unwrap());
        let relation = relation.expect("wide phase collision closes within the measured orbit");
        assert!(partners.squarings < 5000);
        assert_eq!(partners.stored_residues(), 3);
        assert!(relation.verify(&base, &n));
        let (p, q) = crate::shor_braid::phase_factor_register_seeds(
            &n,
            &relation.half_residues.as_ref().unwrap().0,
            &relation.half_residues.as_ref().unwrap().1,
        )
        .unwrap();
        let (small, large) = if expected_p <= expected_q {
            (&expected_p, &expected_q)
        } else {
            (&expected_q, &expected_p)
        };
        assert_eq!(p, biguint_to_tape(small));
        assert_eq!(q, biguint_to_tape(large));
        assert_eq!(crate::morphism_factor::mul(&p, &q), n);
        let radix = crate::morphism_factor::tape_u64(4_294_967_296);
        let product_outer =
            crate::factor_2adic::nest_product_over_prefix(&n, &p, &q, &radix).unwrap();
        let prefix_outer =
            crate::factor_2adic::nest_prefix_over_product(&n, &p, &q, &radix).unwrap();
        assert_eq!(product_outer, prefix_outer);
        assert!(crate::factor_2adic::radix_prefix_closes(&n, &p, &q, &radix,));
    }
}
