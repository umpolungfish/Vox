//! Dynamically sized running-product factor membranes in binary and arbitrary radix.
//!
//! The membrane carries N and the two low-bit factor prefixes as IMASM numeral
//! tapes. At width k, bit_k(P_k Q_k) contains the resolved lower diagonals and
//! their carries. The next bits obey p_k XOR q_k = n_k XOR bit_k(P_k Q_k).

use alloc::vec::Vec;

const ZERO: char = '⊤';
const ONE: char = '⊥';

/// One lossless re-chunking of the LSB-first base encoding. A group is the
/// joint state of `window` two-valued registers; a short final group is kept
/// at its actual width.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FrameSweep {
    pub window: usize,
    pub groups: Vec<Vec<char>>,
    pub tail_len: usize,
}

impl FrameSweep {
    pub fn reconstruct(&self) -> Vec<char> {
        self.groups.iter().flat_map(|group| group.iter().copied()).collect()
    }
}

/// Produce the seven register decompositions for windows 2 through 8.
pub fn frame_sweep(tape: &[char]) -> Vec<FrameSweep> {
    (2..=8).map(|window| {
        let groups: Vec<Vec<char>> = tape.chunks(window).map(|g| g.to_vec()).collect();
        let rem = tape.len() % window;
        FrameSweep { window, groups, tail_len: if rem == 0 { window } else { rem } }
    }).collect()
}

fn trim(mut tape: Vec<char>) -> Vec<char> {
    while tape.last() == Some(&ZERO) { tape.pop(); }
    tape
}

fn bit(tape: &[char], index: usize) -> u8 {
    u8::from(tape.get(index) == Some(&ONE))
}

fn set_bit(tape: &mut Vec<char>, index: usize, value: u8) {
    if tape.len() <= index { tape.resize(index + 1, ZERO); }
    tape[index] = if value == 0 { ZERO } else { ONE };
}

fn cmp(a: &[char], b: &[char]) -> core::cmp::Ordering {
    let a = trim(a.to_vec());
    let b = trim(b.to_vec());
    match a.len().cmp(&b.len()) {
        core::cmp::Ordering::Equal => {
            for i in (0..a.len()).rev() {
                match bit(&a, i).cmp(&bit(&b, i)) {
                    core::cmp::Ordering::Equal => {},
                    order => return order,
                }
            }
            core::cmp::Ordering::Equal
        }
        order => order,
    }
}

/// Exact product of two LSB-first tapes through the shared dynamic limb fold.
/// The membrane state retains only the factor tapes and prefix width.
fn multiply(a: &[char], b: &[char]) -> Vec<char> {
    crate::morphism_factor::mul(a, b)
}

fn multiply_all(factors: &[Vec<char>]) -> Vec<char> {
    factors.iter().fold(alloc::vec![ONE], |product, factor| multiply(&product, factor))
}

/// The r-register version of the running-product membrane. Its next-bit
/// parity is the XOR of the r new factor bits; the caller selects among the
/// admissible joint states.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FoldMembrane {
    pub n: Vec<char>,
    pub factors: Vec<Vec<char>>,
    pub width: usize,
}

