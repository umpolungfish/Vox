//! Replay-checked certificates for dialectic imscription descent.
//!
//! A certificate stores every complete marks-only `DialecticObject` that existed
//! before one descent plus the terminal factor carrier and terminal imscription.
//! Verification starts from those wire images only: every intermediate object is
//! decoded afresh, its next descent is replayed, and the resulting wire image must
//! be byte-for-byte the next certified object. The final replay must close to the
//! exact recorded factor carrier and terminal imscription.

use alloc::string::String;
use alloc::vec::Vec;

use crate::dialectic_reentry::{
    Descent, DialecticObject, Imscription, ImscriptionRwx, IM_RWX,
};
use crate::factor_extract::{FactorCarrier, Mark, Tape};
use crate::provenance_envelope::LaneSupport;
use crate::vox::{EVALF, EVALT};

#[derive(Clone, PartialEq, Debug)]
pub struct DialecticCertificate {
    /// Exact wire image of every unresolved whole object before one descent.
    pub objects: Vec<Vec<Mark>>,
    /// Exact marks-only terminal factor carrier.
    pub terminal_carrier: Vec<Mark>,
    /// Restored support embodied by the terminal whole object.
    pub terminal_support: LaneSupport,
    /// One authoritative closing imscription. Boundary, span and closed word
    /// occur only as endpoints of this relation.
    pub terminal_imscription: Imscription,
    /// Closing coordinate inside the current lattice, carried as a native numeral tape.
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
/// A genuine FOUR=N continuation is an exact whole-object fixed point. This
/// factor-closing certificate has no terminal carrier for that neutral point, so
/// it returns explicitly rather than replaying the same restart object forever.
pub fn certify_dialectic(start: &DialecticObject) -> Result<DialecticCertificate, String> {
    let mut current = start.clone();
    let mut objects = Vec::new();

    loop {
        current.validate()?;
        let current_wire = current.encode();
        objects.push(current_wire.clone());
        match current.descend()? {
            Descent::Continue(next) => {
                if next.encode() == current_wire {
                    return Err(String::from(
                        "dialectic certificate reached FOUR=N unchanged imscription before factor closure",
                    ));
                }
                current = next;
            }
            Descent::Closed(closed) => {
                return Ok(DialecticCertificate {
                    objects,
                    terminal_carrier: closed.carrier.encode(),
                    terminal_support: closed.support,
                    terminal_imscription: closed.imscription,
                    lattice_cell: closed.lattice_cell,
                    lehman_multiplier: closed.lehman_multiplier,
                });
            }
        }
    }
}

/// Verify the whole descent from its persisted marks only.
pub fn verify_dialectic_certificate(
    certificate: &DialecticCertificate,
) -> Result<DialecticCertificateSummary, String> {
    if certificate.objects.is_empty() {
        return Err(String::from("dialectic certificate has no operator-space objects"));
    }
    if certificate.terminal_imscription.rwx.rights != IM_RWX {
        return Err(String::from(
            "dialectic certificate terminal imscription does not expose live r/w/x capabilities",
        ));
    }

    let first = DialecticObject::decode(&certificate.objects[0])?;
    let frozen_n = first.n.clone();
    if !certificate.terminal_imscription.relation_is_live_for(&frozen_n) {
        return Err(String::from(
            "dialectic certificate terminal r/w/x relation is not live for the original bulk",
        ));
    }

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
        if closed.support != certificate.terminal_support
            || closed.imscription != certificate.terminal_imscription
            || closed.lattice_cell != certificate.lattice_cell
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
    push_tape_field(out, &usize_to_tape(len));
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
    let blob_end = cursor
        .checked_add(len)
        .ok_or_else(|| String::from("dialectic certificate blob length overflow"))?;
    if blob_end > end {
        return Err(String::from("truncated dialectic certificate blob"));
    }
    let blob = word[*cursor..blob_end].to_vec();
    *cursor = blob_end;
    Ok(blob)
}

/// Marks-only serialization of the entire dialectic proof object. Terminal
/// boundary, span and word occur once, inside the dynamic r/w/x relation.
pub fn encode_dialectic_certificate(certificate: &DialecticCertificate) -> Vec<Mark> {
    let mut out = Vec::new();
    out.push('⊢');
    out.push('⊙');
    push_len_field(&mut out, certificate.objects.len());
    for object in &certificate.objects {
        push_blob(&mut out, object);
    }
    push_blob(&mut out, &certificate.terminal_carrier);
    push_tape_field(
        &mut out,
        &usize_to_tape(certificate.terminal_support as usize),
    );
    push_tape_field(
        &mut out,
        &usize_to_tape(certificate.terminal_imscription.rwx.rights as usize),
    );
    push_tape_field(&mut out, &certificate.terminal_imscription.rwx.read_bulk);
    push_tape_field(
        &mut out,
        &certificate.terminal_imscription.rwx.write_boundary,
    );
    push_tape_field(
        &mut out,
        &certificate.terminal_imscription.rwx.execute_span,
    );
    push_blob(
        &mut out,
        &certificate.terminal_imscription.rwx.execute_word,
    );
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

/// Decode one persisted dialectic proof object. Proof replay remains a separate
/// explicit call to `verify_dialectic_certificate`.
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
    let terminal_support = tape_to_usize(&read_tape_field(word, &mut cursor, end)?)
        .and_then(|v| u32::try_from(v).ok())
        .ok_or_else(|| String::from("dialectic certificate support field overflow"))?;
    let rights = tape_to_usize(&read_tape_field(word, &mut cursor, end)?)
        .and_then(|v| u8::try_from(v).ok())
        .ok_or_else(|| String::from("dialectic certificate r/w/x rights field overflow"))?;
    let read_bulk = read_tape_field(word, &mut cursor, end)?;
    let write_boundary = read_tape_field(word, &mut cursor, end)?;
    let execute_span = read_tape_field(word, &mut cursor, end)?;
    let execute_word = read_blob(word, &mut cursor, end)?;
    let terminal_imscription = Imscription {
        rwx: ImscriptionRwx {
            rights,
            read_bulk,
            write_boundary,
            execute_span,
            execute_word,
        },
    };
    let lattice_cell = read_tape_field(word, &mut cursor, end)?;
    let multiplier_len = read_len_field(word, &mut cursor, end)?;
    let lehman_multiplier = if multiplier_len == 0 {
        None
    } else {
        let multiplier_end = cursor
            .checked_add(multiplier_len)
            .ok_or_else(|| String::from("dialectic certificate multiplier length overflow"))?;
        if multiplier_end > end {
            return Err(String::from("truncated dialectic certificate multiplier"));
        }
        let multiplier = word[cursor..multiplier_end].to_vec();
        if multiplier
            .iter()
            .any(|&mark| mark != EVALT && mark != EVALF)
        {
            return Err(String::from(
                "dialectic certificate multiplier contains a non-numeral mark",
            ));
        }
        cursor = multiplier_end;
        Some(multiplier)
    };
    if cursor != end {
        return Err(String::from(
            "trailing marks after dialectic certificate payload",
        ));
    }
    Ok(DialecticCertificate {
        objects,
        terminal_carrier,
        terminal_support,
        terminal_imscription,
        lattice_cell,
        lehman_multiplier,
    })
}
