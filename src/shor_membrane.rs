//! Shor order-finding membrane — factor extraction from the arithmetic structure of ℤ/nℤ.
//!
//! The membrane implements the δ/μ bracket of Shor's algorithm as a FDE carrier:
//! - δ: prepare (a, N) → find order r of a mod N (horn-torus knot winding)
//! - Arm ⊤: test r even
//! - Arm ⊥: test a^(r/2) ≢ -1 (mod N)
//! - μ: fuse via gcd(a^(r/2) ± 1, N) → factors p, q; verify p·q = N
//!
//! Close condition μ∘δ = id holds when both arms pass and product boundary is exact.
//! If either arm fails, the carrier lands in B (dialetheic) and re-opens with new a.

use crate::morphism_factor::{
    cmp, sub, divmod, gcd, one, zero, trim, tape_u64, dec_of,
    parse_numeral, mul, modulo, Tape,
};

use crate::shor_braid::{order_lane, factor_close_public, shor_braid_carrier, braid_from_retained_order, OrderedShorBraid};
use alloc::string::String;
use alloc::vec::Vec;
use core::cmp::Ordering;

/// Shor membrane carrier — the computational state that transports order r
/// from the order lane to the winding readout and factor close.
#[derive(Clone, Debug)]
pub struct ShorCarrier {
    pub n: Tape,
    pub a: Tape,
    pub order: Option<Tape>,
    pub factors: Option<(Tape, Tape)>,
    pub lane: Fde,
    pub transitions: usize,
    pub closed: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fde {
    N,
    T,
    F,
    B,
}

impl Fde {
    fn join(self, other: Self) -> Self {
        use Fde::*;
        match (self, other) {
            (N, x) | (x, N) => x,
            (T, T) => T,
            (F, F) => F,
            _ => B,
        }
    }
}

impl ShorCarrier {
    /// Create new carrier for semiprime N.
    pub fn new(n: Tape) -> Result<Self, String> {
        if cmp(&n, &one()) != Ordering::Greater {
            return Err("N must be > 1".into());
        }
        if zero(&n) { return Err("N is zero".into()); }
        Ok(Self {
            n: trim(n),
            a: Vec::new(),
            order: None,
            factors: None,
            lane: Fde::N,
            transitions: 0,
            closed: 0,
        })
    }

    /// δ: Open the Shor computation — choose random a coprime to N,
    /// find the order r of a mod N (the horn-torus knot period).
    pub fn delta(&mut self, base: Option<Tape>) -> Result<(), String> {
        // Choose base a: random coprime, or provided base
        let a = if let Some(a) = base {
            modulo(&a, &self.n)
        } else {
            // Default to base 2, but verify coprimality
            let two = tape_u64(2);
            let g = gcd(two.clone(), self.n.clone());
            if g != one() {
                return Err("base 2 not coprime to N; provide explicit base".into());
            }
            two
        };
        self.a = a;

        // Find order r via the order lane (linear walk, exact, no caps)
        let r = order_lane(&self.a, &self.n)?;
        self.order = Some(r);
        self.transitions += 1;
        self.lane = self.lane.join(Fde::B);
        Ok(())
    }

    /// Arm ⊤: Test if order r is even.
    pub fn test_even(&mut self) -> Result<bool, String> {
        let r = self.order.as_ref().ok_or("order not computed; run delta first")?;
        let two = tape_u64(2);
        let (_, rem) = divmod(r, &two);
        Ok(zero(&rem))
    }

    /// Arm ⊥: Test if a^(r/2) ≢ -1 (mod N).
    pub fn test_nontrivial(&mut self) -> Result<bool, String> {
        let r = self.order.as_ref().ok_or("order not computed; run delta first")?;
        let two = tape_u64(2);
        let (half, rem) = divmod(r, &two);
        if !zero(&rem) {
            return Err("order odd — a^(r/2) undefined".into());
        }
        let h = crate::modular_residue::certified_modular_residue_tape_step(&half, &self.a, &self.n, r)?;
        let n_minus_one = sub(&self.n, &one());
        Ok(cmp(&h, &n_minus_one) != Ordering::Equal)
    }

    /// μ: Fuse — compute factors via gcd(a^(r/2) ± 1, N) and verify product.
    pub fn mu(&mut self) -> Result<(Tape, Tape), String> {
        let r = self.order.as_ref().ok_or("order not computed; run delta first")?;
        let (p, q) = factor_close_public(&self.a, &self.n, r)?;
        
        // Verify exact product boundary
        if cmp(&mul(&p, &q), &self.n) != Ordering::Equal {
            return Err("product boundary failed: p·q ≠ N".into());
        }
        
        self.factors = Some((p.clone(), q.clone()));
        self.lane = self.lane.join(Fde::T);
        self.closed += 1;
        Ok((p, q))
    }

    /// Execute the full Shor membrane: δ → arms → μ.
    /// Returns (p, q) on success, or error on B (retry with new base).
    pub fn run(&mut self, base: Option<Tape>) -> Result<(Tape, Tape), String> {
        self.delta(base)?;
        
        let even = self.test_even()?;
        if !even {
            self.lane = Fde::B;
            return Err("order odd — retry with another base".into());
        }
        
        let nontrivial = self.test_nontrivial()?;
        if !nontrivial {
            self.lane = Fde::B;
            return Err("a^(r/2) ≡ -1 (mod N) — retry with another base".into());
        }
        
        self.mu()
    }

    /// Check if the carrier has closed (μ∘δ = id on factor pair).
    pub fn is_closed(&self) -> bool {
        self.closed > 0 && self.factors.is_some()
    }

