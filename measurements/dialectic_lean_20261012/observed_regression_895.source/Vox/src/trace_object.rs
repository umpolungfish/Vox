//! trace_object.rs — the trace as an executable object. Each record carries the
//! operation that produced the transition, not just the reading, length-delimited.
use alloc::string::String;
use alloc::vec::Vec;

const BIT1: char = '\u{22A4}';   // ⊤ = 1
const BIT0: char = '\u{22A5}';   // ⊥ = 0

/// One trace record: the reading (judgment, source, flag) plus the operation word.
#[derive(Clone, PartialEq, Debug)]
pub struct TraceRecord {
    pub judgment: char,
    pub source: char,
    pub flag: char,          // ⊤ recognised / ⊥ unrecognised
    pub word: String,        // the transformation that was applied
}

/// encode := (J S F <len8> word)*  — same length-delimited technique as the router.
pub fn encode_trace(recs: &[TraceRecord]) -> String {
    let mut s = String::new();
    for r in recs {
        s.push(r.judgment); s.push(r.source); s.push(r.flag);
        let n = r.word.chars().count();
        for bit in (0..8).rev() { s.push(if (n >> bit) & 1 == 1 { BIT1 } else { BIT0 }); }
        s.push_str(&r.word);
    }
    s
}

pub fn decode_trace(s: &str) -> Option<Vec<TraceRecord>> {
    let c: Vec<char> = s.chars().collect();
    let mut i = 0usize;
    let mut out = Vec::new();
    while i < c.len() {
        if i + 3 > c.len() { return None; }
        let judgment = c[i]; let source = c[i + 1]; let flag = c[i + 2]; i += 3;
        let mut len = 0usize;
        for _ in 0..8 {
            let b = *c.get(i)?; i += 1;
            len = (len << 1) | match b { BIT1 => 1usize, BIT0 => 0usize, _ => return None };
        }
        let mut word = String::new();
        for _ in 0..len { word.push(*c.get(i)?); i += 1; }
        out.push(TraceRecord { judgment, source, flag, word });
    }
    Some(out)
}
