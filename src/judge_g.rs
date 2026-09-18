//! judge_g.rs — the judge as a GValue-native function.
//!
//! There is no Judg enum on the production path. The judge's input is one carrier
//! whose marks ARE the representation: a repr tag, then the number N, then the
//! representation's parameter — numerals-as-marks, 64 marks per u64 over {⊤=1,⊥=0}.
//! The judge's output is one mark, and the four judgment marks are the four Belnap
//! values the router already reads positionally:
//!
//!   ⊤ = T  closure           (EVALT,  deposit True)
//!   ⊥ = F  malformed         (EVALF,  deposit False)
//!   ⊞ = B  fork held open    (ENGAGR, the Belnap diagonal)
//!   ⊙ = N  no distinction    (IMSCRIB, identity / wildcard)
//!
//! The verdict logic is identical, mark for mark, to meta_router::judge — this
//! module exists so the marks, not the enums, cross the boundary.

use alloc::vec::Vec;
use crate::carrier::{GValue, Mark};
use crate::meta_router::{pollard_rho, is_prime_u64, Judg};
use crate::nested_frame::fermat_frontier;

/// Representation-tag marks (the same three the carrier admits as repr tags).
pub const REPR_TAG_SYM: Mark = '⊢'; // Symmetric{a}   — the Fermat/root interval
pub const REPR_TAG_RES: Mark = '⊣'; // Residue{k}     — the capacity/residue frame
pub const REPR_TAG_MUL: Mark = '⋈'; // Multiplicative — the x->x^2+c orbit

/// Judgment marks — the four Belnap values as marks.
pub const J_T: Mark = '⊤';
pub const J_F: Mark = '⊥';
pub const J_B: Mark = '⊞';
pub const J_N: Mark = '⊙';

/// One u64 is 64 marks. Bit 63 first, bit 0 last — the mark string reads as the
/// binary numeral.
pub const NUM_BITS: usize = 64;

pub fn encode_u64(x: u64) -> Vec<Mark> {
    let mut v: Vec<Mark> = Vec::with_capacity(NUM_BITS);
    for i in (0..NUM_BITS).rev() {
        v.push(if (x >> i) & 1 == 1 { '⊤' } else { '⊥' });
    }
    v
}

pub fn decode_u64(marks: &[Mark]) -> u64 {
    let mut x: u64 = 0;
    for m in marks.iter().take(NUM_BITS) {
        x = (x << 1) | if *m == '⊤' { 1 } else { 0 };
    }
    x
}

/// The carrier a judge receives: [repr_tag, N(64 marks), param(64 marks)].
/// param is `a` for Symmetric, `k` for Residue, unused (0) for Multiplicative —
/// carried so the object holds its own numeric content, read where a slot needs it.
pub fn repr_value(tag: Mark, n: u64, param: u64) -> GValue {
    let mut marks: Vec<Mark> = Vec::with_capacity(1 + 2 * NUM_BITS);
    marks.push(tag);
    marks.extend(encode_u64(n));
    marks.extend(encode_u64(param));
    GValue { marks }
}

/// Decode a carrier back to (tag, n, param). None if the arity is short.
pub fn repr_decode(v: &GValue) -> Option<(Mark, u64, u64)> {
    if v.arity() < 1 + 2 * NUM_BITS { return None; }
    let tag = v.marks[0];
    let n = decode_u64(&v.marks[1..1 + NUM_BITS]);
    let param = decode_u64(&v.marks[1 + NUM_BITS..1 + 2 * NUM_BITS]);
    Some((tag, n, param))
}

/// The verdict, as a mark — verbatim meta_router::judge's logic on the tag + N.
pub fn judge_mark(tag: Mark, n: u64) -> Mark {
    match tag {
        REPR_TAG_SYM => {
            if n % 2 == 0 { return J_T; }
            if fermat_frontier(n).is_some() { return J_T; }
            if is_prime_u64(n) { return J_N; }
            J_B
        }
        REPR_TAG_RES => {
            let cap = match crate::nested_frame::root_frame(n) {
                Some(f) => f.live_capacity(),
                None => 0,
            };
            if cap == 1 { J_T } else { J_B }
        }
        REPR_TAG_MUL => {
            if is_prime_u64(n) { return J_N; }
            match pollard_rho(n) {
                Some(g) if g > 1 => J_T,
                _ => J_N,
            }
        }
        // A tag that is not a representation is a malformed composition: F.
        _ => J_F,
    }
}

/// The judge. Input: a carrier. Output: a single-mark GValue — the judgment.
pub fn judge_g(v: &GValue) -> GValue {
    match repr_decode(v) {
        Some((tag, n, _param)) => GValue::single(judge_mark(tag, n)),
        None => GValue::single(J_F), // malformed carrier
    }
}

/// The factor a T-judgment hands, as (p,q) — kept beside the mark so the loop can
/// close. Recovered the same way judge does, not stored in the judgment value.
pub fn found_factor(tag: Mark, n: u64) -> Option<(u64, u64)> {
    match tag {
        REPR_TAG_SYM => {
            if n % 2 == 0 { return Some((2, n / 2)); }
            if let Some((p, q, _i)) = fermat_frontier(n) { return Some((p, q)); }
            None
        }
        REPR_TAG_MUL => match pollard_rho(n) { Some(g) if g > 1 => Some((g, n / g)), _ => None },
        _ => None,
    }
}

/// Map a meta_router::Judg to its mark — used only by the agreement harness.
pub fn judg_to_mark(j: Judg) -> Mark {
    match j {
        Judg::T => J_T,
        Judg::B => J_B,
        Judg::N => J_N,
        Judg::F => J_F,
    }
}

// ---- the trajectory loop, in marks ----

/// The re-entry, as a mark — verbatim meta_router::reenter: only (Symmetric,B)
/// holds the fork one level down; every other exposed level is the orbit.
pub fn reenter_mark(tag: Mark, jmark: Mark) -> Mark {
    if tag == REPR_TAG_SYM && jmark == J_B { REPR_TAG_RES } else { REPR_TAG_MUL }
}

/// One recorded transition: representation tag -> judgment mark. No enums.
#[derive(Clone, PartialEq, Debug)]
pub struct GStep { pub tag: Mark, pub judg: Mark }

/// The resident trajectory in marks: N -> repr tag -> judge_g -> reenter_mark ...
/// -> FIX. Returns the ordered factor pair (if it closes) and every mark step.
/// Bit-identical verdict sequence to meta_router::route; the enum does not appear.
pub fn route_g(n: u64) -> (Option<(u64, u64)>, alloc::vec::Vec<GStep>) {
    let mut traj: alloc::vec::Vec<GStep> = alloc::vec::Vec::new();
    let mut tag = REPR_TAG_SYM;
    for _ in 0..6 {
        let param = match tag {
            REPR_TAG_SYM => crate::nested_frame::isqrt_u64(n) + 1,
            REPR_TAG_RES => 1,
            _ => 0,
        };
        let carrier = repr_value(tag, n, param);
        let j = judge_g(&carrier).mark0().unwrap_or(J_F);
        traj.push(GStep { tag, judg: j });
        if j == J_T {
            if let Some((p, q)) = found_factor(tag, n) {
                if p > 1 && q > 1 && p * q == n {
                    let (a, b) = if p <= q { (p, q) } else { (q, p) };
                    return (Some((a, b)), traj);
                }
            }
        }
        if j == J_N { break; }
        tag = reenter_mark(tag, j);
    }
    (None, traj)
}
