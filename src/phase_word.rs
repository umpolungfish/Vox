//! Observation transport under the sequential SIXTEEN_3 register semantics.
//! Structural graph closure is a separate judgment. This evaluator records
//! deposits and their payloads; a seeded register never invents an observation.

use alloc::string::String;
use alloc::vec::Vec;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Deposit<T> {
    pub id: usize,
    pub lane: u8,
    pub observation: T,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Readout<T> {
    pub register: u8,
    pub surviving: Vec<Deposit<T>>,
    pub cleared: usize,
    pub restored: usize,
    pub exposed: usize,
    pub deposits: usize,
    pub seeded: usize,
    pub inert: usize,
    /// Lane-mask of each frame popped by `∋`, innermost first.
    /// The mask is the frame's own deposits unioned with every nested frame
    /// already restored into it, which is `Γ` at that boundary.
    pub restore_supports: Vec<u8>,
}

/// The executing continuation includes its open banks and fixation boundary.
/// A landing register alone cannot determine what a subsequent fuse returns.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Continuation<T> {
    readout: Readout<T>,
    frames: Vec<Vec<Deposit<T>>>,
    fixed: bool,
    next_id: usize,
    bound_ids: alloc::collections::BTreeMap<usize, usize>,
}

impl<T: Clone> Default for Continuation<T> {
    fn default() -> Self { Self::seeded(0) }
}

impl<T: Clone> Continuation<T> {
    fn seeded(seed: u8) -> Self {
        Self {
            readout: Readout {
                register: seed, surviving: Vec::new(), cleared: 0, restored: 0,
                exposed: 0, deposits: 0, seeded: 0, inert: 0,
                restore_supports: Vec::new(),
            },
            frames: Vec::new(), fixed: false, next_id: 0,
            bound_ids: alloc::collections::BTreeMap::new(),
        }
    }

    pub fn frames(&self) -> &[Vec<Deposit<T>>] { &self.frames }
    pub fn readout(&self) -> &Readout<T> { &self.readout }
    pub fn is_fixed(&self) -> bool { self.fixed }
    pub fn next_deposit_id(&self) -> usize { self.next_id }
    pub fn bound_slots(&self) -> &alloc::collections::BTreeMap<usize, usize> { &self.bound_ids }

    /// Add work to this same carrier, including work on a newly added payload.
    pub fn advance_with(
        &mut self, word: &str, observation: &T,
        payload: impl FnMut(char, &T) -> Result<(), String>,
    ) -> Result<(), String> {
        validate_word(word)?;
        self.advance_addressed_iter_with(word.chars(), observation, payload, |_| None)
    }
}

fn validate_word(word: &str) -> Result<(), String> {
    if word.is_empty() || word.chars().any(|m| !matches!(m,
        '⊢' | '⊣' | '≻' | '≺' | '⋈' | '⊙' | '∈' | '∋' | '⊤' | '⊥' | '⊞' | '⊡')) {
        return Err("invalid phase execution word".into());
    }
    Ok(())
}

fn union<T: Clone>(target: &mut Vec<Deposit<T>>, source: &[Deposit<T>]) -> usize {
    let before = target.len();
    for deposit in source {
        if !target.iter().any(|d| d.id == deposit.id) {
            target.push(deposit.clone());
        }
    }
    target.len() - before
}

pub fn execute<T: Clone>(word: &str, observation: &T) -> Result<Readout<T>, String> {
    execute_with(word, observation, |_, _| Ok(()))
}

/// Execute resident payload work while its enclosing frame is still open.
pub fn execute_with<T: Clone>(
    word: &str,
    observation: &T,
    payload: impl FnMut(char, &T) -> Result<(), String>,
) -> Result<Readout<T>, String> {
    // Validate before execution, including marks that would be inert after ⊡.
    validate_word(word)?;
    execute_iter_with(word.chars(), observation, payload)
}

/// A resident protocol may feed its connected body back through an open frame.
/// The iterator emits the enclosing fuse and fixation only after verification.
pub(crate) fn execute_iter_with<T: Clone>(
    word: impl IntoIterator<Item = char>,
    observation: &T,
    payload: impl FnMut(char, &T) -> Result<(), String>,
) -> Result<Readout<T>, String> {
    execute_addressed_iter_with(word, observation, payload, |_| None)
}

/// Bound register slots retain identity when the protocol returns through them.
/// Distinct slots remain distinct even when they carry the same observation.
pub(crate) fn execute_addressed_iter_with<T: Clone>(
    word: impl IntoIterator<Item = char>,
    observation: &T,
    payload: impl FnMut(char, &T) -> Result<(), String>,
    address: impl FnMut(u8) -> Option<usize>,
) -> Result<Readout<T>, String> {
    execute_seeded_iter_with(word, observation, payload, address, 0)
}

