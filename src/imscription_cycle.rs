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
    certify_dialectic, decode_dialectic_certificate, encode_dialectic_certificate,
    verify_dialectic_certificate, DialecticCertificate,
};
use crate::dialectic_reentry::{DialecticObject, IM_RWX};
use crate::factor_extract::{FactorCarrier, Mark, Tape};
use crate::provenance_envelope::LaneSupport;
use crate::reentry_certificate::{
    certify_reentry, decode_reentry_certificate, encode_reentry_certificate,
    verify_reentry_certificate, ReentryCertificate,
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

fn usize_to_tape(mut value: usize) -> Tape {
    if value == 0 {
        return vec![EVALT];
    }
    let mut out = Vec::new();
    while value != 0 {
        out.push(if value & 1 == 1 { EVALF } else { EVALT });
        value >>= 1;
    }
    out
}

fn tape_to_usize(tape: &[Mark]) -> Option<usize> {
    if tape.is_empty() {
        return None;
    }
    let mut value = 0usize;
    for (bit, &mark) in tape.iter().enumerate() {
        match mark {
            EVALT => {}
            EVALF => {
                if bit >= usize::BITS as usize {
                    return None;
                }
                value |= 1usize.checked_shl(bit as u32)?;
            }
            _ => return None,
        }
    }
    Some(value)
}

fn push_tape_field(out: &mut Vec<Mark>, tape: &[Mark]) {
    out.push('∈');
    out.extend_from_slice(tape);
    out.push('∋');
}

fn read_tape_field(word: &[Mark], cursor: &mut usize, end: usize) -> Result<Tape, String> {
    if *cursor >= end || word.get(*cursor).copied() != Some('∈') {
        return Err(String::from("malformed imscription-cycle length field"));
    }
    *cursor += 1;
    let mut out = Vec::new();
    while *cursor < end {
        let mark = word[*cursor];
        *cursor += 1;
        if mark == '∋' {
            if out.is_empty() {
                return Err(String::from("empty imscription-cycle length field"));
            }
            return Ok(out);
        }
        if mark != EVALT && mark != EVALF {
            return Err(String::from("imscription-cycle length field contains a non-numeral mark"));
        }
        out.push(mark);
    }
    Err(String::from("truncated imscription-cycle length field"))
}

fn push_len_field(out: &mut Vec<Mark>, len: usize) {
    push_tape_field(out, &usize_to_tape(len));
}

fn read_len_field(word: &[Mark], cursor: &mut usize, end: usize) -> Result<usize, String> {
    let tape = read_tape_field(word, cursor, end)?;
    tape_to_usize(&tape)
        .ok_or_else(|| String::from("imscription-cycle length field overflows host address space"))
}

fn push_blob(out: &mut Vec<Mark>, blob: &[Mark]) {
    push_len_field(out, blob.len());
    out.extend_from_slice(blob);
}

fn read_blob(word: &[Mark], cursor: &mut usize, end: usize) -> Result<Vec<Mark>, String> {
    let len = read_len_field(word, cursor, end)?;
    let blob_end = cursor.checked_add(len)
        .ok_or_else(|| String::from("imscription-cycle blob length overflow"))?;
    if blob_end > end {
        return Err(String::from("truncated imscription-cycle blob"));
    }
    let blob = word[*cursor..blob_end].to_vec();
    *cursor = blob_end;
    Ok(blob)
}

/// Serialize the complete proof circuit as one marks-only object.  The nested
/// dialectic and quotient certificates retain their own wire formats; the outer
/// cycle only length-frames those exact objects plus the exact lifted bridge.
pub fn encode_imscription_cycle(certificate: &ImscriptionCycleCertificate) -> Vec<Mark> {
    let dialectic = encode_dialectic_certificate(&certificate.dialectic);
    let quotient = encode_reentry_certificate(&certificate.quotient);
    let mut out = Vec::new();
    out.push('⊢');
    out.push('⋈');
    push_blob(&mut out, &dialectic);
    push_blob(&mut out, &certificate.lifted_carrier);
    push_blob(&mut out, &quotient);
    out.push('⊣');
    out
}

/// Reconstruct a complete cycle from one persisted marks-only object.  As with
/// the inner codecs, structural reconstruction and semantic replay are separate:
/// callers must pass the result to `verify_imscription_cycle` to establish the
/// operator-space descent, exact bridge, quotient, and fixed-object return.
pub fn decode_imscription_cycle(word: &[Mark]) -> Result<ImscriptionCycleCertificate, String> {
    if word.len() < 3
        || word.first().copied() != Some('⊢')
        || word.get(1).copied() != Some('⋈')
        || word.last().copied() != Some('⊣')
    {
        return Err(String::from("malformed imscription-cycle framing"));
    }
    let end = word.len() - 1;
    let mut cursor = 2usize;
    let dialectic_wire = read_blob(word, &mut cursor, end)?;
    let lifted_carrier = read_blob(word, &mut cursor, end)?;
    let quotient_wire = read_blob(word, &mut cursor, end)?;
    if cursor != end {
        return Err(String::from("trailing marks after imscription-cycle payload"));
    }
    Ok(ImscriptionCycleCertificate {
        dialectic: decode_dialectic_certificate(&dialectic_wire)?,
        lifted_carrier,
        quotient: decode_reentry_certificate(&quotient_wire)?,
    })
}
