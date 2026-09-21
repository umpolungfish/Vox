//! Exact execution of the repaired fixed-point spectral construction.
//!
//! This module is deliberately narrow.  It executes the repaired IMASM word
//!
//! `⊢≻∈⊤≺⊥∋⋈≻⊙⊞⊡≻⋈∈⊤⊥∋⊡≻⋈⊙⊞⊡⊣`
//!
//! on top of the N-dependent modular operator already owned by
//! `FixedPointSpectralConstruction`.  The `AREV` between the first T and F
//! evaluations is essential: T is banked by the open frame, the live register
//! is cleared, F is deposited, and `FFUSE` restores exactly the banked T.
//! The resulting weight is A = {T,F,t,f}.  The three `IFIX` sites permanently
//! record winding indices 2, 3, and 4 together with their exact modular phase.
//!
//! No residue-domain scan, factor loop, continued-fraction lift, or modular
//! orbit walk occurs here.  If one of those three fixed windings already closes
//! the resident modular phase, it can be handed to the existing Hadamard
//! descent.  Otherwise the read is N: this protocol does not invent a period
//! that its fixed points did not expose.

use alloc::vec;
use alloc::vec::Vec;

use crate::hadamard_factor_bridge::HadamardDescent;
use crate::hadamard_gate::{FixedPointSpectralConstruction, SpectralWinding, Tape};
use crate::morphism_factor::{add, cmp, one};
use crate::vox::{
    verdict, AFWD, AREV, CLINK, ENGAGR, EVALF, EVALT, FFUSE, FSPLIT, IFIX, IMSCRIB,
    TANCH, VINIT,
};

/// The repaired fixed-point spectral word, exactly as specified.
pub const FIXED_POINT_SPECTRAL_WORD: [char; 25] = [
    VINIT, AFWD, FSPLIT, EVALT, AREV, EVALF, FFUSE, CLINK, AFWD, IMSCRIB, ENGAGR,
    IFIX, AFWD, CLINK, FSPLIT, EVALT, EVALF, FFUSE, IFIX, AFWD, CLINK, IMSCRIB,
    ENGAGR, IFIX, TANCH,
];

const W_T: u8 = 0b0001;
const W_F: u8 = 0b0010;
const W_t: u8 = 0b0100;
const W_f: u8 = 0b1000;
const W_A: u8 = W_T | W_F | W_t | W_f;

#[derive(Clone, PartialEq, Debug)]
pub struct FixedWindingDeposit {
    /// Integer winding fixed by this IFIX, retained as a tape.
    pub winding: Tape,
    /// Exact resident modular phase a^w mod N at the fixation boundary.
    pub modular_phase: Tape,
    /// True exactly when this fixed winding closes the resident modular phase.
    pub closes_modular_phase: bool,
}

#[derive(Clone, PartialEq, Debug)]
pub struct FixedPointBoundaryMeasurement {
    pub word: Vec<char>,
    /// SIXTEEN_3 weight register rendered as A for {T,F,t,f}.
    pub final_register: char,
    /// Surviving weights in the order [T, F, t, f].
    pub surviving: [u8; 4],
    pub deposits: usize,
    pub cleared: usize,
    pub restored: usize,
    pub seeded: usize,
    pub inert: usize,
    pub banked_ok: bool,
    pub mu_delta_identity: bool,
    pub tri_ancestral_verdict: char,
    /// Spectral audit only: verdict/pair balance/IFIX count are invariant over
    /// all cyclic readings.  The final register itself remains phase-bearing.
    pub rotat_invariant: bool,
    pub phase_bearing: bool,
    pub fixed: Vec<FixedWindingDeposit>,
}

impl FixedPointBoundaryMeasurement {
    pub fn is_a(&self) -> bool {
        self.final_register == 'A' && self.surviving == [1, 1, 1, 1]
    }

