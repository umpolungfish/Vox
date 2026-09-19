//! One certified cycle from a descending imscription to the passive quotient fixed object.
//!
//! The dialectic certificate proves continuity of the operator-space descent.  Its
//! exact terminal factor object is then lifted with one compact scaffold record per
//! consumed imscription.  The passive re-entry certificate must begin at that exact
//! lifted carrier and delete exactly those scaffold records, returning byte-for-byte
//! to the dialectic terminal carrier.  The bridge is strict; no relaxed equivalence
//! is used to join the two proof layers.

use alloc::string::String;
use alloc::vec::Vec;

use crate::dialectic_certificate::{
    certify_dialectic, verify_dialectic_certificate, DialecticCertificate,
};
use crate::dialectic_reentry::{DialecticObject, IM_RWX};
use crate::factor_extract::{FactorCarrier, Mark};
use crate::provenance_envelope::LaneSupport;
use crate::reentry_certificate::{
    certify_reentry, verify_reentry_certificate, ReentryCertificate,
};
use crate::router_marks::{GStep, M_B, M_T};
use crate::trace_word::{decode_trace, encode_trace};
use crate::vox::{EVALF, EVALT};

const SUPPORT_BITS: usize = 6;
const RWX_BITS: usize = 3;

#[derive(Clone, PartialEq, Debug)]
pub struct ImscriptionCycleCertificate {
    pub dialectic: DialecticCertificate,
    /// Exact factor carrier at the proof-layer bridge.  It carries one compact
    /// scaffold record for every consumed whole imscription, followed by the
    /// dialectic terminal trace.
    pub lifted_carrier: Vec<Mark>,
    pub quotient: ReentryCertificate,
}

#[derive(Clone, PartialEq, Debug)]
pub struct ImscriptionCycleSummary {
    pub descents: usize,
    pub terminal_support: LaneSupport,
    pub quotient_generations: usize,
    pub quotient_transforms: usize,
    pub fixed_carrier: FactorCarrier,
}

fn push_mask(out: &mut Vec<Mark>, mask: u32, bits: usize) {
    for bit in 0..bits {
        out.push(if (mask >> bit) & 1 == 1 { EVALF } else { EVALT });
    }
}

/// Compact quotient-facing projection of one complete imscription object.
///
/// The full bulk, boundary and wire image remain in `DialecticCertificate`; the
/// factor-carrier trace only needs the distinction that this generation existed:
/// support, live r/w/x coupling, and the IMASM word that was executed.  Keeping
/// this payload bounded also avoids making trace payload width depend on numeral
/// width while the exact arbitrary-width object remains certified separately.
fn scaffold_payload(object: &DialecticObject) -> Vec<Mark> {
    let mut payload = Vec::with_capacity(2 + SUPPORT_BITS + RWX_BITS + object.word.len());
    payload.push('⊢');
    push_mask(&mut payload, object.support, SUPPORT_BITS);
    push_mask(&mut payload, object.imscription.rwx as u32, RWX_BITS);
    payload.extend_from_slice(&object.word);
    payload.push('⊣');
    payload
}

fn lift_dialectic_history(
    certificate: &DialecticCertificate,
    terminal: &FactorCarrier,
) -> Result<FactorCarrier, String> {
    let mut steps = Vec::new();
    for wire in &certificate.objects {
        let object = DialecticObject::decode(wire)?;
        if object.n != terminal.n {
            return Err(String::from("imscription history changed bulk N before quotient bridge"));
        }
        if object.imscription.rwx != IM_RWX {
            return Err(String::from("imscription history lost live r/w/x before quotient bridge"));
        }
        steps.push(GStep {
            repr: '⋈',
            judgment: M_B,
            recognised: M_T,
            next: '⋈',
            applied_word: scaffold_payload(&object),
        });
    }

    let terminal_steps = decode_trace(&terminal.trace)
        .ok_or_else(|| String::from("dialectic terminal trace is malformed at quotient bridge"))?;
    if terminal_steps.is_empty() {
        return Err(String::from("dialectic terminal trace is empty at quotient bridge"));
    }
    steps.extend(terminal_steps);

    FactorCarrier::new(
        terminal.n.clone(),
        terminal.p.clone(),
        terminal.q.clone(),
        encode_trace(&steps),
    )
}

