//! perfect_membrane.rs — the membrane as matched circuitry, not as search.
//!
//! A membrane closes when μ∘δ = id on the transformed register: transform, split,
//! fuse, and restore. Here every stack level carries its own transformed tape and
//! local closure witness. The child return is checked against the parent's lane;
//! those adjacent equalities compose to the root register.
//!
//! The tower has depth n >= 2 and no maximum. Each level forks to one deeper
//! register, links that adjacent level, and fuses on the return rail. The closure
//! auditor `vox::verdict` reads the symbolic word; `closure_witness` independently
//! evaluates μ∘δ on the actual transformed tapes at every level.

use crate::morphism_factor::{cmp, dec_of, mul, trim};
use crate::vox::{
    verdict, AFWD, AREV, CLINK, ENGAGR, EVALF, EVALT, FFUSE, FSPLIT, IFIX, IMSCRIB, TANCH, VINIT,
};
use alloc::format;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

pub type Word = Vec<char>;

// ---- executable delta / mu over the bit-tape ----
// The tape is little-endian bits: index i is bit i, EVALF = 1, EVALT = 0.

/// delta: deinterlace into two width-carrying register lanes. Keep high zero cells:
/// they are part of the child register's allocated width even when its value is 0.
fn delta(v: &[char]) -> (Word, Word) {
    let mut a = Word::new();
    let mut b = Word::new();
    for (i, &bit) in v.iter().enumerate() {
        if i % 2 == 0 {
            a.push(bit);
        } else {
            b.push(bit);
        }
    }
    (a, b)
}

/// Width-preserving μ for a transformed register. Canonical numeral trimming can
/// remove high zero cells from either arm, so the register width travels with the
/// frame and is restored explicitly at the fuse.
fn mu_width(a: &[char], b: &[char], width: usize) -> Word {
    let mut out = Vec::with_capacity(width);
    for j in 0..width.div_ceil(2) {
        out.push(*a.get(j).unwrap_or(&EVALT));
        if out.len() < width {
            out.push(*b.get(j).unwrap_or(&EVALT));
        }
    }
    out
}

/// Run a value through the nested membrane and report each transformed register's
/// local split/fuse witness and its composition back to the root.
pub fn run(value: &[char], depth: usize) -> String {
    let v = trim(value.to_vec());
    let audit = transit(&v, depth);
    format!(
        "value in: {}\n{}value out: {}\n  composed μ∘δ = id : {}\n  core lane product: {}\n",
        dec_of(&v),
        audit.trace,
        dec_of(&audit.recovered),
        if audit.closed {
            "CLOSED (every level returned to root)"
        } else {
            "LEAK"
        },
        dec_of(&audit.product)
    )
}

/// Witness for one register in the nested stack. The child return is checked
/// against lane0 before this level fuses its two arms.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LevelClosureWitness {
    pub level: usize,
    pub source: Word,
    pub transformed: Word,
    pub lane0: Word,
    pub lane1: Word,
    pub child_returned: Word,
    pub fused: Word,
    pub restored: Word,
    pub local_identity: bool,
    pub child_return_identity: bool,
    pub child_chain_closed: bool,
    pub composed_identity: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClosureWitness {
    pub levels: Vec<LevelClosureWitness>,
    pub recovered: Word,
    pub product: Word,
    pub closed: bool,
    pub trace: String,
}

/// Apply a cellwise involution to make each level's transformed register explicit.
fn transform(register: &[char]) -> Word {
    register
        .iter()
        .map(|&cell| if cell == EVALF { EVALT } else { EVALF })
        .collect()
}

fn trace_register(register: &[char]) -> String {
    if register.len() <= 512 {
        dec_of(register)
    } else {
        format!("<{}-cell tape>", register.len())
    }
}

/// The transformed object at the core: advance, engage the paradox, imscribe. This
/// is the "transformed" in "mu∘delta = id over a transformed object" — without a
/// real transform the closure is trivial.
fn core() -> Word {
    vec![AFWD, EVALT, AREV, EVALF, ENGAGR, IMSCRIB]
}

/// Every level contains its own transformed work region. The nested fuses then
/// compose the local closures from the deepest register back to the root.
pub fn perfect_membrane(depth: usize) -> Word {
    let n = depth.max(2);
    let mut w = vec![VINIT];
    for _ in 0..n {
        w.push(FSPLIT);
        w.push(CLINK);
        w.extend(core());
    }
    for _ in 0..n {
        w.push(FFUSE);
    }
    w.push(IFIX);
    w.push(TANCH);
    w
}

/// Read the closure of a depth-n perfect membrane with the instrument: its verdict
/// mark, and the fork/fuse surplus (zero when every delta has its mu).
pub fn report(depth: usize) -> (char, i32, Word) {
    let w = perfect_membrane(depth);
    let of = crate::vox::open_forks(&w);
    (verdict(&w), of.surplus, w)
}