impl FoldMembrane {
    pub fn new(n: Vec<char>, arity: usize) -> Result<Self, &'static str> {
        let n = trim(n);
        if n.len() < 2 || bit(&n, 0) != 1 { return Err("fold membrane requires odd N > 1"); }
        if arity < 2 { return Err("fold membrane requires at least two registers"); }
        let state = Self { n, factors: alloc::vec![alloc::vec![ONE]; arity], width: 1 };
        if !state.prefix_matches() { return Err("fold seed does not close on N"); }
        Ok(state)
    }

    pub fn resolved_product_bit(&self) -> u8 {
        bit(&multiply_all(&self.factors), self.width)
    }

    pub fn next_xor(&self) -> u8 {
        bit(&self.n, self.width) ^ self.resolved_product_bit()
    }

    /// Both next-bit pairs satisfying the 2-adic parity equation. The
    /// recurrence constrains parity and leaves one binary branch choice.
    pub fn admissible_next_pairs(&self) -> [(u8, u8); 2] {
        if self.next_xor() == 0 { [(0, 0), (1, 1)] } else { [(0, 1), (1, 0)] }
    }

    pub fn is_fixed_point(&self) -> bool { multiply_all(&self.factors) == self.n }

    fn prefix_matches(&self) -> bool {
        let product = multiply_all(&self.factors);
        (0..self.width).all(|i| bit(&product, i) == bit(&self.n, i))
    }

    pub fn extend(&self, next_bits: &[u8]) -> Result<Self, &'static str> {
        if next_bits.len() != self.factors.len() {
            return Err("joint state must provide one bit per factor register");
        }
        let parity = next_bits.iter().fold(0, |acc, &b| acc ^ (b & 1));
        if parity != self.next_xor() { return Err("joint state violates running-product parity"); }
        let mut factors = self.factors.clone();
        for (factor, &new_bit) in factors.iter_mut().zip(next_bits) {
            set_bit(factor, self.width, new_bit & 1);
            *factor = trim(core::mem::take(factor));
        }
        let next = Self { n: self.n.clone(), factors, width: self.width + 1 };
        if !next.prefix_matches() { return Err("joint extension does not close modulo the new width"); }
        Ok(next)
    }

    /// Extend every factor register by the same multi-bit frame.
    ///
    /// `blocks[i][j]` is bit `width + j` of factor register `i`. Checking the
    /// product modulo the final width also closes every intermediate prefix,
    /// since each is the reduction of that final congruence.
    pub fn extend_block(&self, blocks: &[Vec<u8>]) -> Result<Self, &'static str> {
        if blocks.len() != self.factors.len() {
            return Err("joint block must provide one bit-vector per factor register");
        }
        let block_width = blocks.first().map_or(0, Vec::len);
        if block_width == 0 || blocks.iter().any(|block| block.len() != block_width) {
            return Err("every factor register must provide the same nonempty block width");
        }
        if blocks.iter().flatten().any(|&value| value > 1) {
            return Err("factor-register blocks contain only binary digits");
        }

        let mut factors = self.factors.clone();
        for (factor, block) in factors.iter_mut().zip(blocks) {
            for (offset, &value) in block.iter().enumerate() {
                set_bit(factor, self.width + offset, value);
            }
            *factor = trim(core::mem::take(factor));
        }
        let next = Self {
            n: self.n.clone(),
            factors,
            width: self.width + block_width,
        };
        if !next.prefix_matches() {
            return Err("extended factor blocks do not close modulo the new width");
        }
        Ok(next)
    }
}

fn parity_states(arity: usize, target: u8) -> Vec<Vec<u8>> {
    fn descend(arity: usize, target: u8, state: &mut Vec<u8>, parity: u8, out: &mut Vec<Vec<u8>>) {
        if state.len() == arity {
            if parity == target { out.push(state.clone()); }
            return;
        }
        state.push(0);
        descend(arity, target, state, parity, out);
        state.pop();
        state.push(1);
        descend(arity, target, state, parity ^ 1, out);
        state.pop();
    }
    let mut out = Vec::new();
    descend(arity, target, &mut Vec::new(), 0, &mut out);
    out
}

/// Search exact r-fold product closures. The output is a factor tuple, not a
/// primality certificate; each accepted tuple multiplies exactly to N.
pub fn factor_2adic_n(
    n: &[char], arity: usize, max_solutions: Option<usize>,
) -> Vec<Vec<Vec<char>>> {
    let n = trim(n.to_vec());
    if arity < 2 || n.len() < 2 || bit(&n, 0) == 0 || max_solutions == Some(0) {
        return Vec::new();
    }
    let Ok(seed) = FoldMembrane::new(n.clone(), arity) else { return Vec::new(); };
    let mut stack = alloc::vec![seed];
    let mut out = Vec::new();
    while let Some(state) = stack.pop() {
        if max_solutions.is_some_and(|cap| out.len() >= cap) { break; }
        let product = multiply_all(&state.factors);
        match cmp(&product, &n) {
            core::cmp::Ordering::Equal => {
                if state.factors.iter().all(|factor| factor.len() > 1) {
                    let mut factors = state.factors;
                    factors.sort_unstable_by(|a, b| cmp(a, b));
                    out.push(factors);
                }
                continue;
            }
            core::cmp::Ordering::Greater => continue,
            core::cmp::Ordering::Less => {}
        }
        if state.width >= n.len() { continue; }
        let parity = state.next_xor();
        for bits in parity_states(arity, parity) {
            if let Ok(next) = state.extend(&bits) { stack.push(next); }
        }
    }
    out.sort_unstable_by(|a, b| {
        for (left, right) in a.iter().zip(b) {
            let order = cmp(left, right);
            if order != core::cmp::Ordering::Equal { return order; }
        }
        a.len().cmp(&b.len())
    });
    out.dedup();
    out
}