/// Build the complete proof object.  The dialectic side is verified before its
/// terminal carrier is lifted into quotient-facing provenance.
pub fn certify_imscription_cycle(
    start: &DialecticObject,
) -> Result<ImscriptionCycleCertificate, String> {
    let dialectic = certify_dialectic(start)?;
    let dialectic_summary = verify_dialectic_certificate(&dialectic)?;
    let lifted = lift_dialectic_history(&dialectic, &dialectic_summary.terminal_carrier)?;
    let quotient = certify_reentry(&lifted)?;
    Ok(ImscriptionCycleCertificate {
        dialectic,
        lifted_carrier: lifted.encode(),
        quotient,
    })
}

/// Verify the full circuit:
///
/// `N -> whole-object imscription descent -> exact lifted factor carrier
///    -> passive quotient -> exact dialectic terminal factor object`.
///
/// Every dialectic descent contributes exactly one quotient scaffold record.
/// The fixed carrier must therefore be byte-for-byte the dialectic terminal
/// carrier, not merely relaxed-equivalent to it.
pub fn verify_imscription_cycle(
    certificate: &ImscriptionCycleCertificate,
) -> Result<ImscriptionCycleSummary, String> {
    let dialectic = verify_dialectic_certificate(&certificate.dialectic)?;
    let expected_lifted = lift_dialectic_history(
        &certificate.dialectic,
        &dialectic.terminal_carrier,
    )?;
    if expected_lifted.encode() != certificate.lifted_carrier {
        return Err(String::from("imscription cycle lifted-carrier bridge mismatch"));
    }

    let lifted = FactorCarrier::decode(&certificate.lifted_carrier)?;
    if lifted != expected_lifted {
        return Err(String::from("imscription cycle lifted carrier is not the certified projection"));
    }

    if certificate.quotient.n != lifted.n
        || certificate.quotient.p != lifted.p
        || certificate.quotient.q != lifted.q
    {
        return Err(String::from("imscription cycle quotient changed the exact factor witness at the bridge"));
    }
    let first = certificate.quotient.links.first()
        .ok_or_else(|| String::from("imscription cycle quotient certificate is empty"))?;
    if first.before != lifted.trace {
        return Err(String::from("imscription cycle quotient does not start at the exact lifted carrier"));
    }

    let quotient = verify_reentry_certificate(&certificate.quotient)?;
    if quotient.transforms != dialectic.descents {
        return Err(String::from("imscription cycle quotient did not erase exactly one scaffold per descent"));
    }
    if quotient.generations != dialectic.descents + 1 {
        return Err(String::from("imscription cycle quotient generation count does not close the descent history"));
    }
    if quotient.normal_form != dialectic.terminal_carrier.trace {
        return Err(String::from("imscription cycle quotient fixed trace is not the dialectic terminal trace"));
    }

    let fixed_carrier = FactorCarrier::new(
        certificate.quotient.n.clone(),
        certificate.quotient.p.clone(),
        certificate.quotient.q.clone(),
        quotient.normal_form.clone(),
    )?;
    if fixed_carrier.encode() != dialectic.terminal_carrier.encode() {
        return Err(String::from("imscription cycle did not return to the exact dialectic terminal factor object"));
    }

    Ok(ImscriptionCycleSummary {
        descents: dialectic.descents,
        terminal_support: dialectic.terminal_support,
        quotient_generations: quotient.generations,
        quotient_transforms: quotient.transforms,
        fixed_carrier,
    })
}
