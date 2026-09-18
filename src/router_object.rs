//! router_object.rs — the router as an IMASM object. No host-side policy.
//!
//! The ladder Symmetric → Residue → Multiplicative → FIX is not Rust control flow:
//! it is a list of `RouteClause`s serialized into ONE IMASM word. The control law is
//!
//!     (object, router) -> JUDGE -> imscribe(judgment) -> apply router-word
//!       -> new (object, router)
//!
//! and `router` can itself become `object`: after a trajectory closes, the router
//! word is judged and rewritten, still as an IMASM word. The rewrite relation is
//! encoded in IMASM, never in a Rust `match Belnap { .. }`.
//!
//! Serialization (a decodable IMASM object, the twelve marks only):
//! ```text
//!   router := clause*
//!   clause := ∈ J S ∈ T ∋ N ∋
//!     J  judgment    ⊤=T  ⊥=F  ⊞=B  ⊙=N
//!     S  source repr ⊢=Symmetric ⊣=Residue ⋈=Multiplicative ⊙=any
//!     ∈ T ∋          the transform word — a nested, depth-balanced IMASM word
//!     N  next repr   ⊢  ⊣  ⋈  or  ⊡ = FIX (terminal)
//! ```
//! Depth tracking makes the nesting unambiguous, so `D(E(R)) = R` and
//! `E(D(W)) = W` hold exactly.

use alloc::string::String;
use alloc::vec::Vec;

pub const J_T: char = '⊤';
pub const J_F: char = '⊥';
pub const J_B: char = '⊞';
pub const J_N: char = '⊙';

pub const R_SYM: char = '⊢';
pub const R_RES: char = '⊣';
pub const R_MUL: char = '⋈';
pub const R_ANY: char = '⊙';
pub const R_FIX: char = '⊡';

pub const OPEN: char = '∈';
pub const CLOSE: char = '∋';

/// A representation tag.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ReprTag { Symmetric, Residue, Multiplicative }

impl ReprTag {
    pub fn glyph(self) -> char {
        match self { ReprTag::Symmetric => R_SYM, ReprTag::Residue => R_RES, ReprTag::Multiplicative => R_MUL }
    }
    pub fn from_glyph(g: char) -> Option<ReprTag> {
        match g { R_SYM => Some(ReprTag::Symmetric), R_RES => Some(ReprTag::Residue), R_MUL => Some(ReprTag::Multiplicative), _ => None }
    }
    pub fn name(self) -> &'static str {
        match self { ReprTag::Symmetric => "symmetric", ReprTag::Residue => "residue", ReprTag::Multiplicative => "multiplicative" }
    }
}

/// Belnap judgment.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Belnap { T, F, B, N }

impl Belnap {
    pub fn glyph(self) -> char {
        match self { Belnap::T => J_T, Belnap::F => J_F, Belnap::B => J_B, Belnap::N => J_N }
    }
    pub fn from_glyph(g: char) -> Option<Belnap> {
        match g { J_T => Some(Belnap::T), J_F => Some(Belnap::F), J_B => Some(Belnap::B), J_N => Some(Belnap::N), _ => None }
    }
    pub fn name(self) -> &'static str {
        match self { Belnap::T => "T", Belnap::F => "F", Belnap::B => "B", Belnap::N => "N" }
    }
}

/// One routing clause: (judgment × source_repr) → (transform_word, next_repr).
#[derive(Clone)]
pub struct RouteClause {
    pub judgment: Belnap,
    pub source: Option<ReprTag>,       // None = wildcard (any source)
    pub transform_word: String,        // the IMASM object that performs the transform
    pub next: Option<ReprTag>,         // None = FIX (terminal)
}

/// The router object: a word plus its decoded clauses. The word is the object.
#[derive(Clone)]
pub struct RouterObject {
    pub word: String,
    pub transitions: Vec<RouteClause>,
}

/// Encode one clause: ∈ J S ∈ <len8> ∃ <payload> N ∃
/// The transform payload is a raw IMASM word (it may contain ∈/∃, and need not
/// be balanced), so its length travels explicitly as 8 marks over {⊤=1, ⊥=0}. The
/// payload is then read BY COUNT. This makes the nesting unambiguous for ANY word, so
/// E(D(R)) == R holds exactly — the original delimiter nest could not represent an
/// unbalanced frame word and mis-split on `⊢∈≻⊤⊣`.
pub const BIT1: char = '⊤';
pub const BIT0: char = '⊥';

