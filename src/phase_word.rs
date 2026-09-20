//! Observation transport under the sequential SIXTEEN_3 register semantics.
//! Structural graph closure is a separate judgment. This evaluator records
//! deposits and their payloads; a seeded register never invents an observation.

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
}

fn union<T: Clone>(target: &mut Vec<Deposit<T>>, source: &[Deposit<T>]) -> usize {
    let before = target.len();
    for deposit in source {
        if !target.iter().any(|d| d.id == deposit.id) { target.push(deposit.clone()); }
    }
    target.len() - before
}

pub fn execute<T: Clone>(word: &str, observation: &T) -> Result<Readout<T>, String> {
    // Validate before execution, including marks that would be inert after ⊡.
    if word.is_empty() || word.chars().any(|m| !"⊢⊣≻≺⋈⊙∈∋⊤⊥⊞⊡".contains(m)) {
        return Err("invalid phase execution word".into());
    }
    let mut out = Readout { register: 0, surviving: Vec::new(), cleared: 0,
        restored: 0, exposed: 0, deposits: 0, seeded: 0, inert: 0 };
    let mut frames: Vec<Vec<Deposit<T>>> = Vec::new();
    let mut fixed = false;
    let mut next_id = 0;
    for mark in word.chars() {
        if fixed && mark != '⊡' && mark != '⊙' { out.inert += 1; continue; }
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
                if mark == '⊢' { frames.clear(); }
            }
            '≻' | '⊙' => {
                if out.register == 0 { out.register = 1; out.seeded += 1; }
            }
            '∈' => frames.push(Vec::new()),
            '∋' => {
                if let Some(frame) = frames.pop() {
                    out.restored += union(&mut out.surviving, &frame);
                    for d in &frame { out.register |= d.lane; }
                    if let Some(parent) = frames.last_mut() { union(parent, &frame); }
                }
            }
            '⊤' | '⊥' | '⊞' => {
                out.deposits += 1;
                let lanes: &[u8] = match mark { '⊤' => &[1], '⊥' => &[2], _ => &[4, 8] };
                for &lane in lanes {
                    let d = Deposit { id: next_id, lane, observation: observation.clone() };
                    next_id += 1;
                    out.register |= lane;
                    if let Some(frame) = frames.last_mut() { frame.push(d.clone()); }
                    out.surviving.push(d);
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
    fn all_cuts_match_supplied_weight_trace() {
        let marks: Vec<char> = WORD.chars().collect();
        let payload = vec![0usize, 4, 8, 12];
        for k in 0..marks.len() {
            let word: String = marks[k..].iter().chain(&marks[..k]).collect();
            let r = execute(&word, &payload).unwrap();
            let holds = [0, 1, 2, 11].contains(&k);
            assert_eq!(r.register, if holds {15} else {1}, "cut {k}");
            assert_eq!(r.surviving.len(), if holds {4} else {0}, "cut {k}");
            assert_eq!(r.restored, if holds {4} else {0}, "cut {k}");
            assert_eq!(r.exposed, match k {3=>4,4=>3,5=>2,_=>0}, "cut {k}");
            for d in r.surviving { assert_eq!(d.observation, payload); }
        }
    }

    #[test]
    fn nested_frames_restore_once_and_seed_has_no_payload() {
        let r = execute("∈∈⊤∋≺∋", &42).unwrap();
        assert_eq!(r.surviving.len(), 1);
        assert_eq!(r.restored, 1);
        assert_eq!(r.exposed, 0);
        assert!(execute("⊙⊡", &42).unwrap().surviving.is_empty());
        assert!(execute("⊡?", &42).is_err());
        assert!(execute("", &42).is_err());
    }
}
