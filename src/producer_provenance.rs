//! Producer-route provenance aligned with the native restored-support envelope.
//!
//! Factoring and passive extraction remain separate layers.  This module records
//! which producer support closed a factor before the common product boundary, then
//! runs that two-level deposit schedule through `provenance_envelope`.

use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

use crate::factorization_31_membrane::UnboundedResident;
use crate::provenance_envelope::{restored_support_ladder, LaneSupport};

pub const SUPPORT_PARITY: LaneSupport = 1 << 0;
pub const SUPPORT_PRIMALITY: LaneSupport = 1 << 1;
pub const SUPPORT_SHORT_FRONTIER: LaneSupport = 1 << 2;
pub const SUPPORT_EXTENDED_FERMAT: LaneSupport = 1 << 3;
pub const SUPPORT_DEEP_ARM: LaneSupport = 1 << 4;
pub const SUPPORT_PRODUCT_BOUNDARY: LaneSupport = 1 << 5;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ProducerRouteProvenance {
    pub route: String,
    /// Nested producer deposits, outer route first and common product boundary last.
    pub deposits: Vec<LaneSupport>,
    /// Native restored-support ladder for `deposits`, outermost first.
    pub ladder: Vec<LaneSupport>,
}

fn support_for_route(route: &str) -> Option<LaneSupport> {
    let common = SUPPORT_PARITY | SUPPORT_PRIMALITY | SUPPORT_SHORT_FRONTIER;
    match route {
        "frontier" => Some(common),
        "near-root" => Some(common | SUPPORT_EXTENDED_FERMAT),
        "HARD" => Some(common | SUPPORT_EXTENDED_FERMAT | SUPPORT_DEEP_ARM),
        _ => None,
    }
}

/// Build the native two-level provenance envelope for a named producer route.
///
/// The inner product-boundary deposit is identical for every route.  Route
/// distinctions therefore survive only in the outer restored support, exactly
/// where the producer differs before handing the frozen factor object downstream.
pub fn route_provenance(route: &str) -> Option<ProducerRouteProvenance> {
    let route_support = support_for_route(route)?;
    let deposits = vec![route_support, SUPPORT_PRODUCT_BOUNDARY];
    let ladder = restored_support_ladder(&deposits);
    Some(ProducerRouteProvenance {
        route: route.to_string(),
        deposits,
        ladder,
    })
}

/// Read the producer provenance from a completed arbitrary-width resident.
/// Only routes that reached the common resident product boundary are admitted.
pub fn resident_route_provenance(resident: &UnboundedResident) -> Option<ProducerRouteProvenance> {
    if !resident.boundary_ok || !resident.sidearm_round_trip {
        return None;
    }
    route_provenance(&resident.shape_route)
}
