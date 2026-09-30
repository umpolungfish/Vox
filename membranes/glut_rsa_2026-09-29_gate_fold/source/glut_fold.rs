//! Symbolic bit folds for the glut product relation. Free cells remain free;
//! interval and convolution rails constrain them before a branch materializes.
use super::{bit, mf, trim, ONE, ZERO};
use alloc::vec;
use alloc::vec::Vec;
use core::cmp::Ordering;
#[path = "glut_correlation.rs"]
mod correlation;

#[derive(Clone, Debug, Default)]
pub struct FoldStats {
    pub active: bool,
    pub decisions: usize,
    pub frontier: usize,
    pub peak_frontier: usize,
    pub cells: usize,
    pub peak_cells: usize,
    pub zero_run: usize,
    pub p_width: usize,
    pub q_width: usize,
    pub rejected: usize,
    pub splits: usize,
    pub correlation_decisions: usize,
    pub correlation_conflicts: usize,
    pub correlation_gates: usize,
    pub correlation_cells: usize,
}

#[derive(Clone)]
struct Fold {
    p: Vec<Option<char>>,
    q: Vec<Option<char>>,
    p_range: (Vec<char>, Vec<char>),
    q_range: (Vec<char>, Vec<char>),
}

fn bounds(cells: &[Option<char>]) -> (Vec<char>, Vec<char>) {
    (
        trim(cells.iter().map(|c| c.unwrap_or(ZERO)).collect()),
        trim(cells.iter().map(|c| c.unwrap_or(ONE)).collect()),
    )
}

// Interval intersection retains holes imposed by fixed cells. Four tightness
// states per position replace enumeration of the free bit assignments.
fn extrema(cells: &[Option<char>], lo: &[char], hi: &[char]) -> Option<(Vec<char>, Vec<char>)> {
    if mf::cmp(lo, hi) == Ordering::Greater || trim(lo.to_vec()).len() > cells.len() {
        return None;
    }
    let mut feasible = vec![[false; 4]; cells.len() + 1];
    feasible[0] = [true; 4];
    for k in 1..=cells.len() {
        let index = k - 1;
        for state in 0..4 {
            for mark in [ZERO, ONE] {
                if cells[index].is_some_and(|fixed| fixed != mark) {
                    continue;
                }
                let value = u8::from(mark == ONE);
                let lb = bit(lo, index);
                let ub = bit(hi, index);
                if state & 1 != 0 && value < lb || state & 2 != 0 && value > ub {
                    continue;
                }
                let next = usize::from(state & 1 != 0 && value == lb)
                    | (usize::from(state & 2 != 0 && value == ub) << 1);
                feasible[k][state] |= feasible[k - 1][next];
            }
        }
    }
    if !feasible[cells.len()][3] {
        return None;
    }
    let read = |order: [char; 2]| {
        let mut value = vec![ZERO; cells.len()];
        let mut state = 3;
        for index in (0..cells.len()).rev() {
            for mark in order {
                if cells[index].is_some_and(|fixed| fixed != mark) {
                    continue;
                }
                let b = u8::from(mark == ONE);
                let lb = bit(lo, index);
                let ub = bit(hi, index);
                if state & 1 != 0 && b < lb || state & 2 != 0 && b > ub {
                    continue;
                }
                let next = usize::from(state & 1 != 0 && b == lb)
                    | (usize::from(state & 2 != 0 && b == ub) << 1);
                if feasible[index][next] {
                    value[index] = mark;
                    state = next;
                    break;
                }
            }
        }
        trim(value)
    };
    Some((read([ZERO, ONE]), read([ONE, ZERO])))
}

fn constrain(cells: &mut [Option<char>], lo: &[char], hi: &[char]) -> Option<bool> {
    let (min, max) = extrema(cells, lo, hi)?;
    let mut changed = false;
    for index in (0..cells.len()).rev() {
        if bit(&min, index) != bit(&max, index) {
            break;
        }
        let mark = if bit(&min, index) == 1 { ONE } else { ZERO };
        if cells[index].is_none() {
            cells[index] = Some(mark);
            changed = true;
        }
    }
    Some(changed)
}

