//! Coordinate transforms shared by the `pwoat.html` torus model.
//!
//! The number-line ring, prime-power helix, binary residue layer, and cardioid
//! sweep are views of one torus. Angles are represented as turns before being
//! projected, so full-turn wraps remain available to the caller.

use num_bigint::BigUint;
use num_traits::One;

pub const LOG_PERIOD: f64 = 7.0;
pub const TAU: f64 = 2.0 * core::f64::consts::PI;

/// Retained binary logarithm data. `fractional_log2` is a projection of the
/// leading mantissa; the exponent and mantissa remain available for re-fusion.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Log2Coordinate {
    pub integer_exponent: usize,
    pub leading_mantissa: u64,
    pub mantissa_bits: u8,
    pub fractional_log2: f64,
}

impl Log2Coordinate {
    pub fn fused_log2(self) -> f64 {
        self.integer_exponent as f64 + self.fractional_log2
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RingCoordinate {
    /// Fractional angle in turns, in [0, 1).
    pub phi_turn: f64,
    /// Whole turns removed from log2(n)/7.
    pub phi_wrap: i128,
    /// Number-line depth in turns, exactly 1/7.
    pub theta_turn: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SemiprimeTrine {
    pub p: RingCoordinate,
    pub q: RingCoordinate,
    pub n: RingCoordinate,
    /// Centered angle residual in turns, expected to be zero modulo one.
    pub residual_turn: f64,
    /// Wrap(p) + wrap(q) + fractional carry - wrap(n), expected to be zero.
    pub wrap_balance: i128,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HelixCoordinate {
    pub phi_turn: f64,
    pub theta_turn: f64,
    /// Odd re-entry is displayed on the antipodal sheet.
    pub negative_sheet: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BinaryCoordinate {
    pub phi_turn: f64,
    pub theta_turn: f64,
    pub reached_singletons: usize,
    pub residue_count: BigUint,
    pub diagonal_empty: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CardioidCoordinate {
    pub radius: f64,
    pub phi: f64,
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TorusPoint {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

/// `phi(n)/(2π) = log2(n)/7 mod 1`, on the `k=1` number-line ring.
pub fn number_line(log2_n: f64) -> RingCoordinate {
    let scaled = log2_n / LOG_PERIOD;
    let phi_wrap = scaled.floor() as i128;
    RingCoordinate {
        phi_turn: scaled - phi_wrap as f64,
        phi_wrap,
        theta_turn: 1.0 / LOG_PERIOD,
    }
}


/// Construct a torus coordinate without first fusing the integer exponent
/// into floating point. The integer part is divided by seven exactly.
pub fn number_line_parts(log: Log2Coordinate) -> RingCoordinate {
    let whole_turns = log.integer_exponent / 7;
    let remainder = log.integer_exponent % 7;
    let fractional_turn = (remainder as f64 + log.fractional_log2) / 7.0;
    let carry = fractional_turn.floor() as i128;
    RingCoordinate {
        phi_turn: fractional_turn - carry as f64,
        phi_wrap: whole_turns as i128 + carry,
        theta_turn: 1.0 / LOG_PERIOD,
    }
}

/// Map the multiplicative relation to the additive number-line trine.
pub fn semiprime_trine(log2_p: f64, log2_q: f64, log2_n: f64) -> SemiprimeTrine {
    let p = number_line(log2_p);
    let q = number_line(log2_q);
    let n = number_line(log2_n);
    let sum = p.phi_turn + q.phi_turn;
    let residual = centered_turn(sum - n.phi_turn);
    let fractional_carry = sum.floor() as i128;
    SemiprimeTrine {
        p,
        q,
        n,
        residual_turn: residual,
        wrap_balance: p.phi_wrap + q.phi_wrap + fractional_carry - n.phi_wrap,
    }
}

/// Additive-log trine over retained exponent and mantissa coordinates.
pub fn semiprime_trine_parts(
    log2_p: Log2Coordinate,
    log2_q: Log2Coordinate,
    log2_n: Log2Coordinate,
) -> SemiprimeTrine {
    let p = number_line_parts(log2_p);
    let q = number_line_parts(log2_q);
    let n = number_line_parts(log2_n);
    let sum = p.phi_turn + q.phi_turn;
    let fractional_carry = sum.floor() as i128;
    SemiprimeTrine {
        p,
        q,
        n,
        residual_turn: centered_turn(sum - n.phi_turn),
        wrap_balance: p.phi_wrap + q.phi_wrap + fractional_carry - n.phi_wrap,
    }
}

/// Prime-power helix coordinate, with the negative-sheet parity convention.
pub fn prime_helix(log2_p: f64, k: usize) -> HelixCoordinate {
    let scaled = k as f64 * log2_p / LOG_PERIOD;
    HelixCoordinate {
        phi_turn: scaled.rem_euclid(1.0),
        theta_turn: k as f64 / LOG_PERIOD,
        negative_sheet: k % 2 == 1,
    }
}

/// Project one coordinate through the HTML horn-torus embedding. The negative
/// sheet uses central inversion of all three coordinates.
pub fn torus_point(phi_turn: f64, theta_turn: f64, major: f64, minor: f64,
    negative_sheet: bool) -> TorusPoint {
    let phi = TAU * phi_turn;
    let theta = TAU * theta_turn;
    let ring = major + minor * libm::cos(theta);
    let sign = if negative_sheet { -1.0 } else { 1.0 };
    TorusPoint {
        x: sign * ring * libm::cos(phi),
        y: sign * ring * libm::sin(phi),
        z: sign * minor * libm::sin(theta),
    }
}

/// Binary powerset/residue point B_k(m), with the numeral supplied low bit first.
/// The residue cardinality remains exact at arbitrary depth.
pub fn binary_point(bits_low_first: &[bool]) -> BinaryCoordinate {
    let k = bits_low_first.len();
    let mut phi_turn = 0.0;
    for bit in bits_low_first.iter().rev() {
        phi_turn = (phi_turn + if *bit { 1.0 } else { 0.0 }) / 2.0;
    }
    let residue_count = (BigUint::one() << k) - BigUint::from(k);
    BinaryCoordinate {
        phi_turn,
        theta_turn: k as f64 / LOG_PERIOD,
        reached_singletons: k,
        residue_count,
        diagonal_empty: bits_low_first.iter().all(|bit| !*bit),
    }
}

/// Diagonal torus sweep projected to the cardioid, including slider rotation.
/// `e` is the log2 multiplier in [-7, 7], so the rotation is 2πe/7.
pub fn cardioid(t: f64, major_radius: f64, e: f64) -> CardioidCoordinate {
    let radius = major_radius * (1.0 + libm::cos(t));
    let phi = t + TAU * e / LOG_PERIOD;
    CardioidCoordinate {
        radius,
        phi,
        x: radius * libm::cos(phi),
        y: radius * libm::sin(phi),
    }
}

/// Exact golden-ratio radial identities at the four marked fifth-turn angles.
pub fn golden_cardioid_radii() -> [f64; 4] {
    let phi = (1.0 + libm::sqrt(5.0)) / 2.0;
    [2.0 + phi, phi * phi, 3.0 - phi, 2.0 - phi]
}

pub fn centered_turn(value: f64) -> f64 {
    let residual = value - value.floor();
    if residual > 0.5 { residual - 1.0 } else { residual }
}