    /// The first winding explicitly fixed by the repaired word that closes the
    /// resident modular phase.  This is a selection over three IFIX deposits,
    /// not a walk over successive modular powers.
    pub fn first_closing_winding(&self) -> Option<&[char]> {
        self.fixed
            .iter()
            .find(|deposit| deposit.closes_modular_phase)
            .map(|deposit| deposit.winding.as_slice())
    }
}

#[derive(Clone, Copy)]
struct Frame {
    /// Weight protected by this frame.  Deposits made while the frame is open
    /// are banked here as well as being visible in the live register.
    bank: u8,
}

fn bit_count(mask: u8) -> usize {
    mask.count_ones() as usize
}

fn surviving(mask: u8) -> [u8; 4] {
    [
        u8::from(mask & W_T != 0),
        u8::from(mask & W_F != 0),
        u8::from(mask & W_t != 0),
        u8::from(mask & W_f != 0),
    ]
}

fn deposit(live: &mut u8, frames: &mut [Frame], bits: u8) -> bool {
    let new_bits = bits & !*live;
    if new_bits == 0 {
        return false;
    }
    *live |= new_bits;
    if let Some(frame) = frames.last_mut() {
        frame.bank |= new_bits;
    }
    true
}

fn rotate(word: &[char], by: usize) -> Vec<char> {
    let n = word.len();
    (0..n).map(|i| word[(i + by) % n]).collect()
}

fn rotat_spectral_audit() -> bool {
    let base_verdict = verdict(&FIXED_POINT_SPECTRAL_WORD);
    let base_splits = FIXED_POINT_SPECTRAL_WORD
        .iter()
        .filter(|&&op| op == FSPLIT)
        .count();
    let base_fuses = FIXED_POINT_SPECTRAL_WORD
        .iter()
        .filter(|&&op| op == FFUSE)
        .count();
    let base_fixes = FIXED_POINT_SPECTRAL_WORD
        .iter()
        .filter(|&&op| op == IFIX)
        .count();

    (0..FIXED_POINT_SPECTRAL_WORD.len()).all(|k| {
        let r = rotate(&FIXED_POINT_SPECTRAL_WORD, k);
        verdict(&r) == base_verdict
            && r.iter().filter(|&&op| op == FSPLIT).count() == base_splits
            && r.iter().filter(|&&op| op == FFUSE).count() == base_fuses
            && r.iter().filter(|&&op| op == IFIX).count() == base_fixes
    })
}

impl FixedPointSpectralConstruction {
    /// Execute the repaired 25-op boundary exactly once.
    pub fn run_boundary_measurement(
        &self,
    ) -> Result<FixedPointBoundaryMeasurement, &'static str> {
        let mut live = 0u8;
        let mut frames: Vec<Frame> = Vec::new();
        let mut winding = vec![EVALT]; // integer zero, LSB-first numeral tape
        let mut fixed = Vec::new();

        let mut deposits = 0usize;
        let mut cleared = 0usize;
        let mut restored = 0usize;
        let mut seeded = 0usize;
        let mut inert = 0usize;
        let mut seed_present = false;
        let mut banked_ok = true;