fn min(a: Vec<char>, b: Vec<char>) -> Vec<char> {
    if mf::cmp(&a, &b) == Ordering::Greater {
        b
    } else {
        a
    }
}
fn max(a: Vec<char>, b: Vec<char>) -> Vec<char> {
    if mf::cmp(&a, &b) == Ordering::Less {
        b
    } else {
        a
    }
}
fn ceiling(n: &[char], divisor: &[char]) -> Vec<char> {
    let (quotient, remainder) = mf::divmod(n, divisor);
    if remainder == vec![ZERO] {
        quotient
    } else {
        mf::add(&quotient, &[ONE])
    }
}
fn cell(cells: &[Option<char>], index: usize) -> Option<char> {
    cells.get(index).copied().unwrap_or(Some(ZERO))
}

// In-place glyph full-adder for a unit contribution. Diagonal counters grow
// through carry and never round-trip through a host-width numerical value.
fn increment(tape: &mut Vec<char>) {
    for mark in tape.iter_mut() {
        if *mark == ZERO {
            *mark = ONE;
            return;
        }
        *mark = ZERO;
    }
    tape.push(ONE);
}

// Full convolution envelope connects the terminal carry to every diagonal,
// including diagonals beyond the first unresolved low corner. All arithmetic
// bounds remain numeral tapes. Relaxing cross-diagonal correlations is safe:
// these intervals may retain extra assignments but cannot remove a solution.
fn carry_envelope(p: &mut [Option<char>], q: &mut [Option<char>], n: &[char]) -> Option<bool> {
    let extent = n.len().max(p.len() + q.len());
    let mut diagonals = Vec::with_capacity(extent);
    let mut carries = vec![(vec![ZERO], vec![ZERO])];
    for k in 0..extent {
        let mut lo = vec![ZERO];
        let mut hi = vec![ZERO];
        for i in k.saturating_sub(q.len().saturating_sub(1))..p.len().min(k + 1) {
            let a = p[i];
            let b = q[k - i];
            if a == Some(ONE) && b == Some(ONE) {
                increment(&mut lo);
            }
            if a != Some(ZERO) && b != Some(ZERO) {
                increment(&mut hi);
            }
        }
        let mut lower = mf::add(&carries[k].0, &lo);
        let mut upper = mf::add(&carries[k].1, &hi);
        if bit(&lower, 0) != bit(n, k) {
            lower = mf::add(&lower, &[ONE]);
        }
        if bit(&upper, 0) != bit(n, k) {
            if upper == vec![ZERO] {
                return None;
            }
            upper = mf::sub(&upper, &[ONE]);
        }
        if mf::cmp(&lower, &upper) == Ordering::Greater {
            return None;
        }
        carries.push((trim(lower[1..].to_vec()), trim(upper[1..].to_vec())));
        diagonals.push((lo, hi));
    }
    if carries[extent].0 != vec![ZERO] {
        return None;
    }
    carries[extent] = (vec![ZERO], vec![ZERO]);
    for k in (0..extent).rev() {
        let double = |value: &[char]| {
            let mut tape = vec![if bit(n, k) == 1 { ONE } else { ZERO }];
            tape.extend_from_slice(value);
            trim(tape)
        };
        let next_lo = double(&carries[k + 1].0);
        let next_hi = double(&carries[k + 1].1);
        if mf::cmp(&next_hi, &diagonals[k].0) == Ordering::Less {
            return None;
        }
        let lo = if mf::cmp(&next_lo, &diagonals[k].1) == Ordering::Greater {
            mf::sub(&next_lo, &diagonals[k].1)
        } else {
            vec![ZERO]
        };
        let hi = mf::sub(&next_hi, &diagonals[k].0);
        carries[k].0 = max(carries[k].0.clone(), lo);
        carries[k].1 = min(carries[k].1.clone(), hi);
        if mf::cmp(&carries[k].0, &carries[k].1) == Ordering::Greater {
            return None;
        }
    }
    let mut changed = false;
    for k in 0..extent {
        let sum = |value: &[char]| {
            let mut tape = vec![if bit(n, k) == 1 { ONE } else { ZERO }];
            tape.extend_from_slice(value);
            trim(tape)
        };
        let next_lo = sum(&carries[k + 1].0);
        let next_hi = sum(&carries[k + 1].1);
        if mf::cmp(&next_hi, &carries[k].0) == Ordering::Less {
            return None;
        }
        let required_lo = if mf::cmp(&next_lo, &carries[k].1) == Ordering::Greater {
            mf::sub(&next_lo, &carries[k].1)
        } else {
            vec![ZERO]
        };
        let required_hi = mf::sub(&next_hi, &carries[k].0);
        if mf::cmp(&required_lo, &diagonals[k].1) == Ordering::Greater
            || mf::cmp(&required_hi, &diagonals[k].0) == Ordering::Less
        {
            return None;
        }
        let force_one = required_lo == diagonals[k].1;
        let force_zero = required_hi == diagonals[k].0;
        if !force_one && !force_zero {
            continue;
        }
        for i in k.saturating_sub(q.len().saturating_sub(1))..p.len().min(k + 1) {
            let j = k - i;
            if p[i] == Some(ZERO) || q[j] == Some(ZERO) || p[i] == Some(ONE) && q[j] == Some(ONE) {
                continue;
            }
            if force_one {
                if p[i].is_none() {
                    p[i] = Some(ONE);
                    changed = true;
                }
                if q[j].is_none() {
                    q[j] = Some(ONE);
                    changed = true;
                }
            } else if p[i] == Some(ONE) && q[j].is_none() {
                q[j] = Some(ZERO);
                changed = true;
            } else if q[j] == Some(ONE) && p[i].is_none() {
                p[i] = Some(ZERO);
                changed = true;
            }
        }
    }
    Some(changed)
}