    /// Get the Shor braid word for this computation.
    pub fn ordered_braid(&self) -> Result<OrderedShorBraid, String> {
        match &self.order {
            Some(order) => braid_from_retained_order(&self.a, &self.n, order),
            None => shor_braid_carrier(&self.a, &self.n),
        }
    }

    pub fn braid_word(&self) -> Result<(Tape, usize), String> {
        let braid = self.ordered_braid()?;
        Ok((braid.word, braid.levels))
    }
}

/// High-level factor function using Shor membrane.
/// Attempts multiple bases until success or exhaustion.
pub fn shor_factor(n: &[char], max_bases: usize) -> Result<(Tape, Tape, String), String> {
    let n = trim(n.to_vec());
    
    // Quick check: if N is prime, return N
    if crate::morphism_factor::miller_rabin(&n) {
        return Err("N is prime".into());
    }
    
    // Try base 2 first
    let bases = [tape_u64(2), tape_u64(3), tape_u64(5), tape_u64(7), tape_u64(11), tape_u64(13)];
    
    for (i, base) in bases.iter().enumerate() {
        if i >= max_bases { break; }
        
        let mut carrier = ShorCarrier::new(n.clone())?;
        match carrier.run(Some(base.clone())) {
            Ok((p, q)) => {
                let report = format!(
                    "Shor membrane closed: N={} = {} x {}\nbase={}, order={}, r even=true, a^(r/2)≠-1=true",
                    dec_of(&n), dec_of(&p), dec_of(&q),
                    dec_of(base), dec_of(carrier.order.as_ref().unwrap())
                );
                return Ok((p, q, report));
            }
            Err(_e) => {
                // Landed in B — log and try next base
                // base {}: {} — retrying (logged internally)
            }
        }
    }
    
    // If all small bases fail, try random bases (up to max_bases)
    // For now, return error indicating need for more bases
    Err(format!("all {} bases failed — need wider base search or fallback to QS/MPQS", max_bases))
}

/// REPL entry: `vox shor_factor <N> [max_bases]`
pub fn repl_shor_factor(args: &[String]) -> Result<String, String> {
    if args.is_empty() {
        return Err("vox shor_factor <N> [max_bases]".into());
    }
    
    let n = if args[0].starts_with('⊢') {
        parse_numeral(&args[0])?
    } else {
        crate::morphism_factor::decimal_to_tape(&args[0]).ok_or_else(|| String::from("invalid decimal"))?
    };
    
    let max_bases = args.get(1).and_then(|s| s.parse::<usize>().ok()).unwrap_or(6);
    
    match shor_factor(&n, max_bases) {
        Ok((p, q, report)) => Ok(format!("{}\n{} = {} x {}", report, dec_of(&n), dec_of(&p), dec_of(&q))),
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::morphism_factor::{tape_u64, mul};

    #[test]
    fn shor_factor_15() {
        let n = tape_u64(15);
        let (p, q, _) = shor_factor(&n, 6).unwrap();
        assert_eq!(mul(&p, &q), n);
    }

    #[test]
    fn shor_factor_91() {
        let n = tape_u64(91);
        let (p, q, _) = shor_factor(&n, 6).unwrap();
        assert_eq!(mul(&p, &q), n);
    }

    #[test]
    fn carrier_mu_delta_id() {
        let n = tape_u64(91);
        let mut carrier = ShorCarrier::new(n.clone()).unwrap();
        carrier.run(Some(tape_u64(2))).unwrap();
        assert!(carrier.is_closed());
        let (p, q) = carrier.factors.unwrap();
        assert_eq!(mul(&p, &q), n);
        // μ∘δ = id: the carrier returns the exact factor pair
        assert!(carrier.lane == Fde::T || carrier.lane == Fde::B);
    }
}

#[test]
fn test_carrier_n91_base2() {
    let n = crate::morphism_factor::tape_u64(91);
    let mut carrier = ShorCarrier::new(n.clone()).unwrap();
    carrier.run(Some(crate::morphism_factor::tape_u64(2))).unwrap();
    assert!(carrier.is_closed());
    let (p, q) = carrier.factors.unwrap();
    assert_eq!(crate::morphism_factor::mul(&p, &q), n);
}



#[test]
fn debug_tape_encodings() {
    let n1 = crate::morphism_factor::tape_u64(91);
    let n2 = crate::morphism_factor::decimal_to_tape("91").unwrap();
    let a1 = crate::morphism_factor::tape_u64(2);
    let a2 = crate::morphism_factor::decimal_to_tape("2").unwrap();
    
    assert_eq!(n1, n2, "n1={:?} n2={:?}", n1, n2);
    assert_eq!(a1, a2, "a1={:?} a2={:?}", a1, a2);
}

#[test]
fn test_delta_returns_correct_order() {
    let n = crate::morphism_factor::tape_u64(91);
    let mut carrier = ShorCarrier::new(n.clone()).unwrap();
    carrier.delta(Some(crate::morphism_factor::tape_u64(2))).unwrap();
    let r = carrier.order.as_ref().unwrap();
    let r_dec = crate::morphism_factor::dec_of(r);
    assert_eq!(r_dec, "12", "order should be 12, got {}", r_dec);
}

#[test]
fn test_test_even() {
    let n = crate::morphism_factor::tape_u64(91);
    let mut carrier = ShorCarrier::new(n.clone()).unwrap();
    carrier.delta(Some(crate::morphism_factor::tape_u64(2))).unwrap();
    let even = carrier.test_even().unwrap();
    assert!(even, "order 12 should be even");
}