fn product_bit(a: &[char], b: &[char], index: usize) -> u8 {
    bit(&multiply(a, b), index)
}

fn product_matches_prefix(a: &[char], b: &[char], n: &[char], width: usize) -> bool {
    let product = multiply(a, b);
    (0..width).all(|i| bit(&product, i) == bit(n, i))
}

/// A prefix lift whose modulus grows in an arbitrary baked radix. Numerals
/// remain LSB-first IMASM tapes; radix digits and powers are themselves tapes,
/// so the lift has no machine-word width limit.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RadixPrefixMembrane {
    pub n: Vec<char>,
    pub radix: Vec<char>,
    pub p: Vec<char>,
    pub q: Vec<char>,
    pub width: usize,
    pub place: Vec<char>,
}

impl RadixPrefixMembrane {
    pub fn new(n: Vec<char>, radix: Vec<char>) -> Result<Self, &'static str> {
        let n = trim(n);
        let radix = trim(radix);
        if cmp(&radix, &[ONE]) != core::cmp::Ordering::Greater {
            return Err("lift radix must be greater than one");
        }
        if n.is_empty() { return Err("radix prefix membrane requires a nonzero N"); }
        Ok(Self { n, radix, p: alloc::vec![ZERO], q: alloc::vec![ZERO], width: 0, place: alloc::vec![ONE] })
    }

    fn digit_in_range(&self, digit: &[char]) -> bool {
        cmp(digit, &self.radix) == core::cmp::Ordering::Less
    }

    /// Check the generalized radix-digit equation at the next register.
    /// At the seed, p₀q₀ must match N modulo B. For k ≥ 1 this checks
    /// pₖQₖ + qₖPₖ ≡ nₖ − digitₖ(PₖQₖ) (mod B).
    pub fn candidate_matches_next_digit(&self, p_digit: &[char], q_digit: &[char]) -> bool {
        if !self.digit_in_range(p_digit) || !self.digit_in_range(q_digit) { return false; }
        if self.width == 0 {
            return crate::morphism_factor::modulo(&multiply(p_digit, q_digit), &self.radix)
                == crate::morphism_factor::modulo(&self.n, &self.radix);
        }

        let input_digits = match radix_digits(&self.n, &self.radix) { Ok(digits) => digits, Err(_) => return false };
        let zero = alloc::vec![ZERO];
        let target_digit = input_digits.get(self.width).map(Vec::as_slice).unwrap_or(&zero);
        self.candidate_matches_digit(p_digit, q_digit, target_digit)
    }

    fn candidate_matches_digit(&self, p_digit: &[char], q_digit: &[char], target_digit: &[char]) -> bool {
        if !self.digit_in_range(p_digit) || !self.digit_in_range(q_digit) { return false; }
        if self.width == 0 {
            return crate::morphism_factor::modulo(&multiply(p_digit, q_digit), &self.radix)
                == crate::morphism_factor::modulo(&self.n, &self.radix);
        }
        let product = multiply(&self.p, &self.q);
        let high_product = crate::morphism_factor::divmod(&product, &self.place).0;
        let product_digit = crate::morphism_factor::modulo(&high_product, &self.radix);
        let required = if cmp(target_digit, &product_digit) != core::cmp::Ordering::Less {
            crate::morphism_factor::sub(target_digit, &product_digit)
        } else {
            crate::morphism_factor::sub(&self.radix,
                &crate::morphism_factor::sub(&product_digit, target_digit))
        };
        let p_low = crate::morphism_factor::modulo(&self.p, &self.radix);
        let q_low = crate::morphism_factor::modulo(&self.q, &self.radix);
        let coefficient = crate::morphism_factor::add(
            &multiply(p_digit, &q_low), &multiply(q_digit, &p_low));
        crate::morphism_factor::modulo(&coefficient, &self.radix) == required
    }

    /// Add the next pair of radix digits and preserve N modulo radix^(k+1).
    pub fn extend(&self, p_digit: &[char], q_digit: &[char]) -> Result<Self, &'static str> {
        let target_digits = radix_digits(&self.n, &self.radix)?;
        let zero = alloc::vec![ZERO];
        let target_digit = target_digits.get(self.width).unwrap_or(&zero);
        self.extend_for_digit(p_digit, q_digit, target_digit)
    }

    fn extend_for_digit(&self, p_digit: &[char], q_digit: &[char], target_digit: &[char]) -> Result<Self, &'static str> {
        if !self.candidate_matches_digit(p_digit, q_digit, target_digit) {
            return Err("factor digits violate the running-product radix equation");
        }
        let p = crate::morphism_factor::add(&self.p, &multiply(p_digit, &self.place));
        let q = crate::morphism_factor::add(&self.q, &multiply(q_digit, &self.place));
        Ok(Self {
            n: self.n.clone(), radix: self.radix.clone(), p, q,
            width: self.width + 1, place: multiply(&self.place, &self.radix),
        })
    }

    pub fn is_fixed_point(&self) -> bool { multiply(&self.p, &self.q) == self.n }
}

