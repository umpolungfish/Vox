//! Log-phase and dimension-twelve dual-frame closure for source-bound factor candidates.
//!
//! Candidate production remains a live Vox membrane. This module supplies the
//! geometric index, the cyclic Gabor score, the SIC dual reconstruction and the
//! exact product closure used to accept its returned pair.

use alloc::{string::{String, ToString}, vec, vec::Vec};
use crate::fixed_point_imasm::ImasmNumeralWord;

use crate::{factor_synthesis, morphism_factor as mf};

pub const LOG_PERIOD: u32 = 11;
pub const GABOR_BINS: i64 = 2048;
pub const SIC_DIMENSION: usize = 12;
pub const SIC_OUTCOMES: usize = SIC_DIMENSION * SIC_DIMENSION;
/// Source-bound GFE carrier word; the final ⊡ is the immutable readout latch.
pub const GFE_MORPHISM_WORD: &str = "⊣⊣⊙∈∈≻⊙⊞≺⊤⊥∋∋⊡";
const WIDTH_PHASE_SIGMA_BINS: f64 = 76.0;
// log2(phi), where phi^2 = phi + 1. This lets the linear number-line
// coordinate use powers of the golden number without changing the exact
// multiplicative relation log_phi(pq)=log_phi(p)+log_phi(q).
const LOG2_GOLDEN: f64 = 0.6942419136306174;
pub const SMALL_PRIME_ORBIT_PRIMES: [u32; 15] = [2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47];
const SMALL_PRIME_PHASE_CELLS: usize = GABOR_BINS as usize;
const SMALL_PRIME_DEPTH_CELLS: usize = SIC_DIMENSION;
const SMALL_PRIME_TRINE_CELLS: usize = 7;
const SMALL_PRIME_ORBIT_CELLS: usize =
    SMALL_PRIME_PHASE_CELLS * SMALL_PRIME_DEPTH_CELLS * SMALL_PRIME_TRINE_CELLS;

/// Compute the exact remainder of an IMASM tape numeral modulo a 32-bit integer.
pub fn tape_rem_u32(tape: &[char], modulus: u32) -> u32 {
    let mut acc = 0u64;
    for &ch in tape.iter().rev() {
        let b = if ch == '1' || ch == '⊥' || ch == crate::vox::EVALF {
            1u64
        } else {
            0u64
        };
        acc = ((acc << 1) + b) % (modulus as u64);
    }
    acc as u32
}

/// Finite small-prime orbit sieve stored as one IMASM value word. The three
/// coordinates are phase, SIC depth and the seven-cell number-line section;
/// their addresses are positions, while each mask value is a ⊤/⊥ mark.
#[derive(Clone)]
pub struct SmallPrimeOrbitMask {
    word: ImasmNumeralWord,
    marks: Vec<char>,
    marked_cells: usize,
}

/// Source-bound prime-power re-entry frontier. Each prime p owns the exact
/// braid p, p², p³, … . If gcd(N,p)=1, every later value on that braid is
/// source-incompatible, so the whole p-rail can be removed after its first
/// exact tape check. A hit returns one divisor; the caller derives the other
/// factor from N.
pub struct PrimeBraidProbe {
    source: Vec<char>,
    next_prime: usize,
    closed_braids: Vec<(u32, usize)>,
}

impl PrimeBraidProbe {
    pub fn new(source: &[char]) -> Self {
        Self { source: mf::trim(source.to_vec()), next_prime: 0, closed_braids: Vec::new() }
    }

    pub fn closed_braids(&self) -> &[(u32, usize)] { &self.closed_braids }

    /// Close one exact prime-power braid, or return its first proper divisor.
    /// The count is the number of values p^k<N removed by the source-coprime
    /// proof; powers are generated as IMASM tapes, not inferred from logs.
    pub fn advance(&mut self) -> Result<Option<mf::SourceDivisor>, String> {
        let Some(&prime) = SMALL_PRIME_ORBIT_PRIMES.get(self.next_prime) else {
            return Ok(None);
        };
        let prime_tape = mf::decimal_to_tape(&prime.to_string())
            .ok_or_else(|| String::from("prime braid numeral conversion failed"))?;
        self.next_prime += 1;
        if mf::cmp(&prime_tape, &mf::one()) == core::cmp::Ordering::Greater
            && mf::cmp(&prime_tape, &self.source) == core::cmp::Ordering::Less
        {
            if tape_rem_u32(&self.source, prime) == 0 {
                return Ok(Some(mf::SourceDivisor {
                    divisor: prime_tape, producer: "source-prime-braid",
                }));
            }
        }

        let mut value = prime_tape.clone();
        let mut reentries = 0usize;
        while mf::cmp(&value, &self.source) == core::cmp::Ordering::Less {
            reentries += 1;
            value = mf::mul(&value, &prime_tape);
        }
        self.closed_braids.push((prime, reentries));
        Ok(None)
    }

    /// Check all small prime braids immediately and close their orbits.
    pub fn sweep_all(&mut self) -> Result<Option<mf::SourceDivisor>, String> {
        while self.next_prime < SMALL_PRIME_ORBIT_PRIMES.len() {
            if let Some(divisor) = self.advance()? {
                return Ok(Some(divisor));
            }
        }
        Ok(None)
    }
}

impl SmallPrimeOrbitMask {
    pub fn prepare() -> Result<Self, String> {
        let mut marks = vec!['⊤'; SMALL_PRIME_ORBIT_CELLS];
        for prime in SMALL_PRIME_ORBIT_PRIMES {
            let log = libm::log2(prime as f64);
            let phase_step = phase_bin(log) as usize;
            let trine_step = (log.floor() as usize) % SMALL_PRIME_TRINE_CELLS;
            let phase_period = SMALL_PRIME_PHASE_CELLS / gcd_usize(phase_step, SMALL_PRIME_PHASE_CELLS);
            let trine_period = SMALL_PRIME_TRINE_CELLS / gcd_usize(trine_step, SMALL_PRIME_TRINE_CELLS);
            let period = lcm_usize(lcm_usize(phase_period, SMALL_PRIME_DEPTH_CELLS), trine_period);
            let (mut phase, mut depth, mut trine) = (0usize, 0usize, 0usize);
            for _ in 0..period {
                let address = (depth * SMALL_PRIME_TRINE_CELLS + trine)
                    * SMALL_PRIME_PHASE_CELLS + phase;
                marks[address] = '⊥';
                phase = (phase + phase_step) % SMALL_PRIME_PHASE_CELLS;
                depth = (depth + 1) % SMALL_PRIME_DEPTH_CELLS;
                trine = (trine + trine_step) % SMALL_PRIME_TRINE_CELLS;
            }
        }
        let word = crate::fixed_point_word_arithmetic::from_mark_bits(&marks)
            .map_err(|error| String::from(error))?;
        let mut marks = crate::fixed_point_word_arithmetic::mark_bits(&word)
            .map_err(|error| String::from(error))?;
        marks.resize(SMALL_PRIME_ORBIT_CELLS, '⊤');
        let marked_cells = marks.iter().filter(|&&mark| mark == '⊥').count();
        Ok(Self { word, marks, marked_cells })
    }

