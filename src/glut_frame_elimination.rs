//! Joint radix frames of the same factor supports, with shared carry boundaries.
use super::{bounds, mf, trim, ONE, ZERO};
use alloc::{collections::BTreeMap, vec, vec::Vec};
use core::cmp::Ordering;

type Interval = (Vec<char>, Vec<char>);

#[derive(Clone, Default)]
pub(super) struct FrameBank {
    p: Vec<Option<char>>,
    q: Vec<Option<char>>,
    carries: BTreeMap<usize, Vec<Interval>>,
}
impl FrameBank {
    fn synchronize(&mut self, p: &[Option<char>], q: &[Option<char>]) {
        let extends = |old: &[Option<char>], new: &[Option<char>]| {
            old.len() == new.len() && old.iter().zip(new).all(|(a, b)| a.is_none() || a == b)
        };
        if !extends(&self.p, p) || !extends(&self.q, q) {
            self.carries.clear();
        }
        self.p = p.to_vec();
        self.q = q.to_vec();
    }
}

#[cfg(test)]
fn admits(n: &[char], p: &[Option<char>], q: &[Option<char>], width: usize) -> bool {
    admits_banked(n, p, q, width, &mut FrameBank::default())
}

fn admits_banked(
    n: &[char],
    p: &[Option<char>],
    q: &[Option<char>],
    width: usize,
    bank: &mut FrameBank,
) -> bool {
    let mut radix = vec![ZERO; width];
    radix.push(ONE);
    let left: Vec<_> = p.chunks(width).map(bounds).collect();
    let right: Vec<_> = q.chunks(width).map(bounds).collect();
    let extent = n.len().div_ceil(width).max(left.len() + right.len());
    let mut sums = Vec::with_capacity(extent);
    let mut digits = Vec::with_capacity(extent);
    for column in 0..extent {
        let mut sum = (vec![ZERO], vec![ZERO]);
        for (i, a) in left.iter().enumerate() {
            if let Some(b) = column.checked_sub(i).and_then(|j| right.get(j)) {
                sum.0 = mf::add(&sum.0, &mf::mul(&a.0, &b.0));
                sum.1 = mf::add(&sum.1, &mf::mul(&a.1, &b.1));
            }
        }
        sums.push(sum);
        let start = column * width;
        digits.push(if start < n.len() {
            trim(n[start..n.len().min(start + width)].to_vec())
        } else {
            vec![ZERO]
        });
    }
    let quotient = |value: &[char]| {
        if value.len() > width {
            trim(value[width..].to_vec())
        } else {
            vec![ZERO]
        }
    };
    let shifted = |value: &[char]| {
        if mf::zero(value) {
            vec![ZERO]
        } else {
            let mut result = vec![ZERO; width];
            result.extend_from_slice(value);
            result
        }
    };
    let intersect = |current: &mut Interval, candidate: Interval| {
        if mf::cmp(&current.0, &candidate.0) == Ordering::Less {
            current.0 = candidate.0;
        }
        if mf::cmp(&current.1, &candidate.1) == Ordering::Greater {
            current.1 = candidate.1;
        }
        mf::cmp(&current.0, &current.1) != Ordering::Greater
    };
    let retained = bank.carries.get(&width);
    let mut carries = vec![(vec![ZERO], vec![ZERO])];
    for column in 0..extent {
        let hi = mf::add(&carries[column].1, &sums[column].1);
        let mut next = (vec![ZERO], quotient(&hi));
        if let Some(previous) = retained.and_then(|rows| rows.get(column)) {
            if !intersect(&mut next, previous.clone()) {
                return false;
            }
        }
        carries.push(next);
    }
    if !intersect(&mut carries[extent], (vec![ZERO], vec![ZERO])) {
        return false;
    }
    loop {
        let mut changed = false;
        for column in 0..extent {
            let target = &digits[column];
            let mut lo = mf::add(&carries[column].0, &sums[column].0);
            let mut hi = mf::add(&carries[column].1, &sums[column].1);
            let residue = trim(lo[..lo.len().min(width)].to_vec());
            let increment = if mf::cmp(target, &residue) == Ordering::Less {
                mf::sub(&radix, &mf::sub(&residue, target))
            } else {
                mf::sub(target, &residue)
            };
            lo = mf::add(&lo, &increment);
            let residue = trim(hi[..hi.len().min(width)].to_vec());
            let decrement = if mf::cmp(&residue, target) == Ordering::Less {
                mf::sub(&radix, &mf::sub(target, &residue))
            } else {
                mf::sub(&residue, target)
            };
            if mf::cmp(&decrement, &hi) == Ordering::Greater {
                return false;
            }
            hi = mf::sub(&hi, &decrement);
            if mf::cmp(&lo, &hi) == Ordering::Greater {
                return false;
            }
            let old = carries[column + 1].clone();
            if !intersect(&mut carries[column + 1], (quotient(&lo), quotient(&hi))) {
                return false;
            }
            changed |= old != carries[column + 1];
        }
        for column in (0..extent).rev() {
            let lo = mf::add(&shifted(&carries[column + 1].0), &digits[column]);
            let hi = mf::add(&shifted(&carries[column + 1].1), &digits[column]);
            if mf::cmp(&hi, &sums[column].0) == Ordering::Less {
                return false;
            }
            let incoming_lo = if mf::cmp(&lo, &sums[column].1) == Ordering::Greater {
                mf::sub(&lo, &sums[column].1)
            } else {
                vec![ZERO]
            };
            let incoming_hi = mf::sub(&hi, &sums[column].0);
            let old = carries[column].clone();
            if !intersect(&mut carries[column], (incoming_lo, incoming_hi)) {
                return false;
            }
            changed |= old != carries[column];
        }
        if !changed {
            break;
        }
    }
    bank.carries
        .insert(width, carries.into_iter().skip(1).collect());
    true
}

