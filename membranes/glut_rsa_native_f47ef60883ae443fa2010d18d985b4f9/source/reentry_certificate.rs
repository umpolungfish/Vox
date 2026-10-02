//! First-class certificates for passive factor-carrier self-entry.
//!
//! A certificate records the trace projection before and after every generation
//! while carrying one frozen arbitrary-width `(N,p,q)` witness.  The verifier is
//! deliberately independent of `factor_extract::reenter_once`: it checks each
//! changed link by finding the claimed `delete_word` image under the public
//! tape-native relaxed relation, and checks the terminal self-link by proving no
//! admissible deletion remains.

use alloc::string::String;
use alloc::vec::Vec;

use crate::factor_extract::{reenter_once, FactorCarrier, Mark, Tape};
use crate::tape_delete::delete_word;
use crate::trace_algebra::{admissible_relaxed_with_witness, witness_valid};
use crate::trace_word::decode_trace;
use crate::vox::{EVALF, EVALT};

#[derive(Clone, PartialEq, Debug)]
pub struct ReentryLink {
    pub before: Vec<Mark>,
    pub after: Vec<Mark>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct ReentryCertificate {
    pub n: Tape,
    pub p: Tape,
    pub q: Tape,
    pub links: Vec<ReentryLink>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct ReentryCertificateSummary {
    pub generations: usize,
    pub transforms: usize,
    pub normal_form: Vec<Mark>,
}

impl ReentryCertificate {
    pub fn normal_form(&self) -> Option<&[Mark]> {
        self.links.last().map(|link| link.after.as_slice())
    }
}

/// Produce the canonical front-first self-entry certificate used by the passive
/// extractor.  The last link is always the unchanged fixed-point self-link.
pub fn certify_reentry(carrier: &FactorCarrier) -> Result<ReentryCertificate, String> {
    carrier.validate()?;
    let mut current = carrier.clone();
    let mut links = Vec::new();

    loop {
        let before = current.trace.clone();
        let (next, changed) = reenter_once(&current)?;
        links.push(ReentryLink {
            before,
            after: next.trace.clone(),
        });
        current = next;
        if !changed {
            break;
        }
    }

    Ok(ReentryCertificate {
        n: carrier.n.clone(),
        p: carrier.p.clone(),
        q: carrier.q.clone(),
        links,
    })
}

fn valid_carrier_for_trace(cert: &ReentryCertificate, trace: &[Mark]) -> bool {
    FactorCarrier::new(
        cert.n.clone(),
        cert.p.clone(),
        cert.q.clone(),
        trace.to_vec(),
    )
    .is_ok()
}

fn claimed_single_admissible_deletion(cert: &ReentryCertificate, before: &[Mark], after: &[Mark]) -> bool {
    let Some(steps) = decode_trace(before) else { return false };
    for i in 0..steps.len() {
        let Some(candidate) = delete_word(before, i) else { continue };
        if candidate == after
            && admissible_relaxed_with_witness(
                before,
                &candidate,
                &cert.n,
                &cert.p,
                &cert.q,
            )
        {
            return true;
        }
    }
    false
}

fn has_admissible_deletion(cert: &ReentryCertificate, trace: &[Mark]) -> bool {
    let Some(steps) = decode_trace(trace) else { return false };
    for i in 0..steps.len() {
        let Some(candidate) = delete_word(trace, i) else { continue };
        if admissible_relaxed_with_witness(
            trace,
            &candidate,
            &cert.n,
            &cert.p,
            &cert.q,
        ) {
            return true;
        }
    }
    false
}

/// Verify a self-entry chain without re-running the extractor's reduction
/// schedule.  Accepted certificates establish:
///
/// - the arbitrary-width witness reconstructs `N`;
/// - every trace in the chain is itself a valid closed factor carrier;
/// - adjacent links are contiguous;
/// - every changed link is exactly one admissible structural deletion;
/// - every changed link strictly removes one trace record;
/// - the final link is an unchanged self-link with no admissible deletion.
pub fn verify_reentry_certificate(
    cert: &ReentryCertificate,
) -> Result<ReentryCertificateSummary, String> {
    if !witness_valid(&cert.n, &cert.p, &cert.q) {
        return Err(String::from("certificate witness does not reconstruct N"));
    }
    if cert.links.is_empty() {
        return Err(String::from("certificate has no self-entry links"));
    }

    let mut transforms = 0usize;
    for (generation, link) in cert.links.iter().enumerate() {
        if !valid_carrier_for_trace(cert, &link.before) {
            return Err(String::from("certificate contains an invalid before-carrier"));
        }
        if !valid_carrier_for_trace(cert, &link.after) {
            return Err(String::from("certificate contains an invalid after-carrier"));
        }

        if generation > 0 && cert.links[generation - 1].after != link.before {
            return Err(String::from("certificate chain is not contiguous"));
        }

        if link.before == link.after {
            if generation + 1 != cert.links.len() {
                return Err(String::from("certificate reaches a fixed self-link before the end"));
            }
            if has_admissible_deletion(cert, &link.before) {
                return Err(String::from("certificate claims a fixed point with an admissible deletion remaining"));
            }
            continue;
        }

        if generation + 1 == cert.links.len() {
            return Err(String::from("certificate does not end with an unchanged fixed-point link"));
        }

        let before_count = decode_trace(&link.before)
            .ok_or_else(|| String::from("certificate before-trace is malformed"))?
            .len();
        let after_count = decode_trace(&link.after)
            .ok_or_else(|| String::from("certificate after-trace is malformed"))?
            .len();
        if after_count + 1 != before_count {
            return Err(String::from("certificate changed link is not a one-record descent"));
        }
        if !claimed_single_admissible_deletion(cert, &link.before, &link.after) {
            return Err(String::from("certificate changed link is not an admissible delete_word step"));
        }
        transforms += 1;
    }

    let normal_form = cert.links.last().unwrap().after.clone();
    Ok(ReentryCertificateSummary {
        generations: cert.links.len(),
        transforms,
        normal_form,
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
        return Err(String::from("malformed certificate tape field"));
    }
    *cursor += 1;
    let mut out = Vec::new();
    while *cursor < end {
        let mark = word[*cursor];
        *cursor += 1;
        if mark == '∋' {
            if out.is_empty() {
                return Err(String::from("empty certificate tape field"));
            }
            return Ok(out);
        }
        if mark != EVALT && mark != EVALF {
            return Err(String::from("certificate tape field contains a non-numeral mark"));
        }
        out.push(mark);
    }
    Err(String::from("truncated certificate tape field"))
}

fn push_len_field(out: &mut Vec<Mark>, len: usize) {
    let tape = usize_to_tape(len);
    push_tape_field(out, &tape);
}

fn read_len_field(word: &[Mark], cursor: &mut usize, end: usize) -> Result<usize, String> {
    let tape = read_tape_field(word, cursor, end)?;
    tape_to_usize(&tape).ok_or_else(|| String::from("certificate length field overflows host address space"))
}

/// Marks-only certificate serialization.
///
/// The three arbitrary-width numeral witnesses and every length field are
/// self-delimiting `∈ <EVALT/EVALF tape> ∋` fields.  Each trace then follows its
/// declared length verbatim, so structural-looking glyphs inside an applied-word
/// payload remain data rather than framing.
pub fn encode_reentry_certificate(cert: &ReentryCertificate) -> Vec<Mark> {
    let mut out = Vec::new();
    out.push('⊢');
    push_tape_field(&mut out, &cert.n);
    push_tape_field(&mut out, &cert.p);
    push_tape_field(&mut out, &cert.q);
    push_len_field(&mut out, cert.links.len());
    for link in &cert.links {
        push_len_field(&mut out, link.before.len());
        out.extend_from_slice(&link.before);
        push_len_field(&mut out, link.after.len());
        out.extend_from_slice(&link.after);
    }
    out.push('⊣');
    out
}

/// Decode the marks-only certificate envelope.  Structural decoding and proof
/// verification are intentionally separate operations: callers can transport a
/// certificate, reconstruct it elsewhere, then pass it to
/// `verify_reentry_certificate` as the independent checker.
pub fn decode_reentry_certificate(word: &[Mark]) -> Result<ReentryCertificate, String> {
    if word.len() < 2 || word.first().copied() != Some('⊢') || word.last().copied() != Some('⊣') {
        return Err(String::from("malformed reentry certificate framing"));
    }
    let end = word.len() - 1;
    let mut cursor = 1usize;
    let n = read_tape_field(word, &mut cursor, end)?;
    let p = read_tape_field(word, &mut cursor, end)?;
    let q = read_tape_field(word, &mut cursor, end)?;
    let count = read_len_field(word, &mut cursor, end)?;
    let mut links = Vec::with_capacity(count);

    for _ in 0..count {
        let before_len = read_len_field(word, &mut cursor, end)?;
        let before_end = cursor.checked_add(before_len)
            .ok_or_else(|| String::from("certificate before-trace length overflow"))?;
        if before_end > end {
            return Err(String::from("truncated certificate before-trace"));
        }
        let before = word[cursor..before_end].to_vec();
        cursor = before_end;

        let after_len = read_len_field(word, &mut cursor, end)?;
        let after_end = cursor.checked_add(after_len)
            .ok_or_else(|| String::from("certificate after-trace length overflow"))?;
        if after_end > end {
            return Err(String::from("truncated certificate after-trace"));
        }
        let after = word[cursor..after_end].to_vec();
        cursor = after_end;
        links.push(ReentryLink { before, after });
    }

    if cursor != end {
        return Err(String::from("trailing marks after reentry certificate payload"));
    }

    Ok(ReentryCertificate { n, p, q, links })
}

fn same_unordered_witness(a: &ReentryCertificate, b: &ReentryCertificate) -> bool {
    a.n == b.n
        && ((a.p == b.p && a.q == b.q) || (a.p == b.q && a.q == b.p))
}

/// Splice two chain fragments at an identical represented carrier.  Fragments
/// need not individually end at a fixed point; the composed result should be
/// passed to `verify_reentry_certificate` to establish a complete proof.
///
/// Composition is intentionally stricter than ≡c: a merely equivalent join is
/// not enough.  The prefix's final `after` trace must equal the suffix's first
/// `before` trace byte-for-byte.  An explicit equivalence bridge, if desired,
/// belongs in a separate proof object rather than being silently normalized.
pub fn compose_reentry_fragments(
    prefix: &ReentryCertificate,
    suffix: &ReentryCertificate,
) -> Result<ReentryCertificate, String> {
    if !same_unordered_witness(prefix, suffix)
        || !witness_valid(&prefix.n, &prefix.p, &prefix.q)
        || !witness_valid(&suffix.n, &suffix.p, &suffix.q)
    {
        return Err(String::from("certificate fragments do not carry the same valid witness"));
    }
    let prefix_last = prefix.links.last()
        .ok_or_else(|| String::from("prefix certificate fragment is empty"))?;
    let suffix_first = suffix.links.first()
        .ok_or_else(|| String::from("suffix certificate fragment is empty"))?;
    if prefix_last.before == prefix_last.after {
        return Err(String::from("cannot continue after a fixed-point self-link"));
    }
    if prefix_last.after != suffix_first.before {
        return Err(String::from("certificate fragments do not meet at an identical carrier"));
    }

    let mut links = prefix.links.clone();
    links.extend_from_slice(&suffix.links);
    Ok(ReentryCertificate {
        n: prefix.n.clone(),
        p: prefix.p.clone(),
        q: prefix.q.clone(),
        links,
    })
}