    /// Import a mask word prepared by the GFE Cargo host build.
    pub fn from_imasm_word(word: &str) -> Result<Self, String> {
        let word = ImasmNumeralWord::checked(word).map_err(String::from)?;
        let mut marks = crate::fixed_point_word_arithmetic::mark_bits(&word)
            .map_err(String::from)?;
        if marks.len() > SMALL_PRIME_ORBIT_CELLS {
            return Err(String::from("prepared small-prime mask exceeds its phase/depth/trine carrier"));
        }
        marks.resize(SMALL_PRIME_ORBIT_CELLS, '⊤');
        let marked_cells = marks.iter().filter(|&&mark| mark == '⊥').count();
        Ok(Self { word, marks, marked_cells })
    }

    pub fn word(&self) -> &ImasmNumeralWord { &self.word }

    pub fn marked_cells(&self) -> usize { self.marked_cells }

    /// True when this finite phase/depth/trine address lies on a small-prime
    /// orbit. Callers use this as an ordering signal and retain every lane.
    pub fn contains(&self, phase: usize, depth: usize, trine: usize) -> bool {
        let address = ((depth % SMALL_PRIME_DEPTH_CELLS) * SMALL_PRIME_TRINE_CELLS
            + trine % SMALL_PRIME_TRINE_CELLS) * SMALL_PRIME_PHASE_CELLS
            + phase % SMALL_PRIME_PHASE_CELLS;
        self.marks.get(address) == Some(&'⊥')
    }

    fn covers_phase_section(&self, phase: usize, trine: usize) -> bool {
        (0..SMALL_PRIME_DEPTH_CELLS).any(|depth| self.contains(phase, depth, trine))
    }
}

fn gcd_usize(mut left: usize, mut right: usize) -> usize {
    while right != 0 { let remainder = left % right; left = right; right = remainder; }
    left.max(1)
}

fn lcm_usize(left: usize, right: usize) -> usize {
    left / gcd_usize(left, right) * right
}


#[derive(Clone, Debug)]
pub struct GeometerFrameReadout {
    pub p: Vec<char>,
    pub q: Vec<char>,
    pub producer: &'static str,
    pub phase_bin_p: i64,
    pub phase_bin_q: i64,
    pub phase_bin_n: i64,
    pub phase_residual_bins: i64,
    /// Exponent and mantissa coordinates carried to the terminal fusion.
    pub log2_p: crate::pwoat_geometry::Log2Coordinate,
    pub log2_q: crate::pwoat_geometry::Log2Coordinate,
    pub log2_n: crate::pwoat_geometry::Log2Coordinate,
    /// Terminal fusion of the retained exponent and mantissa coordinates.
    pub fused_log2_p: f64,
    pub fused_log2_q: f64,
    pub fused_log2_n: f64,
    pub source_ring: crate::pwoat_geometry::RingCoordinate,
    /// Fractional turns on the HTML's k=1 ring, each in [0,1).
    pub numberline_turn_p: f64,
    pub numberline_turn_q: f64,
    pub numberline_turn_n: f64,
    /// Whole turns plus the fractional carry close the additive trine.
    pub numberline_wrap_p: i128,
    pub numberline_wrap_q: i128,
    pub numberline_wrap_n: i128,
    pub numberline_turn_residual: f64,
    pub numberline_wrap_balance: i128,
    pub winding_p: i64,
    pub winding_q: i64,
    pub winding_n: i64,
    pub omega_p: u8,
    pub omega_q: u8,
    pub omega_n: u8,
    pub sic_candidate_label: usize,
    pub sic_source_label: usize,
    pub gabor_score: f64,
    pub sic_dual_score: f64,
    pub combined_score: f64,
    pub product_closed: bool,
    pub gfe_powerset_monad_laws_verified: bool,
    pub torus_powerset_monad_laws_verified: bool,
    pub torus_full_orbit_verified: bool,
    pub catalyst_order_applied: bool,
    pub catalyst_inverse_returned: bool,
    pub final_readout_fixed: bool,
    /// Torus branch and orbit position that produced this return, when the
    /// torus arm rather than resident GLuT supplied the divisor.
    pub torus_branch_id: Option<usize>,
    pub torus_branch_position: Option<num_bigint::BigUint>,
    pub torus_bit_braid: Option<crate::pwoat_torus::BitBraid>,
    pub torus_singularity_crossing: Option<crate::pwoat_torus::BitBraidCrossing>,
    pub torus_branch_snapshot: Option<crate::pwoat_torus::OrbitBranchSnapshot>,
    /// The compiled QFT plus QC braid's complex state after acting on the
    /// N-derived phase coordinate and ordering the producing resident axes.
    pub circuit_braid: Option<crate::gfe_circuit_braid::GfeCircuitBraidState>,
    pub morphism_applications: Vec<factor_synthesis::GfeMorphismApplication>,
    /// Exact re-entry dots on the returned factor braids: p^k, q^k, and N^k.
    pub braid_dots: Vec<factor_synthesis::FactorBraidDot>,
}

pub struct GeometerFactorExtractor {
    arena: Vec<GfeResidentState>,
    active: crate::reentry_powerset::Held<usize>,
}

/// Source-port result records whether GFE's own prepared resident already
/// produced and closed the divisor, so the enclosing carrier does not close
/// that return twice.
pub struct GeometerSourcePortReturn {
    pub divisor: mf::SourceDivisor,
    pub resident_closed: bool,
}

