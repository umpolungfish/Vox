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
    /// Live bulk/boundary permissions at closure.
    pub terminal_rwx: u8,
    /// Cell inside the closing lattice.
    pub lattice_cell: usize,
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
                    terminal_rwx: closed.imscription.rwx,
                    lattice_cell: closed.lattice_cell,
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
            || closed.imscription.rwx != certificate.terminal_rwx
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