impl Fold {
    fn cell_count(&self) -> usize {
        self.p.len()
            + self.q.len()
            + self.p_range.0.len()
            + self.p_range.1.len()
            + self.q_range.0.len()
            + self.q_range.1.len()
    }

    fn new(p_width: usize, q_width: usize) -> Self {
        let mut p = vec![None; p_width];
        let mut q = vec![None; q_width];
        p[0] = Some(ONE);
        q[0] = Some(ONE);
        p[p_width - 1] = Some(ONE);
        q[q_width - 1] = Some(ONE);
        let p_range = bounds(&p);
        let q_range = bounds(&q);
        Self {
            p,
            q,
            p_range,
            q_range,
        }
    }

    fn propagate(&mut self, n: &[char]) -> Option<()> {
        loop {
            let mut changed = false;
            let (pmin, pmax) = extrema(&self.p, &self.p_range.0, &self.p_range.1)?;
            let (qmin, qmax) = extrema(&self.q, &self.q_range.0, &self.q_range.1)?;
            if mf::cmp(&mf::mul(&pmin, &qmin), n) == Ordering::Greater
                || mf::cmp(&mf::mul(&pmax, &qmax), n) == Ordering::Less
                || mf::cmp(&pmin, &qmax) == Ordering::Greater
            {
                return None;
            }
            let plo = max(pmin.clone(), ceiling(n, &qmax));
            let phi = min(min(pmax.clone(), mf::divmod(n, &qmin).0), qmax.clone());
            let qlo = max(max(qmin.clone(), ceiling(n, &pmax)), pmin.clone());
            let qhi = min(qmax, mf::divmod(n, &pmin).0);
            changed |= constrain(&mut self.p, &plo, &phi)?;
            changed |= constrain(&mut self.q, &qlo, &qhi)?;
            // Retain numeric intersections, but iterate only when a cell gains
            // information. Chasing endpoint rounding without a bit change can
            // consume the value's magnitude instead of its folded word extent.
            self.p_range = (plo, phi);
            self.q_range = (qlo, qhi);

            // Odd factors have unit cells at position zero. At every resolved
            // diagonal, one free corner is fixed by the source bit and carry.
            let mut carry = vec![ZERO];
            for k in 0..n.len() {
                let mut sum = carry.clone();
                let mut unresolved = Vec::new();
                for i in 0..=k {
                    let a = cell(&self.p, i);
                    let b = cell(&self.q, k - i);
                    match (a, b) {
                        (Some(ZERO), _) | (_, Some(ZERO)) => {}
                        (Some(ONE), Some(ONE)) => sum = mf::add(&sum, &[ONE]),
                        (None, Some(ONE)) => unresolved.push((true, i)),
                        (Some(ONE), None) => unresolved.push((false, k - i)),
                        (None, None) => {
                            unresolved.push((true, i));
                            unresolved.push((false, k - i));
                        }
                        _ => return None,
                    }
                }
                if unresolved.len() > 1 {
                    break;
                }
                if let Some(&(is_p, index)) = unresolved.first() {
                    let mark = if bit(&sum, 0) ^ bit(n, k) == 1 {
                        ONE
                    } else {
                        ZERO
                    };
                    if is_p {
                        self.p[index] = Some(mark);
                    } else {
                        self.q[index] = Some(mark);
                    }
                    if mark == ONE {
                        sum = mf::add(&sum, &[ONE]);
                    }
                    changed = true;
                } else if bit(&sum, 0) != bit(n, k) {
                    return None;
                }
                carry = trim(sum[1..].to_vec());
            }
            // Let the inexpensive resolved corner and interval rails settle
            // before the complete convolution envelope scans their masks.
            if !changed {
                changed = carry_envelope(&mut self.p, &mut self.q, n)?;
            }
            if !changed {
                return Some(());
            }
        }
    }

