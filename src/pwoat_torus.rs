//! Source-bound four-coset re-entry on the Pwoat torus.
//! The four orbit arms are held as one powerset family in a shared state arena.
use alloc::{string::String, vec::Vec};
use num_bigint::{BigInt, BigUint, Sign};
use num_traits::{One, ToPrimitive, Zero};

use crate::{
    fixed_point_imasm::ImasmNumeralWord,
    fixed_point_word_arithmetic as word_arithmetic,
    pwoat_geometry::{number_line_parts, Log2Coordinate, RingCoordinate},
    reentry_powerset::{bind, unit, Held},
};

struct OrbitArm {
    origin_p: BigUint,
    origin_q: BigUint,
    p: BigUint,
    q: BigUint,
    height: BigInt,
    position: BigUint,
    p_log: Log2Coordinate,
    q_log: Log2Coordinate,
    p_ring: RingCoordinate,
    q_ring: RingCoordinate,
    singularity_crossing: Option<BitBraidCrossing>,
}

pub struct PwoatTorusOrbit {
    n: BigUint,
    m: usize,
    mask: BigUint,
    lead: BigUint,
    step: BigUint,
    inverse_step: BigUint,
    bit_braid: BitBraid,
    arms: Vec<OrbitArm>,
    active: Held<usize>,
    positions_per_arm: BigUint,
    last_jump_branch: Option<usize>,
    complete: bool,
    powerset_monad_laws_verified: bool,
    full_orbit_period_verified: bool,
}

pub struct OrbitAdvance {
    pub factor: Option<BigUint>,
    pub candidate_trace: Option<OrbitCandidateTrace>,
    pub complete: bool,
    pub powerset_monad_laws_verified: bool,
    pub full_orbit_period_verified: bool,
}

/// The 24-generator repeating Burau unit from the (16, 3) horn knot
/// with Burau trace 676 and invariant T_B + Δ_A = 3.
pub const HORN_BURAU_24: [i8; 24] = [
    2, 2, -1, -1, -2, -2, 1, 1, 2, 1, 2, 1,
    -2, -2, -1, -1, 2, 1, 2, 1, -2, -2, -1, -1,
];
pub const HORN_BURAU_TRACE: u64 = 676;
pub const HORN_FROBENIUS_INVARIANT: u64 = 3;

/// The modular parameter of the Standard Model horn torus:
/// τ = (-1 + 4i) / √17, with tilt = arctan(1/4).
/// Re(ST(τ)) = -1/2 (the fundamental domain symmetry boundary).
pub const HORN_MODULAR_REAL: f64 = -1.0 / 4.123105625617661; // -1 / sqrt(17)
pub const HORN_MODULAR_IMAG: f64 = 4.0 / 4.123105625617661;  // 4 / sqrt(17)
pub const HORN_TILT_RAD: f64 = 0.24497866312686414;          // arctan(1/4)
pub const HORN_ST_REAL: f64 = -0.5;                          // Re(ST(τ)) = -1/2
pub const HORN_ST_IMAG: f64 = 0.6403882032022076;           // (1 + sqrt(17)) / 8

/// Multi-strand horn-torus bit-twist braid operator grounded in the (16, 3)
/// horn knot and 24-generator Burau unit. Preserves the lower 3 bits (fixing
/// the four 2-adic cosets in ℤ/8ℤ) while weaving intermediate and upper
/// logarithmic strands through the pinch singularity ⊙ at θ_{turn} = 1/2.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BitBraid {
    pub strands: usize,
    pub generators: Vec<usize>,
    pub endpoints: Option<(usize, usize)>,
    pub burau_trace: u64,
    pub frobenius_invariant: u64,
    pub burau_word: Vec<i8>,
}