// ---- the operculum: a membrane is a vessel with one puncture ----
//
// A membrane's entry wound is its exit wound: the frame that opens the split is
// the frame that closes the fuse, one opening, not an in-port and an out-port. So
// you do not pass a value through it. You open the operculum, drop the value into
// the sealed interior, close it, run the circuit while it is shut, then open the
// same operculum and lift the result out. The lifecycle below enforces that: you
// cannot run unsealed, cannot deposit once sealed, cannot extract before the run.

/// State of the operculum, the membrane's single lid.
#[derive(PartialEq)]
pub enum Lid {
    Open,
    Sealed,
    Spent,
}

/// A depth-n perfect membrane as a loadable vessel.
pub struct Operculum {
    depth: usize,
    lid: Lid,
    payload: Option<Word>,
    recovered: Option<Word>,
    product: Option<Word>,
    audit: Option<ClosureWitness>,
}

impl Operculum {
    /// Open a fresh operculum at the given depth, lid up, interior empty.
    pub fn open(depth: usize) -> Self {
        Operculum {
            depth: depth.max(2),
            lid: Lid::Open,
            payload: None,
            recovered: None,
            product: None,
            audit: None,
        }
    }

    /// Drop a value into the interior through the open lid.
    pub fn deposit(&mut self, value: &[char]) -> Result<(), &'static str> {
        if self.lid != Lid::Open {
            return Err("operculum is not open");
        }
        self.payload = Some(trim(value.to_vec()));
        Ok(())
    }

    /// Seal the lid. Nothing enters or leaves while it is shut.
    pub fn seal(&mut self) -> Result<(), &'static str> {
        if self.lid != Lid::Open {
            return Err("operculum is not open");
        }
        if self.payload.is_none() {
            return Err("nothing deposited to seal");
        }
        self.lid = Lid::Sealed;
        Ok(())
    }

    /// Run the closed circuit while sealed and retain the per-register witnesses.
    pub fn run(&mut self) -> Result<(), &'static str> {
        if self.lid != Lid::Sealed {
            return Err("operculum must be sealed before it runs");
        }
        let v = self.payload.as_ref().unwrap();
        let audit = transit(v, self.depth);
        self.recovered = Some(audit.recovered.clone());
        self.product = Some(audit.product.clone());
        self.audit = Some(audit);
        Ok(())
    }

    /// Open the lid again and lift out what the run left: the recovered value (the
    /// entry, returned through the one puncture) and the core transform's product.
    pub fn extract(&mut self) -> Result<(Word, Word), &'static str> {
        if self.lid != Lid::Sealed || self.recovered.is_none() {
            return Err("nothing to extract; seal and run first");
        }
        self.lid = Lid::Spent;
        Ok((self.recovered.take().unwrap(), self.product.take().unwrap()))
    }
}

/// Build and compose one μ∘δ witness per transformed register. Each level applies
/// the same involution, splits the transformed tape, receives the child's returned
/// lane, fuses the two lanes, then applies the involution again to restore its input.
pub fn closure_witness(value: &[char], depth: usize) -> ClosureWitness {
    let n = depth.max(2);
    let v = trim(value.to_vec());
    let mut frames: Vec<(usize, Word, Word, Word, Word)> = Vec::new();
    let mut cur = v.clone();
    for level in 1..=n {
        let transformed = transform(&cur);
        let (a, b) = delta(&transformed);
        frames.push((level, cur, transformed, a.clone(), b));
        cur = a;
    }
    let deepest = frames.last().unwrap();
    let product = mul(&deepest.3, &deepest.4);
    let mut levels = Vec::with_capacity(n);
    let mut child_chain_closed = true;
    for (level, source, transformed, lane0, lane1) in frames.into_iter().rev() {
        let child_returned = cur.clone();
        let child_return_identity = child_returned == lane0;
        let fused = mu_width(&child_returned, &lane1, transformed.len());
        let local_identity = fused == transformed;
        let restored = transform(&fused);
        let source_identity = restored == source;
        let this_chain_closed =
            child_chain_closed && child_return_identity && local_identity && source_identity;
        levels.push(LevelClosureWitness {
            level,
            source,
            transformed,
            lane0,
            lane1,
            child_returned,
            fused: fused.clone(),
            restored: restored.clone(),
            local_identity,
            child_return_identity,
            child_chain_closed,
            composed_identity: this_chain_closed,
        });
        cur = restored;
        child_chain_closed = this_chain_closed;
    }
    levels.reverse();
    let closed = child_chain_closed && cur == v;
    let mut trace = String::new();
    for witness in &levels {
        trace.push_str(&format!(
            "  L{} width={} register={} transformed={} δ=({}, {}) arm-widths=({}, {}) child-return={} μδ={} local={} child={} composed={} restored={}\n",
            witness.level,
            witness.source.len(),
            trace_register(&witness.source),
            trace_register(&witness.transformed),
            trace_register(&witness.lane0),
            trace_register(&witness.lane1),
            witness.lane0.len(),
            witness.lane1.len(),
            trace_register(&witness.child_returned),
            trace_register(&witness.fused),
            if witness.local_identity { "id" } else { "LEAK" },
            if witness.child_chain_closed { "closed" } else { "LEAK" },
            if witness.composed_identity { "id" } else { "LEAK" },
            trace_register(&witness.restored)
        ));
    }
    ClosureWitness {
        levels,
        recovered: cur,
        product,
        closed,
        trace,
    }
}

