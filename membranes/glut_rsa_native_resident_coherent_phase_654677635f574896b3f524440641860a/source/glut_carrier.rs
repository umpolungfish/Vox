//! Carrier assembled from bound GLUT component ports over one resident register.
use super::{
    fold::{Port, Resident},
    GlutExecution,
};
use alloc::{rc::Rc, string::String, vec::Vec};
use core::cell::{Cell, RefCell};

pub struct PreparedGlutMembrane {
    nested: Vec<char>,
    collapsed: Vec<char>,
    body: Vec<(char, Option<Port>)>,
    resident: Rc<RefCell<Resident>>,
}
impl PreparedGlutMembrane {
    pub fn prepare(n: &[char]) -> Result<Self, String> {
        // Skin -> GLUT -> component protocols. Consume each component's source
        // and terminal ports; retain its work and dyad inside the GLUT frame.
        let mut nested: Vec<char> = "⊢∈∈".chars().collect();
        let mut body = Vec::new();
        for port in Port::ORDER {
            let word: Vec<char> = port.word().chars().collect();
            if word.first() != Some(&'⊢') || word.last() != Some(&'⊣') {
                return Err("GLUT component lacks source and terminal ports".into());
            }
            let mut bound = false;
            for &mark in &word[1..word.len() - 1] {
                nested.push(mark);
                if matches!(mark, '∈' | '∋') {
                    continue;
                }
                if mark == '⊡' {
                    return Err("GLUT component fixes before its enclosing fuse".into());
                }
                let binding = if !bound && mark == port.trigger() {
                    bound = true;
                    Some(port)
                } else {
                    None
                };
                body.push((mark, binding));
            }
            if !bound {
                return Err("GLUT component has no bound working port".into());
            }
        }
        nested.extend("∋∋⊡⊣".chars());
        let (_, open, overfused) = crate::vox::pairing(&nested);
        if !open.is_empty() || !overfused.is_empty() {
            return Err("assembled GLUT protocol has unmatched boundaries".into());
        }
        let collapsed = crate::fixed_point_hypernest::collapse_to_nested(&nested);
        let mut expected: Vec<char> = "⊢∈".chars().collect();
        expected.extend(body.iter().map(|&(mark, _)| mark));
        expected.extend("∋⊡⊣".chars());
        if collapsed != expected {
            return Err("GLUT dissolution changed connected component work".into());
        }
        Ok(Self {
            nested,
            collapsed,
            body,
            resident: Rc::new(RefCell::new(Resident::new(n, |_| {}))),
        })
    }
    pub fn nested_word(&self) -> &[char] {
        &self.nested
    }
    pub fn membrane_word(&self) -> &[char] {
        &self.collapsed
    }
    pub fn execute(self) -> Result<Option<GlutExecution>, String> {
        let binding = Rc::new(Cell::new(None));
        let address = Rc::new(Cell::new(None));
        let walk = CarrierWalk {
            body: self.body,
            position: 0,
            prefix: 0,
            tail: None,
            binding: binding.clone(),
            address: address.clone(),
            resident: self.resident.clone(),
        };
        let returned = crate::phase_word::execute_addressed_iter_with(
            walk,
            &self.resident,
            |_, resident| {
                if let Some(port) = binding.get() {
                    resident.borrow_mut().activate(port, |_| {});
                }
                Ok(())
            },
            |lane| {
                address
                    .get()
                    .map(|slot| slot * 4 + lane.trailing_zeros() as usize)
            },
        )?;
        if returned.exposed != 0 || returned.surviving.is_empty() {
            return Err("GLUT return lost its enclosing bank".into());
        }
        if returned
            .surviving
            .iter()
            .any(|d| !Rc::ptr_eq(&self.resident, &d.observation))
        {
            return Err("GLUT return changed its resident register".into());
        }
        let mut resident = self.resident.borrow_mut();
        if !resident.finished() {
            return Err("GLUT fused before its verification return".into());
        }
        if !resident.quantum_phase_closed() {
            return Err("GLUT fused before its quantum factor-phase return".into());
        }
        Ok(resident.take_execution())
    }
}

/// The return edge repeats component work within the same open skin. Register
/// state survives each pass. Only the verified return releases fuse/fix/readout.
struct CarrierWalk {
    body: Vec<(char, Option<Port>)>,
    position: usize,
    prefix: usize,
    tail: Option<usize>,
    binding: Rc<Cell<Option<Port>>>,
    address: Rc<Cell<Option<usize>>>,
    resident: Rc<RefCell<Resident>>,
}
impl Iterator for CarrierWalk {
    type Item = char;
    fn next(&mut self) -> Option<char> {
        self.binding.set(None);
        self.address.set(None);
        if self.prefix < 2 {
            let mark = ['⊢', '∈'][self.prefix];
            self.prefix += 1;
            return Some(mark);
        }
        if self.position == self.body.len() && self.tail.is_none() {
            if self.resident.borrow().finished() {
                self.tail = Some(0);
            } else {
                self.position = 0;
            }
        }
        if let Some(index) = self.tail {
            self.tail = Some(index + 1);
            return ['∋', '⊡', '⊣'].get(index).copied();
        }
        self.address.set(Some(self.position));
        let (mark, port) = self.body[self.position];
        self.position += 1;
        self.binding.set(port);
        Some(mark)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn random_2048_source_is_resident_in_the_component_derived_carrier() {
        let n = crate::morphism_factor::parse_numeral(
            include_str!("../tests/fixtures/random_rsa_2048.imasm").trim(),
        )
        .unwrap();
        let prepared = PreparedGlutMembrane::prepare(&n).unwrap();
        let slots: Vec<_> = prepared.body.iter().filter_map(|&(_, p)| p).collect();
        assert_eq!(slots, Port::ORDER);
        assert_eq!(
            prepared
                .membrane_word()
                .iter()
                .filter(|&&m| m == '∈')
                .count(),
            1
        );
        assert_eq!(
            prepared
                .membrane_word()
                .iter()
                .filter(|&&m| m == '∋')
                .count(),
            1
        );
        assert_eq!(
            prepared
                .membrane_word()
                .iter()
                .filter(|&&m| m == '⊡')
                .count(),
            1
        );
        assert!(prepared.nested_word().len() > prepared.membrane_word().len());
        assert_eq!(prepared.resident.borrow().source(), n);
        let id = prepared.resident.clone();
        let binding = Rc::new(Cell::new(None));
        let mut walk = CarrierWalk {
            body: prepared.body,
            position: 0,
            prefix: 0,
            tail: None,
            binding: binding.clone(),
            address: Rc::new(Cell::new(None)),
            resident: prepared.resident,
        };
        let mut engaged = Vec::new();
        for _ in 0..2 {
            while let Some(mark) = walk.next() {
                assert!(
                    mark != '⊡' && mark != '∋',
                    "unfinished return must keep the skin open"
                );
                if let Some(port) = binding.get() {
                    engaged.push(port);
                }
                if walk.position == walk.body.len() {
                    break;
                }
            }
        }
        assert_eq!(engaged, [Port::ORDER, Port::ORDER].concat());
        assert!(Rc::ptr_eq(&id, &walk.resident));
        assert!(!walk.resident.borrow().finished());
    }
}
