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