pub fn encode_clause(c: &RouteClause) -> String {
    let mut s = String::new();
    s.push(OPEN);
    s.push(c.judgment.glyph());
    s.push(match c.source { Some(t) => t.glyph(), None => R_ANY });
    s.push(OPEN);
    let n = c.transform_word.chars().count();
    for bit in (0..8).rev() { s.push(if (n >> bit) & 1 == 1 { BIT1 } else { BIT0 }); }
    s.push(CLOSE);
    s.push_str(&c.transform_word);
    s.push(match c.next { Some(t) => t.glyph(), None => R_FIX });
    s.push(CLOSE);
    s
}

/// Decode one clause at `i`, advancing `i`.
pub fn decode_clause(chars: &[char], i: &mut usize) -> Option<RouteClause> {
    if *chars.get(*i)? != OPEN { return None; }
    *i += 1;
    let judgment = Belnap::from_glyph(*chars.get(*i)?)?; *i += 1;
    let sg = *chars.get(*i)?; *i += 1;
    let source = if sg == R_ANY { None } else { Some(ReprTag::from_glyph(sg)?) };
    if *chars.get(*i)? != OPEN { return None; }
    *i += 1;
    let mut len = 0usize;
    for _ in 0..8 {
        let b = *chars.get(*i)?; *i += 1;
        len = (len << 1) | match b { BIT1 => 1usize, BIT0 => 0usize, _ => return None };
    }
    if *chars.get(*i)? != CLOSE { return None; }
    *i += 1;
    let mut transform_word = String::new();
    for _ in 0..len { transform_word.push(*chars.get(*i)?); *i += 1; }
    let ng = *chars.get(*i)?; *i += 1;
    let next = if ng == R_FIX { None } else { Some(ReprTag::from_glyph(ng)?) };
    if *chars.get(*i)? != CLOSE { return None; }
    *i += 1;
    Some(RouteClause { judgment, source, transform_word, next })
}

impl RouterObject {
    /// The canonical router: the clause list IS the ladder, no Rust control flow.
    pub fn initial() -> RouterObject {
        let to_sym = String::from("⊢∈≻⊤⊣");
        let to_res = String::from("⊢∈≻⊤∈≺∋∋⊙⊡⊣");
        let to_mul = String::from("⊢∈≻⋈⊙⊤≻⋈⊥≺⋈⊞∋⊡⋈⊙⊣");
        let fix = String::from("⊢⊙⊡⊣");
        let transitions = alloc::vec![
            // T (any source): the represented class closed -> FIX
            RouteClause { judgment: Belnap::T, source: None, transform_word: fix.clone(), next: None },
            // B @ symmetric: fork open near the root -> transform to the residue representation
            RouteClause { judgment: Belnap::B, source: Some(ReprTag::Symmetric), transform_word: to_res.clone(), next: Some(ReprTag::Residue) },
            // B @ residue: fork open, representation exposes nothing -> transform to the orbit
            RouteClause { judgment: Belnap::B, source: Some(ReprTag::Residue), transform_word: to_mul.clone(), next: Some(ReprTag::Multiplicative) },
            // B @ multiplicative: last representation -> terminal attempt
            RouteClause { judgment: Belnap::B, source: Some(ReprTag::Multiplicative), transform_word: to_mul.clone(), next: None },
            // N (any): no distinction present (prime) -> stop
            RouteClause { judgment: Belnap::N, source: None, transform_word: fix.clone(), next: None },
            // F (any): malformed composition -> rewrite the representation
            RouteClause { judgment: Belnap::F, source: None, transform_word: to_mul.clone(), next: None },
            RouteClause { judgment: Belnap::T, source: Some(ReprTag::Symmetric), transform_word: to_sym, next: Some(ReprTag::Symmetric) },
        ];
        RouterObject { word: String::new(), transitions }
    }

