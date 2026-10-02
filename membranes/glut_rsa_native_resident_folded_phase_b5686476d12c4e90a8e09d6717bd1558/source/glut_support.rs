//! Sparse address supports folded into dynamically retained parity words.
//! Empty blocks dissolve during cancellation; surviving addresses keep their
//! complete coordinate and no source-width extent is allocated in advance.
use alloc::vec::Vec;

pub(super) struct Support {
    words: Vec<(usize, u64)>,
}
impl Support {
    pub(super) fn from_sorted(addresses: impl IntoIterator<Item = usize>) -> Self {
        let mut words: Vec<(usize, u64)> = Vec::new();
        for address in addresses {
            let block = address / u64::BITS as usize;
            let bit = 1u64 << (address % u64::BITS as usize);
            if let Some((previous, mask)) = words.last_mut() {
                assert!(
                    *previous <= block,
                    "support addresses retain their ordered coordinates"
                );
                if *previous == block {
                    *mask ^= bit;
                    if *mask == 0 {
                        words.pop();
                    }
                    continue;
                }
            }
            words.push((block, bit));
        }
        Self { words }
    }
    pub(super) fn extent(&self) -> usize {
        2 * self.words.len()
    }
    pub(super) fn pivot(&self) -> Option<usize> {
        self.words.last().map(|&(block, mask)| {
            block * u64::BITS as usize + (u64::BITS - 1 - mask.leading_zeros()) as usize
        })
    }
    pub(super) fn xor(&self, right: &Self) -> Self {
        let mut words = Vec::new();
        let (mut a, mut b) = (0, 0);
        while a < self.words.len() || b < right.words.len() {
            match (self.words.get(a), right.words.get(b)) {
                (Some(&(i, left)), Some(&(j, right))) if i == j => {
                    let mask = left ^ right;
                    if mask != 0 {
                        words.push((i, mask));
                    }
                    a += 1;
                    b += 1;
                }
                (Some(&left), Some(&(j, _))) if left.0 < j => {
                    words.push(left);
                    a += 1;
                }
                (Some(_), Some(&right)) => {
                    words.push(right);
                    b += 1;
                }
                (Some(&left), None) => {
                    words.push(left);
                    a += 1;
                }
                (None, Some(&right)) => {
                    words.push(right);
                    b += 1;
                }
                (None, None) => break,
            }
        }
        Self { words }
    }
    pub(super) fn addresses(&self) -> Addresses<'_> {
        Addresses {
            words: self.words.iter(),
            pending: None,
        }
    }
}
pub(super) struct Addresses<'a> {
    words: core::slice::Iter<'a, (usize, u64)>,
    pending: Option<(usize, u64)>,
}
impl Iterator for Addresses<'_> {
    type Item = usize;
    fn next(&mut self) -> Option<usize> {
        loop {
            if let Some((block, mask)) = &mut self.pending {
                if *mask != 0 {
                    let address = *block * u64::BITS as usize + mask.trailing_zeros() as usize;
                    *mask &= *mask - 1;
                    return Some(address);
                }
            }
            self.pending = Some(*self.words.next()?);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::morphism_factor as mf;
    use alloc::collections::BTreeSet;
    #[test]
    fn folded_parity_retains_the_random_2048_source_coordinates() {
        let source =
            mf::parse_numeral(include_str!("../tests/fixtures/random_rsa_2048.imasm").trim())
                .unwrap();
        let left: Vec<_> = source
            .iter()
            .enumerate()
            .filter_map(|(i, &mark)| (mark == '⊥').then_some(i))
            .collect();
        let right: Vec<_> = source
            .iter()
            .enumerate()
            .filter_map(|(i, &mark)| (mark == '⊤').then_some(i + source.len() / 2))
            .collect();
        let expected: BTreeSet<_> =
            left.iter()
                .chain(&right)
                .fold(BTreeSet::new(), |mut set, &i| {
                    if !set.insert(i) {
                        set.remove(&i);
                    }
                    set
                });
        let left = Support::from_sorted(left);
        let right = Support::from_sorted(right);
        let joined = left.xor(&right);
        assert_eq!(joined.addresses().collect::<BTreeSet<_>>(), expected);
        assert_eq!(joined.pivot(), expected.last().copied());
        assert_eq!(
            joined.xor(&right).addresses().collect::<Vec<_>>(),
            left.addresses().collect::<Vec<_>>()
        );
        assert!(joined.xor(&joined).pivot().is_none());
        assert!(joined.extent() < expected.len());
    }
}
