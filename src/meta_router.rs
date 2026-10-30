//! meta_router.rs — the resident self-judging trajectory.
//!
//! The routing is itself an IMASM object. The carrier does not choose among fixed
//! algorithms on a fixed representation of N; it constructs a *representation*,
//! judges it, imscribes the judgment, and re-enters at the level the judgment
//! exposes, until the product closes:
//!
//!   N -> construct representation -> judge -> imscribe(judgment)
//!     -> re-enter (transform the representation) -> judge again -> FIX.
//!
//! Belnap judgments:  T closure (reconstruct and agree) · B a fork held open
//! (productive unresolved structure) · N no distinction present (generate one)
//! · F a malformed composition (type error, not an arithmetic dead-end).
//!
//! Every step carries an IMASM word, so a successful factorization is a trajectory
//! `representation -> judgment -> re-entry -> new representation -> closure`, and
//! recurrent words across trajectories are candidate new primitive carriers.

use crate::nested_frame::{fermat_frontier, isqrt_u64};
use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;

/// Router operators, each an IMASM word over the twelve marks.
pub const OP_SHAPE:    &str = "⊢∈≻⊤⊣";              // construct the symmetric (root) representation
pub const OP_RESIDUE:  &str = "⊢∈≻⊤∈≺∋∋⊙⊡⊣";       // the capacity/residue representation
pub const OP_MEMBRANE: &str = "⊢∈≻⋈⊙⊤≻⋈⊥≺⋈⊞∋⊡⋈⊙⊣"; // the membrane: rho fused with the sieve
pub const OP_JUDGE:    &str = "⊢∈⊙∋⊣";              // judge: imscribe the current representation
pub const OP_REENTER:  &str = "⊢∈≺∋⊣";              // re-enter at the level the judgment exposes
pub const OP_FIX:      &str = "⊢⊙⊡⊣";              // FIX once reconstruction agrees

/// The representation the router currently holds the factorization problem in.
#[derive(Clone, PartialEq, Debug)]
pub enum Repr {
    /// Interval centred on ceil(sqrt N): the Fermat/root representation.
    Symmetric { a: u64 },
    /// Low 2-adic residue for p plus the p/q intervals: the capacity representation.
    Residue { k: u32 },
    /// The multiplicative orbit x -> x^2 + c mod N: the product-structure representation.
    Multiplicative,
}

impl Repr {
    pub fn word(&self) -> &'static str {
        match self {
            Repr::Symmetric { .. } => OP_SHAPE,
            Repr::Residue { .. }   => OP_RESIDUE,
            Repr::Multiplicative   => OP_MEMBRANE,
        }
    }
    pub fn name(&self) -> &'static str {
        match self {
            Repr::Symmetric { .. } => "symmetric",
            Repr::Residue { .. }   => "residue",
            Repr::Multiplicative   => "multiplicative",
        }
    }
}

/// Belnap judgment of a representation.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Judg { T, B, N, F }

impl Judg {
    pub fn name(self) -> &'static str {
        match self { Judg::T => "T", Judg::B => "B", Judg::N => "N", Judg::F => "F" }
    }
}

/// One recorded transition: representation -> judgment -> (its word).
#[derive(Clone)]
pub struct Step {
    pub repr: Repr,
    pub judg: Judg,
    pub word: &'static str,
    pub note: String,
}

pub fn words() -> String {
    format!("SHAPE={OP_SHAPE} RESIDUE={OP_RESIDUE} MEMBRANE={OP_MEMBRANE} JUDGE={OP_JUDGE} REENTER={OP_REENTER} FIX={OP_FIX}")
}

// ---- number theory for the multiplicative representation ----
fn gcd(mut a: u64, mut b: u64) -> u64 { while b != 0 { let t = a % b; a = b; b = t; } a }