struct GfeResidentState {
    source: Vec<char>,
    source_log: crate::pwoat_geometry::Log2Coordinate,
    source_ring: crate::pwoat_geometry::RingCoordinate,
    circuit_braid: crate::gfe_circuit_braid::GfeCircuitBraidState,
    torus_powerset_monad_laws_verified: bool,
    torus_full_orbit_verified: bool,
    torus_source_seek_done: bool,
    horn_shiab_done: bool,
    source_carrier_started: bool,
    source_carrier_progress: Option<(&'static str, u64)>,
    prime_braids: PrimeBraidProbe,
    torus_candidate_trace: Option<crate::pwoat_torus::OrbitCandidateTrace>,
    small_prime_orbit_mask: SmallPrimeOrbitMask,
    torus: crate::pwoat_torus::PwoatTorusOrbit,
    synthesis: factor_synthesis::FactorSynthesis,
    terminal_readout: Option<GeometerFrameReadout>,
}

/// Re-enter the source's SHIAB frame through the resident horn braid. Each
/// cyclic source view is deinterleaved, both lanes receive the same exact
/// strand permutation, and the Picard-Lefschetz shear (a,b) -> (a+b,b) is
/// evaluated as a product residual against N. A proper gcd is returned only
/// after exact quotient and product closure.
fn horn_torus_shiab_divisor(
    source: &[char],
    braid: &crate::pwoat_torus::BitBraid,
) -> Result<Option<(usize, Vec<char>, Vec<char>)>, String> {
    let source = mf::trim(source.to_vec());
    if source.len() < 2 { return Ok(None); }
    let width = source.len().div_ceil(2);
    let one = mf::one();
    for shift in 0..source.len() {
        let mut even = Vec::with_capacity(width);
        let mut odd = Vec::with_capacity(width);
        for index in 0..source.len() {
            let bit = source[(index + shift) % source.len()];
            if index % 2 == 0 { even.push(bit); } else { odd.push(bit); }
        }
        let before = crate::ordering::selected("shear", 2)?.apply(&[mf::trim(even), mf::trim(odd)])?;
        let composition = crate::ordering::selected("shear-braid", 2)?;
        let lanes = execute_ordered_shear_braid(&before, braid, &composition)?;
        let sheared_product = mf::mul(&lanes[0], &lanes[1]);
        let residual = if mf::cmp(&source, &sheared_product) == core::cmp::Ordering::Greater {
            mf::sub(&source, &sheared_product)
        } else {
            mf::sub(&sheared_product, &source)
        };
        let candidate = mf::gcd(residual, source.clone());
        if mf::cmp(&candidate, &one) != core::cmp::Ordering::Greater
            || mf::cmp(&candidate, &source) != core::cmp::Ordering::Less
        {
            continue;
        }
        let (quotient, remainder) = mf::divmod(&source, &candidate);
        if !mf::zero(&remainder)
            || mf::cmp(&mf::mul(&candidate, &quotient), &source) != core::cmp::Ordering::Equal
        {
            return Err(String::from("horn-torus SHIAB return failed exact N closure"));
        }
        return Ok(Some((shift, candidate, quotient)));
    }
    Ok(None)
}

fn execute_ordered_shear_braid(
    before: &[Vec<char>],
    braid: &crate::pwoat_torus::BitBraid,
    composition: &crate::ordering::Permutation,
) -> Result<Vec<Vec<char>>, String> {
    if before.len() != 2 {
        return Err("shear/braid carrier requires exactly two lanes".into());
    }
    crate::ordering::Permutation::checked(composition.indices.clone(), 2)?;
    let braid_lane = |lane: &[char], inverse: bool| -> Result<Vec<char>, String> {
        let word = ImasmNumeralWord::checked(&mf::emit_numeral(lane)).map_err(String::from)?;
        let lane_width = braid.strands.max(
            crate::fixed_point_word_arithmetic::mark_bits(&word)
                .map_err(String::from)?.len(),
        );
        let result = if inverse { braid.return_word(&word, lane_width)? } else { braid.apply_word(&word, lane_width)? };
        mf::parse_numeral(result.as_str())
    };
    let mut lanes = before.to_vec();
    for &operation in &composition.indices {
        if operation == 0 {
            lanes = lanes.iter().map(|lane| braid_lane(lane, false)).collect::<Result<_, _>>()?;
        } else {
            lanes[0] = mf::add(&lanes[0], &lanes[1]);
        }
    }
    let mut recovered = lanes.clone();
    for &operation in composition.indices.iter().rev() {
        if operation == 0 {
            recovered = recovered.iter().map(|lane| braid_lane(lane, true)).collect::<Result<_, _>>()?;
        } else {
            if mf::cmp(&recovered[0], &recovered[1]) == core::cmp::Ordering::Less {
                return Err("shear inverse lost its lane boundary".into());
            }
            recovered[0] = mf::sub(&recovered[0], &recovered[1]);
        }
    }
    if recovered != before {
        return Err("selected shear/braid composition failed exact return".into());
    }
    Ok(lanes)
}

#[cfg(test)]
mod shear_braid_order_tests {
    use super::*;

    const TARGET: &str = include_str!("../../G-mOMonadOS/measurements/geometer_factor_battery_20261024/horn_shiab_target_250/input.decimal");
    const CONTROL: &str = include_str!("../../G-mOMonadOS/measurements/geometer_factor_battery_20261024/horn_shiab_control_250/input.decimal");

    fn split_source(raw: &str) -> (usize, Vec<char>, Vec<Vec<char>>) {
        let number = num_bigint::BigUint::parse_bytes(raw.trim().as_bytes(), 10).unwrap();
        assert!(number.bits() >= 200);
        let word = ImasmNumeralWord::checked(&crate::combined_factor::native_word(&number)).unwrap();
        let source = mf::parse_numeral(word.as_str()).unwrap();
        let braid_strands = source.len().div_ceil(2);
        let mut lanes = [Vec::new(), Vec::new()];
        for (index, mark) in source.iter().copied().enumerate() {
            lanes[index % 2].push(mark);
        }
        (braid_strands, source, lanes.into_iter().map(mf::trim).collect())
    }