/// LSB-first radix digits represented as dynamically sized numeral tapes.
pub fn radix_digits(value: &[char], radix: &[char]) -> Result<Vec<Vec<char>>, &'static str> {
    if cmp(radix, &[ONE]) != core::cmp::Ordering::Greater {
        return Err("digit radix must be greater than one");
    }
    let mut remaining = trim(value.to_vec());
    if remaining.is_empty() { return Ok(alloc::vec![alloc::vec![ZERO]]); }
    let mut digits = Vec::new();
    while !remaining.is_empty() {
        let (quotient, remainder) = crate::morphism_factor::divmod(&remaining, radix);
        digits.push(remainder);
        remaining = trim(quotient);
    }
    Ok(digits)
}

/// Fold a known pair through every radix-prefix register. The returned state
/// carries the inductive digit closure; callers choose when to perform exact
/// product closure so both nesting orders can share this deterministic fold.
pub fn radix_prefix_fold(n: &[char], p: &[char], q: &[char], radix: &[char]) -> Option<RadixPrefixMembrane> {
    let (Ok(pd), Ok(qd)) = (radix_digits(p, radix), radix_digits(q, radix)) else { return None; };
    let Ok(nd) = radix_digits(n, radix) else { return None; };
    let Ok(mut state) = RadixPrefixMembrane::new(n.to_vec(), radix.to_vec()) else { return None; };
    let width = pd.len().max(qd.len());
    let zero = alloc::vec![ZERO];
    for index in 0..width {
        let p_digit = pd.get(index).unwrap_or(&zero);
        let q_digit = qd.get(index).unwrap_or(&zero);
        let target_digit = nd.get(index).unwrap_or(&zero);
        let Ok(next) = state.extend_for_digit(p_digit, q_digit, target_digit) else { return None; };
        state = next;
    }
    Some(state)
}

/// Close a known pair through every radix-prefix register and the exact
/// terminal product fixed point.
pub fn radix_prefix_closes(n: &[char], p: &[char], q: &[char], radix: &[char]) -> bool {
    radix_prefix_fold(n, p, q, radix).is_some_and(|state| state.is_fixed_point())
}

/// One state contains only the input and the two factor prefixes. Width is the
/// number of low bits currently fixed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PrefixMembrane {
    pub n: Vec<char>,
    pub p: Vec<char>,
    pub q: Vec<char>,
    pub width: usize,
}