pub fn is_prime_u64(n: u64) -> bool {
    if n < 2 { return false; }
    for p in [2u64, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37] { if n % p == 0 { return n == p; } }
    let mut d = n - 1; let mut r = 0u32;
    while d & 1 == 0 { d >>= 1; r += 1; }
    'w: for &a in &[2u64, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37] {
        let mut x = { let mut b = a % n; let mut e = d; let mut res = 1u64; while e > 0 { if e & 1 == 1 { res = (res as u128 * b as u128 % n as u128) as u64; } b = (b as u128 * b as u128 % n as u128) as u64; e >>= 1; } res };
        if x == 1 || x == n - 1 { continue; }
        for _ in 0..r - 1 { x = (x as u128 * x as u128 % n as u128) as u64; if x == n - 1 { continue 'w; } }
        return false;
    }
    true
}

/// Pollard-Brent rho: the multiplicative-orbit representation of the problem.
/// x -> x^2 + c (mod N); a collision exposes the multiplicative distinction.
pub fn pollard_rho(n: u64) -> Option<u64> {
    if n % 2 == 0 { return Some(2); }
    if is_prime_u64(n) { return None; }
    let c: u64 = 1;
    let f = |x: u64| ((x as u128 * x as u128 + c as u128) % n as u128) as u64;
    let (mut y, mut r, mut q, mut g, mut x, mut ys) = (2u64, 1u64, 1u64, 1u64, 0u64, 0u64);
    let m: u64 = 128;
    while g == 1 {
        x = y;
        for _ in 0..r { y = f(y); }
        let mut k = 0u64;
        while k < r && g == 1 {
            ys = y;
            for _ in 0..m.min(r - k) { y = f(y); q = (q as u128 * x.abs_diff(y) as u128 % n as u128) as u64; }
            g = gcd(q, n);
            k += m;
        }
        r = r.saturating_mul(2);
        if r > 1 << 40 { break; }
    }
    if g == n { let mut s = ys; loop { s = f(s); g = gcd(x.abs_diff(s), n); if g > 1 || s == x { break; } } }
    if g > 1 && g < n { Some(g) } else { None }
}

/// Judge a representation: return its Belnap value, any factor it hands, and a note.
pub fn judge(n: u64, repr: &Repr) -> (Judg, Option<(u64, u64)>, String) {
    match repr {
        Repr::Symmetric { .. } => {
            if n % 2 == 0 { return (Judg::T, Some((2, n / 2)), String::from("2-part closes")); }
            if let Some((p, q, i)) = fermat_frontier(n) {
                return (Judg::T, Some((p, q)), format!("frontier closes at step {i}"));
            }
            if is_prime_u64(n) { return (Judg::N, None, String::from("prime: no distinction to find")); }
            (Judg::B, None, String::from("frontier holds the fork: factors not near the root"))
        }
        Repr::Residue { k } => {
            // the capacity representation keeps the fork: root capacity ~ sqrt(N)/2, never 1
            let cap = match crate::nested_frame::root_frame(n) { Some(f) => f.live_capacity(), None => 0 };
            if cap == 1 { (Judg::T, None, format!("residue k={k}: single completion")) }
            else { (Judg::B, None, format!("residue k={k}: capacity {cap} — fork open, representation does not expose the factor")) }
        }
        Repr::Multiplicative => {
            if is_prime_u64(n) { return (Judg::N, None, String::from("prime in orbit: no distinction")); }
            match pollard_rho(n) {
                Some(g) if g > 1 => (Judg::T, Some((g, n / g)), format!("orbit collision hands {g}")),
                _ => (Judg::N, None, String::from("orbit found no distinction")),
            }
        }
    }
}

/// Re-enter: transform the representation at the level the judgment exposes.
pub fn reenter(_n: u64, repr: &Repr, judg: Judg) -> Repr {
    match (repr, judg) {
        (Repr::Symmetric { .. }, Judg::B) => Repr::Residue { k: 1 },
        (Repr::Residue { .. }, Judg::B)   => Repr::Multiplicative,
        (Repr::Symmetric { .. }, Judg::F) => Repr::Multiplicative,   // malformed -> rewrite the representation
        (Repr::Multiplicative, _)         => Repr::Multiplicative,
        _ => Repr::Multiplicative,
    }
}

/// The resident trajectory: N -> representation -> judge -> re-enter -> ... -> FIX.
/// Returns the ordered factor pair (if the trajectory closes) and every step.
pub fn route(n: u64) -> (Option<(u64, u64)>, Vec<Step>) {
    let mut traj: Vec<Step> = Vec::new();
    let mut repr = Repr::Symmetric { a: isqrt_u64(n) + 1 };
    for _ in 0..6 {
        let (j, found, note) = judge(n, &repr);
        traj.push(Step { repr: repr.clone(), judg: j, word: repr.word(), note });
        if j == Judg::T {
            if let Some((p, q)) = found {
                if p > 1 && q > 1 && p * q == n {
                    let (a, b) = if p <= q { (p, q) } else { (q, p) };
                    return (Some((a, b)), traj);
                }
            }
        }
        if j == Judg::N { break; }
        repr = reenter(n, &repr, j);
    }
    (None, traj)
}

/// The trajectory's IMASM word sequence — representation, judgment, re-entry, FIX.
pub fn trajectory_word(traj: &[Step]) -> String {
    let mut parts: Vec<String> = Vec::new();
    for s in traj { parts.push(format!("{}({})", s.repr.name(), s.judg.name())); }
    parts.push(String::from("FIX"));
    format!("{}  ::  {}", parts.join(" -> "), traj.last().map(|s| s.word).unwrap_or(OP_FIX))
}