        for &op in &FIXED_POINT_SPECTRAL_WORD {
            let mut changed = false;
            match op {
                VINIT => {
                    // CLEAR at the ground.  In the repaired word this loses 0.
                    let lost = bit_count(live);
                    if lost != 0 {
                        cleared += lost;
                        live = 0;
                        changed = true;
                    }
                }
                AFWD => {
                    winding = add(&winding, &one());
                    changed = true;
                    if !seed_present {
                        // The first lift seeds logical T but contributes no
                        // topological weight.  EVALT supplies that weight later.
                        seed_present = true;
                        seeded += 1;
                    }
                }
                FSPLIT => {
                    frames.push(Frame { bank: live });
                    changed = true;
                }
                EVALT => {
                    if deposit(&mut live, frames.as_mut_slice(), W_T) {
                        deposits += 1;
                        changed = true;
                    }
                }
                AREV => {
                    let lost_mask = live;
                    let lost = bit_count(lost_mask);
                    if lost != 0 {
                        let protected = frames.iter().fold(0u8, |acc, frame| acc | frame.bank);
                        if lost_mask & !protected != 0 {
                            banked_ok = false;
                        }
                        cleared += lost;
                        live = 0;
                        changed = true;
                    }
                }
                EVALF => {
                    if deposit(&mut live, frames.as_mut_slice(), W_F) {
                        deposits += 1;
                        changed = true;
                    }
                }
                FFUSE => {
                    let frame = frames.pop().ok_or("fixed-point FFUSE has no open frame")?;
                    let missing = frame.bank & !live;
                    restored += bit_count(missing);
                    live |= frame.bank;
                    changed = true;
                }
                CLINK | IMSCRIB => {}
                ENGAGR => {
                    // One paradice engagement deposits the lower-case pair as
                    // one event.  At the second ENGAGR A is already saturated.
                    if deposit(&mut live, frames.as_mut_slice(), W_t | W_f) {
                        deposits += 1;
                        changed = true;
                    }
                }
                IFIX => {
                    let modular_phase = self.modular_branch(&winding)?;
                    let closes_modular_phase = cmp(&modular_phase, &one()) == core::cmp::Ordering::Equal;
                    fixed.push(FixedWindingDeposit {
                        winding: winding.clone(),
                        modular_phase,
                        closes_modular_phase,
                    });
                    changed = true;
                }
                TANCH => {}
                _ => return Err("unexpected opcode in fixed-point spectral word"),
            }

            if !changed {
                inert += 1;
            }
        }

        if !frames.is_empty() {
            return Err("fixed-point spectral word leaves a frame open");
        }
        if live != W_A {
            return Err("fixed-point spectral word did not close at A");
        }
        if fixed.len() != 3 {
            return Err("fixed-point spectral word did not make three IFIX deposits");
        }

        // Re-use the exact spectral split/fuse implementation already carried
        // by the Hadamard layer.  This makes μ∘δ=id executable here too.
        let identity_probe = SpectralWinding::new(&winding, &one())?;
        let mu_delta_identity = identity_probe.delta().mu()? == identity_probe;

