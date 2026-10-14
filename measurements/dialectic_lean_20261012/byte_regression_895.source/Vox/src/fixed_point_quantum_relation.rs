//! Resident N-dependent relation for the collapsed quantum landing.
//!
//! The modulus and base are not metadata attached to a generic membrane.  Their
//! IMASM numeral cells are inside the ancestry region that performs the one-shot
//! pair-before-advance readout.  The complete resident object is therefore one
//! glyph word in the common language before fixation.

use alloc::vec::Vec;

use crate::fixed_point_quantum_phase::PhaseLandingProgram;
use crate::vox::{
    pairing, verdict, AFWD, AREV, CLINK, EVALF, EVALT, FFUSE, FSPLIT, IFIX,
    IMSCRIB, TANCH, VINIT,
};

fn numeral(tape: &[char]) -> bool {
    !tape.is_empty() && tape.iter().all(|&mark| mark == EVALT || mark == EVALF)
}

/// Materialize the complete resident landing relation as one properly nested
/// IMASM word.
///
/// Layout:
///
///   ⊢ ∈ N ∈ a ⊙ ∈ ⊤ ≺ ⊥ ⋈ ≻ ∋ ∋ ∋ ⊡ ⊣
///
/// The three frames are ancestry containment, not three adjacent programs:
/// - outer frame owns the modulus N and the complete modular relation;
/// - middle frame owns the base a and the readout action;
/// - inner frame banks the unique live clear and forms the pair before advance.
///
/// There is one VINIT, one TANCH and one terminal IFIX after every fuse.  N and
/// a remain native LSB-first EVALT/EVALF tapes; no host integer is reconstructed.
pub(crate) fn resident_landing_word(
    landing: &PhaseLandingProgram,
) -> Result<Vec<char>, &'static str> {
    let n = landing.n();
    let base = landing.base();
    if !numeral(n) || !numeral(base) {
        return Err("resident quantum landing contains a malformed IMASM numeral");
    }

    let mut word = Vec::with_capacity(n.len() + base.len() + 15);
    word.push(VINIT);
    word.push(FSPLIT);
    word.extend_from_slice(n);
    word.push(FSPLIT);
    word.extend_from_slice(base);
    word.extend_from_slice(&[
        IMSCRIB,
        FSPLIT,
        EVALT,
        AREV,
        EVALF,
        CLINK,
        AFWD,
        FFUSE,
        FFUSE,
        FFUSE,
        IFIX,
        TANCH,
    ]);

    if word.iter().filter(|&&mark| mark == VINIT).count() != 1
        || word.iter().filter(|&&mark| mark == TANCH).count() != 1
        || word.iter().filter(|&&mark| mark == IFIX).count() != 1
        || word.iter().filter(|&&mark| mark == AREV).count() != 1
        || word.iter().filter(|&&mark| mark == FSPLIT).count() != 3
        || word.iter().filter(|&&mark| mark == FFUSE).count() != 3
    {
        return Err("resident quantum landing lost its single-carrier topology");
    }

    let clear = word
        .iter()
        .position(|&mark| mark == AREV)
        .ok_or("resident quantum landing lost its live clear")?;
    let link = word
        .iter()
        .position(|&mark| mark == CLINK)
        .ok_or("resident quantum landing lost its spectral pair")?;
    let advance = word
        .iter()
        .position(|&mark| mark == AFWD)
        .ok_or("resident quantum landing lost its transport")?;
    let fixation = word
        .iter()
        .position(|&mark| mark == IFIX)
        .ok_or("resident quantum landing lost its fixation")?;
    if link >= advance {
        return Err("resident quantum landing advances before forming its pair");
    }

    let (regions, unanswered, unopened) = pairing(&word);
    if !unanswered.is_empty() || !unopened.is_empty() || regions.len() != 3 {
        return Err("resident quantum landing is not a complete ancestry nest");
    }
    if regions
        .iter()
        .any(|region| !(region.split < clear && clear < region.fuse))
    {
        return Err("resident quantum landing exposes its live clear");
    }
    if regions.iter().any(|region| fixation <= region.fuse) {
        return Err("resident quantum landing fixes before every frame has fused");
    }
    if verdict(&word) != 'T' {
        return Err("resident quantum landing does not close");
    }

    Ok(word)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixed_point_quantum_membrane::FixedPointQuantumMembrane;
    use crate::morphism_factor::tape_u64;

    fn landing(n: u64) -> PhaseLandingProgram {
        FixedPointQuantumMembrane::from_n(&tape_u64(n)).unwrap()
            .phase_estimation_register().unwrap()
            .into_measurement_program().unwrap()
            .into_landing_program().unwrap()
    }

    #[test]
    fn resident_relation_is_n_dependent_topology_not_side_metadata() {
        let a = landing(257);
        let b = landing(263);
        let aw = resident_landing_word(&a).unwrap();
        let bw = resident_landing_word(&b).unwrap();
        assert_ne!(aw, bw);
        assert!(aw.windows(a.n().len()).any(|window| window == a.n()));
        assert!(aw.windows(a.base().len()).any(|window| window == a.base()));
        assert_eq!(verdict(&aw), 'T');
    }

    #[test]
    fn resident_relation_banks_one_clear_through_all_three_frames() {
        let landing = landing(65_537);
        let word = resident_landing_word(&landing).unwrap();
        let clear = word.iter().position(|&mark| mark == AREV).unwrap();
        let (regions, unanswered, unopened) = pairing(&word);
        assert!(unanswered.is_empty());
        assert!(unopened.is_empty());
        assert_eq!(regions.len(), 3);
        assert!(regions.iter().all(|region| region.split < clear && clear < region.fuse));
        assert_eq!(word.iter().filter(|&&mark| mark == IFIX).count(), 1);
    }
}
