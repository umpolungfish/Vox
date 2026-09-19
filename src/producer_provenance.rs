//! Producer-route provenance aligned with the native restored-support envelope.
//!
//! Factoring and passive extraction remain separate layers.  This module records
//! which producer support closed a factor before the common product boundary, then
//! runs that two-level deposit schedule through `provenance_envelope`.

use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use core::cmp::Ordering;

use crate::factorization_31_membrane::UnboundedResident;
use crate::morphism_factor::{add, cmp, divmod, isqrt, mul, sub, tape_u64, zero};
use crate::provenance_envelope::{restored_support_ladder, LaneSupport};

pub const SUPPORT_PARITY: LaneSupport = 1 << 0;
pub const SUPPORT_PRIMALITY: LaneSupport = 1 << 1;
pub const SUPPORT_SHORT_FRONTIER: LaneSupport = 1 << 2;
pub const SUPPORT_EXTENDED_FERMAT: LaneSupport = 1 << 3;
pub const SUPPORT_DEEP_ARM: LaneSupport = 1 << 4;
pub const SUPPORT_PRODUCT_BOUNDARY: LaneSupport = 1 << 5;

/// The arbitrary-width resident's explicit Fermat arm walks this many cells.
/// `shape_route` is scout telemetry and may say `near-root` before that arm has
/// actually closed; provenance below canonicalizes that telemetry against the
/// carried factor relation when the pair proves the Fermat arm could not close.
const RESIDENT_FERMAT_CELLS: u64 = 1_000_000;

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

/// Exact Fermat cell for the first carried factor pair, measured from
/// `ceil(sqrt(N))`.  This is marks-native and does not search the lattice.
fn resident_factor_fermat_cell(resident: &UnboundedResident) -> Option<Vec<char>> {
    let one = tape_u64(1);
    let factor = resident
        .factors
        .iter()
        .find(|factor| {
            cmp(factor, &one) == Ordering::Greater
                && cmp(factor, &resident.n) == Ordering::Less
        })?;
    let (cofactor, remainder) = divmod(&resident.n, factor);
    if !zero(&remainder) || cmp(&cofactor, &one) != Ordering::Greater {
        return None;
    }

    // For the odd semiprime pair, Fermat closes at a=(p+q)/2.  If the sum is
    // not even this relation does not define a cell for the resident Fermat arm.
    let two = tape_u64(2);
    let (a, parity) = divmod(&add(factor, &cofactor), &two);
    if !zero(&parity) {
        return None;
    }

    let mut origin = isqrt(&resident.n);
    if cmp(&mul(&origin, &origin), &resident.n) == Ordering::Less {
        origin = add(&origin, &one);
    }
    if cmp(&a, &origin) == Ordering::Less {
        return None;
    }
    Some(sub(&a, &origin))
}

/// Canonical producer route for a completed resident.
///
/// `shape_route` is assigned by the scout before the later Fermat/deep arms run.
/// Therefore a scout miss can leave `near-root` behind even when the million-cell
/// Fermat arm also misses and the deep arm actually closes.  We only override
/// that telemetry when the carried factor pair proves its Fermat closing cell is
/// outside the resident's own Fermat window; then `near-root` is impossible and
/// the completed producer provenance is necessarily `HARD`.
fn canonical_resident_route(resident: &UnboundedResident) -> String {
    if resident.shape_route == "near-root" {
        if let Some(cell) = resident_factor_fermat_cell(resident) {
            if cmp(&cell, &tape_u64(RESIDENT_FERMAT_CELLS)) != Ordering::Less {
                return String::from("HARD");
            }
        }
    }
    resident.shape_route.clone()
}

/// Read the producer provenance from a completed arbitrary-width resident.
/// Only routes that reached the common resident product boundary are admitted.
pub fn resident_route_provenance(resident: &UnboundedResident) -> Option<ProducerRouteProvenance> {
    if !resident.boundary_ok || !resident.sidearm_round_trip {
        return None;
    }
    let route = canonical_resident_route(resident);
    route_provenance(&route)
}