/// Observe the complete register map without inventing payload deposits for a seed.
/// The native kernel uses N,T,F,B bits; the runtime uses T,F,t,f bits.
pub fn native_transition_table(word: &str) -> Result<[u8; 16], String> {
    execute(word, &())?;
    let mut table = [0; 16];
    for native in 0..16u8 {
        let runtime = runtime_mask_from_native(native);
        let read = execute_seeded_iter_with(word.chars(), &(), |_, _| Ok(()), |_| None, runtime)?;
        let r = read.register;
        table[native as usize] = native_mask_from_runtime(r);
    }
    Ok(table)
}

/// The existing basis transport: runtime T,F,t,f to kernel N,T,F,B.
pub fn native_mask_from_runtime(r:u8)->u8{
    ((r&1)<<3)|((r&2)<<1)|((r&4)>>1)|((r&8)>>3)
}
pub fn runtime_mask_from_native(r:u8)->u8{
    ((r&8)>>3)|((r&4)>>1)|((r&2)<<1)|((r&1)<<3)
}

/// Every source-word prefix executes through the same banked evaluator.
/// Preserve its ordered native states instead of only its terminal register.
pub fn native_state_path(word: &str) -> Result<Vec<u8>, String> {
    Ok(native_state_trace(word)?.0)
}

/// Prefixes advance one retained carrier. The caller can keep its actual banks
/// and continue it; replaying the terminal register would lose that capability.
pub fn native_state_trace(word: &str) -> Result<(Vec<u8>, Continuation<()>), String> {
    validate_word(word)?;
    let mut carried = Continuation::default();
    let mut states=Vec::with_capacity(word.chars().count());
    for mark in word.chars() {
        carried.advance_addressed_iter_with(core::iter::once(mark), &(), |_, _| Ok(()), |_| None)?;
        states.push(native_mask_from_runtime(carried.readout.register));
    }
    Ok((states, carried))
}

fn execute_seeded_iter_with<T: Clone>(
    word: impl IntoIterator<Item = char>,
    observation: &T,
    payload: impl FnMut(char, &T) -> Result<(), String>,
    address: impl FnMut(u8) -> Option<usize>,
    seed: u8,
) -> Result<Readout<T>, String> {
    let mut carried = Continuation::seeded(seed);
    carried.advance_addressed_iter_with(word, observation, payload, address)?;
    Ok(carried.readout)
}

impl<T: Clone> Continuation<T> {
fn advance_addressed_iter_with(
    &mut self, word: impl IntoIterator<Item = char>, observation: &T,
    mut payload: impl FnMut(char, &T) -> Result<(), String>,
    mut address: impl FnMut(u8) -> Option<usize>,
) -> Result<(), String> {
    let out = &mut self.readout;
    let frames = &mut self.frames;
    let fixed = &mut self.fixed;
    let next_id = &mut self.next_id;
    let bound_ids = &mut self.bound_ids;
    for mark in word {
        if !matches!(
            mark,
            '⊢' | '⊣' | '≻' | '≺' | '⋈' | '⊙' | '∈' | '∋' | '⊤' | '⊥' | '⊞' | '⊡'
        ) {
            return Err("invalid resident protocol mark".into());
        }
        if *fixed && mark != '⊡' && mark != '⊙' {
            out.inert += 1;
            continue;
        }
        payload(mark, observation)?;
        match mark {
            '⊢' | '≺' => {
                out.cleared += out.surviving.len();
                for d in &out.surviving {
                    if mark == '⊢' || !frames.iter().flatten().any(|b| b.id == d.id) {
                        out.exposed += 1;
                    }
                }
                out.surviving.clear();
                out.register = 0;
                if mark == '⊢' {
                    frames.clear();
                }
            }
            '≻' | '⊙' => {
                if out.register == 0 {
                    out.register = 1;
                    out.seeded += 1;
                }
            }
            '∈' => frames.push(Vec::new()),
            '∋' => {
                if let Some(frame) = frames.pop() {
                    let mut support = 0u8;
                    for d in &frame {
                        support |= d.lane;
                    }
                    out.restore_supports.push(support);
                    out.restored += union(&mut out.surviving, &frame);
                    for d in &frame {
                        out.register |= d.lane;
                    }
                    if let Some(parent) = frames.last_mut() {
                        union(parent, &frame);
                    }
                }
            }
            '⊤' | '⊥' | '⊞' => {
                out.deposits += 1;
                let lanes: &[u8] = match mark {
                    '⊤' => &[1],
                    '⊥' => &[2],
                    _ => &[4, 8],
                };
                for &lane in lanes {
                    let slot = address(lane);
                    let id = if let Some(slot) = slot {
                        *bound_ids.entry(slot).or_insert_with(|| {
                            let id = *next_id;
                            *next_id += 1;
                            id
                        })
                    } else {
                        let id = *next_id;
                        *next_id += 1;
                        id
                    };
                    let d = Deposit {
                        id,
                        lane,
                        observation: observation.clone(),
                    };
                    out.register |= lane;
                    if let Some(frame) = frames.last_mut() {
                        if slot.is_some() {
                            union(frame, core::slice::from_ref(&d));
                        } else {
                            frame.push(d.clone());
                        }
                    }
                    if slot.is_some() {
                        union(&mut out.surviving, core::slice::from_ref(&d));
                    } else {
                        out.surviving.push(d);
                    }
                }
            }
            '⊡' => *fixed = true,
            '⋈' | '⊣' => (),
            _ => unreachable!(),
        }
    }
    Ok(())
}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::morphism_factor;
    const WORD: &str = "⊢≻∈⊤⊥⊞⋈≺∋⊙⊡⊣";