    #[test]
    fn selected_shear_braid_order_returns_target_and_control_through_its_inverse() {
        let braid_order = crate::ordering::Permutation::parse("reverse", 2).unwrap();
        let canonical = crate::ordering::Permutation::parse("canonical", 2).unwrap();
        let reversed = crate::ordering::Permutation::parse("reverse", 2).unwrap();
        let mut expanded_before_braid = false;
        for raw in [TARGET, CONTROL] {
            let (strands, source, source_lanes) = split_source(raw);
            let braid = crate::pwoat_torus::BitBraid::torus_twist(strands);
            let selected_lanes = braid_order.apply(&source_lanes).unwrap();
            let selected_result = execute_ordered_shear_braid(&selected_lanes, &braid, &reversed).unwrap();
            let canonical_result = execute_ordered_shear_braid(&source_lanes, &braid, &canonical).unwrap();
            assert_ne!(selected_result, canonical_result);

            for shift in 0..source.len() {
                let mut even = Vec::new();
                let mut odd = Vec::new();
                for index in 0..source.len() {
                    if index % 2 == 0 { even.push(source[(index + shift) % source.len()]); }
                    else { odd.push(source[(index + shift) % source.len()]); }
                }
                let before = braid_order.apply(&[mf::trim(even), mf::trim(odd)]).unwrap();
                let sheared = mf::add(&before[0], &before[1]);
                let sheared_word = ImasmNumeralWord::checked(&mf::emit_numeral(&sheared)).unwrap();
                let expanded = crate::fixed_point_word_arithmetic::mark_bits(&sheared_word).unwrap().len() > strands;
                if expanded {
                    expanded_before_braid = true;
                    execute_ordered_shear_braid(&before, &braid, &reversed).unwrap();
                    break;
                }
            }
        }
        assert!(expanded_before_braid, "control battery never exercised a shear-expanded braid carrier");
    }
}

impl GeometerFactorExtractor {
    /// Prepares the Vox candidate membrane from N alone.
    pub fn new(source: &[char]) -> Result<Self, String> {
        Self::new_with_orbit_mask(source, SmallPrimeOrbitMask::prepare()?)
    }

    /// Prepares from an IMASM source value and a build-time IMASM orbit mask.
    pub fn new_with_orbit_mask(
        source: &[char],
        small_prime_orbit_mask: SmallPrimeOrbitMask,
    ) -> Result<Self, String> {
        let source = mf::trim(source.to_vec());
        let source_number = num_bigint::BigUint::new(
            mf::tape_to_limbs(&source, mf::limb_count_for_bits(source.len())),
        );
        Self::from_tape_value(source, source_number, small_prime_orbit_mask)
    }

    /// Parse and prepare the N-only decimal input inside the initial powerset
    /// bind, so the source numeral and all derived state enter one resident.
    pub fn from_decimal(number: &str) -> Result<Self, String> {
        let small_prime_orbit_mask = SmallPrimeOrbitMask::prepare()?;
        let mut arena = Vec::new();
        let active = crate::reentry_powerset::bind(
            crate::reentry_powerset::unit(0usize),
            |slot| {
                if slot != 0 { return Err(String::from("GFE preparation lost its N-only arena slot")); }
                let source_number = num_bigint::BigUint::parse_bytes(number.as_bytes(), 10)
                    .ok_or_else(|| String::from("GFE N-only input is not a decimal integer"))?;
                if source_number <= num_bigint::BigUint::from(1u8) {
                    return Err(String::from("GFE N-only input must be greater than one"));
                }
                let source = mf::decimal_to_tape(&source_number.to_str_radix(10))
                    .ok_or_else(|| String::from("GFE source numeral conversion failed"))?;
                arena.push(prepare_gfe_resident(
                    source, source_number, small_prime_orbit_mask.clone(),
                )?);
                Ok::<_, String>(crate::reentry_powerset::unit(slot))
            },
        )?;
        Ok(Self { arena, active })
    }

    fn from_tape_value(
        source: Vec<char>,
        source_number: num_bigint::BigUint,
        small_prime_orbit_mask: SmallPrimeOrbitMask,
    ) -> Result<Self, String> {
        let mut arena = Vec::new();
        let mut input = Some((source, source_number));
        let active = crate::reentry_powerset::bind(
            crate::reentry_powerset::unit(0usize),
            |slot| {
                if slot != 0 { return Err(String::from("GFE preparation lost its N-only arena slot")); }
                let (source, source_number) = input.take()
                    .ok_or_else(|| String::from("GFE source unit was consumed more than once"))?;
                arena.push(prepare_gfe_resident(
                    source, source_number, small_prime_orbit_mask.clone(),
                )?);
                Ok::<_, String>(crate::reentry_powerset::unit(slot))
            },
        )?;
        Ok(Self { arena, active })
    }

    /// Advances the candidate membrane and returns its geometrically scored closure.
    pub fn advance(&mut self) -> Result<Option<GeometerFrameReadout>, String> {
        self.advance_observed(|_| {})
    }

    pub fn progress_report(&self) -> String {
        match self.active.iter().next().and_then(|slot| self.arena.get(*slot)) {
            Some(state) if !state.torus.is_complete() => format!(
                "stage=pwoat-torus position={} of {} active_cosets={} full_orbit_verified={} small_prime_orbit_mask={}/{} phase-depth-trine_cells={} singularity_theta=1/2 singularity_crossings={} bit_braid_generators={} torus_monad_laws={} resident_monad_laws={}",
                state.torus.positions_visited(),
                state.torus.positions_per_arm(),
                4,
                state.torus.full_orbit_period_verified(),
                state.small_prime_orbit_mask.marked_cells(),
                SMALL_PRIME_ORBIT_CELLS,
                state.small_prime_orbit_mask.word().as_str().chars().count(),
                state.torus.singularity_crossings(),
                state.torus.bit_braid().generators.len(),
                state.torus.powerset_monad_laws_verified(),
                state.synthesis.powerset_monad_laws_verified(),
            ),
            Some(state) => {
                let source_carrier = state.source_carrier_progress
                    .map(|(stage, count)| alloc::format!(" source_carrier={stage}:{count}"))
                    .unwrap_or_default();
                let pruned_reentries: usize = state.prime_braids.closed_braids()
                    .iter().map(|(_, count)| *count).sum();
                format!(
                    "stage=resident-glut monad_laws={} torus_monad_laws={} resident={:?} prime_braids={}/{} prime_reentries_pruned={}{}",
                    state.synthesis.powerset_monad_laws_verified(),
                    state.torus.powerset_monad_laws_verified(),
                    state.synthesis.resident_stats(), state.prime_braids.closed_braids().len(),
                    SMALL_PRIME_ORBIT_PRIMES.len(), pruned_reentries, source_carrier,
                )
            }
            None => String::from("stage=closed-no-active-resident"),
        }
    }

    /// Expose the executed N-bound circuit state while the factor membrane is
    /// open, at its preparation boundary.
    pub fn circuit_braid_state(&self) -> Option<&crate::gfe_circuit_braid::GfeCircuitBraidState> {
        self.active.iter().next()
            .and_then(|slot| self.arena.get(*slot))
            .map(|state| &state.circuit_braid)
    }

