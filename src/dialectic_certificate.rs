//! Replay-checked certificates for dialectic imscription descent.
//!
//! A certificate stores every complete marks-only `DialecticObject` that existed
//! before one descent plus the terminal factor carrier and terminal imscription
//! metadata. Verification starts from those wire images only: every intermediate
//! object is decoded afresh, its next descent is replayed, and the resulting wire
//! image must be byte-for-byte the next certified object. The final replay must
//! close to the exact recorded factor carrier and terminal imscription.
//!
//! This checker intentionally replays `DialecticObject::descend`; unlike
//! `reentry_certificate`, it is a persistence/continuity certificate for the
//! operator-space transformation itself, not an independent implementation of
//! the factoring lattice.

use alloc::string::String;
use alloc::vec::Vec;

use crate::dialectic_reentry::{Descent, DialecticObject, IM_RWX};
use crate::factor_extract::{FactorCarrier, Mark, Tape};
use crate::provenance_envelope::LaneSupport;
use crate::vox::{EVALF, EVALT};

#[derive(Clone, PartialEq, Debug)]
pub struct DialecticCertificate {
    /// Exact wire image of every unresolved whole object before one descent.
    pub objects: Vec<Vec<Mark>>,
    /// Exact marks-only terminal factor carrier.
    pub terminal_carrier: Vec<Mark>,
    /// Closing IMASM word that contracted around the factor pair.
    pub terminal_word: Vec<Mark>,
    /// Restored support embodied by the terminal whole object.
    pub terminal_support: LaneSupport,
    /// Boundary at which the closing lattice locked.
    pub terminal_boundary: Tape,
    /// Exact finite lattice region imscribed at closure.
    pub terminal_span: Tape,
    /// Live bulk/boundary permissions at closure.
    pub terminal_rwx: u8,
    /// Closing coordinate inside the current lattice, carried as a native numeral tape.
    /// The local executor may use a host loop index while walking one finite word,
    /// but that host width is not part of the persisted proof state.
    pub lattice_cell: Tape,
    /// Present when the closing lattice was Lehman's multiplier lattice.
    pub lehman_multiplier: Option<Tape>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct DialecticCertificateSummary {
    /// Number of consumed whole-object imscriptions.
    pub descents: usize,
    /// Support carried by each unresolved object in descent order.
    pub supports: Vec<LaneSupport>,
    pub terminal_support: LaneSupport,
    pub terminal_carrier: FactorCarrier,
}

/// Build a certificate by repeatedly consuming complete operator-space objects.
/// Each next object is serialized before it is allowed to descend again.
pub fn certify_dialectic(start: &DialecticObject) -> Result<DialecticCertificate, String> {
    let mut current = start.clone();
    let mut objects = Vec::new();

    loop {
        current.validate()?;
        objects.push(current.encode());
        match current.descend()? {
            Descent::Continue(next) => current = next,
            Descent::Closed(closed) => {
                return Ok(DialecticCertificate {
                    objects,
                    terminal_carrier: closed.carrier.encode(),
                    terminal_word: closed.word,
                    terminal_support: closed.support,
                    terminal_boundary: closed.imscription.boundary,
                    terminal_span: closed.imscription.span,
                    terminal_rwx: closed.imscription.rwx,
                    lattice_cell: usize_to_tape(closed.lattice_cell),
                    lehman_multiplier: closed.lehman_multiplier,
                });
            }
        }
    }
}

/// Verify the whole descent from its persisted marks only.
///
/// Continuations must be exact wire identities with the next certified object;
/// no host cursor, hidden route state, or reconstructed boundary may bridge a
/// generation. The final descent must reproduce the recorded terminal carrier
/// and every terminal imscription field exactly.
pub fn verify_dialectic_certificate(
    certificate: &DialecticCertificate,
) -> Result<DialecticCertificateSummary, String> {
    if certificate.objects.is_empty() {
        return Err(String::from("dialectic certificate has no operator-space objects"));
    }
    if certificate.terminal_rwx != IM_RWX {
        return Err(String::from("dialectic certificate terminal imscription is not live r/w/x"));
    }

    let first = DialecticObject::decode(&certificate.objects[0])?;
    let frozen_n = first.n.clone();
    let mut supports = Vec::with_capacity(certificate.objects.len());

    for (i, wire) in certificate.objects.iter().enumerate() {
        let object = DialecticObject::decode(wire)?;
        if object.n != frozen_n {
            return Err(String::from("dialectic certificate changed the bulk N"));
        }
        supports.push(object.support);

        let replay = object.descend()?;
        let last = i + 1 == certificate.objects.len();
        if !last {
            match replay {
                Descent::Continue(next) => {
                    if next.encode() != certificate.objects[i + 1] {
                        return Err(String::from(
                            "dialectic certificate continuation does not equal the next persisted whole object",
                        ));
                    }
                }
                Descent::Closed(_) => {
                    return Err(String::from(
                        "dialectic certificate continues after the imscription already closed",
                    ));
                }
            }
            continue;
        }

        let closed = match replay {
            Descent::Closed(closed) => closed,
            Descent::Continue(_) => {
                return Err(String::from(
                    "dialectic certificate ends before the operator-space imscription closes",
                ));
            }
        };

        if closed.carrier.encode() != certificate.terminal_carrier {
            return Err(String::from("dialectic certificate terminal factor carrier mismatch"));
        }
        if closed.word != certificate.terminal_word
            || closed.support != certificate.terminal_support
            || closed.imscription.boundary != certificate.terminal_boundary
            || closed.imscription.span != certificate.terminal_span
            || closed.imscription.rwx != certificate.terminal_rwx
            || usize_to_tape(closed.lattice_cell) != certificate.lattice_cell
            || closed.lehman_multiplier != certificate.lehman_multiplier
        {
            return Err(String::from("dialectic certificate terminal imscription mismatch"));
        }

        let terminal_carrier = FactorCarrier::decode(&certificate.terminal_carrier)?;
        return Ok(DialecticCertificateSummary {
            descents: certificate.objects.len(),
            supports,
            terminal_support: certificate.terminal_support,
            terminal_carrier,
        });
    }

    Err(String::from("dialectic certificate verification fell through"))
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
        return Err(String::from("malformed dialectic certificate tape field"));
    }
    *cursor += 1;
    let mut out = Vec::new();
    while *cursor < end {
        let mark = word[*cursor];
        *cursor += 1;
        if mark == '∋' {
            if out.is_empty() {
                return Err(String::from("empty dialectic certificate tape field"));
            }
            return Ok(out);
        }
        if mark != EVALT && mark != EVALF {
            return Err(String::from("dialectic certificate tape field contains a non-numeral mark"));
        }
        out.push(mark);
    }
    Err(String::from("truncated dialectic certificate tape field"))
}