    #[test]
    fn identical_complete_readouts_can_have_distinct_fuse_successors() {
        let mut truth = Continuation::default();
        let mut falsity = Continuation::default();
        truth.advance_with("∈⊤≺", &"same enclosing ensemble", |_, _| Ok(())).unwrap();
        falsity.advance_with("∈⊥≺", &"same enclosing ensemble", |_, _| Ok(())).unwrap();
        assert_eq!(truth.readout, falsity.readout);
        assert_eq!(truth.readout.register, 0);
        truth.advance_with("∋", &"new observation", |_, _| Ok(())).unwrap();
        falsity.advance_with("∋", &"new observation", |_, _| Ok(())).unwrap();
        assert_eq!(truth.readout.register, 1);
        assert_eq!(falsity.readout.register, 2);
        assert_eq!(truth.readout.surviving[0].observation, "same enclosing ensemble");
        assert_eq!(falsity.readout.surviving[0].observation, "same enclosing ensemble");
    }

    #[test]
    fn adding_a_part_changes_the_whole_return_and_preserves_both_sources() {
        let mut ensemble = Continuation::default();
        ensemble.advance_with("∈⊤", &"first part", |_, _| Ok(())).unwrap();
        let before_addition = ensemble.clone();
        ensemble.advance_with("∈⊥", &"added part", |_, _| Ok(())).unwrap();
        ensemble.advance_with("≺∋≺∋", &"return", |_, _| Ok(())).unwrap();
        let mut control = before_addition;
        control.advance_with("∈⋈", &"empty added part", |_, _| Ok(())).unwrap();
        control.advance_with("≺∋≺∋", &"return", |_, _| Ok(())).unwrap();
        assert_eq!(control.readout.register, 1);
        assert_eq!(ensemble.readout.register, 3);
        assert_eq!(ensemble.readout.surviving.iter().map(|d| d.observation).collect::<Vec<_>>(),
                   vec!["first part", "added part"]);
        assert_eq!(ensemble.readout.restore_supports, vec![2, 3]);
        assert!(ensemble.frames().is_empty());
    }

    #[test]
    fn fixation_and_banked_identity_act_in_the_retained_continuation() {
        let mut live = Continuation::default();
        let mut fixed = Continuation::default();
        live.advance_with("⊤⋈", &0, |_, _| Ok(())).unwrap();
        fixed.advance_with("⊤⊡", &0, |_, _| Ok(())).unwrap();
        assert_eq!(live.readout, fixed.readout);
        live.advance_with("⊥", &1, |_, _| Ok(())).unwrap();
        fixed.advance_with("⊥", &1, |_, _| Ok(())).unwrap();
        assert_eq!(live.readout.register, 3);
        assert_eq!(fixed.readout.register, 1);

        let mut banked = Continuation::default();
        banked.advance_addressed_iter_with("∈⊤≺".chars(), &7,
            |_, _| Ok(()), |_| Some(11)).unwrap();
        banked.advance_addressed_iter_with("⊤≺∋".chars(), &7,
            |_, _| Ok(()), |_| Some(11)).unwrap();
        assert_eq!(banked.readout.surviving.len(), 1);
        assert_eq!(banked.next_deposit_id(), 1);
        assert_eq!(banked.bound_slots().get(&11), Some(&0));
    }

