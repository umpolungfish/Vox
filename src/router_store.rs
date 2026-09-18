//! router_store.rs — resident execution. The rewrite is a WORD run on a store.
//!
//! The privileged `rewrite()` (Rust appends a clause) is replaced by a resident tape:
//!
//!     IStore { tape: [ROUTER marks][REF marks], cut, cursor, out, armed, committed }
//!     execute(op_word, &mut store, probe)     // GENERIC tape ops
//!
//! `op_word` is an ordinary IMASM word over the twelve marks. Its ops are GENERIC —
//! the machine never names a router part:
//!
//!     ⊢ VINIT    cursor = 0
//!     ⊣ TANCH    halt
//!     ≻ AFWD     cursor += 1
//!     ≺ AREV     cursor -= 1 (saturating)
//!     ⊞ ENGAGR   arm: tape[cursor..cursor+2] == probe
//!     ⋈ CLINK    if armed, capture the balanced word starting at cursor-1 into `out`
//!     ⊡ IFIX     if `out` nonempty, splice it in at the router/ref boundary (commit)
//!     ⊤ ⊥ ∈ ∋ ⊙  deposits / frames / copy-through (generic reading)
//!
//! The word never names a clause; it scans the reference region for the probe and
//! appends what it finds. This machine can move bytes between two tape regions and
//! nothing more — so the privilege cannot re-enter here disguised as an opcode.
//!
//! Discovery: all three host transformations (PRESERVE / EXPOSE / DISTINGUISH)
//! collapse to ONE generic word, `⊢≻⊞⋈⊡`, under different probes.

use alloc::string::String;
use alloc::vec::Vec;

pub type Mark = char;

pub struct IStore {
    pub tape: Vec<Mark>,
    pub cut: usize,   // router region = tape[0..cut]; ref region = tape[cut..]
    pub cursor: usize,
    pub armed: bool,
    pub out: Vec<Mark>,
    pub committed: bool,
}

impl IStore {
    /// Load a resident store: the router object's word followed by the reference word.
    pub fn new(router: &str, reference: &str) -> IStore {
        let mut tape: Vec<Mark> = router.chars().collect();
        let cut = tape.len();
        tape.extend(reference.chars());
        IStore { tape, cut, cursor: 0, armed: false, out: Vec::new(), committed: false }
    }

    /// The router region after execution — the object that resulted.
    pub fn router_word(&self) -> String { self.tape[..self.cut].iter().collect() }

    /// Splice `out` in at the router/ref boundary and grow the router region.
    fn commit(&mut self) {
        let n = self.out.len();
        let tail: Vec<Mark> = self.tape.split_off(self.cut);
        self.tape.extend(self.out.drain(..));
        self.tape.extend(tail);
        self.cut += n;
        self.committed = true;
    }

    /// Capture the depth-balanced word beginning at `start` (an `∈` … its matching `∋`).
    pub fn capture_balanced(&mut self, start: usize) {
        self.out.clear();
        let mut depth = 0i32;
        let mut j = start;
        while j < self.tape.len() {
            match self.tape[j] {
                '∈' => depth += 1,
                '∋' => { depth -= 1; if depth == 0 { self.out.extend_from_slice(&self.tape[start..=j]); return; } }
                _ => {}
            }
            j += 1;
        }
        // no balanced close found: capture nothing
    }
}

/// Execute `op` on the resident store, looping the word as a ring until it commits or
/// runs off the tape. `probe` is the two-mark pair the word is looking for (the reading
/// the process handed in). Returns whether a commit occurred.
pub fn execute(op: &[Mark], store: &mut IStore, probe: [Mark; 2]) -> bool {
    store.cursor = 0;
    store.out.clear();
    store.committed = false;
    let mut ip = 0usize;
    let mut budget = 8192usize;
    while budget > 0 {
        if ip >= op.len() {
            if store.committed || store.cursor >= store.tape.len() { break; }
            ip = 0; budget -= 1; continue;
        }
        match op[ip] {
            '⊢' => store.cursor = 0,
            '⊣' => break,
            '≻' => store.cursor += 1,
            '≺' => store.cursor = store.cursor.saturating_sub(1),
            '⊞' => store.armed = store.cursor + 1 < store.tape.len()
                        && store.tape[store.cursor] == probe[0]
                        && store.tape[store.cursor + 1] == probe[1],
            '⋈' => if store.armed && store.cursor >= 1 { store.capture_balanced(store.cursor - 1); },
            '⊡' => if !store.out.is_empty() { store.commit(); },
            _ => {}
        }
        ip += 1;
        budget -= 1;
    }
    store.committed
}

/// PRESERVE — halt at once, out stays empty, the router is returned unchanged.
pub const OP_PRESERVE: &str = "⊢⊣";

/// SCAN-APPEND — the single generic transformation: advance until the probe matches the
/// (judgment, source) of a reference clause, capture that balanced clause, splice it in.
/// EXPOSE (probe = B × exposed repr) and DISTINGUISH (probe = N × any) are the SAME word.
pub const OP_SCAN_APPEND: &str = "≻⊞⋈⊡";