    /// Encode the clause list into the single router word.
    pub fn encode(&self) -> String {
        let mut s = String::new();
        for c in &self.transitions { s.push_str(&encode_clause(c)); }
        s
    }

    /// Decode a router word back into the clause list.
    pub fn decode(word: &str) -> Option<RouterObject> {
        let chars: Vec<char> = word.chars().collect();
        let mut i = 0usize;
        let mut transitions = Vec::new();
        while i < chars.len() {
            transitions.push(decode_clause(&chars, &mut i)?);
        }
        Some(RouterObject { word: word.into(), transitions })
    }

    /// E(D(R)) — the decoded router must re-encode to the same word.
    pub fn canonicalize(&mut self) {
        self.word = self.encode();
    }

    /// Apply the router word to a judgment at a source representation: the control
    /// law reads a CLAUSE, never a Rust `match` on Belnap meaning.
    pub fn apply(&self, judgment: Belnap, source: ReprTag) -> Option<&RouteClause> {
        self.transitions.iter().find(|c| c.judgment == judgment && (c.source == Some(source) || c.source.is_none()))
    }
}

use crate::meta_router::{Repr, judge as mjudge, Judg};
use crate::nested_frame::isqrt_u64;

fn rtag(r: &Repr) -> ReprTag {
    match r { Repr::Symmetric { .. } => ReprTag::Symmetric, Repr::Residue { .. } => ReprTag::Residue, Repr::Multiplicative => ReprTag::Multiplicative }
}
fn bj(j: Judg) -> Belnap {
    match j { Judg::T => Belnap::T, Judg::F => Belnap::F, Judg::B => Belnap::B, Judg::N => Belnap::N }
}
fn state_for(t: ReprTag, n: u64) -> Repr {
    match t {
        ReprTag::Symmetric => Repr::Symmetric { a: isqrt_u64(n) + 1 },
        ReprTag::Residue => Repr::Residue { k: 1 },
        ReprTag::Multiplicative => Repr::Multiplicative,
    }
}

/// One recorded transition of the meta-trajectory.
#[derive(Clone)]
pub struct TraceStep {
    pub repr: ReprTag,
    pub judgment: Belnap,
    pub applied_word: String,   // the transform word the router clause handed
    pub note: String,
    pub next: Option<ReprTag>,
    pub recognised: bool,       // did a router clause cover (judgment, repr)?
}

/// (object, router) state. `router` is an IMASM word and can become `object`.
#[derive(Clone)]
pub struct MetaState {
    pub object: Repr,
    pub router: RouterObject,
    pub judgment: Option<Belnap>,
    pub trace: Vec<TraceStep>,
}

impl MetaState {
    pub fn new(router: RouterObject, n: u64) -> MetaState {
        MetaState { object: Repr::Symmetric { a: isqrt_u64(n) + 1 }, router, judgment: None, trace: Vec::new() }
    }

    /// The control law. Reads a CLAUSE from the router word; never a Rust match on
    /// Belnap meaning. Returns Some(factor pair) on closure.
    pub fn step(&mut self, n: u64) -> Option<(u64, u64)> {
        let (j, found, note) = mjudge(n, &self.object);
        let jb = bj(j);
        let tag = rtag(&self.object);
        self.judgment = Some(jb);
        let clause = self.router.apply(jb, tag);
        let (word, next, recognised) = match clause {
            Some(c) => (c.transform_word.clone(), c.next, true),
            None => (String::from("—"), None, false),
        };
        self.trace.push(TraceStep { repr: tag, judgment: jb, applied_word: word, note, next, recognised });
        if jb == Belnap::T {
            if let Some((p, q)) = found { if p > 1 && q > 1 && p * q == n { return Some((p, q)); } }
        }
        if jb == Belnap::N { return None; }
        match next { Some(t) => self.object = state_for(t, n), None => return None }
        None
    }
}

/// Run a router to closure (or stop). Returns the factor pair and the trace.
pub fn run(router: &RouterObject, n: u64, max_steps: usize) -> (Option<(u64, u64)>, Vec<TraceStep>) {
    let mut st = MetaState::new(router.clone(), n);
    for _ in 0..max_steps { if let Some(f) = st.step(n) { return (Some(f), st.trace); } }
    (None, st.trace)
}