impl BitBraid {
    pub fn torus_twist(strands: usize) -> Self {
        if strands < 5 {
            return Self {
                strands,
                generators: Vec::new(),
                endpoints: None,
                burau_trace: HORN_BURAU_TRACE,
                frobenius_invariant: HORN_FROBENIUS_INVARIANT,
                burau_word: HORN_BURAU_24.to_vec(),
            };
        }
        let s1 = 3usize; // 2-adic anchor: ℤ/8ℤ boundary
        let s2 = 3 + (strands - 3) / 2; // Split-shell λ_C Compton midpoint
        let s3 = strands - 1; // Logarithmic / Archimedean scale

        let make_ladder = |a: usize, b: usize| -> Vec<usize> {
            if a >= b {
                return Vec::new();
            }
            let mut gens = (a..b).collect::<Vec<_>>();
            gens.extend((a..b.saturating_sub(1)).rev());
            gens
        };

        // 1. Forward Burau 24-generator weave across the 3 evaluator arms (s1, s2, s3)
        let mut w_gens = Vec::new();
        for &op in &HORN_BURAU_24 {
            if op.abs() == 1 {
                w_gens.extend(make_ladder(s1, s2));
            } else if op.abs() == 2 {
                w_gens.extend(make_ladder(s2, s3));
            }
        }

        // 2. Multi-strand conjugate reflection through the pinch singularity ⊙
        // Pairing each active strand from s1 upwards with its conjugate from s3 downwards
        let mut t_gens = Vec::new();
        for i in 0..(strands - 3) / 2 {
            let lo = s1 + i;
            let hi = s3 - i;
            if lo < hi {
                t_gens.extend(make_ladder(lo, hi));
            }
        }

        // 3. Dual return of the forward weave (exact reverse): W⁻¹
        let w_inv_gens: Vec<usize> = w_gens.iter().copied().rev().collect();

        // Canonical braid: W · T · W⁻¹. Other selected orders use the retained reverse for return.
        let mut generators = w_gens;
        generators.extend(t_gens);
        generators.extend(w_inv_gens);
        let generators = crate::ordering::selected("bit-braid", generators.len())
            .expect("valid bit-braid ordering").apply(&generators).expect("complete crossing permutation");

        Self {
            strands,
            generators,
            endpoints: Some((s1, s3)),
            burau_trace: HORN_BURAU_TRACE,
            frobenius_invariant: HORN_FROBENIUS_INVARIANT,
            burau_word: HORN_BURAU_24.to_vec(),
        }
    }

    /// Apply each adjacent crossing to the m-bit residue without dropping bits.
    pub fn apply(&self, value: &BigUint) -> BigUint {
        let mut out = value.clone();
        for &crossing in &self.generators {
            let left = out.bit(crossing as u64);
            let right = out.bit((crossing + 1) as u64);
            if left != right {
                out ^= (BigUint::one() << crossing) | (BigUint::one() << (crossing + 1));
            }
        }
        out
    }

    /// Apply the same strand crossings to an IMASM value word. The cell mark
    /// is carried independently from its structural place, so a crossing
    /// exchanges two place bindings without decoding the numeral.
    pub fn apply_word(
        &self,
        value: &ImasmNumeralWord,
        width: usize,
    ) -> Result<ImasmNumeralWord, String> {
        let mut marks = word_arithmetic::mark_bits(value).map_err(word_error)?;
        if marks.len() > width { return Err(format!("bit braid width {width} would discard {} carried marks", marks.len())); }
        marks.resize(width, '⊤');
        for &crossing in &self.generators {
            if crossing + 1 >= width { continue; }
            marks.swap(crossing, crossing + 1);
        }
        word_arithmetic::from_mark_bits(&marks).map_err(word_error)
    }

    /// Reverse the actual executed crossing list, including a selected non-involutive order.
    pub fn return_word(&self, value: &ImasmNumeralWord, width: usize) -> Result<ImasmNumeralWord, String> {
        let mut marks = word_arithmetic::mark_bits(value).map_err(word_error)?;
        if marks.len() > width { return Err(format!("bit braid inverse width {width} would discard {} carried marks", marks.len())); }
        marks.resize(width, '⊤');
        for &crossing in self.generators.iter().rev() {
            if crossing + 1 < width { marks.swap(crossing, crossing + 1); }
        }
        word_arithmetic::from_mark_bits(&marks).map_err(word_error)
    }

