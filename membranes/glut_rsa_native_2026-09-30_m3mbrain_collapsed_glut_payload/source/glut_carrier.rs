//! GLUT resident payload inside a nested carrier, collapsed before execution.
use super::{GlutExecution, GlutSieve};
use alloc::{rc::Rc, string::String, vec::Vec};
use core::cell::RefCell;

pub struct PreparedGlutMembrane {
    nested: Vec<char>,
    collapsed: Vec<char>,
    resident: Rc<RefCell<GlutSieve>>,
}

impl PreparedGlutMembrane {
    pub fn prepare(n: &[char], nested_word: &str) -> Result<Self, String> {
        let nested: Vec<char> = nested_word.chars().collect();
        if nested.first() != Some(&'⊢') || !nested.ends_with(&['⊡', '⊣']) {
            return Err("GLUT payload requires an enclosing fixed carrier".into());
        }
        let (_, open, overfused) = crate::vox::pairing(&nested);
        if !open.is_empty() || !overfused.is_empty() {
            return Err("GLUT payload carrier has unmatched boundaries".into());
        }
        let collapsed = crate::fixed_point_hypernest::collapse_to_nested(&nested);
        if collapsed.iter().filter(|&&m| m == '∈').count() != 1
            || collapsed.iter().filter(|&&m| m == '∋').count() != 1 {
            return Err("GLUT payload must collapse into one enclosing frame".into());
        }
        Ok(Self { nested, collapsed, resident: Rc::new(RefCell::new(GlutSieve::new(n))) })
    }

    pub fn nested_word(&self) -> &[char] { &self.nested }
    pub fn membrane_word(&self) -> &[char] { &self.collapsed }

    pub fn execute(self) -> Result<Option<GlutExecution>, String> {
        let word: String = self.collapsed.iter().collect();
        // Deposits bank references to the same resident GLUT object. Neither
        // nesting nor dissolution duplicates its source, frontier or kernel.
        let mut executed = false;
        let transported = crate::phase_word::execute_with(&word, &self.resident, |mark, resident| {
            if mark == '⊞' && !executed {
                resident.borrow_mut().frame_sweep();
                executed = true;
            }
            Ok(())
        })?;
        if !executed { return Err("collapsed carrier did not engage its GLUT payload".into()); }
        if transported.exposed != 0 || transported.surviving.is_empty() {
            return Err("collapsed carrier did not retain its GLUT payload".into());
        }
        let selected = &transported.surviving[0].observation;
        if transported.surviving.iter().any(|d| !Rc::ptr_eq(selected, &d.observation)) {
            return Err("collapsed carrier changed its resident GLUT identity".into());
        }
        // GLUT executed inside the enclosing frame, before fuse and fixation.
        // Readout now comes from that same returned resident payload.
        let execution = selected.borrow().readout_execution();
        Ok(execution)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn random_2048_source_is_resident_in_the_users_collapsed_carrier() {
        let n = crate::morphism_factor::parse_numeral(include_str!(
            "../tests/fixtures/random_rsa_2048.imasm"
        ).trim()).unwrap();
        let nested = "⊢∈∈⊞∈⊙∈≻∈≺∈⋈∈⊤∈⊥∋∋∋∋∋∋∋∋⊡⊣";
        let prepared = PreparedGlutMembrane::prepare(&n, nested).unwrap();
        assert_eq!(prepared.nested_word().iter().collect::<String>(), nested);
        assert_eq!(prepared.membrane_word().iter().collect::<String>(), "⊢∈⊞⊙≻≺⋈⊤⊥∋⊡⊣");
        let mut events = Vec::new();
        let transported = crate::phase_word::execute_with("⊢∈⊞⊙≻≺⋈⊤⊥∋⊡⊣", &prepared.resident, |mark, resident| {
            assert!(Rc::ptr_eq(resident, &prepared.resident));
            events.push(mark);
            Ok(())
        }).unwrap();
        let engage = events.iter().position(|&mark| mark == '⊞').unwrap();
        let fuse = events.iter().position(|&mark| mark == '∋').unwrap();
        assert!(events[..engage].contains(&'∈') && engage < fuse);
        assert_eq!(transported.exposed, 0);
        assert!(!transported.surviving.is_empty());
        for deposit in transported.surviving {
            assert!(Rc::ptr_eq(&deposit.observation, &prepared.resident));
            assert_eq!(deposit.observation.borrow().n_tape, n);
        }
    }
}