/// Judge the ROUTER itself from the trajectories it produced — the judge judged.
pub fn judge_router(traces: &[Vec<TraceStep>], closures: usize, corpus: usize) -> Belnap {
    let malformed = traces.iter().flatten().any(|s| !s.recognised);
    if malformed { return Belnap::F; }
    if closures == corpus { return Belnap::T; }
    if closures == 0 { return Belnap::N; }
    Belnap::B
}


/// v2 — the judge judged. A trajectory step is ALWAYS a legal composition: `recognised
/// == false` means NO CLAUSE MATCHED (the router has drawn no distinction for this
/// state), never that the word is malformed. So v2 maps every unrecognised step to N
/// (genuine no-distinction), never F. v1's `unrecognized -> F` conflates the two.
pub fn judge_router_v2(traces: &[Vec<TraceStep>], closures: usize, corpus: usize) -> Belnap {
    let unmatched = traces.iter().flatten().any(|s| !s.recognised);
    if unmatched { return Belnap::N; }
    if closures == corpus { return Belnap::T; }
    if closures == 0 { return Belnap::N; }
    Belnap::B
}


/// A router that covers every B-path to a TERMINAL but carries NO N clause, so a prime
/// terminates at a terminal B step and the verdict is N with no N clause present — the
/// only way DISTINGUISH can fire.
pub fn no_n_router() -> RouterObject {
    let to_res = String::from("⊢∈≻⊤∈≺∋∋⊙⊡⊣");
    let to_mul = String::from("⊢∈≻⊸⊙⊤≻⊸⊥≺⊸⊞∋⊡⊸⊙⊣");
    let fix = String::from("⊢⊙⊡⊣");
    let transitions = alloc::vec![
        RouteClause { judgment: Belnap::T, source: None, transform_word: fix, next: None },
        RouteClause { judgment: Belnap::B, source: Some(ReprTag::Symmetric), transform_word: to_res, next: Some(ReprTag::Residue) },
        RouteClause { judgment: Belnap::B, source: Some(ReprTag::Residue), transform_word: to_mul.clone(), next: Some(ReprTag::Multiplicative) },
        RouteClause { judgment: Belnap::B, source: Some(ReprTag::Multiplicative), transform_word: to_mul, next: None },
        RouteClause { judgment: Belnap::F, source: None, transform_word: String::from("⊢∈≻⊸⊙⊤≻⊸⊥≺⊸⊞∋⊡⊸⊙⊣"), next: None },
    ];
    let mut r = RouterObject { word: String::new(), transitions };
    r.canonicalize();
    r
}

/// A partial router: only the direct T-closure clause. It has not yet learned to
/// advance the representation, so a far-apart semiprime exposes an uncovered branch.
pub fn partial_router() -> RouterObject {
    let fix = String::from("⊢⊙⊡⊣");
    let transitions = alloc::vec![
        RouteClause { judgment: Belnap::T, source: None, transform_word: fix, next: None },
    ];
    let mut r = RouterObject { word: String::new(), transitions };
    r.canonicalize();
    r
}

/// Rewrite op words (the transform payloads of the policy object).
pub const RW_PRESERVE:     &str = "⊢⊙⊡⊣";
pub const RW_EXPOSE:       &str = "⊢∈≻⊤∈≺∋∋⊙⊡⊣";

/// The rewrite policy as an IMASM object: verdict -> rewrite-op word. The DECISION is
/// data (a clause read from this object), never a `match Belnap`.
pub fn rewrite_policy() -> RouterObject {
    let transitions = alloc::vec![
        RouteClause { judgment: Belnap::T, source: None, transform_word: String::from(RW_PRESERVE), next: None },
        RouteClause { judgment: Belnap::F, source: None, transform_word: String::from(RW_EXPOSE), next: None },
        RouteClause { judgment: Belnap::B, source: None, transform_word: String::from(RW_EXPOSE), next: None },
        RouteClause { judgment: Belnap::N, source: None, transform_word: String::from(RW_EXPOSE), next: None },
    ];
    let mut r = RouterObject { word: String::new(), transitions };
    r.canonicalize();
    r
}

