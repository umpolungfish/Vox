//! Joint radix frames of the same factor supports, with shared carry boundaries.
use super::{bounds, mf, trim, ONE, ZERO};
use alloc::{vec, vec::Vec};
use core::cmp::Ordering;

fn admits(n: &[char], p: &[Option<char>], q: &[Option<char>], width: usize) -> bool {
    let mut radix = vec![ZERO; width]; radix.push(ONE);
    let left: Vec<_> = p.chunks(width).map(bounds).collect();
    let right: Vec<_> = q.chunks(width).map(bounds).collect();
    let mut carry = (vec![ZERO], vec![ZERO]);
    let extent = n.len().div_ceil(width).max(left.len()+right.len());
    for column in 0..extent {
        let mut lo = carry.0;
        let mut hi = carry.1;
        for (i, a) in left.iter().enumerate() {
            if let Some(j) = column.checked_sub(i) {
                if let Some(b) = right.get(j) {
                    lo = mf::add(&lo, &mf::mul(&a.0, &b.0));
                    hi = mf::add(&hi, &mf::mul(&a.1, &b.1));
                }
            }
        }
        let start = column*width;
        let target = if start<n.len() {trim(n[start..n.len().min(start+width)].to_vec())}
            else {vec![ZERO]};
        let lower_residue = mf::modulo(&lo, &radix);
        let increment = if mf::cmp(&target, &lower_residue)==Ordering::Less {
            mf::sub(&radix, &mf::sub(&lower_residue, &target))
        } else {mf::sub(&target, &lower_residue)};
        lo = mf::add(&lo, &increment);
        let upper_residue = mf::modulo(&hi, &radix);
        let decrement = if mf::cmp(&upper_residue, &target)==Ordering::Less {
            mf::sub(&radix, &mf::sub(&target, &upper_residue))
        } else {mf::sub(&upper_residue, &target)};
        if mf::cmp(&decrement, &hi)==Ordering::Greater {return false;}
        hi = mf::sub(&hi, &decrement);
        if mf::cmp(&lo, &hi)==Ordering::Greater {return false;}
        carry = (mf::divmod(&lo, &radix).0, mf::divmod(&hi, &radix).0);
    }
    mf::zero(&carry.0)
}

/// One coordinate is eliminated against a joint product/carry frame. Window
/// and coordinate advance with the resident extent; all other bits stay free.
pub(super) fn eliminate(
    n: &[char], p: &mut [Option<char>], q: &mut [Option<char>], turn: usize,
) -> Option<bool> {
    let width = 1 + turn % n.len();
    if !admits(n, p, q, width) {return None;}
    let extent = p.len()+q.len();
    let selected = (0..extent).map(|step| (turn+step)%extent).find(|&index| {
        if index<p.len() {p[index].is_none()} else {q[index-p.len()].is_none()}
    });
    let Some(index) = selected else {return Some(false);};
    let set = |p: &mut [Option<char>], q: &mut [Option<char>], value| {
        if index<p.len() {p[index]=value;} else {q[index-p.len()]=value;}
    };
    set(p, q, Some(ZERO)); let zero = admits(n, p, q, width);
    set(p, q, Some(ONE)); let one = admits(n, p, q, width);
    let value = match (zero, one) {
        (false, false) => {set(p, q, None); return None;}
        (true, false) => Some(ZERO),
        (false, true) => Some(ONE),
        (true, true) => None,
    };
    set(p, q, value);
    Some(value.is_some())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn every_joint_frame_retains_the_random_2048_pair_and_rejects_a_wrong_bit() {
        let n=mf::parse_numeral(include_str!("../tests/fixtures/random_rsa_2048.imasm").trim()).unwrap();
        let p=mf::parse_numeral(include_str!("../tests/fixtures/random_rsa_2048.p.imasm").trim()).unwrap();
        let q=mf::parse_numeral(include_str!("../tests/fixtures/random_rsa_2048.q.imasm").trim()).unwrap();
        let left: Vec<_>=p.iter().copied().map(Some).collect();
        let right: Vec<_>=q.iter().copied().map(Some).collect();
        let mut wrong=left.clone();
        wrong[p.len()/2]=Some(if p[p.len()/2]==ONE {ZERO} else {ONE});
        for width in 1..=n.len() {
            assert!(admits(&n,&left,&right,width), "width {width}");
            assert!(!admits(&n,&wrong,&right,width), "width {width}");
        }
        let mut partial=left.clone();
        partial[p.len()/2]=None;
        assert_eq!(eliminate(&n,&mut partial,&mut right.clone(),n.len()/2),Some(true));
        assert_eq!(partial,left);
    }
}