    fn branch(&self) -> Option<(bool, usize)> {
        // Descend from the low rail; the interval rail simultaneously folds
        // the high cells. A free factor corner lets the opposite rail return.
        for index in 1..self.p.len().max(self.q.len()) {
            if self.p.get(index) == Some(&None) {
                return Some((true, index));
            }
            if self.q.get(index) == Some(&None) {
                return Some((false, index));
            }
        }
        None
    }
}

fn visit(
    lane: &mut Vec<Fold>,
    n: &[char],
    stats: &mut FoldStats,
    pending_count: &mut usize,
    pending_cells: &mut usize,
    observe: &mut impl FnMut(&FoldStats),
) -> Option<(Vec<char>, Vec<char>)> {
    let mut current = lane.pop()?;
    *pending_count -= 1;
    *pending_cells -= current.cell_count();
    stats.decisions += 1;
    stats.p_width = current.p.len();
    stats.q_width = current.q.len();
    stats.frontier = *pending_count + 1;
    stats.cells = *pending_cells + current.cell_count();
    let new_peak = stats.frontier > stats.peak_frontier;
    stats.peak_frontier = stats.peak_frontier.max(stats.frontier);
    stats.peak_cells = stats.peak_cells.max(stats.cells);
    if new_peak || stats.decisions.is_power_of_two() {
        observe(stats);
    }
    if current.propagate(n).is_none() {
        stats.rejected += 1;
        return None;
    }
    if let Some((is_p, index)) = current.branch() {
        stats.splits += 1;
        let mut other = current.clone();
        if is_p {
            current.p[index] = Some(ONE);
            other.p[index] = Some(ZERO);
        } else {
            current.q[index] = Some(ONE);
            other.q[index] = Some(ZERO);
        }
        *pending_count += 2;
        *pending_cells += current.cell_count() + other.cell_count();
        lane.push(other);
        lane.push(current);
        None
    } else {
        let (p, _) = bounds(&current.p);
        let (q, _) = bounds(&current.q);
        if crate::trace_algebra::witness_valid(n, &p, &q) && mf::cmp(&p, &q) != Ordering::Greater {
            observe(stats);
            Some((p, q))
        } else {
            None
        }
    }
}