    #[test]
    fn every_split_of_a_banked_word_continues_as_the_unsplit_execution() {
        for word in [WORD, "∈⊤∈⊥≺∋≺∋⊞⊡⊥", "⊤∈⊥⊢⊞∋⊙", "∈∈⊤∋≺∋"] {
            let mut whole = Continuation::default();
            whole.advance_with(word, &42, |_, _| Ok(())).unwrap();
            for split in word.char_indices().map(|(i, _)| i).chain(core::iter::once(word.len())) {
                let mut partial = Continuation::default();
                for part in [&word[..split], &word[split..]] {
                    if !part.is_empty() { partial.advance_with(part, &42, |_, _| Ok(())).unwrap(); }
                }
                assert_eq!(partial, whole, "{word} split at {split}");
            }
            let (states, terminal) = native_state_trace(word).unwrap();
            for ((start, mark), state) in word.char_indices().zip(states) {
                assert_eq!(state, native_mask_from_runtime(execute(&word[..start+mark.len_utf8()], &()).unwrap().register));
            }
            assert_eq!(terminal.readout, execute(word, &()).unwrap());
        }
    }

    #[test]
    fn resident_slots_fold_return_passes_and_restore_the_random_2048_payload() {
        let payload = morphism_factor::parse_numeral(
            include_str!("../tests/fixtures/random_rsa_2048.imasm").trim(),
        )
        .unwrap();
        let word = "⊢∈⊤⊥⊞≺⊤⊥⊞≺∋⊡⊣";
        let folded = execute_addressed_iter_with(
            word.chars(),
            &payload,
            |_, _| Ok(()),
            |lane| Some(usize::from(lane)),
        )
        .unwrap();
        let separate = execute(word, &payload).unwrap();
        assert_eq!(folded.exposed, 0);
        assert_eq!(folded.cleared, 8);
        assert_eq!(folded.restored, 4);
        assert_eq!(folded.surviving.len(), 4);
        assert_eq!(separate.surviving.len(), 8);
        assert_eq!(folded.register, separate.register);
        assert!(folded.surviving.iter().all(|d| d.observation == payload));
    }

    #[test]
    fn all_cuts_match_supplied_weight_trace() {
        let marks: Vec<char> = WORD.chars().collect();
        let payload = vec![0usize, 4, 8, 12];
        for k in 0..marks.len() {
            let word: String = marks[k..].iter().chain(&marks[..k]).collect();
            let r = execute(&word, &payload).unwrap();
            let holds = [0, 1, 2, 11].contains(&k);
            assert_eq!(r.register, if holds { 15 } else { 1 }, "cut {k}");
            assert_eq!(r.surviving.len(), if holds { 4 } else { 0 }, "cut {k}");
            assert_eq!(r.restored, if holds { 4 } else { 0 }, "cut {k}");
            assert_eq!(
                r.exposed,
                match k {
                    3 => 4,
                    4 => 3,
                    5 => 2,
                    _ => 0,
                },
                "cut {k}"
            );
            for d in r.surviving {
                assert_eq!(d.observation, payload);
            }
        }
    }

    #[test]
    fn nested_frames_restore_once_and_seed_has_no_payload() {
        let r = execute("∈∈⊤∋≺∋", &42).unwrap();
        assert_eq!(r.surviving.len(), 1);
        assert_eq!(r.restored, 1);
        assert_eq!(r.exposed, 0);
        assert_eq!(r.restore_supports, vec![1, 1]);
        assert!(execute("⊙⊡", &42).unwrap().surviving.is_empty());
        assert!(execute("⊡?", &42).is_err());
        assert!(execute("", &42).is_err());
    }

    #[test]
    fn native_table_preserves_carrier_coordinates_and_default_feedback() {
        assert_eq!(native_transition_table("⋈").unwrap(), core::array::from_fn(|i| i as u8));
        for (word, evidence) in [("⊤", 8u8), ("⊥", 4), ("⊞", 3)] {
            assert_eq!(native_transition_table(word).unwrap(), core::array::from_fn(|i| i as u8 | evidence));
        }
        assert_eq!(native_transition_table("∈⊤≺∋").unwrap(), [8; 16]);
        assert_eq!(native_transition_table("⊢∈≻⋈⊙⊤≻⋈⊥≺⋈⊞∋⊡⋈⊙⊣").unwrap(), [15; 16]);
        let seed = native_transition_table("⊙").unwrap();
        assert_eq!(seed[0], 8);
        assert_eq!(seed[1], 1); // This map is not information monotone.
        assert!(native_transition_table("⊡?").is_err());
    }
}
