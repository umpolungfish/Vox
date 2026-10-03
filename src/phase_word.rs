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

#[derive(Debug)]
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
    if word.is_empty()
        || word.chars().any(|m| {
            !matches!(
                m,
                '⊢' | '⊣' | '≻' | '≺' | '⋈' | '⊙' | '∈' | '∋' | '⊤' | '⊥' | '⊞' | '⊡'
            )
        })
    {
        return Err("invalid phase execution word".into());
    }
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
    mut payload: impl FnMut(char, &T) -> Result<(), String>,
    mut address: impl FnMut(u8) -> Option<usize>,
) -> Result<Readout<T>, String> {
    let mut out = Readout {
        register: 0,
        surviving: Vec::new(),
        cleared: 0,
        restored: 0,
        exposed: 0,
        deposits: 0,
        seeded: 0,
        inert: 0,
        restore_supports: Vec::new(),
    };
    let mut frames: Vec<Vec<Deposit<T>>> = Vec::new();
    let mut fixed = false;
    let mut next_id = 0;
    let mut bound_ids = alloc::collections::BTreeMap::new();
    for mark in word {
        if !matches!(
            mark,
            '⊢' | '⊣' | '≻' | '≺' | '⋈' | '⊙' | '∈' | '∋' | '⊤' | '⊥' | '⊞' | '⊡'
        ) {
            return Err("invalid resident protocol mark".into());
        }
        if fixed && mark != '⊡' && mark != '⊙' {
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
                            let id = next_id;
                            next_id += 1;
                            id
                        })
                    } else {
                        let id = next_id;
                        next_id += 1;
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
            '⊡' => fixed = true,
            '⋈' | '⊣' => (),
            _ => unreachable!(),
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    const WORD: &str = "⊢≻∈⊤⊥⊞⋈≺∋⊙⊡⊣";

    #[test]
    fn resident_slots_fold_return_passes_and_restore_the_random_2048_payload() {
        let payload = vox::morphism_factor::parse_numeral(
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
}