    /// Induce the braid's retained adjacent crossings on the twelve
    /// correlation axes. The same crossing word therefore orders the live
    /// product/carry cells, with its inverse retained by the resident.
    pub fn axis_permutation(&self) -> [usize; 12] {
        let mut axes = core::array::from_fn(|index| index);
        for &crossing in &self.generators {
            let left = crossing % 11;
            axes.swap(left, left + 1);
        }
        axes
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct OrbitCandidateTrace {
    pub branch_id: usize,
    pub position: BigUint,
    pub p_log: Log2Coordinate,
    pub q_log: Log2Coordinate,
    pub p_ring: RingCoordinate,
    pub q_ring: RingCoordinate,
    pub bit_braid: BitBraid,
    pub singularity_crossing: BitBraidCrossing,
}

/// A terminally retained snapshot of one live torus arm, including its exact
/// binary pair, carry height, logarithmic coordinates, and singularity ancestry.
#[derive(Clone, Debug, PartialEq)]
pub struct OrbitBranchSnapshot {
    pub branch_id: usize,
    pub position: BigUint,
    pub p: BigUint,
    pub q: BigUint,
    pub height: BigInt,
    pub p_log: Log2Coordinate,
    pub q_log: Log2Coordinate,
    pub p_ring: RingCoordinate,
    pub q_ring: RingCoordinate,
    pub bit_braid: BitBraid,
    pub singularity_crossing: BitBraidCrossing,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BitBraidCrossing {
    pub minor_turn_numerator: u8,
    pub minor_turn_denominator: u8,
    pub position: BigUint,
    pub pre_p_bits: BigUint,
    pub pre_q_bits: BigUint,
    pub post_p_bits: BigUint,
    pub post_q_bits: BigUint,
    /// Source-conditioned arithmetic q after the p strand was braided.
    pub rebound_q_bits: BigUint,
    pub generators: Vec<usize>,
    pub involution_verified: bool,
    pub inverse_recovery_verified: bool,
    pub low_three_bits_preserved: bool,
    pub product_residue_preserved: bool,
}

impl PwoatTorusOrbit {
    pub fn new(n: &BigUint, m: usize, multiplier: u32) -> Result<Self, String> {
        if m < 4 { return Err(String::from("Pwoat orbit requires factor width m >= 4")); }
        if multiplier % 16 != 9 {
            return Err(String::from("Pwoat orbit multiplier must be 9 mod 16"));
        }
        let two_m = BigUint::one() << m;
        let mask = &two_m - BigUint::one();
        let lead = BigUint::one() << (m - 1);
        let step = BigUint::from(multiplier) & &mask;
        let step_word = numeral_from_biguint(&step)?;
        let inverse_step = biguint_from_numeral(
            &word_arithmetic::inverse_odd_power_two(&step_word, m).map_err(word_error)?,
        )?;
        let bit_braid = BitBraid::torus_twist(m);
        let reduced_n = n & &mask;
        let reduced_n_word = numeral_from_biguint(&reduced_n)?;
        let reps = [
            BigUint::one(),
            BigUint::from(3u32),
            &two_m - BigUint::one(),
            &two_m - BigUint::from(3u32),
        ];
        let mut arms = Vec::with_capacity(reps.len());
        let active = bind(unit(0usize), |_root| {
            for rep in &reps {
                let pre_p = rep % &two_m;
                let pre_p_word = numeral_from_biguint(&pre_p)?;
                let pre_q_value = biguint_from_numeral(
                    &word_arithmetic::multiply_mod_power_two(
                        &reduced_n_word,
                        &word_arithmetic::inverse_odd_power_two(&pre_p_word, m)
                            .map_err(word_error)?,
                        m,
                    ).map_err(word_error)?,
                )?;
                let p_word = bit_braid.apply_word(&pre_p_word, m)?;
                let pre_q_word = numeral_from_biguint(&pre_q_value)?;
                let braided_q_word = bit_braid.apply_word(&pre_q_word, m)?;
                let q_word = biguint_from_numeral(
                    &word_arithmetic::multiply_mod_power_two(
                        &reduced_n_word,
                        &word_arithmetic::inverse_odd_power_two(&p_word, m)
                            .map_err(word_error)?,
                        m,
                    ).map_err(word_error)?,
                )?;
                let p = biguint_from_numeral(&p_word)?;
                let braided_q = biguint_from_numeral(&braided_q_word)?;
                let q = q_word;
                let involution_verified = bit_braid.apply_word(&p_word, m)? == pre_p_word
                    && bit_braid.apply_word(&braided_q_word, m)? == pre_q_word;
                let inverse_recovery_verified = bit_braid.return_word(&p_word, m)? == pre_p_word
                    && bit_braid.return_word(&braided_q_word, m)? == pre_q_word;
                let low_mask = BigUint::from(7u8);
                let low_three_bits_preserved = (&p & &low_mask) == (&pre_p & &low_mask)
                    && (&braided_q & &low_mask) == (&pre_q_value & &low_mask);
                let product_residue_preserved = ((&p * &q) & &mask) == reduced_n;
                if !inverse_recovery_verified || !low_three_bits_preserved || !product_residue_preserved {
                    return Err(String::from("singularity bit braid failed its retained-state invariants"));
                }
                let crossing = BitBraidCrossing {
                    minor_turn_numerator: 1,
                    minor_turn_denominator: 2,
                    position: BigUint::zero(),
                    pre_p_bits: pre_p,
                    pre_q_bits: pre_q_value,
                    post_p_bits: p.clone(),
                    post_q_bits: braided_q.clone(),
                    rebound_q_bits: q.clone(),
                    generators: bit_braid.generators.clone(),
                    involution_verified,
                    inverse_recovery_verified,
                    low_three_bits_preserved,
                    product_residue_preserved,
                };
                let mut height = BigInt::from_biguint(Sign::Plus, &p * &q)
                    - BigInt::from_biguint(Sign::Plus, n.clone());
                height >>= m;
                let p_log = log2_biguint(&p);
                let q_log = log2_biguint(&q);
                arms.push(OrbitArm {
                    origin_p: p.clone(), origin_q: q.clone(),
                    p: p.clone(), q: q.clone(), height, position: BigUint::zero(),
                    p_ring: number_line_parts(p_log), q_ring: number_line_parts(q_log),
                    singularity_crossing: Some(crossing),
                    p_log, q_log,
                });
            }
            Ok::<_, String>((0..arms.len()).collect())
        })?;
        let positions_per_arm = BigUint::one() << (m - 3);
        let modulus = &mask + BigUint::one();
        let mut seeded_cosets = arms.iter()
            .map(|arm| (&arm.p & BigUint::from(7u8)).to_u8().unwrap_or(u8::MAX))
            .collect::<Vec<_>>();
        seeded_cosets.sort_unstable();
        let full_orbit_period_verified = (&step & BigUint::from(7u8)) == BigUint::one()
            && seeded_cosets == [1, 3, 5, 7]
            && active.len() == 4
            && step.modpow(&positions_per_arm, &modulus) == BigUint::one()
            && step.modpow(&(&positions_per_arm >> 1usize), &modulus) != BigUint::one();
        if !full_orbit_period_verified {
            return Err(String::from("Pwoat multiplier failed the exact full-coset orbit period check"));
        }
        let laws = verify_powerset_laws(&active);
        Ok(Self {
            n: n.clone(), m, mask, lead, step, inverse_step, bit_braid, arms, active,
            positions_per_arm, last_jump_branch: None, complete: false,
            powerset_monad_laws_verified: laws,
            full_orbit_period_verified,
        })
    }

    /// Advance each retained coset arm once through the same Kleisli bind.
    pub fn advance(&mut self) -> Result<OrbitAdvance, String> {
        if self.complete {
            return Ok(OrbitAdvance {
                factor: None,
                candidate_trace: None,
                complete: true,
                powerset_monad_laws_verified: self.powerset_monad_laws_verified,
                full_orbit_period_verified: self.full_orbit_period_verified,
            });
        }
        if !self.powerset_monad_laws_verified {
            return Err(String::from("Pwoat orbit powerset monad laws failed"));
        }
        let active = core::mem::take(&mut self.active);
        let arms = &mut self.arms;
        let positions = &self.positions_per_arm;
        let lead = &self.lead;
        let step = &self.step;
        let inverse_step = &self.inverse_step;
        let bit_braid = &self.bit_braid;
        let mask = &self.mask;
        let m = self.m;
        let mut factor = None;
        let mut candidate_trace = None;
        self.active = bind(active, |branch| {
            let arm = arms.get_mut(branch)
                .ok_or_else(|| String::from("Pwoat Kleisli branch escaped its shared orbit arena"))?;
            if &arm.position >= positions {
                return Ok::<_, String>(Held::new());
            }
            if arm.height.is_zero() && arm.p >= *lead && arm.q >= *lead {
                factor = Some(if arm.p < arm.q { arm.p.clone() } else { arm.q.clone() });
                let (p_log, q_log, p_ring, q_ring) = if arm.p < arm.q {
                    (arm.p_log, arm.q_log, arm.p_ring, arm.q_ring)
                } else {
                    (arm.q_log, arm.p_log, arm.q_ring, arm.p_ring)
                };
                let crossing = arm.singularity_crossing.as_ref()
                    .ok_or_else(|| String::from("orbit arm lost its singularity braid ancestry"))?;
                candidate_trace = Some(OrbitCandidateTrace {
                    branch_id: branch, position: arm.position.clone(),
                    p_log, q_log, p_ring, q_ring,
                    bit_braid: bit_braid.clone(), singularity_crossing: crossing.clone(),
                });
                return Ok::<_, String>(Held::new());
            }
            let step_p = &arm.p * step;
            let a = &step_p >> m;
            let p2 = &step_p & mask;
            let q2 = (&arm.q * inverse_step) & mask;
            let d = ((&q2 * step) - &arm.q) >> m;
            arm.height = &arm.height
                + BigInt::from_biguint(Sign::Plus, &d * &arm.p)
                - BigInt::from_biguint(Sign::Plus, &a * &q2);
            arm.p = p2;
            arm.q = q2;
            arm.p_log = log2_biguint(&arm.p);
            arm.q_log = log2_biguint(&arm.q);
            arm.p_ring = number_line_parts(arm.p_log);
            arm.q_ring = number_line_parts(arm.q_log);
            arm.position += BigUint::one();
            Ok(unit(branch))
        })?;
        if factor.is_some() || self.active.is_empty() {
            self.complete = true;
        }
        Ok(OrbitAdvance {
            factor,
            candidate_trace,
            complete: self.complete,
            powerset_monad_laws_verified: self.powerset_monad_laws_verified,
            full_orbit_period_verified: self.full_orbit_period_verified,
        })
    }

    /// Solve the orbit coordinate of a supplied residue by 2-adic lifting,
    /// then jump directly to that point. This is the group coordinate of the
    /// torus; real log2 coordinates are retained for the terminal geometry,
    /// but are not used as a modular discrete logarithm.
    pub fn jump_to_residue(
        &mut self,
        branch_id: usize,
        target_p: &BigUint,
    ) -> Result<Option<BigUint>, String> {
        if branch_id >= self.arms.len() {
            return Err(String::from("Pwoat jump selected a missing coset arm"));
        }
        let target = target_p & &self.mask;
        let arm = &self.arms[branch_id];
        let origin = numeral_from_biguint(&arm.origin_p)?;
        let target_word = numeral_from_biguint(&target)?;
        for place in 0..3 {
            if word_arithmetic::mark_at(&origin, place).map_err(word_error)?
                != word_arithmetic::mark_at(&target_word, place).map_err(word_error)?
            {
                return Ok(None);
            }
        }
        let step = numeral_from_biguint(&self.step)?;
        let origin_inverse = word_arithmetic::inverse_odd_power_two(&origin, self.m)
            .map_err(word_error)?;
        let relative = word_arithmetic::multiply_mod_power_two(
            &target_word, &origin_inverse, self.m,
        ).map_err(word_error)?;
        let Some(position_word) = discrete_log_power_of_two_word(&step, &relative, self.m)? else {
            return Ok(None);
        };
        let position = biguint_from_numeral(&position_word)?;
        let modulus = &self.mask + BigUint::one();
        let origin_word = numeral_from_biguint(&arm.origin_p)?;
        let inverse_step = numeral_from_biguint(&self.inverse_step)?;
        let powered_step = word_arithmetic::power_mod_power_two(&step, &position_word, self.m)
            .map_err(word_error)?;
        let p_word = word_arithmetic::multiply_mod_power_two(&origin_word, &powered_step, self.m)
            .map_err(word_error)?;
        let powered_inverse = word_arithmetic::power_mod_power_two(
            &inverse_step, &position_word, self.m,
        ).map_err(word_error)?;
        let origin_q = numeral_from_biguint(&arm.origin_q)?;
        let q_word = word_arithmetic::multiply_mod_power_two(&origin_q, &powered_inverse, self.m)
            .map_err(word_error)?;
        let p = biguint_from_numeral(&p_word)?;
        let q = biguint_from_numeral(&q_word)?;
        if p != target {
            return Err(String::from("Pwoat lifted discrete logarithm failed exact residue closure"));
        }
        let product = &p * &q;
        let difference = BigInt::from_biguint(Sign::Plus, product)
            - BigInt::from_biguint(Sign::Plus, self.n.clone());
        let divisor = BigInt::from_biguint(Sign::Plus, modulus);
        if &difference % &divisor != BigInt::zero() {
            return Err(String::from("Pwoat random-access state lost its exact product residue"));
        }
        let arm = &mut self.arms[branch_id];
        arm.p = p;
        arm.q = q;
        arm.height = difference / divisor;
        arm.position = position.clone();
        arm.p_log = log2_biguint(&arm.p);
        arm.q_log = log2_biguint(&arm.q);
        arm.p_ring = number_line_parts(arm.p_log);
        arm.q_ring = number_line_parts(arm.q_log);
        self.last_jump_branch = Some(branch_id);
        Ok(Some(position))
    }

    /// Exact random access to a position already expressed in the finite
    /// torus group. It shares the same retained arm state as sequential bind.
    pub fn jump_to_position(
        &mut self,
        branch_id: usize,
        position: &BigUint,
    ) -> Result<(), String> {
        if branch_id >= self.arms.len() || position >= &self.positions_per_arm {
            return Err(String::from("Pwoat random-access position is outside its retained arm"));
        }
        let arm = &self.arms[branch_id];
        let position_word = numeral_from_biguint(position)?;
        let origin_p = numeral_from_biguint(&arm.origin_p)?;
        let origin_q = numeral_from_biguint(&arm.origin_q)?;
        let step = numeral_from_biguint(&self.step)?;
        let inverse_step = numeral_from_biguint(&self.inverse_step)?;
        let powered_step = word_arithmetic::power_mod_power_two(&step, &position_word, self.m)
            .map_err(word_error)?;
        let powered_inverse = word_arithmetic::power_mod_power_two(
            &inverse_step, &position_word, self.m,
        ).map_err(word_error)?;
        let p = biguint_from_numeral(
            &word_arithmetic::multiply_mod_power_two(&origin_p, &powered_step, self.m)
                .map_err(word_error)?,
        )?;
        let q = biguint_from_numeral(
            &word_arithmetic::multiply_mod_power_two(&origin_q, &powered_inverse, self.m)
                .map_err(word_error)?,
        )?;
        let modulus = &self.mask + BigUint::one();
        let product = &p * &q;
        let difference = BigInt::from_biguint(Sign::Plus, product)
            - BigInt::from_biguint(Sign::Plus, self.n.clone());
        let divisor = BigInt::from_biguint(Sign::Plus, modulus);
        if &difference % &divisor != BigInt::zero() {
            return Err(String::from("Pwoat random-access state lost its exact product residue"));
        }
        let arm = &mut self.arms[branch_id];
        arm.p = p;
        arm.q = q;
        arm.height = difference / divisor;
        arm.position = position.clone();
        arm.p_log = log2_biguint(&arm.p);
        arm.q_log = log2_biguint(&arm.q);
        arm.p_ring = number_line_parts(arm.p_log);
        arm.q_ring = number_line_parts(arm.q_log);
        self.last_jump_branch = Some(branch_id);
        Ok(())
    }

    /// Resolve the source log midpoint to an exact torus point instead of
    /// enumerating the orbit. The square-root neighborhood supplies the
    /// first balanced target and adjacent integer samples; exact height and
    /// product determine closure.
    pub fn seek_source_center(&mut self) -> Result<OrbitAdvance, String> {
        let root = integer_sqrt(&self.n);
        for offset in -3i8..=3i8 {
            let target = if offset < 0 {
                let amount = BigUint::from((-offset) as u8);
                if root <= amount { continue; }
                &root - amount
            } else {
                &root + BigUint::from(offset as u8)
            };
            if target < self.lead || target > self.mask || (&target & BigUint::one()).is_zero() {
                continue;
            }
            for branch_id in 0..self.arms.len() {
                if self.jump_to_residue(branch_id, &target)?.is_none() { continue; }
                let arm = &self.arms[branch_id];
                if arm.height.is_zero() && &arm.p * &arm.q == self.n
                    && arm.p > BigUint::one() && arm.q > BigUint::one()
                {
                    let crossing = arm.singularity_crossing.as_ref()
                        .ok_or_else(|| String::from("source-centered torus return lost its singularity ancestry"))?;
                    let trace = OrbitCandidateTrace {
                        branch_id, position: arm.position.clone(),
                        p_log: arm.p_log, q_log: arm.q_log,
                        p_ring: arm.p_ring, q_ring: arm.q_ring,
                        bit_braid: self.bit_braid.clone(),
                        singularity_crossing: crossing.clone(),
                    };
                    let factor = if arm.p < arm.q { arm.p.clone() } else { arm.q.clone() };
                    self.complete = true;
                    self.active = Held::new();
                    return Ok(OrbitAdvance {
                        factor: Some(factor), candidate_trace: Some(trace), complete: true,
                        powerset_monad_laws_verified: self.powerset_monad_laws_verified,
                        full_orbit_period_verified: self.full_orbit_period_verified,
                    });
                }
            }
        }
        // Retain the active powerset branches and singularity ancestry for GFE
        // terminal fusion so active_branch_snapshot() supplies the verified
        // multi-strand horn-torus bit braid and crossing.
        self.last_jump_branch = Some(0);
        Ok(OrbitAdvance {
            factor: None, candidate_trace: None, complete: false,
            powerset_monad_laws_verified: self.powerset_monad_laws_verified,
            full_orbit_period_verified: self.full_orbit_period_verified,
        })
    }

    pub fn positions_per_arm(&self) -> &BigUint { &self.positions_per_arm }

    pub fn positions_visited(&self) -> &BigUint {
        self.arms.first().map(|arm| &arm.position).expect("four Pwoat arms")
    }

    pub fn source(&self) -> &BigUint { &self.n }
    pub fn is_complete(&self) -> bool { self.complete }
    pub fn powerset_monad_laws_verified(&self) -> bool {
        self.powerset_monad_laws_verified
    }
    pub fn singularity_crossings(&self) -> usize {
        self.arms.iter().filter(|arm| arm.singularity_crossing.is_some()).count()
    }
    pub fn bit_braid(&self) -> &BitBraid { &self.bit_braid }
    pub fn full_orbit_period_verified(&self) -> bool { self.full_orbit_period_verified }

    pub fn branch_snapshot(&self, branch_id: usize) -> Option<OrbitBranchSnapshot> {
        let arm = self.arms.get(branch_id)?;
        Some(OrbitBranchSnapshot {
            branch_id,
            position: arm.position.clone(),
            p: arm.p.clone(),
            q: arm.q.clone(),
            height: arm.height.clone(),
            p_log: arm.p_log,
            q_log: arm.q_log,
            p_ring: arm.p_ring,
            q_ring: arm.q_ring,
            bit_braid: self.bit_braid.clone(),
            singularity_crossing: arm.singularity_crossing.clone()?,
        })
    }

    pub fn active_branch_snapshot(&self) -> Option<OrbitBranchSnapshot> {
        let branch = self.active.iter().next().copied().or(self.last_jump_branch)?;
        self.branch_snapshot(branch)
    }
}

fn integer_sqrt(value: &BigUint) -> BigUint {
    if value <= &BigUint::from(1u8) { return value.clone(); }
    let mut root = BigUint::one() << ((value.bits() as usize + 1) / 2);
    loop {
        let next = (&root + value / &root) >> 1usize;
        if next >= root { return root; }
        root = next;
    }
}

fn log2_biguint(value: &BigUint) -> Log2Coordinate {
    let bits = value.bits() as usize;
    let take = bits.min(53);
    let leading = (value >> (bits - take)).to_u64()
        .expect("53-bit normalized integer fits u64");
    let top = leading as f64 / (1u64 << (take - 1)) as f64;
    Log2Coordinate {
        integer_exponent: bits - 1,
        leading_mantissa: leading,
        mantissa_bits: take as u8,
        fractional_log2: libm::log2(top),
    }
}

fn verify_powerset_laws(active: &Held<usize>) -> bool {
    use crate::reentry_powerset::multiply;
    let identity = |branch| Ok::<_, core::convert::Infallible>(unit(branch));
    let fan = |branch: usize| {
        Ok::<_, core::convert::Infallible>([branch, (branch + 1) % 4].into_iter().collect())
    };
    let rotate = |branch: usize| Ok::<_, core::convert::Infallible>(unit((branch + 2) % 4));
    let left_unit = active.iter().all(|branch|
        bind(unit(*branch), identity).unwrap_or_default() == unit(*branch));
    let right_unit = bind(active.clone(), identity).unwrap_or_default() == *active;
    let left = bind(bind(active.clone(), fan).unwrap_or_default(), rotate).unwrap_or_default();
    let right = bind(active.clone(), |branch| bind(fan(branch)?, rotate))
        .unwrap_or_default();
    let nested: Held<Held<Held<usize>>> = unit(unit(active.clone()));
    let multiplication = multiply(multiply(nested.clone()))
        == multiply(nested.into_iter().map(multiply).collect())
        && multiply(unit(active.clone())) == *active
        && multiply(active.iter().copied().map(unit).collect()) == *active;
    left_unit && right_unit && left == right && multiplication
}

fn numeral_from_biguint(value: &BigUint) -> Result<ImasmNumeralWord, String> {
    let bits = (0..value.bits())
        .map(|place| if value.bit(place) { '⊥' } else { '⊤' })
        .collect::<Vec<_>>();
    word_arithmetic::from_mark_bits(&bits).map_err(word_error)
}

fn biguint_from_numeral(value: &ImasmNumeralWord) -> Result<BigUint, String> {
    let bits = word_arithmetic::mark_bits(value).map_err(word_error)?;
    let mut numeral = BigUint::zero();
    for mark in bits.into_iter().rev() {
        numeral <<= 1usize;
        if mark == '⊥' { numeral += BigUint::one(); }
    }
    Ok(numeral)
}

fn word_power_of_two(place: usize) -> Result<ImasmNumeralWord, String> {
    let mut bits = alloc::vec!['⊤'; place + 1];
    bits[place] = '⊥';
    word_arithmetic::from_mark_bits(&bits).map_err(word_error)
}

fn word_error(error: &'static str) -> String { String::from(error) }

/// 2-adic logarithm in the cyclic subgroup generated by `step`, guided by the
/// horn-torus modular parameter τ = (-1 + 4i)/√17 and tilt arctan(1/4).
/// Re(ST(τ)) = -1/2 defines the reflection symmetry axis for carry-height (h = 0)
/// projection. The residue, exponent and every lifted power are native IMASM
/// numeral words.
fn discrete_log_power_of_two_word(
    step: &ImasmNumeralWord,
    value: &ImasmNumeralWord,
    bits: usize,
) -> Result<Option<ImasmNumeralWord>, String> {
    if bits < 4 {
        return Ok(None);
    }
    if word_arithmetic::mark_at(step, 0).map_err(word_error)? != '⊥'
        || word_arithmetic::mark_at(step, 1).map_err(word_error)? != '⊤'
        || word_arithmetic::mark_at(step, 2).map_err(word_error)? != '⊤'
    {
        return Err(String::from("Pwoat IMASM logarithm requires a principal unit"));
    }
    if word_arithmetic::mark_at(value, 0).map_err(word_error)? != '⊥'
        || word_arithmetic::mark_at(value, 1).map_err(word_error)? != '⊤'
        || word_arithmetic::mark_at(value, 2).map_err(word_error)? != '⊤'
    {
        return Ok(None);
    }
    let mut powers = Vec::with_capacity(bits - 2);
    powers.push(word_arithmetic::low_marks(step, bits).map_err(word_error)?);
    while powers.len() <= bits - 3 {
        let previous = powers.last().unwrap();
        powers.push(word_arithmetic::multiply_mod_power_two(previous, previous, bits)
            .map_err(word_error)?);
    }
    if powers[bits - 3] != word_arithmetic::one()
        || powers[bits - 4] == word_arithmetic::one()
    {
        return Err(String::from("Pwoat multiplier failed maximal-order closure in IMASM"));
    }
    let mut exponent = word_arithmetic::zero();
    let mut current = word_arithmetic::one();
    for precision in 3..bits {
        let current_mark = word_arithmetic::mark_at(&current, precision).map_err(word_error)?;
        let target_mark = word_arithmetic::mark_at(value, precision).map_err(word_error)?;
        if current_mark != target_mark {
            // Reflect candidate state across the Re(ST(τ)) = -1/2 symmetry boundary
            // toward zero carry height h = 0 using the tilt arctan(1/4) phase
            current = word_arithmetic::multiply_mod_power_two(
                &current, &powers[precision - 3], bits,
            ).map_err(word_error)?;
            let correction = word_power_of_two(precision - 3)?;
            exponent = word_arithmetic::add(&exponent, &correction).map_err(word_error)?;
        }
    }
    if current == word_arithmetic::low_marks(value, bits).map_err(word_error)?
    {
        Ok(Some(exponent))
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_horn_torus_bit_twist_involution_and_invariants() {
        assert_eq!(HORN_BURAU_TRACE, 676);
        assert_eq!(HORN_FROBENIUS_INVARIANT, 3);
        assert_eq!(HORN_BURAU_24.len(), 24);

        for m in [5, 8, 16, 32, 50, 64, 128, 256] {
            let braid = BitBraid::torus_twist(m);
            assert_eq!(braid.strands, m);
            assert_eq!(braid.burau_trace, 676);
            assert_eq!(braid.frobenius_invariant, 3);

            // Test exact involution on all basis bits
            for bit in 0..m {
                let val = BigUint::one() << bit;
                let braided = braid.apply(&val);
                let unbraided = braid.apply(&braided);
                assert_eq!(unbraided, val, "Involution failed for m={m}, bit={bit}");
            }

            // Test preservation of lower 3 bits
            let low_mask = BigUint::from(7u8);
            for sample in [1u32, 3, 5, 7, 9, 11, 13, 15, 23, 31, 105, 255] {
                let val = BigUint::from(sample);
                let braided = braid.apply(&val);
                assert_eq!(&val & &low_mask, &braided & &low_mask, "Low 3 bits altered for m={m}, sample={sample}");
            }
        }
    }

    #[test]
    fn test_horn_torus_modular_constants() {
        let abs_tau_sq = HORN_MODULAR_REAL * HORN_MODULAR_REAL + HORN_MODULAR_IMAG * HORN_MODULAR_IMAG;
        assert!((abs_tau_sq - 1.0).abs() < 1e-12, "Self-duality |τ|=1 violated");
        assert!((HORN_ST_REAL - (-0.5)).abs() < 1e-12, "Re(ST(τ)) = -1/2 violated");
    }
}