impl PrefixMembrane {
    /// Open on an encoded, odd numeral greater than one.
    pub fn new(n: Vec<char>) -> Result<Self, &'static str> {
        let n = trim(n);
        if n.len() < 2 || bit(&n, 0) != 1 {
            return Err("the prefix membrane requires an encoded odd N > 1");
        }
        let state = Self { n, p: alloc::vec![ONE], q: alloc::vec![ONE], width: 1 };
        if !product_matches_prefix(&state.p, &state.q, &state.n, 1) {
            return Err("the one-bit seed does not close on N");
        }
        Ok(state)
    }

    /// b_k = bit_k(P_k Q_k), read from the running product.
    pub fn resolved_product_bit(&self) -> u8 {
        product_bit(&self.p, &self.q, self.width)
    }

    /// Required parity of the two next factor bits.
    pub fn next_xor(&self) -> u8 {
        bit(&self.n, self.width) ^ self.resolved_product_bit()
    }

    /// Both next-bit pairs satisfying the 2-adic parity equation. The
    /// recurrence constrains parity and leaves one binary branch choice.
    pub fn admissible_next_pairs(&self) -> [(u8, u8); 2] {
        if self.next_xor() == 0 { [(0, 0), (1, 1)] } else { [(0, 1), (1, 0)] }
    }

    /// The membrane closes when its running product is exactly the baked N.
    pub fn is_fixed_point(&self) -> bool {
        multiply(&self.p, &self.q) == self.n
    }

    /// Apply one admissible factor-bit pair and verify the next prefix closure.
    pub fn extend(&self, p_bit: u8, q_bit: u8) -> Result<Self, &'static str> {
        if (p_bit ^ q_bit) != self.next_xor() {
            return Err("factor-bit pair does not satisfy the running-product law");
        }
        let mut p = self.p.clone();
        let mut q = self.q.clone();
        set_bit(&mut p, self.width, p_bit);
        set_bit(&mut q, self.width, q_bit);
        let next = Self { n: self.n.clone(), p: trim(p), q: trim(q), width: self.width + 1 };
        if !product_matches_prefix(&next.p, &next.q, &next.n, next.width) {
            return Err("extended prefixes do not close modulo the new width");
        }
        Ok(next)
    }

    /// Lift a whole register frame in one transition. The final closure check
    /// implies closure at each bit width within the frame.
    pub fn extend_block(&self, p_bits: &[u8], q_bits: &[u8]) -> Result<Self, &'static str> {
        if p_bits.is_empty() || p_bits.len() != q_bits.len() {
            return Err("both factor registers must provide the same nonempty block width");
        }
        if p_bits.iter().chain(q_bits).any(|&value| value > 1) {
            return Err("factor-register blocks contain only binary digits");
        }
        let mut p = self.p.clone();
        let mut q = self.q.clone();
        for (offset, (&p_bit, &q_bit)) in p_bits.iter().zip(q_bits).enumerate() {
            set_bit(&mut p, self.width + offset, p_bit);
            set_bit(&mut q, self.width + offset, q_bit);
        }
        let next = Self {
            n: self.n.clone(),
            p: trim(p),
            q: trim(q),
            width: self.width + p_bits.len(),
        };
        if !product_matches_prefix(&next.p, &next.q, &next.n, next.width) {
            return Err("extended factor blocks do not close modulo the new width");
        }
        Ok(next)
    }
}

/// Search the comultiplicative prefix circuit until multiplication closes on N.
/// The branch state is only (N, P_k, Q_k, k); every accepted terminal is an
/// exact product fixed point, not a factor candidate checked afterward.
pub fn factor_2adic(n: &[char], max_solutions: Option<usize>) -> Vec<(Vec<char>, Vec<char>)> {
    let n = trim(n.to_vec());
    if n.len() < 2 || bit(&n, 0) == 0 || max_solutions == Some(0) { return Vec::new(); }
    let width_limit = n.len();
    let mut out = Vec::new();

    let Ok(seed) = PrefixMembrane::new(n.clone()) else { return out; };
    let mut stack = alloc::vec![seed];
    while let Some(state) = stack.pop() {
        if max_solutions.is_some_and(|cap| out.len() >= cap) { break; }

        let product = multiply(&state.p, &state.q);
        match cmp(&product, &state.n) {
            core::cmp::Ordering::Equal => {
                if state.p.len() > 1 && state.q.len() > 1 {
                    let (p, q) = if cmp(&state.p, &state.q) == core::cmp::Ordering::Greater {
                        (state.q, state.p)
                    } else { (state.p, state.q) };
                    out.push((p, q));
                }
                continue;
            }
            core::cmp::Ordering::Greater => continue,
            core::cmp::Ordering::Less => {}
        }

        if state.width >= width_limit { continue; }
        // The returned pair is ordered small-first. Once P_k exceeds sqrt(N),
        // this branch cannot close in that orientation. The swapped dyadic
        // branch remains in the same circuit.
        if cmp(&multiply(&state.p, &state.p), &state.n) == core::cmp::Ordering::Greater {
            continue;
        }

        let candidates = state.admissible_next_pairs();
        // First escape the trivial P=1 lane; after P has grown, prefer holding
        // its high bits at zero so a small factor prefix stays small while Q
        // is determined by the recurrence.
        let preferred = if state.p.len() == 1 { 1 } else { 0 };
        let ordered = [1 - preferred, preferred];
        for p_bit in ordered {
            let q_bit = candidates.iter().find(|(candidate_p, _)| *candidate_p == p_bit).unwrap().1;
            if let Ok(next) = state.extend(p_bit, q_bit) {
                stack.push(next);
            }
        }
    }

    out.sort_unstable_by(|a, b| cmp(&a.0, &b.0).then_with(|| cmp(&a.1, &b.1)));
    out.dedup();
    out
}