        Ok(FixedPointBoundaryMeasurement {
            word: FIXED_POINT_SPECTRAL_WORD.to_vec(),
            final_register: 'A',
            surviving: surviving(live),
            deposits,
            cleared,
            restored,
            seeded,
            inert,
            banked_ok,
            mu_delta_identity,
            tri_ancestral_verdict: verdict(&FIXED_POINT_SPECTRAL_WORD),
            rotat_invariant: rotat_spectral_audit(),
            phase_bearing: true,
            fixed,
        })
    }

    /// Select a useful fixed winding directly from the repaired boundary.
    /// Returns None when none of its three IFIX deposits closes the resident
    /// modular phase.  No order walk or fallback search is performed.
    pub fn instant_non_walking_read(&self) -> Result<Option<Tape>, &'static str> {
        let measurement = self.run_boundary_measurement()?;
        Ok(measurement.first_closing_winding().map(|w| w.to_vec()))
    }

    /// Literal long-form entry point for the same read.  A returned tape is an
    /// exact multiplicative order because it is the least closing exponent among
    /// the repaired protocol's ordered IFIX deposits.  None means this fixed
    /// protocol did not expose the order; it does not trigger an orbit walk.
    pub fn an_instant_non_walking_read_of_the_multiplicative_order(
        &self,
    ) -> Result<Option<Tape>, &'static str> {
        self.instant_non_walking_read()
    }

    /// Complete the requested architecture in one call when the repaired fixed
    /// boundary exposes a usable period:
    /// N -> fixed spectrum -> fixed winding -> existing Hadamard descent.
    /// An unexposed period routes around as N; there is no hidden fallback.
    pub fn descend_boundary_measurement(self) -> HadamardDescent {
        let base = self.base().to_vec();
        let period = match self.instant_non_walking_read() {
            Ok(period) => period,
            Err(_) => return HadamardDescent::F,
        };
        let carrier = self.into_carrier();
        match period {
            Some(period) => carrier.descend_phase_order(&base, &period),
            None => HadamardDescent::N(carrier),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::factor_extract::extract;
    use crate::hadamard_gate::HadamardCarrier;
    use crate::morphism_factor::tape_u64;

    #[test]
    fn repaired_word_is_exact() {
        let expected: Vec<char> = "⊢≻∈⊤≺⊥∋⋈≻⊙⊞⊡≻⋈∈⊤⊥∋⊡≻⋈⊙⊞⊡⊣".chars().collect();
        assert_eq!(FIXED_POINT_SPECTRAL_WORD.as_slice(), expected.as_slice());
    }

    #[test]
    fn repaired_weight_audit_matches_the_specification() {
        let spectral = HadamardCarrier::new(&tape_u64(15))
            .unwrap()
            .fixed_point_spectral_construction()
            .unwrap();
        let read = spectral.run_boundary_measurement().unwrap();

        assert!(read.is_a());
        assert_eq!(read.deposits, 3);
        assert_eq!(read.cleared, 1);
        assert_eq!(read.restored, 1);
        assert_eq!(read.seeded, 1);
        assert_eq!(read.inert, 10);
        assert!(read.banked_ok);
        assert!(read.mu_delta_identity);
        assert_eq!(read.tri_ancestral_verdict, 'T');
        assert!(read.rotat_invariant);
        assert!(read.phase_bearing);
    }

    #[test]
    fn ifix_sites_are_exactly_windings_two_three_four() {
        let spectral = HadamardCarrier::new(&tape_u64(15))
            .unwrap()
            .fixed_point_spectral_construction()
            .unwrap();
        let read = spectral.run_boundary_measurement().unwrap();
        let fixed: Vec<Tape> = read.fixed.iter().map(|d| d.winding.clone()).collect();
        assert_eq!(fixed, vec![tape_u64(2), tape_u64(3), tape_u64(4)]);
        assert_eq!(read.fixed[0].modular_phase, tape_u64(4));
        assert_eq!(read.fixed[1].modular_phase, tape_u64(8));
        assert_eq!(read.fixed[2].modular_phase, tape_u64(1));
    }

    #[test]
    fn non_walking_read_selects_the_fixed_closing_winding() {
        let spectral = HadamardCarrier::new(&tape_u64(15))
            .unwrap()
            .fixed_point_spectral_construction()
            .unwrap();
        assert_eq!(spectral.instant_non_walking_read().unwrap(), Some(tape_u64(4)));
    }

    #[test]
    fn repaired_boundary_feeds_the_existing_hadamard_descent() {
        let spectral = HadamardCarrier::new(&tape_u64(15))
            .unwrap()
            .fixed_point_spectral_construction()
            .unwrap();
        let carrier = match spectral.descend_boundary_measurement() {
            HadamardDescent::T(carrier) => carrier,
            other => panic!("expected T factor carrier, got {other:?}"),
        };
        let out = extract(&carrier).unwrap();
        assert_eq!((out.p.0, out.q.0), (tape_u64(3), tape_u64(5)));
    }

    #[test]
    fn no_hidden_order_walk_when_the_three_fixed_windings_do_not_close() {
        // ord_21(2)=6, which the exact repaired word does not IFIX.  The read
        // must therefore return None/N rather than silently walking to six.
        let spectral = HadamardCarrier::new(&tape_u64(21))
            .unwrap()
            .fixed_point_spectral_construction()
            .unwrap();
        assert_eq!(spectral.instant_non_walking_read().unwrap(), None);
        assert!(matches!(spectral.descend_boundary_measurement(), HadamardDescent::N(_)));
    }
}