/// The rewrite relation as an operation on the router WORD.
///
///   op = rewrite_policy().apply(verdict)          // DATA: which rewrite to perform
///   exposed branch -> clause pulled from `reference`   // DATA: the advance word
///
/// The remaining host code only concatenates clauses and re-canonicalizes; it never
/// branches on Belnap MEANING. Making this op a VM program (imasm_vm.rs, router word
/// as store) removes the last structural host step.
pub fn rewrite(router: &RouterObject, verdict: Belnap, traces: &[Vec<TraceStep>], reference: &RouterObject) -> RouterObject {
    let pol = rewrite_policy();
    let op = pol.apply(verdict, ReprTag::Symmetric).map(|c| c.transform_word.clone()).unwrap_or_else(|| String::from(RW_PRESERVE));
    let mut clauses = router.transitions.clone();
    if op == RW_EXPOSE {
        for t in traces {
            for s in t {
                if !s.recognised {
                    let present = clauses.iter().any(|c| c.judgment == s.judgment && c.source == Some(s.repr));
                    if !present {
                        if let Some(rc) = reference.apply(s.judgment, s.repr) { clauses.push(rc.clone()); }
                    }
                }
            }
        }
    }
    let mut r = RouterObject { word: String::new(), transitions: clauses };
    r.canonicalize();
    r
}

// ---- the mark-native trajectory: the judge is judge_g, the loop state is marks ----

use crate::judge_g::{judge_g, repr_value, found_factor,
    REPR_TAG_SYM, REPR_TAG_RES};

/// One transition, all marks: repr tag -> judgment mark -> next tag mark.
#[derive(Clone)]
pub struct GTraceStep {
    pub tag: char,
    pub judgment: char,
    pub applied_word: String,
    pub next: Option<char>,
    pub recognised: bool,
}

fn tag_param(tag: char, n: u64) -> u64 {
    match tag { REPR_TAG_SYM => isqrt_u64(n) + 1, REPR_TAG_RES => 1, _ => 0 }
}

/// The mark-native control law. The object is a tuple of marks; the judge is
/// judge_g (a GValue); the transformation is the router clause. No Judg and no Repr
/// appear in the loop's state — the clause table is the only data, and it is read,
/// never branched on by meaning.
pub fn run_g(router: &RouterObject, n: u64, max_steps: usize) -> (Option<(u64, u64)>, Vec<GTraceStep>) {
    let mut tag: char = REPR_TAG_SYM;
    let mut traj: Vec<GTraceStep> = Vec::new();
    for _ in 0..max_steps {
        let carrier = repr_value(tag, n, tag_param(tag, n));
        let jmark = judge_g(&carrier).mark0().unwrap_or('\u{22A5}');
        let clause = match (Belnap::from_glyph(jmark), ReprTag::from_glyph(tag)) {
            (Some(j), Some(t)) => router.apply(j, t),
            _ => None,
        };
        let (word, next, recognised) = match clause {
            Some(c) => (c.transform_word.clone(), c.next.map(|t| t.glyph()), true),
            None => (String::from("\u{2014}"), None, false),
        };
        traj.push(GTraceStep { tag, judgment: jmark, applied_word: word, next, recognised });
        if jmark == J_T {
            if let Some((p, q)) = found_factor(tag, n) {
                if p > 1 && q > 1 && p * q == n { return (Some((p, q)), traj); }
            }
        }
        // mirror MetaState::step exactly: an N judgment returns before the object
        // update; a None next also leaves the object unchanged; otherwise advance.
        if jmark != J_N {
            if let Some(t) = next { tag = t; }
        }
    }
    (None, traj)
}

/// v2 as a mark — the judge judged, returning one of ⊤/⊞/⊙. Verbatim the same
/// clauses as `judge_router_v2`; only the return type changes from `Belnap` to a mark.
pub fn judge_router_v2_mark(traces: &[Vec<TraceStep>], closures: usize, corpus: usize) -> char {
    let unmatched = traces.iter().flatten().any(|s| !s.recognised);
    if unmatched { return J_N; }
    if closures == corpus { return J_T; }
    if closures == 0 { return J_N; }
    J_B
}