fn push_len_field(out: &mut Vec<Mark>, len: usize) {
    let tape = usize_to_tape(len);
    push_tape_field(out, &tape);
}

fn read_len_field(word: &[Mark], cursor: &mut usize, end: usize) -> Result<usize, String> {
    let tape = read_tape_field(word, cursor, end)?;
    tape_to_usize(&tape)
        .ok_or_else(|| String::from("dialectic certificate length field overflows host address space"))
}

fn push_blob(out: &mut Vec<Mark>, blob: &[Mark]) {
    push_len_field(out, blob.len());
    out.extend_from_slice(blob);
}

fn read_blob(word: &[Mark], cursor: &mut usize, end: usize) -> Result<Vec<Mark>, String> {
    let len = read_len_field(word, cursor, end)?;
    let blob_end = cursor.checked_add(len)
        .ok_or_else(|| String::from("dialectic certificate blob length overflow"))?;
    if blob_end > end {
        return Err(String::from("truncated dialectic certificate blob"));
    }
    let blob = word[*cursor..blob_end].to_vec();
    *cursor = blob_end;
    Ok(blob)
}

/// Marks-only serialization of the entire dialectic proof object.
///
/// Every nested object and structural-looking payload is length-framed, while
/// counts and scalar metadata are carried as native numeral tapes. The terminal
/// boundary, span and lattice coordinate are exact tape numerals. The optional
/// Lehman multiplier uses a zero length for `None`; a present multiplier is a raw
/// numeral tape whose positive length is written immediately before it.
pub fn encode_dialectic_certificate(certificate: &DialecticCertificate) -> Vec<Mark> {
    let mut out = Vec::new();
    out.push('⊢');
    out.push('⊙');
    push_len_field(&mut out, certificate.objects.len());
    for object in &certificate.objects {
        push_blob(&mut out, object);
    }
    push_blob(&mut out, &certificate.terminal_carrier);
    push_blob(&mut out, &certificate.terminal_word);
    push_tape_field(&mut out, &usize_to_tape(certificate.terminal_support as usize));
    push_tape_field(&mut out, &certificate.terminal_boundary);
    push_tape_field(&mut out, &certificate.terminal_span);
    push_tape_field(&mut out, &usize_to_tape(certificate.terminal_rwx as usize));
    push_tape_field(&mut out, &certificate.lattice_cell);
    match &certificate.lehman_multiplier {
        Some(multiplier) => {
            push_len_field(&mut out, multiplier.len());
            out.extend_from_slice(multiplier);
        }
        None => push_len_field(&mut out, 0),
    }
    out.push('⊣');
    out
}

