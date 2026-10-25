//! Shared canonical boundary dissolution, used by Vox and MoDoT.
extern crate alloc;
use self::alloc::vec::Vec;

fn retain(word: &[char], mut emit: impl FnMut(usize, char)) {
    let last_fuse = word.iter().rposition(|&mark| mark == '∋');
    let last_anchor = word.iter().rposition(|&mark| mark == '⊣');
    let (mut source_seen, mut fork_seen) = (false, false);
    for (position, &mark) in word.iter().enumerate() {
        let keep = match mark {
            '⊢' => { let first = !source_seen; source_seen = true; first }
            '∈' => { let first = !fork_seen; fork_seen = true; first }
            '∋' => Some(position) == last_fuse,
            '⊣' => Some(position) == last_anchor,
            _ => true,
        };
        if keep { emit(position, mark); }
    }
}

pub fn collapse_to_nested(word: &[char]) -> Vec<char> {
    let mut result = Vec::with_capacity(word.len());
    retain(word, |_, mark| result.push(mark));
    result
}

pub fn retained_positions(word: &[char]) -> Vec<usize> {
    let mut result = Vec::with_capacity(word.len());
    retain(word, |position, _| result.push(position));
    result
}

/// Compose complete Type programs under one frame. Their final fuse closes
/// after all contained work; component readout tails stay inside that frame.
pub fn enclose_type_programs(word: &[char]) -> (Vec<char>, Vec<usize>) {
    let mut nested = collapse_to_nested(word);
    let mut positions = retained_positions(word);
    nested.retain(|&mark| mark != '∋' && mark != '⊣');
    positions.retain(|&position| word[position] != '∋' && word[position] != '⊣');
    for boundary in ['∋', '⊣'] {
        if let Some(position) = word.iter().rposition(|&mark| mark == boundary) {
            nested.push(boundary);
            positions.push(position);
        }
    }
    (nested, positions)
}