pub(super) fn solve(
    n: &[char],
    mut observe: impl FnMut(&FoldStats),
) -> (Option<(Vec<char>, Vec<char>)>, FoldStats) {
    let mut stats = FoldStats {
        active: true,
        ..FoldStats::default()
    };
    stats.zero_run = n.iter().position(|&c| c == ONE).unwrap_or(n.len());
    if stats.zero_run == n.len() {
        observe(&stats);
        return (None, stats);
    }
    if stats.zero_run > 0 {
        // v(p*q)=v(p)+v(q): fold the entire leading-zero relation before
        // materializing any of its unconstrained prefixes. Unit odd residues
        // close directly within a nontrivial valuation sector.
        let odd = trim(n[stats.zero_run..].to_vec());
        let left_shift = if odd == vec![ONE] {
            stats.zero_run / 2
        } else {
            stats.zero_run
        };
        let right_shift = stats.zero_run - left_shift;
        let mut p = vec![ZERO; left_shift];
        p.push(ONE);
        let mut q = vec![ZERO; right_shift];
        q.extend(odd);
        if mf::cmp(&p, &q) == Ordering::Greater {
            core::mem::swap(&mut p, &mut q);
        }
        stats.cells = p.len() + q.len();
        stats.peak_cells = stats.cells;
        stats.frontier = 1;
        stats.peak_frontier = 1;
        observe(&stats);
        let proper = crate::trace_algebra::witness_valid(n, &p, &q);
        return (proper.then_some((p, q)), stats);
    }
    let factor_bound = mf::isqrt(n);
    let cofactor_bound = ceiling(n, &factor_bound);
    let widest = n.len().div_ceil(2);
    let mut widths = Vec::new();
    let mut low = 2;
    let mut high = widest;
    while low <= high {
        widths.push(low);
        if low != high {
            widths.push(high);
        }
        low += 1;
        high -= 1;
    }
    // Each geometry advances one decision at a time. A wide lane therefore
    // cannot strand a closing narrow lane behind its entire search tree.
    let mut lanes = Vec::new();
    let mut pending_count = 0;
    let mut pending_cells = 0;
    for p_width in widths {
        for sum in [n.len(), n.len() + 1] {
            let q_width = sum - p_width;
            if q_width < p_width {
                continue;
            }
            let mut root = Fold::new(p_width, q_width);
            root.p_range.1 = min(root.p_range.1, factor_bound.clone());
            root.q_range.0 = max(root.q_range.0, cofactor_bound.clone());
            pending_count += 1;
            pending_cells += root.cell_count();
            lanes.push(vec![root]);
            if let Some(pair) = visit(
                lanes.last_mut().unwrap(),
                n,
                &mut stats,
                &mut pending_count,
                &mut pending_cells,
                &mut observe,
            ) {
                return (Some(pair), stats);
            }
        }
    }
    // A shared multiplier keeps the balanced geometry's internal product and
    // carry implications across decisions. Other geometries retain fair turns.
    let mut shared = correlation::Correlation::new(n, widest, &factor_bound, &cofactor_bound);
    stats.correlation_gates = shared.gate_count();
    stats.correlation_cells = shared.cell_count();
    let mut shared_live = true;
    while pending_count != 0 || shared_live {
        for index in 0..lanes.len().max(1) {
            if shared_live {
                let step = shared.advance();
                stats.correlation_decisions = shared.decisions;
                stats.correlation_conflicts = shared.conflicts;
                if stats.correlation_decisions.is_power_of_two()
                    || !matches!(step, correlation::Step::Running)
                {
                    observe(&stats);
                }
                match step {
                    correlation::Step::Running => {}
                    correlation::Step::Empty => shared_live = false,
                    correlation::Step::Closed(p, q) => {
                        assert!(crate::trace_algebra::witness_valid(n, &p, &q));
                        return (Some((p, q)), stats);
                    }
                }
            }
            if index >= lanes.len() {
                continue;
            }
            if let Some(pair) = visit(
                &mut lanes[index],
                n,
                &mut stats,
                &mut pending_count,
                &mut pending_cells,
                &mut observe,
            ) {
                return (Some(pair), stats);
            }
        }
    }
    stats.frontier = 0;
    stats.cells = 0;
    observe(&stats);
    (None, stats)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn terminal_carry_envelope_preserves_every_small_product_assignment() {
        for pattern in 0..729 {
            let mut digits = pattern;
            let masks: Vec<_> = (0..6)
                .map(|_| {
                    let mark = match digits % 3 {
                        0 => None,
                        1 => Some(ZERO),
                        _ => Some(ONE),
                    };
                    digits /= 3;
                    mark
                })
                .collect();
            let compatible = |value: u64, mask: &[Option<char>]| {
                mask.iter()
                    .enumerate()
                    .all(|(i, c)| c.is_none_or(|m| ((value >> i) & 1 == 1) == (m == ONE)))
            };
            for source in 0..=49u64 {
                let valid: Vec<_> = (0..8)
                    .flat_map(|p| (0..8).map(move |q| (p, q)))
                    .filter(|&(p, q)| {
                        p * q == source && compatible(p, &masks[..3]) && compatible(q, &masks[3..])
                    })
                    .collect();
                let mut p = masks[..3].to_vec();
                let mut q = masks[3..].to_vec();
                let result = carry_envelope(&mut p, &mut q, &mf::tape_u64(source));
                assert!(
                    valid.is_empty() || result.is_some(),
                    "pattern={pattern} source={source}"
                );
                assert!(
                    valid
                        .iter()
                        .all(|&(pv, qv)| compatible(pv, &p) && compatible(qv, &q)),
                    "pattern={pattern} source={source}"
                );
            }
        }
    }

    #[test]
    fn terminal_carry_forces_high_cells_across_unresolved_low_diagonals() {
        let mut p = vec![Some(ONE), None, Some(ONE)];
        let mut q = p.clone();
        assert_eq!(
            carry_envelope(&mut p, &mut q, &mf::tape_u64(49)),
            Some(true)
        );
        assert_eq!(p, vec![Some(ONE); 3]);
        assert_eq!(q, p);
    }

    #[test]
    fn interval_fold_keeps_every_assignment_across_fixed_cell_holes() {
        for pattern in 0..81 {
            let mut digits = pattern;
            let cells: Vec<_> = (0..4)
                .map(|_| {
                    let cell = match digits % 3 {
                        0 => None,
                        1 => Some(ZERO),
                        _ => Some(ONE),
                    };
                    digits /= 3;
                    cell
                })
                .collect();
            for lo in 0..16 {
                for hi in lo..16 {
                    let values: Vec<_> = (lo..=hi)
                        .filter(|value| {
                            cells.iter().enumerate().all(|(i, c)| {
                                c.is_none_or(|m| (value >> i) & 1 == u64::from(m == ONE))
                            })
                        })
                        .collect();
                    let actual = extrema(&cells, &mf::tape_u64(lo), &mf::tape_u64(hi));
                    let expected = values
                        .first()
                        .zip(values.last())
                        .map(|(&a, &b)| (mf::tape_u64(a), mf::tape_u64(b)));
                    assert_eq!(actual, expected, "pattern={pattern}, interval={lo}..{hi}");
                    let mut folded = cells.clone();
                    let changed = constrain(&mut folded, &mf::tape_u64(lo), &mf::tape_u64(hi));
                    assert_eq!(changed.is_some(), !values.is_empty());
                    assert!(values
                        .iter()
                        .all(|value| folded.iter().enumerate().all(
                            |(i, c)| c.is_none_or(|m| (value >> i) & 1 == u64::from(m == ONE))
                        )));
                }
            }
        }
    }
    #[test]
    fn folded_relation_matches_small_exact_product_census() {
        for value in 0..=1024 {
            let n = mf::tape_u64(value);
            let (pair, _) = solve(&n, |_| {});
            let expected = (2..=value).any(|p| p * p <= value && value % p == 0);
            assert_eq!(pair.is_some(), expected, "N={value}");
            if let Some((p, q)) = pair {
                assert!(crate::trace_algebra::witness_valid(&n, &p, &q));
                assert!(mf::cmp(&p, &q) != Ordering::Greater);
            }
        }
    }
}