    /// Advances while exposing the resident fold observations at their native boundaries.
    pub fn advance_observed(
        &mut self,
        observe: impl FnMut(&crate::glut_system::FoldStats),
    ) -> Result<Option<GeometerFrameReadout>, String> {
        self.advance_observed_with_source_port(observe, |_, synthesis, observe| {
            if let Some(returned) = synthesis.advance_observed(|stats| observe(stats))? {
                Ok(Some(GeometerSourcePortReturn {
                    divisor: mf::SourceDivisor {
                        divisor: returned.p.clone(), producer: returned.producer,
                    },
                    resident_closed: true,
                }))
            } else {
                Ok(None)
            }
        })
    }

    /// Advances GFE while letting a host supply the N-only source-carrier
    /// computation. A returned divisor still enters GFE's own prepared
    /// resident before its geometric and exact terminal readout. This is the
    /// integration seam for resident mechanisms hosted by a higher crate.
    pub fn advance_observed_with_source_port(
        &mut self,
        mut observe: impl FnMut(&crate::glut_system::FoldStats),
        mut source_port: impl FnMut(&[char], &mut factor_synthesis::FactorSynthesis,
            &mut dyn FnMut(&crate::glut_system::FoldStats))
            -> Result<Option<GeometerSourcePortReturn>, String>,
    ) -> Result<Option<GeometerFrameReadout>, String> {
        if let Some(readout) = self.active.iter().next()
            .and_then(|slot| self.arena.get(*slot))
            .and_then(|state| state.terminal_readout.as_ref())
        {
            return Ok(Some(readout.clone()));
        }
        let active = core::mem::take(&mut self.active);
        let arena = &mut self.arena;
        self.active = crate::reentry_powerset::bind(active, |slot| {
            let state = arena.get_mut(slot).ok_or_else(|| String::from("GFE bind reached a missing resident"))?;
            let mut torus_return = None;
            let orbit = if !state.torus_source_seek_done {
                state.torus_source_seek_done = true;
                Some(state.torus.seek_source_center()?)
            } else {
                // The full orbit period is certified during preparation. The
                // source-centered probe is the torus candidate operation; do
                // not linearly enumerate the astronomical orbit on later binds.
                None
            };
            if let Some(orbit) = orbit {
                if !orbit.powerset_monad_laws_verified {
                    return Err(String::from("Pwoat torus powerset monad laws failed during re-entry"));
                }
                if !orbit.full_orbit_period_verified {
                    return Err(String::from("Pwoat orbit full-period invariant failed during re-entry"));
                }
                if let Some(p) = orbit.factor {
                    state.torus_candidate_trace = orbit.candidate_trace;
                    let n = state.torus.source();
                    if p <= num_bigint::BigUint::from(1u8) || &p >= n || n % &p != num_bigint::BigUint::from(0u8) {
                        return Err(String::from("Pwoat orbit returned a non-divisor candidate"));
                    }
                    let q = n / &p;
                    let product = &p * &q;
                    if product != *n {
                        return Err(String::from("Pwoat orbit return failed exact product closure"));
                    }
                    let p_tape = mf::decimal_to_tape(&p.to_str_radix(10))
                        .ok_or_else(|| String::from("Pwoat divisor tape conversion failed"))?;
                    let q_tape = mf::decimal_to_tape(&q.to_str_radix(10))
                        .ok_or_else(|| String::from("Pwoat quotient tape conversion failed"))?;
                    torus_return = Some(state.synthesis.close_candidate(
                        p_tape, q_tape, "pwoat-torus")?);
                }
            }
            if torus_return.is_none() && !state.horn_shiab_done {
                state.horn_shiab_done = true;
                let horn_return = horn_torus_shiab_divisor(
                    &state.source, state.torus.bit_braid(),
                )?;
                #[cfg(feature = "protein-cli")]
                std::eprintln!("gfe horn-torus-shiab reentries={} factor-returned={}",
                    horn_return.as_ref().map_or(state.source.len(), |(shift, _, _)| shift + 1),
                    horn_return.is_some());
                if let Some((_shift, p, q)) = horn_return {
                    torus_return = Some(state.synthesis.close_candidate(
                        p, q, "horn-torus-shiab-monodromy")?);
                }
            }
            // Use the working membrane's source-port composition: the N-only
            // source carrier invokes this resident at each bound port. A
            // produced divisor enters the live product/carry state before the
            // GFE geometric and exact terminal readout.
            let source_carrier_return = if torus_return.is_none()
                && !state.source_carrier_started
            {
                state.source_carrier_started = true;
                let source = state.source.clone();
                if let Some(divisor) = state.prime_braids.sweep_all()? {
                    let p = divisor.divisor;
                    let (q, _) = mf::divmod(&source, &p);
                    torus_return = Some(state.synthesis.close_candidate(p, q, divisor.producer)?);
                    None
                } else {
                    let mut synthesis_returned = false;
                    let mut source_port_calls = 0u64;
                    let mut carrier_progress = None;
                    #[cfg(feature = "protein-cli")]
                    let carrier_start = std::time::Instant::now();
                    #[cfg(feature = "protein-cli")]
                    let timeout_seconds = std::env::var("GFE_TIMEOUT_SECONDS")
                        .ok()
                        .and_then(|s| s.parse::<f64>().ok())
                        .unwrap_or(300.0);
                    let candidate = mf::source_carrier_divisor_with_port_and_observer(
                        &source,
                        |port_source| {
                            if port_source != source.as_slice() {
                                return Err(String::from("GFE source carrier changed its N-only port"));
                            }
                            source_port_calls = source_port_calls.saturating_add(1);
                            #[cfg(feature = "protein-cli")]
                            {
                                if carrier_start.elapsed().as_secs_f64() > timeout_seconds {
                                    return Err(format!("{timeout_seconds:.0}-second execution limit reached"));
                                }
                                std::eprintln!("gfe source-port call={source_port_calls}");
                            }
                            if let Some(returned) = source_port(
                                port_source, &mut state.synthesis, &mut observe,
                            )? {
                                synthesis_returned = returned.resident_closed;
                                Ok(Some(returned.divisor))
                            } else {
                                Ok(None)
                            }
                        },
                        |stage, count| {
                            carrier_progress = Some((stage, count));
                            #[cfg(feature = "protein-cli")]
                            if count == 0 || stage == "carrier-round-start"
                                || stage == "carrier-round-complete"
                                || count % 100 == 0 || stage.ends_with("return")
                                || stage.ends_with("exhausted")
                            {
                                std::eprintln!("gfe source-carrier stage={stage} count={count}");
                            }
                        },
                    )?;
                    state.source_carrier_progress = carrier_progress;
                    if synthesis_returned {
                        state.synthesis.advance_observed(|_| {})?
                    } else if let Some(candidate) = candidate {
                        let p = candidate.divisor;
                        if mf::cmp(&p, &mf::one()) != core::cmp::Ordering::Greater
                            || mf::cmp(&p, &state.source) != core::cmp::Ordering::Less
                        {
                            return Err(String::from("GFE source carrier returned an improper divisor"));
                        }
                        let (q, remainder) = mf::divmod(&state.source, &p);
                        if !mf::zero(&remainder) || mf::mul(&p, &q) != state.source {
                            return Err(String::from("GFE source carrier failed exact product closure"));
                        }
                        Some(state.synthesis.close_candidate(p, q, candidate.producer)?)
                    } else {
                        None
                    }
                }
            } else {
                None
            };
            let torus_return = match (torus_return, source_carrier_return) {
                (Some(candidate), _) => Some(candidate),
                (None, Some(candidate)) => Some(candidate),
                (None, None) => None,
            };
            // The torus first solves its source-centered target by a direct
            // 2-adic jump. The prepared product resident then continues under
            // this same outer GFE bind; no torus period walk gates its return.
            let returned = match torus_return {
                Some(candidate) => Some(candidate),
                None => state.synthesis.advance_observed(&mut observe)?,
            };
            if let Some(candidate) = returned {
                let mut readout = score_candidate_with_braid(
                    &state.source, &candidate.p, &candidate.q, candidate.producer,
                    candidate.braid_dots, state.source_log,
                )?;
                readout.source_ring = state.source_ring;
                if candidate.circuit_braid != state.circuit_braid {
                    return Err(String::from("GFE circuit ancestry differs from its producing resident"));
                }
                readout.circuit_braid = Some(candidate.circuit_braid.clone());
                readout.gfe_powerset_monad_laws_verified = candidate.powerset_monad_laws_verified;
                readout.torus_powerset_monad_laws_verified = state.torus_powerset_monad_laws_verified;
                readout.torus_full_orbit_verified = state.torus_full_orbit_verified;
                readout.catalyst_order_applied = candidate.catalyst_order_applied;
                readout.catalyst_inverse_returned = candidate.catalyst_inverse_returned;
                if let Some(trace) = state.torus_candidate_trace.as_ref() {
                    let (expected_p_log, expected_q_log, expected_p_ring, expected_q_ring) =
                        if mf::cmp(&candidate.p, &candidate.q) != core::cmp::Ordering::Greater {
                            (trace.p_log, trace.q_log, trace.p_ring, trace.q_ring)
                        } else {
                            (trace.q_log, trace.p_log, trace.q_ring, trace.p_ring)
                        };
                    if readout.log2_p != expected_p_log || readout.log2_q != expected_q_log
                        || crate::pwoat_geometry::number_line_parts(readout.log2_p) != expected_p_ring
                        || crate::pwoat_geometry::number_line_parts(readout.log2_q) != expected_q_ring
                    {
                        return Err(String::from("terminal GFE fusion differs from retained torus branch coordinates"));
                    }
                    readout.torus_branch_id = Some(trace.branch_id);
                    readout.torus_branch_position = Some(trace.position.clone());
                    readout.torus_bit_braid = Some(trace.bit_braid.clone());
                    readout.torus_singularity_crossing = Some(trace.singularity_crossing.clone());
                    readout.torus_branch_snapshot = state.torus.branch_snapshot(trace.branch_id);
                } else if let Some(snapshot) = state.torus.active_branch_snapshot() {
                    readout.torus_branch_id = Some(snapshot.branch_id);
                    readout.torus_branch_position = Some(snapshot.position.clone());
                    readout.torus_bit_braid = Some(snapshot.bit_braid.clone());
                    readout.torus_singularity_crossing = Some(snapshot.singularity_crossing.clone());
                    readout.torus_branch_snapshot = Some(snapshot);
                }
                readout.morphism_applications = candidate.morphism_applications;
                state.synthesis.latch_terminal_readout(&mut readout)?;
                if state.terminal_readout.replace(readout).is_some() {
                    return Err(String::from("GFE terminal readout latch was already fixed"));
                }
            }
            Ok::<_, String>(crate::reentry_powerset::unit(slot))
        })?;
        let mut terminal = None;
        for slot in self.active.iter() {
            let state = self.arena.get_mut(*slot)
                .ok_or_else(|| String::from("GFE terminal bind lost its resident"))?;
            if state.terminal_readout.is_some() {
                if terminal.is_some() {
                    return Err(String::from("GFE bind produced multiple terminal readouts"));
                }
                terminal = state.terminal_readout.clone();
            }
        }
        Ok(terminal)
    }
}

fn prepare_gfe_resident(
    source: Vec<char>,
    source_number: num_bigint::BigUint,
    small_prime_orbit_mask: SmallPrimeOrbitMask,
) -> Result<GfeResidentState, String> {
    let source = mf::trim(source);
    if source.is_empty() || source.iter().any(|mark| !matches!(mark, '⊤' | '⊥')) {
        return Err(String::from("GFE source must be a nonempty canonical bit tape"));
    }
    let recovered = num_bigint::BigUint::new(
        mf::tape_to_limbs(&source, mf::limb_count_for_bits(source.len())),
    );
    if recovered != source_number {
        return Err(String::from("GFE N-only tape differs from its source numeral"));
    }
    let source_log = log2_coordinate(&source);
    let source_ring = crate::pwoat_geometry::number_line_parts(source_log);
    let torus = crate::pwoat_torus::PwoatTorusOrbit::new(
        &source_number,
        source.len().div_ceil(2),
        phase_seeded_torus_multiplier(&source),
    )?;
    let torus_powerset_monad_laws_verified = torus.powerset_monad_laws_verified();
    let torus_full_orbit_verified = torus.full_orbit_period_verified();
    let torus_axes = torus.bit_braid().axis_permutation();
    let synthesis = factor_synthesis::FactorSynthesis::new_with_torus_braid(
        &source, Some(torus_axes), small_prime_orbit_mask.clone(),
    )?;
    let circuit_braid = synthesis.circuit_braid_state().clone();
    let prime_braids = PrimeBraidProbe::new(&source);
    Ok(GfeResidentState {
        source, source_log, source_ring, circuit_braid,
        torus_powerset_monad_laws_verified,
        torus_full_orbit_verified,
        torus_source_seek_done: false,
        horn_shiab_done: false,
        source_carrier_started: false,
        source_carrier_progress: None,
        prime_braids,
        torus_candidate_trace: None,
        small_prime_orbit_mask,
        torus,
        synthesis,
        terminal_readout: None,
    })
}

pub fn prepare_source_circuit_braid(
    source: &[char], torus_axes: [usize; 12],
) -> Result<crate::gfe_circuit_braid::GfeCircuitBraidState, String> {
    crate::gfe_circuit_braid::GfeCircuitBraidState::prepare(
        source_phase_bin(source), log2_coordinate(source), torus_axes,
    )
}

fn phase_bin_parts(log: crate::pwoat_geometry::Log2Coordinate) -> i64 {
    let exponent_remainder = (log.integer_exponent % LOG_PERIOD as usize) as f64;
    (((exponent_remainder + log.fractional_log2) * GABOR_BINS as f64
        / LOG_PERIOD as f64 + 0.5) as i64)
        .rem_euclid(GABOR_BINS)
}

fn phase_bin(log2_value: f64) -> i64 {
    ((log2_value * GABOR_BINS as f64 / LOG_PERIOD as f64 + 0.5) as i64)
        .rem_euclid(GABOR_BINS)
}

/// Source log-phase coordinate shared with the G-mOMonadOS toroidal kernel.
pub fn source_phase_bin(source: &[char]) -> i64 {
    phase_bin_parts(log2_coordinate(&mf::trim(source.to_vec())))
}

/// Choose a maximal-order 2-adic torus winding step from the source phase.
/// Every returned multiplier is 9 modulo 16, preserving order 2^(m-3).
pub fn phase_seeded_torus_multiplier(source: &[char]) -> u32 {
    9 + 16 * source_phase_bin(source).rem_euclid(GABOR_BINS) as u32
}

/// Golden-number coordinate of the source on the additive logarithmic line.
/// The torus sweep projects to the cardioid already parameterized by
/// `HornTorus::winding_point`: x=r(1+cos t)cos t, y=r(1+cos t)sin t.
/// This alternate phase changes only the orbit ordering, not its 2-adic order.
pub fn golden_source_phase_bin(source: &[char]) -> i64 {
    let log = log2_coordinate(&mf::trim(source.to_vec()));
    phase_bin(log.fused_log2() / LOG2_GOLDEN)
}

/// Select the same maximal-order 2-adic orbit using the golden-log line phase.
pub fn golden_phase_seeded_torus_multiplier(source: &[char]) -> u32 {
    9 + 16 * golden_source_phase_bin(source).rem_euclid(GABOR_BINS) as u32
}

/// Order source-derived factor-width lanes by balance, then by the `pwoat.html`
/// k=1 trine residual on its seven-unit number-line ring. Widths describe
/// possible intervals, not supplied factor widths. The complete geometry set
/// is retained; the torus coordinate changes traversal order only.
pub fn rank_width_geometries(
    source: &[char],
    geometries: &mut [(usize, usize)],
) {
    rank_width_geometry_order(source, geometries, None);
}

/// Stable lane ordering by source geometry, with small-prime orbit cells
/// moved behind phase/depth/trine cells outside the mask. No width is removed.
pub fn rank_width_geometries_with_mask(
    source: &[char],
    geometries: &mut [(usize, usize)],
    mask: &SmallPrimeOrbitMask,
) {
    rank_width_geometry_order(source, geometries, Some(mask));
}

fn rank_width_geometry_order(
    source: &[char],
    geometries: &mut [(usize, usize)],
    mask: Option<&SmallPrimeOrbitMask>,
) {
    let target_log = log2_coordinate(&mf::trim(source.to_vec())).fused_log2();
    geometries.sort_by(|&(left_p, left_q), &(right_p, right_q)| {
        let lane_geometry = |(p_width, q_width): (usize, usize)| {
            let p_center = p_width as f64 - 0.5;
            let q_center = q_width as f64 - 0.5;
            let p_phase = phase_bin(p_center) as usize;
            let q_phase = phase_bin(q_center) as usize;
            let p_trine = (p_center.floor() as usize) % SMALL_PRIME_TRINE_CELLS;
            let q_trine = (q_center.floor() as usize) % SMALL_PRIME_TRINE_CELLS;
            let small_prime_hits = mask.map_or(0, |mask|
                usize::from(mask.covers_phase_section(p_phase, p_trine))
                    + usize::from(mask.covers_phase_section(q_phase, q_trine)));
            let trine = crate::pwoat_geometry::semiprime_trine(
                p_center, q_center, target_log,
            );
            let residual_turn = trine.residual_turn;
            let scaled = residual_turn * GABOR_BINS as f64 / WIDTH_PHASE_SIGMA_BINS;
            (q_width.abs_diff(p_width),
                trine.wrap_balance.unsigned_abs(),
                libm::exp(-0.5 * scaled * scaled),
                small_prime_hits)
        };
        let (left_balance, left_wrap, left_score, left_hits) = lane_geometry((left_p, left_q));
        let (right_balance, right_wrap, right_score, right_hits) = lane_geometry((right_p, right_q));
        left_balance.cmp(&right_balance)
            .then_with(|| left_wrap.cmp(&right_wrap))
            .then_with(|| right_score.partial_cmp(&left_score)
                .unwrap_or(core::cmp::Ordering::Equal))
            .then_with(|| left_hits.cmp(&right_hits))
            .then_with(|| left_p.cmp(&right_p))
    });
}

fn score_candidate_with_braid(
    source: &[char],
    p: &[char],
    q: &[char],
    producer: &'static str,
    braid_dots: Vec<factor_synthesis::FactorBraidDot>,
    source_log: crate::pwoat_geometry::Log2Coordinate,
) -> Result<GeometerFrameReadout, String> {
    let source = mf::trim(source.to_vec());
    let p = mf::trim(p.to_vec());
    let q = mf::trim(q.to_vec());
    if mf::cmp(&p, &mf::one()) != core::cmp::Ordering::Greater
        || mf::cmp(&q, &mf::one()) != core::cmp::Ordering::Greater
        || mf::mul(&p, &q) != source
    {
        return Err(String::from("GFE candidate did not close the exact source product"));
    }

    let lp = log2_coordinate(&p);
    let lq = log2_coordinate(&q);
    let ln = source_log;
    let bin_p = phase_bin_parts(lp);
    let bin_q = phase_bin_parts(lq);
    let bin_n = phase_bin_parts(ln);
    let residual = centered_residue(bin_p + bin_q - bin_n, GABOR_BINS);
    let winding_p = winding(lp.fused_log2());
    let winding_q = winding(lq.fused_log2());
    let winding_n = winding(ln.fused_log2());
    let trine = crate::pwoat_geometry::semiprime_trine_parts(lp, lq, ln);

    let sigma_bins = 1.0f64;
    let scaled = residual as f64 / sigma_bins;
    let gabor_score = libm::exp(-0.5 * scaled * scaled);

    let omega_p = omega_residue(&p)?;
    let omega_q = omega_residue(&q)?;
    let omega_n = (omega_p + omega_q) % SIC_DIMENSION as u8;
    let product_coordinate = (omega_p + omega_q) % SIC_DIMENSION as u8;
    // Bind the exact d=12 SIC depth label to the carried log-phase coordinate.
    // The SIC outcome is the depth residue crossed with the 12-cell phase
    // section; the 2048-cell Gabor score remains its continuous-direction
    // companion in the tensor-frame readout.
    let candidate_sic_label = product_coordinate as usize * SIC_DIMENSION
        + (bin_p + bin_q).rem_euclid(SIC_DIMENSION as i64) as usize;
    let source_sic_label = omega_n as usize * SIC_DIMENSION
        + bin_n.rem_euclid(SIC_DIMENSION as i64) as usize;
    let sic_dual_score = sic_dual_overlap(candidate_sic_label, source_sic_label);
    let expected_braid = factor_synthesis::build_factor_braid_dots(&source, &p, &q)?;
    if braid_dots != expected_braid {
        return Err(String::from("GFE readout braid differs from the resident factor return"));
    }

    Ok(GeometerFrameReadout {
        p,
        q,
        producer,
        phase_bin_p: bin_p,
        phase_bin_q: bin_q,
        phase_bin_n: bin_n,
        phase_residual_bins: residual,
        log2_p: lp,
        log2_q: lq,
        log2_n: ln,
        fused_log2_p: lp.fused_log2(),
        fused_log2_q: lq.fused_log2(),
        fused_log2_n: ln.fused_log2(),
        source_ring: crate::pwoat_geometry::number_line_parts(ln),
        numberline_turn_p: trine.p.phi_turn,
        numberline_turn_q: trine.q.phi_turn,
        numberline_turn_n: trine.n.phi_turn,
        numberline_wrap_p: trine.p.phi_wrap,
        numberline_wrap_q: trine.q.phi_wrap,
        numberline_wrap_n: trine.n.phi_wrap,
        numberline_turn_residual: trine.residual_turn,
        numberline_wrap_balance: trine.wrap_balance,
        winding_p,
        winding_q,
        winding_n,
        omega_p,
        omega_q,
        omega_n,
        sic_candidate_label: candidate_sic_label,
        sic_source_label: source_sic_label,
        gabor_score,
        sic_dual_score,
        combined_score: gabor_score * sic_dual_score,
        product_closed: true,
        gfe_powerset_monad_laws_verified: false,
        torus_powerset_monad_laws_verified: false,
        torus_full_orbit_verified: false,
        catalyst_order_applied: false,
        catalyst_inverse_returned: false,
        final_readout_fixed: false,
        torus_branch_id: None,
        torus_branch_position: None,
        torus_bit_braid: None,
        torus_singularity_crossing: None,
        torus_branch_snapshot: None,
        circuit_braid: None,
        morphism_applications: Vec::new(),
        braid_dots,
    })
}

fn log2_coordinate(value: &[char]) -> crate::pwoat_geometry::Log2Coordinate {
    let bits = value.len();
    let take = bits.min(53);
    let mut leading = 0u64;
    for offset in 0..take {
        if value[bits - 1 - offset] == '⊥' {
            leading |= 1u64 << (take - 1 - offset);
        }
    }
    let top = leading as f64 / (1u64 << (take - 1)) as f64;
    crate::pwoat_geometry::Log2Coordinate {
        integer_exponent: bits - 1,
        leading_mantissa: leading,
        mantissa_bits: take as u8,
        fractional_log2: libm::log2(top),
    }
}

fn winding(log2_value: f64) -> i64 {
    (log2_value / LOG_PERIOD as f64) as i64
}

fn centered_residue(value: i64, modulus: i64) -> i64 {
    let residue = value.rem_euclid(modulus);
    if residue > modulus / 2 { residue - modulus } else { residue }
}

fn omega_residue(value: &[char]) -> Result<u8, String> {
    let mut pending = vec![mf::trim(value.to_vec())];
    let mut count = 0u8;
    while let Some(current) = pending.pop() {
        if current == mf::one() {
            continue;
        }
        if mf::miller_rabin(&current) {
            count = (count + 1) % SIC_DIMENSION as u8;
            continue;
        }
        let (split, _) = mf::scout_factor(&current);
        let Some((left, right, _)) = split else {
            return Err(String::from("GFE depth coordinate could not split a composite candidate"));
        };
        pending.push(left);
        pending.push(right);
    }
    Ok(count)
}

fn sic_dual_overlap(candidate_label: usize, source_label: usize) -> f64 {
    // For a d=12 SIC, measuring one SIC ray gives 1/12 on its matching
    // outcome and 1/156 on each of the other 143 outcomes. Store those POVM
    // masses over the common denominator 156, then apply the exact dual Gram
    // inverse (13/12)I - (1/144)J using integer numerators. The recovered
    // coordinate has numerator 156 at the ray label and zero elsewhere,
    // over denominator 1872. This is the analytic d=12 frame operation;
    // no fabricated floating mass vector or post-hoc depth-only overlap is
    // involved.
    fn dual_coordinates(label: usize) -> [u16; SIC_OUTCOMES] {
        let mut mass_numerators = [1u16; SIC_OUTCOMES];
        mass_numerators[label % SIC_OUTCOMES] = 13;
        let total: u16 = mass_numerators.iter().copied().sum();
        core::array::from_fn(|index| {
            // Common denominator 1872 = 12 * 156.
            (13u16 * mass_numerators[index] - total / 12) as u16
        })
    }
    let candidate = dual_coordinates(candidate_label);
    let source = dual_coordinates(source_label);
    let numerator: u64 = candidate.iter().zip(source.iter())
        .map(|(left, right)| u64::from(*left) * u64::from(*right))
        .sum();
    numerator as f64 / (1872.0 * 1872.0)
}