/// The first carry bounds the product of both low-bit prefixes. Return that
/// interval to the same factor masks rather than leaving it inside the view.
fn return_prefix(
    n: &[char],
    p: &mut [Option<char>],
    q: &mut [Option<char>],
    width: usize,
    bank: &FrameBank,
) -> Option<bool> {
    let carry = bank.carries.get(&width)?.first()?;
    let residue = trim(n[..n.len().min(width)].to_vec());
    let lift = |value: &[char]| {
        let mut result = vec![ZERO; width];
        result.extend_from_slice(value);
        mf::add(&trim(result), &residue)
    };
    let product = (lift(&carry.0), lift(&carry.1));
    let mut changed = false;
    for side in 0..2 {
        let (target, other) = if side == 0 {
            (&mut *p, &*q)
        } else {
            (&mut *q, &*p)
        };
        let other = bounds(&other[..other.len().min(width)]);
        if mf::zero(&other.1) {
            if !mf::zero(&product.0) {
                return None;
            }
            continue;
        }
        let (mut lo, remainder) = mf::divmod(&product.0, &other.1);
        if !mf::zero(&remainder) {
            lo = mf::add(&lo, &[ONE]);
        }
        let length = target.len().min(width);
        let mut hi = if mf::zero(&other.0) {
            bounds(&target[..length]).1
        } else {
            mf::divmod(&product.1, &other.0).0
        };
        let domain = bounds(&target[..length]);
        if mf::cmp(&hi, &domain.1) == Ordering::Greater {
            hi = domain.1;
        }
        if mf::cmp(&lo, &domain.0) == Ordering::Less {
            lo = domain.0;
        }
        if mf::cmp(&lo, &hi) == Ordering::Greater {
            return None;
        }
        changed |= super::constrain(&mut target[..length], &lo, &hi)?;
    }
    Some(changed)
}

