//! Square-congruence phase relations over resident IMASM numeral tapes.
//! Sparse DQI row elimination cancels odd prime columns. Shared columns fold
//! into the square-root accumulator, retaining an exact modular square witness.
use super::{mf, ONE, ZERO};
use alloc::{collections::BTreeMap, vec, vec::Vec};
#[path = "glut_residue.rs"]
mod residual;
#[path = "glut_support.rs"]
mod support;
#[path = "glut_root_reentry.rs"]
mod root_reentry;
use support::Support;
type Tape = Vec<char>;
fn address(mut value: usize) -> Tape {
    let mut tape = Vec::new();
    while value != 0 {
        tape.push(if value & 1 == 1 { ONE } else { ZERO });
        value >>= 1;
    }
    if tape.is_empty() {
        tape.push(ZERO);
    }
    tape
}
fn offset(tape: &[char]) -> usize {
    let mut value = 0;
    for &mark in tape.iter().rev() {
        value = value * 2 + usize::from(mark == ONE);
    }
    value
}
fn mm(a: &[char], b: &[char], n: &[char]) -> Tape {
    mf::modulo(&mf::mul(a, b), n)
}
fn root(n: &[char], p: &[char]) -> Option<Tape> {
    use crate::belnap_word::{BelnapCoordinate64 as Coordinate, PackedWord};
    // Field primes are structural addresses in the enclosing sieve. Reduce
    // the wide source once, then keep the complete root operation on its
    // checked truth/refutation coordinates until the field returns.
    let prime = PackedWord::from_parity(p).definite_u64().expect("retained field prime");
    let modulus = Coordinate::from_definite_u64(prime);
    let value = |x: Coordinate| x.definite_u64().expect("retained definite root coordinate");
    let one = Coordinate::from_definite_u64(1);
    let source = Coordinate::from_definite_u64(mf::BelnapResidue::new(n).remainder_word(prime));
    if prime == 2 { return Some(mf::tape_u64(value(source))); }
    let less = modulus.subtract_mod(&one, &modulus).expect("field predecessor");
    let half = Coordinate::from_definite_u64(value(less) >> 1);
    let power = |a: Coordinate, exponent: Coordinate| a.power_mod(&exponent, &modulus).expect("retained field power");
    let multiply = |a: Coordinate, b: Coordinate| a.multiply_mod(&b, &modulus).expect("retained field product");
    if value(power(source, half)) != 1 { return None; }
    let count = value(less).trailing_zeros() as usize;
    let odd = Coordinate::from_definite_u64(value(less) >> count);
    let mut nonresidue = Coordinate::from_definite_u64(2);
    while value(power(nonresidue, half)) == 1 {
        nonresidue = nonresidue.add_mod(&one, &modulus).expect("next field nonresidue");
    }
    let mut c = power(nonresidue, odd);
    let mut t = power(source, odd);
    let root_exponent = Coordinate::from_definite_u64((value(odd) >> 1) + 1);
    let mut r = power(source, root_exponent);
    let mut extent = count;
    while value(t) != 1 {
        let mut at = 0;
        let mut test = t;
        while value(test) != 1 {
            test = multiply(test, test);
            at += 1;
            if at == extent {
                return None;
            }
        }
        let mut b = c;
        for _ in 0..extent - at - 1 {
            b = multiply(b, b);
        }
        c = multiply(b, b);
        t = multiply(t, c);
        r = multiply(r, b);
        extent = at;
    }
    assert_eq!(value(multiply(r, r)), value(source), "field root lost its source congruence");
    Some(mf::tape_u64(value(r)))
}
struct Column {
    prime: Tape,
    step: usize,
    roots: (Tape, Tape),
    root_words: (u64, u64),
}
#[derive(Clone)]
struct Hits {
    column: usize,
    record_link: bool,
    step: usize,
    weight: usize,
    first: usize,
    second: Option<usize>,
}
enum WeightCells {
    Byte(Vec<u8>),
    Half(Vec<u16>),
    Word(Vec<u32>),
    Address(Vec<usize>),
}
trait FoldedWeight: Copy {
    fn as_usize(self) -> usize;
    fn from_usize(value: usize) -> Self;
    fn accumulate(self, weight: Self, threshold: Self) -> Self;
    fn reaches(self, threshold: usize) -> bool {
        self.as_usize() >= threshold
    }
}
impl FoldedWeight for u8 {
    fn as_usize(self) -> usize {
        usize::from(self)
    }
    fn from_usize(value: usize) -> Self {
        value as u8
    }
    fn accumulate(self, weight: Self, threshold: Self) -> Self {
        self.saturating_add(weight).min(threshold)
    }
}
impl FoldedWeight for u16 {
    fn as_usize(self) -> usize {
        usize::from(self)
    }
    fn from_usize(value: usize) -> Self {
        value as u16
    }
    fn accumulate(self, weight: Self, threshold: Self) -> Self {
        self.saturating_add(weight).min(threshold)
    }
}
impl FoldedWeight for u32 {
    fn as_usize(self) -> usize {
        self as usize
    }
    fn from_usize(value: usize) -> Self {
        value as u32
    }
    fn accumulate(self, weight: Self, threshold: Self) -> Self {
        self.saturating_add(weight).min(threshold)
    }
}
impl FoldedWeight for usize {
    fn as_usize(self) -> usize {
        self
    }
    fn from_usize(value: usize) -> Self {
        value
    }
    fn accumulate(self, weight: Self, threshold: Self) -> Self {
        self.saturating_add(weight).min(threshold)
    }
}
fn mark_roots<T: FoldedWeight>(
    positions: &mut [Hits],
    weights: &mut [T],
    hit_links: &mut Vec<(usize, usize)>,
    block_start: usize,
    block_end: usize,
    threshold: usize,
) -> usize {
    let mut visits = 0;
    for hits in positions {
        let weight = T::from_usize(hits.weight.min(threshold));
        let threshold = T::from_usize(threshold);
        for position in core::iter::once(&mut hits.first).chain(hits.second.as_mut()) {
            while *position < block_end {
                let local = *position - block_start;
                weights[local] = weights[local].accumulate(weight, threshold);
                if hits.record_link { hit_links.push((hits.column, local)); }
                *position += hits.step;
                visits += 1;
            }
        }
    }
    visits
}
fn compact_root_links<T: FoldedWeight>(
    weights: &[T],
    hit_heads: &mut [usize],
    hit_links: &mut Vec<(usize, usize)>,
    threshold: usize,
) -> usize {
    let mut retained = 0;
    for at in 0..hit_links.len() {
        let (column, local) = hit_links[at];
        if weights[local].reaches(threshold) {
            hit_links[retained] = (column, hit_heads[local]);
            retained += 1;
            hit_heads[local] = retained;
        }
    }
    hit_links.truncate(retained);
    retained
}
impl WeightCells {
    fn new(extent: usize, threshold: usize) -> Self {
        if threshold <= u8::MAX as usize {
            Self::Byte(vec![0; extent])
        } else if threshold <= u16::MAX as usize {
            Self::Half(vec![0; extent])
        } else if threshold <= u32::MAX as usize {
            Self::Word(vec![0; extent])
        } else {
            Self::Address(vec![0; extent])
        }
    }
    fn reset(&mut self, extent: usize, threshold: usize) {
        let correct = match &*self {
            Self::Byte(_) => threshold <= u8::MAX as usize,
            Self::Half(_) => threshold > u8::MAX as usize && threshold <= u16::MAX as usize,
            Self::Word(_) => threshold > u16::MAX as usize && threshold <= u32::MAX as usize,
            Self::Address(_) => threshold > u32::MAX as usize,
        };
        if !correct {
            *self = Self::new(extent, threshold);
        }
        match self {
            Self::Byte(values) => {
                values.resize(extent, 0);
                values.fill(0);
            }
            Self::Half(values) => {
                values.resize(extent, 0);
                values.fill(0);
            }
            Self::Word(values) => {
                values.resize(extent, 0);
                values.fill(0);
            }
            Self::Address(values) => {
                values.resize(extent, 0);
                values.fill(0);
            }
        }
    }
    fn mark_roots(
        &mut self,
        positions: &mut [Hits],
        hit_links: &mut Vec<(usize, usize)>,
        block_start: usize,
        block_end: usize,
        threshold: usize,
    ) -> usize {
        match self {
            Self::Byte(values) => mark_roots(
                positions,
                values,
                hit_links,
                block_start,
                block_end,
                threshold,
            ),
            Self::Half(values) => mark_roots(
                positions,
                values,
                hit_links,
                block_start,
                block_end,
                threshold,
            ),
            Self::Word(values) => mark_roots(
                positions,
                values,
                hit_links,
                block_start,
                block_end,
                threshold,
            ),
            Self::Address(values) => mark_roots(
                positions,
                values,
                hit_links,
                block_start,
                block_end,
                threshold,
            ),
        }
    }
    fn compact_root_links(
        &self,
        hit_heads: &mut [usize],
        hit_links: &mut Vec<(usize, usize)>,
        threshold: usize,
    ) -> usize {
        match self {
            Self::Byte(values) => compact_root_links(values, hit_heads, hit_links, threshold),
            Self::Half(values) => compact_root_links(values, hit_heads, hit_links, threshold),
            Self::Word(values) => compact_root_links(values, hit_heads, hit_links, threshold),
            Self::Address(values) => compact_root_links(values, hit_heads, hit_links, threshold),
        }
    }
    fn reaches(&self, index: usize, threshold: usize) -> bool {
        match self {
            Self::Byte(values) => values[index].reaches(threshold),
            Self::Half(values) => values[index].reaches(threshold),
            Self::Word(values) => values[index].reaches(threshold),
            Self::Address(values) => values[index].reaches(threshold),
        }
    }
    fn len(&self) -> usize {
        match self {
            Self::Byte(v) => v.len(),
            Self::Half(v) => v.len(),
            Self::Word(v) => v.len(),
            Self::Address(v) => v.len(),
        }
    }
}
#[derive(Clone)]
struct TorusRootFrame {
    p: crate::belnap_word::BelnapWord,
    q: crate::belnap_word::BelnapWord,
    clock: crate::belnap_word::BelnapWord,
    axes: [usize;12],
    evidence: crate::belnap::B4,
}
struct RootSignTransport {
    frame: TorusRootFrame,
    permutation: Vec<usize>,
    flips: Vec<bool>,
    fields: Vec<serde_json::Value>,
}
fn signed_polynomial(a:&[char],terms:&[Tape],signs:&[bool])->Tape {
    terms.iter().zip(signs).fold(vec![ZERO],|value,(term,&negative)|{
        let term=if negative{mf::sub(a,term)}else{term.clone()};
        mf::modulo(&mf::add(&value,&term),a)
    })
}
struct PolynomialFamily {
    id: usize,
    a: Tape,
    b: Tape,
    factors: Vec<usize>,
    terms: Vec<Tape>,
    signs: Vec<bool>,
    inverses: Vec<Option<u64>>,
    transport: Option<RootSignTransport>,
}
impl PolynomialFamily {
    fn apply_torus(&mut self,n:&[char])->Option<serde_json::Value>{
        let before=signed_polynomial(&self.a,&self.terms,&self.signs);
        let Some(transport)=&self.transport else {self.b=before;return None;};
        let after_signs:Vec<_>=transport.permutation.iter().zip(&transport.flips)
            .map(|(&at,&flip)|self.signs[at]^flip).collect();
        let mut recovered=vec![false;self.signs.len()];
        for (i,&at) in transport.permutation.iter().enumerate(){
            recovered[at]=after_signs[i]^transport.flips[i];
        }
        assert_eq!(recovered,self.signs,"torus root-sign inverse lost a resident alternative");
        self.b=signed_polynomial(&self.a,&self.terms,&after_signs);
        assert_eq!(mm(&self.b,&self.b,&self.a),mf::modulo(n,&self.a),"torus root transport lost source congruence");
        Some(serde_json::json!({"family":self.id,"source":mf::dec_of(n),"a":mf::dec_of(&self.a),
            "before_b":mf::dec_of(&before),"after_b":mf::dec_of(&self.b),
            "terms":self.terms.iter().map(|term|mf::dec_of(term)).collect::<Vec<_>>(),
            "counter_signs":self.signs,"after_signs":after_signs,"recovered_signs":recovered,
            "permutation":transport.permutation,"flips":transport.flips,"fields":transport.fields,
            "frame":{"p":crate::toroidal_division::word(&transport.frame.p),
                "q":crate::toroidal_division::word(&transport.frame.q),
                "clock":crate::toroidal_division::word(&transport.frame.clock),
                "axes":transport.frame.axes,"evidence":transport.frame.evidence.name()},
            "inverse_verified":true,"source_congruence_verified":true,"changed_b":before!=self.b}))
    }
}
struct Relation {
    cells: Support,
    residues: Support,
    history: Support,
}
impl Relation {
    fn extent(&self) -> usize {
        self.cells.extent() + self.residues.extent() + self.history.extent()
    }
}
struct Witness {
    cells: Vec<usize>,
    x: Tape,
    y: Tape,
    residual: Tape,
    factors: Vec<Tape>,
}
#[cfg(feature = "protein-cli")]
struct WitnessWorkers {
    sender: Option<std::sync::mpsc::Sender<(u64, usize, Witness,
        std::sync::mpsc::Sender<(u64, usize, Witness)>)>>,
    handles: Vec<std::thread::JoinHandle<()>>,
    epoch: u64,
}
#[cfg(feature = "protein-cli")]
impl WitnessWorkers {
    fn new(count: usize, source: Tape) -> Self {
        let (sender, receiver) = std::sync::mpsc::channel::<(u64, usize, Witness,
            std::sync::mpsc::Sender<(u64, usize, Witness)>)>();
        let receiver = std::sync::Arc::new(std::sync::Mutex::new(receiver));
        let source = std::sync::Arc::new(source);
        let handles = (0..count).map(|_| {
            let receiver = receiver.clone();
            let source = source.clone();
            std::thread::spawn(move || loop {
                let job = receiver.lock().expect("square job boundary poisoned").recv();
                let Ok((epoch, slot, mut witness, returned)) = job else { break; };
                if !mf::zero(&witness.residual) {
                    witness.factors = residual::factors(&witness.residual);
                }
                if std::env::var_os("GM_FACTOR_TRACE").is_some() {
                    std::eprintln!("square-worker-return thread={:?} epoch={} slot={} source-bits={}",
                        std::thread::current().id(), epoch, slot, source.len());
                }
                let _ = returned.send((epoch, slot, witness));
            })
        }).collect();
        Self { sender: Some(sender), handles, epoch: 0 }
    }
    fn map(&mut self, rows: Vec<Witness>) -> Vec<Witness> {
        self.epoch += 1;
        let count = rows.len();
        let (returned, receiver) = std::sync::mpsc::channel();
        for (slot, row) in rows.into_iter().enumerate() {
            self.sender.as_ref().unwrap().send((self.epoch, slot, row, returned.clone()))
                .unwrap_or_else(|_| panic!("square worker lost its marked input"));
        }
        drop(returned);
        let mut rows: Vec<Option<Witness>> = (0..count).map(|_| None).collect();
        for _ in 0..count {
            let (epoch, slot, row) = receiver.recv().expect("square worker lost its return");
            assert_eq!(epoch, self.epoch, "square worker crossed a re-entry boundary");
            assert!(rows[slot].is_none(), "square worker duplicated an ancestry slot");
            rows[slot] = Some(row);
        }
        rows.into_iter().map(|row| row.expect("square ancestry slot missing")).collect()
    }
}
#[cfg(feature = "protein-cli")]
impl Drop for WitnessWorkers {
    fn drop(&mut self) {
        drop(self.sender.take());
        for worker in self.handles.drain(..) { let _ = worker.join(); }
    }
}
pub(super) struct SquarePhase {
    n: Tape,
    start: Tape,
    coefficient_seed: Tape,
    columns: Vec<Column>,
    basis: BTreeMap<usize, Relation>,
    witnesses: Vec<Witness>,
    partial: BTreeMap<usize, Relation>,
    residual_ids: BTreeMap<Tape, usize>,
    bound: usize,
    span: usize,
    examined: usize,
    composite: Vec<bool>,
    ready: Option<Tape>,
    hit_heads: Vec<usize>,
    hit_links: Vec<(usize, usize)>,
    weights: WeightCells,
    positions: Vec<Hits>,
    retained_cells: usize,
    family: Option<PolynomialFamily>,
    family_pool: Vec<usize>,
    family_choice: Vec<usize>,
    family_columns: usize,
    #[cfg(feature = "protein-cli")]
    workers: Option<WitnessWorkers>,
    torus_winding: Option<(Tape, Tape)>,
    torus_observations: Vec<crate::glut_system::SquareTorusObservation>,
    pending_root_frame: Option<TorusRootFrame>,
    polynomial_actions: Vec<serde_json::Value>,
    family_counter: usize,
    #[cfg(feature = "root-work-profile")]
    root_marks: usize,
    #[cfg(feature = "root-work-profile")]
    root_link_visits: usize,
    #[cfg(feature = "root-work-profile")]
    root_links: usize,
    #[cfg(feature = "root-work-profile")]
    root_candidates: usize,
}
impl SquarePhase {
    pub fn bind_root_frame(&mut self,p:crate::belnap_word::BelnapWord,q:crate::belnap_word::BelnapWord,
        axes:[usize;12],clock:crate::belnap_word::BelnapWord,evidence:crate::belnap::B4)->Result<(),alloc::string::String>{
        p.to_parity().map_err(alloc::string::String::from)?;
        q.to_parity().map_err(alloc::string::String::from)?;
        self.pending_root_frame=Some(TorusRootFrame{p,q,axes,clock,evidence});Ok(())
    }
    pub fn polynomial_actions(&self)->Vec<serde_json::Value>{self.polynomial_actions.clone()}
    pub fn bind_torus_winding(&mut self, step: usize) -> Result<(), alloc::string::String> {
        if self.torus_winding.is_some() || !self.witnesses.is_empty() {
            return Err("square torus must bind once before witness production".into());
        }
        let winding = mf::modulo(&address(step), &self.n);
        match mf::mod_inv(&winding, &self.n) {
            Ok(inverse) => self.torus_winding = Some((winding, inverse)),
            Err(divisor) => {
                if mf::cmp(&divisor, &[ONE]).is_gt() && mf::cmp(&divisor, &self.n).is_lt() {
                    self.ready = Some(divisor);
                } else {
                    return Err("source winding has no proper modular action".into());
                }
            }
        }
        Ok(())
    }
    pub fn torus_observations(&self) -> &[crate::glut_system::SquareTorusObservation] {
        &self.torus_observations
    }
    pub fn new(n: &[char], source_root: &[char]) -> Self {
        let start = if mf::mul(source_root, source_root) == n {
            source_root.to_vec()
        } else {
            mf::add(source_root, &[ONE])
        };
        let span = n.len() * n.len();
        let target = mf::divmod(&mf::isqrt(&mf::add(n, n)), &address(span)).0;
        let mut coefficient_seed = target;
        if mf::cmp(&coefficient_seed, &[ONE, ONE]) == core::cmp::Ordering::Less {
            coefficient_seed = vec![ONE, ONE];
        }
        if coefficient_seed[0] == ZERO {
            coefficient_seed = mf::add(&coefficient_seed, &[ONE]);
        }
        let retained_cells = mf::sub(n, &[ONE]).len() + 2;
        Self {
            n: n.to_vec(),
            start,
            coefficient_seed,
            columns: vec![Column {
                prime: mf::sub(n, &[ONE]),
                step: 0,
                roots: (vec![ZERO], vec![ZERO]),
                root_words: (0, 0),
            }],
            basis: BTreeMap::new(),
            witnesses: Vec::new(),
            partial: BTreeMap::new(),
            residual_ids: BTreeMap::new(),
            bound: n.len() * n.len(),
            span,
            examined: 1,
            composite: Vec::new(),
            ready: None,
            hit_heads: Vec::new(),
            hit_links: Vec::new(),
            weights: WeightCells::new(0, 0),
            positions: Vec::new(),
            retained_cells,
            family: None,
            family_pool: Vec::new(),
            family_choice: Vec::new(),
            family_columns: 0,
            #[cfg(feature = "protein-cli")]
            workers: None,
            torus_winding: None,
            torus_observations: Vec::new(),
            pending_root_frame: None,
            polynomial_actions: Vec::new(),
            family_counter: 0,
            #[cfg(feature = "root-work-profile")]
            root_marks: 0,
            #[cfg(feature = "root-work-profile")]
            root_link_visits: 0,
            #[cfg(feature = "root-work-profile")]
            root_links: 0,
            #[cfg(feature = "root-work-profile")]
            root_candidates: 0,
        }
    }
    #[cfg(feature = "root-work-profile")]
    pub fn root_work(&self) -> (usize, usize, usize, usize) {
        (
            self.root_marks,
            self.root_link_visits,
            self.root_links,
            self.root_candidates,
        )
    }
    pub fn source(&self) -> &[char] {
        &self.n
    }
    pub fn cells(&self) -> usize {
        // Account for the field and its retained ancestry without rescanning
        // every witness after each activation. Scheduling uses work_extent.
        self.bound
            + 1
            + self.n.len()
            + self.start.len()
            + self.retained_cells
            + self.hit_heads.len()
            + 2 * self.hit_links.len()
            + self.weights.len()
            + 6 * self.positions.len()
            + self.family_pool.len()
            + self.family_choice.len()
            + self.family.as_ref().map_or(0, |f| {
                f.a.len()
                    + f.b.len()
                    + f.factors.len()
                    + f.terms.iter().map(Vec::len).sum::<usize>()
                    + f.signs.len()
                    + f.inverses.len()
            })
    }
    /// Active work follows the source-width preparation frame or one sieve
    /// window. Retained witnesses are not traversed by every activation.
    pub fn work_extent(&self) -> usize {
        if self.examined < self.bound {
            self.n.len() * address(self.bound).len()
        } else {
            self.span + self.columns.len() + self.n.len()
        }
    }
    pub fn folded_extent(&self) -> usize {
        if self.examined < self.bound {
            1
        } else {
            // A prepared phase traverses both the source limb coordinates
            // and the dynamic column-address frame. Return the enclosing
            // batch after that shared frame, rather than after each limb.
            self.n.len().div_ceil(u64::BITS as usize) * address(self.columns.len()).len()
        }
    }
    fn expand_field(&mut self) {
        self.bound += self.bound;
        self.span = self.bound;
        let target = mf::divmod(&mf::isqrt(&mf::add(&self.n, &self.n)), &address(self.span)).0;
        self.coefficient_seed = target;
        if mf::cmp(&self.coefficient_seed, &[ONE, ONE]) == core::cmp::Ordering::Less {
            self.coefficient_seed = vec![ONE, ONE];
        }
        self.family = None;
        self.family_columns = 0;
    }
    fn grow(&mut self, observe: &mut impl FnMut(char, usize, usize, usize)) {
        observe(
            '∈',
            self.columns.len(),
            self.basis.len(),
            self.partial.len(),
        );
        if self.composite.len() != self.bound + 1 {
            self.composite = vec![false; self.bound + 1];
            for position in 2..=self.bound {
                if self.composite[position] {
                    continue;
                }
                let Some(mut multiple) = position.checked_mul(position) else {
                    break;
                };
                if multiple > self.bound {
                    break;
                }
                while multiple <= self.bound {
                    self.composite[multiple] = true;
                    multiple += position;
                }
            }
        }
        // A source-width frame is one scheduling turn, not a terminal field
        // limit. Resume from the retained address on the next call so the
        // other resident lanes advance during wide field preparation.
        let end = self.bound.min(self.examined.saturating_add(self.n.len()));
        for position in self.examined + 1..=end {
            if self.composite[position] {
                continue;
            }
            if position.is_power_of_two() {
                observe(
                    '∈',
                    self.columns.len(),
                    self.basis.len(),
                    self.partial.len(),
                );
            }
            let prime = address(position);
            if mf::cmp(&prime, &self.n) != core::cmp::Ordering::Less {
                continue;
            }
            let residue = mf::modulo(&self.n, &prime);
            if mf::zero(&residue) {
                self.ready = Some(prime);
                return;
            }
            if let Some(r) = root(&residue, &prime) {
                let other = if mf::zero(&r) {
                    r.clone()
                } else {
                    mf::sub(&prime, &r)
                };
                self.retained_cells += prime.len() + r.len() + other.len();
                let root_words = (offset(&r) as u64, offset(&other) as u64);
                self.columns.push(Column {
                    prime,
                    step: position,
                    roots: (r, other),
                    root_words,
                });
            }
        }
        self.examined = end;
    }
    fn join(&self, left: Relation, right: &Relation) -> Relation {
        Relation {
            cells: left.cells.xor(&right.cells),
            residues: left.residues.xor(&right.residues),
            history: left.history.xor(&right.history),
        }
    }
    fn witness(&self, history: &Support) -> (Tape, Tape) {
        use alloc::collections::BTreeSet;
        let mut x = vec![ONE];
        let mut y = vec![ONE];
        let mut odd = BTreeSet::new();
        let mut residuals = BTreeSet::new();
        for index in history.addresses() {
            let leaf = &self.witnesses[index];
            let product = leaf
                .factors
                .iter()
                .fold(vec![ONE], |value, factor| mf::mul(&value, factor));
            assert_eq!(
                product, leaf.residual,
                "residual factor views must retain their complete value"
            );
            x = mm(&x, &leaf.x, &self.n);
            y = mm(&y, &leaf.y, &self.n);
            for &column in &leaf.cells {
                if !odd.insert(column) {
                    odd.remove(&column);
                    y = mm(&y, &self.columns[column].prime, &self.n);
                }
            }
            for factor in &leaf.factors {
                if !residuals.insert(factor.clone()) {
                    residuals.remove(factor);
                    y = mm(&y, factor, &self.n);
                }
            }
        }
        assert!(
            odd.is_empty() && residuals.is_empty(),
            "phase ancestry must cancel every column"
        );
        (x, y)
    }
    fn retain(&mut self, mut relation: Relation) -> Option<Tape> {
        // Residual factors have their own ports. Close their cycles before
        // returning the resulting row to the prepared prime-column basis.
        while let Some(pivot) = relation.residues.pivot() {
            if let Some(other) = self.partial.get(&pivot) {
                relation = self.join(relation, other);
            } else {
                self.retained_cells += relation.extent();
                self.partial.insert(pivot, relation);
                return None;
            }
        }
        while let Some(pivot) = relation.cells.pivot() {
            if let Some(other) = self.basis.get(&pivot) {
                relation = self.join(relation, other);
            } else {
                self.retained_cells += relation.extent();
                self.basis.insert(pivot, relation);
                return None;
            }
        }
        // Materialize values only after their symbolic phase ancestry closes.
        let (x, y) = self.witness(&relation.history);
        assert_eq!(mm(&x, &x, &self.n), mm(&y, &y, &self.n));
        let difference = if mf::cmp(&x, &y) == core::cmp::Ordering::Less {
            mf::sub(&y, &x)
        } else {
            mf::sub(&x, &y)
        };
        let mut arms = [mf::modulo(&difference, &self.n), mf::modulo(&mf::add(&x, &y), &self.n)];
        if let Some((winding, inverse)) = &self.torus_winding {
            // The torus acts on the actual square boundary a*b=0 (mod N).
            // Opposite winding on its two arms preserves that product. Every
            // gcd below consumes the transformed arms, with no untransformed
            // candidate producer or post-production witness substitution.
            let before = arms.clone();
            let (mut forward, mut reverse) = (vec![ONE], vec![ONE]);
            for reentry in 1..=3 {
                forward = mm(&forward, winding, &self.n);
                reverse = mm(&reverse, inverse, &self.n);
                arms = [mm(&before[0], &forward, &self.n), mm(&before[1], &reverse, &self.n)];
                let recovered = [mm(&arms[0], &reverse, &self.n), mm(&arms[1], &forward, &self.n)];
                assert_eq!(mm(&arms[0], &arms[1], &self.n), mm(&before[0], &before[1], &self.n),
                    "torus square action lost its common product boundary");
                assert_eq!(recovered, before, "torus square action lost inverse recovery");
                self.torus_observations.push(crate::glut_system::SquareTorusObservation {
                    source: self.n.clone(), winding: forward.clone(), inverse: reverse.clone(),
                    before: before.clone(), after: arms.clone(), recovered, reentry,
                });
            }
        }
        for value in arms {
            let divisor = mf::gcd(value, self.n.clone());
            if mf::cmp(&divisor, &[ONE]) == core::cmp::Ordering::Greater
                && mf::cmp(&divisor, &self.n) == core::cmp::Ordering::Less
            {
                return Some(divisor);
            }
        }
        None
    }
    fn polynomial(
        &mut self,
        observe: &mut impl FnMut(char, usize, usize, usize),
    ) -> Option<(Tape, Tape, Vec<usize>)> {
        observe(
            '≻',
            self.columns.len(),
            self.basis.len(),
            self.partial.len(),
        );
        // Each enclosing coefficient owns a product of distinct resident primes.
        // Its sign siblings share that coefficient, its roots and its inverses.
        // The sign register grows with the chosen product; no machine shift
        // bounds the number of siblings.
        if self.family_columns == self.columns.len() {
            if let Some(family) = &mut self.family {
                let mut position = 1;
                while position < family.signs.len() && family.signs[position] {
                    family.signs[position] = false;
                    position += 1;
                }
                if position < family.signs.len() {
                    family.signs[position] = true;
                    if let Some(action)=family.apply_torus(&self.n){self.polynomial_actions.push(action);}
                    return Some((family.a.clone(), family.b.clone(), family.factors.clone()));
                }
                // Advance the prime-subset register after all its phase
                // siblings return. Exhaustion expands the enclosing field.
                let width = self.family_choice.len();
                let mut position = width;
                while position > 0
                    && self.family_choice[position - 1]
                        == self.family_pool.len() - width + position - 1
                {
                    position -= 1;
                }
                if position == 0 {
                    self.family = None;
                    return None;
                }
                self.family_choice[position - 1] += 1;
                for i in position..width {
                    self.family_choice[i] = self.family_choice[i - 1] + 1;
                }
            }
        } else {
            self.family = None;
            self.family_columns = self.columns.len();
            let largest = self.columns.last()?.step;
            let mut width = 1;
            let mut extent = address(largest);
            while mf::cmp(&extent, &self.coefficient_seed) == core::cmp::Ordering::Less {
                extent = mf::mul(&extent, &address(largest));
                width += 1;
            }
            // Find the per-prime coordinate of the source-derived coefficient.
            // Comparisons use complete tapes, including above a machine word.
            let (mut low, mut high) = (2, largest);
            while low < high {
                let middle = low + (high - low).div_ceil(2);
                let mut product = vec![ONE];
                for _ in 0..width {
                    product = mf::mul(&product, &address(middle));
                }
                if mf::cmp(&product, &self.coefficient_seed) == core::cmp::Ordering::Greater {
                    high = middle - 1;
                } else {
                    low = middle;
                }
            }
            let center = low;
            self.family_pool = (1..self.columns.len())
                .filter(|&i| self.columns[i].step > 2)
                .collect();
            self.family_pool
                .sort_unstable_by_key(|&i| self.columns[i].step.abs_diff(center));
            if self.family_pool.len() < width {
                return None;
            }
            self.family_choice = (0..width).collect();
        }
        let mut selected: Vec<usize> = self
            .family_choice
            .iter()
            .map(|&i| self.family_pool[i])
            .collect();
        selected.sort_unstable();
        let mut a = vec![ONE];
        for &column in &selected {
            a = mf::mul(&a, &self.columns[column].prime);
        }
        let mut terms = Vec::new();
        let mut b = vec![ZERO];
        observe(
            '⊞',
            self.columns.len(),
            self.basis.len(),
            self.partial.len(),
        );
        for &column in &selected {
            let field = &self.columns[column];
            let q = &field.prime;
            let r = &field.roots.0;
            let (other, remainder) = mf::divmod(&a, q);
            assert!(mf::zero(&remainder));
            let inverse = mf::mod_inv(&other, q).expect("distinct coefficient fields are coprime");
            let term = mf::mul(&other, &mm(r, &inverse, q));
            b = mf::modulo(&mf::add(&b, &term), &a);
            terms.push(term);
        }
        observe(
            '⋈',
            self.columns.len(),
            self.basis.len(),
            self.partial.len(),
        );
        let folded_a = mf::BelnapResidue::new(&a);
        let inverses = self
            .columns
            .iter()
            .map(|field| {
                if field.step == 0 {
                    None
                } else {
                    mf::inverse_residue_word(
                        folded_a.remainder_word(field.step as u64),
                        field.step as u64,
                    )
                }
            })
            .collect();
        assert_eq!(mf::modulo(&mf::mul(&b, &b), &a), mf::modulo(&self.n, &a));
        // Bind once for this complete coefficient family. Its counter still
        // visits every sign alternative modulo global negation. A new body
        // observation may select the next family's frame, never erase the
        // unvisited alternatives of this one.
        let transport=self.pending_root_frame.clone().map(|frame|{
            use crate::belnap_word::BelnapCoordinate64 as Coordinate;
            let width=terms.len();
            let mut permutation:Vec<_>=(0..width).collect();
            for i in 0..12 {permutation.swap(i%width,frame.axes[i]%width);}
            let p=mf::BelnapResidue::new(&frame.p.to_parity().unwrap());
            let q=mf::BelnapResidue::new(&frame.q.to_parity().unwrap());
            let mut flips=Vec::new();let mut fields=Vec::new();
            for &column in &selected {
                let prime=self.columns[column].step as u64;
                let modulus=Coordinate::from_definite_u64(prime);
                let left=Coordinate::from_definite_u64(p.remainder_word(prime));
                let right=Coordinate::from_definite_u64(q.remainder_word(prime));
                let inverse=right.inverse_mod(&modulus).expect("definite root-frame coordinate");
                let unit=inverse.map(|inverse|left.multiply_mod(&inverse,&modulus).unwrap());
                let exponent=Coordinate::from_definite_u64((prime-1)/2);
                let character=unit.map(|unit|unit.power_mod(&exponent,&modulus).unwrap().definite_u64().unwrap());
                let nonunit=character.is_none_or(|value|value==0);
                let flip=character==Some(prime-1);
                assert!(nonunit || matches!(character,Some(1)) || flip);
                flips.push(flip);
                fields.push(serde_json::json!({"prime":prime,"p_residue":left.definite_u64().unwrap(),
                    "q_residue":right.definite_u64().unwrap(),"inverse":inverse.map(|v|v.definite_u64().unwrap()),
                    "unit":unit.map(|v|v.definite_u64().unwrap()),"character":character,
                    "held_nonunit":nonunit,"flip":flip}));
            }
            RootSignTransport{frame,permutation,flips,fields}
        });
        self.family_counter+=1;
        self.family = Some(PolynomialFamily {
            id:self.family_counter,
            a: a.clone(),
            b: b.clone(),
            factors: selected.clone(),
            signs: vec![false; terms.len()],
            terms,
            inverses,
            transport,
        });
        if let Some(action)=self.family.as_mut().unwrap().apply_torus(&self.n){self.polynomial_actions.push(action);}
        let b=self.family.as_ref().unwrap().b.clone();
        Some((a, b, selected))
    }
    pub fn advance(
        &mut self,
        mut observe: impl FnMut(char, usize, usize, usize),
    ) -> Option<Tape> {
        if self.examined < self.bound {
            self.grow(&mut observe);
        }
        if let Some(p) = self.ready.take() {
            return Some(p);
        }
        if self.examined < self.bound {
            return None;
        }
        let Some((a, b, coefficient_columns)) = self.polynomial(&mut observe) else {
            self.expand_field();
            return None;
        };
        let half = self.span / 2;
        let half_tape = address(half);
        self.positions.clear();
        let folded_b = mf::BelnapResidue::new(&b);
        let root_polynomial = root_reentry::RootPolynomial::new(&a, &b, &self.n, half);
        for (column, field) in self.columns.iter().enumerate() {
            if column.is_power_of_two() {
                observe(
                    '∋',
                    self.columns.len(),
                    self.basis.len(),
                    self.partial.len(),
                );
            }
            if field.step == 0 {
                continue;
            }
            let inverse = self.family.as_ref().unwrap().inverses[column];
            let reduced_b_word = folded_b.remainder_word(field.step as u64);
            let (first, second) = if inverse.is_none() {
                let position = root_polynomial.linear_root(field.step);
                (position, position)
            } else {
                let inverse = inverse.unwrap();
                let position = |root: u64| {
                    let modulus=field.step as u64;
                    let displacement=mf::residue_sub(root,reduced_b_word,modulus);
                    mf::residue_add(mf::residue_mul(displacement,inverse,modulus),half as u64,modulus) as usize
                };
                (position(field.root_words.0), position(field.root_words.1))
            };
            self.positions.push(Hits {
                column,
                record_link: true,
                step: field.step,
                weight: field.prime.len() - 1,
                first,
                second: (second != first).then_some(second),
            });
            // Re-enter each root through its prime-power fibers before the
            // candidate threshold. Only the base root supplies an ancestry
            // link: contraction already divides out the full valuation.
            let roots = if first == second { vec![first] } else { vec![first, second] };
            for (modulus, position) in root_reentry::hensel_lift_bridge(&root_polynomial, field.step, &roots, self.span) {
                self.positions.push(Hits { column, record_link: false, step: modulus,
                    weight: field.prime.len() - 1, first: position, second: None });
            }

        }
        let expected = a.len() + 2 * half_tape.len();
        let slack = 2 * address(self.bound).len() + 4;
        let threshold = expected.saturating_sub(slack);
        let mut useful = 0;
        let partial_before = self.partial.len();
        // The enclosing window retains each root's running coordinate. Each
        // block consumes its local hits and witnesses before reusing storage.
        // The block extent follows the source's folded frame and never limits
        // the complete window or its dynamically growing prime field.
        let block_extent = self.n.len() * u64::BITS as usize;
        let mut block_start = 0;
        while block_start < self.span {
            let block_end = self.span.min(block_start.saturating_add(block_extent));
            let extent = block_end - block_start;
            self.hit_heads.resize(extent, 0);
            self.hit_heads.fill(0);
            self.weights.reset(extent, threshold);
            self.hit_links.clear();
            observe(
                '∋',
                self.columns.len(),
                self.basis.len(),
                self.partial.len(),
            );
            let root_marks = self.weights.mark_roots(
                &mut self.positions,
                &mut self.hit_links,
                block_start,
                block_end,
                threshold,
            );
            observe(
                '≺',
                self.columns.len(),
                self.basis.len(),
                self.partial.len(),
            );
            let root_links = self.weights.compact_root_links(
                &mut self.hit_heads,
                &mut self.hit_links,
                threshold,
            );
            let link_visits = root_marks;
            observe(
                '⋈',
                self.columns.len(),
                self.basis.len(),
                self.partial.len(),
            );
            #[cfg(feature = "root-work-profile")]
            {
                self.root_marks += root_marks;
                self.root_link_visits += link_visits;
                self.root_links += root_links;
            }
            #[cfg(not(feature = "root-work-profile"))]
            {
                let _ = (root_marks, link_visits, root_links);
            }
            observe(
                '⊥',
                self.columns.len(),
                self.basis.len(),
                self.partial.len(),
            );
            // Mark disjoint root-candidate addresses under one immutable
            // source/polynomial frame. Workers return numerical witnesses;
            // only this owner assigns ancestry IDs and contracts relations.
            let candidates: Vec<_> = (0..extent)
                .filter(|&local| self.weights.reaches(local, threshold)).collect();
            let prepare = |&local: &usize| {
                let index = block_start + local;
                let term = mf::mul(&a, &address(index.abs_diff(half)));
                let x = if index >= half {
                    mf::add(&term, &b)
                } else if mf::cmp(&term, &b) == core::cmp::Ordering::Less {
                    mf::sub(&b, &term)
                } else {
                    mf::sub(&term, &b)
                };
                let square = mf::mul(&x, &x);
                let negative = mf::cmp(&square, &self.n) == core::cmp::Ordering::Less;
                let difference = if negative {
                    mf::sub(&self.n, &square)
                } else {
                    mf::sub(&square, &self.n)
                };
                let (residue, remainder) = mf::divmod(&difference, &a);
                assert!(mf::zero(&remainder));
                if mf::zero(&residue) {
                    return Witness { cells: Vec::new(), x: x.clone(), y: x, residual: residue, factors: Vec::new() };
                }
                let mut folded_residue = mf::BelnapResidue::new(&residue);
                // x² = A*g. The coefficient's known prime columns enter the same
                // parity register as g, and cancel there with matching divisions.
                let mut cells = coefficient_columns.clone();
                if negative {
                    cells.push(0);
                }
                let mut y = vec![ONE];
                let mut link = self.hit_heads[local];
                while link != 0 {
                    let (column, next) = self.hit_links[link - 1];
                    link = next;
                    let field = &self.columns[column];
                    let mut odd = if let Some(at) = cells.iter().position(|&i| i == column) {
                        cells.swap_remove(at);
                        true
                    } else {
                        false
                    };
                    while folded_residue.divide_word_exact(field.step as u64) {
                        odd = !odd;
                        if !odd {
                            y = mm(&y, &field.prime, &self.n);
                        }
                    }
                    if odd {
                        cells.push(column);
                    }
                }
                let residue = folded_residue.into_tape();
                cells.sort_unstable();
                let factors = Vec::new();
                Witness { cells, x, y, residual: residue, factors }
            };
            let mut prepared: Vec<Witness> = candidates.iter().map(&prepare).collect();
            #[cfg(feature = "protein-cli")]
            let workers = std::env::var("GFE_WORKERS").ok()
                .and_then(|v| v.parse::<usize>().ok()).unwrap_or(1).max(1).min(8);
            #[cfg(not(feature = "protein-cli"))]
            let workers = 1usize;
            #[cfg(feature = "protein-cli")]
            if workers > 1 && !prepared.is_empty() {
                let pool = self.workers.get_or_insert_with(|| WitnessWorkers::new(workers, self.n.clone()));
                prepared = pool.map(prepared);
            }
            if workers == 1 {
                for row in &mut prepared {
                    if !mf::zero(&row.residual) { row.factors = residual::factors(&row.residual); }
                }
            }
            #[cfg(feature = "root-work-profile")]
            { self.root_candidates += prepared.len(); }
            for prepared in prepared {
                let Witness { cells, x, y, residual: residue, factors } = prepared;
                if mf::zero(&residue) { return Some(x); }
                let factors = if self.residual_ids.contains_key(&residue) {
                    vec![residue.clone()]
                } else { factors };
                let mut residues = alloc::collections::BTreeSet::new();
                for factor in &factors {
                    let next = self.residual_ids.len();
                    let id = if let Some(&id) = self.residual_ids.get(factor) {
                        id
                    } else {
                        self.retained_cells += factor.len() + 1;
                        self.residual_ids.insert(factor.clone(), next);
                        next
                    };
                    if !residues.insert(id) {
                        residues.remove(&id);
                    }
                }
                let leaf = self.witnesses.len();
                self.retained_cells += cells.len()
                    + x.len()
                    + y.len()
                    + residue.len()
                    + factors.iter().map(Vec::len).sum::<usize>();
                self.witnesses.push(Witness {
                    cells: cells.clone(),
                    x,
                    y,
                    residual: residue,
                    factors,
                });
                let relation = Relation {
                    cells: Support::from_sorted(cells),
                    residues: Support::from_sorted(residues),
                    history: Support::from_sorted([leaf]),
                };
                useful += 1;
                observe(
                    '⊡',
                    self.columns.len(),
                    self.basis.len(),
                    self.partial.len(),
                );
                let found = self.retain(relation);
                observe(
                    '⊥',
                    self.columns.len(),
                    self.basis.len(),
                    self.partial.len(),
                );
                if let Some(p) = found {
                    return Some(p);
                }
            }
            block_start = block_end;
        }
        self.start = mf::add(&self.start, &address(self.span));
        // Relation support grows when a block contributes no reduced rows.
        // Existing columns, partial relations and ancestry remain resident.
        if useful == 0 && self.partial.len() == partial_before {
            self.expand_field();
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn folded_weight_register_retains_the_full_threshold_relation() {
        for threshold in [
            0,
            1,
            u8::MAX as usize,
            u8::MAX as usize + 1,
            u16::MAX as usize,
            u16::MAX as usize + 1,
            u32::MAX as usize,
            u32::MAX as usize + 1,
            usize::MAX,
        ] {
            let additions = [1, 255, 256, 65_535, 65_536, usize::MAX];
            let mut folded = WeightCells::new(0, 0);
            folded.reset(additions.len(), threshold);
            let mut full = vec![0usize; additions.len()];
            let mut events = Vec::new();
            for (index, &weight) in additions.iter().enumerate() {
                events.push(Hits {
                    column: index,
                    record_link: true,
                    step: additions.len() + 1,
                    weight,
                    first: index,
                    second: None,
                });
                full[index] = weight;
                for at in 0..index {
                    events.push(Hits {
                        column: index,
                    record_link: true,
                        step: additions.len() + 1,
                        weight,
                        first: at,
                        second: None,
                    });
                    full[at] = full[at].saturating_add(weight);
                }
            }
            let mut heads = vec![0; additions.len()];
            let mut links = Vec::new();
            let root_origins = events.clone();
            let _ = folded.mark_roots(&mut events, &mut links, 0, additions.len(), threshold);
            let _ = folded.compact_root_links(&mut heads, &mut links, threshold);
            for (at, &score) in full.iter().enumerate() {
                assert_eq!(
                    folded.reaches(at, threshold),
                    score >= threshold,
                    "threshold={threshold} cell={at} score={score}"
                );
                let mut actual = Vec::new();
                let mut link = heads[at];
                while link != 0 {
                    let (column, next) = links[link - 1];
                    actual.push(column);
                    link = next;
                }
                actual.sort_unstable();
                let expected = if score >= threshold {
                    root_origins
                        .iter()
                        .filter_map(|hit| {
                            (at >= hit.first && (at - hit.first) % hit.step == 0)
                                .then_some(hit.column)
                        })
                        .collect::<Vec<_>>()
                } else {
                    Vec::new()
                };
                assert_eq!(
                    actual, expected,
                    "threshold={threshold} cell={at} factor roots"
                );
            }
            assert_eq!(folded.len(), additions.len());
        }
    }
    #[test]
    fn sibling_fields_retain_the_random_192_source_congruence() {
        let n = mf::parse_numeral(include_str!("../tests/fixtures/random_rsa_192.imasm").trim())
            .unwrap();
        let mut phase = SquarePhase::new(&n, &mf::isqrt(&n));
        while phase.examined < phase.bound {
            phase.grow(&mut |_, _, _, _| {});
        }
        let mut siblings = 0;
        let mut previous = None;
        let mut distinct = alloc::collections::BTreeSet::new();
        for _ in 0..n.len() {
            let (a, b, factors) = phase.polynomial(&mut |_, _, _, _| {}).unwrap();
            let mut product = vec![ONE];
            for &column in &factors {
                product = mf::mul(&product, &phase.columns[column].prime);
            }
            assert_eq!(product, a);
            assert_eq!(mm(&b, &b, &a), mf::modulo(&n, &a));
            if previous.as_ref() == Some(&a) {
                siblings += 1;
            }
            previous = Some(a.clone());
            assert!(distinct.insert((a.clone(), b)));
            let folded = mf::BelnapResidue::new(&a);
            for (column, inverse) in phase
                .columns
                .iter()
                .zip(&phase.family.as_ref().unwrap().inverses)
                .skip(1)
            {
                let residue = folded.remainder_word(column.step as u64);
                if let Some(inverse) = inverse {
                    assert_eq!(
                        (u128::from(residue) * u128::from(*inverse)) % (column.step as u128),
                        1
                    );
                } else {
                    assert_eq!(residue, 0);
                }
            }
        }
        assert!(siblings > 0);
        let retained = phase
            .columns
            .iter()
            .map(|c| c.prime.len() + c.roots.0.len() + c.roots.1.len())
            .sum::<usize>();
        assert_eq!(phase.retained_cells, retained);
        // Field expansion can insert preparation activations before a sibling
        // reaches the witness boundary. Follow that boundary, not a fixed
        // number of scheduling turns.
        while phase.witnesses.is_empty() {
            phase.advance(|_, _, _, _| {});
        }
        for leaf in &phase.witnesses {
            let mut value = mm(&mm(&leaf.y, &leaf.y, &n), &leaf.residual, &n);
            for &column in &leaf.cells {
                value = mm(&value, &phase.columns[column].prime, &n);
            }
            assert_eq!(mm(&leaf.x, &leaf.x, &n), value);
        }
        let retained = phase
            .columns
            .iter()
            .map(|c| c.prime.len() + c.roots.0.len() + c.roots.1.len())
            .sum::<usize>()
            + phase
                .basis
                .values()
                .chain(phase.partial.values())
                .map(Relation::extent)
                .sum::<usize>()
            + phase
                .witnesses
                .iter()
                .map(|r| {
                    r.cells.len()
                        + r.x.len()
                        + r.y.len()
                        + r.residual.len()
                        + r.factors.iter().map(Vec::len).sum::<usize>()
                })
                .sum::<usize>()
            + phase
                .residual_ids
                .keys()
                .map(|value| value.len() + 1)
                .sum::<usize>();
        assert_eq!(phase.retained_cells, retained);
    }
    #[test]
    fn modular_roots_match_the_resident_2048_bit_source() {
        let n = mf::parse_numeral(include_str!("../tests/fixtures/random_rsa_2048.imasm").trim())
            .unwrap();
        assert_eq!(n.len(), 2048);
        let mut checked = 0;
        for position in [257, 263, 269, 271, 277, 281, 283, 293] {
            let p = address(position);
            let residue = mf::modulo(&n, &p);
            if let Some(r) = root(&residue, &p) {
                assert_eq!(mm(&r, &r, &p), residue);
                checked += 1;
            }
        }
        assert!(checked > 1);
    }
}
