#![allow(dead_code)]
/// Belnap FOUR truth values.
#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
#[repr(u8)]
pub enum B4 {
    N = 0, // None  — void, absence
    T = 1, // True  — affirmation
    F = 2, // False — negation
    B = 3, // Both  — paradox stabilized
}

impl B4 {
    pub fn name(self) -> &'static str {
        match self {
            B4::N => "N",
            B4::T => "T",
            B4::F => "F",
            B4::B => "B",
        }
    }

    pub const fn from_u8(v: u8) -> Self {
        match v & 0b11 {
            1 => B4::T,
            2 => B4::F,
            3 => B4::B,
            _ => B4::N,
        }
    }

    pub const fn to_u8(self) -> u8 {
        self as u8
    }

    /// Information-order meet: intersection of independently carried evidence.
    pub const fn meet(self, other: B4) -> B4 {
        B4::from_u8(self as u8 & other as u8)
    }

    /// Information-order join: union of independently carried evidence.
    pub const fn join(self, other: B4) -> B4 {
        B4::from_u8(self as u8 | other as u8)
    }

    /// Knowledge-order consensus (lub in ≤k): bitwise OR = join.
    /// T ⊗ F = B (both affirmations → paradox).
    pub fn band(self, other: B4) -> B4 {
        self.join(other)
    }

    /// Knowledge-order gullibility (glb in ≤k): bitwise AND = meet.
    /// T ⊕ F = N (no shared ground).
    pub fn bor(self, other: B4) -> B4 {
        self.meet(other)
    }

    /// Truth-functional conjunction — the bilattice's *other* axis.
    /// Mirrors `Belnap.lean` `band` and `belnap.py` `band`: F absorbs,
    /// N conjoined with B yields F. Distinct from `band` above, which is the knowledge
    /// consensus: T ∧ F = F here, where T ⊗ F = B there.
    pub const fn truth_and(self, other: B4) -> B4 {
        use B4::*;
        match (self, other) {
            (F, _) | (_, F) => F,
            (B, T) | (T, B) => B,
            (B, N) | (N, B) => F,
            (T, T) => T,
            (T, N) | (N, T) => N,
            (N, N) => N,
            (B, B) => B,
        }
    }

    /// Truth-functional disjunction. Mirrors `Belnap.lean` `bor`:
    /// T absorbs, N disjoined with B yields T. T ∨ F = T here, where T ⊕ F = N for `bor`.
    pub const fn truth_or(self, other: B4) -> B4 {
        use B4::*;
        match (self, other) {
            (T, _) | (_, T) => T,
            (B, F) | (F, B) => B,
            (B, N) | (N, B) => T,
            (F, F) => F,
            (F, N) | (N, F) => N,
            (N, N) => N,
            (B, B) => B,
        }
    }

    /// Negation: swap T↔F, preserve N and B.
    pub const fn bnot(self) -> B4 {
        let b = self as u8;
        B4::from_u8(((b & 1) << 1) | ((b & 2) >> 1))
    }

    /// Is this value paradox-stabilized (Both)?
    pub fn dialetheic(self) -> bool {
        self == B4::B
    }

    /// Is this value designated (T or B) in Belnap logic?
    pub fn designated(self) -> bool {
        matches!(self, B4::T | B4::B)
    }

    /// Knowledge-order comparison: self ≤k other.
    /// x ≤k y iff x ⊓k y = x, and the knowledge glb is `meet`.
    pub fn approx_le(self, other: B4) -> bool {
        self.meet(other) == self
    }

    /// Encode to WH2 pair: (is_true, is_false).
    pub fn to_wh2(self) -> (bool, bool) {
        let b = self as u8;
        ((b & 1) != 0, (b & 2) != 0)
    }

    /// Decode from WH2 pair.
    pub fn from_wh2(t: bool, f: bool) -> Self {
        B4::from_u8((t as u8) | ((f as u8) << 1))
    }
}


impl B4 {
    /// Exclusive disjunction in the FDE truth algebra, without designation.
    pub const fn truth_xor(self, other: Self) -> Self {
        self.truth_and(other.bnot()).truth_or(self.bnot().truth_and(other))
    }

    /// A numeral boundary admits only definite evidence. Neither and both
    /// remain resident and have no implicit Boolean interpretation.
    pub const fn definite(self) -> Option<bool> {
        match self { Self::T => Some(true), Self::F => Some(false), _ => None }
    }
}

impl B4 {
    /// Read the definite numerical parity slice. A glut must be handled by its
    /// retained conflict frame before a branch can request a numeral mark.
    pub fn parity(self) -> Option<char> {
        match self { Self::N => None, Self::T => Some('⊥'), Self::F => Some('⊤'),
            Self::B => panic!("held contradictory evidence requires its branch frame") }
    }
    pub fn from_parity(mark: char) -> Self {
        match mark { '⊥' => Self::T, '⊤' => Self::F,
            _ => panic!("invalid definite numeral parity") }
    }
}

impl B4 {
    pub fn label(self) -> &'static str { self.name() }
    /// Judgment glyphs have their own typed boundary. In particular they are
    /// not the numerical parity map (where even and odd use these two marks).
    pub fn glyph(self) -> char {
        match self { Self::T => '⊤', Self::F => '⊥', Self::B => '⊞', Self::N => '⊙' }
    }
    pub fn from_glyph(mark: char) -> Option<Self> {
        match mark { '⊤' => Some(Self::T), '⊥' => Some(Self::F),
            '⊞' => Some(Self::B), '⊙' => Some(Self::N), _ => None }
    }
}