/// Decode one persisted dialectic proof object without relying on any runtime
/// state from the original descent. Proof replay remains a separate explicit
/// call to `verify_dialectic_certificate`.
pub fn decode_dialectic_certificate(word: &[Mark]) -> Result<DialecticCertificate, String> {
    if word.len() < 3
        || word.first().copied() != Some('⊢')
        || word.get(1).copied() != Some('⊙')
        || word.last().copied() != Some('⊣')
    {
        return Err(String::from("malformed dialectic certificate framing"));
    }
    let end = word.len() - 1;
    let mut cursor = 2usize;
    let count = read_len_field(word, &mut cursor, end)?;
    let mut objects = Vec::with_capacity(count);
    for _ in 0..count {
        objects.push(read_blob(word, &mut cursor, end)?);
    }
    let terminal_carrier = read_blob(word, &mut cursor, end)?;
    let terminal_word = read_blob(word, &mut cursor, end)?;
    let terminal_support = tape_to_usize(&read_tape_field(word, &mut cursor, end)?)
        .and_then(|v| u32::try_from(v).ok())
        .ok_or_else(|| String::from("dialectic certificate support field overflow"))?;
    let terminal_boundary = read_tape_field(word, &mut cursor, end)?;
    let terminal_span = read_tape_field(word, &mut cursor, end)?;
    let terminal_rwx = tape_to_usize(&read_tape_field(word, &mut cursor, end)?)
        .and_then(|v| u8::try_from(v).ok())
        .ok_or_else(|| String::from("dialectic certificate r/w/x field overflow"))?;
    let lattice_cell = read_tape_field(word, &mut cursor, end)?;
    let multiplier_len = read_len_field(word, &mut cursor, end)?;
    let lehman_multiplier = if multiplier_len == 0 {
        None
    } else {
        let multiplier_end = cursor.checked_add(multiplier_len)
            .ok_or_else(|| String::from("dialectic certificate multiplier length overflow"))?;
        if multiplier_end > end {
            return Err(String::from("truncated dialectic certificate multiplier"));
        }
        let multiplier = word[cursor..multiplier_end].to_vec();
        if multiplier.iter().any(|&mark| mark != EVALT && mark != EVALF) {
            return Err(String::from("dialectic certificate multiplier contains a non-numeral mark"));
        }
        cursor = multiplier_end;
        Some(multiplier)
    };
    if cursor != end {
        return Err(String::from("trailing marks after dialectic certificate payload"));
    }
    Ok(DialecticCertificate {
        objects,
        terminal_carrier,
        terminal_word,
        terminal_support,
        terminal_boundary,
        terminal_span,
        terminal_rwx,
        lattice_cell,
        lehman_multiplier,
    })
}