/// One coordinate is eliminated against a joint product/carry frame. Window
/// and coordinate advance with the resident extent; all other bits stay free.
pub(super) fn eliminate(
    n: &[char],
    p: &mut [Option<char>],
    q: &mut [Option<char>],
    turn: usize,
    bank: &mut FrameBank,
) -> Option<bool> {
    let width = 1 + turn % n.len();
    bank.synchronize(p, q);
    if !admits_banked(n, p, q, width, bank) {
        return None;
    }
    let returned = return_prefix(n, p, q, width, bank)?;
    let extent = p.len() + q.len();
    let selected = (0..extent)
        .map(|step| (turn + step) % extent)
        .find(|&index| {
            if index < p.len() {
                p[index].is_none()
            } else {
                q[index - p.len()].is_none()
            }
        });
    let Some(index) = selected else {
        return Some(returned);
    };
    let set = |p: &mut [Option<char>], q: &mut [Option<char>], value| {
        if index < p.len() {
            p[index] = value;
        } else {
            q[index - p.len()] = value;
        }
    };
    // A probe writes only its own view. Other retained views remain resident
    // instead of being copied for both candidate bits.
    let fork = || {
        let mut probe = FrameBank::default();
        if let Some(rows) = bank.carries.get(&width) {
            probe.carries.insert(width, rows.clone());
        }
        probe
    };
    let mut zero_bank = fork();
    let mut one_bank = fork();
    set(p, q, Some(ZERO));
    let zero = admits_banked(n, p, q, width, &mut zero_bank);
    set(p, q, Some(ONE));
    let one = admits_banked(n, p, q, width, &mut one_bank);
    let value = match (zero, one) {
        (false, false) => {
            set(p, q, None);
            return None;
        }
        (true, false) => Some(ZERO),
        (false, true) => Some(ONE),
        (true, true) => None,
    };
    set(p, q, value);
    let selected_bank = match value {
        Some(ZERO) => Some(zero_bank),
        Some(ONE) => Some(one_bank),
        _ => None,
    };
    if let Some(mut selected_bank) = selected_bank {
        bank.carries.insert(
            width,
            selected_bank
                .carries
                .remove(&width)
                .expect("an admitted probe retains its carry view"),
        );
    }
    bank.p = p.to_vec();
    bank.q = q.to_vec();
    Some(returned || value.is_some())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn every_joint_frame_retains_the_random_2048_pair_and_rejects_a_wrong_bit() {
        let n = mf::parse_numeral(include_str!("../tests/fixtures/random_rsa_2048.imasm").trim())
            .unwrap();
        let p = mf::parse_numeral(include_str!("../tests/fixtures/random_rsa_2048.p.imasm").trim())
            .unwrap();
        let q = mf::parse_numeral(include_str!("../tests/fixtures/random_rsa_2048.q.imasm").trim())
            .unwrap();
        let left: Vec<_> = p.iter().copied().map(Some).collect();
        let right: Vec<_> = q.iter().copied().map(Some).collect();
        let mut wrong = left.clone();
        wrong[p.len() / 2] = Some(if p[p.len() / 2] == ONE { ZERO } else { ONE });
        let masked_left: Vec<_> = left
            .iter()
            .enumerate()
            .map(|(i, &value)| if n[i] == ONE { value } else { None })
            .collect();
        let masked_right: Vec<_> = right
            .iter()
            .enumerate()
            .map(|(i, &value)| if n[i] == ZERO { value } else { None })
            .collect();
        for width in 1..=n.len() {
            assert!(admits(&n, &left, &right, width), "width {width}");
            assert!(
                admits(&n, &masked_left, &masked_right, width),
                "masked width {width}"
            );
            assert!(!admits(&n, &wrong, &right, width), "width {width}");
        }
        let mut bank = FrameBank::default();
        bank.synchronize(&masked_left, &masked_right);
        for width in 1..=n.len() {
            assert!(admits_banked(
                &n,
                &masked_left,
                &masked_right,
                width,
                &mut bank
            ));
        }
        bank.synchronize(&left, &right);
        assert_eq!(bank.carries.len(), n.len());
        for width in 1..=n.len() {
            assert!(
                admits_banked(&n, &left, &right, width, &mut bank),
                "retained width {width}"
            );
            let mut a = masked_left.clone();
            let mut b = masked_right.clone();
            assert!(
                return_prefix(&n, &mut a, &mut b, width, &bank).is_some(),
                "prefix width {width}"
            );
            assert!(a.iter().zip(&left).all(|(a, b)| a.is_none() || a == b));
            assert!(b.iter().zip(&right).all(|(a, b)| a.is_none() || a == b));
        }
        bank.synchronize(&masked_left, &masked_right);
        assert!(
            bank.carries.is_empty(),
            "relaxed supports must discard branch constraints"
        );
        assert!(admits_banked(&n, &masked_left, &masked_right, 1, &mut bank));
        let mut partial = left.clone();
        partial[p.len() / 2] = None;
        assert_eq!(
            eliminate(
                &n,
                &mut partial,
                &mut right.clone(),
                n.len() / 2,
                &mut FrameBank::default()
            ),
            Some(true)
        );
        assert_eq!(partial, left);
    }
}