fn transit(value: &[char], depth: usize) -> ClosureWitness {
    closure_witness(value, depth)
}

/// Walk the operculum lifecycle on a value and narrate each step, for the CLI.
pub fn operculum_demo(value: &[char], depth: usize) -> String {
    let mut out = String::new();
    let mut op = Operculum::open(depth);
    out.push_str(&format!("open operculum (depth {depth})\n"));
    op.deposit(value).unwrap();
    out.push_str(&format!("deposit: {}\n", dec_of(&trim(value.to_vec()))));
    op.seal().unwrap();
    out.push_str("seal\n");
    op.run().unwrap();
    let audit_trace = op.audit.as_ref().unwrap().trace.clone();
    let audit_closed = op.audit.as_ref().unwrap().closed;
    out.push_str("run (sealed)\n");
    out.push_str(&audit_trace);
    let (recovered, product) = op.extract().unwrap();
    let closed =
        audit_closed && cmp(&recovered, &trim(value.to_vec())) == core::cmp::Ordering::Equal;
    out.push_str(&format!(
        "open operculum, extract: value {}  transform {}\n  entry wound = exit wound : {}\n",
        dec_of(&recovered),
        dec_of(&product),
        if closed {
            "CLOSED (recovered through the one puncture)"
        } else {
            "LEAK"
        }
    ));
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::morphism_factor::decimal_to_tape;

    #[test]
    fn operculum_recovers_through_one_puncture() {
        let v = decimal_to_tape("1234567890123456789").unwrap();
        let mut op = Operculum::open(4);
        assert!(op.run().is_err(), "cannot run before sealing");
        op.deposit(&v).unwrap();
        assert!(op.extract().is_err(), "cannot extract before running");
        op.seal().unwrap();
        assert!(op.deposit(&v).is_err(), "cannot deposit once sealed");
        op.run().unwrap();
        let (recovered, _product) = op.extract().unwrap();
        assert_eq!(
            trim(recovered),
            trim(v),
            "the entry must return as the exit"
        );
    }

    #[test]
    fn transformed_register_witnesses_compose_to_root_at_every_depth() {
        let value = decimal_to_tape("1234567890123456789").unwrap();
        for n in 2..=32 {
            let (v, surplus, _) = report(n);
            assert_eq!(surplus, 0, "depth {n}: forks and fuses must match");
            assert_eq!(v, 'T', "depth {n}: perfect membrane must close with work");
            let audit = closure_witness(&value, n);
            assert!(
                audit.closed,
                "depth {n}: composed root closure\n{}",
                audit.trace
            );
            assert_eq!(audit.levels.len(), n);
            assert_eq!(trim(audit.recovered), trim(value.clone()));
            for witness in audit.levels {
                assert!(
                    witness.local_identity,
                    "depth {n}, level {}: μδ=id",
                    witness.level
                );
                assert!(
                    witness.child_return_identity,
                    "depth {n}, level {}: child returns to lane",
                    witness.level
                );
                assert!(
                    witness.child_chain_closed,
                    "depth {n}, level {}: child chain composes",
                    witness.level
                );
                assert!(
                    witness.composed_identity,
                    "depth {n}, level {}: composed μδ=id",
                    witness.level
                );
                assert_eq!(witness.fused, witness.transformed);
                assert_eq!(witness.restored, witness.source);
            }
        }
    }

    #[test]
    fn width_carrying_closure_scales_with_larger_registers() {
        for width in [
            128usize, 256, 512, 1024, 2048, 4096, 8192, 16_384, 32_768, 65_536, 131_072, 262_144,
            1_048_576, 4_194_304,
        ] {
            let mut depth = 1usize;
            let mut child_width = width;
            while child_width > 1 {
                child_width = child_width.div_ceil(2);
                depth += 1;
            }
            let value = vec![EVALF; width];
            let audit = closure_witness(&value, depth);
            assert!(audit.closed, "width {width}: root closure");
            assert_eq!(audit.levels.len(), depth);
            let mut expected_width = width;
            for witness in audit.levels {
                assert_eq!(
                    witness.source.len(),
                    expected_width,
                    "width {width}, level {}",
                    witness.level
                );
                assert_eq!(
                    witness.transformed.len(),
                    expected_width,
                    "width {width}, level {}",
                    witness.level
                );
                assert!(
                    witness.local_identity,
                    "width {width}, level {}",
                    witness.level
                );
                assert!(
                    witness.child_return_identity,
                    "width {width}, level {}",
                    witness.level
                );
                assert!(
                    witness.composed_identity,
                    "width {width}, level {}",
                    witness.level
                );
                expected_width = expected_width.div_ceil(2);
            }
        }
    }
}