/// Reverse nesting: the multiplication membrane owns the traversal state and
/// delegates each admissible bit extension to the comultiplicative prefix
/// membrane. `product` is resident alongside that prefix state and updated at
/// every edge, rather than requested by the prefix membrane when needed.
pub fn factor_2adic_multiplication_outer(
    n: &[char], max_solutions: Option<usize>,
) -> Vec<(Vec<char>, Vec<char>)> {
    #[derive(Clone)]
    struct ProductShell {
        prefix: PrefixMembrane,
        product: Vec<char>,
    }

    let n = trim(n.to_vec());
    if n.len() < 2 || bit(&n, 0) == 0 || max_solutions == Some(0) { return Vec::new(); }
    let Ok(seed) = PrefixMembrane::new(n.clone()) else { return Vec::new(); };
    let mut stack = alloc::vec![ProductShell { product: multiply(&seed.p, &seed.q), prefix: seed }];
    let mut out = Vec::new();

    while let Some(shell) = stack.pop() {
        if max_solutions.is_some_and(|cap| out.len() >= cap) { break; }
        match cmp(&shell.product, &n) {
            core::cmp::Ordering::Equal => {
                let state = shell.prefix;
                if state.p.len() > 1 && state.q.len() > 1 {
                    let (p, q) = if cmp(&state.p, &state.q) == core::cmp::Ordering::Greater {
                        (state.q, state.p)
                    } else { (state.p, state.q) };
                    out.push((p, q));
                }
                continue;
            }
            core::cmp::Ordering::Greater => continue,
            core::cmp::Ordering::Less => {}
        }
        if shell.prefix.width >= n.len()
            || cmp(&multiply(&shell.prefix.p, &shell.prefix.p), &n) == core::cmp::Ordering::Greater {
            continue;
        }

        let preferred = if shell.prefix.p.len() == 1 { 1 } else { 0 };
        let candidates = shell.prefix.admissible_next_pairs();
        for p_bit in [1 - preferred, preferred] {
            let q_bit = candidates.iter().find(|(candidate_p, _)| *candidate_p == p_bit).unwrap().1;
            if let Ok(prefix) = shell.prefix.extend(p_bit, q_bit) {
                let product = multiply(&prefix.p, &prefix.q);
                stack.push(ProductShell { prefix, product });
            }
        }
    }
    out.sort_unstable_by(|a, b| cmp(&a.0, &b.0).then_with(|| cmp(&a.1, &b.1)));
    out.dedup();
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn from_one_bits(indices: &[usize]) -> Vec<char> {
        let mut tape = Vec::new();
        for &index in indices { set_bit(&mut tape, index, 1); }
        trim(tape)
    }

    #[test]
    fn running_product_bit_selects_the_next_pair_parity() {
        let s = PrefixMembrane::new(from_one_bits(&[0, 1, 2, 3])).unwrap(); // N=15
        assert_eq!(s.resolved_product_bit(), 0);
        assert_eq!(s.next_xor(), 1);
        let next = s.extend(0, 1).unwrap();
        assert_eq!((bit(&next.p, 0), bit(&next.p, 1)), (1, 0));
        assert_eq!((bit(&next.q, 0), bit(&next.q, 1)), (1, 1));
        assert_eq!(next.width, 2);
        assert!(s.extend(1, 1).is_err());
        let closed = PrefixMembrane::new(from_one_bits(&[0, 1, 2, 3])).unwrap()
            .extend(1, 0).unwrap().extend(0, 1).unwrap(); // prefixes 3 and 5
        assert_eq!((closed.p.clone(), closed.q.clone()),
            (from_one_bits(&[0, 1]), from_one_bits(&[0, 2])));
        assert!(closed.is_fixed_point());
    }

    #[test]
    fn bit_parity_keeps_both_factor_swap_branches_under_equal_length_bounds() {
        let n = from_one_bits(&[0, 1, 5]); // 35 = 5 * 7, both factors three bits
        let seed = PrefixMembrane::new(n).unwrap();
        assert_eq!(seed.admissible_next_pairs(), [(0, 1), (1, 0)]);
        let closures: Vec<_> = seed.admissible_next_pairs().into_iter().map(|(pb, qb)| {
            seed.extend(pb, qb).unwrap().extend(1, 1).unwrap()
        }).collect();
        assert!(closures.iter().all(PrefixMembrane::is_fixed_point));
        assert_eq!(closures.iter().map(|s| (s.p.clone(), s.q.clone())).collect::<Vec<_>>(), vec![
            (from_one_bits(&[0, 2]), from_one_bits(&[0, 1, 2])),
            (from_one_bits(&[0, 1, 2]), from_one_bits(&[0, 2])),
        ]);
    }

    #[test]
    fn running_product_membrane_finds_small_factor_pairs() {
        let n15 = from_one_bits(&[0, 1, 2, 3]);
        assert_eq!(factor_2adic(&n15, None), vec![
            (from_one_bits(&[0, 1]), from_one_bits(&[0, 2])),
        ]);

        let n105 = from_one_bits(&[0, 3, 5, 6]);
        assert_eq!(factor_2adic(&n105, None), vec![
            (from_one_bits(&[0, 1]), from_one_bits(&[0, 1, 5])),
            (from_one_bits(&[0, 2]), from_one_bits(&[0, 2, 4])),
            (from_one_bits(&[0, 1, 2]), from_one_bits(&[0, 1, 2, 3])),
        ]);

        let n221 = from_one_bits(&[0, 2, 3, 4, 6, 7]);
        assert_eq!(factor_2adic(&n221, None), vec![
            (from_one_bits(&[0, 2, 3]), from_one_bits(&[0, 4])),
        ]);
        assert_eq!(factor_2adic_multiplication_outer(&n15, None), factor_2adic(&n15, None));
        assert_eq!(factor_2adic_multiplication_outer(&n105, None), factor_2adic(&n105, None));
        assert_eq!(factor_2adic_multiplication_outer(&n221, None), factor_2adic(&n221, None));
    }

    #[test]
    fn prefixes_scale_past_machine_word_width() {
        // 3 × (2^70 + 1), represented and multiplied only as bit tapes.
        let p = from_one_bits(&[0, 1]);
        let q = from_one_bits(&[0, 70]);
        let n = multiply(&p, &q);
        let factors = factor_2adic(&n, Some(1));
        assert_eq!(factors.len(), 1);
        assert_eq!(factors[0], (p, q));
        assert_eq!(factor_2adic_multiplication_outer(&n, Some(1)), factors);
    }

    #[test]
    fn frame_sweep_is_lossless_at_every_window() {
        let tape = from_one_bits(&[0, 2, 5, 8, 11]);
        let frames = frame_sweep(&tape);
        let mut flipped = tape.clone();
        let flipped_value = bit(&flipped, 5) ^ 1;
        set_bit(&mut flipped, 5, flipped_value);
        let flipped_frames = frame_sweep(&flipped);
        assert_eq!(frames.len(), 7);
        for (frame, changed_frame) in frames.iter().zip(flipped_frames.iter()) {
            assert_eq!(frame.reconstruct(), tape);
            assert_eq!(frame.groups.iter().map(Vec::len).sum::<usize>(), tape.len());
            let rem = tape.len() % frame.window;
            assert_eq!(frame.tail_len, if rem == 0 { frame.window } else { rem });
            let changed: Vec<_> = frame.groups.iter().zip(&changed_frame.groups).enumerate()
                .filter_map(|(i, (old, new))| (old != new).then_some(i)).collect();
            assert_eq!(changed, vec![5 / frame.window]);
            assert_eq!(changed_frame.reconstruct(), flipped);
        }
    }

    #[test]
    fn n_register_lift_closes_a_three_factor_product() {
        let p = from_one_bits(&[0, 1]);
        let q = from_one_bits(&[0, 2]);
        let r = from_one_bits(&[0, 1, 2]);
        let n = multiply_all(&[p.clone(), q.clone(), r.clone()]);
        let mut state = FoldMembrane::new(n.clone(), 3).unwrap();
        state = state.extend(&[1, 0, 1]).unwrap();
        state = state.extend(&[0, 1, 1]).unwrap();
        state = state.extend(&[0, 0, 0]).unwrap();
        assert_eq!(state.factors, vec![p, q, r]);
        assert!(state.is_fixed_point());
        assert_eq!(factor_2adic_n(&n, 3, Some(1)), vec![vec![
            from_one_bits(&[0, 1]), from_one_bits(&[0, 2]), from_one_bits(&[0, 1, 2]),
        ]]);
    }

    #[test]
    fn block_lift_matches_single_bit_pair_folding() {
        let p = from_one_bits(&[0, 2, 3]); // 13
        let q = from_one_bits(&[0, 4]); // 17
        let n = multiply(&p, &q);
        let seed = PrefixMembrane::new(n.clone()).unwrap();
        let p_block = [0, 1, 1, 0];
        let q_block = [0, 0, 0, 1];

        let folded = seed.extend_block(&p_block, &q_block).unwrap();
        let stepped = p_block.iter().zip(q_block).fold(seed, |state, (&pb, qb)| {
            state.extend(pb, qb).unwrap()
        });
        assert_eq!(folded, stepped);
        assert!(folded.is_fixed_point());

        assert!(PrefixMembrane::new(n).unwrap().extend_block(&[0, 2], &[0, 0]).is_err());
        assert!(PrefixMembrane::new(multiply(&p, &q)).unwrap()
            .extend_block(&[0, 1], &[0]).is_err());
    }

    #[test]
    fn framed_block_lift_scales_past_machine_word_width() {
        let p = from_one_bits(&[0, 1]);
        let q = from_one_bits(&[0, 70]);
        let n = multiply(&p, &q);
        let mut state = PrefixMembrane::new(n.clone()).unwrap();
        let mut start = state.width;
        while start < n.len() {
            let end = (start + 8).min(n.len());
            let p_block: Vec<u8> = (start..end).map(|i| bit(&p, i)).collect();
            let q_block: Vec<u8> = (start..end).map(|i| bit(&q, i)).collect();
            state = state.extend_block(&p_block, &q_block).unwrap();
            start = end;
        }
        assert_eq!(state.p, p);
        assert_eq!(state.q, q);
        assert!(state.is_fixed_point());
    }

    #[test]
    fn n_register_block_lift_closes_at_frame_boundary() {
        let factors = vec![
            from_one_bits(&[0, 2]), // 5
            from_one_bits(&[0, 1, 3]), // 11
            from_one_bits(&[0, 2, 4]), // 21
        ];
        let n = multiply_all(&factors);
        let seed = FoldMembrane::new(n, 3).unwrap();
        let blocks = vec![vec![0, 1, 0, 0], vec![1, 0, 1, 0], vec![0, 1, 0, 1]];
        let lifted = seed.extend_block(&blocks).unwrap();
        assert_eq!(lifted.factors, factors);
        assert!(lifted.is_fixed_point());
    }

    #[test]
    fn radix_prefix_fold_closes_in_multiple_dynamic_modular_bases() {
        let p = from_one_bits(&[0, 1, 2]); // 7
        let q = from_one_bits(&[0, 2, 3]); // 13
        let n = multiply(&p, &q);
        for radix in [
            from_one_bits(&[1]),       // 2
            from_one_bits(&[0, 1]),    // 3
            from_one_bits(&[0, 2]),    // 5
            from_one_bits(&[1, 3]),    // 10
            from_one_bits(&[0, 1, 3]), // 11
            from_one_bits(&[0, 80]),   // 2^80 + 1, beyond machine-word radix
        ] {
            assert!(radix_prefix_closes(&n, &p, &q, &radix));
        }
        let wrong_q = from_one_bits(&[0, 1, 3]); // 11
        assert!(!radix_prefix_closes(&n, &p, &wrong_q, &from_one_bits(&[0, 1])));
        assert!(RadixPrefixMembrane::new(n, alloc::vec![ONE]).is_err());
    }

    #[test]
    fn radix_lift_enforces_the_seed_and_running_digit_equations() {
        let p = from_one_bits(&[0, 1, 2]); // 7
        let q = from_one_bits(&[0, 2, 3]); // 13
        let n = multiply(&p, &q);
        let radix = from_one_bits(&[0, 1]); // 3
        let p_digits = radix_digits(&p, &radix).unwrap();
        let q_digits = radix_digits(&q, &radix).unwrap();
        let mut state = RadixPrefixMembrane::new(n, radix).unwrap();
        assert!(state.candidate_matches_next_digit(&p_digits[0], &q_digits[0]));
        assert!(!state.candidate_matches_next_digit(&[ZERO], &q_digits[0]));
        let zero = alloc::vec![ZERO];
        for index in 0..p_digits.len().max(q_digits.len()) {
            let p_digit = p_digits.get(index).unwrap_or(&zero);
            let q_digit = q_digits.get(index).unwrap_or(&zero);
            if index == 1 {
                assert!(!state.candidate_matches_next_digit(&from_one_bits(&[0]), q_digit));
            }
            state = state.extend(p_digit, q_digit).unwrap();
        }
        assert!(state.is_fixed_point());
    }

}
